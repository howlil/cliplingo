//! Capture authorization: fail closed before crossing the UIA boundary.
//! F1 intentionally has no clipboard, OCR, hover, elevation or cloud fallback.

pub const MAX_SELECTION_UNITS: usize = 8_192;

// App-level default blocklist is intentionally conservative, not a security
// guarantee against renamed processes. User-configurable exclusions come later.
const BLOCKED: &[&str] = &[
    "cliplingo.exe", "1password.exe", "bitwarden.exe",
    "keepass.exe", "keepassxc.exe", "lastpass.exe",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CaptureDenied {
    UnverifiableSource,
    OwnProcess,
    ExcludedApp,
    PasswordField,
    SelectionUnavailable,
    OversizedSelection,
    ProviderError,
    ForegroundChanged,
    Busy,
}

pub fn authorize_process(process_name: &str, pid: u32, self_pid: u32) -> Result<(), CaptureDenied> {
    if pid == 0 || process_name.is_empty() {
        return Err(CaptureDenied::UnverifiableSource);
    }
    if pid == self_pid {
        return Err(CaptureDenied::OwnProcess);
    }
    if BLOCKED.iter().any(|name| name.eq_ignore_ascii_case(process_name)) {
        return Err(CaptureDenied::ExcludedApp);
    }
    Ok(())
}

pub fn validate_selection(text: String) -> Result<String, CaptureDenied> {
    if text.trim().is_empty() {
        return Err(CaptureDenied::SelectionUnavailable);
    }
    if text.encode_utf16().count() > MAX_SELECTION_UNITS || text.contains('\0') {
        return Err(CaptureDenied::OversizedSelection);
    }
    // Preserve the selected span exactly; trimming changes user intent.
    Ok(text)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn unknown_self_and_excluded_processes_fail_closed() {
        assert_eq!(authorize_process("", 2, 1), Err(CaptureDenied::UnverifiableSource));
        assert_eq!(authorize_process("any.exe", 0, 1), Err(CaptureDenied::UnverifiableSource));
        assert_eq!(authorize_process("any.exe", 1, 1), Err(CaptureDenied::OwnProcess));
        assert_eq!(authorize_process("BitWarden.EXE", 2, 1), Err(CaptureDenied::ExcludedApp));
        assert_eq!(authorize_process("chrome.exe", 2, 1), Ok(()));
    }
    #[test]
    fn selection_preserves_whitespace_but_rejects_absence_and_overflow() {
        assert_eq!(validate_selection("  selected\r\n".into()).unwrap(), "  selected\r\n");
        assert_eq!(validate_selection("\t \r\n".into()), Err(CaptureDenied::SelectionUnavailable));
        assert_eq!(validate_selection("x".repeat(MAX_SELECTION_UNITS+1)), Err(CaptureDenied::OversizedSelection));
        assert_eq!(validate_selection("a\0b".into()), Err(CaptureDenied::OversizedSelection));
    }
    #[test]
    fn unicode_limit_counts_utf16_units_not_scalar_values() {
        assert!(validate_selection("😀".repeat(MAX_SELECTION_UNITS/2)).is_ok());
        assert!(validate_selection("😀".repeat(MAX_SELECTION_UNITS/2+1)).is_err());
    }
}
