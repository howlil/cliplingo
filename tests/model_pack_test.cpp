#include "core.h"
#include <windows.h>
#include <filesystem>
#include <fstream>
#include <iostream>
#include <string>

namespace fs = std::filesystem;
void put(const fs::path& path, const std::string& contents) {
  std::ofstream file(path, std::ios::binary);
  file << contents;
}
int main() {
  wchar_t tmp[MAX_PATH]{};
  if (!GetTempPathW(MAX_PATH, tmp)) return 1;
  auto dir = fs::path(tmp) / (L"cliplingo-test-" + std::to_wstring(GetCurrentProcessId()));
  fs::create_directories(dir);
  struct Cleanup { fs::path p; ~Cleanup() { std::error_code e; fs::remove_all(p, e); } } cleanup{dir};
  put(dir / L"model.bin", "model");
  put(dir / L"config.json", "config");
  put(dir / L"source.spm", "source");
  put(dir / L"target.spm", "target");
  put(dir / L"cliplingo-model.sha256",
    "ClipLingoModelPackV1 en-id\n"
    "b79606fb3afea5bd1609ed40b622142f1c98125abcfe89a76a661b0e8e343910  config.json\n"
    "9372c470eeadd5ecd9c3c74c2b3cb633f8e2f2fad799250a0f70d652b6b825e4  model.bin\n"
    "41cf6794ba4200b839c53531555f0f3998df4cbb01a4d5cb0b94e3ca5e23947d  source.spm\n"
    "34a04005bcaf206eec990bd9637d9fdb6725e0a0c0d4aebf003f17f4c956eb5c  target.spm\n");
  if (!verify_model_pack(dir)) { std::cerr << "valid pack rejected\n"; return 1; }
  put(dir / L"target.spm", "modified");
  if (verify_model_pack(dir)) { std::cerr << "modified pack accepted\n"; return 2; }
  put(dir / L"target.spm", "target");
  put(dir / L"unexpected.dll", "unexpected");
  if (verify_model_pack(dir)) { std::cerr << "unsigned extra file accepted\n"; return 3; }
  return 0;
}