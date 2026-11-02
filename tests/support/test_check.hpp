#pragma once

// Deliberately dependency-free test harness (D9): CTest drives one binary, and
// nothing is fetched at configure time. Replace only when a lockfile-backed
// framework is actually justified.

#include <cstdio>
#include <functional>
#include <string>
#include <vector>

namespace cl_test {

struct Case {
  const char* name = "";
  std::function<void()> body;
};

inline std::vector<Case>& Registry() {
  static std::vector<Case> registry;
  return registry;
}

inline int& FailureCount() {
  static int failures = 0;
  return failures;
}

struct Registrar {
  Registrar(const char* name, std::function<void()> body) {
    Registry().push_back(Case{name, std::move(body)});
  }
};

inline void Report(bool passed, const char* expression, const char* file, int line) {
  if (passed) {
    return;
  }
  ++FailureCount();
  std::printf("    FAILED %s:%d  %s\n", file, line, expression);
}

}  // namespace cl_test

#define CL_TEST(name)                                                    \
  static void name();                                                    \
  static ::cl_test::Registrar name##_registrar_(#name, name);            \
  static void name()

#define CL_CHECK(expression) \
  ::cl_test::Report(static_cast<bool>(expression), #expression, __FILE__, __LINE__)

#define CL_CHECK_EQ(actual, expected)                                          \
  do {                                                                         \
    const auto& cl_actual_ = (actual);                                         \
    const auto& cl_expected_ = (expected);                                     \
    ::cl_test::Report(cl_actual_ == cl_expected_, #actual " == " #expected,     \
                      __FILE__, __LINE__);                                     \
  } while (false)
