#include "core.h"
#include "scope_text.h"
#include <wrl/client.h>
#include <UIAutomation.h>
#include <oleauto.h>
#include <algorithm>
#include <string>

using Microsoft::WRL::ComPtr;

namespace {
constexpr int kMaxText = 8192;

CaptureResult text_from_range(IUIAutomationTextRange* range) {
  BSTR value = nullptr;
  if (!range || FAILED(range->GetText(kMaxText + 1, &value)) || !value)
    return {Failure::CaptureFailed, {}};
  std::wstring text(value, SysStringLen(value));
  SysFreeString(value);
  if (text.size() > kMaxText) return {Failure::TooLong, {}};
  text = scope_text::trim(text);
  return text.empty() ? CaptureResult{Failure::NoSelection, {}} :
                        CaptureResult{Failure::None, std::move(text)};
}

// The range must have visible bounds directly under the frozen pointer.
// UIA's RangeFromPoint by itself ALWAYS returns a nearest caret.
bool hit_test(IUIAutomationTextRange* word, POINT cursor) {
  SAFEARRAY* array = nullptr;
  if (!word || FAILED(word->GetBoundingRectangles(&array)) || !array) return false;
  const auto cleanup = [&] { SafeArrayDestroy(array); };
  if (SafeArrayGetDim(array) != 1) { cleanup(); return false; }
  LONG low = 0, high = -1;
  if (FAILED(SafeArrayGetLBound(array, 1, &low)) ||
      FAILED(SafeArrayGetUBound(array, 1, &high)) ||
      high < low || (high - low + 1) % 4 != 0 ||
      high - low + 1 > 4096) { cleanup(); return false; }
  double* rectangles = nullptr;
  if (FAILED(SafeArrayAccessData(array, reinterpret_cast<void**>(&rectangles)))) {
    cleanup(); return false;
  }
  bool found = false;
  for (LONG i = 0; i + 3 < high - low + 1; i += 4) {
    const scope_text::Rect bounds{rectangles[i], rectangles[i+1],
                                  rectangles[i+2], rectangles[i+3]};
    if (scope_text::contains(bounds, cursor.x, cursor.y)) { found = true; break; }
  }
  SafeArrayUnaccessData(array);
  cleanup();
  return found;
}

} // namespace

CaptureResult capture_hover_text(HWND expected_foreground, POINT cursor, CaptureScope scope) {
  if (scope == CaptureScope::SelectionOnly) return {Failure::NoSelection, {}};
  if (!expected_foreground || GetForegroundWindow() != expected_foreground)
    return {Failure::NoSelection, {}};

  // Exclude a different application's floating window, tooltip or toolbar.
  HWND pointed_window = WindowFromPoint(cursor);
  if (!pointed_window ||
      GetAncestor(pointed_window, GA_ROOT) != GetAncestor(expected_foreground, GA_ROOT))
    return {Failure::NoSelection, {}};

  ComPtr<IUIAutomation> automation;
  if (FAILED(CoCreateInstance(CLSID_CUIAutomation, nullptr, CLSCTX_INPROC_SERVER,
                              IID_PPV_ARGS(automation.GetAddressOf()))))
    return {Failure::CaptureFailed, {}};
  ComPtr<IUIAutomationElement> element;
  if (FAILED(automation->ElementFromPoint(cursor, element.GetAddressOf())) || !element)
    return {Failure::Unsupported, {}};

  // Chrome/editor providers often expose TextPattern on a document/container
  // parent, while ElementFromPoint returns a leaf text element.
  ComPtr<IUIAutomationTextPattern> pattern;
  ComPtr<IUIAutomationTreeWalker> walker;
  if (FAILED(automation->get_ControlViewWalker(walker.GetAddressOf())) || !walker)
    return {Failure::Unsupported, {}};
  for (int depth = 0; depth < 8 && element; ++depth) {
    ComPtr<IUnknown> raw;
    if (SUCCEEDED(element->GetCurrentPattern(UIA_TextPatternId, raw.GetAddressOf())) && raw &&
        SUCCEEDED(raw.As(&pattern)) && pattern)
      break;
    ComPtr<IUIAutomationElement> parent;
    if (FAILED(walker->GetParentElement(element.Get(), parent.GetAddressOf())) || !parent)
      break;
    element = std::move(parent);
  }
  if (!pattern) return {Failure::Unsupported, {}};

  ComPtr<IUIAutomationTextRange> caret;
  if (FAILED(pattern->RangeFromPoint(cursor, caret.GetAddressOf())) || !caret)
    return {Failure::Unsupported, {}};
  ComPtr<IUIAutomationTextRange> word;
  if (FAILED(caret->Clone(word.GetAddressOf())) || !word ||
      FAILED(word->ExpandToEnclosingUnit(TextUnit_Word)))
    return {Failure::Unsupported, {}};

  // First inspect the single word; bounding rectangles for a whole paragraph
  // could accept blank areas far away from the exact hover target.
  if (!hit_test(word.Get(), cursor)) return {Failure::NoSelection, {}};
  const auto hovered_word = text_from_range(word.Get());
  if (!hovered_word) return hovered_word;

  if (scope == CaptureScope::Word) return hovered_word;
  if (scope != CaptureScope::Sentence && scope != CaptureScope::Paragraph)
    return {Failure::Unsupported, {}};

  ComPtr<IUIAutomationTextRange> paragraph;
  if (FAILED(caret->Clone(paragraph.GetAddressOf())) || !paragraph ||
      FAILED(paragraph->ExpandToEnclosingUnit(TextUnit_Paragraph)))
    return {Failure::Unsupported, {}};

  const auto paragraph_text = text_from_range(paragraph.Get());
  if (!paragraph_text) return paragraph_text;
  if (scope == CaptureScope::Paragraph) {
    if (GetForegroundWindow() != expected_foreground) return {Failure::NoSelection, {}};
    return paragraph_text;
  }

  // Read text from the paragraph start to the *insertion point* to find a
  // sentence offset. UIA has Word and Paragraph units but no Sentence unit.
  ComPtr<IUIAutomationTextRange> prefix;
  if (FAILED(paragraph->Clone(prefix.GetAddressOf())) || !prefix ||
      FAILED(prefix->MoveEndpointByRange(TextPatternRangeEndpoint_End, caret.Get(),
                                         TextPatternRangeEndpoint_Start)))
    return {Failure::CaptureFailed, {}};
  BSTR prefix_value = nullptr;
  if (FAILED(prefix->GetText(kMaxText + 1, &prefix_value)) || !prefix_value)
    return {Failure::CaptureFailed, {}};
  const auto offset = static_cast<std::size_t>(SysStringLen(prefix_value));
  SysFreeString(prefix_value);
  // Fetch untrimmed paragraph for stable UIA caret offsets; normalization is
  // done only after segmentation, preserving leading whitespace positions.
  const auto original = [&]() -> std::wstring {
    BSTR all = nullptr;
    if (FAILED(paragraph->GetText(kMaxText + 1, &all)) || !all) return {};
    std::wstring out(all, SysStringLen(all));
    SysFreeString(all);
    return out;
  }();
  if (original.empty() || original.size() > kMaxText || offset > original.size())
    return {Failure::CaptureFailed, {}};
  std::wstring sentence = scope_text::sentence_at(original, offset);
  if (sentence.empty()) return {Failure::NoSelection, {}};
  if (GetForegroundWindow() != expected_foreground) return {Failure::NoSelection, {}};
  return {Failure::None, std::move(sentence)};
}

CaptureResult capture_with_hover(HWND foreground, POINT cursor, CaptureScope scope) {
  const auto selected = capture_selected_text(foreground);
  if (selected || scope == CaptureScope::SelectionOnly) return selected;
  // Real selected text must never be overridden by hover. Fatal, invalid or
  // oversized selection failures must not turn into another text target.
  if (selected.failure != Failure::NoSelection) return selected;
  return capture_hover_text(foreground, cursor, scope);
}
