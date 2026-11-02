#include <string>

#include "core/preferences/request_policy.hpp"
#include "support/test_check.hpp"

namespace {

using namespace cliplingo;

Preferences Granted() {
  Preferences prefs;
  prefs.consent = OnlineConsent::Granted;
  return prefs;
}

}  // namespace

CL_TEST(capture_is_local_and_needs_no_consent) {
  const RequestPolicy policy;
  Preferences prefs;
  prefs.consent = OnlineConsent::Unset;

  CL_CHECK(policy.EvaluateCapture(prefs, L"notepad.exe") == PolicyVerdict::Allow);
}

CL_TEST(capture_respects_exclusions) {
  const RequestPolicy policy;
  Preferences prefs = Granted();
  prefs.excluded_apps = {L"chrome.exe"};

  CL_CHECK(policy.EvaluateCapture(prefs, L"CHROME.EXE") == PolicyVerdict::DeniedExcluded);
  CL_CHECK(policy.EvaluateCapture(prefs, L"C:\\Windows\\System32\\notepad.exe") ==
           PolicyVerdict::Allow);
}

CL_TEST(unset_consent_denies_outbound) {
  const RequestPolicy policy;
  Preferences prefs;  // default Unset

  CL_CHECK(policy.EvaluateOutbound(prefs, L"notepad.exe", L"hello") ==
           PolicyVerdict::DeniedConsent);
}

CL_TEST(denied_consent_denies_outbound) {
  const RequestPolicy policy;
  Preferences prefs;
  prefs.consent = OnlineConsent::Denied;

  CL_CHECK(policy.EvaluateOutbound(prefs, L"notepad.exe", L"hello") ==
           PolicyVerdict::DeniedConsent);
}

CL_TEST(granted_consent_allows_outbound) {
  const RequestPolicy policy;
  const Preferences prefs = Granted();

  CL_CHECK(policy.EvaluateOutbound(prefs, L"notepad.exe", L"hello") == PolicyVerdict::Allow);
}

CL_TEST(exclusion_wins_over_consent_and_matches_basename) {
  const RequestPolicy policy;
  Preferences prefs = Granted();
  prefs.excluded_apps = {L"Code.exe"};

  CL_CHECK(policy.EvaluateOutbound(prefs, L"C:\\Program Files\\Microsoft VS Code\\CODE.EXE",
                                   L"hello") == PolicyVerdict::DeniedExcluded);
}

CL_TEST(blank_text_is_never_sent) {
  const RequestPolicy policy;
  const Preferences prefs = Granted();

  CL_CHECK(policy.EvaluateOutbound(prefs, L"notepad.exe", L"   \t\r\n ") ==
           PolicyVerdict::DeniedEmptyText);
}
