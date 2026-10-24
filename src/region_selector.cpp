#include "region_selector.h"
#include "region_geometry.h"
#include <algorithm>
#include <windowsx.h>
#include <utility>

bool RegionSelector::Start(HINSTANCE instance, HWND restore_focus, Done done) {
  if (hwnd_ || !done) return false;
  WNDCLASSEXW wc{sizeof(wc)};
  wc.hInstance = instance;
  wc.lpfnWndProc = Proc;
  wc.lpszClassName = L"ClipLingoRegionSelector";
  wc.hCursor = LoadCursorW(nullptr, IDC_CROSS);
  if (!RegisterClassExW(&wc) && GetLastError() != ERROR_CLASS_ALREADY_EXISTS)
    return false;
  desktop_ = {GetSystemMetrics(SM_XVIRTUALSCREEN), GetSystemMetrics(SM_YVIRTUALSCREEN),
              GetSystemMetrics(SM_XVIRTUALSCREEN) + GetSystemMetrics(SM_CXVIRTUALSCREEN),
              GetSystemMetrics(SM_YVIRTUALSCREEN) + GetSystemMetrics(SM_CYVIRTUALSCREEN)};
  restore_focus_ = restore_focus;
  done_ = std::move(done);
  finished_ = dragging_ = false;
  hwnd_ = CreateWindowExW(WS_EX_TOPMOST | WS_EX_TOOLWINDOW | WS_EX_LAYERED,
                          wc.lpszClassName, L"Select text region", WS_POPUP,
                          desktop_.left, desktop_.top,
                          desktop_.right - desktop_.left, desktop_.bottom - desktop_.top,
                          nullptr, nullptr, instance, this);
  if (!hwnd_) { done_ = {}; return false; }
  SetLayeredWindowAttributes(hwnd_, 0, 105, LWA_ALPHA);
  ShowWindow(hwnd_, SW_SHOWNORMAL);
  SetForegroundWindow(hwnd_);  // temporary keyboard focus for Escape only
  return true;
}

void RegionSelector::Cancel() { if (hwnd_) Finish(false); }

LRESULT CALLBACK RegionSelector::Proc(HWND hwnd, UINT message, WPARAM wp, LPARAM lp) {
  auto* self = reinterpret_cast<RegionSelector*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
  if (message == WM_NCCREATE) {
    self = static_cast<RegionSelector*>(reinterpret_cast<CREATESTRUCTW*>(lp)->lpCreateParams);
    SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    return TRUE;
  }
  return self ? self->Handle(message, wp, lp) : DefWindowProcW(hwnd, message, wp, lp);
}

void RegionSelector::Finish(bool accepted) {
  if (!hwnd_ || finished_) return;
  finished_ = true;
  auto callback = std::move(done_);
  HWND to_destroy = hwnd_;
  hwnd_ = nullptr;
  if (GetCapture() == to_destroy) ReleaseCapture();
  ShowWindow(to_destroy, SW_HIDE);
  DestroyWindow(to_destroy);
  if (restore_focus_ && IsWindow(restore_focus_))
    SetForegroundWindow(restore_focus_);
  restore_focus_ = nullptr;
  if (callback) {
    if (accepted) {
      auto r = region_geometry::clamp(
          region_geometry::normalize(initial_.x, initial_.y, current_.x, current_.y),
          {desktop_.left, desktop_.top, desktop_.right, desktop_.bottom});
      if (region_geometry::valid(r)) {
        callback(RECT{r.left, r.top, r.right, r.bottom});
        return;
      }
    }
    callback(std::nullopt);
  }
}

LRESULT RegionSelector::Handle(UINT message, WPARAM wp, LPARAM lp) {
  switch (message) {
    case WM_ERASEBKGND: return 1;
    case WM_LBUTTONDOWN: {
      dragging_ = true;
      initial_ = {GET_X_LPARAM(lp) + desktop_.left, GET_Y_LPARAM(lp) + desktop_.top};
      current_ = initial_;
      SetCapture(hwnd_);
      InvalidateRect(hwnd_, nullptr, FALSE);
      return 0;
    }
    case WM_MOUSEMOVE:
      if (dragging_) {
        current_ = {GET_X_LPARAM(lp) + desktop_.left, GET_Y_LPARAM(lp) + desktop_.top};
        InvalidateRect(hwnd_, nullptr, FALSE);
      }
      return 0;
    case WM_LBUTTONUP:
      if (dragging_) {
        current_ = {GET_X_LPARAM(lp) + desktop_.left, GET_Y_LPARAM(lp) + desktop_.top};
        Finish(true);
      }
      return 0;
    case WM_CAPTURECHANGED: if (dragging_ && !finished_) Finish(false); return 0;
    case WM_RBUTTONDOWN: Finish(false); return 0;
    case WM_KEYDOWN: if (wp == VK_ESCAPE) Finish(false); return 0;
    case WM_PAINT: {
      PAINTSTRUCT ps{};
      HDC dc = BeginPaint(hwnd_, &ps);
      RECT client{};
      GetClientRect(hwnd_, &client);
      HBRUSH background = CreateSolidBrush(RGB(33, 43, 57));
      FillRect(dc, &client, background);
      DeleteObject(background);
      SetBkMode(dc, TRANSPARENT);
      SetTextColor(dc, RGB(255,255,255));
      RECT label{24, 16, 660, 58};
      DrawTextW(dc, L"Drag to select screen text   •   Esc to cancel", -1,
                &label, DT_SINGLELINE | DT_LEFT | DT_VCENTER);
      if (dragging_) {
        auto r = region_geometry::normalize(initial_.x, initial_.y, current_.x, current_.y);
        RECT local{r.left-desktop_.left, r.top-desktop_.top,
                   r.right-desktop_.left, r.bottom-desktop_.top};
        HBRUSH inside = CreateSolidBrush(RGB(70, 91, 118));
        FillRect(dc, &local, inside);
        DeleteObject(inside);
        HPEN pen = CreatePen(PS_SOLID, 3, RGB(0, 122, 255));
        auto old_pen = SelectObject(dc, pen);
        auto old_brush = SelectObject(dc, GetStockObject(HOLLOW_BRUSH));
        Rectangle(dc, local.left, local.top, local.right, local.bottom);
        SelectObject(dc, old_brush);
        SelectObject(dc, old_pen);
        DeleteObject(pen);
      }
      EndPaint(hwnd_, &ps);
      return 0;
    }
    case WM_NCDESTROY: return 0;
  }
  return DefWindowProcW(hwnd_, message, wp, lp);
}
