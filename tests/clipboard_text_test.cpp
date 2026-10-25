#include "clipboard_text.h"
#include <cstdlib>
#include <string>

void require(bool ok) { if (!ok) std::abort(); }

int main() {
  using namespace clipboard_text;
  const wchar_t simple[] = L"Hello world";
  const auto valid = inspect(simple, sizeof(simple) / sizeof(wchar_t));
  require(valid.state == State::Valid && valid.text == L"Hello world");
  const wchar_t spaced[] = L" \t Hello \r\n";
  require(inspect(spaced, sizeof(spaced) / sizeof(wchar_t)).text == L"Hello");
  const wchar_t blank[] = L"\r\n \t";
  require(inspect(blank, sizeof(blank) / sizeof(wchar_t)).state == State::Empty);
  require(inspect(nullptr, 1).state == State::Invalid);
  const wchar_t no_null[] = {L'A', L'B', L'C'};
  require(inspect(no_null, 3).state == State::Invalid);
  std::wstring longest(kMaxUnits, L'a');
  longest.push_back(L'\0');
  require(inspect(longest.data(), longest.size()).state == State::Valid);
  std::wstring too_long(kMaxUnits + 1, L'x');
  too_long.push_back(L'\0');
  require(inspect(too_long.data(), too_long.size()).state == State::TooLong);
  // Never interpret only the first 8192 codepoints as a valid substring.
  require(inspect(too_long.data(), kMaxUnits + 1).state == State::TooLong);
  const wchar_t embedded[] = {L'A', L'\0', L'B', L'\0'};
  require(inspect(embedded, 4).text == L"A");
  return 0;
}
