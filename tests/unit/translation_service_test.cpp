#include <cstdint>
#include <optional>
#include <string>
#include <utility>

#include "core/translation/translation_service.hpp"
#include "fixtures/fakes.hpp"
#include "support/test_check.hpp"

namespace {

using namespace cliplingo;

struct Harness {
  cl_test::FakeTranslationGateway gateway;
  TranslationService service{gateway};
  std::optional<TranslateResult> delivered;

  void Call(const TranslateRequest& request) {
    delivered.reset();
    service.Translate(request, [this](TranslateResult result) {
      delivered = std::move(result);
    });
  }
};

TranslateRequest Request(std::wstring text, std::string source = "en", std::string target = "id") {
  TranslateRequest request;
  request.generation = 7;
  request.text = std::move(text);
  request.source_lang = std::move(source);
  request.target_lang = std::move(target);
  return request;
}

std::wstring RepeatWords(int count) {
  std::wstring text;
  for (int index = 0; index < count; ++index) {
    if (index != 0) {
      text.push_back(L' ');
    }
    text += L"word";
  }
  return text;
}

}  // namespace

CL_TEST(unsupported_pair_never_reaches_the_provider) {
  Harness harness;
  harness.Call(Request(L"hello", "en", "fr"));

  CL_CHECK_EQ(harness.gateway.calls, 0);
  CL_CHECK(harness.delivered.has_value() &&
           harness.delivered->error == TranslationError::UnsupportedLanguage);
  CL_CHECK_EQ(harness.delivered->generation, std::uint64_t{7});
}

CL_TEST(same_language_pair_is_rejected_without_a_call) {
  Harness harness;
  harness.Call(Request(L"hello", "EN", "en"));

  CL_CHECK_EQ(harness.gateway.calls, 0);
  CL_CHECK(harness.delivered.has_value() &&
           harness.delivered->error == TranslationError::UnsupportedLanguage);
}

CL_TEST(provider_error_is_passed_through_typed) {
  Harness harness;
  harness.Call(Request(L"hello"));

  TranslateResult failure;
  failure.generation = 7;
  failure.error = TranslationError::RateLimited;
  harness.gateway.Deliver(failure);

  CL_CHECK(harness.delivered.has_value());
  CL_CHECK(harness.delivered.has_value() &&
           harness.delivered->error == TranslationError::RateLimited);
  CL_CHECK(harness.delivered.has_value() && !harness.delivered->ok());
  CL_CHECK(harness.delivered.has_value() && harness.delivered->generation == std::uint64_t{7});
}

CL_TEST(shape_is_classified_before_the_provider_call) {
  Harness harness;

  harness.Call(Request(L"hello"));
  CL_CHECK(harness.gateway.last_request->shape == TextShape::Word);

  harness.Call(Request(L"one two three four five six seven eight"));
  CL_CHECK(harness.gateway.last_request->shape == TextShape::Sentence);

  harness.Call(Request(RepeatWords(60)));
  CL_CHECK(harness.gateway.last_request->shape == TextShape::Paragraph);

  harness.Call(Request(L"line one\nline two\nline three"));
  CL_CHECK(harness.gateway.last_request->shape == TextShape::Paragraph);
}

CL_TEST(service_owns_the_returned_shape_and_unknown_means_no_egress) {
  Harness harness;
  harness.Call(Request(RepeatWords(60)));

  TranslateResult answer;
  answer.generation = 7;
  answer.translated = L"jawaban";
  answer.shape = TextShape::Word;  // provider does not decide typography
  harness.gateway.Deliver(answer);

  CL_CHECK(harness.delivered.has_value() && harness.delivered->shape == TextShape::Paragraph);
  CL_CHECK(harness.delivered.has_value() && harness.delivered->ok());
  CL_CHECK(harness.delivered.has_value() && harness.delivered->context_used == false);
}

CL_TEST(context_is_forwarded_only_when_present) {
  Harness harness;
  TranslateRequest request = Request(L"hello");
  harness.Call(request);
  CL_CHECK(!harness.gateway.last_request->context.has_value());

  request.context = L"surrounding sentence";
  harness.Call(request);
  CL_CHECK(harness.gateway.last_request->context.has_value());
  CL_CHECK(harness.gateway.last_request->context.value() == std::wstring(L"surrounding sentence"));
}
