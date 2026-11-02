#pragma once

namespace cliplingo {

// Screen-space geometry expressed without Win32 types so core code stays
// portable and testable (F1). Platform adapters convert RECT/POINT at their own
// boundary.
struct Point {
  int x = 0;
  int y = 0;

  friend bool operator==(const Point&, const Point&) = default;
};

struct Rect {
  int left = 0;
  int top = 0;
  int right = 0;
  int bottom = 0;

  [[nodiscard]] int width() const noexcept { return right - left; }
  [[nodiscard]] int height() const noexcept { return bottom - top; }
  [[nodiscard]] bool is_empty() const noexcept { return width() <= 0 || height() <= 0; }

  friend bool operator==(const Rect&, const Rect&) = default;
};

}  // namespace cliplingo
