#include "scope_text.h"
#include <cassert>
#include <string>

int main() {
  using namespace scope_text;
  assert(contains({-800, -100, 140, 32}, -760, -89));
  assert(!contains({-800, -100, 140, 32}, -640, -89)); // right edge exclusive
  assert(!contains({0, 0, 0, 50}, 0, 20));           // no visible text
  assert(!contains({0, 0, 100, 10}, 99, 12));        // nearest != inside
  assert(!contains({0, 0, 100, 10}, 1000, 0));
  assert(sentence_at(L"Hello world! The book is here.", 3) == L"Hello world!");
  assert(sentence_at(L"Hello world! The book is here.", 22) == L"The book is here.");
  assert(sentence_at(L"Dr. Smith arrived. He left.", 10) == L"Dr. Smith arrived.");
  assert(sentence_at(L"Dr. Smith arrived. He left.", 21) == L"He left.");
  assert(sentence_at(L"The price is 3.14 dollars. Done.", 19) == L"The price is 3.14 dollars.");
  assert(sentence_at(L"A sentence?  Another one!", 15) == L"Another one!");
  assert(sentence_at(L"  One sentence without a period  ", 12) == L"One sentence without a period");
  assert(sentence_at(L"", 0).empty());
  assert(sentence_at(L"abc", 99).empty());
  assert(trim(L" \n\tText  ") == L"Text");
}
