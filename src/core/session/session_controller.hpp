#pragma once

#include <cstdint>
#include <memory>
#include <optional>
#include <string>

#include "contracts/capture.hpp"
#include "contracts/popup.hpp"
#include "contracts/preferences.hpp"
#include "contracts/translation.hpp"
#include "core/preferences/request_policy.hpp"
#include "core/session/ports/capture_client.hpp"
#include "core/session/ports/popup_presenter.hpp"
#include "core/translation/translation_service.hpp"

namespace cliplingo {

// K3 / S1 (R4). Single-threaded by contract: every method runs on the UI
// thread and is the only owner of the active generation. Stale results are
// dropped at the one publish boundary (G8, X9), so cancellation never has to be
// reliable.
class SessionController {
 public:
  // The composition root must still stop workers before destroying the
  // collaborators (D13). Results that arrive after this object is destroyed are
  // ignored rather than dereferencing a dead controller.
  SessionController(CaptureClient& capture, TranslationService& translation,
                    PopupPresenter& presenter, RequestPolicy policy = {});
  ~SessionController();

  SessionController(const SessionController&) = delete;
  SessionController& operator=(const SessionController&) = delete;

  // B1: a new trigger supersedes whatever was in flight before any work starts.
  void BeginTrigger(SourceAnchor anchor, const Preferences& prefs, std::wstring source_exe);

  void OnCaptureResult(const CaptureResult& result);
  void OnTranslationResult(const TranslateResult& result);
  void OnDismiss();  // M4 / B5

  // Granting resumes the still-current request after re-checking exclusions;
  // revoking invalidates in-flight work but leaves a rendered result readable.
  void OnConsentChanged(OnlineConsent consent);

  [[nodiscard]] std::uint64_t generation() const noexcept { return generation_; }

 private:
  [[nodiscard]] bool IsPublishable(std::uint64_t generation) const noexcept {
    return visible_ && generation != 0 && generation == generation_;
  }

  void Publish(PopupPhase phase);
  void StartTranslation();
  void ResumeBlockedRequest();
  void ApplyOutboundVerdict(PolicyVerdict verdict);
  void FailCapture(CaptureStatus status);
  void FailTranslation(TranslationError error);

  CaptureClient& capture_;
  TranslationService& translation_;
  PopupPresenter& presenter_;
  const RequestPolicy policy_;
  // Liveness token captured by the sinks so a late callback after destruction
  // is a no-op instead of UB (D13).
  std::shared_ptr<bool> alive_;

  std::uint64_t generation_ = 0;
  PopupPhase phase_ = PopupPhase::Hidden;
  bool visible_ = false;
  Point anchor_{};
  TextShape shape_ = TextShape::Sentence;
  std::wstring source_exe_;
  Preferences prefs_{};
  std::optional<std::wstring> source_text_;     // S1: transient, never persisted (DEC7)
  std::optional<std::wstring> source_context_;  // needed to resume without re-capturing
  std::optional<CaptureStatus> capture_status_;
  std::optional<TranslationError> error_;
  std::optional<TranslateResult> result_;
};

}  // namespace cliplingo
