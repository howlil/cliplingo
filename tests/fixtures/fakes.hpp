#pragma once

#include <cstdint>
#include <optional>
#include <utility>
#include <vector>

#include "contracts/capture.hpp"
#include "contracts/popup.hpp"
#include "contracts/translation.hpp"
#include "core/session/ports/capture_client.hpp"
#include "core/session/ports/popup_presenter.hpp"
#include "core/translation/ports/translation_gateway.hpp"

namespace cl_test {

// A fake proves only the behaviour whose semantics it preserves: here that is
// "the sink is owed exactly one result, delivered when the test decides", which
// is what out-of-order arrival means for the generation gate.
class FakeCaptureClient final : public cliplingo::CaptureClient {
 public:
  void Capture(const cliplingo::CaptureRequest& request, cliplingo::CaptureSink sink) override {
    ++calls;
    last_request = request;
    if (synchronous_result) {
      // A conforming adapter may resolve inside Capture(); the controller must
      // stay consistent when that happens.
      auto result = *synchronous_result;
      result.generation = request.generation;
      sink(result);
      return;
    }
    pending = std::move(sink);
  }

  void Cancel(std::uint64_t generation) override {
    ++cancels;
    last_cancelled = generation;
  }

  void Deliver(cliplingo::CaptureResult result) {
    if (pending) {
      pending(std::move(result));
    }
  }

  int calls = 0;
  int cancels = 0;
  std::uint64_t last_cancelled = 0;
  std::optional<cliplingo::CaptureRequest> last_request;
  std::optional<cliplingo::CaptureResult> synchronous_result;
  cliplingo::CaptureSink pending;
};

class FakeTranslationGateway final : public cliplingo::TranslationGateway {
 public:
  void Translate(const cliplingo::TranslateRequest& request,
                 cliplingo::TranslationSink sink) override {
    ++calls;
    last_request = request;
    if (synchronous_result) {
      auto result = *synchronous_result;
      result.generation = request.generation;
      sink(result);
      return;
    }
    pending = std::move(sink);
  }

  void Deliver(cliplingo::TranslateResult result) {
    if (pending) {
      pending(std::move(result));
    }
  }

  int calls = 0;
  std::optional<cliplingo::TranslateRequest> last_request;
  std::optional<cliplingo::TranslateResult> synchronous_result;
  cliplingo::TranslationSink pending;
};

class RecordingPresenter final : public cliplingo::PopupPresenter {
 public:
  void Present(const cliplingo::PopupViewState& state) override { states.push_back(state); }

  [[nodiscard]] bool Saw(cliplingo::PopupPhase phase, std::uint64_t generation) const {
    for (const auto& state : states) {
      if (state.phase == phase && state.generation == generation) {
        return true;
      }
    }
    return false;
  }

  [[nodiscard]] const cliplingo::PopupViewState& Last() const { return states.back(); }

  std::vector<cliplingo::PopupViewState> states;
};

}  // namespace cl_test
