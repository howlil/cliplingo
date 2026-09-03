#[derive(Clone, Debug, PartialEq)]
pub struct ScreenRect {
    pub x: f64,
    pub y: f64,
    pub width: f64,
    pub height: f64,
}

impl ScreenRect {
    pub fn right(&self) -> f64 {
        self.x + self.width
    }

    pub fn bottom(&self) -> f64 {
        self.y + self.height
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenSize {
    pub width: f64,
    pub height: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct ScreenPoint {
    pub x: f64,
    pub y: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub enum SelectionSource {
    UiAutomation,
    Clipboard,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Selection {
    pub text: String,
    pub source: SelectionSource,
    pub bounds: Option<ScreenRect>,
    pub work_area: Option<ScreenRect>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct TranslationRequest {
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Translation {
    pub text: String,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CaptureError {
    NoSelection,
    Unsupported,
    ClipboardUnavailable,
    ClipboardPreservationUnsupported,
    Timeout,
    NativeFailure { operation: &'static str, code: i32 },
}

#[derive(Clone, Debug, PartialEq)]
pub enum TranslationError {
    ModelUnavailable,
    Failed,
}
