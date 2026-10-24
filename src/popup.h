#pragma once
#include "core.h"
#include <d2d1.h>
#include <dwrite.h>
#include <wrl/client.h>
#include <UIAutomation.h>

class TextPopup {
public:
  bool Create(HINSTANCE instance);
  void ShowText(const std::wstring& text, POINT anchor, const Preferences& prefs);
  void Hide();
  HWND handle() const { return hwnd_; }
  void ScrollBy(float delta_dip);
private:
  static LRESULT CALLBACK Proc(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);
  LRESULT Handle(HWND hwnd, UINT msg, WPARAM wparam, LPARAM lparam);
  void Paint();
  bool EnsureTarget();
  HWND hwnd_{nullptr};
  float dpi_{96.f};
  float width_dip_{0}, height_dip_{0}, scroll_dip_{0}, content_height_dip_{0};
  Microsoft::WRL::ComPtr<IDWriteFactory> writing_;
  Microsoft::WRL::ComPtr<IDWriteTextFormat> format_;
  Microsoft::WRL::ComPtr<IDWriteTextLayout> layout_;
  Microsoft::WRL::ComPtr<ID2D1Factory> drawing_;
  Microsoft::WRL::ComPtr<ID2D1HwndRenderTarget> target_;
  Microsoft::WRL::ComPtr<ID2D1SolidColorBrush> brush_;
  Microsoft::WRL::ComPtr<IRawElementProviderSimple> accessibility_;
};