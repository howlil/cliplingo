#pragma once

#include <chrono>
#include <cstddef>
#include <cstdint>
#include <functional>
#include <string>
#include <vector>

#include "contracts/geometry.hpp"

namespace cliplingo {

// K2 -> K3 result kinds. Every value is an expected, user-explainable outcome
// (X1-X3, X10); defects are not represented here and must not be invented.
enum class CaptureStatus {
  Ok,
  Empty,
  Unsupported,
  Protected,
  Excluded,
  TimedOut,
  Internal,
};

// Frozen at trigger time (S2). Later focus changes must not retarget capture.
struct SourceAnchor {
  std::uintptr_t source_hwnd = 0;
  Point cursor{};
  Rect selection{};
  bool has_selection_rect = false;
};

// K3 -> K2 request. `deadline` bounds the caller's wait, not the COM call.
struct CaptureRequest {
  std::uint64_t generation = 0;
  SourceAnchor anchor{};
  std::size_t max_chars = 8000;
  std::chrono::milliseconds deadline{400};
  std::wstring source_exe;
  std::vector<std::wstring> excluded_apps;
};

struct CaptureResult {
  std::uint64_t generation = 0;
  CaptureStatus status = CaptureStatus::Internal;
  std::wstring text;
  std::wstring context;
  std::wstring failure_detail;  // local diagnostics only, never shown raw to users
};

// Contract: the implementation invokes the sink on the UI thread, and may
// invoke it after Cancel(). Consumers must re-check the generation.
using CaptureSink = std::function<void(CaptureResult)>;

}  // namespace cliplingo
