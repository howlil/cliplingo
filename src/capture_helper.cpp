#include "core.h"
#include <windows.h>
#include <shellapi.h>
#include <cstdint>
#include <cwchar>
#include <string>
#include <vector>
#include <algorithm>

namespace {
constexpr std::uint32_t kMagic = 0x434C5531; // CLU1
constexpr DWORD kCaptureTimeoutMs = 1800;
struct SharedCapture {
  std::uint32_t magic;
  std::uint32_t failure;
  std::uint32_t length;
  wchar_t text[8193];
};
struct UniqueHandle {
  HANDLE h{nullptr};
  ~UniqueHandle() { if (h && h != INVALID_HANDLE_VALUE) CloseHandle(h); }
  UniqueHandle() = default;
  explicit UniqueHandle(HANDLE handle) : h(handle) {}
  UniqueHandle(const UniqueHandle&) = delete;
  UniqueHandle& operator=(const UniqueHandle&) = delete;
  HANDLE get() const { return h; }
};
struct MappedCapture {
  SharedCapture* ptr{nullptr};
  explicit MappedCapture(HANDLE mapping)
    : ptr(static_cast<SharedCapture*>(MapViewOfFile(mapping, FILE_MAP_ALL_ACCESS, 0, 0, sizeof(SharedCapture)))) {}
  ~MappedCapture() { if (ptr) UnmapViewOfFile(ptr); }
  MappedCapture(const MappedCapture&) = delete;
  MappedCapture& operator=(const MappedCapture&) = delete;
};
struct ChildProcess {
  PROCESS_INFORMATION info{};
  ~ChildProcess() { if (info.hThread) CloseHandle(info.hThread); if (info.hProcess) CloseHandle(info.hProcess); }
};
}

// A crash, COM provider deadlock, or hard timeout only kills the capture helper.
// This is intentionally per-request (bounded resource lifetime). Benchmark before
// considering a persistent helper: never replace a safety property speculatively.
CaptureResult capture_isolated(HWND expected_foreground) {
  if (!expected_foreground) return {Failure::NoSelection, {}};
  SECURITY_ATTRIBUTES security{sizeof(security), nullptr, TRUE};
  UniqueHandle mapping(CreateFileMappingW(INVALID_HANDLE_VALUE, &security, PAGE_READWRITE,
                                          0, sizeof(SharedCapture), nullptr));
  if (!mapping.get()) return {Failure::CaptureFailed, {}};
  MappedCapture shared(mapping.get());
  if (!shared.ptr) return {Failure::CaptureFailed, {}};
  *shared.ptr = SharedCapture{};

  wchar_t executable[MAX_PATH]{};
  const DWORD length = GetModuleFileNameW(nullptr, executable, MAX_PATH);
  if (!length || length >= MAX_PATH) return {Failure::CaptureFailed, {}};
  std::wstring command = L"\"" + std::wstring(executable) + L"\" --capture-helper " +
    std::to_wstring(reinterpret_cast<std::uintptr_t>(expected_foreground)) + L" " +
    std::to_wstring(reinterpret_cast<std::uintptr_t>(mapping.get()));

  // Inherit *only* the shared-memory handle using the extended startup attribute list.
  SIZE_T bytes = 0;
  InitializeProcThreadAttributeList(nullptr, 1, 0, &bytes);
  if (!bytes) return {Failure::CaptureFailed, {}};
  std::vector<unsigned char> attribute_buffer(bytes);
  auto* attributes = reinterpret_cast<PPROC_THREAD_ATTRIBUTE_LIST>(attribute_buffer.data());
  if (!InitializeProcThreadAttributeList(attributes, 1, 0, &bytes)) return {Failure::CaptureFailed, {}};
  struct AttributeScope {
    PPROC_THREAD_ATTRIBUTE_LIST list;
    ~AttributeScope() { DeleteProcThreadAttributeList(list); }
  } attribute_scope{attributes};
  HANDLE handles[] = {mapping.get()};
  if (!UpdateProcThreadAttribute(attributes, 0, PROC_THREAD_ATTRIBUTE_HANDLE_LIST,
                                  handles, sizeof(handles), nullptr, nullptr))
    return {Failure::CaptureFailed, {}};
  STARTUPINFOEXW startup{};
  startup.StartupInfo.cb = sizeof(startup);
  startup.lpAttributeList = attributes;
  ChildProcess child;
  if (!CreateProcessW(executable, command.data(), nullptr, nullptr, TRUE,
      EXTENDED_STARTUPINFO_PRESENT | CREATE_NO_WINDOW, nullptr, nullptr,
      &startup.StartupInfo, &child.info))
    return {Failure::CaptureFailed, {}};
  const DWORD waited = WaitForSingleObject(child.info.hProcess, kCaptureTimeoutMs);
  if (waited != WAIT_OBJECT_0) {
    TerminateProcess(child.info.hProcess, ERROR_TIMEOUT);
    WaitForSingleObject(child.info.hProcess, INFINITE);
    return {Failure::CaptureFailed, {}};
  }
  DWORD exit_code = 0;
  if (!GetExitCodeProcess(child.info.hProcess, &exit_code) || exit_code != 0 ||
      shared.ptr->magic != kMagic || shared.ptr->length > 8192 ||
      shared.ptr->failure > static_cast<std::uint32_t>(Failure::TranslationFailed))
    return {Failure::CaptureFailed, {}};
  const auto code = static_cast<Failure>(shared.ptr->failure);
  if (code != Failure::None) return {code, {}};
  if (shared.ptr->length == 0) return {Failure::NoSelection, {}};
  return {Failure::None, std::wstring(shared.ptr->text, shared.ptr->length)};
}

// The helper MUST NOT instantiate the app, model, tray or popup. Only one COM MTA
// apartment, bounded IPC buffer, then process exit. Never marshal COM objects.
bool maybe_run_capture_helper(int& exit_code) {
  int argc = 0;
  LPWSTR* argv = CommandLineToArgvW(GetCommandLineW(), &argc);
  if (!argv) return false;
  const bool is_helper = argc >= 2 && wcscmp(argv[1], L"--capture-helper") == 0;
  if (!is_helper) { LocalFree(argv); return false; }
  exit_code = 1;
  if (argc == 4) {
    wchar_t *target_end = nullptr, *map_end = nullptr;
    const auto target = _wcstoui64(argv[2], &target_end, 10);
    const auto shared_handle = _wcstoui64(argv[3], &map_end, 10);
    if (*target_end == 0 && *map_end == 0 && target != 0 && shared_handle != 0) {
      MappedCapture shared(reinterpret_cast<HANDLE>(static_cast<std::uintptr_t>(shared_handle)));
      if (shared.ptr) {
        const HRESULT hr = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
        if (SUCCEEDED(hr)) {
          auto result = capture_selected_text(reinterpret_cast<HWND>(static_cast<std::uintptr_t>(target)));
          shared.ptr->failure = static_cast<std::uint32_t>(result.failure);
          if (result && result.text.size() <= 8192) {
            shared.ptr->length = static_cast<std::uint32_t>(result.text.size());
            std::copy(result.text.begin(), result.text.end(), shared.ptr->text);
          }
          shared.ptr->magic = kMagic; // written LAST; parent checks after process exit
          CoUninitialize();
          exit_code = 0;
        }
      }
    }
  }
  LocalFree(argv);
  return true;
}