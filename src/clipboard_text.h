#pragma once
#include <algorithm>
#include <cwctype>
#include <cstddef>
#include <string>
#include <string_view>

namespace clipboard_text {
constexpr std::size_t kMaxUnits = 8192;
enum class State { Valid, Empty, TooLong, Invalid };
struct Result { State state{State::Invalid}; std::wstring text; };

// Inspect a foreign CF_UNICODETEXT allocation without assuming a terminator,
// trusted size, or intact Unicode contents. Never silently truncate a passage.
inline Result inspect(const wchar_t* data, std::size_t available_units) {
  if (!data || !available_units) return {};
  const auto limit = std::min(available_units, kMaxUnits + 1);
  std::size_t count = 0;
  while (count < limit && data[count] != L'\0') ++count;
  if (count == limit) return {
      available_units > kMaxUnits ? State::TooLong : State::Invalid, {}};
  if (count > kMaxUnits) return {State::TooLong, {}};
  const std::wstring_view view(data, count);
  auto first = view.find_first_not_of(L" \t\r\n\f\v");
  if (first == std::wstring_view::npos) return {State::Empty, {}};
  auto last = view.find_last_not_of(L" \t\r\n\f\v");
  return {State::Valid, std::wstring(view.substr(first, last - first + 1))};
}
}  // namespace clipboard_text
