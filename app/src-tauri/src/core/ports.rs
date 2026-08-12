use super::{
    CaptureError, PopupViewModel, Selection, Translation, TranslationError, TranslationRequest,
};

pub trait SelectionProvider: Send {
    fn capture(&mut self) -> Result<Selection, CaptureError>;
}

pub trait Translator: Send {
    fn translate(&mut self, request: &TranslationRequest) -> Result<Translation, TranslationError>;
}

pub trait PopupPort: Send + Sync {
    fn show(&self, state: PopupViewModel);
    fn update(&self, state: PopupViewModel);
    fn move_to(&self, x: f64, y: f64);
    fn hide(&self);
}
