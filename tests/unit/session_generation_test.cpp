#include <cstdint>
#include <string>
#include <utility>

#include "core/preferences/request_policy.hpp"
#include "core/session/session_controller.hpp"
#include "core/translation/translation_service.hpp"
#include "fixtures/fakes.hpp"
#include "support/test_check.hpp"

namespace {

using namespace cliplingo;

struct Harness {
  cl_test::FakeCaptureClient capture;
  cl_test::FakeTranslationGateway gateway;
  cl_test::RecordingPresenter presenter;
  TranslationService translation{gateway};
  SessionController session{capture, translation, presenter, RequestPolicy{}};
  Preferences prefs;

  Harness() { prefs.consent = OnlineConsent::Granted; }

  void Trigger(std::wstring source_exe = L"notepad.exe") {
    SourceAnchor anchor;
    anchor.source_hwnd = 1;
    anchor.cursor = Point{100, 200};
    session.BeginTrigger(anchor, prefs, std::move(source_exe));
  }

  // Successful capture for the current generation; returns the capture sink so
  // a test can replay it after the request went stale.
  CaptureSink CaptureOk(std::wstring text = L"hello world") {
    CaptureSink stale = capture.pending;
    CaptureResult result;
    result.generation = session.generation();
    result.status = CaptureStatus::Ok;
    result.text = std::move(text);
    capture.Deliver(result);
    return stale;
  }

  TranslationSink TranslateInFlight() {
    TranslationSink stale = gateway.pending;
    return stale;
  }
};

}  // namespace

CL_TEST(late_capture_of_old_generation_is_dropped) {
  Harness harness;
  harness.Trigger();
  const std::uint64_t first_generation = harness.session.generation();
  const CaptureSink first_sink = harness.capture.pending;

  harness.Trigger();
  const std::uint64_t second_generation = harness.session.generation();

  // The old capture resolves after the new trigger (X9).
  CaptureResult late;
  late.generation = first_generation;
  late.status = CaptureStatus::Ok;
  late.text = L"stale selection";
  first_sink(late);

  CL_CHECK(first_generation != second_generation);
  CL_CHECK(!harness.presenter.Saw(PopupPhase::Translating, first_generation));
  CL_CHECK(!harness.presenter.Saw(PopupPhase::Success, first_generation));
  CL_CHECK_EQ(harness.gateway.calls, 0);
  CL_CHECK_EQ(harness.presenter.Last().generation, second_generation);
}

CL_TEST(late_translation_of_old_generation_is_dropped) {
  Harness harness;
  harness.Trigger();
  const std::uint64_t first_generation = harness.session.generation();
  harness.CaptureOk(L"hello world");
  const TranslationSink first_sink = harness.TranslateInFlight();
  CL_CHECK_EQ(harness.gateway.calls, 1);

  harness.Trigger();
  const std::uint64_t second_generation = harness.session.generation();

  TranslateResult late;
  late.generation = first_generation;
  late.translated = L"halo dunia";
  first_sink(late);

  CL_CHECK(!harness.presenter.Saw(PopupPhase::Success, first_generation));
  CL_CHECK_EQ(harness.presenter.Last().generation, second_generation);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Capturing);
}

CL_TEST(current_generation_publishes_success) {
  Harness harness;
  harness.Trigger();
  harness.CaptureOk(L"hello world");

  TranslateResult answer;
  answer.generation = harness.session.generation();
  answer.translated = L"halo dunia";
  answer.provider = L"fake";
  harness.gateway.Deliver(answer);

  const auto& last = harness.presenter.Last();
  CL_CHECK(last.phase == PopupPhase::Success);
  CL_CHECK_EQ(last.generation, harness.session.generation());
  CL_CHECK(last.result.has_value());
  CL_CHECK(last.result.has_value() && last.result->translated == std::wstring(L"halo dunia"));
  CL_CHECK(last.source_text.has_value());
  CL_CHECK(!last.error.has_value());
}

CL_TEST(dismiss_ignores_late_result) {
  Harness harness;
  harness.Trigger();
  const std::uint64_t generation = harness.session.generation();
  harness.CaptureOk();
  const TranslationSink stale = harness.TranslateInFlight();

  harness.session.OnDismiss();

  TranslateResult late;
  late.generation = generation;
  late.translated = L"halo dunia";
  stale(late);

  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Hidden);
  CL_CHECK(!harness.presenter.Saw(PopupPhase::Success, generation));
}

CL_TEST(consent_revocation_drops_inflight_success) {
  Harness harness;
  harness.Trigger();
  const std::uint64_t first_generation = harness.session.generation();
  harness.CaptureOk();
  const TranslationSink stale = harness.TranslateInFlight();

  harness.session.OnConsentChanged(OnlineConsent::Denied);

  TranslateResult late;
  late.generation = first_generation;
  late.translated = L"halo dunia";
  stale(late);

  CL_CHECK(!harness.presenter.Saw(PopupPhase::Success, first_generation));
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::ConsentRequired);
  CL_CHECK(harness.presenter.Last().error.has_value());
  CL_CHECK(harness.presenter.Last().error.has_value() &&
           harness.presenter.Last().error.value() == TranslationError::ConsentRequired);
}

CL_TEST(empty_capture_maps_to_actionable_failure) {
  Harness harness;
  harness.Trigger();

  CaptureResult empty;
  empty.generation = harness.session.generation();
  empty.status = CaptureStatus::Empty;
  harness.capture.Deliver(empty);

  const auto& last = harness.presenter.Last();
  CL_CHECK(last.phase == PopupPhase::Failure);
  CL_CHECK(last.capture_status.has_value());
  CL_CHECK(last.capture_status.has_value() &&
           last.capture_status.value() == CaptureStatus::Empty);
  CL_CHECK_EQ(harness.gateway.calls, 0);
}

CL_TEST(excluded_app_skips_capture_and_translation) {
  Harness harness;
  harness.prefs.excluded_apps = {L"chrome.exe"};
  harness.Trigger(L"C:\\Program Files\\Google\\Chrome\\Application\\CHROME.EXE");

  const auto& last = harness.presenter.Last();
  CL_CHECK_EQ(harness.capture.calls, 0);
  CL_CHECK_EQ(harness.gateway.calls, 0);
  CL_CHECK(last.phase == PopupPhase::Failure);
  CL_CHECK(last.capture_status.has_value() &&
           last.capture_status.value() == CaptureStatus::Excluded);
}

CL_TEST(unset_consent_blocks_outbound_after_capture) {
  Harness harness;
  harness.prefs.consent = OnlineConsent::Unset;
  harness.Trigger();
  harness.CaptureOk(L"hello world");

  const auto& last = harness.presenter.Last();
  CL_CHECK_EQ(harness.capture.calls, 1);  // capture is local: allowed
  CL_CHECK_EQ(harness.gateway.calls, 0);  // egress: denied (X10, G7)
  CL_CHECK(last.phase == PopupPhase::ConsentRequired);
  CL_CHECK(last.error.has_value() && last.error.value() == TranslationError::ConsentRequired);
}

CL_TEST(retrigger_cancels_previous_generation_best_effort) {
  Harness harness;
  harness.Trigger();
  const std::uint64_t first_generation = harness.session.generation();
  harness.Trigger();

  CL_CHECK_EQ(harness.capture.cancels, 1);
  CL_CHECK_EQ(harness.capture.last_cancelled, first_generation);
  CL_CHECK(harness.session.generation() == first_generation + 1);
}
