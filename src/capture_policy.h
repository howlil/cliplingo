#pragma once
#include <algorithm>
#include <cwctype>
#include <string>
#include <string_view>
#include <vector>

// User-defined privacy policy, deliberately independent of Win32/COM.
// One executable basename per line or semicolon; never accept paths/globs,
// which could match the wrong process or expose selected text unexpectedly.
namespace capture_policy {
struct ParseResult {
  bool valid{false};
  std::vector<std::wstring> names;
  std::wstring error;
};
inline std::wstring lower(std::wstring_view source) {
  std::wstring result;
  result.reserve(source.size());
  for (wchar_t ch : source) result.push_back(static_cast<wchar_t>(std::towlower(ch)));
  return result;
}
inline std::wstring trim(std::wstring_view text) {
  while (!text.empty() && std::iswspace(text.front())) text.remove_prefix(1);
  while (!text.empty() && std::iswspace(text.back())) text.remove_suffix(1);
  return std::wstring(text);
}
inline bool valid_executable(std::wstring_view s) {
  if (s.size() < 5 || s.size() > 96) return false;
  const auto suffix = lower(s.substr(s.size() - 4));
  if (suffix != L".exe") return false;
  for (wchar_t c : s) {
    if (c < 0x20 || c == 0x7F || c == L'\\' || c == L'/' ||
        c == L':' || c == L'*' || c == L'?' || c == L'"' ||
        c == L'<' || c == L'>' || c == L'|' || c == L';')
      return false;
  }
  return s != L".exe";
}
inline ParseResult parse(std::wstring_view input) {
  ParseResult out;
  if (input.size() > 4096) { out.error = L"Policy exceeds 4096 characters."; return out; }
  std::size_t start = 0;
  while (start <= input.size()) {
    const auto end = input.find_first_of(L";\r\n", start);
    const auto piece = trim(input.substr(start, end == std::wstring_view::npos ?
                                              end : end - start));
    if (!piece.empty()) {
      if (!valid_executable(piece)) {
        out.error = L"Each entry must be an executable filename such as browser.exe, not a path or wildcard.";
        return out;
      }
      const auto lowered = lower(piece);
      if (std::find(out.names.begin(), out.names.end(), lowered) == out.names.end()) {
        out.names.push_back(lowered);
        if (out.names.size() > 32) {
          out.error = L"Maximum 32 excluded executable names.";
          return out;
        }
      }
    }
    if (end == std::wstring_view::npos) break;
    start = end + 1;
  }
  out.valid = true;
  return out;
}
inline std::wstring serialize(const std::vector<std::wstring>& names, wchar_t separator = L';') {
  std::wstring result;
  for (const auto& name : names) {
    if (!result.empty()) result += separator;
    result += name;
  }
  return result;
}
} // namespace capture_policy
