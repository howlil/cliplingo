#include "core/preferences/request_policy.hpp"

#include <cwctype>
#include <string>

namespace cliplingo {
namespace {

// Exclusions are matched on the executable name: full paths break across
// versions, "Program Files (x86)", and portable installs.
std::wstring_view BaseName(std::wstring_view path) {
  const auto separator = path.find_last_of(L"\\/");
  if (separator == std::wstring_view::npos) {
    return path;
  }
  return path.substr(separator + 1);
}

bool EqualsIgnoreCase(std::wstring_view left, std::wstring_view right) {
  if (left.size() != right.size()) {
    return false;
  }
  for (std::size_t index = 0; index < left.size(); ++index) {
    if (std::towlower(static_cast<std::wint_t>(left[index])) !=
        std::towlower(static_cast<std::wint_t>(right[index]))) {
      return false;
    }
  }
  return true;
}

bool IsBlank(std::wstring_view text) {
  for (const wchar_t value : text) {
    if (!std::iswspace(static_cast<std::wint_t>(value))) {
      return false;
    }
  }
  return true;
}

bool IsExcluded(const Preferences& prefs, std::wstring_view source_exe) {
  // An unknown source app cannot be proven excluded; consent still gates egress.
  if (source_exe.empty()) {
    return false;
  }
  const std::wstring_view name = BaseName(source_exe);
  for (const std::wstring& excluded : prefs.excluded_apps) {
    if (EqualsIgnoreCase(name, BaseName(excluded))) {
      return true;
    }
  }
  return false;
}

}  // namespace

PolicyVerdict RequestPolicy::EvaluateCapture(const Preferences& prefs,
                                             std::wstring_view source_exe) const {
  return IsExcluded(prefs, source_exe) ? PolicyVerdict::DeniedExcluded : PolicyVerdict::Allow;
}

PolicyVerdict RequestPolicy::EvaluateOutbound(const Preferences& prefs,
                                              std::wstring_view source_exe,
                                              std::wstring_view text) const {
  if (IsExcluded(prefs, source_exe)) {
    return PolicyVerdict::DeniedExcluded;
  }
  if (prefs.consent != OnlineConsent::Granted) {
    return PolicyVerdict::DeniedConsent;
  }
  if (IsBlank(text)) {
    return PolicyVerdict::DeniedEmptyText;
  }
  return PolicyVerdict::Allow;
}

}  // namespace cliplingo
