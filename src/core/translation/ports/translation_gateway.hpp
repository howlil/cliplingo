#pragma once

#include "contracts/translation.hpp"

namespace cliplingo {

// K4 -> DEP1 port (R5). Implemented by platform/translation, which owns the
// bounded network worker and maps HTTP/vendor failures into TranslationError.
// Swapping vendors must not change this header (DEC4).
class TranslationGateway {
 public:
  virtual ~TranslationGateway() = default;

  // Non-blocking for the caller: the call is queued and the sink is invoked on
  // the UI thread, possibly after the request went stale.
  virtual void Translate(const TranslateRequest& request, TranslationSink sink) = 0;
};

}  // namespace cliplingo
