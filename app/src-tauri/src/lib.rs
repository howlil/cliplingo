pub mod application;
pub mod core;

use std::sync::Arc;

use application::{FakeTranslator, InteractionCoordinator};
use core::{CaptureError, PopupPort, PopupViewModel, Selection, SelectionProvider};

struct NoSelectionProvider;

impl SelectionProvider for NoSelectionProvider {
    fn capture(&mut self) -> Result<Selection, CaptureError> {
        Err(CaptureError::Unsupported)
    }
}

#[derive(Default)]
struct NullPopup;

impl PopupPort for NullPopup {
    fn show(&self, _state: PopupViewModel) {}
    fn update(&self, _state: PopupViewModel) {}
    fn move_to(&self, _x: f64, _y: f64) {}
    fn hide(&self) {}
}

pub fn run() {
    tauri::Builder::default()
        .setup(|_app| {
            let _coordinator = InteractionCoordinator::start(
                Box::new(NoSelectionProvider),
                Box::new(FakeTranslator),
                Arc::new(NullPopup),
            );
            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running ClipLingo");
}
