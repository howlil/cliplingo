use std::sync::{Arc, Condvar, Mutex};
use std::thread;

use crate::core::{
    place_popup, ApplyResult, CaptureError, PopupErrorCode, PopupPort, PopupSession, PopupState,
    PopupViewModel, RequestId, ScreenSize, SelectionProvider, Translation, TranslationError,
    TranslationRequest, Translator,
};

const POPUP_SIZE: ScreenSize = ScreenSize {
    width: 420.0,
    height: 180.0,
};
const POPUP_MARGIN: f64 = 8.0;

#[derive(Debug)]
pub struct PendingSlot<T> {
    value: Mutex<Option<T>>,
    wake: Condvar,
}

impl<T> Default for PendingSlot<T> {
    fn default() -> Self {
        Self {
            value: Mutex::new(None),
            wake: Condvar::new(),
        }
    }
}

impl<T> PendingSlot<T> {
    pub fn submit(&self, value: T) {
        let mut slot = self.value.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        *slot = Some(value);
        self.wake.notify_one();
    }

    pub fn take_now(&self) -> Option<T> {
        self.value
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .take()
    }

    fn wait_take(&self) -> T {
        let mut slot = self.value.lock().unwrap_or_else(|poisoned| poisoned.into_inner());
        loop {
            if let Some(value) = slot.take() {
                return value;
            }
            slot = self
                .wake
                .wait(slot)
                .unwrap_or_else(|poisoned| poisoned.into_inner());
        }
    }
}

pub struct FakeTranslator;

impl Translator for FakeTranslator {
    fn translate(
        &mut self,
        request: &TranslationRequest,
    ) -> Result<Translation, TranslationError> {
        Ok(Translation {
            text: format!("[FAKE] {}", request.text),
        })
    }
}

pub struct InteractionCoordinator {
    session: Arc<Mutex<PopupSession>>,
    pending: Arc<PendingSlot<RequestId>>,
    popup: Arc<dyn PopupPort>,
}

impl InteractionCoordinator {
    pub fn start(
        mut provider: Box<dyn SelectionProvider>,
        mut translator: Box<dyn Translator>,
        popup: Arc<dyn PopupPort>,
    ) -> Arc<Self> {
        let coordinator = Arc::new(Self {
            session: Arc::new(Mutex::new(PopupSession::default())),
            pending: Arc::new(PendingSlot::default()),
            popup,
        });

        let session = Arc::clone(&coordinator.session);
        let pending = Arc::clone(&coordinator.pending);
        let popup = Arc::clone(&coordinator.popup);
        thread::Builder::new()
            .name("cliplingo-interaction".into())
            .spawn(move || loop {
                let request_id = pending.wait_take();
                Self::process_request(
                    request_id,
                    provider.as_mut(),
                    translator.as_mut(),
                    session.as_ref(),
                    popup.as_ref(),
                );
            })
            .expect("failed to start interaction worker");

        coordinator
    }

    pub fn trigger(&self) -> RequestId {
        let (request_id, model) = {
            let mut session = self
                .session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            let request_id = session.begin_request();
            (request_id, session.snapshot().view_model())
        };

        self.popup.show(model);
        self.pending.submit(request_id);
        request_id
    }

    pub fn dismiss(&self) {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .hide();
        self.popup.hide();
    }

    pub fn snapshot(&self) -> PopupViewModel {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .snapshot()
            .view_model()
    }

    pub fn is_visible(&self) -> bool {
        !matches!(
            self.session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .snapshot(),
            PopupState::Hidden
        )
    }

    fn process_request(
        request_id: RequestId,
        provider: &mut dyn SelectionProvider,
        translator: &mut dyn Translator,
        session: &Mutex<PopupSession>,
        popup: &dyn PopupPort,
    ) {
        let selection = match provider.capture() {
            Ok(selection) => selection,
            Err(error) => {
                Self::publish_error(request_id, capture_error_code(&error), session, popup);
                return;
            }
        };

        if let (Some(anchor), Some(work_area)) = (&selection.bounds, &selection.work_area) {
            let position = place_popup(anchor, &POPUP_SIZE, work_area, POPUP_MARGIN);
            popup.move_to(position.x, position.y);
        }

        let state = {
            let mut session = session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            session.mark_translating(request_id, &selection)
        };
        let ApplyResult::Applied(state) = state else {
            return;
        };
        popup.update(state.view_model());

        let translation = translator.translate(&TranslationRequest {
            text: selection.text,
        });
        let state = {
            let mut session = session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match translation {
                Ok(value) => session.complete(request_id, value),
                Err(_) => session.fail(request_id, PopupErrorCode::TranslationFailed),
            }
        };
        if let ApplyResult::Applied(state) = state {
            popup.update(state.view_model());
        }
    }

    fn publish_error(
        request_id: RequestId,
        code: PopupErrorCode,
        session: &Mutex<PopupSession>,
        popup: &dyn PopupPort,
    ) {
        let result = session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .fail(request_id, code);
        if let ApplyResult::Applied(state) = result {
            popup.update(state.view_model());
        }
    }
}

fn capture_error_code(error: &CaptureError) -> PopupErrorCode {
    match error {
        CaptureError::NoSelection => PopupErrorCode::NoSelection,
        CaptureError::ClipboardPreservationUnsupported => {
            PopupErrorCode::ClipboardPreservationUnsupported
        }
        _ => PopupErrorCode::CaptureUnavailable,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ScreenRect, Selection, SelectionSource};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::{Duration, Instant};

    #[test]
    fn pending_slot_keeps_only_latest() {
        let slot = PendingSlot::default();
        slot.submit(1);
        slot.submit(2);
        slot.submit(3);
        assert_eq!(slot.take_now(), Some(3));
        assert_eq!(slot.take_now(), None);
    }

    #[test]
    fn fake_translator_is_deterministic() {
        let mut translator = FakeTranslator;
        let result = translator
            .translate(&TranslationRequest {
                text: "こんにちは".into(),
            })
            .unwrap();
        assert_eq!(result.text, "[FAKE] こんにちは");
    }

    struct ConstantSelection;

    impl SelectionProvider for ConstantSelection {
        fn capture(&mut self) -> Result<Selection, CaptureError> {
            Ok(Selection {
                text: "hello".into(),
                source: SelectionSource::UiAutomation,
                bounds: Some(ScreenRect {
                    x: 10.0,
                    y: 10.0,
                    width: 10.0,
                    height: 10.0,
                }),
                work_area: Some(ScreenRect {
                    x: 0.0,
                    y: 0.0,
                    width: 1920.0,
                    height: 1040.0,
                }),
            })
        }
    }

    #[derive(Default)]
    struct RecordingPopup {
        updates: AtomicUsize,
    }

    impl PopupPort for RecordingPopup {
        fn show(&self, _state: PopupViewModel) {
            self.updates.fetch_add(1, Ordering::Relaxed);
        }

        fn update(&self, _state: PopupViewModel) {
            self.updates.fetch_add(1, Ordering::Relaxed);
        }

        fn move_to(&self, _x: f64, _y: f64) {}
        fn hide(&self) {}
    }

    #[test]
    fn coordinator_reaches_ready_with_fakes() {
        let popup = Arc::new(RecordingPopup::default());
        let coordinator = InteractionCoordinator::start(
            Box::new(ConstantSelection),
            Box::new(FakeTranslator),
            popup,
        );
        coordinator.trigger();

        let deadline = Instant::now() + Duration::from_secs(1);
        while coordinator.snapshot().status != "ready" && Instant::now() < deadline {
            thread::yield_now();
        }

        assert_eq!(coordinator.snapshot().status, "ready");
        assert_eq!(
            coordinator.snapshot().translated_text.as_deref(),
            Some("[FAKE] hello")
        );
    }
}
