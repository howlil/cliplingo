#pragma once

#include <cstdint>

#include "contracts/capture.hpp"

namespace cliplingo {

// K3 -> K2 port (R2). Implemented by platform/windows; faked in unit tests.
class CaptureClient {
 public:
  virtual ~CaptureClient() = default;

  // Contract: the sink is invoked on the UI thread and may arrive after Cancel.
  // Implementations must bound their own work with CaptureRequest::deadline.
  virtual void Capture(const CaptureRequest& request, CaptureSink sink) = 0;

  // Best effort (DEC6): correctness comes from the generation check at publish,
  // not from successful cancellation. Cancelling a generation that never had a
  // request must be a no-op; implementations must not cancel "whatever is
  // current" instead.
  virtual void Cancel(std::uint64_t generation) = 0;
};

}  // namespace cliplingo
