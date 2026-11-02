#pragma once

#include <cstdint>
#include <optional>
#include <string>

#include "contracts/capture.hpp"
#include "contracts/geometry.hpp"
#include "contracts/translation.hpp"

namespace cliplingo {

// K3 -> K5 (S3). Presentation phases only; the popup owns no translation truth.
enum class PopupPhase {
  Hidden,
  Capturing,
  ConsentRequired,
  Translating,
  Success,
  Failure,
};

// Immutable snapshot handed to the UI. Fields are populated per phase so the UI
// never has to infer state from missing data.
struct PopupViewState {
  std::uint64_t generation = 0;
  PopupPhase phase = PopupPhase::Hidden;
  TextShape shape = TextShape::Sentence;
  Point anchor{};

  std::optional<std::wstring> source_text;
  std::optional<CaptureStatus> capture_status;      // V5, V8
  std::optional<TranslationError> error;            // V6
  std::optional<TranslateResult> result;            // V2, V3, V4
};

}  // namespace cliplingo
