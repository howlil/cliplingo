#include "capture_policy.h"
#include <cassert>
#include <string>
#ifdef _WIN32
#include <windows.h>
#endif

int main() {
  using namespace capture_policy;
  auto empty = parse(L"");
  assert(empty.valid && empty.names.empty());
  auto parsed = parse(L"  Browser.EXE\r\nVAULT.exe;browser.exe;Password Manager.exe\n");
  assert(parsed.valid && parsed.names.size() == 3);
  assert(parsed.names[0] == L"browser.exe");
  assert(parsed.names[1] == L"vault.exe");
  assert(parsed.names[2] == L"password manager.exe");
  assert(serialize(parsed.names) == L"browser.exe;vault.exe;password manager.exe");
  assert(parse(serialize(parsed.names)).valid);
  assert(!parse(L"C:\\Sensitive\\vault.exe").valid);
  assert(!parse(L"*.exe").valid);
  assert(!parse(L"browser").valid);
  assert(!parse(L"browser.dll").valid);
  assert(!parse(L"example.exe|other.exe").valid);
  assert(!parse(L".exe").valid);
  assert(!parse(L"vau\tlt.exe").valid);
  assert(!parse(std::wstring(4097, L'a')).valid);
  std::wstring many;
  for (int i = 0; i < 33; ++i) {
    if (!many.empty()) many += L';';
    many += L"app" + std::to_wstring(i) + L".exe";
  }
  assert(!parse(many).valid);
  assert(parse(L"APP.exe;app.exe").names.size() == 1);
#ifdef _WIN32
  // Test real Win32 INI semantics: separators may not silently truncate a
  // privacy denylist during persistence/reload.
  wchar_t temp[32768]{};
  const DWORD n = GetTempPathW(32768, temp);
  assert(n > 0 && n < 32768);
  const std::wstring file = std::wstring(temp) +
      L"cliplingo-m4-policy-" + std::to_wstring(GetCurrentProcessId()) + L".ini";
  const auto persisted = serialize(parsed.names);
  assert(WritePrivateProfileStringW(L"privacy", L"excluded_apps",
                                   persisted.c_str(), file.c_str()));
  wchar_t restored[4097]{};
  const DWORD count = GetPrivateProfileStringW(L"privacy", L"excluded_apps",
                                              L"", restored, 4097, file.c_str());
  assert(count == persisted.size());
  assert(std::wstring_view(restored, count) == persisted);
  assert(parse(std::wstring_view(restored, count)).names.size() == 3);
  DeleteFileW(file.c_str());
#endif
  return 0;
}
