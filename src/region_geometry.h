#pragma once
#include <cstdint>

namespace region_geometry {
struct Rect { int left, top, right, bottom; };
inline Rect normalize(int ax, int ay, int bx, int by) {
  return {ax < bx ? ax : bx, ay < by ? ay : by,
          ax > bx ? ax : bx, ay > by ? ay : by};
}
inline Rect clamp(Rect selected, Rect desktop) {
  auto max_int = [](int a, int b) { return a > b ? a : b; };
  auto min_int = [](int a, int b) { return a < b ? a : b; };
  return {max_int(desktop.left, min_int(selected.left, desktop.right)),
          max_int(desktop.top, min_int(selected.top, desktop.bottom)),
          max_int(desktop.left, min_int(selected.right, desktop.right)),
          max_int(desktop.top, min_int(selected.bottom, desktop.bottom))};
}
inline bool valid(Rect r) {
  const std::int64_t w = static_cast<std::int64_t>(r.right) - r.left;
  const std::int64_t h = static_cast<std::int64_t>(r.bottom) - r.top;
  // 12 MP / 48 MB maximum capture; prevent overflow or excessive OCR work.
  return w >= 8 && h >= 8 && w <= 8192 && h <= 8192 &&
         w * h <= 12000000;
}
}
