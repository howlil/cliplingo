use std::env;
use std::path::PathBuf;
use std::process::{Child, Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::core::{
    Translation, TranslationError, TranslationRequest, Translator, WorkerLifecycle, WorkerMessage,
    WorkerState,
};
use crate::platform::windows::WorkerPipeClient;

use super::model_pack;

const WORKER_EXE_NAME: &str = "cliplingo-worker.exe";
const MAX_RESTART_ATTEMPTS: u8 = 1;
const CONNECT_TIMEOUT: Duration = Duration::from_secs(2);
const CONNECT_RETRY_DELAY: Duration = Duration::from_millis(20);

pub struct WorkerTranslator {
    executable: PathBuf,
    model_pack: Option<PathBuf>,
    lifecycle: WorkerLifecycle,
    child: Option<Child>,
    client: Option<WorkerPipeClient>,
    next_request_id: u64,
}

impl WorkerTranslator {
    pub fn new_default(model_pack: PathBuf) -> Self {
        Self::with_configuration(default_worker_executable(), Some(model_pack))
    }

    pub fn with_executable(executable: impl Into<PathBuf>) -> Self {
        Self::with_configuration(executable.into(), None)
    }

    pub fn with_executable_and_model(
        executable: impl Into<PathBuf>,
        model_pack: impl Into<PathBuf>,
    ) -> Self {
        Self::with_configuration(executable.into(), Some(model_pack.into()))
    }

    fn with_configuration(executable: PathBuf, model_pack: Option<PathBuf>) -> Self {
        Self {
            executable,
            model_pack,
            lifecycle: WorkerLifecycle::new(MAX_RESTART_ATTEMPTS),
            child: None,
            client: None,
            next_request_id: 1,
        }
    }

    fn ensure_ready(&mut self) -> Result<(), TranslationError> {
        if self.lifecycle.state() == WorkerState::Ready {
            if self.client.is_some() && self.child_is_running() {
                return Ok(());
            }
            self.fail_runtime();
        }

        match self.lifecycle.state() {
            WorkerState::Stopped => self
                .lifecycle
                .begin_start()
                .map_err(|_| TranslationError::Failed)?,
            WorkerState::Failed => self
                .lifecycle
                .begin_restart()
                .map_err(|_| TranslationError::Failed)?,
            WorkerState::Starting => {}
            WorkerState::Ready => return Ok(()),
            WorkerState::Busy => return Err(TranslationError::Failed),
        }

        self.spawn_and_connect()
    }

    fn spawn_and_connect(&mut self) -> Result<(), TranslationError> {
        self.clear_runtime();
        let mut command = Command::new(&self.executable);
        command
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        if let Some(model_pack) = &self.model_pack {
            command.env("CLIPLINGO_MODEL_PACK", model_pack);
        }

        let child = command.spawn().map_err(|_| {
            self.lifecycle.fail();
            TranslationError::Failed
        })?;
        self.child = Some(child);

        let deadline = Instant::now() + CONNECT_TIMEOUT;
        loop {
            match WorkerPipeClient::connect_default() {
                Ok(client) => {
                    self.client = Some(client);
                    return self
                        .lifecycle
                        .mark_ready()
                        .map_err(|_| TranslationError::Failed);
                }
                Err(_) if Instant::now() < deadline && self.child_is_running() => {
                    thread::sleep(CONNECT_RETRY_DELAY);
                }
                Err(_) => {
                    self.fail_runtime();
                    return Err(TranslationError::Failed);
                }
            }
        }
    }

    fn child_is_running(&mut self) -> bool {
        match self.child.as_mut() {
            Some(child) => matches!(child.try_wait(), Ok(None)),
            None => false,
        }
    }

    fn fail_runtime(&mut self) {
        self.lifecycle.fail();
        self.clear_runtime();
    }

    fn clear_runtime(&mut self) {
        self.client.take();
        if let Some(mut child) = self.child.take() {
            if matches!(child.try_wait(), Ok(None)) {
                let _ = child.kill();
            }
            let _ = child.wait();
        }
    }

    fn next_request_id(&mut self) -> u64 {
        let request_id = self.next_request_id;
        self.next_request_id = self.next_request_id.wrapping_add(1);
        if self.next_request_id == 0 {
            self.next_request_id = 1;
        }
        request_id
    }

    fn translate_once(
        &mut self,
        request: &TranslationRequest,
    ) -> Result<Translation, TranslationError> {
        if self
            .model_pack
            .as_deref()
            .is_some_and(|directory| !model_pack::is_complete(directory))
        {
            return Err(TranslationError::ModelUnavailable);
        }

        self.ensure_ready()?;
        self.lifecycle
            .begin_request()
            .map_err(|_| TranslationError::Failed)?;

        let request_id = self.next_request_id();
        let response = match self.client.as_mut() {
            Some(client) => client.translate(request_id, &request.text),
            None => {
                self.fail_runtime();
                return Err(TranslationError::Failed);
            }
        };

        match response {
            Ok(WorkerMessage::TranslateResponse { text, .. }) => {
                self.lifecycle
                    .finish_request()
                    .map_err(|_| TranslationError::Failed)?;
                Ok(Translation { text })
            }
            Ok(WorkerMessage::ErrorResponse { .. }) => {
                self.lifecycle
                    .finish_request()
                    .map_err(|_| TranslationError::Failed)?;
                Err(TranslationError::Failed)
            }
            Ok(WorkerMessage::TranslateRequest { .. }) | Err(_) => {
                self.fail_runtime();
                Err(TranslationError::Failed)
            }
        }
    }
}

impl Translator for WorkerTranslator {
    fn translate(&mut self, request: &TranslationRequest) -> Result<Translation, TranslationError> {
        for _ in 0..=MAX_RESTART_ATTEMPTS {
            match self.translate_once(request) {
                Ok(translation) => return Ok(translation),
                Err(TranslationError::ModelUnavailable) => {
                    return Err(TranslationError::ModelUnavailable)
                }
                Err(error) if self.lifecycle.state() == WorkerState::Failed => {
                    if self.lifecycle.restart_attempts() >= self.lifecycle.max_restart_attempts() {
                        return Err(error);
                    }
                }
                Err(error) => return Err(error),
            }
        }
        Err(TranslationError::Failed)
    }
}

impl Drop for WorkerTranslator {
    fn drop(&mut self) {
        self.clear_runtime();
        self.lifecycle.stop();
    }
}

fn default_worker_executable() -> PathBuf {
    env::current_exe()
        .ok()
        .and_then(|path| path.parent().map(|parent| parent.join(WORKER_EXE_NAME)))
        .unwrap_or_else(|| PathBuf::from(WORKER_EXE_NAME))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn missing_worker_fails_without_panicking() {
        let mut translator = WorkerTranslator::with_executable(
            r"Z:\cliplingo-definitely-missing\cliplingo-worker.exe",
        );

        assert_eq!(
            translator.translate(&TranslationRequest {
                text: "private text".into(),
            }),
            Err(TranslationError::Failed)
        );
    }

    #[test]
    fn missing_model_is_reported_before_worker_start() {
        let model = temporary_directory("missing-model");
        let mut translator = WorkerTranslator::with_executable_and_model(
            r"Z:\cliplingo-definitely-missing\cliplingo-worker.exe",
            &model,
        );

        assert_eq!(
            translator.translate(&TranslationRequest {
                text: "private text".into(),
            }),
            Err(TranslationError::ModelUnavailable)
        );
        assert!(!model.exists());
    }

    #[test]
    fn default_worker_path_is_sibling_executable() {
        let path = default_worker_executable();
        assert_eq!(
            path.file_name().and_then(|value| value.to_str()),
            Some(WORKER_EXE_NAME)
        );
    }

    fn temporary_directory(label: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("cliplingo-{label}-{nonce}"));
        let _ = fs::remove_dir_all(&path);
        path
    }
}
