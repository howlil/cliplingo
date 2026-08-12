use std::mem::size_of;

use windows::Win32::Foundation::{GetLastError, POINT};
use windows::Win32::UI::WindowsAndMessaging::{
    GetCursorPos, GetMonitorInfoW, MonitorFromPoint, MONITORINFO, MONITOR_DEFAULTTONEAREST,
};

use crate::core::{CaptureError, ScreenRect};

#[derive(Clone, Debug, PartialEq)]
pub struct CursorContext {
    pub anchor: ScreenRect,
    pub work_area: ScreenRect,
}

pub fn cursor_anchor() -> Result<CursorContext, CaptureError> {
    let mut point = POINT::default();
    unsafe {
        GetCursorPos(&mut point).map_err(|_| native_error("GetCursorPos"))?;
    }

    let monitor = unsafe { MonitorFromPoint(point, MONITOR_DEFAULTTONEAREST) };
    let mut info = MONITORINFO {
        cbSize: size_of::<MONITORINFO>() as u32,
        ..Default::default()
    };
    let ok = unsafe { GetMonitorInfoW(monitor, &mut info) };
    if !ok.as_bool() {
        return Err(native_error("GetMonitorInfoW"));
    }

    Ok(CursorContext {
        anchor: ScreenRect {
            x: f64::from(point.x),
            y: f64::from(point.y),
            width: 1.0,
            height: 1.0,
        },
        work_area: ScreenRect {
            x: f64::from(info.rcWork.left),
            y: f64::from(info.rcWork.top),
            width: f64::from(info.rcWork.right - info.rcWork.left),
            height: f64::from(info.rcWork.bottom - info.rcWork.top),
        },
    })
}

fn native_error(operation: &'static str) -> CaptureError {
    let code = unsafe { GetLastError().0 as i32 };
    CaptureError::NativeFailure { operation, code }
}
