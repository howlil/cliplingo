use std::mem::size_of;
use std::ptr;
use std::slice;

use windows::core::w;
use windows::Win32::Foundation::{GetLastError, GlobalFree, HANDLE, HGLOBAL};
use windows::Win32::System::DataExchange::{
    AddClipboardFormatListener, CloseClipboard, EmptyClipboard, EnumClipboardFormats,
    GetClipboardData, OpenClipboard, RemoveClipboardFormatListener, SetClipboardData,
};
use windows::Win32::System::Memory::{
    GlobalAlloc, GlobalLock, GlobalSize, GlobalUnlock, GMEM_MOVEABLE, GMEM_ZEROINIT,
};
use windows::Win32::System::Ole::CF_UNICODETEXT;
use windows::Win32::UI::Input::KeyboardAndMouse::{
    SendInput, INPUT, INPUT_0, INPUT_KEYBOARD, KEYBDINPUT, KEYEVENTF_KEYUP, VK_C, VK_CONTROL,
};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, GetMessageW, KillTimer, SetTimer, HWND_MESSAGE, MSG,
    WINDOW_EX_STYLE, WINDOW_STYLE, WM_CLIPBOARDUPDATE, WM_TIMER,
};

use crate::core::{CaptureError, Selection, SelectionProvider, SelectionSource};

const CLIPBOARD_TIMEOUT_MS: u32 = 750;
const TIMER_ID: usize = 1;

#[derive(Clone, Debug, PartialEq)]
enum ClipboardSnapshot {
    Empty,
    Unicode(Vec<u16>),
}

pub struct ClipboardSelectionProvider;

impl SelectionProvider for ClipboardSelectionProvider {
    fn capture(&mut self) -> Result<Selection, CaptureError> {
        let snapshot = snapshot_clipboard()?;
        let listener = ClipboardListener::new()?;

        let copied = match send_copy_shortcut().and_then(|_| listener.wait_for_update()) {
            Ok(()) => read_unicode_clipboard(),
            Err(error) => Err(error),
        };
        drop(listener);

        let restored = restore_clipboard(snapshot);
        let text = finish_capture(copied, restored)?;
        if text.trim().is_empty() {
            return Err(CaptureError::NoSelection);
        }

        Ok(Selection {
            text,
            source: SelectionSource::Clipboard,
            bounds: None,
            work_area: None,
        })
    }
}

fn finish_capture(
    copied: Result<String, CaptureError>,
    restored: Result<(), CaptureError>,
) -> Result<String, CaptureError> {
    restored?;
    copied
}

fn snapshot_clipboard() -> Result<ClipboardSnapshot, CaptureError> {
    with_clipboard(|| {
        let formats = enumerate_formats();
        if !formats_are_restorable(&formats) {
            return Err(CaptureError::ClipboardPreservationUnsupported);
        }

        if formats.is_empty() {
            Ok(ClipboardSnapshot::Empty)
        } else {
            read_unicode_units_while_open().map(ClipboardSnapshot::Unicode)
        }
    })
}

fn restore_clipboard(snapshot: ClipboardSnapshot) -> Result<(), CaptureError> {
    with_clipboard(|| {
        unsafe {
            EmptyClipboard().map_err(|_| clipboard_error("EmptyClipboard"))?;
        }

        if let ClipboardSnapshot::Unicode(units) = snapshot {
            set_unicode_units_while_open(&units)?;
        }
        Ok(())
    })
}

fn read_unicode_clipboard() -> Result<String, CaptureError> {
    with_clipboard(|| read_unicode_units_while_open().map(|units| unicode_text(&units)))
}

fn read_unicode_units_while_open() -> Result<Vec<u16>, CaptureError> {
    let handle = unsafe { GetClipboardData(u32::from(CF_UNICODETEXT.0)) }
        .map_err(|_| CaptureError::NoSelection)?;
    let memory = HGLOBAL(handle.0);
    let bytes = unsafe { GlobalSize(memory) };
    if bytes < size_of::<u16>() || bytes % size_of::<u16>() != 0 {
        return Err(CaptureError::NoSelection);
    }

    let raw = unsafe { GlobalLock(memory) }.cast::<u16>();
    if raw.is_null() {
        return Err(clipboard_error("GlobalLock"));
    }

    let capacity = bytes / size_of::<u16>();
    let units = unsafe { slice::from_raw_parts(raw, capacity) }.to_vec();
    let _ = unsafe { GlobalUnlock(memory) };
    Ok(units)
}

fn unicode_text(units: &[u16]) -> String {
    let len = units.iter().position(|unit| *unit == 0).unwrap_or(units.len());
    String::from_utf16_lossy(&units[..len])
}

fn set_unicode_units_while_open(units: &[u16]) -> Result<(), CaptureError> {
    let bytes = units.len() * size_of::<u16>();
    let memory = unsafe { GlobalAlloc(GMEM_MOVEABLE | GMEM_ZEROINIT, bytes) }
        .map_err(|_| clipboard_error("GlobalAlloc"))?;
    let raw = unsafe { GlobalLock(memory) }.cast::<u16>();
    if raw.is_null() {
        let _ = unsafe { GlobalFree(Some(memory)) };
        return Err(clipboard_error("GlobalLock"));
    }

    unsafe {
        ptr::copy_nonoverlapping(units.as_ptr(), raw, units.len());
    }
    let _ = unsafe { GlobalUnlock(memory) };

    let handle = HANDLE(memory.0);
    if unsafe { SetClipboardData(u32::from(CF_UNICODETEXT.0), Some(handle)) }.is_err() {
        let _ = unsafe { GlobalFree(Some(memory)) };
        return Err(clipboard_error("SetClipboardData"));
    }
    Ok(())
}

fn enumerate_formats() -> Vec<u32> {
    let mut result = Vec::new();
    let mut current = 0;
    loop {
        current = unsafe { EnumClipboardFormats(current) };
        if current == 0 {
            return result;
        }
        result.push(current);
    }
}

fn formats_are_restorable(formats: &[u32]) -> bool {
    formats
        .iter()
        .all(|format| *format == u32::from(CF_UNICODETEXT.0))
}

fn with_clipboard<T>(
    operation: impl FnOnce() -> Result<T, CaptureError>,
) -> Result<T, CaptureError> {
    unsafe { OpenClipboard(None) }.map_err(|_| CaptureError::ClipboardUnavailable)?;
    let result = operation();
    let close = unsafe { CloseClipboard() }.map_err(|_| clipboard_error("CloseClipboard"));
    match result {
        Ok(value) => close.map(|_| value),
        Err(error) => {
            let _ = close;
            Err(error)
        }
    }
}

fn send_copy_shortcut() -> Result<(), CaptureError> {
    let inputs = [
        key_input(VK_CONTROL, false),
        key_input(VK_C, false),
        key_input(VK_C, true),
        key_input(VK_CONTROL, true),
    ];
    let sent = unsafe { SendInput(&inputs, size_of::<INPUT>() as i32) };
    if sent != inputs.len() as u32 {
        return Err(CaptureError::ClipboardUnavailable);
    }
    Ok(())
}

fn key_input(key: windows::Win32::UI::Input::KeyboardAndMouse::VIRTUAL_KEY, up: bool) -> INPUT {
    INPUT {
        r#type: INPUT_KEYBOARD,
        Anonymous: INPUT_0 {
            ki: KEYBDINPUT {
                wVk: key,
                dwFlags: if up {
                    KEYEVENTF_KEYUP
                } else {
                    Default::default()
                },
                ..Default::default()
            },
        },
    }
}

struct ClipboardListener {
    hwnd: windows::Win32::Foundation::HWND,
    timer_id: usize,
}

impl ClipboardListener {
    fn new() -> Result<Self, CaptureError> {
        let hwnd = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("STATIC"),
                w!("ClipLingoClipboardListener"),
                WINDOW_STYLE::default(),
                0,
                0,
                0,
                0,
                Some(HWND_MESSAGE),
                None,
                None,
                None,
            )
        }
        .map_err(|_| clipboard_error("CreateWindowExW"))?;

        if unsafe { AddClipboardFormatListener(hwnd) }.is_err() {
            let _ = unsafe { DestroyWindow(hwnd) };
            return Err(clipboard_error("AddClipboardFormatListener"));
        }

        let timer_id = unsafe { SetTimer(Some(hwnd), TIMER_ID, CLIPBOARD_TIMEOUT_MS, None) };
        if timer_id == 0 {
            let _ = unsafe { RemoveClipboardFormatListener(hwnd) };
            let _ = unsafe { DestroyWindow(hwnd) };
            return Err(clipboard_error("SetTimer"));
        }

        Ok(Self { hwnd, timer_id })
    }

    fn wait_for_update(&self) -> Result<(), CaptureError> {
        loop {
            let mut message = MSG::default();
            let status = unsafe { GetMessageW(&mut message, Some(self.hwnd), 0, 0) };
            if status.0 == -1 {
                return Err(clipboard_error("GetMessageW"));
            }
            if status.0 == 0 || message.message == WM_TIMER {
                return Err(CaptureError::Timeout);
            }
            if message.message == WM_CLIPBOARDUPDATE {
                return Ok(());
            }
        }
    }
}

impl Drop for ClipboardListener {
    fn drop(&mut self) {
        let _ = unsafe { KillTimer(Some(self.hwnd), self.timer_id) };
        let _ = unsafe { RemoveClipboardFormatListener(self.hwnd) };
        let _ = unsafe { DestroyWindow(self.hwnd) };
    }
}

fn clipboard_error(operation: &'static str) -> CaptureError {
    CaptureError::NativeFailure {
        operation,
        code: unsafe { GetLastError().0 as i32 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_empty_or_unicode_clipboards_are_mutable_in_iteration_one() {
        assert!(formats_are_restorable(&[]));
        assert!(formats_are_restorable(&[u32::from(CF_UNICODETEXT.0)]));
        assert!(!formats_are_restorable(&[u32::from(CF_UNICODETEXT.0), 15]));
    }

    #[test]
    fn snapshot_can_preserve_raw_utf16_units_losslessly() {
        let original = vec![0x0041, 0xD800, 0x0000];
        let snapshot = ClipboardSnapshot::Unicode(original.clone());
        assert_eq!(snapshot, ClipboardSnapshot::Unicode(original));
    }

    #[test]
    fn unicode_text_stops_at_first_nul() {
        assert_eq!(unicode_text(&[0x0041, 0x0042, 0, 0x0043]), "AB");
    }

    #[test]
    fn restore_failure_takes_precedence_over_capture_failure() {
        let result = finish_capture(
            Err(CaptureError::Timeout),
            Err(CaptureError::ClipboardUnavailable),
        );
        assert_eq!(result, Err(CaptureError::ClipboardUnavailable));
    }
}
