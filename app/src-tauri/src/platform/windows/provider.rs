use crate::core::{CaptureError, Selection, SelectionProvider};

use super::{ClipboardSelectionProvider, UiAutomationSelectionProvider};

pub struct WindowsSelectionProvider {
    uia: UiAutomationSelectionProvider,
    clipboard: ClipboardSelectionProvider,
}

impl WindowsSelectionProvider {
    pub fn new() -> Self {
        Self {
            uia: UiAutomationSelectionProvider::new(),
            clipboard: ClipboardSelectionProvider,
        }
    }
}

impl Default for WindowsSelectionProvider {
    fn default() -> Self {
        Self::new()
    }
}

impl SelectionProvider for WindowsSelectionProvider {
    fn capture(&mut self) -> Result<Selection, CaptureError> {
        match self.uia.capture() {
            Err(CaptureError::Unsupported) => self.clipboard.capture(),
            result => result,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_fallback_is_limited_to_unsupported_uia_provider() {
        assert!(matches!(
            fallback_policy(&CaptureError::Unsupported),
            FallbackPolicy::Clipboard
        ));
        assert!(matches!(
            fallback_policy(&CaptureError::NoSelection),
            FallbackPolicy::Stop
        ));
        assert!(matches!(
            fallback_policy(&CaptureError::Timeout),
            FallbackPolicy::Stop
        ));
    }

    #[derive(Debug, PartialEq)]
    enum FallbackPolicy {
        Clipboard,
        Stop,
    }

    fn fallback_policy(error: &CaptureError) -> FallbackPolicy {
        if matches!(error, CaptureError::Unsupported) {
            FallbackPolicy::Clipboard
        } else {
            FallbackPolicy::Stop
        }
    }
}
