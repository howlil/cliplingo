#pragma once
#include <algorithm>

namespace popup_geometry {
struct Rect { int left, top, right, bottom; };
struct Point { int x, y; };
inline Rect place(Rect work, Point anchor, int width, int height, int gap) {
  const int available_w = std::max(1, work.right - work.left);
  const int available_h = std::max(1, work.bottom - work.top);
  width = std::clamp(width, 1, available_w);
  height = std::clamp(height, 1, available_h);
  const int x = std::clamp(anchor.x - width / 2, work.left, work.right - width);
  const bool above = anchor.y - gap - height >= work.top;
  const int candidate_y = above ? anchor.y - gap - height : anchor.y + gap;
  const int y = std::clamp(candidate_y, work.top, work.bottom - height);
  return {x, y, x + width, y + height};
}
}