//! F0-only native tooltip renderer. It never captures or translates text.
//! All HWND/GDI calls are confined to the Tauri event-loop (creator) thread.
//! Later milestones can replace the GDI text backend with DirectWrite without
//! changing caller ownership or the no-activation policy.
use std::{
    cell::{Cell, RefCell},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, POINT, RECT, WPARAM},
    Graphics::Gdi::{
        BeginPaint, CreateFontW, CreateSolidBrush, DeleteObject, DrawTextW, EndPaint, FillRect,
        GetDC, GetMonitorInfoW, InvalidateRect, MonitorFromPoint, ReleaseDC, SelectObject, SetBkMode,
        SetTextColor, MONITORINFO, MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
        DT_CALCRECT, DT_LEFT, DT_NOPREFIX, DT_WORDBREAK, TRANSPARENT,
    },
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::{
        HiDpi::GetDpiForWindow,
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetForegroundWindow,
            RegisterClassW, SetWindowPos, WNDCLASSW,
            HWND_TOPMOST, MA_NOACTIVATE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            WM_MOUSEACTIVATE, WM_PAINT, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
            WS_POPUP,
        },
    },
};

const MAX_WIDTH_DIP: i32 = 420;
const MAX_HEIGHT_DIP: i32 = 300;
const PADDING_DIP: i32 = 14;
const GAP_DIP: i32 = 12;
const PREVIEW: &str = "Pratinjau ClipLingo — bukan hasil terjemahan.";
static LAST_PAINT_US: AtomicU64 = AtomicU64::new(0);

thread_local! {
    static DISPLAY_TEXT: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
    static PREVIEW_START: Cell<Option<Instant>> = const { Cell::new(None) };
}

fn utf16(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

fn px(dip: i32, dpi: i32) -> i32 {
    ((dip as i64 * dpi.max(96) as i64 + 48) / 96) as i32
}

pub(crate) fn layout_position(anchor: (i32, i32), work: (i32, i32, i32, i32), wh: (i32, i32), gap: i32) -> (i32, i32) {
    let (left, top, right, bottom) = work;
    let (w, h) = wh;
    let x = (anchor.0 - w / 2).clamp(left, (right - w).max(left));
    let preferred_above = anchor.1 - gap - h;
    let y = if preferred_above >= top {
        preferred_above
    } else {
        anchor.1 + gap
    }.clamp(top, (bottom - h).max(top));
    (x, y)
}

pub fn cursor_snapshot() -> Result<(i32, i32), String> {
    let mut point = POINT::default();
    let success = unsafe { GetCursorPos(&mut point) };
    if success == 0 {
        Err("GetCursorPos failed".to_owned())
    } else {
        Ok((point.x, point.y))
    }
}

pub fn last_paint_us() -> Option<u64> {
    match LAST_PAINT_US.load(Ordering::Relaxed) {
        0 => None,
        value => Some(value),
    }
}

/// Only create, operate and destroy on the Windows event-loop thread.
/// Store HWND as an integer to let the Tauri-managed state be Send+Sync;
/// the creator-thread check is the safety invariant.
pub struct NativePopup {
    hwnd: isize,
    creator_thread: u32,
}

impl NativePopup {
    pub fn create() -> Result<Self, String> {
        let class = utf16("ClipLingoNativePopupF0");
        unsafe {
            // SAFETY: class and module pointers are valid for the OS call;
            // the window is created on the calling event-loop thread.
            let hinstance = GetModuleHandleW(std::ptr::null());
            let spec = WNDCLASSW {
                lpfnWndProc: Some(window_proc),
                hInstance: hinstance,
                lpszClassName: class.as_ptr(),
                ..Default::default()
            };
            // RegisterClassW returning 0 is acceptable if already registered
            // by an earlier native window in this process.
            RegisterClassW(&spec);
            let hwnd = CreateWindowExW(
                WS_EX_TOOLWINDOW | WS_EX_NOACTIVATE | WS_EX_TOPMOST,
                class.as_ptr(),
                std::ptr::null(),
                WS_POPUP,
                0, 0, 1, 1,
                std::ptr::null_mut(), std::ptr::null_mut(), hinstance,
                std::ptr::null(),
            );
            if hwnd.is_null() {
                return Err("Could not create no-activate native preview HWND".into());
            }
            Ok(Self { hwnd: hwnd as isize, creator_thread: GetCurrentThreadId() })
        }
    }

    fn hwnd(&self) -> Result<HWND, String> {
        let current = unsafe { GetCurrentThreadId() };
        if current != self.creator_thread {
            return Err("native popup called from wrong Windows thread".into());
        }
        Ok(self.hwnd as HWND)
    }

    /// Preview is ONLY callable from explicit tray/debug proof route, never
    /// the production Ctrl+Alt+T translation shortcut before capture exists.
    pub fn show_preview_at(&self, anchor: (i32, i32)) -> Result<(), String> {
        let hwnd = self.hwnd()?;
        unsafe {
            // SAFETY: HWND created on this UI thread and GDI handles are
            // selected/restored on the same thread before deletion.
            let cursor = POINT { x: anchor.0, y: anchor.1 };
            let monitor = MonitorFromPoint(cursor, MONITOR_DEFAULTTONEAREST);
            let mut monitor_info = MONITORINFO {
                cbSize: std::mem::size_of::<MONITORINFO>() as u32,
                ..Default::default()
            };
            if GetMonitorInfoW(monitor, &mut monitor_info) == 0 {
                return Err("GetMonitorInfoW failed".into());
            }
            let dpi = GetDpiForWindow(hwnd).max(96) as i32;
            let pad = px(PADDING_DIP, dpi);
            let max_w = px(MAX_WIDTH_DIP, dpi);
            let max_h = px(MAX_HEIGHT_DIP, dpi);
            let mut rect = RECT { left: 0, top: 0, right: max_w - pad * 2, bottom: 0 };
            let label = utf16(PREVIEW);
            let hdc = GetDC(hwnd);
            if hdc.is_null() {
                return Err("GetDC failed".into());
            }
            let family = utf16("Segoe UI");
            let font = CreateFontW(-px(15, dpi), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, family.as_ptr());
            let old = if !font.is_null() { SelectObject(hdc, font) } else { std::ptr::null_mut() };
            let flags = DT_WORDBREAK | DT_CALCRECT | DT_NOPREFIX | DT_LEFT;
            let measured = DrawTextW(hdc, label.as_ptr(), (label.len() - 1) as i32, &mut rect, flags);
            if !old.is_null() { SelectObject(hdc, old); }
            if !font.is_null() { DeleteObject(font); }
            ReleaseDC(hwnd, hdc);
            if measured == 0 {
                return Err("Native text measurement failed".into());
            }
            let width = (rect.right - rect.left + pad * 2).clamp(1, max_w);
            let height = (rect.bottom - rect.top + pad * 2).clamp(1, max_h);
            let area = monitor_info.rcWork;
            let (x, y) = layout_position(
                (cursor.x, cursor.y), (area.left, area.top, area.right, area.bottom),
                (width, height), px(GAP_DIP, dpi),
            );
            DISPLAY_TEXT.with(|cell| *cell.borrow_mut() = label);
            PREVIEW_START.with(|start| start.set(Some(Instant::now())));
            if SetWindowPos(hwnd, HWND_TOPMOST, x, y, width, height, SWP_NOACTIVATE | SWP_SHOWWINDOW) == 0 {
                return Err("SetWindowPos failed".into());
            }
            InvalidateRect(hwnd, std::ptr::null(), 1);
            // WS_EX_NOACTIVATE and SWP_NOACTIVATE are necessary but not
            // sufficient proof: a real Windows foreground/focus test is gated.
            let _ = GetForegroundWindow();
            Ok(())
        }
    }

    pub fn destroy(&self) {
        if let Ok(hwnd) = self.hwnd() {
            unsafe { DestroyWindow(hwnd); }
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
        WM_PAINT => {
            // SAFETY: dispatched by Windows on the owning window thread.
            let mut paint = PAINTSTRUCT::default();
            let hdc = BeginPaint(hwnd, &mut paint);
            if !hdc.is_null() {
                let mut bounds = RECT::default();
                windows_sys::Win32::UI::WindowsAndMessaging::GetClientRect(hwnd, &mut bounds);
                let white = CreateSolidBrush(0x00FF_FFFF);
                FillRect(hdc, &bounds, white);
                DeleteObject(white);
                let dpi = GetDpiForWindow(hwnd).max(96) as i32;
                let family = utf16("Segoe UI");
                let font = CreateFontW(-px(15, dpi), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, family.as_ptr());
                let old = if !font.is_null() { SelectObject(hdc, font) } else { std::ptr::null_mut() };
                SetBkMode(hdc, TRANSPARENT as i32);
                SetTextColor(hdc, 0x0030_241D);
                let pad = px(PADDING_DIP, dpi);
                bounds.left += pad;
                bounds.top += pad;
                bounds.right -= pad;
                bounds.bottom -= pad;
                DISPLAY_TEXT.with(|cell| {
                    let text = cell.borrow();
                    if text.len() > 1 {
                        DrawTextW(hdc, text.as_ptr(), (text.len() - 1) as i32, &mut bounds, DT_WORDBREAK | DT_NOPREFIX | DT_LEFT);
                    }
                });
                if !old.is_null() { SelectObject(hdc, old); }
                if !font.is_null() { DeleteObject(font); }
            }
            EndPaint(hwnd, &paint);
            PREVIEW_START.with(|start| {
                if let Some(t) = start.take() {
                    // Dispatch-to-WM_PAINT completion is NOT actual DWM-visible time.
                    LAST_PAINT_US.store(t.elapsed().as_micros().min(u64::MAX as u128) as u64, Ordering::Relaxed);
                }
            });
            0
        }
        _ => DefWindowProcW(hwnd, msg, w, l),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn position_above_pointer_within_work_area() {
        assert_eq!(layout_position((200, 180), (0, 0, 400, 300), (120, 40), 12), (140, 128));
    }
    #[test]
    fn bottom_fallback_and_negative_monitor_coordinates() {
        assert_eq!(layout_position((-1800, -850), (-1920, -1080, 0, 0), (200, 70), 12), (-1900, -932));
        assert_eq!(layout_position((20, 5), (0, 0, 500, 500), (100, 50), 12), (0, 17));
    }
    #[test]
    fn clipping_never_exits_small_work_area() {
        let (x, y) = layout_position((500, 500), (0, 0, 100, 80), (90, 70), 12);
        assert!((0..=10).contains(&x));
        assert!((0..=10).contains(&y));
    }
}
