//! Native, no-focus translation result renderer with bounded scrollable text.
//! Only translated target text is provided; never source text or history.
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
        GetDC, GetMonitorInfoW, IntersectClipRect, InvalidateRect, MonitorFromPoint, ReleaseDC,
        RestoreDC, SaveDC, SelectObject, SetBkMode, SetTextColor, MONITORINFO,
        MONITOR_DEFAULTTONEAREST, PAINTSTRUCT,
        DT_CALCRECT, DT_LEFT, DT_NOPREFIX, DT_SINGLELINE, DT_WORDBREAK, TRANSPARENT,
    },
    System::{LibraryLoader::GetModuleHandleW, Threading::GetCurrentThreadId},
    UI::{
        HiDpi::{GetDpiForMonitor, GetDpiForWindow, MDT_EFFECTIVE_DPI},
        WindowsAndMessaging::{
            CreateWindowExW, DefWindowProcW, DestroyWindow, GetCursorPos, GetForegroundWindow,
            KillTimer, RegisterClassW, SetTimer, SetWindowPos, ShowWindow, WNDCLASSW,
            HWND_TOPMOST, MA_NOACTIVATE, SW_HIDE, SWP_NOACTIVATE, SWP_SHOWWINDOW,
            WM_MOUSEACTIVATE, WM_MOUSEWHEEL, WM_PAINT, WM_TIMER, WS_EX_NOACTIVATE, WS_EX_TOOLWINDOW, WS_EX_TOPMOST,
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

// No Windows controls or focusable child elements. Mouse wheel events are
// processed only when the OS routes them to this no-activate native HWND.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
struct ScrollState {
    content_px: i32,
    viewport_px: i32,
    offset_px: i32,
    line_step_px: i32,
    wheel_remainder: i32,
}
impl ScrollState {
    fn new(content_px: i32, viewport_px: i32, line_step_px: i32) -> Self {
        Self { content_px, viewport_px, line_step_px: line_step_px.max(1),
               offset_px: 0, wheel_remainder: 0 }
    }
    fn max_offset(self) -> i32 {
        (self.content_px - self.viewport_px).max(0)
    }
    fn can_scroll(self) -> bool { self.max_offset() > 0 }
    fn apply_wheel(&mut self, delta: i32) -> bool {
        if !self.can_scroll() { return false; }
        self.wheel_remainder = self.wheel_remainder.saturating_add(delta);
        let notches = self.wheel_remainder / 120;
        self.wheel_remainder %= 120;
        if notches == 0 { return false; }
        let old = self.offset_px;
        // Three text lines per wheel notch, bounded to the complete text height.
        self.offset_px = self.offset_px.saturating_sub(
            notches.saturating_mul(self.line_step_px).saturating_mul(3)
        ).clamp(0, self.max_offset());
        old != self.offset_px
    }
}

thread_local! {
    static DISPLAY_TEXT: RefCell<Vec<u16>> = const { RefCell::new(Vec::new()) };
    static PREVIEW_START: Cell<Option<Instant>> = const { Cell::new(None) };
    static SCROLL: Cell<ScrollState> = const { Cell::new(ScrollState { content_px: 0, viewport_px: 0, offset_px: 0, line_step_px: 1, wheel_remainder: 0 }) };
    // Keep paint font/padding at the same DPI as measurement for each result.
    static RENDER_DPI: Cell<i32> = const { Cell::new(96) };
}

// The scrolling text is clipped to this inset, never to the entire HWND.
// Keep the padding clear even when content is offset above or below the viewport.
fn text_viewport(client: RECT, padding: i32) -> Option<RECT> {
    let rect = RECT {
        left: client.left.saturating_add(padding),
        top: client.top.saturating_add(padding),
        right: client.right.saturating_sub(padding),
        bottom: client.bottom.saturating_sub(padding),
    };
    (rect.left < rect.right && rect.top < rect.bottom).then_some(rect)
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
        self.show_text_at(anchor, PREVIEW)
    }

    /// Only pass actual target-language inference output from the verified worker.
    /// HWND painting and resource ownership remain on the Tauri event-loop thread.
    pub fn show_text_at(&self, anchor: (i32, i32), target: &str) -> Result<(), String> {
        if target.trim().is_empty() || target.encode_utf16().count() > 4_096 || target.contains('\0') {
            return Err("Invalid translated text length".into());
        }
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
            // The current HWND may still belong to the previous monitor.
            // Measure for the trigger cursor's monitor, not its old position.
            let mut monitor_dpi_x = 0u32;
            let mut monitor_dpi_y = 0u32;
            let dpi = if GetDpiForMonitor(monitor, MDT_EFFECTIVE_DPI,
                &mut monitor_dpi_x, &mut monitor_dpi_y) >= 0 && monitor_dpi_x > 0 {
                monitor_dpi_x as i32
            } else {
                GetDpiForWindow(hwnd).max(96) as i32
            };
            let pad = px(PADDING_DIP, dpi);
            let area = monitor_info.rcWork;
            let max_w = px(MAX_WIDTH_DIP, dpi).min(area.right - area.left);
            let max_h = px(MAX_HEIGHT_DIP, dpi).min(area.bottom - area.top);
            if max_w <= pad * 2 || max_h <= pad * 2 {
                return Err("Monitor work area too small for readable native popup".into());
            }
            let mut rect = RECT { left: 0, top: 0, right: max_w - pad * 2, bottom: 0 };
            let label = utf16(target);
            let hdc = GetDC(hwnd);
            if hdc.is_null() {
                return Err("GetDC failed".into());
            }
            let family = utf16("Segoe UI");
            let font = CreateFontW(-px(15, dpi), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, family.as_ptr());
            let old = if !font.is_null() { SelectObject(hdc, font) } else { std::ptr::null_mut() };
            // Size short target text intrinsically; wrap only above width cap.
            let mut measured = DrawTextW(hdc, label.as_ptr(), (label.len() - 1) as i32, &mut rect,
                DT_CALCRECT | DT_SINGLELINE | DT_NOPREFIX | DT_LEFT);
            let wrapped = target.contains('\n') || target.contains('\r') ||
                rect.right - rect.left > max_w - pad * 2;
            if wrapped {
                rect = RECT { left: 0, top: 0, right: max_w - pad * 2, bottom: 0 };
                measured = DrawTextW(hdc, label.as_ptr(), (label.len() - 1) as i32, &mut rect,
                    DT_WORDBREAK | DT_CALCRECT | DT_NOPREFIX | DT_LEFT);
            }
            if !old.is_null() { SelectObject(hdc, old); }
            if !font.is_null() { DeleteObject(font); }
            ReleaseDC(hwnd, hdc);
            if measured == 0 {
                return Err("Native text measurement failed".into());
            }
            let content_height = rect.bottom - rect.top;
            // DT_CALCRECT may shrink the rectangle width. Wrapped text must
            // paint at the SAME width used for measurement or it can reflow
            // and hide bottom lines. Very long unbroken words fail closed.
            let measured_width = rect.right - rect.left + pad * 2;
            if content_height <= 0 || measured_width <= 0 || measured_width > max_w {
                return Err("Native target contains an unrenderable long word or invalid dimensions".into());
            }
            let width = if wrapped { max_w } else { measured_width };
            // Entire target stays in DISPLAY_TEXT; only the visible viewport is
            // painted. Never discard the unshown lines or silently truncate.
            let height = (content_height + pad * 2).min(max_h);
            let viewport_height = height - pad * 2;
            let scroll = ScrollState::new(content_height, viewport_height, px(24, dpi));
            let (x, y) = layout_position(
                (cursor.x, cursor.y), (area.left, area.top, area.right, area.bottom),
                (width, height), px(GAP_DIP, dpi),
            );
            DISPLAY_TEXT.with(|cell| *cell.borrow_mut() = label);
            SCROLL.with(|cell| cell.set(scroll)); // reset scroll when result is replaced
            RENDER_DPI.with(|cell| cell.set(dpi));
            PREVIEW_START.with(|start| start.set(Some(Instant::now())));
            if SetWindowPos(hwnd, HWND_TOPMOST, x, y, width, height, SWP_NOACTIVATE | SWP_SHOWWINDOW) == 0 {
                return Err("SetWindowPos failed".into());
            }
            InvalidateRect(hwnd, std::ptr::null(), 1);
            KillTimer(hwnd, 1);
            // Longer output remains readable while the user scrolls;
            // each wheel interaction refreshes the inactivity deadline.
            let dismissal_ms = if target == PREVIEW { 2_500 } else if scroll.can_scroll() {
                20_000
            } else {
                4_500 + (target.chars().count() as u32).saturating_mul(40).min(8_000)
            };
            if SetTimer(hwnd, 1, dismissal_ms, None) == 0 {
                ShowWindow(hwnd, SW_HIDE);
                return Err("Unable to arm automatic preview dismissal".into());
            }
            // WS_EX_NOACTIVATE and SWP_NOACTIVATE are necessary but not
            // sufficient proof: a real Windows foreground/focus test is gated.
            let _ = GetForegroundWindow();
            Ok(())
        }
    }

    pub fn hide(&self) {
        if let Ok(hwnd) = self.hwnd() {
            unsafe { KillTimer(hwnd, 1); ShowWindow(hwnd, SW_HIDE); }
            DISPLAY_TEXT.with(|cell| cell.borrow_mut().clear());
            SCROLL.with(|cell| cell.set(ScrollState::default()));
        }
    }

    pub fn destroy(&self) {
        if let Ok(hwnd) = self.hwnd() {
            unsafe { KillTimer(hwnd, 1); DestroyWindow(hwnd); }
        }
    }
}

unsafe extern "system" fn window_proc(hwnd: HWND, msg: u32, w: WPARAM, l: LPARAM) -> LRESULT {
    match msg {
        WM_MOUSEACTIVATE => MA_NOACTIVATE as LRESULT,
        WM_TIMER if w == 1 => {
            KillTimer(hwnd, 1);
            ShowWindow(hwnd, SW_HIDE);
            DISPLAY_TEXT.with(|cell| cell.borrow_mut().clear());
            SCROLL.with(|cell| cell.set(ScrollState::default()));
            0
        },
        WM_MOUSEWHEEL => {
            let delta = ((w >> 16) as u16 as i16) as i32;
            let mut scrollable = false;
            let changed = SCROLL.with(|cell| {
                let mut state = cell.get();
                scrollable = state.can_scroll();
                let moved = state.apply_wheel(delta);
                cell.set(state);
                moved
            });
            if scrollable {
                if changed {
                    InvalidateRect(hwnd, std::ptr::null(), 1);
                }
                // Even a wheel attempt at the first/last line indicates
                // active reading; do not dismiss while the user is trying.
                KillTimer(hwnd, 1);
                SetTimer(hwnd, 1, 20_000, None);
                return 0;
            }
            DefWindowProcW(hwnd, msg, w, l)
        },
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
                let dpi = RENDER_DPI.with(|cell| cell.get());
                let family = utf16("Segoe UI");
                let font = CreateFontW(-px(15, dpi), 0, 0, 0, 400, 0, 0, 0, 1, 0, 0, 0, 0, family.as_ptr());
                let old = if !font.is_null() { SelectObject(hdc, font) } else { std::ptr::null_mut() };
                SetBkMode(hdc, TRANSPARENT as i32);
                SetTextColor(hdc, 0x0030_241D);
                let pad = px(PADDING_DIP, dpi);
                if let Some(viewport) = text_viewport(bounds, pad) {
                    // BeginPaint clips only to the HWND's client/update region.
                    // Intersect with the padded *text* viewport before drawing
                    // scrolled lines, then restore the original GDI clip state.
                    let saved = SaveDC(hdc);
                    if saved != 0 {
                        if IntersectClipRect(
                            hdc, viewport.left, viewport.top,
                            viewport.right, viewport.bottom
                        ) != 0 {
                            let scroll = SCROLL.with(|cell| cell.get());
                            let mut text_bounds = viewport;
                            text_bounds.top -= scroll.offset_px;
                            text_bounds.bottom = text_bounds.top + scroll.content_px;
                            DISPLAY_TEXT.with(|cell| {
                                let text = cell.borrow();
                                if text.len() > 1 {
                                    DrawTextW(hdc, text.as_ptr(), (text.len() - 1) as i32,
                                        &mut text_bounds, DT_WORDBREAK | DT_NOPREFIX | DT_LEFT);
                                }
                            });
                        }
                        RestoreDC(hdc, saved);
                    }
                }
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
    #[test]
    fn scrolled_text_viewport_preserves_inner_padding() {
        let full = RECT { left: 0, top: 0, right: 420, bottom: 300 };
        let viewport = text_viewport(full, 14).expect("valid padded viewport");
        assert_eq!((viewport.left, viewport.top, viewport.right, viewport.bottom),
                   (14, 14, 406, 286));
        let too_small = RECT { left: 0, top: 0, right: 27, bottom: 28 };
        assert!(text_viewport(too_small, 14).is_none());
        let negative = RECT { left: -300, top: -200, right: 120, bottom: 100 };
        let viewport = text_viewport(negative, 14).unwrap();
        assert_eq!((viewport.left, viewport.top, viewport.right, viewport.bottom),
                   (-286, -186, 106, 86));
    }
    #[test]
    fn long_translation_keeps_full_content_and_scrolls_to_last_line() {
        let mut state = ScrollState::new(1300, 250, 20);
        assert_eq!(state.max_offset(), 1050);
        assert!(state.apply_wheel(-120));
        assert_eq!(state.offset_px, 60);
        assert!(state.apply_wheel(-6000));
        assert_eq!(state.offset_px, 1050);
        assert!(!state.apply_wheel(-120)); // at end, not silently wrapped
        assert!(state.apply_wheel(120));
        assert_eq!(state.offset_px, 990);
        assert!(state.apply_wheel(99999));
        assert_eq!(state.offset_px, 0);
    }
    #[test]
    fn short_text_is_not_scrollable_and_partial_wheel_deltas_accumulate() {
        let mut short = ScrollState::new(100, 250, 20);
        assert!(!short.can_scroll());
        assert!(!short.apply_wheel(-120));
        let mut long = ScrollState::new(600, 250, 20);
        assert!(!long.apply_wheel(-60));
        assert!(long.apply_wheel(-60));
        assert_eq!(long.offset_px, 60);
    }
    #[test]
    fn new_translation_restarts_at_top() {
        let mut previous = ScrollState::new(1200, 250, 20);
        previous.apply_wheel(-240);
        let next = ScrollState::new(800, 250, 20);
        assert_eq!(next.offset_px, 0);
        assert_eq!(next.max_offset(), 550);
    }
}
