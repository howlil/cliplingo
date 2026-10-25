#include "core.h"
#include "popup.h"
#include "region_selector.h"
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
constexpr UINT kModelInstalledMessage = WM_APP + 4;
constexpr UINT_PTR kTrayId = 1;
constexpr int kHotkeyId = 10;
constexpr int kScrollUpHotkeyId = 11;
constexpr int kScrollDownHotkeyId = 12;
constexpr int kSidebar = 101;
constexpr int kChooseModel = 102;
constexpr int kApplySize = 103;
constexpr int kMaxWidthEdit = 104;
constexpr int kMaxHeightEdit = 105;
constexpr int kPauseButton = 106;
constexpr int kStartupToggle = 107;
constexpr int kThemeCombo = 108;
constexpr int kSurfaceCombo = 109;
constexpr int kPreviewPopup = 110;
constexpr int kHoverEnabled = 111;
constexpr int kHoverScope = 112;
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

bool startup_enabled() {
  HKEY key = nullptr;
  if (RegOpenKeyExW(HKEY_CURRENT_USER,
      L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, KEY_QUERY_VALUE, &key) != ERROR_SUCCESS)
    return false;
  DWORD size = 0;
  const auto status = RegQueryValueExW(key, L"ClipLingo", nullptr, nullptr, nullptr, &size);
  RegCloseKey(key);
  return status == ERROR_SUCCESS;
}
bool set_startup(bool enabled) {
  HKEY key = nullptr;
  if (RegCreateKeyExW(HKEY_CURRENT_USER,
      L"Software\\Microsoft\\Windows\\CurrentVersion\\Run", 0, nullptr, 0,
      KEY_SET_VALUE, nullptr, &key, nullptr) != ERROR_SUCCESS) return false;
  LONG code = ERROR_SUCCESS;
  if (enabled) {
    wchar_t exe[MAX_PATH]{};
    const auto n = GetModuleFileNameW(nullptr, exe, MAX_PATH);
    if (!n || n >= MAX_PATH) code = ERROR_INVALID_PARAMETER;
    else {
      const std::wstring command = L"\"" + std::wstring(exe) + L"\"";
      code = RegSetValueExW(key, L"ClipLingo", 0, REG_SZ,
          reinterpret_cast<const BYTE*>(command.c_str()),
          static_cast<DWORD>((command.size() + 1) * sizeof(wchar_t)));
    }
  } else {
    code = RegDeleteValueW(key, L"ClipLingo");
    if (code == ERROR_FILE_NOT_FOUND) code = ERROR_SUCCESS;
  }
  RegCloseKey(key);
  return code == ERROR_SUCCESS;
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
    single_instance_ = CreateMutexW(nullptr, TRUE, L"Local\\ClipLingoWindowsNativeM1");
    if (!single_instance_ || GetLastError() == ERROR_ALREADY_EXISTS) return false;
    const auto cfg = config_file();
    if (!cfg.empty()) {
      wchar_t path[32768]{};
      GetPrivateProfileStringW(L"translation", L"model_path", L"", path, 32768, cfg.c_str());
      prefs_.model_path = path;
      // Do not hash the whole model on every app startup; integrity is
      // checked on import and on first translation model load.
      if (!prefs_.model_path.empty() &&
          !std::filesystem::exists(prefs_.model_path / L"cliplingo-model.sha256")) {
        prefs_.model_path.clear();
        failure_ = Failure::ModelInvalid;
      }
      prefs_.max_width_dip = std::clamp(static_cast<int>(GetPrivateProfileIntW(
         L"popup", L"max_width", 420, cfg.c_str())), 180, 1200);
      prefs_.max_height_dip = std::clamp(static_cast<int>(GetPrivateProfileIntW(
         L"popup", L"max_height", 300, cfg.c_str())), 80, 900);
      prefs_.theme = std::clamp(static_cast<int>(GetPrivateProfileIntW(L"appearance", L"theme", 0, cfg.c_str())), 0, 1);
      prefs_.popup_style = std::clamp(static_cast<int>(GetPrivateProfileIntW(L"appearance", L"surface", 0, cfg.c_str())), 0, 1);
      prefs_.hover_enabled = GetPrivateProfileIntW(L"capture", L"hover_enabled", 1, cfg.c_str()) != 0;
      prefs_.hover_scope = std::clamp(static_cast<int>(GetPrivateProfileIntW(
          L"capture", L"hover_scope", 0, cfg.c_str())), 0, 2);
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
    scroll_up_registered_ = RegisterHotKey(host_, kScrollUpHotkeyId, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_UP) != 0;
    scroll_down_registered_ = RegisterHotKey(host_, kScrollDownHotkeyId, MOD_CONTROL | MOD_ALT | MOD_NOREPEAT, VK_DOWN) != 0;
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
    selector_.Cancel();
    if (host_) {
      if (shortcut_registered_) UnregisterHotKey(host_, kHotkeyId);
      if (scroll_up_registered_) UnregisterHotKey(host_, kScrollUpHotkeyId);
      if (scroll_down_registered_) UnregisterHotKey(host_, kScrollDownHotkeyId);
      NOTIFYICONDATAW nid{sizeof(nid)}; nid.hWnd = host_; nid.uID = kTrayId;
      Shell_NotifyIconW(NIM_DELETE, &nid);
    }
    worker_.request_stop();
    jobs_cv_.notify_all();
    if (installer_.joinable()) installer_.join();
    // Caution: join may wait for a blocked third-party UIA call. Address with process isolation after M1.
    if (worker_.joinable()) worker_.join();
    // The message pump has stopped, so reclaim any result messages queued by the worker.
    MSG undelivered{};
    while (host_ && PeekMessageW(&undelivered, host_, kResultMessage,
                                 kResultMessage, PM_REMOVE)) {
      delete reinterpret_cast<JobResult*>(undelivered.lParam);
    }
    while (host_ && PeekMessageW(&undelivered, host_, kModelInstalledMessage,
                                 kModelInstalledMessage, PM_REMOVE)) {
      delete reinterpret_cast<ModelInstallResult*>(undelivered.lParam);
    }
    if (settings_) DestroyWindow(settings_);
    if (popup_.handle()) DestroyWindow(popup_.handle());
    if (host_) DestroyWindow(host_);
    if (icon_) DestroyIcon(icon_);
    if (font_) DeleteObject(font_);
    if (sidebar_brush_) DeleteObject(sidebar_brush_);
    if (single_instance_) CloseHandle(single_instance_);
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
      case WM_HOTKEY:
        if (wp == kHotkeyId) BeginTranslation();
        else if (wp == kScrollUpHotkeyId) popup_.ScrollBy(-64.f);
        else if (wp == kScrollDownHotkeyId) popup_.ScrollBy(64.f);
        return 0;
      case kModelInstalledMessage: {
        std::unique_ptr<ModelInstallResult> installed(reinterpret_cast<ModelInstallResult*>(lp));
        installing_ = false;
        if (installed && installed->success) {
          prefs_.model_path = installed->path;
          failure_ = Failure::None;
          SaveSettings();
        } else {
          failure_ = Failure::ModelInvalid;
        }
        if (installed) install_status_ = installed->message;
        RefreshSettings();
        return 0;
      }
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
          case kTrayCapture: BeginRegionSelection(); break;
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
    AppendMenuW(menu, MF_STRING | (paused_ ? MF_GRAYED : 0), kTrayCapture,
                L"Capture screen text (select area)...");
    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(menu, MF_STRING, kTraySettings, L"Settings...");
    AppendMenuW(menu, MF_SEPARATOR, 0, nullptr);
    AppendMenuW(menu, MF_STRING, kTrayQuit, L"Quit ClipLingo");
    tray_previous_foreground_ = GetForegroundWindow();
    POINT where{}; GetCursorPos(&where);
    SetForegroundWindow(host_);
    TrackPopupMenu(menu, TPM_RIGHTBUTTON, where.x, where.y, 0, host_, nullptr);
    PostMessageW(host_, WM_NULL, 0, 0);
    DestroyMenu(menu);
  }
  void BeginRegionSelection() {
    if (paused_ || selector_.active()) return;
    if (!ocr_available()) {
      failure_ = Failure::OcrUnavailable;
      ShowSettings(1);
      return;
    }
    popup_.Hide();
    KillTimer(host_, kDismissTimer);
    ++generation_; // invalidate in-flight translation before region selection
    {
      std::lock_guard lock(jobs_mutex_);
      pending_.reset();
    }
    if (!selector_.Start(instance_, tray_previous_foreground_,
          [this](std::optional<RECT> chosen) {
            if (!chosen || paused_) return;
            Request req{};
            req.kind = RequestKind::ManualOcr;
            req.region = *chosen;
            // The drag-release cursor is the interaction anchor. Freeze it
            // before OCR/inference; never chase subsequent pointer movement.
            GetCursorPos(&req.cursor);
            req.generation = ++generation_;
            {
              std::lock_guard lock(jobs_mutex_);
              pending_ = req;
              worker_prefs_ = prefs_;
            }
            jobs_cv_.notify_one();
          })) {
      failure_ = Failure::CaptureFailed;
      RefreshSettings();
    }
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
    // UI Automation runs only in a time-bounded helper process.
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
      const auto scope = !settings.hover_enabled ? CaptureScope::SelectionOnly :
          static_cast<CaptureScope>(settings.hover_scope + 1);
      CaptureResult captured;
      try {
        captured = current.kind == RequestKind::ManualOcr ?
            capture_ocr_isolated(current.region) :
            capture_isolated(current.foreground, current.cursor, scope);
      } catch (const std::exception&) {
        // Recoverable native library/allocator errors must not unwind out of
        // the jthread entrypoint or terminate the tray application.
        captured = {Failure::CaptureFailed, {}};
      }
      result->failure = captured.failure;
      if (captured) {
        // A newer hotkey may have arrived while the bounded UIA helper ran.
        // Avoid wasting model inference on a request that can no longer render.
        bool superseded = false;
        {
          std::lock_guard lock(jobs_mutex_);
          superseded = pending_.has_value() &&
                        pending_->generation > current.generation;
        }
        if (superseded) continue;
        const auto translated = translate_en_id(captured.text, settings.model_path);
        result->failure = translated.failure;
        if (translated) result->translated = translated.text;
      }
      // Post ownership to UI thread. No worker holds or mutates popup HWND.
      if (PostMessageW(host_, kResultMessage, 0, reinterpret_cast<LPARAM>(result.get())))
        result.release();
    }
    unload_translation_model();
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
    WritePrivateProfileStringW(L"appearance", L"theme", std::to_wstring(prefs_.theme).c_str(), path.c_str());
    WritePrivateProfileStringW(L"appearance", L"surface", std::to_wstring(prefs_.popup_style).c_str(), path.c_str());
    WritePrivateProfileStringW(L"capture", L"hover_enabled", prefs_.hover_enabled ? L"1" : L"0", path.c_str());
    WritePrivateProfileStringW(L"capture", L"hover_scope", std::to_wstring(prefs_.hover_scope).c_str(), path.c_str());
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
    if (msg == WM_ERASEBKGND) {
      RECT r{}; GetClientRect(hwnd, &r);
      FillRect(reinterpret_cast<HDC>(wp), &r, reinterpret_cast<HBRUSH>(GetStockObject(WHITE_BRUSH)));
      RECT sidebar_panel{0, 0, 226, r.bottom};
      if (sidebar_brush_) FillRect(reinterpret_cast<HDC>(wp), &sidebar_panel, sidebar_brush_);
      return 1;
    }
    if (msg == WM_MEASUREITEM && wp == kSidebar) {
      auto* measure = reinterpret_cast<MEASUREITEMSTRUCT*>(lp);
      measure->itemHeight = 48;
      return TRUE;
    }
    if (msg == WM_DRAWITEM && wp == kSidebar) {
      auto* draw = reinterpret_cast<DRAWITEMSTRUCT*>(lp);
      if (draw->itemID == static_cast<UINT>(-1)) return TRUE;
      wchar_t name[80]{};
      SendMessageW(sidebar_, LB_GETTEXT, draw->itemID, reinterpret_cast<LPARAM>(name));
      RECT rect = draw->rcItem;
      RECT highlight{rect.left + 4, rect.top + 2, rect.right - 4, rect.bottom - 2};
      HIGHCONTRASTW contrast{sizeof(contrast)};
      SystemParametersInfoW(SPI_GETHIGHCONTRAST, sizeof(contrast), &contrast, 0);
      const bool hc = (contrast.dwFlags & HCF_HIGHCONTRASTON) != 0;
      const bool selected = (draw->itemState & ODS_SELECTED) != 0;
      HBRUSH fill = CreateSolidBrush(hc ? GetSysColor(selected ? COLOR_HIGHLIGHT : COLOR_WINDOW) :
                        (selected ? RGB(232,242,255) : RGB(245,247,250)));
      FillRect(draw->hDC, &highlight, fill); DeleteObject(fill);
      SetBkMode(draw->hDC, TRANSPARENT);
      SetTextColor(draw->hDC, hc ? GetSysColor(selected ? COLOR_HIGHLIGHTTEXT : COLOR_WINDOWTEXT) :
                              (selected ? RGB(0,122,255) : RGB(29,36,48)));
      RECT text = highlight; text.left += 16;
      const auto old_font = SelectObject(draw->hDC, font_);
      DrawTextW(draw->hDC, name, -1, &text, DT_SINGLELINE | DT_LEFT | DT_VCENTER);
      SelectObject(draw->hDC, old_font);
      if (draw->itemState & ODS_FOCUS) DrawFocusRect(draw->hDC, &highlight);
      return TRUE;
    }
    if (msg == WM_CTLCOLORLISTBOX) {
      SetBkMode(reinterpret_cast<HDC>(wp), TRANSPARENT);
      return reinterpret_cast<LRESULT>(sidebar_brush_);
    }
    if (msg == WM_COMMAND) {
      const int id = LOWORD(wp);
      if (id == kSidebar && HIWORD(wp) == LBN_SELCHANGE) { DrawSettingsSection(); return 0; }
      if (id == kChooseModel && HIWORD(wp) == BN_CLICKED) { ChooseModel(); return 0; }
      if (id == kThemeCombo && HIWORD(wp) == CBN_SELCHANGE) {
        prefs_.theme = static_cast<int>(SendMessageW(theme_combo_, CB_GETCURSEL, 0, 0));
        SaveSettings(); return 0;
      }
      if (id == kSurfaceCombo && HIWORD(wp) == CBN_SELCHANGE) {
        prefs_.popup_style = static_cast<int>(SendMessageW(surface_combo_, CB_GETCURSEL, 0, 0));
        SaveSettings(); return 0;
      }
      if (id == kHoverEnabled && HIWORD(wp) == BN_CLICKED) {
        prefs_.hover_enabled = SendMessageW(reinterpret_cast<HWND>(lp), BM_GETCHECK, 0, 0) == BST_CHECKED;
        SaveSettings();
        DrawSettingsSection();
        return 0;
      }
      if (id == kHoverScope && HIWORD(wp) == CBN_SELCHANGE) {
        const int scope = static_cast<int>(SendMessageW(reinterpret_cast<HWND>(lp), CB_GETCURSEL, 0, 0));
        if (scope >= 0 && scope <= 2) { prefs_.hover_scope = scope; SaveSettings(); }
        return 0;
      }
      if (id == kPreviewPopup && HIWORD(wp) == BN_CLICKED) {
        POINT anchor{}; GetCursorPos(&anchor);
        popup_.ShowText(L"Terjemahan muncul di atas kursor, dengan ukuran sesuai isi teks.", anchor, prefs_);
        KillTimer(host_, kDismissTimer);
        SetTimer(host_, kDismissTimer, 10000, nullptr);
        return 0;
      }
      if (id == kStartupToggle && HIWORD(wp) == BN_CLICKED) {
        if (!set_startup(!startup_enabled()))
          MessageBoxW(hwnd, L"Couldn't update Windows startup preference.", L"ClipLingo", MB_ICONWARNING);
        DrawSettingsSection(); return 0;
      }
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
      sidebar_brush_ = CreateSolidBrush(RGB(245,247,250));
      sidebar_ = Control(L"LISTBOX", L"", WS_CHILD | WS_VISIBLE | LBS_NOTIFY | LBS_HASSTRINGS | LBS_OWNERDRAWFIXED | LBS_NOINTEGRALHEIGHT,
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
    width_edit_ = height_edit_ = theme_combo_ = surface_combo_ = nullptr;
    const int section = static_cast<int>(SendMessageW(sidebar_, LB_GETCURSEL, 0, 0));
    switch (section) {
      case 0:
        Label(L"General", 32); Label(L"ClipLingo runs from the Windows notification area.", 78);
        Label(shortcut_registered_ ? L"Shortcut: Ctrl + Alt + T  —  Registered" :
              L"Shortcut: Ctrl + Alt + T  —  Unavailable (conflict)", 126);
        Control(L"BUTTON", paused_ ? L"Resume translation" : L"Pause translation",
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 246, 193, 185, 33, kPauseButton);
        Label(startup_enabled() ? L"Launch at Windows sign-in: enabled" :
              L"Launch at Windows sign-in: disabled", 255);
        Control(L"BUTTON", startup_enabled() ? L"Disable launch at sign-in" : L"Enable launch at sign-in",
                WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON, 246, 303, 220, 33, kStartupToggle);
        break;
      case 1:
        Label(L"Text Capture", 32);
        Label(L"Selected text is always translated exactly, before hover fallback.", 78);
        {
          const auto checkbox = Control(L"BUTTON", L"Enable hover fallback on shortcut",
              WS_CHILD | WS_VISIBLE | WS_TABSTOP | BS_AUTOCHECKBOX,
              246, 139, 350, 28, kHoverEnabled);
          SendMessageW(checkbox, BM_SETCHECK, prefs_.hover_enabled ? BST_CHECKED : BST_UNCHECKED, 0);
          Label(L"Hover text scope", 190, 230);
          const auto combo = Control(L"COMBOBOX", L"", WS_CHILD | WS_VISIBLE |
              WS_TABSTOP | CBS_DROPDOWNLIST | WS_VSCROLL, 490, 184, 180, 120, kHoverScope);
          for (const auto label : {L"Word", L"Sentence", L"Paragraph"})
            SendMessageW(combo, CB_ADDSTRING, 0, reinterpret_cast<LPARAM>(label));
          SendMessageW(combo, CB_SETCURSEL, prefs_.hover_scope, 0);
          EnableWindow(combo, prefs_.hover_enabled);
        }
        Label(L"No background tracking. Hover is read only after Ctrl+Alt+T.", 251);
        Label(ocr_available() ?
              L"Region OCR: available. Choose Capture screen text from the tray." :
              L"Region OCR unavailable: install optional Tesseract OCR + eng.traineddata.", 304);
        Label(L"OCR is manual only. No clipboard monitoring or saved screenshots.", 348);
        break;
      case 2:
        Label(L"Translation Popup", 32);
        Label(L"Text only • above trigger cursor • automatic content sizing", 80);
        Label(L"Scroll long text: Ctrl+Alt+Up / Ctrl+Alt+Down", 118);
        Label(L"Maximum width (DIPs)", 153, 240);
        width_edit_ = Control(L"EDIT", std::to_wstring(prefs_.max_width_dip).c_str(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER, 490, 148, 110, 30, kMaxWidthEdit);
        Label(L"Maximum height (DIPs)", 199, 240);
        height_edit_ = Control(L"EDIT", std::to_wstring(prefs_.max_height_dip).c_str(),
                WS_CHILD | WS_VISIBLE | WS_BORDER | ES_NUMBER, 490, 194, 110, 30, kMaxHeightEdit);
        Control(L"BUTTON", L"Apply", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                490, 244, 110, 33, kApplySize);
        Control(L"BUTTON", L"Preview translation", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                246, 316, 185, 33, kPreviewPopup);
        break;
      case 3:
        Label(L"Appearance", 32);
        Label(L"Native typography: Segoe UI Variable, with accessible color fallback.", 80);
        Label(L"Theme", 147, 230);
        theme_combo_ = Control(L"COMBOBOX", L"", WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
                               490, 140, 160, 130, kThemeCombo);
        SendMessageW(theme_combo_, CB_ADDSTRING, 0, reinterpret_cast<LPARAM>(L"Light"));
        SendMessageW(theme_combo_, CB_ADDSTRING, 0, reinterpret_cast<LPARAM>(L"Dark"));
        SendMessageW(theme_combo_, CB_SETCURSEL, prefs_.theme, 0);
        Label(L"Popup surface", 200, 230);
        surface_combo_ = Control(L"COMBOBOX", L"", WS_CHILD | WS_VISIBLE | CBS_DROPDOWNLIST | WS_VSCROLL,
                                 490, 193, 160, 130, kSurfaceCombo);
        SendMessageW(surface_combo_, CB_ADDSTRING, 0, reinterpret_cast<LPARAM>(L"Solid"));
        SendMessageW(surface_combo_, CB_ADDSTRING, 0, reinterpret_cast<LPARAM>(L"Tinted"));
        SendMessageW(surface_combo_, CB_SETCURSEL, prefs_.popup_style, 0);
        Label(L"Glass remains unavailable until compositor/contrast validation.", 267);
        Control(L"BUTTON", L"Preview translation", WS_CHILD | WS_VISIBLE | BS_PUSHBUTTON,
                246, 337, 185, 33, kPreviewPopup);
        break;
      case 4:
        Label(L"Language Models", 32);
        Label(L"Offline EN → ID model: verified CTranslate2 package with SentencePiece.", 80);
        Label(installing_ ? L"Model: Verifying and installing..." :
          (prefs_.model_path.empty() ? L"Model: Not installed" : L"Model: Configured (verified on load)"), 136);
        Label(install_status_.c_str(), 169);
        Control(L"BUTTON", L"Install verified model pack...", WS_CHILD | WS_VISIBLE |
                (installing_ ? WS_DISABLED : 0) | BS_PUSHBUTTON,
                246, 219, 265, 33, kChooseModel);
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
    if (installing_) return;
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
    if (installer_.joinable()) installer_.join(); // completed previous install
    installing_ = true;
    install_status_ = L"Checking package hashes, then copying to private model storage.";
    DrawSettingsSection();
    installer_ = std::jthread([this, candidate]() {
      auto result = std::make_unique<ModelInstallResult>(install_model_pack(candidate));
      if (PostMessageW(host_, kModelInstalledMessage, 0,
                       reinterpret_cast<LPARAM>(result.get()))) result.release();
    });
  }
  void RefreshSettings() { if (settings_ && IsWindowVisible(settings_)) DrawSettingsSection(); }

  HINSTANCE instance_{nullptr};
  HANDLE single_instance_{nullptr};
  HWND host_{nullptr}, settings_{nullptr}, sidebar_{nullptr}, width_edit_{nullptr}, height_edit_{nullptr};
  HWND theme_combo_{nullptr}, surface_combo_{nullptr};
  HFONT font_{nullptr}; HICON icon_{nullptr};
  HBRUSH sidebar_brush_{nullptr};
  UINT taskbar_created_{0};
  bool shortcut_registered_{false}, paused_{false};
  bool scroll_up_registered_{false}, scroll_down_registered_{false};
  Failure failure_{Failure::None};
  Preferences prefs_, worker_prefs_;
  TextPopup popup_;
  RegionSelector selector_;
  HWND tray_previous_foreground_{nullptr};
  std::jthread installer_;
  bool installing_{false};
  std::wstring install_status_{L"Install a locally prepared EN-ID pack; no account needed."};
  std::uint64_t generation_{0};
  std::mutex jobs_mutex_;
  std::condition_variable_any jobs_cv_;
  std::optional<Request> pending_;
  std::jthread worker_;
};

// Headless native release acceptance: uses the SAME C++ inference entrypoint as
// the hotkey worker. No web runtime, no model download, no UI or raw-text logs.
// Exit 0 only when the bundled EN->ID model really produces Indonesian output.
int native_model_probe() {
  int argc = 0;
  LPWSTR* argv = CommandLineToArgvW(GetCommandLineW(), &argc);
  if (!argv) return -1;
  const bool probe = argc > 1 && wcscmp(argv[1], L"--verify-model") == 0;
  if (!probe) { LocalFree(argv); return -1; }
  if (argc != 3) { LocalFree(argv); return 2; }
  const std::filesystem::path model(argv[2]);
  LocalFree(argv);
  const auto translated = translate_en_id(L"Hello world!", model);
  const bool valid = translated && translated.text.find(L"dunia") != std::wstring::npos;
  unload_translation_model();
  return valid ? 0 : 3;
}

int WINAPI wWinMain(HINSTANCE instance, HINSTANCE, PWSTR, int) {
  int child_exit = 1;
  if (maybe_run_capture_helper(child_exit)) return child_exit;
  if (const int diagnostic = native_model_probe(); diagnostic >= 0) return diagnostic;
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