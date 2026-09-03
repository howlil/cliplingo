use std::sync::{Arc, Condvar, Mutex};
use std::thread;
use std::time::Instant;

use crate::core::{
    place_popup, ApplyResult, CaptureError, PopupErrorCode, PopupPort, PopupSession, PopupState,
    PopupViewModel, RequestId, ScreenRect, ScreenSize, SelectionProvider, SelectionSource,
    Translation, TranslationError, TranslationRequest, Translator,
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
        let mut slot = self
            .value
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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
        let mut slot = self
            .value
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner());
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

#[derive(Clone, Debug)]
struct PendingInteraction {
    request_id: RequestId,
    started_at: Instant,
    fallback_position: Option<(ScreenRect, ScreenRect)>,
}

pub struct FakeTranslator;

impl Translator for FakeTranslator {
    fn translate(&mut self, request: &TranslationRequest) -> Result<Translation, TranslationError> {
        Ok(Translation {
            text: format!("[FAKE] {}", request.text),
        })
    }
}

pub struct InteractionCoordinator {
    session: Arc<Mutex<PopupSession>>,
    pending: Arc<PendingSlot<PendingInteraction>>,
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
                let interaction = pending.wait_take();
                Self::process_request(
                    interaction,
                    provider.as_mut(),
                    translator.as_mut(),
                    session.as_ref(),
                    popup.as_ref(),
                );
            })
            .expect("failed to start interaction worker");

        coordinator
    }

    pub fn trigger_at(
        &self,
        anchor: &ScreenRect,
        work_area: &ScreenRect,
        started_at: Instant,
    ) -> RequestId {
        let request_id = self.begin_request();
        self.pending.submit(PendingInteraction {
            request_id,
            started_at,
            fallback_position: Some((anchor.clone(), work_area.clone())),
        });
        request_id
    }

    pub fn trigger(&self, started_at: Instant) -> RequestId {
        let request_id = self.begin_request();
        self.pending.submit(PendingInteraction {
            request_id,
            started_at,
            fallback_position: None,
        });
        request_id
    }

    fn begin_request(&self) -> RequestId {
        self.session
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
            .begin_request()
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
        matches!(
            self.session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner())
                .snapshot(),
            PopupState::Translating { .. } | PopupState::Ready { .. } | PopupState::Error { .. }
        )
    }

    fn process_request(
        interaction: PendingInteraction,
        provider: &mut dyn SelectionProvider,
        translator: &mut dyn Translator,
        session: &Mutex<PopupSession>,
        popup: &dyn PopupPort,
    ) {
        let request_id = interaction.request_id;
        let capture_started = Instant::now();
        let selection = match provider.capture() {
            Ok(selection) => {
                eprintln!(
                    "event=interaction_timing request_id={request_id} metric=capture duration_us={} capture_source={}",
                    capture_started.elapsed().as_micros(),
                    selection_source_label(&selection.source)
                );
                selection
            }
            Err(error) => {
                eprintln!(
                    "event=interaction_timing request_id={request_id} metric=capture duration_us={} status=ignored error_code={}",
                    capture_started.elapsed().as_micros(),
                    capture_error_label(&error)
                );
                let _ = session
                    .lock()
                    .unwrap_or_else(|poisoned| poisoned.into_inner())
                    .cancel(request_id);
                return;
            }
        };

        let placement = selection
            .bounds
            .as_ref()
            .zip(selection.work_area.as_ref())
            .or_else(|| {
                interaction
                    .fallback_position
                    .as_ref()
                    .map(|(anchor, work_area)| (anchor, work_area))
            });
        if let Some((anchor, work_area)) = placement {
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
        popup.show(state.view_model());
        eprintln!(
            "event=interaction_timing request_id={request_id} metric=hotkey_to_popup_show_request duration_us={}",
            interaction.started_at.elapsed().as_micros()
        );

        let translation_started = Instant::now();
        let translation = translator.translate(&TranslationRequest {
            text: selection.text,
        });
        eprintln!(
            "event=interaction_timing request_id={request_id} metric=translation duration_us={} status={}",
            translation_started.elapsed().as_micros(),
            if translation.is_ok() { "ok" } else { "error" }
        );

        let state = {
            let mut session = session
                .lock()
                .unwrap_or_else(|poisoned| poisoned.into_inner());
            match translation {
                Ok(value) => session.complete(request_id, value),
                Err(TranslationError::ModelUnavailable) => {
                    session.fail(request_id, PopupErrorCode::ModelUnavailable)
                }
                Err(TranslationError::Failed) => {
                    session.fail(request_id, PopupErrorCode::TranslationFailed)
                }
            }
        };
        if let ApplyResult::Applied(state) = state {
            popup.update(state.view_model());
            eprintln!(
                "event=interaction_timing request_id={request_id} metric=hotkey_to_ready_request duration_us={}",
                interaction.started_at.elapsed().as_micros()
            );
        }
    }
}

fn selection_source_label(source: &SelectionSource) -> &'static str {
    match source {
        SelectionSource::UiAutomation => "uia",
        SelectionSource::Clipboard => "clipboard",
    }
}

fn capture_error_label(error: &CaptureError) -> &'static str {
    match error {
        CaptureError::NoSelection => "no_selection",
        CaptureError::Unsupported => "unsupported",
        CaptureError::ClipboardUnavailable => "clipboard_unavailable",
        CaptureError::ClipboardPreservationUnsupported => "clipboard_preservation_unsupported",
        CaptureError::Timeout => "timeout",
        CaptureError::NativeFailure { .. } => "native_failure",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ScreenRect, Selection, SelectionSource};
    use std::sync::atomic::{AtomicUsize, Ordering};
    use std::time::Duration;

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
                text: "hello".into(),
            })
            .unwrap();
        assert_eq!(result.text, "[FAKE] hello");
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

    struct NoSelection;

    impl SelectionProvider for NoSelection {
        fn capture(&mut self) -> Result<Selection, CaptureError> {
            Err(CaptureError::NoSelection)
        }
    }

    struct CountingTranslator {
        calls: Arc<AtomicUsize>,
    }

    impl Translator for CountingTranslator {
        fn translate(
            &mut self,
            request: &TranslationRequest,
        ) -> Result<Translation, TranslationError> {
            self.calls.fetch_add(1, Ordering::Relaxed);
            Ok(Translation {
                text: format!("translated: {}", request.text),
            })
        }
    }

    #[derive(Default)]
    struct RecordingPopup {
        shows: AtomicUsize,
        updates: AtomicUsize,
    }

    impl PopupPort for RecordingPopup {
        fn show(&self, _state: PopupViewModel) {
            self.shows.fetch_add(1, Ordering::Relaxed);
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
            popup.clone(),
        );
        coordinator.trigger(Instant::now());

        let deadline = Instant::now() + Duration::from_secs(1);
        while coordinator.snapshot().status != "ready" && Instant::now() < deadline {
            thread::yield_now();
        }

        assert_eq!(coordinator.snapshot().status, "ready");
        assert_eq!(
            coordinator.snapshot().translated_text.as_deref(),
            Some("[FAKE] hello")
        );
        assert_eq!(popup.shows.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn no_selection_is_silent_and_never_calls_translator() {
        let popup = Arc::new(RecordingPopup::default());
        let translation_calls = Arc::new(AtomicUsize::new(0));
        let coordinator = InteractionCoordinator::start(
            Box::new(NoSelection),
            Box::new(CountingTranslator {
                calls: Arc::clone(&translation_calls),
            }),
            popup.clone(),
        );
        coordinator.trigger(Instant::now());

        let deadline = Instant::now() + Duration::from_secs(1);
        while coordinator.snapshot().status != "hidden" && Instant::now() < deadline {
            thread::yield_now();
        }

        assert_eq!(coordinator.snapshot().status, "hidden");
        assert_eq!(popup.shows.load(Ordering::Relaxed), 0);
        assert_eq!(popup.updates.load(Ordering::Relaxed), 0);
        assert_eq!(translation_calls.load(Ordering::Relaxed), 0);
    }
}
