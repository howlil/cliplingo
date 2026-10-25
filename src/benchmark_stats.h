#pragma once
#include <algorithm>
#include <cmath>
#include <cstddef>
#include <cstdint>
#include <limits>
#include <vector>

namespace bench_stats {
// Nearest-rank percentiles make CI output reproducible and avoid relying on
// floating point interpolation or rounding a single slow measurement away.
inline double percentile_ms(std::vector<double> samples, unsigned percentile) {
  if (samples.empty() || percentile < 1 || percentile > 100) return 0.0;
  for (const double v : samples)
    if (!std::isfinite(v) || v < 0.0) return 0.0;
  std::sort(samples.begin(), samples.end());
  const std::size_t index =
      (samples.size() * static_cast<std::size_t>(percentile) + 99) / 100 - 1;
  return samples[std::min(index, samples.size() - 1)];
}
inline bool valid_iterations(unsigned value) {
  return value >= 5 && value <= 200; // bound benchmark duration and output
}
} // namespace bench_stats
