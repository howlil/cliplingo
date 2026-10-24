#include "core.h"
#include "popup.h"
#include <shellapi.h>
#include <shobjidl.h>
#include <commctrl.h>
#include <wrl/client.h>
#include <algorithm>
#include <condition_variable>
#include <chrono>
#include <filesystem>
#include <mutex>
#include <optional>
#include <stop_token>
#include <thread>
#include <memory>
#include <string>

using Microsoft::WRL::ComPtr;
namespace {
constexpr UINT kTrayMessage = WM_APP + 1;
constexpr UINT kResultMessage = WM_APP + 2;
constexpr UINT kTaskbarMessage = WM_APP + 3;
constexpr UINT_PTR kTrayId = 1;
constexpr int kHotkeyId = 10;
constexpr int kSidebar = 101;
constexpr int kChooseModel = 102;
constexpr int kApplySize = 103;
constexpr int kMaxWidthEdit = 104;
constexpr int kMaxHeightEdit = 105;
constexpr int kPauseButton = 106;
constexpr int kTrayPause = 201;
constexpr int kTraySettings = 202;
constexpr int kTrayQuit = 203;
constexpr int kTrayCapture = 204;
constexpr UINT kDismissTimer = 1;

struct JobResult {
  std::uint64_t generation{};
  POINT cursor{};
  Failure failure{Failure::None};
  std::wstring translated;
};

std::filesystem::path config_file() {
  wchar_t path[32768]{};
  const DWORD len = GetEnvironmentVariableW(L"LOCALAPPDATA", path, 32768);
  if (!len || len >= 32768) return {};
  return std::filesystem::path(path) / L"ClipLingo" / L"settings.ini";
}

std::optional<int> parse_positive_int(const std::wstring& input, int lo, int hi) {
  try {
    size_t used = 0;
    const auto value = std::stoi(input, &used);
    if (used != input.size() || value < lo || value > hi) return std::nullopt;
    return value;
  } catch (...) { return std::nullopt; }
}

HICON make_tray_icon() {
  constexpr int n = 32;
  unsigned char pixels[n * n * 4]{};
  unsigned char mask[n * n / 8]{}; // transparent mask bits zero: opaque ARGB shape
  for (int y = 0; y < n; ++y) for (int x = 0; x < n; ++x) {
    const bool in = x >= 2 && x < 30 && y >= 2 && y < 30;
    const bool corner = (x < 8 || x > 23) && (y < 8 || y > 23);
    const bool tile = in && (!corner || ((x < 8 ? x-8 : x-23)*(x < 8 ? x-8 : x-23) +
                   (y < 8 ? y-8 : y-23)*(y < 8 ? y-8 : y-23) <= 36));
    const bool mark = (x >= 11 && x <= 14 && y >= 9 && y <= 23) ||
                      (y >= 20 && y <= 23 && x >= 11 && x <= 23);
    const int offset = (y * n + x) * 4;
    if (tile) {
      pixels[offset + 0] = mark ? 255 : 255; // blue channel
      pixels[offset + 1] = mark ? 255 : 122; // green
      pixels[offset + 2] = mark ? 255 : 0;   // red
      pixels[offset + 3] = 255;
    }
  }
  return CreateIcon(GetModuleHandleW(nullptr), n, n, 1, 32, mask, pixels);
}
}

class App {
public:
  bool Start(HINSTANCE instance) {
    instance_ = instance;
    const auto cfg = config_file();
    if (!cfg.empty()) {
      wchar_t path[32768]{};
      GetPrivateProfileStringW(L"translation", L"model_path", L"", path, 32768, cfg.c_str());
      prefs_.model_path = path;
      prefs_.max_width_dip = std::clamp(static_cast<int>(GetPrivateProfileIntW(
         L"popup", L"max_width", 420, cfg.c_str())), 180, 1200);
      prefs_.max_height_dip = std::clamp(static_cast<int>(GetPrivateProfileIntW(
         L"popup", L"max_height", 300, cfg.c_str())), 80, 900);
    }
    WNDCLASSEXW wc{sizeof(wc)};
    wc.hInstance = instance;
    wc.lpfnWndProc = HostProc;
    wc.lpszClassName = L"ClipLingoHost";
    if (!RegisterClassExW(&wc)) return false;
    host_ = CreateWindowExW(0, wc.lpszClassName, L"", 0, 0, 0, 0, 0,
                           nullptr, nullptr, instance_, this);
    if (!host_) return false;
    taskbar_created_ = RegisterWindowMessageW(L"TaskbarCreated");
    // Hidden top-level host receives registered Explorer TaskbarCreated broadcast.
    if (!popup_.Create(instance)) return false;
    icon_ = make_tray_icon();
    AddTray();
    shortcut_registered_ = RegisterHotKey(host_, kHotkeyId, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, 'T') != 0;
    worker_ = std::jthread([this](std::stop_token stop) { Work(stop); });
    return true;
  }
  int Run() {
    MSG message{};
    while (GetMessageW(&message, nullptr, 0, 0) > 0) {
      if (settings_ && IsDialogMessageW(settings_, &message)) continue;
      TranslateMessage(&message);
      DispatchMessageW(&message);
    }
    return static_cast<int>(message.wParam);
  }
  ~App() {
    if (host_) {
      if (shortcut_registered_) UnregisterHotKey(host_, kHotkeyId);
      NOTIFYICONDATAW nid{sizeof(nid)}; nid.hWnd = host_; nid.uID = kTrayId;
      Shell_NotifyIconW(NIM_DELETE, &nid);
    }
    worker_.request_stop();
    jobs_cv_.notify_all();
    // Caution: join may wait for a blocked third-party UIA call. Address with process isolation after M1.
    if (worker_.joinable()) worker_.join();
    // The message pump has stopped, so reclaim any result messages queued by the worker.
    MSG undelivered{};
    while (host_ && PeekMessageW(&undelivered, host_, kResultMessage,
                                 kResultMessage, PM_REMOVE)) {
      delete reinterpret_cast<JobResult*>(undelivered.lParam);
    }
    if (settings_) DestroyWindow(settings_);
    if (popup_.handle()) DestroyWindow(popup_.handle());
    if (host_) DestroyWindow(host_);
    if (icon_) DestroyIcon(icon_);
    if (font_) DeleteObject(font_);
  }
private:
  static LRESULT CALLBACK HostProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    auto* self = reinterpret_cast<App*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    if (msg == WM_NCCREATE) {
      self = static_cast<App*>(reinterpret_cast<CREATESTRUCTW*>(lp)->lpCreateParams);
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    }
    if (self && msg == self->taskbar_created_ && self->taskbar_created_ != 0) {
      self->AddTray(); return 0;
    }
    if (msg == WM_NCCREATE) return TRUE;
    return self ? self->HostMessage(hwnd, msg, wp, lp) : DefWindowProcW(hwnd, msg, wp, lp);
  }
  LRESULT HostMessage(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    switch (msg) {
      case WM_HOTKEY: if (wp == kHotkeyId) BeginTranslation(); return 0;
      case kResultMessage: {
        std::unique_ptr<JobResult> answer(reinterpret_cast<JobResult*>(lp));
        if (answer && answer->generation == generation_ && !paused_) {
          failure_ = answer->failure;
          if (answer->failure == Failure::None && !answer->translated.empty()) {
            popup_.ShowText(answer->translated, answer->cursor, prefs_);
            KillTimer(host_, kDismissTimer);
            SetTimer(host_, kDismissTimer, 30000, nullptr);
          }
          RefreshSettings();
        }
        return 0;
      }
      case WM_TIMER: if (wp == kDismissTimer) { popup_.Hide(); KillTimer(host_, kDismissTimer); } return 0;
      case kTrayMessage: {
        const UINT event = LOWORD(lp);
        if (event == WM_RBUTTONUP || event == WM_CONTEXTMENU || event == NIN_KEYSELECT ||
            event == NIN_SELECT) OpenTrayMenu();
        return 0;
      }
      case WM_COMMAND: {
        switch (LOWORD(wp)) {
          case kTraySettings: ShowSettings(); break;
          case kTrayPause: paused_ = !paused_; ++generation_; popup_.Hide(); RefreshSettings(); break;
          case kTrayQuit: PostQuitMessage(0); break;
          case kTrayCapture: ShowSettings(1); break; // OCR is intentionally not implemented in M1
        }
        return 0;
      }
      case WM_DESTROY: return 0;
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
  }
  void AddTray() {
    NOTIFYICONDATAW nid{sizeof(nid)};
    nid.hWnd = host_; nid.uID = kTrayId;
    nid.uFlags = NIF_ICON | NIF_TIP | NIF_MESSAGE;
    nid.uCallbackMessage = kTrayMessage;
    nid.hIcon = icon_;
    wcscpy_s(nid.szTip, L"ClipLingo — Offline Translator");
    Shell_NotifyIconW(NIM_ADD, &nid);
    nid.uVersion = NOTIFYICON_VERSION_4;
    Shell_NotifyIconW(NIM_SETVERSION, &nid);
  }
  void OpenTrayMenu() {
    HMENU menu = CreatePopupMenu();
    if (!menu) return;
    AppendMenuW(menu, MF_STRING, kTrayPause, paused_ ? L"Resume" : L"Pause");
    AppendMenuW(menu, MF_STRING | MF_GRAYED, kTrayCapture, L"Capture screen text (later milestone)");
    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(menu, MF_STRING, kTraySettings, L"Settings...");
    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(menu, MF_STRING, kTrayQuit, L"Quit ClipLingo");
    POINT where{}; GetCursorPos(&where);
    SetForegroundWindow(host_);
    TrackPopupMenu(menu, TPM_RIGHTBUTTON, where.x, where.y, 0, host_, nullptr);
    PostMessageW(host_, WM_NULL, 0, 0);
    DestroyMenu(menu);
  }
  void BeginTranslation() {
    if (paused_ || !shortcut_registered_) return;
    popup_.Hide();
    KillTimer(host_, kDismissTimer);
    Request request{};
    request.generation = ++generation_;
    request.foreground = GetForegroundWindow();
    GetCursorPos(&request.cursor);
    {
      std::lock_guard lock(jobs_mutex_);
      pending_ = request;  // replace queued stale request; never unbounded backlog
      worker_prefs_ = prefs_;
    }
    jobs_cv_.notify_one();
  }
  void Work(std::stop_token stop) {
    const HRESULT com = CoInitializeEx(nullptr, COINIT_MULTITHREADED);
    if (FAILED(com)) return;
    while (!stop.stop_requested()) {
      Request current;
      Preferences settings;
      {
        std::unique_lock lock(jobs_mutex_);
        if (!jobs_cv_.wait_for(lock, stop, std::chrono::minutes(2),
              [this] { return pending_.has_value(); })) {
          if (stop.stop_requested()) break;
          lock.unlock();
          unload_translation_model(); // reclaim model memory after idle timeout
          continue;
        }
        current = *pending_;
        pending_.reset();
        settings = worker_prefs_;
      }
      auto result = std::make_unique<JobResult>();
      result->generation = current.generation;
      result->cursor = current.cursor;
      const auto captured = capture_selected_text(current.foreground);
      result->failure = captured.failure;
      if (captured) {
        const auto translated = translate_en_id(captured.text, settings.model_path);
        result->failure = translated.failure;
        if (translated) result->translated = translated.text;
      }
      // Post ownership to UI thread. No worker holds or mutates popup HWND.
      if (PostMessageW(host_, kResultMessage, 0, reinterpret_cast<LPARAM>(result.get())))
        result.release();
    }
    unload_translation_model();
    CoUninitialize();
  }

  void SaveSettings() {
    const auto path = config_file();
    if (path.empty()) return;
    std::error_code ec;
    std::filesystem::create_directories(path.parent_path(), ec);
    if (ec) { failure_ = Failure::CaptureFailed; return; }
    WritePrivateProfileStringW(L"translation", L"model_path", prefs_.model_path.c_str(), path.c_str());
    WritePrivateProfileStringW(L"popup", L"max_width", std::to_wstring(prefs_.max_width_dip).c_str(), path.c_str());
    WritePrivateProfileStringW(L"popup", L"max_height", std::to_wstring(prefs_.max_height_dip).c_str(), path.c_str());
  }

  static LRESULT CALLBACK SettingsProc(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    auto* self = reinterpret_cast<App*>(GetWindowLongPtrW(hwnd, GWLP_USERDATA));
    if (msg == WM_NCCREATE) {
      self = static_cast<App*>(reinterpret_cast<CREATESTRUCTW*>(lp)->lpCreateParams);
      SetWindowLongPtrW(hwnd, GWLP_USERDATA, reinterpret_cast<LONG_PTR>(self));
    }
    if (msg == WM_NCCREATE) return TRUE;
    if (self) return self->SettingsMessage(hwnd, msg, wp, lp);
    return DefWindowProcW(hwnd, msg, wp, lp);
  }
  LRESULT SettingsMessage(HWND hwnd, UINT msg, WPARAM wp, LPARAM lp) {
    if (msg == WM_CLOSE) { ShowWindow(hwnd, SW_HIDE); return 0; }
    if (msg == WM_COMMAND) {
      const int id = LOWORD(wp);
      if (id == kSidebar && HIWORD(wp) == LBN_SELCHANGE) { DrawSettingsSection(); return 0; }
      if (id == kChooseModel && HIWORD(wp) == BN_CLICKED) { ChooseModel(); return 0; }
      if (id == kPauseButton && HIWORD(wp) == BN_CLICKED) {
        paused_ = !paused_; ++generation_; popup_.Hide(); DrawSettingsSection(); return 0;
      }
      if (id == kApplySize && HIWORD(wp) == BN_CLICKED) {
        wchar_t width[32]{}, height[32]{};
        GetWindowTextW(width_edit_, width, 32); GetWindowTextW(height_edit_, height, 32);
        const auto w = parse_positive_int(width, 180, 1200);
        const auto h = parse_positive_int(height, 80, 900);
        if (!w || !h) { MessageBoxW(hwnd, L"Width must be 180–1200 and height 80–900.", L"ClipLingo", MB_ICONWARNING); return 0; }
        prefs_.max_width_dip = *w; prefs_.max_height_dip = *h;
        SaveSettings(); DrawSettingsSection(); return 0;
      }
    }
    if (msg == WM_CTLCOLORSTATIC) {
      SetBkMode(reinterpret_cast<HDC>(wp), TRANSPARENT);
      return reinterpret_cast<LRESULT>(GetStockObject(WHITE_BRUSH));
    }
    return DefWindowProcW(hwnd, msg, wp, lp);
  }
  void ShowSettings(int section = -1) {
    if (!settings_) {
      WNDCLASSEXW wc{sizeof(wc)};
      wc.hInstance = instance_;
      wc.lpfnWndProc = SettingsProc;
      wc.lpszClassName = L"ClipLingoSettings";
      wc.hbrBackground = reinterpret_cast<HBRUSH>(GetStockObject(WHITE_BRUSH));
      wc.hCursor = LoadCursorW(nullptr, IDC_ARROW);
      RegisterClassExW(&wc);
      settings_ = CreateWindowExW(WS_EX_CONTROLPARENT, wc.lpszClassName, L"ClipLingo Settings",
                  WS_OVERLAPPED | WS_CAPTION | WS_SYSMENU | WS_MINIMIZEBOX,
                  CW_USEDEFAULT, CW_USEDEFAULT, 848, 624, nullptr, nullptr, instance_, this);
      if (!settings_) return;
      font_ = CreateFontW(-17, 0, 0, 0, FW_NORMAL, FALSE, FALSE, FALSE, DEFAULT_CHARSET,
                 OUT_DEFAULT_PRECIS, CLIP_DEFAULT_PRECIS, CLEARTYPE_QUALITY,
                 DEFAULT_PITCH, L"Segoe UI Variable");
      sidebar_ = Control(L"LISTBOX", L"", WS_CHILD | WS_VISIBLE | WS_VSCROLL | LBS_NOTIFY | LBS_HASSTRINGS,
                         20, 35, 192, 498, kSidebar);
      for (const auto label : {L"General", L"Text Capture", L"Translation Popup", L"Appearance", L"Language Models", L"Privacy"})
        SendMessageW(sidebar_, LB_ADDSTRING, 0, reinterpret_cast<LPARAM>(label));
      SendMessageW(sidebar_, LB_SETCURSEL, 0, 0);
    }
    if (section >= 0) SendMessageW(sidebar_, LB_SETCURSEL, static_cast<WPARAM>(section), 0);
    DrawSettingsSection();
    ShowWindow(settings_, SW_SHOWNORMAL);
    SetForegroundWindow(settings_);
  }
  HWND Control(const wchar_t* cls, const wchar_t* text, DWORD style, int x, int y, int w, int h, int id = 0) {
    HWND hwnd = CreateWindowExW(0, cls, text, style, x, y, w, h,
                settings_, reinterpret_cast<HMENU>(static_cast<INT_PTR>(id)), instance_, nullptr);
    if (font_) SendMessageW(hwnd, WM_SETFONT, reinterpret_cast<WPARAM>(font_), TRUE);
    return hwnd;
  }
  void Label(const wchar_t* text, int y, int width = 560) {
    Control(L"STATIC", text, WS_CHILD | WS_VISIBLE | SS_LEFT, 246, y, width, 42);
  }
  void DrawSettingsSection() {
    if (!settings_ || !sidebar_) return;
    for (auto child = GetWindow(settings_, GW_CHILD); child;) {
      auto next = GetWindow(child, GW_HWNDNEXT);
      if (child != sidebar_) DestroyWindow(child);
      child = next;
    }
    width_edit_ = height_edit_ = nullptr;
    const int section = static_cast<int>(SendMessageW(sidebar_, LB_GETCURSEL, 0, 0));
    switch (section) {
      case 0:
        Label(L"General", 32); Label(L"ClipLingo runs from the Windows notification area.", 78);
        Label(shortcut_registered_ ? L"Shortcut: Ctrl + Alt + T  —  Registered" :
              L"Shortcut: Ctrl + Alt + T  —  Unavailable (conflict)", 126);
        Control(L"BUTTON", paused_ ? L"Resume translation" : L"Pause translation",
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 246, 193, 185, 33, kPauseButton);
        break;
      case 1:
        Label(L"Text Capture", 32);
        Label(L"M1 uses selection from Windows UI Automation text patterns.", 78);
        Label(L"Hover, clipboard fallback, and region OCR are not implemented in M1.", 125);
        break;
      case 2:
        Label(L"Translation Popup", 32);
        Label(L"Text only • above trigger cursor • automatic content sizing", 80);
        Label(L"Maximum width (DIPs)", 153, 240);
        width_edit_ = Control(L"EDIT", std::to_wstring(prefs_.max_width_dip).c_str(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER, 490, 148, 110, 30, kMaxWidthEdit);
        Label(L"Maximum height (DIPs)", 199, 240);
        height_edit_ = Control(L"EDIT", std::to_wstring(prefs_.max_height_dip).c_str(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER, 490, 194, 110, 30, kMaxHeightEdit);
        Control(L"BUTTON", L"Apply", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                490, 244, 110, 33, kApplySize);
        break;
      case 3:
        Label(L"Appearance", 32);
        Label(L"Native Windows typography: Segoe UI Variable.", 78);
        Label(L"M1 ships the approved Solid light surface. Glass/tinted/theme controls require native visual testing.", 124);
        break;
      case 4:
        Label(L"Language Models", 32);
        Label(L"Direct English → Indonesian; local CTranslate2 model + SentencePiece tokenizers.", 80);
        Label(prefs_.model_path.empty() ? L"Model: Not configured" : L"Model: Configured (validated on translate)", 136);
        Control(L"BUTTON", L"Choose converted model folder...", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                246, 193, 265, 33, kChooseModel);
        break;
      case 5:
        Label(L"Privacy", 32);
        Label(L"Selected text is processed locally. No translation history or content logs.", 80);
        Label(L"Do not use ClipLingo for elevated or protected application content.", 128);
        break;
    }
    const auto detail = failure_ == Failure::NoSelection ? L"Ready" : failure_label(failure_);
    Control(L"STATIC", (std::wstring(L"Last status: ") + detail).c_str(),
            WS_CHILD | WS_VISIBLE | SS_LEFT, 246, 475, 540, 48);
  }
  void ChooseModel() {
    ComPtr<IFileOpenDialog> picker;
    if (FAILED(CoCreateInstance(CLSID_FileOpenDialog, nullptr, CLSCTX_INPROC_SERVER,
                                IID_PPV_ARGS(picker.GetAddressOf())))) return;
    DWORD flags{};
    picker->GetOptions(&flags);
    picker->SetOptions(flags | FOS_PICKFOLDERS | FOS_FORCEFILESYSTEM);
    if (FAILED(picker->Show(settings_))) return;
    ComPtr<IShellItem> selected;
    if (FAILED(picker->GetResult(selected.GetAddressOf()))) return;
    PWSTR folder = nullptr;
    if (FAILED(selected->GetDisplayName(SIGDN_FILESYSPATH, &folder)) || !folder) return;
    std::filesystem::path candidate(folder);
    CoTaskMemFree(folder);
    std::error_code ec;
    if (!std::filesystem::is_regular_file(candidate / L"model.bin", ec) || ec ||
        !std::filesystem::is_regular_file(candidate / L"source.spm", ec) || ec ||
        !std::filesystem::is_regular_file(candidate / L"target.spm", ec) || ec) {
      MessageBoxW(settings_, L"Expected model.bin, source.spm, target.spm in the selected folder.",
                  L"Invalid model folder", MB_ICONWARNING);
      return;
    }
    prefs_.model_path = candidate;
    failure_ = Failure::None;
    SaveSettings(); DrawSettingsSection();
  }
  void RefreshSettings() { if (settings_ && IsWindowVisible(settings_)) DrawSettingsSection(); }

  HINSTANCE instance_{nullptr};
  HWND host_{nullptr}, settings_{nullptr}, sidebar_{nullptr}, width_edit_{nullptr}, height_edit_{nullptr};
  HFONT font_{nullptr}; HICON icon_{nullptr};
  UINT taskbar_created_{0};
  bool shortcut_registered_{false}, paused_{false};
  Failure failure_{Failure::None};
  Preferences prefs_, worker_prefs_;
  TextPopup popup_;
  std::uint64_t generation_{0};
  std::mutex jobs_mutex_;
  std::condition_variable_any jobs_cv_;
  std::optional<Request> pending_;
  std::jthread worker_;
};

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR, int) {
  SetProcessDpiAwarenessContext(DPI_AWARENESS_CONTEXT_PER_MONITOR_AWARE_V2);
  const HRESULT com = CoInitializeEx(nullptr, COINIT_APARTMENTTHREADED);
  if (FAILED(com)) return 1;
  int result = 1;
  {
    App app;
    if (app.Start(instance)) result = app.Run();
  }
  CoUninitialize();
  return result;
}