#pragma once

#include <cstddef>
#include <string_view>

#include "contracts/translation.hpp"
#include "core/translation/ports/translation_gateway.hpp"

namespace cliplingo {

// K4 (R5). Owns the language allowlist, text-shape classification and context
// policy so that K3 and the UI never see vendor JSON or HTTP status codes.
class TranslationService {
 public:
  explicit TranslationService(TranslationGateway& gateway);

  // Never blocks. Validation failures are delivered through the sink, and in
  // that case the gateway is not called at all (X5, X6).
  void Translate(const TranslateRequest& request, TranslationSink sink);

  // A 300-500 word paragraph must render as a paragraph (DESIGN.md popup type).
  static constexpr std::size_t kParagraphTokenThreshold = 40;

 private:
  static bool IsSupportedLanguage(std::string_view language);
  static TextShape Classify(std::wstring_view text);

  TranslationGateway& gateway_;
};

}  // namespace cliplingo
