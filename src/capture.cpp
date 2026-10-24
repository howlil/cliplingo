#include "core.h"
#include <UIAutomation.h>
#include <wrl/client.h>
#include <algorithm>
#include <string>

using Microsoft::WRL::ComPtr;

CaptureResult capture_selected_text(HWND expected_foreground) {
  // Called only from the coordinator's COM MTA worker. No clipboard hijacking.
  if (!expected_foreground || GetForegroundWindow() != expected_foreground)
    return {Failure::NoSelection, {}};
  ComPtr<IUIAutomation> automation;
  HRESULT hr = CoCreateInstance(CLSID_CUIAutomation, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(automation.GetAddressOf()));
  if (FAILED(hr)) return {Failure::CaptureFailed, {}};
  ComPtr<IUIAutomationElement> focused;
  hr = automation->GetFocusedElement(focused.GetAddressOf());
  if (FAILED(hr) || !focused) return {Failure::Unsupported, {}};
  ComPtr<IUnknown> raw_pattern;
  hr = focused->GetCurrentPattern(UIA_TextPatternId, raw_pattern.GetAddressOf());
  if (FAILED(hr) || !raw_pattern) return {Failure::Unsupported, {}};
  ComPtr<IUIAutomationTextPattern> pattern;
  hr = raw_pattern.As(&pattern);
  if (FAILED(hr) || !pattern) return {Failure::Unsupported, {}};
  ComPtr<IUIAutomationTextRangeArray> ranges;
  hr = pattern->GetSelection(ranges.GetAddressOf());
  if (FAILED(hr) || !ranges) return {Failure::CaptureFailed, {}};
  int count = 0;
  if (FAILED(ranges->get_Length(&count)) || count <= 0)
    return {Failure::NoSelection, {}};
  if (count > 1) return {Failure::Unsupported, {}}; // never translate a partial multi-range selection
  ComPtr<IUIAutomationTextRange> selected;
  if (FAILED(ranges->GetElement(0, selected.GetAddressOf())) || !selected)
    return {Failure::CaptureFailed, {}};
  // Input cap prevents accidental enormous capture and protects the inference process.
  BSTR raw = nullptr;
  hr = selected->GetText(8193, &raw);
  if (FAILED(hr) || !raw) return {Failure::CaptureFailed, {}};
  std::wstring text(raw, SysStringLen(raw));
  SysFreeString(raw);
  if (text.size() > 8192) return {Failure::TooLong, {}};
  const auto first = text.find_first_not_of(L" \r\n\t\v\f");
  if (first == std::wstring::npos) return {Failure::NoSelection, {}};
  const auto last = text.find_last_not_of(L" \r\n\t\v\f");
  text = text.substr(first, last - first + 1);
  if (GetForegroundWindow() != expected_foreground) return {Failure::NoSelection, {}};
  return {Failure::None, std::move(text)};
}