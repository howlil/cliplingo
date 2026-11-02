#pragma once

#include <string>
#include <vector>

namespace cliplingo {

// S5: deny by default. Unset is not consent (X10).
enum class OnlineConsent {
  Unset,
  Denied,
  Granted,
};

struct LanguagePair {
  std::string source = "en";
  std::string target = "id";

  friend bool operator==(const LanguagePair&, const LanguagePair&) = default;
};

// Slice 1 subset of S4/S5. Hotkey, popup size and provider selection arrive
// with the slices that consume them.
struct Preferences {
  LanguagePair pair{};
  OnlineConsent consent = OnlineConsent::Unset;
  std::vector<std::wstring> excluded_apps;
};

}  // namespace cliplingo
