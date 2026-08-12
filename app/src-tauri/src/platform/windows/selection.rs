use std::ffi::c_void;
use std::ptr;
use std::slice;

use windows::core::Error as WindowsError;
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
};
use windows::Win32::System::Ole::{
    SafeArrayAccessData, SafeArrayDestroy, SafeArrayGetLBound, SafeArrayGetUBound,
    SafeArrayUnaccessData,
};
use windows::Win32::UI::Accessibility::{
    CUIAutomation, IUIAutomation, IUIAutomationTextPattern, IUIAutomationTextRange,
    UIA_TextPatternId,
};

use crate::core::{CaptureError, ScreenRect, Selection, SelectionProvider, SelectionSource};

use super::work_area_for_rect;

pub struct UiAutomationSelectionProvider {
    com_initialized: bool,
}

impl UiAutomationSelectionProvider {
    pub fn new() -> Self {
        Self {
            com_initialized: false,
        }
    }

    fn ensure_com(&mut self) -> Result<(), CaptureError> {
        if self.com_initialized {
            return Ok(());
        }

        let result = unsafe { CoInitializeEx(None, COINIT_MULTITHREADED) };
        if result.is_err() {
            return Err(CaptureError::NativeFailure {
                operation: "CoInitializeEx",
                code: result.0,
            });
        }
        self.com_initialized = true;
        Ok(())
    }
}

impl Default for UiAutomationSelectionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for UiAutomationSelectionProvider {
    fn drop(&mut self) {
        if self.com_initialized {
            unsafe { CoUninitialize() };
        }
    }
}

impl SelectionProvider for UiAutomationSelectionProvider {
    fn capture(&mut self) -> Result<Selection, CaptureError> {
        self.ensure_com()?;

        let automation: IUIAutomation = unsafe {
            CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
                .map_err(|error| native_error("CoCreateInstance(CUIAutomation)", &error))?
        };
        let focused = unsafe {
            automation
                .GetFocusedElement()
                .map_err(|error| native_error("IUIAutomation::GetFocusedElement", &error))?
        };
        let pattern: IUIAutomationTextPattern = unsafe {
            focused
                .GetCurrentPatternAs(UIA_TextPatternId)
                .map_err(|_| CaptureError::Unsupported)?
        };
        let ranges = unsafe {
            pattern
                .GetSelection()
                .map_err(|error| native_error("IUIAutomationTextPattern::GetSelection", &error))?
        };
        let count = unsafe {
            ranges
                .Length()
                .map_err(|error| native_error("IUIAutomationTextRangeArray::Length", &error))?
        };
        if count <= 0 {
            return Err(CaptureError::NoSelection);
        }

        for index in 0..count {
            let range = unsafe {
                ranges.GetElement(index).map_err(|error| {
                    native_error("IUIAutomationTextRangeArray::GetElement", &error)
                })?
            };
            let text = unsafe {
                range
                    .GetText(-1)
                    .map_err(|error| native_error("IUIAutomationTextRange::GetText", &error))?
                    .to_string()
            };
            if text.trim().is_empty() {
                continue;
            }

            let bounds = bounding_rect(&range);
            let work_area = bounds
                .as_ref()
                .and_then(|rect| work_area_for_rect(rect).ok());

            return Ok(Selection {
                text,
                source: SelectionSource::UiAutomation,
                bounds,
                work_area,
            });
        }

        Err(CaptureError::NoSelection)
    }
}

fn bounding_rect(range: &IUIAutomationTextRange) -> Option<ScreenRect> {
    let array = unsafe { range.GetBoundingRectangles().ok()? };
    if array.is_null() {
        return None;
    }

    let result = unsafe { read_last_rect(array) };
    unsafe {
        let _ = SafeArrayDestroy(array);
    }
    result
}

unsafe fn read_last_rect(array: *mut windows::Win32::System::Com::SAFEARRAY) -> Option<ScreenRect> {
    let lower = SafeArrayGetLBound(array, 1).ok()?;
    let upper = SafeArrayGetUBound(array, 1).ok()?;
    if upper < lower {
        return None;
    }
    let len = usize::try_from(upper - lower + 1).ok()?;
    if len < 4 || len % 4 != 0 {
        return None;
    }

    let mut data: *mut c_void = ptr::null_mut();
    SafeArrayAccessData(array, &mut data).ok()?;
    let values = slice::from_raw_parts(data.cast::<f64>(), len);
    let start = len - 4;
    let rect = ScreenRect {
        x: values[start],
        y: values[start + 1],
        width: values[start + 2],
        height: values[start + 3],
    };
    let _ = SafeArrayUnaccessData(array);

    if rect.width <= 0.0 || rect.height <= 0.0 {
        None
    } else {
        Some(rect)
    }
}

fn native_error(operation: &'static str, error: &WindowsError) -> CaptureError {
    CaptureError::NativeFailure {
        operation,
        code: error.code().0,
    }
}
