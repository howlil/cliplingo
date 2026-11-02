#include "core/session/session_controller.hpp"

#include <utility>

namespace cliplingo {

SessionController::SessionController(CaptureClient& capture, TranslationService& translation,
                                     PopupPresenter& presenter, RequestPolicy policy)
    : capture_(capture),
      translation_(translation),
      presenter_(presenter),
      policy_(policy),
      alive_(std::make_shared<bool>(true)) {}

SessionController::~SessionController() { *alive_ = false; }

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
  source_context_.reset();
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

  const std::shared_ptr<bool> alive = alive_;
  capture_.Capture(request, [this, alive](CaptureResult capture_result) {
    if (*alive) {
      OnCaptureResult(capture_result);
    }
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
  source_context_ = result.context.empty() ? std::nullopt
                                           : std::optional<std::wstring>(result.context);

  // Egress stage: last local checkpoint before selected text may leave (G7).
  ApplyOutboundVerdict(policy_.EvaluateOutbound(prefs_, source_exe_, *source_text_));
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
  source_context_.reset();
  capture_status_.reset();
  error_.reset();
  result_.reset();
  Publish(PopupPhase::Hidden);
}

void SessionController::OnConsentChanged(OnlineConsent consent) {
  prefs_.consent = consent;
  if (consent == OnlineConsent::Granted) {
    ResumeBlockedRequest();
    return;
  }

  // Revocation invalidates in-flight work even when provider cancellation fails
  // (X10). The generation bump is what makes that true.
  ++generation_;
  capture_.Cancel(generation_ - 1);

  if (!visible_) {
    return;
  }
  // Only unrendered work is invalidated: a result already on screen was never
  // transmitted and stays readable (K8 -> K3).
  if (phase_ == PopupPhase::Capturing || phase_ == PopupPhase::Translating) {
    FailTranslation(TranslationError::ConsentRequired);
  }
}

void SessionController::Publish(PopupPhase phase) {
  phase_ = phase;

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

void SessionController::StartTranslation() {
  Publish(PopupPhase::Translating);

  TranslateRequest request;
  request.generation = generation_;
  request.text = *source_text_;
  request.source_lang = prefs_.pair.source;
  request.target_lang = prefs_.pair.target;
  request.context = source_context_;

  const std::shared_ptr<bool> alive = alive_;
  translation_.Translate(request, [this, alive](TranslateResult translate_result) {
    if (*alive) {
      OnTranslationResult(translate_result);
    }
  });
}

void SessionController::ResumeBlockedRequest() {
  if (!visible_ || phase_ != PopupPhase::ConsentRequired || !source_text_.has_value()) {
    return;
  }
  // Consent alone is not enough: exclusions are re-checked before sending.
  ApplyOutboundVerdict(policy_.EvaluateOutbound(prefs_, source_exe_, *source_text_));
}

void SessionController::ApplyOutboundVerdict(PolicyVerdict verdict) {
  switch (verdict) {
    case PolicyVerdict::Allow:
      StartTranslation();
      return;
    case PolicyVerdict::DeniedExcluded:
      FailTranslation(TranslationError::Excluded);
      return;
    case PolicyVerdict::DeniedConsent:
      FailTranslation(TranslationError::ConsentRequired);
      return;
    case PolicyVerdict::DeniedEmptyText:
      // Whitespace-only selection is "no text", not an internal fault (X1).
      FailCapture(CaptureStatus::Empty);
      return;
  }
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
