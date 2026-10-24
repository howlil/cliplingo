#pragma once
#include <windows.h>
#include <functional>
#include <optional>

// Transient, user-initiated native selection surface. No screenshot or OCR
// allocation while idle. Owns only its temporary HWND and pointer interaction.
class RegionSelector {
 public:
  using Done = std::function<void(std::optional<RECT>)>;
  bool Start(HINSTANCE instance, HWND restore_focus, Done done);
  void Cancel();
  bool active() const { return hwnd_ != nullptr; }
 private:
  static LRESULT CALLBACK Proc(HWND hwnd, UINT message, WPARAM wp, LPARAM lp);
  LRESULT Handle(UINT message, WPARAM wp, LPARAM lp);
  void Finish(bool accepted);
  HWND hwnd_{nullptr};
  HWND restore_focus_{nullptr};
  RECT desktop_{}, selected_{};
  POINT initial_{}, current_{};
  bool dragging_{false}, finished_{false};
  Done done_;
};
