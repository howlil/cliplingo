use serde::Serialize;

use super::{Selection, Translation};

pub type RequestId = u64;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PopupErrorCode {
    NoSelection,
    ClipboardPreservationUnsupported,
    CaptureUnavailable,
    TranslationFailed,
}

impl PopupErrorCode {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::NoSelection => "no_selection",
            Self::ClipboardPreservationUnsupported => "clipboard_preservation_unsupported",
            Self::CaptureUnavailable => "capture_unavailable",
            Self::TranslationFailed => "translation_failed",
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum PopupState {
    Hidden,
    Capturing {
        request_id: RequestId,
    },
    Translating {
        request_id: RequestId,
        source_text: String,
    },
    Ready {
        request_id: RequestId,
        source_text: String,
        translated_text: String,
    },
    Error {
        request_id: RequestId,
        code: PopupErrorCode,
    },
}

impl PopupState {
    pub fn request_id(&self) -> Option<RequestId> {
        match self {
            Self::Hidden => None,
            Self::Capturing { request_id }
            | Self::Translating { request_id, .. }
            | Self::Ready { request_id, .. }
            | Self::Error { request_id, .. } => Some(*request_id),
        }
    }

    pub fn view_model(&self) -> PopupViewModel {
        match self {
            Self::Hidden => PopupViewModel::hidden(),
            Self::Capturing { .. } => PopupViewModel::capturing(),
            Self::Translating { source_text, .. } => PopupViewModel {
                status: "translating",
                source_text: Some(source_text.clone()),
                translated_text: None,
                error_code: None,
            },
            Self::Ready {
                source_text,
                translated_text,
                ..
            } => PopupViewModel {
                status: "ready",
                source_text: Some(source_text.clone()),
                translated_text: Some(translated_text.clone()),
                error_code: None,
            },
            Self::Error { code, .. } => PopupViewModel {
                status: "error",
                source_text: None,
                translated_text: None,
                error_code: Some(code.as_str()),
            },
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct PopupViewModel {
    pub status: &'static str,
    pub source_text: Option<String>,
    pub translated_text: Option<String>,
    pub error_code: Option<&'static str>,
}

impl PopupViewModel {
    pub fn hidden() -> Self {
        Self {
            status: "hidden",
            source_text: None,
            translated_text: None,
            error_code: None,
        }
    }

    pub fn capturing() -> Self {
        Self {
            status: "capturing",
            source_text: None,
            translated_text: None,
            error_code: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum ApplyResult {
    Applied(PopupState),
    Stale,
}

#[derive(Debug)]
pub struct PopupSession {
    next_request_id: RequestId,
    state: PopupState,
}

impl Default for PopupSession {
    fn default() -> Self {
        Self {
            next_request_id: 1,
            state: PopupState::Hidden,
        }
    }
}

impl PopupSession {
    pub fn snapshot(&self) -> PopupState {
        self.state.clone()
    }

    pub fn begin_request(&mut self) -> RequestId {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.saturating_add(1);
        self.state = PopupState::Capturing { request_id };
        request_id
    }

    pub fn mark_translating(
        &mut self,
        request_id: RequestId,
        selection: &Selection,
    ) -> ApplyResult {
        if !self.is_current(request_id) {
            return ApplyResult::Stale;
        }
        self.state = PopupState::Translating {
            request_id,
            source_text: selection.text.clone(),
        };
        ApplyResult::Applied(self.state.clone())
    }

    pub fn complete(&mut self, request_id: RequestId, translation: Translation) -> ApplyResult {
        if !self.is_current(request_id) {
            return ApplyResult::Stale;
        }
        let source_text = match &self.state {
            PopupState::Translating { source_text, .. } => source_text.clone(),
            _ => return ApplyResult::Stale,
        };
        self.state = PopupState::Ready {
            request_id,
            source_text,
            translated_text: translation.text,
        };
        ApplyResult::Applied(self.state.clone())
    }

    pub fn fail(&mut self, request_id: RequestId, code: PopupErrorCode) -> ApplyResult {
        if !self.is_current(request_id) {
            return ApplyResult::Stale;
        }
        self.state = PopupState::Error { request_id, code };
        ApplyResult::Applied(self.state.clone())
    }

    pub fn hide(&mut self) -> PopupState {
        self.state = PopupState::Hidden;
        self.state.clone()
    }

    fn is_current(&self, request_id: RequestId) -> bool {
        self.state.request_id() == Some(request_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{SelectionSource, Translation};

    fn selection(text: &str) -> Selection {
        Selection {
            text: text.to_owned(),
            source: SelectionSource::UiAutomation,
            bounds: None,
            work_area: None,
        }
    }

    #[test]
    fn new_session_is_hidden() {
        let session = PopupSession::default();
        assert_eq!(session.snapshot(), PopupState::Hidden);
    }

    #[test]
    fn stale_completion_cannot_replace_new_request() {
        let mut session = PopupSession::default();
        let first = session.begin_request();
        let second = session.begin_request();

        assert_eq!(
            session.complete(first, Translation { text: "old".into() }),
            ApplyResult::Stale
        );

        assert!(matches!(
            session.mark_translating(second, &selection("new")),
            ApplyResult::Applied(PopupState::Translating { .. })
        ));
        assert!(matches!(
            session.complete(
                second,
                Translation {
                    text: "new-result".into()
                }
            ),
            ApplyResult::Applied(PopupState::Ready { .. })
        ));
    }

    #[test]
    fn hide_invalidates_in_flight_completion() {
        let mut session = PopupSession::default();
        let request = session.begin_request();
        session.mark_translating(request, &selection("secret"));
        session.hide();

        assert_eq!(
            session.complete(request, Translation { text: "late".into() }),
            ApplyResult::Stale
        );
        assert_eq!(session.snapshot(), PopupState::Hidden);
    }
}
