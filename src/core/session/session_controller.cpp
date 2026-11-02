#include "core/session/session_controller.hpp"

#include <utility>

namespace cliplingo {
namespace {

TranslationError ErrorForVerdict(PolicyVerdict verdict) {
  switch (verdict) {
    case PolicyVerdict::DeniedExcluded:
      return TranslationError::Excluded;
    case PolicyVerdict::DeniedConsent:
      return TranslationError::ConsentRequired;
    case PolicyVerdict::DeniedEmptyText:
      return TranslationError::Internal;
    case PolicyVerdict::Allow:
      break;
  }
  return TranslationError::Internal;
}

}  // namespace

SessionController::SessionController(CaptureClient& capture, TranslationService& translation,
                                     PopupPresenter& presenter, RequestPolicy policy)
    : capture_(capture), translation_(translation), presenter_(presenter), policy_(policy) {}

void SessionController::BeginTrigger(SourceAnchor anchor, const Preferences& prefs,
                                     std::wstring source_exe) {
  const std::uint64_t previous = generation_;
  ++generation_;

  visible_ = true;
  anchor_ = anchor.cursor;
  prefs_ = prefs;
  source_exe_ = std::move(source_exe);
  shape_ = TextShape::Sentence;
  source_text_.reset();
  capture_status_.reset();
  error_.reset();
  result_.reset();

  if (previous != 0) {
    capture_.Cancel(previous);  // best effort only (DEC6)
  }

  // Local stage: exclusions apply, consent does not (X10 gates transmission).
  const PolicyVerdict verdict = policy_.EvaluateCapture(prefs_, source_exe_);
  if (verdict != PolicyVerdict::Allow) {
    FailCapture(verdict == PolicyVerdict::DeniedExcluded ? CaptureStatus::Excluded
                                                        : CaptureStatus::Internal);
    return;
  }

  // Visible before any capture work: the popup must not wait on UIA (G1, B3).
  Publish(PopupPhase::Capturing);

  CaptureRequest request;
  request.generation = generation_;
  request.anchor = anchor;
  request.source_exe = source_exe_;
  request.excluded_apps = prefs_.excluded_apps;
  capture_.Capture(request, [this](CaptureResult capture_result) {
    OnCaptureResult(capture_result);
  });
}

void SessionController::OnCaptureResult(const CaptureResult& result) {
  if (!IsPublishable(result.generation)) {
    return;  // X9: superseded capture must not open a popup
  }
  if (result.status != CaptureStatus::Ok) {
    FailCapture(result.status);
    return;
  }
  if (result.text.empty()) {
    FailCapture(CaptureStatus::Empty);
    return;
  }
  source_text_ = result.text;

  // Egress stage: last local checkpoint before selected text may leave (G7).
  const PolicyVerdict verdict = policy_.EvaluateOutbound(prefs_, source_exe_, *source_text_);
  if (verdict != PolicyVerdict::Allow) {
    FailTranslation(ErrorForVerdict(verdict));
    return;
  }

  Publish(PopupPhase::Translating);

  TranslateRequest request;
  request.generation = generation_;
  request.text = *source_text_;
  request.source_lang = prefs_.pair.source;
  request.target_lang = prefs_.pair.target;
  if (!result.context.empty()) {
    request.context = result.context;
  }
  translation_.Translate(request, [this](TranslateResult translate_result) {
    OnTranslationResult(translate_result);
  });
}

void SessionController::OnTranslationResult(const TranslateResult& result) {
  if (!IsPublishable(result.generation)) {
    return;  // G8: a late N must never overwrite N+1
  }
  if (!result.ok()) {
    FailTranslation(result.error);
    return;
  }

  shape_ = result.shape;
  result_ = result;
  error_.reset();
  capture_status_.reset();
  Publish(PopupPhase::Success);
}

void SessionController::OnDismiss() {
  visible_ = false;
  source_text_.reset();
  capture_status_.reset();
  error_.reset();
  result_.reset();
  Publish(PopupPhase::Hidden);
}

void SessionController::OnConsentChanged(OnlineConsent consent) {
  prefs_.consent = consent;
  if (consent == OnlineConsent::Granted) {
    return;  // resuming the still-current request is handled by the caller
  }

  // Revocation invalidates in-flight work even when provider cancellation fails
  // (X10). The generation bump is what makes that true.
  ++generation_;
  capture_.Cancel(generation_ - 1);

  if (!visible_) {
    return;
  }
  FailTranslation(TranslationError::ConsentRequired);
}

void SessionController::Publish(PopupPhase phase) {
  PopupViewState state;
  state.generation = generation_;
  state.phase = phase;
  state.shape = shape_;
  state.anchor = anchor_;
  state.source_text = source_text_;
  state.capture_status = capture_status_;
  state.error = error_;
  state.result = result_;
  presenter_.Present(state);
}

void SessionController::FailCapture(CaptureStatus status) {
  capture_status_ = status;
  error_.reset();
  result_.reset();
  Publish(PopupPhase::Failure);
}

void SessionController::FailTranslation(TranslationError error) {
  error_ = error;
  result_.reset();
  Publish(error == TranslationError::ConsentRequired ? PopupPhase::ConsentRequired
                                                    : PopupPhase::Failure);
}

}  // namespace cliplingo
