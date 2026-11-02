#pragma once

#include <string_view>

#include "contracts/preferences.hpp"

namespace cliplingo {

enum class PolicyVerdict {
  Allow,
  DeniedConsent,
  DeniedExcluded,
  DeniedEmptyText,
};

// K8-owned egress rules (R7). Two stages because they gate different things:
// capture is local, transmission is not (X10, G7). Pure and stateless, so it is
// cheap to evaluate on the UI thread before any work is started.
class RequestPolicy {
 public:
  // Local-only stage: no consent needed, exclusions still apply.
  [[nodiscard]] PolicyVerdict EvaluateCapture(const Preferences& prefs,
                                              std::wstring_view source_exe) const;

  // Egress stage: the last check before selected text may leave the device.
  [[nodiscard]] PolicyVerdict EvaluateOutbound(const Preferences& prefs,
                                               std::wstring_view source_exe,
                                               std::wstring_view text) const;
};

}  // namespace cliplingo
