#include "popup.h"
#include "popup_geometry.h"
#include <d2d1helper.h>
#include <dwmapi.h>
#include <shellscalingapi.h>
#include <algorithm>
#include <cmath>
#include <memory>
#include <atomic>
#include <UIAutomationCoreApi.h>
#include <oleauto.h>

using Microsoft::WRL::ComPtr;
namespace {
// Non-activating UIA live-region provider. Exposes full text as an accessible
// name so long paragraphs aren't limited to the visible DirectWrite clip.
class PopupAccessibility final : public IRawElementProviderSimple {
 public:
  explicit PopupAccessibility(HWND hwnd) : hwnd_(hwnd) {}
  HRESULT STDMETHODCALLTYPE QueryInterface(REFIID iid, void** out) override {
    if (!out) return E_POINTER;
    *out = nullptr;
    if (iid == __uuidof(IUnknown) || iid == __uuidof(IRawElementProviderSimple)) {
      *out = static_cast<IRawElementProviderSimple*>(this);
      AddRef(); return S_OK;
    }
    return E_NOINTERFACE;
  }
  ULONG STDMETHODCALLTYPE AddRef() override { return ++refs_; }
  ULONG STDMETHODCALLTYPE Release() override {
    const auto refs = --refs_;
    if (!refs) delete this;
    return refs;
  }
  HRESULT STDMETHODCALLTYPE get_ProviderOptions(ProviderOptions* out) override {
    if (!out) return E_POINTER;
    *out = ProviderOptions_ServerSideProvider;
    return S_OK;
  }
  HRESULT STDMETHODCALLTYPE GetPatternProvider(PATTERNID, IUnknown** out) override {
    if (!out) return E_POINTER;
    *out = nullptr;
    return S_OK;
  }
  HRESULT STDMETHODCALLTYPE GetPropertyValue(PROPERTYID id, VARIANT* out) override {
    if (!out) return E_POINTER;
    VariantInit(out);
    if (id == UIA_ControlTypePropertyId) {
      out->vt = VT_I4; out->lVal = UIA_TextControlTypeId;
    } else if (id == UIA_NamePropertyId) {
      if (!IsWindow(hwnd_)) return UIA_E_ELEMENTNOTAVAILABLE;
      const auto count = GetWindowTextLengthW(hwnd_);
      std::wstring text(static_cast<size_t>(count) + 1, L'\0');
      GetWindowTextW(hwnd_, text.data(), count + 1);
      out->vt = VT_BSTR;
      out->bstrVal = SysAllocStringLen(text.data(), static_cast<UINT>(count));
      if (!out->bstrVal) return E_OUTOFMEMORY;
    } else if (id == UIA_LiveSettingPropertyId) {
      out->vt = VT_I4; out->lVal = 2; // Assertive
    } else if (id == UIA_IsContentElementPropertyId || id == UIA_IsControlElementPropertyId) {
      out->vt = VT_BOOL; out->boolVal = VARIANT_TRUE;
    }
    return S_OK;
  }
  HRESULT STDMETHODCALLTYPE get_HostRawElementProvider(IRawElementProviderSimple** out) override {
    if (!out) return E_POINTER;
    *out = nullptr;
    return IsWindow(hwnd_) ? UiaHostProviderFromHwnd(hwnd_, out) : UIA_E_ELEMENTNOTAVAILABLE;
  }
 private:
  ~PopupAccessibility() = default;
  std::atomic<ULONG> refs_{1};
  HWND hwnd_;
};
constexpr float kPad = 14.f;
constexpr float kGap = 12.f;
constexpr float kFont = 15.f;
int Pixels(float dip, float dpi) { return std::max(1, static_cast<int>(std::ceil(dip * dpi / 96.f))); }
float Dip(int pixels, float dpi) { return pixels * 96.f / dpi; }
}

bool TextPopup::Create(HINSTANCE instance) {
  if (FAILED(DWriteCreateFactory(DWRITE_FACTORY_TYPE_SHARED, __uuidof(IDWriteFactory),
                                  reinterpret_cast<IUnknown**>(writing_.GetAddressOf())))) return false;
  if (FAILED(D2D1CreateFactory(D2D1_FACTORY_TYPE_SINGLE_THREADED, drawing_.GetAddressOf()))) return false;
  if (FAILED(writing_->CreateTextFormat(L"Segoe UI Variable", nullptr, DWRITE_FONT_WEIGHT_NORMAL,
      DWRITE_FONT_STYLE_NORMAL, DWRITE_FONT_STRETCH_NORMAL, kFont, L"id-ID", format_.GetAddressOf())))
    return false;
  format_->SetWordWrapping(DWRITE_WORD_WRAPPING_WRAP);
  WNDCLASSEXW wc{sizeof(wc)};
  wc.hInstance = instance;
  wc.lpfnWndProc = TextPopup::Proc;
  wc.lpszClassName = L"ClipLingoTextPopup";
  wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
  wc.hbrBackground = nullptr;
  if (!RegisterClassExW(&wc) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS) return false;
  hwnd_ = CreateWindowExW(WS_EX_NOACTIVATE | WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
    wc.lpszClassName, L"", WS_POPUP, 0, 0, 1, 1, nullptr, nullptr, instance, this);
  if (!hwnd_) return false;
  accessibility_.Attach(new PopupAccessibility(hwnd_));
  const DWM_WINDOW_CORNER_PREFERENCE corners = DWMWCP_ROUND;
  DwmSetWindowAttribute(hwnd_, DWMWA_WINDOW_CORNER_PREFERENCE, &corners, sizeof(corners));
  return true;
}

void TextPopup::ShowText(const std::wstring& text, POINT anchor, const Preferences& prefs) {
  if (!hwnd_ || text.empty()) return;
  SetWindowTextW(hwnd_, text.c_str());
  HIGHCONTRASTW contrast{sizeof(contrast)};
  SystemParametersInfoW(SPI_GETHIGHCONTRAST, sizeof(contrast), &contrast, 0);
  if (contrast.dwFlags & HCF_HIGHCONTRASTON) {
    const COLORREF bg = GetSysColor(COLOR_WINDOW), fg = GetSysColor(COLOR_WINDOWTEXT);
    background_ = D2D1::ColorF(GetRValue(bg) / 255.f, GetGValue(bg) / 255.f, GetBValue(bg) / 255.f);
    foreground_ = D2D1::ColorF(GetRValue(fg) / 255.f, GetGValue(fg) / 255.f, GetBValue(fg) / 255.f);
  } else {
    const bool dark = prefs.theme == 1;
    background_ = D2D1::ColorF(dark ? (prefs.popup_style == 1 ? 0x173451 : 0x202631) :
                               (prefs.popup_style == 1 ? 0xE8F2FF : 0xFFFFFF));
    foreground_ = D2D1::ColorF(dark ? 0xF5F7FC : 0x1D2430);
  }
  const HMONITOR monitor = MonitorFromPoint(anchor, MONITOR_DEFAULTTONEAREST);
  MONITORINFO monitor_info{sizeof(monitor_info)};
  if (!GetMonitorInfoW(monitor, &monitor_info)) return;
  UINT dpi_x = 96, dpi_y = 96;
  if (FAILED(GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI, &dpi_x, &dpi_y))) dpi_x = dpi_y = 96;
  dpi_ = static_cast<float>(dpi_x);
  const RECT work = monitor_info.rcWork;
  const float work_width = Dip(work.right - work.left, dpi_);
  const float work_height = Dip(work.bottom - work.top, dpi_);
  const float max_width = std::clamp(static_cast<float>(prefs.max_width_dip), 160.f,
                                    std::max(160.f, work_width - 16.f));
  const float max_height = std::clamp(static_cast<float>(prefs.max_height_dip), 70.f,
                                     std::max(70.f, work_height - 16.f));
  layout_.Reset();
  if (FAILED(writing_->CreateTextLayout(text.c_str(), static_cast<UINT32>(text.size()),
       format_.Get(), 32768.f, 32768.f, layout_.GetAddressOf()))) return;
  DWRITE_TEXT_METRICS natural{};
  if (FAILED(layout_->GetMetrics(&natural))) return;
  width_dip_ = std::max(2.f * kPad + 1.f,
              std::min(max_width, natural.widthIncludingTrailingWhitespace + 2.f * kPad));
  layout_.Reset();
  if (FAILED(writing_->CreateTextLayout(text.c_str(), static_cast<UINT32>(text.size()),
         format_.Get(), std::max(1.f, width_dip_ - 2.f * kPad), 32768.f, layout_.GetAddressOf()))) return;
  DWRITE_TEXT_METRICS wrapped{};
  if (FAILED(layout_->GetMetrics(&wrapped))) return;
  content_height_dip_ = wrapped.height;
  height_dip_ = std::min(max_height, content_height_dip_ + 2.f * kPad);
  scroll_dip_ = 0;
  const int width_px = Pixels(width_dip_, dpi_);
  const int height_px = Pixels(height_dip_, dpi_);
  const int gap_px = Pixels(kGap, dpi_);
  const auto placed = popup_geometry::place(
      {work.left, work.top, work.right, work.bottom},
      {anchor.x, anchor.y}, width_px, height_px, gap_px);
  // Hwnd render target works in per-monitor DPI-adjusted DIPs.
  if (target_) {
    target_->SetDpi(dpi_, dpi_);
    target_->Resize(D2D1::SizeU(static_cast<UINT32>(width_px), static_cast<UINT32>(height_px)));
  }
  SetWindowPos(hwnd_, HWND_TOPMOST, placed.left, placed.top, width_px, height_px,
               SWP_NOACTIVATE | SWP_NOOWNERZORDER | SWP_SHOWWINDOW);
  InvalidateRect(hwnd_, nullptr, FALSE);
  if (accessibility_) UiaRaiseAutomationEvent(accessibility_.Get(), UIA_LiveRegionChangedEventId);
}

void TextPopup::ScrollBy(float delta_dip) {
  if (!hwnd_ || !IsWindowVisible(hwnd_)) return;
  const float max_scroll = std::max(0.f, content_height_dip_ - (height_dip_ - 2.f * kPad));
  scroll_dip_ = std::clamp(scroll_dip_ + delta_dip, 0.f, max_scroll);
  InvalidateRect(hwnd_, nullptr, FALSE);
}

void TextPopup::Hide() {
  if (hwnd_) ShowWindow(hwnd_, SW_HIDE);
}

bool TextPopup::EnsureTarget() {
  if (target_) return true;
  if (!hwnd_) return false;
  RECT rect{};
  GetClientRect(hwnd_, &rect);
  const auto size = D2D1::SizeU(static_cast<UINT32>(std::max(1L, rect.right)),
                                static_cast<UINT32>(std::max(1L, rect.bottom)));
  const auto props = D2D1::RenderTargetProperties(D2D1_RENDER_TARGET_TYPE_DEFAULT,
       D2D1::PixelFormat(DXGI_FORMAT_UNKNOWN, D2D1_ALPHA_MODE_IGNORE), dpi_, dpi_);
  if (FAILED(drawing_->CreateHwndRenderTarget(props, D2D1::HwndRenderTargetProperties(hwnd_, size),
                                         target_.GetAddressOf()))) return false;
  return SUCCEEDED(target_->CreateSolidColorBrush(D2D1::ColorF(0x1D2430), brush_.GetAddressOf()));
}

void TextPopup::Paint() {
  if (!EnsureTarget()) return;
  target_->BeginDraw();
  target_->Clear(background_);
  if (brush_) brush_->SetColor(foreground_);
  if (layout_) {
    target_->PushAxisAlignedClip(D2D1::RectF(kPad, kPad, width_dip_ - kPad,
                            height_dip_ - kPad), D2D1_ANTIALIAS_MODE_PER_PRIMITIVE);
    target_->DrawTextLayout(D2D1::Point2F(kPad, kPad - scroll_dip_), layout_.Get(),
                            brush_.Get(), D2D1_DRAW_TEXT_OPTIONS_CLIP);
    target_->PopAxisAlignedClip();
  }
  const auto hr = target_->EndDraw();
  if (hr == D2DERR_RECREATE_TARGET) { brush_.Reset(); target_.Reset(); InvalidateRect(hwnd_, nullptr, FALSE); }
}

LRESULT CALLBACK TextPopup::Proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
  auto* self = reinterpret_cast<TextPopup*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
  if (msg == WM_NCCREATE) {
    self = static_cast<TextPopup*>(reinterpret_cast<CREATESTRUCTW*>(lparam)->lpCreateParams);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
  }
  if (msg == WM_NCCREATE) return TRUE;
  return self ? self->Handle(hwnd, msg, wparam, lparam) : DefWindowProcW(hwnd, msg, wparam, lparam);
}

LRESULT TextPopup::Handle(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam) {
  switch (msg) {
    case WM_MOUSEACTIVATE: return MA_NOACTIVATE;
    case WM_GETOBJECT:
      if (static_cast<LONG>(lparam) == UiaRootObjectId && accessibility_)
        return UiaReturnRawElementProvider(hwnd, wparam, lparam, accessibility_.Get());
      break;
    case WM_ERASEBKGND: return 1;
    case WM_MOUSEWHEEL: {
      ScrollBy(-GET_WHEEL_DELTA_WPARAM(wparam) / 120.f * 32.f);
      return 0;
    }
    case WM_PAINT: {
      PAINTSTRUCT ps{};
      BeginPaint(hwnd_, &ps);
      Paint();
      EndPaint(hwnd_, &ps);
      return 0;
    }
    case WM_SIZE: {
      if (target_) {
        target_->Resize(D2D1::SizeU(LOWORD(lparam), HIWORD(lparam)));
      }
      return 0;
    }
    case WM_DESTROY: { UiaReturnRawElementProvider(hwnd, 0, 0, nullptr); accessibility_.Reset(); brush_.Reset(); target_.Reset(); layout_.Reset(); hwnd_ = nullptr; return 0; }
  }
  return DefWindowProcW(hwnd, msg, wparam, lparam);
}