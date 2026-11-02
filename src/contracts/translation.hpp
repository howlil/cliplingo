#pragma once

#include <cstdint>
#include <functional>
#include <optional>
#include <string>

namespace cliplingo {

// Drives popup typography (DESIGN.md: word/sentence large, paragraph smaller).
enum class TextShape {
  Word,
  Sentence,
  Paragraph,
};

// Vendor-independent failure kinds (X5, X10). Adapters map HTTP/vendor errors
// into these; callers never see vendor JSON or HTTP status codes.
enum class TranslationError {
  None,
  ConsentRequired,
  Excluded,
  Offline,
  Timeout,
  RateLimited,
  Unauthorized,
  UnsupportedLanguage,
  ProviderRejected,
  Internal,
};

struct TranslateRequest {
  std::uint64_t generation = 0;
  std::wstring text;
  std::string source_lang;
  std::string target_lang;
  std::optional<std::wstring> context;
  TextShape shape = TextShape::Sentence;
};

struct TranslateResult {
  std::uint64_t generation = 0;
  TranslationError error = TranslationError::None;
  std::wstring translated;
  std::wstring provider;
  std::string detected_lang;
  TextShape shape = TextShape::Sentence;
  bool context_used = false;

  [[nodiscard]] bool ok() const noexcept { return error == TranslationError::None; }
};

// Same sink discipline as CaptureSink: invoked on the UI thread, possibly after
// the request became stale.
using TranslationSink = std::function<void(TranslateResult)>;

}  // namespace cliplingo
