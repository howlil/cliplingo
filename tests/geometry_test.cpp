#include "popup_geometry.h"
#include <cassert>
int main() {
  using namespace popup_geometry;
  const Rect screen{0, 0, 1920, 1040};
  const auto center = place(screen, {900, 600}, 420, 90, 12);
  assert(center.left == 690 && center.top == 498 && center.bottom == 588);
  const auto top = place(screen, {40, 5}, 420, 140, 12);
  assert(top.top == 17 && top.left == 0);
  const auto right = place(screen, {1918, 900}, 420, 140, 12);
  assert(right.right == 1920);
  const Rect left_monitor{-1920, -100, 0, 980};
  const auto negative = place(left_monitor, {-1800, 0}, 400, 100, 12);
  assert(negative.left == -1920 && negative.top == 12);
  const auto oversize = place(screen, {900, 200}, 3000, 3000, 12);
  assert(oversize.right == 1920 && oversize.bottom == 1040);
}