#pragma once

#include <cstdint>
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
  // All collaborators outlive the controller: worker threads are stopped before
  // the controller is destroyed (see the composition root in a later slice).
  SessionController(CaptureClient& capture, TranslationService& translation,
                    PopupPresenter& presenter, RequestPolicy policy = {});
  SessionController(const SessionController&) = delete;
  SessionController& operator=(const SessionController&) = delete;

  // B1: a new trigger supersedes whatever was in flight before any work starts.
  void BeginTrigger(SourceAnchor anchor, const Preferences& prefs, std::wstring source_exe);

  void OnCaptureResult(const CaptureResult& result);
  void OnTranslationResult(const TranslateResult& result);
  void OnDismiss();  // M4 / B5
  void OnConsentChanged(OnlineConsent consent);

  [[nodiscard]] std::uint64_t generation() const noexcept { return generation_; }

 private:
  [[nodiscard]] bool IsPublishable(std::uint64_t generation) const noexcept {
    return visible_ && generation != 0 && generation == generation_;
  }

  void Publish(PopupPhase phase);
  void FailCapture(CaptureStatus status);
  void FailTranslation(TranslationError error);

  CaptureClient& capture_;
  TranslationService& translation_;
  PopupPresenter& presenter_;
  const RequestPolicy policy_;

  std::uint64_t generation_ = 0;
  bool visible_ = false;
  Point anchor_{};
  TextShape shape_ = TextShape::Sentence;
  std::wstring source_exe_;
  Preferences prefs_{};
  std::optional<std::wstring> source_text_;  // S1: transient, never persisted (DEC7)
  std::optional<CaptureStatus> capture_status_;
  std::optional<TranslationError> error_;
  std::optional<TranslateResult> result_;
};

}  // namespace cliplingo
