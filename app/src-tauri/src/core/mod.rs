pub mod popup;
pub mod ports;
pub mod positioning;
pub mod types;
pub mod worker_protocol;

pub use popup::{ApplyResult, PopupErrorCode, PopupSession, PopupState, PopupViewModel, RequestId};
pub use ports::{PopupPort, SelectionProvider, Translator};
pub use positioning::place_popup;
pub use types::{
    CaptureError, ScreenPoint, ScreenRect, ScreenSize, Selection, SelectionSource, Translation,
    TranslationError, TranslationRequest,
};
pub use worker_protocol::{
    decode_message as decode_worker_message, encode_message as encode_worker_message,
    ProtocolError, WorkerErrorCode, WorkerMessage, HEADER_LEN as WORKER_HEADER_LEN,
    MAX_PAYLOAD_LEN as WORKER_MAX_PAYLOAD_LEN, PROTOCOL_VERSION as WORKER_PROTOCOL_VERSION,
};
