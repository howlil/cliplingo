#include "region_geometry.h"
#include <cassert>
int main() {
  using namespace region_geometry;
  auto r = normalize(150, 210, -350, -70);
  assert(r.left == -350 && r.top == -70 && r.right == 150 && r.bottom == 210);
  assert(valid(r));
  assert(!valid({0,0,0,900}));
  assert(!valid({0,0,7,400}));
  assert(valid({0,0,8192,100}));
  assert(!valid({0,0,8192,8192}));
  assert(!valid({0,0,4000,4000}));
  assert(valid({-2000,-1000,2000,1000}));
  auto bounded = clamp({-500,-500,900,900}, {-200,-100,800,600});
  assert(bounded.left == -200 && bounded.top == -100);
  assert(bounded.right == 800 && bounded.bottom == 600);
  assert(!valid(clamp({999,999,1200,1200}, {-200,-100,800,600})));
}
