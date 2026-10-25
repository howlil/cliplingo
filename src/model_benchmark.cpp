#include "core.h"
#include "benchmark_stats.h"
#include <windows.h>
#include <shellapi.h>
#include <psapi.h>
#include <algorithm>
#include <array>
#include <chrono>
#include <cstdint>
#include <cwchar>
#include <filesystem>
#include <fstream>
#include <iomanip>
#include <numeric>
#include <optional>
#include <string>
#include <string_view>
#include <vector>

namespace {
using Clock = std::chrono::steady_clock;
struct Memory {
  std::uint64_t private_bytes{};
  std::uint64_t working_set_bytes{};
  std::uint64_t peak_working_set_bytes{};
};
std::optional<Memory> memory_sample() {
  PROCESS_MEMORY_COUNTERS_EX counters{};
  counters.cb = sizeof(counters);
  if (!GetProcessMemoryInfo(GetCurrentProcess(),
                            reinterpret_cast<PROCESS_MEMORY_COUNTERS*>(&counters),
                            static_cast<DWORD>(sizeof(counters)))) return std::nullopt;
  return Memory{static_cast<std::uint64_t>(counters.PrivateUsage),
                static_cast<std::uint64_t>(counters.WorkingSetSize),
                static_cast<std::uint64_t>(counters.PeakWorkingSetSize)};
}
std::optional<std::uint64_t> cpu_time_100ns() {
  FILETIME created{}, exited{}, kernel{}, user{};
  if (!GetProcessTimes(GetCurrentProcess(), &created, &exited, &kernel, &user))
    return std::nullopt;
  const auto units = [](FILETIME f) -> std::uint64_t {
    return (static_cast<std::uint64_t>(f.dwHighDateTime) << 32) |
           static_cast<std::uint64_t>(f.dwLowDateTime);
  };
  return units(kernel) + units(user);
}
double elapsed_ms(Clock::time_point start, Clock::time_point end) {
  return std::chrono::duration<double, std::milli>(end - start).count();
}
struct Cohort {
  const char* name;
  const wchar_t* english;
  std::vector<double> elapsed;
};
struct ModelUnload {
  ~ModelUnload() { unload_translation_model(); }
};
// Native CLI entry path calls exactly the same translator used by the
// GUI worker. There are no selected-text logs, telemetry or external APIs.
int execute_benchmark(const std::filesystem::path& model,
                      const std::filesystem::path& output, unsigned iterations) {
  if (!bench_stats::valid_iterations(iterations) || model.empty() || output.empty())
    return 2;
  auto before = memory_sample();
  const auto cpu_before = cpu_time_100ns();
  if (!before || !cpu_before) return 4;
  ModelUnload cleanup; // one unload even if inference or I/O throws

  const auto cold_start = Clock::now();
  auto cold_result = translate_en_id(L"Hello world!", model);
  const double cold_ms = elapsed_ms(cold_start, Clock::now());
  if (!cold_result || cold_result.text.find(L"dunia") == std::wstring::npos)
    return 5;
  const auto loaded = memory_sample();
  if (!loaded) return 4;

  std::array<Cohort, 3> cohorts{{
      {"word", L"Hello!", {}},
      {"sentence", L"The book is on the table.", {}},
      {"paragraph",
       L"Hello world! The book is on the table. Thank you for your help.", {}},
  }};
  for (auto& cohort : cohorts) {
    cohort.elapsed.reserve(iterations);
    for (unsigned i = 0; i < iterations; ++i) {
      const auto start = Clock::now();
      auto result = translate_en_id(cohort.english, model);
      const double ms = elapsed_ms(start, Clock::now());
      if (!result || result.text.empty()) return 6;
      cohort.elapsed.push_back(ms);
    }
  }
  const auto warm = memory_sample();
  const auto cpu_after = cpu_time_100ns();
  if (!warm || !cpu_after || *cpu_after < *cpu_before) return 4;

  // Explicit user-specified artifact. Never write captured text, model paths,
  // or source-document contents into the report.
  auto temporary = output;
  temporary += L".partial";
  std::ofstream file(temporary, std::ios::binary | std::ios::trunc);
  if (!file) return 7;
  file << std::fixed << std::setprecision(3);
  file << "{\n  \"schema\": 1,\n  \"workload\": \"fixed-offline-en-id\",\n"
       << "  \"iterations_per_cohort\": " << iterations << ",\n"
       << "  \"cold_model_ms\": " << cold_ms << ",\n"
       << "  \"cpu_total_ms\": " << ((*cpu_after - *cpu_before) / 10000.0) << ",\n"
       << "  \"process_memory\": {\n"
       << "    \"before_private_bytes\": " << before->private_bytes << ",\n"
       << "    \"before_working_set_bytes\": " << before->working_set_bytes << ",\n"
       << "    \"model_loaded_private_bytes\": " << loaded->private_bytes << ",\n"
       << "    \"warm_private_bytes\": " << warm->private_bytes << ",\n"
       << "    \"warm_working_set_bytes\": " << warm->working_set_bytes << ",\n"
       << "    \"peak_working_set_bytes\": " << warm->peak_working_set_bytes
       << "\n  },\n  \"cohorts\": {\n";
  for (std::size_t index = 0; index < cohorts.size(); ++index) {
    const auto& cohort = cohorts[index];
    const double sum = std::accumulate(cohort.elapsed.begin(), cohort.elapsed.end(), 0.0);
    file << "    \"" << cohort.name << "\": {"
         << "\"mean_ms\": " << sum / cohort.elapsed.size()
         << ", \"p50_ms\": " << bench_stats::percentile_ms(cohort.elapsed, 50)
         << ", \"p95_ms\": " << bench_stats::percentile_ms(cohort.elapsed, 95)
         << ", \"p99_ms\": " << bench_stats::percentile_ms(cohort.elapsed, 99)
         << "}" << (index + 1 < cohorts.size() ? "," : "") << "\n";
  }
  file << "  }\n}\n";
  file.flush();
  if (!file) return 7;
  file.close();
  if (!file) return 7; // preserve previous report if close/disk flush fails
  if (!MoveFileExW(temporary.c_str(), output.c_str(),
                   MOVEFILE_REPLACE_EXISTING | MOVEFILE_WRITE_THROUGH)) return 7;
  return 0;
}
} // namespace

// -1 means "not my CLI", so normal ClipLingo startup continues untouched.
int run_benchmark_if_requested() {
  int argc = 0;
  LPWSTR* args = CommandLineToArgvW(GetCommandLineW(), &argc);
  if (!args) return -1;
  const bool selected = argc > 1 && wcscmp(args[1], L"--benchmark-model") == 0;
  if (!selected) { LocalFree(args); return -1; }
  int result = 2;
  try {
    if (argc == 4 || argc == 5) {
      const std::filesystem::path model(args[2]), report(args[3]);
      unsigned iterations = 20;
      if (argc == 5) {
        wchar_t* end = nullptr;
        const auto count = std::wcstoul(args[4], &end, 10);
        if (!end || *end != L'\0' || count > 200) {
          LocalFree(args); return 2;
        }
        iterations = static_cast<unsigned>(count);
      }
      result = execute_benchmark(model, report, iterations);
    }
  } catch (...) {
    // All resource-owning types above use RAII. Nothing may unwind through
    // the Win32 wWinMain boundary; the process returns a failure code.
    result = 8;
  }
  LocalFree(args);
  return result;
}
