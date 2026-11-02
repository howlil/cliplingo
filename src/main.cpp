#include <windows.h>

namespace {
constexpr wchar_t kWindowClass[] = L"ClipLingo.MainWindow";
constexpr wchar_t kWindowTitle[] = L"ClipLingo";

LRESULT CALLBACK WindowProc(HWND window, UINT message, WPARAM wparam,
                            LPARAM lparam) {
  switch (message) {
    case WM_PAINT: {
      PAINTSTRUCT paint{};
      HDC dc = BeginPaint(window, &paint);
      RECT area{};
      GetClientRect(window, &area);
      area.left += 32;
      area.top += 32;
      area.right -= 32;
      area.bottom -= 32;
      SetBkMode(dc, TRANSPARENT);
      HGDIOBJ old_font = SelectObject(dc, GetStockObject(DEFAULT_GUI_FONT));
      constexpr wchar_t text[] =
          L"ClipLingo native shell\n\n"
          L"Project initialized. Translation and capture are not implemented.\n"
          L"UI reference: design/reference/cliplingo-v6.html";
      DrawTextW(dc, text, -1, &area, DT_TOP | DT_LEFT | DT_WORDBREAK);
      SelectObject(dc, old_font);
      EndPaint(window, &paint);
      return 0;
    }
    case WM_DESTROY:
      PostQuitMessage(0);
      return 0;
    default:
      return DefWindowProcW(window, message, wparam, lparam);
  }
}
}  // namespace

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR, int show_command) {
  WNDCLASSEXW window_class{};
  window_class.cbSize = sizeof(window_class);
  window_class.lpfnWndProc = WindowProc;
  window_class.hInstance = instance;
  window_class.hCursor = LoadCursorW(nullptr, IDC_ARROW);
  window_class.hbrBackground = reinterpret_cast<HBRUSH>(COLOR_WINDOW + 1);
  window_class.lpszClassName = kWindowClass;

  if (!RegisterClassExW(&window_class)) {
    return 1;
  }

  HWND window = CreateWindowExW(0, kWindowClass, kWindowTitle,
                               WS_OVERLAPPEDWINDOW, CW_USEDEFAULT,
                               CW_USEDEFAULT, 880, 520, nullptr, nullptr,
                               instance, nullptr);
  if (!window) {
    return 1;
  }

  ShowWindow(window, show_command);
  UpdateWindow(window);

  MSG message{};
  BOOL status;
  while ((status = GetMessageW(&message, nullptr, 0, 0)) > 0) {
    TranslateMessage(&message);
    DispatchMessageW(&message);
  }
  return status == -1 ? 1 : static_cast<int>(message.wParam);
}
