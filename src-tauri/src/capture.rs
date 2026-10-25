//! Win32 foreground snapshot + one non-window-owning UI Automation MTA worker.
//! The shell never sees captured text. No source content is logged, persisted,
//! published to a Tauri event, uploaded, cached or shown in the preview popup.
//! One bounded pending request avoids unbounded COM work on rapid hotkeys.

use std::{
    sync::{
        mpsc::{self, SyncSender, TrySendError},
        Arc, Mutex,
    },
    thread,
};
use windows::{
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
use crate::translation::TranslationRequest;

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
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Queued => "capturing",
            Self::Verified => "selection_verified",
            Self::Empty => "no_selection",
            Self::Denied => "source_denied",
            Self::Unsupported => "selection_unsupported",
            Self::Busy => "capture_busy",
            Self::ProviderFailure => "provider_unavailable",
        }
    }
}

#[derive(Default)]
struct Telemetry {
    generation: u64,
    state: CaptureState,
    selection_units: usize,
    verified_count: u64,
}

impl Default for CaptureState {
    fn default() -> Self { Self::Idle }
}

impl Telemetry {
    fn next_request(&mut self) -> u64 {
        self.generation = self.generation.wrapping_add(1);
        self.state = CaptureState::Queued;
        self.selection_units = 0;
        self.generation
    }
    fn complete(&mut self, generation: u64, result: Result<String, CaptureDenied>) {
        if self.generation != generation { return; }
        match result {
            Ok(text) => {
                self.selection_units = text.encode_utf16().count();
                self.verified_count += 1;
                self.state = CaptureState::Verified;
            }
            Err(reason) => {
                self.selection_units = 0;
                self.state = state_for_error(reason);
            }
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
    telemetry: Arc<Mutex<Telemetry>>,
}
impl CaptureCoordinator {
    pub fn start(on_selection: Arc<dyn Fn(TranslationRequest) + Send + Sync>) -> Result<Self, String> {
        let (sender, receiver) = mpsc::sync_channel::<Request>(1);
        let telemetry = Arc::new(Mutex::new(Telemetry::default()));
        let status = telemetry.clone();
        thread::Builder::new().name("cliplingo-uia-mta".into()).spawn(move || {
            // A provider call can hang; the bounded queue limits pending work.
            let _apartment = match ComApartment::init() {
                Ok(value) => value,
                Err(_) => {
                    status.lock().unwrap().state = CaptureState::ProviderFailure;
                    return;
                }
            };
            let automation: IUIAutomation = match unsafe {
                CoCreateInstance(&CUIAutomation, None, CLSCTX_INPROC_SERVER)
            } {
                Ok(value) => value,
                Err(_) => {
                    status.lock().unwrap().state = CaptureState::ProviderFailure;
                    return;
                }
            };
            while let Ok(request) = receiver.recv() {
                if status.lock().unwrap().generation != request.generation { continue; }
                let result = read_selection(&automation, &request.source);
                // Hold the same generation owner through the handoff into
                // the translator: a new hotkey cannot be overtaken by an
                // older result after its generation was invalidated.
                finish_and_forward(&status, request, result, on_selection.as_ref());
            }
        }).map_err(|e| format!("Could not start UIA thread: {e}"))?;
        Ok(Self { sender, telemetry })
    }

    pub fn capture_hotkey(&self, cursor: Result<(i32, i32), String>) {
        // Each new hotkey (even rejected) invalidates older requests.
        let generation = self.telemetry.lock().unwrap().next_request();
        let source = match cursor {
            Ok(point) => SourceSnapshot::at_hotkey(point),
            Err(_) => Err(CaptureDenied::UnverifiableSource),
        };
        match source {
            Ok(source) => match self.sender.try_send(Request { generation, source }) {
                Ok(()) => {},
                Err(TrySendError::Full(_)) => self.set_if_current(generation, CaptureDenied::Busy),
                Err(TrySendError::Disconnected(_)) => self.set_if_current(generation, CaptureDenied::ProviderError),
            },
            Err(err) => self.set_if_current(generation, err),
        }
    }

    fn set_if_current(&self, generation: u64, error: CaptureDenied) {
        self.telemetry.lock().unwrap().complete(generation, Err(error));
    }

    pub fn status(&self) -> CaptureStatus {
        let state = self.telemetry.lock().unwrap();
        CaptureStatus {
            state: state.state,
            selection_units: state.selection_units,
            verified_count: state.verified_count,
            generation: state.generation,
        }
    }

    pub fn with_current<F: FnOnce()>(&self, generation: u64, present: F) {
        // Hold generation owner until native presentation has completed.
        // Otherwise a hotkey could invalidate after a check but before ShowWindow.
        let guard = self.telemetry.lock().unwrap();
        if guard.generation == generation {
            present();
        }
    }

    pub fn is_current(&self, generation: u64) -> bool {
        self.telemetry.lock().unwrap().generation == generation
    }

    pub fn invalidate(&self) {
        let mut state = self.telemetry.lock().unwrap();
        state.generation = state.generation.wrapping_add(1);
        state.selection_units = 0;
        state.state = CaptureState::Idle;
    }
}

fn finish_and_forward(
    status: &Mutex<Telemetry>,
    request: Request,
    result: Result<String, CaptureDenied>,
    on_selection: &dyn Fn(TranslationRequest),
) {
    let mut state = status.lock().unwrap();
    if state.generation != request.generation {
        return;
    }
    match result {
        Ok(text) => {
            // Do not clone or persist source text. The sensitive String moves
            // exactly once to the local translation worker.
            state.selection_units = text.encode_utf16().count();
            state.verified_count += 1;
            state.state = CaptureState::Verified;
            on_selection(TranslationRequest {
                generation: request.generation,
                anchor: request.source.cursor,
                source_hwnd: request.source.hwnd,
                text,
            });
        }
        Err(error) => state.complete(request.generation, Err(error)),
    }
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

pub fn is_foreground_hwnd(window: isize) -> bool {
    unsafe {
        let current = GetForegroundWindow();
        !current.is_null() && current as isize == window
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
    let same_root = if element_hwnd.0.is_null() {
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
    fn stale_uia_result_cannot_submit_to_translator() {
        use std::sync::atomic::{AtomicUsize, Ordering};
        let status = Mutex::new(Telemetry::default());
        let first = status.lock().unwrap().next_request();
        let second = status.lock().unwrap().next_request();
        let forwarded = AtomicUsize::new(0);
        let source = SourceSnapshot {
            hwnd: 12,
            pid: 345,
            process_name: "notepad.exe".into(),
            cursor: (10, 20),
        };
        let emit = |req: TranslationRequest| {
            assert_eq!(req.generation, second);
            assert_eq!(req.text, "current");
            forwarded.fetch_add(1, Ordering::Relaxed);
        };
        finish_and_forward(&status, Request {generation:first, source:source.clone()},
            Ok("stale".into()), &emit);
        finish_and_forward(&status, Request {generation:second, source},
            Ok("current".into()), &emit);
        assert_eq!(forwarded.load(Ordering::Relaxed), 1);
        let state = status.lock().unwrap();
        assert_eq!(state.verified_count, 1);
        assert_eq!(state.selection_units, 7);
    }

    #[test]
    fn stale_selection_cannot_override_latest_result() {
        let mut telemetry = Telemetry::default();
        let first = telemetry.next_request();
        let second = telemetry.next_request();
        telemetry.complete(first, Ok("old".into()));
        assert_eq!(telemetry.state, CaptureState::Queued);
        telemetry.complete(second, Ok("new".into()));
        assert_eq!(telemetry.state, CaptureState::Verified);
        assert_eq!(telemetry.selection_units, 3);
        assert_eq!(telemetry.verified_count, 1);
    }
    #[test]
    fn denied_newer_request_cancels_prior_success() {
        let mut telemetry = Telemetry::default();
        let first = telemetry.next_request();
        let second = telemetry.next_request();
        telemetry.complete(second, Err(CaptureDenied::UnverifiableSource));
        telemetry.complete(first, Ok("late".into()));
        assert_eq!(telemetry.state, CaptureState::Denied);
        assert_eq!(telemetry.verified_count, 0);
        assert_eq!(state_for_error(CaptureDenied::PasswordField), CaptureState::Denied);
    }
}
