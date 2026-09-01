const MAGIC: [u8; 4] = *b"CLNG";
pub const PROTOCOL_VERSION: u8 = 1;
pub const HEADER_LEN: usize = 18;
pub const MAX_PAYLOAD_LEN: usize = 1024 * 1024;

const TYPE_TRANSLATE_REQUEST: u8 = 0x01;
const TYPE_TRANSLATE_RESPONSE: u8 = 0x02;
const TYPE_ERROR_RESPONSE: u8 = 0x03;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WorkerErrorCode {
    MalformedRequest = 0x01,
    UnsupportedRequest = 0x02,
    TranslationFailed = 0x03,
    WorkerUnavailable = 0x04,
}

impl TryFrom<u8> for WorkerErrorCode {
    type Error = ProtocolError;

    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0x01 => Ok(Self::MalformedRequest),
            0x02 => Ok(Self::UnsupportedRequest),
            0x03 => Ok(Self::TranslationFailed),
            0x04 => Ok(Self::WorkerUnavailable),
            other => Err(ProtocolError::UnknownErrorCode(other)),
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum WorkerMessage {
    TranslateRequest {
        request_id: u64,
        text: String,
    },
    TranslateResponse {
        request_id: u64,
        text: String,
    },
    ErrorResponse {
        request_id: u64,
        code: WorkerErrorCode,
    },
}

impl WorkerMessage {
    pub fn request_id(&self) -> u64 {
        match self {
            Self::TranslateRequest { request_id, .. }
            | Self::TranslateResponse { request_id, .. }
            | Self::ErrorResponse { request_id, .. } => *request_id,
        }
    }
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum ProtocolError {
    FrameTooShort { actual: usize },
    InvalidMagic,
    UnsupportedVersion(u8),
    UnknownMessageType(u8),
    PayloadTooLarge { len: usize },
    LengthMismatch { expected: usize, actual: usize },
    InvalidUtf8,
    InvalidErrorPayloadLength(usize),
    UnknownErrorCode(u8),
}

pub fn encode_message(message: &WorkerMessage) -> Result<Vec<u8>, ProtocolError> {
    let request_id = message.request_id();
    let (message_type, payload): (u8, Vec<u8>) = match message {
        WorkerMessage::TranslateRequest { text, .. } => {
            (TYPE_TRANSLATE_REQUEST, text.as_bytes().to_vec())
        }
        WorkerMessage::TranslateResponse { text, .. } => {
            (TYPE_TRANSLATE_RESPONSE, text.as_bytes().to_vec())
        }
        WorkerMessage::ErrorResponse { code, .. } => (TYPE_ERROR_RESPONSE, vec![*code as u8]),
    };

    if payload.len() > MAX_PAYLOAD_LEN {
        return Err(ProtocolError::PayloadTooLarge { len: payload.len() });
    }

    let mut frame = Vec::with_capacity(HEADER_LEN + payload.len());
    frame.extend_from_slice(&MAGIC);
    frame.push(PROTOCOL_VERSION);
    frame.push(message_type);
    frame.extend_from_slice(&request_id.to_le_bytes());
    frame.extend_from_slice(&(payload.len() as u32).to_le_bytes());
    frame.extend_from_slice(&payload);
    Ok(frame)
}

pub fn decode_message(frame: &[u8]) -> Result<WorkerMessage, ProtocolError> {
    if frame.len() < HEADER_LEN {
        return Err(ProtocolError::FrameTooShort {
            actual: frame.len(),
        });
    }

    if frame[..4] != MAGIC {
        return Err(ProtocolError::InvalidMagic);
    }

    let version = frame[4];
    if version != PROTOCOL_VERSION {
        return Err(ProtocolError::UnsupportedVersion(version));
    }

    let message_type = frame[5];
    if !matches!(
        message_type,
        TYPE_TRANSLATE_REQUEST | TYPE_TRANSLATE_RESPONSE | TYPE_ERROR_RESPONSE
    ) {
        return Err(ProtocolError::UnknownMessageType(message_type));
    }

    let request_id = u64::from_le_bytes(
        frame[6..14]
            .try_into()
            .expect("worker protocol request id header has fixed width"),
    );
    let payload_len = u32::from_le_bytes(
        frame[14..18]
            .try_into()
            .expect("worker protocol payload length header has fixed width"),
    ) as usize;

    if payload_len > MAX_PAYLOAD_LEN {
        return Err(ProtocolError::PayloadTooLarge { len: payload_len });
    }

    let expected_len = HEADER_LEN + payload_len;
    if frame.len() != expected_len {
        return Err(ProtocolError::LengthMismatch {
            expected: expected_len,
            actual: frame.len(),
        });
    }

    let payload = &frame[HEADER_LEN..];
    match message_type {
        TYPE_TRANSLATE_REQUEST => Ok(WorkerMessage::TranslateRequest {
            request_id,
            text: decode_text(payload)?,
        }),
        TYPE_TRANSLATE_RESPONSE => Ok(WorkerMessage::TranslateResponse {
            request_id,
            text: decode_text(payload)?,
        }),
        TYPE_ERROR_RESPONSE => {
            if payload.len() != 1 {
                return Err(ProtocolError::InvalidErrorPayloadLength(payload.len()));
            }
            Ok(WorkerMessage::ErrorResponse {
                request_id,
                code: WorkerErrorCode::try_from(payload[0])?,
            })
        }
        _ => unreachable!("worker protocol message type validated above"),
    }
}

fn decode_text(payload: &[u8]) -> Result<String, ProtocolError> {
    std::str::from_utf8(payload)
        .map(str::to_owned)
        .map_err(|_| ProtocolError::InvalidUtf8)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translate_request_round_trips_unicode() {
        let message = WorkerMessage::TranslateRequest {
            request_id: 42,
            text: "こんにちは dunia 🌏".into(),
        };

        let encoded = encode_message(&message).unwrap();
        assert_eq!(decode_message(&encoded).unwrap(), message);
    }

    #[test]
    fn translate_response_round_trips() {
        let message = WorkerMessage::TranslateResponse {
            request_id: u64::MAX - 1,
            text: "[FAKE] halo".into(),
        };

        let encoded = encode_message(&message).unwrap();
        assert_eq!(decode_message(&encoded).unwrap(), message);
    }

    #[test]
    fn error_response_round_trips() {
        let message = WorkerMessage::ErrorResponse {
            request_id: 7,
            code: WorkerErrorCode::TranslationFailed,
        };

        let encoded = encode_message(&message).unwrap();
        assert_eq!(decode_message(&encoded).unwrap(), message);
    }

    #[test]
    fn rejects_invalid_magic() {
        let mut encoded = encode_message(&WorkerMessage::TranslateRequest {
            request_id: 1,
            text: "hello".into(),
        })
        .unwrap();
        encoded[0] = b'X';

        assert_eq!(decode_message(&encoded), Err(ProtocolError::InvalidMagic));
    }

    #[test]
    fn rejects_unsupported_version() {
        let mut encoded = encode_message(&WorkerMessage::TranslateRequest {
            request_id: 1,
            text: "hello".into(),
        })
        .unwrap();
        encoded[4] = PROTOCOL_VERSION + 1;

        assert_eq!(
            decode_message(&encoded),
            Err(ProtocolError::UnsupportedVersion(PROTOCOL_VERSION + 1))
        );
    }

    #[test]
    fn rejects_unknown_message_type() {
        let mut encoded = encode_message(&WorkerMessage::TranslateRequest {
            request_id: 1,
            text: "hello".into(),
        })
        .unwrap();
        encoded[5] = 0xff;

        assert_eq!(
            decode_message(&encoded),
            Err(ProtocolError::UnknownMessageType(0xff))
        );
    }

    #[test]
    fn rejects_truncated_or_trailing_frame() {
        let encoded = encode_message(&WorkerMessage::TranslateRequest {
            request_id: 1,
            text: "hello".into(),
        })
        .unwrap();

        let truncated = &encoded[..encoded.len() - 1];
        assert_eq!(
            decode_message(truncated),
            Err(ProtocolError::LengthMismatch {
                expected: encoded.len(),
                actual: encoded.len() - 1,
            })
        );

        let mut trailing = encoded.clone();
        trailing.push(0);
        assert_eq!(
            decode_message(&trailing),
            Err(ProtocolError::LengthMismatch {
                expected: encoded.len(),
                actual: encoded.len() + 1,
            })
        );
    }

    #[test]
    fn rejects_oversized_payload_before_encoding() {
        let message = WorkerMessage::TranslateRequest {
            request_id: 1,
            text: "x".repeat(MAX_PAYLOAD_LEN + 1),
        };

        assert_eq!(
            encode_message(&message),
            Err(ProtocolError::PayloadTooLarge {
                len: MAX_PAYLOAD_LEN + 1,
            })
        );
    }

    #[test]
    fn rejects_oversized_declared_payload_before_body_validation() {
        let mut frame = Vec::with_capacity(HEADER_LEN);
        frame.extend_from_slice(&MAGIC);
        frame.push(PROTOCOL_VERSION);
        frame.push(TYPE_TRANSLATE_REQUEST);
        frame.extend_from_slice(&1_u64.to_le_bytes());
        frame.extend_from_slice(&((MAX_PAYLOAD_LEN + 1) as u32).to_le_bytes());

        assert_eq!(
            decode_message(&frame),
            Err(ProtocolError::PayloadTooLarge {
                len: MAX_PAYLOAD_LEN + 1,
            })
        );
    }

    #[test]
    fn rejects_invalid_utf8_payload() {
        let mut frame = Vec::with_capacity(HEADER_LEN + 1);
        frame.extend_from_slice(&MAGIC);
        frame.push(PROTOCOL_VERSION);
        frame.push(TYPE_TRANSLATE_REQUEST);
        frame.extend_from_slice(&1_u64.to_le_bytes());
        frame.extend_from_slice(&1_u32.to_le_bytes());
        frame.push(0xff);

        assert_eq!(decode_message(&frame), Err(ProtocolError::InvalidUtf8));
    }

    #[test]
    fn rejects_invalid_error_payload() {
        let mut frame = Vec::with_capacity(HEADER_LEN + 2);
        frame.extend_from_slice(&MAGIC);
        frame.push(PROTOCOL_VERSION);
        frame.push(TYPE_ERROR_RESPONSE);
        frame.extend_from_slice(&1_u64.to_le_bytes());
        frame.extend_from_slice(&2_u32.to_le_bytes());
        frame.extend_from_slice(&[WorkerErrorCode::TranslationFailed as u8, 0]);

        assert_eq!(
            decode_message(&frame),
            Err(ProtocolError::InvalidErrorPayloadLength(2))
        );
    }
}
