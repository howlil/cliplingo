#include "benchmark_stats.h"
#include <cstdlib>
#include <vector>
#include <limits>
#include <cmath>
void require(bool value) { if (!value) std::abort(); }
int main() {
  using namespace bench_stats;
  require(!valid_iterations(0));
  require(!valid_iterations(4));
  require(valid_iterations(5));
  require(valid_iterations(20));
  require(valid_iterations(200));
  require(!valid_iterations(201));
  require(percentile_ms({}, 95) == 0);
  require(percentile_ms({3, 1, 5, 2, 4}, 50) == 3);
  require(percentile_ms({3, 1, 5, 2, 4}, 95) == 5);
  require(percentile_ms({3, 1, 5, 2, 4}, 99) == 5);
  require(percentile_ms({3, 1, 5, 2, 4}, 100) == 5);
  require(percentile_ms({1, 2, 3}, 0) == 0);
  require(percentile_ms({1, 2, 3}, 101) == 0);
  require(percentile_ms({1, -1, 3}, 50) == 0);
  require(percentile_ms({1, std::numeric_limits<double>::quiet_NaN()}, 50) == 0);
  std::vector<double> data;
  for (int i=1; i<=100; ++i) data.push_back(static_cast<double>(i));
  require(percentile_ms(data, 50) == 50);
  require(percentile_ms(data, 95) == 95);
  require(percentile_ms(data, 99) == 99);
  return 0;
}
