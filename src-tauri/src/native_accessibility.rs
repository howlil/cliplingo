//! Minimal native UIA property surface for the existing no-activate GDI popup.
//! Only the translated TARGET is exposed; no source text, clipboard or history.
//! Full ITextProvider/ITextRangeProvider and Narrator/NVDA acceptance remain open.
use std::{
    cell::RefCell,
    ffi::c_void,
    sync::{Arc, Mutex},
};
use windows::{
    core::{implement, BSTR, Error, IUnknown, Interface, Result},
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

type Target = Arc<Mutex<String>>;

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
        // Intentionally no fake TextPattern until full range/navigation semantics
        // are implemented and verified on an interactive Windows desktop.
        Err(Error::from(windows::core::HRESULT(0)))
    }

    fn GetPropertyValue(&self, property: UIA_PROPERTY_ID) -> Result<VARIANT> {
        match property {
            UIA_ControlTypePropertyId => Ok(VARIANT::from(UIA_TextControlTypeId.0)),
            UIA_NamePropertyId => {
                // COM/UIA clients may call this on a non-window thread. Never
                // borrow thread-local GDI state or HWND objects from here.
                let value = match self.target.lock() {
                    Ok(text) => text.clone(),
                    Err(poisoned) => poisoned.into_inner().clone(),
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
    let target = Arc::new(Mutex::new(String::new()));
    let provider: IRawElementProviderSimple = NativeTargetProvider {
        hwnd,
        target: target.clone(),
    }.into();
    POPUP_UIA.with(|state| {
        *state.borrow_mut() = Some(PopupUiaState { target, provider });
    });
}

pub(super) fn set_target(translated: &str) {
    POPUP_UIA.with(|state| {
        if let Some(state) = state.borrow().as_ref() {
            if let Ok(mut value) = state.target.lock() {
                value.clear();
                value.push_str(translated);
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
    fn provider_target_follows_replacement_and_is_cleared() {
        install(0);
        set_target("Selamat datang. Paragraf kedua.\nBaris terakhir.");
        POPUP_UIA.with(|slot| {
            let slot = slot.borrow();
            assert_eq!(slot.as_ref().unwrap().target.lock().unwrap().as_str(),
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
            assert_eq!(slot.as_ref().unwrap().target.lock().unwrap().as_str(), "Hasil B");
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
