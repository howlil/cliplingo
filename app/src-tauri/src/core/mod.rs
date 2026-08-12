pub mod popup;
pub mod ports;
pub mod positioning;
pub mod types;

pub use popup::{ApplyResult, PopupErrorCode, PopupSession, PopupState, PopupViewModel, RequestId};
pub use ports::{PopupPort, SelectionProvider, Translator};
pub use positioning::place_popup;
pub use types::{
    CaptureError, ScreenPoint, ScreenRect, ScreenSize, Selection, SelectionSource, Translation,
    TranslationError, TranslationRequest,
};
