//! Minimal native UIA property surface for the existing no-activate GDI popup.
//! Only the translated TARGET is exposed; no source text, clipboard or history.
//! Full ITextProvider/ITextRangeProvider and Narrator/NVDA acceptance remain open.
use crate::native_text_document::NativeTextDocument;
use std::{
    cell::RefCell,
    ffi::c_void,
    sync::{atomic::{AtomicU32, Ordering, fence}, Arc, Mutex},
};
use windows::{
    core::{implement, BSTR, Error, GUID, HRESULT, IUnknown, Interface, Result},
    Win32::{
        Foundation::HWND,
        System::Variant::VARIANT,
        UI::Accessibility::{
            IRawElementProviderSimple, IRawElementProviderSimple_Impl,
            ProviderOptions, ProviderOptions_ServerSideProvider,
            UiaHostProviderFromHwnd, UiaRootObjectId,
            UIA_PATTERN_ID, UIA_PROPERTY_ID,
            UIA_ControlTypePropertyId, UIA_IsContentElementPropertyId,
            UIA_IsControlElementPropertyId, UIA_IsKeyboardFocusablePropertyId,
            UIA_NamePropertyId, UIA_TextControlTypeId,
        },
    },
};

#[link(name = "Uiautomationcore")]
unsafe extern "system" {
    // The Windows binding accepts an interface reference; the raw signature also
    // allows the documented NULL provider on WM_DESTROY to drop UIA's event map.
    fn UiaReturnRawElementProvider(
        hwnd: *mut c_void,
        wparam: usize,
        lparam: isize,
        provider: *mut c_void,
    ) -> isize;
}

type Target = Arc<Mutex<NativeTextDocument>>;

#[implement(IRawElementProviderSimple)]
struct NativeTargetProvider {
    hwnd: isize,
    target: Target,
}

impl IRawElementProviderSimple_Impl for NativeTargetProvider_Impl {
    fn ProviderOptions(&self) -> Result<ProviderOptions> {
        Ok(ProviderOptions_ServerSideProvider)
    }

    fn GetPatternProvider(&self, _pattern: UIA_PATTERN_ID) -> Result<IUnknown> {
        // windows-rs' generated return type is non-nullable IUnknown and
        // cannot represent the required S_OK + null output. This delegate
        // is never returned to UIA directly: PatternSafeProvider wraps its
        // COM ABI and explicitly zeroes unsupported pattern outputs.
        Err(Error::from(HRESULT(0x80004001u32 as i32))) // E_NOTIMPL
    }

    fn GetPropertyValue(&self, property: UIA_PROPERTY_ID) -> Result<VARIANT> {
        match property {
            UIA_ControlTypePropertyId => Ok(VARIANT::from(UIA_TextControlTypeId.0)),
            UIA_NamePropertyId => {
                // COM/UIA clients may call this on a non-window thread. Never
                // borrow thread-local GDI state or HWND objects from here.
                let value = match self.target.lock() {
                    Ok(text) => text.name(),
                    Err(poisoned) => poisoned.into_inner().name(),
                };
                Ok(VARIANT::from(BSTR::from(value)))
            }
            UIA_IsKeyboardFocusablePropertyId => Ok(VARIANT::from(false)),
            UIA_IsContentElementPropertyId | UIA_IsControlElementPropertyId =>
                Ok(VARIANT::from(true)),
            _ => Ok(VARIANT::default()),
        }
    }

    fn HostRawElementProvider(&self) -> Result<IRawElementProviderSimple> {
        // The HWND is valid while owned by NativePopup. A previously retained
        // provider can only hold its cleared target after destroy.
        unsafe { UiaHostProviderFromHwnd(HWND(self.hwnd as *mut c_void)) }
    }
}

// The generated windows-rs trait represents GetPatternProvider as
// Result<IUnknown>, which cannot express success with a NULL COM pointer.
// A thin, reference-counted ABI wrapper exposes the generated provider's
// three implemented operations but owns the unsupported-pattern ABI exactly.
// Only the wrapper is returned to UI Automation.
const S_OK: HRESULT = HRESULT(0);
const E_POINTER: HRESULT = HRESULT(0x80004003u32 as i32);
const E_NOINTERFACE: HRESULT = HRESULT(0x80004002u32 as i32);

#[repr(C)]
struct PatternSafeVtable {
    query_interface: unsafe extern "system" fn(*mut c_void, *const GUID, *mut *mut c_void) -> HRESULT,
    add_ref: unsafe extern "system" fn(*mut c_void) -> u32,
    release: unsafe extern "system" fn(*mut c_void) -> u32,
    provider_options: unsafe extern "system" fn(*mut c_void, *mut ProviderOptions) -> HRESULT,
    get_pattern: unsafe extern "system" fn(*mut c_void, UIA_PATTERN_ID, *mut *mut c_void) -> HRESULT,
    get_property: unsafe extern "system" fn(*mut c_void, UIA_PROPERTY_ID, *mut VARIANT) -> HRESULT,
    host_provider: unsafe extern "system" fn(*mut c_void, *mut *mut c_void) -> HRESULT,
}

#[repr(C)]
struct PatternSafeProvider {
    vtable: *const PatternSafeVtable,
    refs: AtomicU32,
    delegate: IRawElementProviderSimple,
}

unsafe extern "system" fn abi_query_interface(
    this: *mut c_void, iid: *const GUID, output: *mut *mut c_void
) -> HRESULT {
    if output.is_null() { return E_POINTER; }
    unsafe {
        *output = std::ptr::null_mut();
        if iid.is_null() { return E_POINTER; }
        if *iid == IRawElementProviderSimple::IID || *iid == IUnknown::IID {
            abi_add_ref(this);
            *output = this;
            S_OK
        } else {
            E_NOINTERFACE
        }
    }
}

unsafe extern "system" fn abi_add_ref(this: *mut c_void) -> u32 {
    let this = unsafe { &*(this as *const PatternSafeProvider) };
    this.refs.fetch_add(1, Ordering::Relaxed) + 1
}

unsafe extern "system" fn abi_release(this: *mut c_void) -> u32 {
    let provider = unsafe { &*(this as *const PatternSafeProvider) };
    let refs = provider.refs.fetch_sub(1, Ordering::Release) - 1;
    if refs == 0 {
        fence(Ordering::Acquire);
        unsafe { drop(Box::from_raw(this as *mut PatternSafeProvider)); }
    }
    refs
}

unsafe extern "system" fn abi_provider_options(
    this: *mut c_void, output: *mut ProviderOptions
) -> HRESULT {
    if output.is_null() { return E_POINTER; }
    let provider = unsafe { &*(this as *const PatternSafeProvider) };
    match unsafe { provider.delegate.ProviderOptions() } {
        Ok(value) => { unsafe { output.write(value); } S_OK },
        Err(error) => error.code(),
    }
}

unsafe extern "system" fn abi_get_pattern(
    _this: *mut c_void, _pattern: UIA_PATTERN_ID, output: *mut *mut c_void
) -> HRESULT {
    if output.is_null() { return E_POINTER; }
    // Windows passes uninitialized output storage. Initialize it even when
    // the requested pattern has never been supported by this GDI renderer.
    unsafe { *output = std::ptr::null_mut(); }
    S_OK
}

unsafe extern "system" fn abi_get_property(
    this: *mut c_void, property: UIA_PROPERTY_ID, output: *mut VARIANT
) -> HRESULT {
    if output.is_null() { return E_POINTER; }
    let provider = unsafe { &*(this as *const PatternSafeProvider) };
    match unsafe { provider.delegate.GetPropertyValue(property) } {
        Ok(value) => { unsafe { output.write(value); } S_OK },
        Err(error) => error.code(),
    }
}

unsafe extern "system" fn abi_host_provider(
    this: *mut c_void, output: *mut *mut c_void
) -> HRESULT {
    if output.is_null() { return E_POINTER; }
    unsafe { *output = std::ptr::null_mut(); }
    let provider = unsafe { &*(this as *const PatternSafeProvider) };
    match unsafe { provider.delegate.HostRawElementProvider() } {
        Ok(value) => { unsafe { *output = value.into_raw(); } S_OK },
        Err(error) => error.code(),
    }
}

static PATTERN_SAFE_VTABLE: PatternSafeVtable = PatternSafeVtable {
    query_interface: abi_query_interface,
    add_ref: abi_add_ref,
    release: abi_release,
    provider_options: abi_provider_options,
    get_pattern: abi_get_pattern,
    get_property: abi_get_property,
    host_provider: abi_host_provider,
};

fn wrap_pattern_safe(delegate: IRawElementProviderSimple) -> IRawElementProviderSimple {
    let boxed = Box::new(PatternSafeProvider {
        vtable: &PATTERN_SAFE_VTABLE,
        refs: AtomicU32::new(1),
        delegate,
    });
    // SAFETY: PatternSafeProvider starts with the seven-method COM vtable
    // for IRawElementProviderSimple, owns one reference, and implements
    // QueryInterface/AddRef/Release with atomic lifetime management.
    unsafe { IRawElementProviderSimple::from_raw(Box::into_raw(boxed).cast()) }
}

struct PopupUiaState {
    target: Target,
    provider: IRawElementProviderSimple,
}

thread_local! {
    // WM_GETOBJECT is dispatched on the native window thread; the COM provider
    // itself owns an Arc<Mutex> for calls originating on accessibility threads.
    static POPUP_UIA: RefCell<Option<PopupUiaState>> = const { RefCell::new(None) };
}

pub(super) fn install(hwnd: isize) {
    let target = Arc::new(Mutex::new(NativeTextDocument::default()));
    let delegate: IRawElementProviderSimple = NativeTargetProvider {
        hwnd,
        target: target.clone(),
    }.into();
    let provider = wrap_pattern_safe(delegate);
    POPUP_UIA.with(|state| {
        *state.borrow_mut() = Some(PopupUiaState { target, provider });
    });
}

pub(super) fn set_target(translated: &str) {
    POPUP_UIA.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
            if let Ok(mut value) = state.target.lock() {
                value.replace(translated);
            }
        }
    });
}

pub(super) fn clear_target() {
    POPUP_UIA.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
            if let Ok(mut value) = state.target.lock() {
                value.clear();
            }
        }
    });
}

/// Respond only to the UIA root request; leave other accessibility IDs to the
/// default Win32 provider. The UIA call occurs after the RefCell borrow ends:
/// UIA can synchronously re-enter WM_GETOBJECT.
pub(super) fn get_object(hwnd: *mut c_void, wparam: usize, lparam: isize) -> Option<isize> {
    if lparam as i32 != UiaRootObjectId {
        return None;
    }
    let provider = POPUP_UIA.with(|state| {
        let state = state.borrow();
        let current = state.as_ref()?;
        let has_text = current.target.lock().map(|s| !s.is_empty()).unwrap_or(false);
        has_text.then(|| current.provider.clone())
    });
    provider.map(|provider| unsafe {
        UiaReturnRawElementProvider(hwnd, wparam, lparam, provider.as_raw())
    })
}

pub(super) fn window_destroyed(hwnd: *mut c_void) {
    clear_target();
    // Release the reference map for this window (Microsoft's WM_DESTROY rule).
    unsafe { UiaReturnRawElementProvider(hwnd, 0, 0, std::ptr::null_mut()); }
    POPUP_UIA.with(|state| *state.borrow_mut() = None);
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unsupported_pattern_returns_s_ok_and_initializes_null_output() {
        let target = Arc::new(Mutex::new(NativeTextDocument::default()));
        let delegate: IRawElementProviderSimple = NativeTargetProvider {
            hwnd: 0,
            target,
        }.into();
        let provider = wrap_pattern_safe(delegate);
        // This assertion must exercise the actual COM vtable, not the
        // convenience method (which rejects a NULL IUnknown on success).
        let vtable = unsafe {
            *(provider.as_raw() as *const *const PatternSafeVtable)
        };
        let mut output = 1usize as *mut c_void; // deliberate non-null sentinel
        let result = unsafe {
            ((*vtable).get_pattern)(
                provider.as_raw(), UIA_PATTERN_ID(10014), &mut output
            )
        };
        assert_eq!(result, S_OK);
        assert!(output.is_null());

        let null_output = unsafe {
            ((*vtable).get_pattern)(
                provider.as_raw(), UIA_PATTERN_ID(10014), std::ptr::null_mut()
            )
        };
        assert_eq!(null_output, E_POINTER);

        let mut queried = std::ptr::null_mut();
        let qi = unsafe {
            ((*vtable).query_interface)(provider.as_raw(),
                &IRawElementProviderSimple::IID, &mut queried)
        };
        assert_eq!(qi, S_OK);
        assert_eq!(queried, provider.as_raw());
        let refs = unsafe { ((*vtable).release)(queried) };
        assert!(refs >= 1);
    }

    #[test]
    fn provider_target_follows_replacement_and_is_cleared() {
        install(0);
        set_target("Selamat datang. Paragraf kedua.\nBaris terakhir.");
        POPUP_UIA.with(|slot| {
            let slot = slot.borrow();
            assert_eq!(slot.as_ref().unwrap().target.lock().unwrap().name(),
                "Selamat datang. Paragraf kedua.\nBaris terakhir.");
        });
        // Query the *actual COM interface* rather than only checking the Rust
        // Mutex; this guards the UIA Name property mapping for long content.
        let provider = POPUP_UIA.with(|slot| {
            slot.borrow().as_ref().unwrap().provider.clone()
        });
        let name_property = unsafe { provider.GetPropertyValue(UIA_NamePropertyId) }
            .expect("COM Name property");
        let exposed = BSTR::try_from(&name_property).expect("Name must be BSTR");
        assert_eq!(exposed.to_string(), "Selamat datang. Paragraf kedua.\nBaris terakhir.");
        set_target("Hasil B");
        POPUP_UIA.with(|slot| {
            let slot = slot.borrow();
            assert_eq!(slot.as_ref().unwrap().target.lock().unwrap().name(), "Hasil B");
        });
        let name_property = unsafe { provider.GetPropertyValue(UIA_NamePropertyId) }
            .expect("new COM Name property");
        assert_eq!(BSTR::try_from(&name_property).unwrap().to_string(), "Hasil B");
        clear_target();
        let name_property = unsafe { provider.GetPropertyValue(UIA_NamePropertyId) }
            .expect("hidden COM Name property");
        assert_eq!(BSTR::try_from(&name_property).unwrap().to_string(), "");
        POPUP_UIA.with(|slot| {
            assert!(slot.borrow().as_ref().unwrap().target.lock().unwrap().is_empty());
        });
        // Do not call WM_DESTROY with a fake HWND in a unit test.
        POPUP_UIA.with(|slot| *slot.borrow_mut() = None);
    }
}
