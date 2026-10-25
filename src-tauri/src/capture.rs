//! Win32 foreground snapshot + one non-window-owning UI Automation MTA worker.
//! The shell never sees captured text. No source content is logged, persisted,
//! published to a Tauri event, uploaded, cached or shown in the preview popup.
//! One bounded pending request avoids unbounded COM work on rapid hotkeys.

use std::{
    sync::{
        atomic::{AtomicU64, AtomicU8, AtomicUsize, Ordering},
        mpsc::{self, SyncSender, TrySendError},
        Arc,
    },
    thread,
};
use windows::{
    core::Interface,
    Win32::{
        System::Com::{
            CoCreateInstance, CoInitializeEx, CoUninitialize,
            CLSCTX_INPROC_SERVER, COINIT_MULTITHREADED,
        },
        UI::Accessibility::{CUIAutomation, IUIAutomation, IUIAutomationTextPattern, UIA_TextPatternId},
    },
};
use windows_sys::Win32::{
    Foundation::HWND,
    System::Threading::{
        GetCurrentProcessId, OpenProcess, QueryFullProcessImageNameW,
        PROCESS_QUERY_LIMITED_INFORMATION,
    },
    Foundation::CloseHandle,
    UI::WindowsAndMessaging::{
        GetAncestor, GetForegroundWindow, GetWindowThreadProcessId, GA_ROOT,
    },
};
use crate::capture_policy::{authorize_process, validate_selection, CaptureDenied, MAX_SELECTION_UNITS};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum CaptureState {
    Idle = 0,
    Queued = 1,
    Verified = 2,
    Empty = 3,
    Denied = 4,
    Unsupported = 5,
    Busy = 6,
    ProviderFailure = 7,
}

impl CaptureState {
    fn from_u8(code: u8) -> Self {
        match code {
            1 => Self::Queued,
            2 => Self::Verified,
            3 => Self::Empty,
            4 => Self::Denied,
            5 => Self::Unsupported,
            6 => Self::Busy,
            7 => Self::ProviderFailure,
            _ => Self::Idle,
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Queued => "capturing",
            Self::Verified => "selection_verified_no_translation_engine",
            Self::Empty => "no_selection",
            Self::Denied => "source_denied",
            Self::Unsupported => "selection_unsupported",
            Self::Busy => "capture_busy",
            Self::ProviderFailure => "provider_unavailable",
        }
    }
}

struct Telemetry {
    state: AtomicU8,
    selection_units: AtomicUsize,
    verified_count: AtomicU64,
}
impl Default for Telemetry {
    fn default() -> Self {
        Self {
            state: AtomicU8::new(CaptureState::Idle as u8),
            selection_units: AtomicUsize::new(0),
            verified_count: AtomicU64::new(0),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CaptureStatus {
    pub state: CaptureState,
    pub selection_units: usize,
    pub verified_count: u64,
    pub generation: u64,
}

// The source snapshot must be taken before dispatch; the COM worker never
// resolves an arbitrary foreground app to select on its own.
#[derive(Clone, Debug)]
struct SourceSnapshot {
    hwnd: isize,
    pid: u32,
    process_name: String,
    cursor: (i32, i32),
}

impl SourceSnapshot {
    fn at_hotkey(cursor: (i32, i32)) -> Result<Self, CaptureDenied> {
        unsafe {
            // SAFETY: HWND/process APIs are queried synchronously and handles
            // are closed in this scope. We fail closed when provenance is unknown.
            let hwnd = GetForegroundWindow();
            if hwnd.is_null() { return Err(CaptureDenied::UnverifiableSource); }
            let mut pid: u32 = 0;
            if GetWindowThreadProcessId(hwnd, &mut pid) == 0 {
                return Err(CaptureDenied::UnverifiableSource);
            }
            let self_pid = GetCurrentProcessId();
            if pid == self_pid { return Err(CaptureDenied::OwnProcess); }
            let process = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
            if process.is_null() { return Err(CaptureDenied::UnverifiableSource); }
            let mut buf = [0u16; 32_768];
            let mut count = buf.len() as u32;
            let ok = QueryFullProcessImageNameW(process, 0, buf.as_mut_ptr(), &mut count);
            CloseHandle(process);
            if ok == 0 || count == 0 { return Err(CaptureDenied::UnverifiableSource); }
            let path = String::from_utf16(&buf[..count as usize])
                .map_err(|_| CaptureDenied::UnverifiableSource)?;
            let process_name = path.rsplit(['\\', '/']).next().unwrap_or_default().to_owned();
            authorize_process(&process_name, pid, self_pid)?;
            Ok(Self { hwnd: hwnd as isize, pid, process_name, cursor })
        }
    }
}

struct Request {
    generation: u64,
    source: SourceSnapshot,
}

// COM objects never leave this non-window-owning MTA thread.
struct ComApartment;
impl ComApartment {
    fn init() -> Result<Self, CaptureDenied> {
        unsafe {
            CoInitializeEx(None, COINIT_MULTITHREADED)
                .ok().map_err(|_| CaptureDenied::ProviderError)?;
        }
        Ok(Self)
    }
}
impl Drop for ComApartment {
    fn drop(&mut self) {
        unsafe { CoUninitialize(); }
    }
}

pub struct CaptureCoordinator {
    sender: SyncSender<Request>,
    generation: Arc<AtomicU64>,
    telemetry: Arc<Telemetry>,
}
impl CaptureCoordinator {
    pub fn start() -> Result<Self, String> {
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let generation = Arc::new(AtomicU64::new(0));
        let telemetry = Arc::new(Telemetry::default());
        let current = generation.clone();
        let status = telemetry.clone();
        thread::Builder::new().name("cliplingo-uia-mta".into()).spawn(move || {
            // A provider call may hang; unlike async timeouts this worker does
            // not pretend to interrupt COM. The queue stays bounded at 1.
            let _apartment = match ComApartment::init() {
                Ok(value) => value,
                Err(_) => {
                    status.state.store(CaptureState::ProviderFailure as u8, Ordering::Release);
                    return;
                }
            };
            let automation: IUIAutomation = match unsafe {
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            } {
                Ok(value) => value,
                Err(_) => {
                    status.state.store(CaptureState::ProviderFailure as u8, Ordering::Release);
                    return;
                }
            };
            while let Ok(request) = receiver.recv() {
                if !is_current(&current, request.generation) { continue; }
                let result = read_selection(&automation, &request.source);
                // Results from an older request cannot change UI or status.
                if !is_current(&current, request.generation) { continue; }
                match result {
                    Ok(text) => {
                        // F1 verifies that real UIA selection exists; drop immediately.
                        // F2 will add an explicitly approved inference handoff.
                        status.selection_units.store(text.encode_utf16().count(), Ordering::Release);
                        status.verified_count.fetch_add(1, Ordering::Relaxed);
                        status.state.store(CaptureState::Verified as u8, Ordering::Release);
                    }
                    Err(reason) => {
                        status.selection_units.store(0, Ordering::Release);
                        status.state.store(state_for_error(reason) as u8, Ordering::Release);
                    }
                }
            }
        }).map_err(|e| format!("Could not start UIA thread: {e}"))?;
        Ok(Self { sender, generation, telemetry })
    }

    pub fn capture_hotkey(&self, cursor: Result<(i32, i32), String>) {
        // Even denied requests invalidate previous in-flight captures.
        let generation = self.generation.fetch_add(1, Ordering::AcqRel) + 1;
        self.telemetry.selection_units.store(0, Ordering::Release);
        self.telemetry.state.store(CaptureState::Queued as u8, Ordering::Release);
        let source = match cursor {
            Ok(point) => SourceSnapshot::at_hotkey(point),
            Err(_) => Err(CaptureDenied::UnverifiableSource),
        };
        match source {
            Ok(source) => match self.sender.try_send(Request { generation, source }) {
                Ok(()) => {},
                Err(TrySendError::Full(_)) => self.set_if_current(generation, CaptureState::Busy),
                Err(TrySendError::Disconnected(_)) => self.set_if_current(generation, CaptureState::ProviderFailure),
            },
            Err(err) => self.set_if_current(generation, state_for_error(err)),
        }
    }

    fn set_if_current(&self, generation: u64, state: CaptureState) {
        if is_current(&self.generation, generation) {
            self.telemetry.state.store(state as u8, Ordering::Release);
        }
    }

    pub fn status(&self) -> CaptureStatus {
        CaptureStatus {
            state: CaptureState::from_u8(self.telemetry.state.load(Ordering::Acquire)),
            selection_units: self.telemetry.selection_units.load(Ordering::Acquire),
            verified_count: self.telemetry.verified_count.load(Ordering::Relaxed),
            generation: self.generation.load(Ordering::Acquire),
        }
    }

    pub fn invalidate(&self) {
        self.generation.fetch_add(1, Ordering::AcqRel);
        self.telemetry.selection_units.store(0, Ordering::Release);
        self.telemetry.state.store(CaptureState::Idle as u8, Ordering::Release);
    }
}

fn is_current(counter: &AtomicU64, generation: u64) -> bool {
    counter.load(Ordering::Acquire) == generation
}
fn state_for_error(error: CaptureDenied) -> CaptureState {
    match error {
        CaptureDenied::SelectionUnavailable => CaptureState::Empty,
        CaptureDenied::OversizedSelection => CaptureState::Unsupported,
        CaptureDenied::ProviderError => CaptureState::ProviderFailure,
        CaptureDenied::Busy => CaptureState::Busy,
        CaptureDenied::ExcludedApp | CaptureDenied::OwnProcess |
        CaptureDenied::PasswordField | CaptureDenied::UnverifiableSource |
        CaptureDenied::ForegroundChanged => CaptureState::Denied,
    }
}

fn source_still_foreground(source: &SourceSnapshot) -> bool {
    unsafe {
        let hwnd = GetForegroundWindow();
        if hwnd.is_null() || hwnd as isize != source.hwnd { return false; }
        let mut pid = 0;
        GetWindowThreadProcessId(hwnd, &mut pid);
        pid == source.pid
    }
}

fn read_selection(automation: &IUIAutomation, source: &SourceSnapshot) -> Result<String, CaptureDenied> {
    if !source_still_foreground(source) {
        return Err(CaptureDenied::ForegroundChanged);
    }
    let element = unsafe { automation.GetFocusedElement() }
        .map_err(|_| CaptureDenied::SelectionUnavailable)?;
    let element_pid = unsafe { element.CurrentProcessId() }
        .map_err(|_| CaptureDenied::UnverifiableSource)? as u32;
    // Multi-process browsers may host focused controls in a child process.
    // Only accept cross-process UIA identity if the focused control supplies
    // a native child HWND proven to belong to our original foreground HWND.
    let element_hwnd = unsafe { element.CurrentNativeWindowHandle() }
        .map_err(|_| CaptureDenied::UnverifiableSource)?;
    let same_root = if element_hwnd.is_null() {
        false
    } else {
        let root = unsafe { GetAncestor(element_hwnd.0 as HWND, GA_ROOT) };
        !root.is_null() && root as isize == source.hwnd
    };
    if element_pid != source.pid && !same_root {
        return Err(CaptureDenied::UnverifiableSource);
    }
    if unsafe { element.CurrentIsPassword() }
        .map_err(|_| CaptureDenied::UnverifiableSource)?.as_bool() {
        return Err(CaptureDenied::PasswordField);
    }
    let pattern: IUIAutomationTextPattern = unsafe { element.GetCurrentPatternAs(UIA_TextPatternId) }
        .map_err(|_| CaptureDenied::SelectionUnavailable)?;
    let ranges = unsafe { pattern.GetSelection() }
        .map_err(|_| CaptureDenied::ProviderError)?;
    let count = unsafe { ranges.Length() }.map_err(|_| CaptureDenied::ProviderError)?;
    // Disjoint selections require explicit product semantics; never silently
    // join unrelated ranges or translate the whole document.
    if count != 1 { return Err(CaptureDenied::SelectionUnavailable); }
    let range = unsafe { ranges.GetElement(0) }.map_err(|_| CaptureDenied::ProviderError)?;
    let bstr = unsafe { range.GetText((MAX_SELECTION_UNITS+1) as i32) }
        .map_err(|_| CaptureDenied::ProviderError)?;
    let text = validate_selection(bstr.to_string())?;
    if !source_still_foreground(source) {
        return Err(CaptureDenied::ForegroundChanged);
    }
    let _ = (source.cursor, &source.process_name); // Immutable provenance retained for F2.
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn stale_generation_cannot_take_result_ownership() {
        let generation = AtomicU64::new(1);
        assert!(is_current(&generation, 1));
        generation.store(2, Ordering::Release);
        assert!(!is_current(&generation, 1));
        assert!(is_current(&generation, 2));
    }
    #[test]
    fn capture_state_is_closed_over_untrusted_codes() {
        assert_eq!(CaptureState::from_u8(255), CaptureState::Idle);
        assert_eq!(state_for_error(CaptureDenied::PasswordField), CaptureState::Denied);
        assert_eq!(state_for_error(CaptureDenied::SelectionUnavailable), CaptureState::Empty);
    }
}
