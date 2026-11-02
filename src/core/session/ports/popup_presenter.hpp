#pragma once

#include "contracts/popup.hpp"

namespace cliplingo {

// K3 -> K5 port (R3, S3). Implemented by ui/win32; the popup owns no truth
// about the active request.
class PopupPresenter {
 public:
  virtual ~PopupPresenter() = default;

  // Called on the UI thread. Implementations must not block: the capture and
  // network work stays off this path (B3, G1).
  virtual void Present(const PopupViewState& state) = 0;
};

}  // namespace cliplingo
