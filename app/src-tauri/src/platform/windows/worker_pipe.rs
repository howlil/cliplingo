use std::fs::{File, OpenOptions};
use std::io::{Read, Write};

use crate::core::{
    read_worker_message, write_worker_message, WorkerIoError, WorkerMessage,
};

pub const WORKER_PIPE_PATH: &str = r"\\.\pipe\cliplingo-worker-v1";

#[derive(Debug)]
pub enum WorkerPipeError {
    Connect(std::io::Error),
    Transport(WorkerIoError),
    UnexpectedResponse,
    RequestIdMismatch { expected: u64, actual: u64 },
}

pub struct WorkerPipeClient {
    stream: File,
}

impl WorkerPipeClient {
    pub fn connect_default() -> Result<Self, WorkerPipeError> {
        let stream = OpenOptions::new()
            .read(true)
            .write(true)
            .open(WORKER_PIPE_PATH)
            .map_err(WorkerPipeError::Connect)?;
        Ok(Self { stream })
    }

    pub fn translate(
        &mut self,
        request_id: u64,
        text: &str,
    ) -> Result<WorkerMessage, WorkerPipeError> {
        translate_on_stream(&mut self.stream, request_id, text)
    }
}

fn translate_on_stream<S: Read + Write>(
    stream: &mut S,
    request_id: u64,
    text: &str,
) -> Result<WorkerMessage, WorkerPipeError> {
    write_worker_message(
        stream,
        &WorkerMessage::TranslateRequest {
            request_id,
            text: text.to_owned(),
        },
    )
    .map_err(WorkerPipeError::Transport)?;

    let response = read_worker_message(stream).map_err(WorkerPipeError::Transport)?;
    validate_response(request_id, response)
}

fn validate_response(
    expected_request_id: u64,
    response: WorkerMessage,
) -> Result<WorkerMessage, WorkerPipeError> {
    let actual_request_id = response.request_id();
    if actual_request_id != expected_request_id {
        return Err(WorkerPipeError::RequestIdMismatch {
            expected: expected_request_id,
            actual: actual_request_id,
        });
    }

    match response {
        WorkerMessage::TranslateResponse { .. } | WorkerMessage::ErrorResponse { .. } => {
            Ok(response)
        }
        WorkerMessage::TranslateRequest { .. } => Err(WorkerPipeError::UnexpectedResponse),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{decode_worker_message, encode_worker_message, WorkerErrorCode};
    use std::io::{Cursor, Result as IoResult};

    struct ScriptedStream {
        read: Cursor<Vec<u8>>,
        written: Vec<u8>,
    }

    impl ScriptedStream {
        fn with_response(response: &WorkerMessage) -> Self {
            Self {
                read: Cursor::new(encode_worker_message(response).unwrap()),
                written: Vec::new(),
            }
        }
    }

    impl Read for ScriptedStream {
        fn read(&mut self, buf: &mut [u8]) -> IoResult<usize> {
            self.read.read(buf)
        }
    }

    impl Write for ScriptedStream {
        fn write(&mut self, buf: &[u8]) -> IoResult<usize> {
            self.written.extend_from_slice(buf);
            Ok(buf.len())
        }

        fn flush(&mut self) -> IoResult<()> {
            Ok(())
        }
    }

    #[test]
    fn translate_writes_request_and_accepts_correlated_response() {
        let response = WorkerMessage::TranslateResponse {
            request_id: 19,
            text: "halo".into(),
        };
        let mut stream = ScriptedStream::with_response(&response);

        let actual = translate_on_stream(&mut stream, 19, "こんにちは").unwrap();

        assert_eq!(actual, response);
        assert_eq!(
            decode_worker_message(&stream.written).unwrap(),
            WorkerMessage::TranslateRequest {
                request_id: 19,
                text: "こんにちは".into(),
            }
        );
    }

    #[test]
    fn translate_accepts_correlated_worker_error() {
        let response = WorkerMessage::ErrorResponse {
            request_id: 7,
            code: WorkerErrorCode::TranslationFailed,
        };
        let mut stream = ScriptedStream::with_response(&response);

        assert_eq!(translate_on_stream(&mut stream, 7, "hello").unwrap(), response);
    }

    #[test]
    fn translate_rejects_mismatched_request_id() {
        let response = WorkerMessage::TranslateResponse {
            request_id: 8,
            text: "ignored".into(),
        };
        let mut stream = ScriptedStream::with_response(&response);

        match translate_on_stream(&mut stream, 7, "secret") {
            Err(WorkerPipeError::RequestIdMismatch { expected, actual }) => {
                assert_eq!(expected, 7);
                assert_eq!(actual, 8);
            }
            other => panic!("unexpected result: {other:?}"),
        }
    }

    #[test]
    fn translate_rejects_request_message_as_response() {
        let response = WorkerMessage::TranslateRequest {
            request_id: 5,
            text: "unexpected".into(),
        };
        let mut stream = ScriptedStream::with_response(&response);

        assert!(matches!(
            translate_on_stream(&mut stream, 5, "hello"),
            Err(WorkerPipeError::UnexpectedResponse)
        ));
    }
}
