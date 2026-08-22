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
            Err(error) if should_fallback_to_clipboard(&error) => self.clipboard.capture(),
            result => result,
        }
    }
}

fn should_fallback_to_clipboard(error: &CaptureError) -> bool {
    matches!(error, CaptureError::Unsupported)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn clipboard_fallback_is_limited_to_unsupported_uia_provider() {
        assert!(should_fallback_to_clipboard(&CaptureError::Unsupported));
        assert!(!should_fallback_to_clipboard(&CaptureError::NoSelection));
        assert!(!should_fallback_to_clipboard(&CaptureError::Timeout));
        assert!(!should_fallback_to_clipboard(
            &CaptureError::ClipboardUnavailable
        ));
    }
}
