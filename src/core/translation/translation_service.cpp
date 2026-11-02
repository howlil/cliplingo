#include "core/translation/translation_service.hpp"

#include <algorithm>
#include <array>
#include <cctype>
#include <utility>

namespace cliplingo {
namespace {

constexpr std::array<std::string_view, 2> kSupportedLanguages{"en", "id"};

bool EqualsIgnoreCaseAscii(std::string_view left, std::string_view right) {
  if (left.size() != right.size()) {
    return false;
  }
  return std::equal(left.begin(), left.end(), right.begin(), [](char lhs, char rhs) {
    return std::tolower(static_cast<unsigned char>(lhs)) ==
           std::tolower(static_cast<unsigned char>(rhs));
  });
}

bool IsSpace(wchar_t value) {
  return value == L' ' || value == L'\t' || value == L'\r' || value == L'\n';
}

}  // namespace

TranslationService::TranslationService(TranslationGateway& gateway) : gateway_(gateway) {}

void TranslationService::Translate(const TranslateRequest& request, TranslationSink sink) {
  if (!sink) {
    return;
  }

  TranslateResult result;
  result.generation = request.generation;
  result.shape = request.shape;

  const bool same_language = EqualsIgnoreCaseAscii(request.source_lang, request.target_lang);
  if (!IsSupportedLanguage(request.source_lang) || !IsSupportedLanguage(request.target_lang) ||
      same_language) {
    // No provider call: an unsupported pair must never become an egress (X5).
    result.error = TranslationError::UnsupportedLanguage;
    sink(std::move(result));
    return;
  }

  TranslateRequest normalized = request;
  normalized.shape = Classify(request.text);
  const TextShape shape = normalized.shape;

  gateway_.Translate(normalized, [sink = std::move(sink), shape](TranslateResult translated) mutable {
    // The adapter reports the vendor answer; the shape is decided here.
    translated.shape = shape;
    sink(std::move(translated));
  });
}

bool TranslationService::IsSupportedLanguage(std::string_view language) {
  return std::any_of(kSupportedLanguages.begin(), kSupportedLanguages.end(),
                     [language](std::string_view supported) {
                       return EqualsIgnoreCaseAscii(language, supported);
                     });
}

TextShape TranslationService::Classify(std::wstring_view text) {
  std::size_t tokens = 0;
  std::size_t newlines = 0;
  bool inside_token = false;

  for (const wchar_t value : text) {
    if (value == L'\n') {
      ++newlines;
    }
    if (IsSpace(value)) {
      inside_token = false;
      continue;
    }
    if (!inside_token) {
      inside_token = true;
      ++tokens;
    }
  }

  if (tokens > kParagraphTokenThreshold || newlines >= 2) {
    return TextShape::Paragraph;
  }
  if (tokens <= 1) {
    return TextShape::Word;
  }
  return TextShape::Sentence;
}

}  // namespace cliplingo
