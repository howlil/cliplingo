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

// RV2 (X1): a whitespace-only selection is "no text", not an internal fault.
CL_TEST(capture_ok_with_whitespace_only_is_reported_as_empty) {
  Harness harness;
  harness.Trigger();
  harness.CaptureOk(L"   \t\n  ");

  const auto& last = harness.presenter.Last();
  CL_CHECK(last.phase == PopupPhase::Failure);
  CL_CHECK(last.capture_status.has_value() &&
           last.capture_status.value() == CaptureStatus::Empty);
  CL_CHECK(!last.error.has_value());
  CL_CHECK_EQ(harness.gateway.calls, 0);
}

// RV6: dismiss then trigger must reopen the popup, not stay hidden.
CL_TEST(retrigger_after_dismiss_restarts_the_popup) {
  Harness harness;
  harness.Trigger();
  harness.CaptureOk();
  harness.session.OnDismiss();
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Hidden);
  const std::uint64_t dismissed_generation = harness.presenter.Last().generation;

  harness.Trigger();
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Capturing);
  CL_CHECK(harness.session.generation() == dismissed_generation + 1);

  harness.CaptureOk(L"second selection");
  TranslateResult answer;
  answer.generation = harness.session.generation();
  answer.translated = L"pilihan kedua";
  harness.gateway.Deliver(answer);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Success);
}

// RV6: an adapter may resolve inside Capture()/Translate(); the state machine
// must reach the same final state as the asynchronous path.
CL_TEST(synchronous_sink_delivery_reaches_success) {
  Harness harness;
  CaptureResult captured;
  captured.status = CaptureStatus::Ok;
  captured.text = L"hello world";
  harness.capture.synchronous_result = captured;
  TranslateResult answer;
  answer.translated = L"halo dunia";
  harness.gateway.synchronous_result = answer;

  harness.Trigger();

  CL_CHECK_EQ(harness.capture.calls, 1);
  CL_CHECK_EQ(harness.gateway.calls, 1);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Success);
  CL_CHECK_EQ(harness.presenter.Last().generation, harness.session.generation());
}

// RV4 (flow 3 / K8->K3): consent given resumes the current request without a
// new trigger, but only after the exclusion policy is re-checked.
CL_TEST(granted_consent_resumes_the_blocked_request) {
  Harness harness;
  harness.prefs.consent = OnlineConsent::Unset;
  harness.Trigger();
  const std::uint64_t generation = harness.session.generation();
  harness.CaptureOk(L"hello world");
  CL_CHECK_EQ(harness.gateway.calls, 0);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::ConsentRequired);

  harness.session.OnConsentChanged(OnlineConsent::Granted);

  CL_CHECK_EQ(harness.session.generation(), generation);  // resume, not re-trigger
  CL_CHECK_EQ(harness.gateway.calls, 1);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Translating);

  TranslateResult answer;
  answer.generation = generation;
  answer.translated = L"halo dunia";
  harness.gateway.Deliver(answer);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Success);
}

CL_TEST(granted_consent_does_not_send_from_an_excluded_app) {
  Harness harness;
  harness.prefs.consent = OnlineConsent::Unset;
  harness.prefs.excluded_apps = {L"notepad.exe"};
  harness.Trigger(L"notepad.exe");

  harness.session.OnConsentChanged(OnlineConsent::Granted);

  CL_CHECK_EQ(harness.gateway.calls, 0);
  CL_CHECK_EQ(harness.capture.calls, 0);
}

// RV1 (K8->K3): revocation invalidates in-flight work, but a rendered result
// was never transmitted and must stay readable.
CL_TEST(revocation_keeps_the_rendered_result_readable) {
  Harness harness;
  harness.Trigger();
  harness.CaptureOk(L"hello world");
  TranslateResult answer;
  answer.generation = harness.session.generation();
  answer.translated = L"halo dunia";
  harness.gateway.Deliver(answer);
  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Success);

  harness.session.OnConsentChanged(OnlineConsent::Denied);

  CL_CHECK(harness.presenter.Last().phase == PopupPhase::Success);
  CL_CHECK(harness.presenter.Last().result.has_value());
  CL_CHECK(!harness.presenter.Saw(PopupPhase::ConsentRequired,
                                  harness.presenter.Last().generation));
}

// RV7 (LF/D13): a sink that fires after destruction must be a no-op, not UB.
CL_TEST(a_result_after_controller_destruction_is_ignored) {
  cl_test::FakeCaptureClient capture;
  cl_test::FakeTranslationGateway gateway;
  cl_test::RecordingPresenter presenter;
  TranslationService translation{gateway};

  CaptureSink late;
  {
    SessionController session{capture, translation, presenter, RequestPolicy{}};
    Preferences prefs;
    prefs.consent = OnlineConsent::Granted;
    SourceAnchor anchor;
    anchor.source_hwnd = 1;
    session.BeginTrigger(anchor, prefs, L"notepad.exe");
    late = capture.pending;
  }

  const std::size_t published_before = presenter.states.size();
  CaptureResult result;
  result.generation = 1;
  result.status = CaptureStatus::Ok;
  result.text = L"hello world";
  late(result);

  CL_CHECK_EQ(presenter.states.size(), published_before);
  CL_CHECK_EQ(gateway.calls, 0);
}
