#pragma once
#include <algorithm>
#include <cmath>
#include <cwctype>
#include <string>
#include <string_view>

namespace scope_text {
struct Rect { double left, top, width, height; };

// UIA RangeFromPoint snaps to nearby text even when the pointer is in empty
// space. Only a real visible text rectangle can authorize a hover translation.
inline bool contains(Rect r, double x, double y) {
  return std::isfinite(r.left) && std::isfinite(r.top) &&
         std::isfinite(r.width) && std::isfinite(r.height) &&
         r.width > 0 && r.height > 0 &&
         x >= r.left && x < r.left + r.width &&
         y >= r.top && y < r.top + r.height;
}

inline std::wstring trim(std::wstring_view s) {
  while (!s.empty() && std::iswspace(s.front())) s.remove_prefix(1);
  while (!s.empty() && std::iswspace(s.back())) s.remove_suffix(1);
  return std::wstring(s);
}

inline bool abbreviation(std::wstring_view s, std::size_t punctuation_at) {
  std::size_t start = punctuation_at;
  while (start > 0 && std::iswalpha(s[start - 1])) --start;
  if (start == punctuation_at) return false;
  std::wstring word;
  for (auto ch : s.substr(start, punctuation_at - start)) word.push_back(std::towlower(ch));
  // Conservative English list; avoid interpreting titles and initials as
  // sentence boundaries. UIA has no TextUnit_Sentence.
  return word.size() == 1 || word == L"mr" || word == L"mrs" ||
         word == L"ms" || word == L"dr" || word == L"prof" ||
         word == L"sr" || word == L"jr" || word == L"st" ||
         word == L"vs" || word == L"etc";
}

inline bool boundary(std::wstring_view s, std::size_t at) {
  if (at >= s.size()) return false;
  const wchar_t ch = s[at];
  if (ch != L'.' && ch != L'!' && ch != L'?') return false;
  if (ch == L'.' && abbreviation(s, at)) return false;
  // Do not split decimals, ellipses, URLs or punctuation within a token.
  std::size_t after = at + 1;
  while (after < s.size() && (s[after] == L'"' || s[after] == L'\'' ||
                              s[after] == L')' || s[after] == L'”'))
    ++after;
  return after == s.size() || std::iswspace(s[after]);
}

// A bounded paragraph around the UIA insertion-point, not the full document.
// The caret offset comes from two UIA ranges of the SAME TextPattern provider.
// An empty string is a conservative failure, never a signal to expand to all text.
inline std::wstring sentence_at(std::wstring_view paragraph, std::size_t caret) {
  if (paragraph.empty() || caret > paragraph.size()) return {};
  const auto anchor = std::min(caret, paragraph.size() - 1);
  std::size_t start = 0;
  while (start < paragraph.size() && std::iswspace(paragraph[start])) ++start;
  for (std::size_t i = 0; i < paragraph.size(); ++i) {
    if (!boundary(paragraph, i)) continue;
    std::size_t end = i + 1;
    while (end < paragraph.size() && (paragraph[end] == L'"' ||
           paragraph[end] == L'\'' || paragraph[end] == L')' || paragraph[end] == L'”')) ++end;
    if (anchor < end) return anchor < start ? std::wstring{} : trim(paragraph.substr(start, end - start));
    start = end;
    while (start < paragraph.size() && std::iswspace(paragraph[start])) ++start;
  }
  return anchor < start ? std::wstring{} : trim(paragraph.substr(start));
}
}  // namespace scope_text
