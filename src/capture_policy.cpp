#include "core.h"
#include "capture_policy.h"
#include <windows.h>
#include <string>
#include <string_view>
#include <vector>

namespace {
struct ProcessHandle {
  HANDLE value{};
  explicit ProcessHandle(HANDLE h) : value(h) {}
  ~ProcessHandle() { if (value) CloseHandle(value); }
  ProcessHandle(const ProcessHandle&) = delete;
  ProcessHandle& operator=(const ProcessHandle&) = delete;
};
}

// Fail closed if we cannot prove the originating app's identity. This is
// triggered only by a user hotkey, never periodically while idle.
bool capture_source_allowed(HWND foreground, std::wstring_view policy) {
  const auto parsed = capture_policy::parse(policy);
  if (!parsed.valid || !foreground || !IsWindow(foreground))
    return false;
  DWORD pid = 0;
  GetWindowThreadProcessId(foreground, &pid);
  if (!pid || pid == GetCurrentProcessId()) return false;
  ProcessHandle process(OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, FALSE, pid));
  if (!process.value) return false;

  // Use full path only to extract the final basename. Never persist, record or
  // compare user directory information; it is unnecessary for exclusions.
  std::wstring path(32768, L'\0');
  DWORD length = static_cast<DWORD>(path.size());
  if (!QueryFullProcessImageNameW(process.value, 0, path.data(), &length) || !length ||
      length >= path.size()) return false;
  path.resize(length);
  const auto slash = path.find_last_of(L"\\/");
  const std::wstring_view name = std::wstring_view(path).substr(
      slash == std::wstring::npos ? 0 : slash + 1);
  if (name.empty()) return false;
  for (const auto& excluded : parsed.names) {
    // CompareStringOrdinal is Windows' invariant case-insensitive Unicode
    // comparison (no current user locale / path-prefix guesses).
    if (CompareStringOrdinal(name.data(), static_cast<int>(name.size()),
                             excluded.c_str(), static_cast<int>(excluded.size()),
                             TRUE) == CSTR_EQUAL)
      return false;
  }
  return true;
}
