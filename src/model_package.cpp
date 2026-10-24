#include "core.h"
#include <windows.h>
#include <bcrypt.h>
#include <algorithm>
#include <array>
#include <filesystem>
#include <fstream>
#include <set>
#include <string>
#include <vector>
#pragma comment(lib, "bcrypt.lib")

namespace {
namespace fs = std::filesystem;
constexpr std::uintmax_t kMaxPackSize = 1000ULL * 1024 * 1024;
struct Algorithm {
  BCRYPT_ALG_HANDLE handle{};
  ~Algorithm() { if (handle) BCryptCloseAlgorithmProvider(handle, 0); }
};
struct Hash {
  BCRYPT_HASH_HANDLE handle{};
  ~Hash() { if (handle) BCryptDestroyHash(handle); }
};
std::string sha256(const fs::path& file) {
  Algorithm alg;
  if (BCryptOpenAlgorithmProvider(&alg.handle, BCRYPT_SHA256_ALGORITHM, nullptr, 0) < 0)
    return {};
  Hash hash;
  if (BCryptCreateHash(alg.handle, &hash.handle, nullptr, 0, nullptr, 0, 0) < 0)
    return {};
  std::ifstream in(file, std::ios::binary);
  if (!in) return {};
  std::array<char, 65536> chunk{};
  while (in) {
    in.read(chunk.data(), chunk.size());
    const auto n = in.gcount();
    if (n > 0 && BCryptHashData(hash.handle, reinterpret_cast<PUCHAR>(chunk.data()),
                                 static_cast<ULONG>(n), 0) < 0) return {};
  }
  if (!in.eof()) return {};
  std::array<unsigned char, 32> bytes{};
  if (BCryptFinishHash(hash.handle, bytes.data(), static_cast<ULONG>(bytes.size()), 0) < 0)
    return {};
  constexpr char digits[] = "0123456789abcdef";
  std::string out;
  out.reserve(64);
  for (const auto byte : bytes) { out += digits[byte >> 4]; out += digits[byte & 15]; }
  return out;
}
std::wstring model_root_path() {
  wchar_t path[32768]{};
  const auto n = GetEnvironmentVariableW(L"LOCALAPPDATA", path, 32768);
  if (n == 0 || n >= 32768) return {};
  return (fs::path(path) / L"ClipLingo" / L"models").wstring();
}
bool safe_filename(const std::string& filename) {
  if (filename.empty() || filename.size() > 160 || filename == "." || filename == "..") return false;
  if (filename.find('/') != std::string::npos || filename.find('\\') != std::string::npos || filename.find(':') != std::string::npos) return false;
  return std::all_of(filename.begin(), filename.end(), [](unsigned char c) {
    return (c >= 'a' && c <= 'z') || (c >= 'A' && c <= 'Z') ||
      (c >= '0' && c <= '9') || c == '_' || c == '-' || c == '.';
  });
}
}

bool verify_model_pack(const std::filesystem::path& dir) {
  try {
    if (!fs::is_directory(dir) || fs::is_symlink(fs::symlink_status(dir))) return false;
    std::ifstream manifest(dir / L"cliplingo-model.sha256", std::ios::binary);
    std::string heading;
    if (!manifest || !std::getline(manifest, heading) || heading != "ClipLingoModelPackV1 en-id") return false;
    std::set<std::string> seen;
    std::uintmax_t total = 0;
    std::string entry;
    while (std::getline(manifest, entry)) {
      if (!entry.empty() && entry.back() == '\r') entry.pop_back();
      if (entry.size() < 67 || entry.substr(64, 2) != "  ") return false;
      const std::string expected = entry.substr(0, 64);
      const std::string name = entry.substr(66);
      if (!safe_filename(name) || !seen.insert(name).second || seen.size() > 32 ||
          !std::all_of(expected.begin(), expected.end(), [](unsigned char c) { return (c >= '0' && c <= '9') || (c >= 'a' && c <= 'f'); }))
        return false;
      const auto file = dir / fs::path(name);
      if (!fs::is_regular_file(fs::symlink_status(file))) return false;
      total += fs::file_size(file);
      if (total > kMaxPackSize || sha256(file) != expected) return false;
    }
    if (!manifest.eof() || !seen.contains("model.bin") || !seen.contains("config.json") ||
        !seen.contains("source.spm") || !seen.contains("target.spm")) return false;
    // No unsigned, unexamined model auxiliary files and no subdirectories.
    for (const auto& entry : fs::directory_iterator(dir)) {
      if (!fs::is_regular_file(fs::symlink_status(entry.path()))) return false;
      const auto filename = entry.path().filename().string();
      if (filename != "cliplingo-model.sha256" && !seen.contains(filename)) return false;
    }
    return true;
  } catch (const fs::filesystem_error&) { return false; }
}

ModelInstallResult install_model_pack(const std::filesystem::path& source) {
  if (!verify_model_pack(source)) return {false, {}, L"Invalid or incomplete model pack (manifest/hash mismatch)."};
  try {
    const auto root_text = model_root_path();
    if (root_text.empty()) return {false, {}, L"LOCALAPPDATA is unavailable."};
    const fs::path root(root_text);
    fs::create_directories(root);
    const auto hash = sha256(source / L"cliplingo-model.sha256");
    if (hash.size() != 64) return {false, {}, L"Could not verify the model manifest."};
    const fs::path final_path = root / (L"en-id-" + std::wstring(hash.begin(), hash.begin() + 16));
    if (fs::exists(final_path)) {
      if (verify_model_pack(final_path)) return {true, final_path, L"Model already installed."};
      return {false, {}, L"An existing installation at this version is corrupt."};
    }
    const fs::path staging = root / (L".staging-" + std::to_wstring(GetCurrentProcessId()) +
                                      L"-" + std::to_wstring(GetTickCount64()));
    struct Rollback {
      fs::path path;
      ~Rollback() { if (!path.empty()) { std::error_code ignored; fs::remove_all(path, ignored); } }
    } rollback{staging};
    fs::create_directory(staging);
    for (const auto& file : fs::directory_iterator(source))
      fs::copy_file(file.path(), staging / file.path().filename(), fs::copy_options::none);
    if (!verify_model_pack(staging)) return {false, {}, L"Staged model verification failed."};
    fs::rename(staging, final_path);
    rollback.path.clear();
    return {true, final_path, L"Model installed and verified."};
  } catch (const fs::filesystem_error&) { return {false, {}, L"Model installation failed. Check disk space and permissions."}; }
}