#include <cstdio>

#include "support/test_check.hpp"

int main() {
  // Unbuffered: a crashing case must still show which check killed it.
  std::setvbuf(stdout, nullptr, _IONBF, 0);
  int failed_cases = 0;
  for (const auto& test_case : cl_test::Registry()) {
    const int failures_before = cl_test::FailureCount();
    test_case.body();
    const bool passed = cl_test::FailureCount() == failures_before;
    std::printf("%s %s\n", passed ? "[ ok ]" : "[fail]", test_case.name);
    if (!passed) {
      ++failed_cases;
    }
  }

  std::printf("\n%d test(s), %d failed case(s), %d failed check(s)\n",
              static_cast<int>(cl_test::Registry().size()), failed_cases,
              cl_test::FailureCount());
  return failed_cases == 0 ? 0 : 1;
}
