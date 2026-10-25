#include "core.h"
#include <ctranslate2/translator.h>
#include <sentencepiece_processor.h>
#include <vector>
#include <string>
#include <filesystem>
#include <algorithm>
#include <memory>
#include <stdexcept>

namespace {
std::string utf8(const std::wstring& value) {
  if (value.empty()) return {};
  const int n = WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, value.data(),
                                    static_cast<int>(value.size()), nullptr, 0, nullptr, nullptr);
  if (n == 0) throw std::runtime_error("invalid UTF-16");
  std::string out(static_cast<size_t>(n), '\0');
  if (!WideCharToMultiByte(CP_UTF8, WC_ERR_INVALID_CHARS, value.data(),
                           static_cast<int>(value.size()), out.data(), n, nullptr, nullptr))
    throw std::runtime_error("UTF-16 to UTF-8 failed");
  return out;
}
std::wstring wide(const std::string& value) {
  if (value.empty()) return {};
  const int n = MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, value.data(),
                                   static_cast<int>(value.size()), nullptr, 0);
  if (n == 0) throw std::runtime_error("invalid UTF-8");
  std::wstring out(static_cast<size_t>(n), L'\0');
  if (!MultiByteToWideChar(CP_UTF8, MB_ERR_INVALID_CHARS, value.data(),
                           static_cast<int>(value.size()), out.data(), n))
    throw std::runtime_error("UTF-8 to UTF-16 failed");
  return out;
}
std::string path_utf8(const std::filesystem::path& path) {
  const auto bytes = path.u8string();
  return {reinterpret_cast<const char*>(bytes.data()), bytes.size()};
}
struct Session {
  std::filesystem::path path;
  sentencepiece::SentencePieceProcessor source;
  sentencepiece::SentencePieceProcessor target;
  std::unique_ptr<ctranslate2::Translator> translator;
};
thread_local std::unique_ptr<Session> session;
}

void unload_translation_model() { session.reset(); }

TranslationResult translate_en_id(const std::wstring& selected, const std::filesystem::path& model_dir) {
  if (selected.empty()) return {Failure::NoSelection, {}};
  if (model_dir.empty()) return {Failure::ModelMissing, {}};
  try {
    if (!std::filesystem::is_regular_file(model_dir / L"model.bin") ||
        !std::filesystem::is_regular_file(model_dir / L"source.spm") ||
        !std::filesystem::is_regular_file(model_dir / L"target.spm"))
      return {Failure::ModelMissing, {}};
    if (!session || session->path != model_dir) {
      if (!verify_model_pack(model_dir)) return {Failure::ModelInvalid, {}};
      // Make a new complete session before switching. No half-initialized model published.
      auto next = std::make_unique<Session>();
      next->path = model_dir;
      if (!next->source.Load(path_utf8(model_dir / L"source.spm")).ok() ||
          !next->target.Load(path_utf8(model_dir / L"target.spm")).ok())
        return {Failure::ModelInvalid, {}};
      ctranslate2::models::ModelLoader loader(path_utf8(model_dir));
      next->translator = std::make_unique<ctranslate2::Translator>(loader);
      session = std::move(next);
    }
    std::vector<std::string> source_tokens;
    if (!session->source.Encode(utf8(selected), &source_tokens).ok() || source_tokens.empty())
      return {Failure::ModelInvalid, {}};
    source_tokens.emplace_back("</s>");
    const auto results = session->translator->translate_batch({source_tokens});
    if (results.empty()) return {Failure::TranslationFailed, {}};
    std::vector<std::string> output = results[0].output();
    output.erase(std::remove(output.begin(), output.end(), "</s>"), output.end());
    output.erase(std::remove(output.begin(), output.end(), "<pad>"), output.end());
    std::string decoded;
    if (!session->target.Decode(output, &decoded).ok() || decoded.empty())
      return {Failure::TranslationFailed, {}};
    return {Failure::None, wide(decoded)};
  } catch (const std::filesystem::filesystem_error&) {
    return {Failure::ModelInvalid, {}};
  } catch (const std::exception&) {
    // Inference/session may be poisoned by an exception: invalidate warm cache.
    session.reset();
    return {Failure::TranslationFailed, {}};
  }
}

std::wstring failure_label(Failure failure) {
  switch (failure) {
    case Failure::None: return L"Ready";
    case Failure::NoSelection: return L"No selected text";
    case Failure::Unsupported: return L"Selection unsupported by this application";
    case Failure::TooLong: return L"Selection exceeds M1's 8192-character limit";
    case Failure::CaptureFailed: return L"Text capture failed";
    case Failure::ModelMissing: return L"EN-ID model is not installed. Choose a converted model folder in Settings.";
    case Failure::ModelInvalid: return L"Model/tokenizer files are invalid";
    case Failure::TranslationFailed: return L"Offline translation failed";
    case Failure::ClipboardUnavailable: return L"Clipboard unavailable, busy or Unicode text missing";
    case Failure::ClipboardChanged: return L"Clipboard content changed before it could be read";
  }
  return L"Unknown failure";
}