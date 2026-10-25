//! One bounded translation worker; model is never constructed on the hotkey or
//! Tauri event loop. Only verified translated *target* text reaches PopupOwner.
use crate::model_pack::{default_pack_dir, verify_pack, ModelError};
use std::{
    path::Path,
    sync::{Arc, Mutex, mpsc::{self, SyncSender, TrySendError}},
    thread,
    time::{Duration, Instant},
};

const MODEL_IDLE_EVICTION: Duration = Duration::from_secs(90);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TranslationState {
    Idle,
    MissingModel,
    InvalidModel,
    Loading,
    Translating,
    Ready,
    Failed,
    BackendUnavailable,
    QueueFull,
}
impl TranslationState {
    pub fn label(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::MissingModel => "model_missing",
            Self::InvalidModel => "model_invalid",
            Self::Loading => "model_loading",
            Self::Translating => "translating",
            Self::Ready => "ready",
            Self::Failed => "translation_failed",
            Self::BackendUnavailable => "backend_not_compiled",
            Self::QueueFull => "translation_busy",
        }
    }
}
#[derive(Debug, Clone, Copy)]
pub struct TranslationStatus {
    pub state: TranslationState,
    pub successful: u64,
}

struct Telemetry {
    state: TranslationState,
    successful: u64,
}
impl Default for Telemetry {
    fn default() -> Self { Self { state: TranslationState::Idle, successful: 0 } }
}

pub struct TranslationRequest {
    pub generation: u64,
    pub anchor: (i32, i32),
    pub source_hwnd: isize,
    pub text: String, // sensitive; owned and dropped only by the worker
}
pub struct TranslationResult {
    pub generation: u64,
    pub anchor: (i32, i32),
    pub source_hwnd: isize,
    pub target: String, // translated output only
}

#[cfg(feature = "offline-ct2")]
struct Loaded {
    translator: ct2rs::Translator<ct2rs::tokenizers::sentencepiece::Tokenizer>,
}
#[cfg(feature = "offline-ct2")]
impl Loaded {
    fn load(model_dir: &Path) -> Result<Self, TranslationState> {
        verify_pack(model_dir).map_err(classify_pack)?;
        let config = ct2rs::Config {
            device: ct2rs::Device::CPU,
            num_threads_per_replica: 2,
            ..Default::default()
        };
        let tokenizer = ct2rs::tokenizers::sentencepiece::Tokenizer::from_file(
            model_dir.join("source.spm"), model_dir.join("target.spm")
        ).map_err(|_| TranslationState::InvalidModel)?;
        let translator = ct2rs::Translator::with_tokenizer(model_dir, tokenizer, &config)
            .map_err(|_| TranslationState::InvalidModel)?;
        Ok(Self { translator })
    }
    fn translate(&self, text: &str) -> Result<String, TranslationState> {
        let options = ct2rs::TranslationOptions::<&str, &str> {
            beam_size: 2,
            max_input_length: 1024,
            max_decoding_length: 256,
            ..Default::default()
        };
        let results = self.translator.translate_batch(&[text], &options, None)
            .map_err(|_| TranslationState::Failed)?;
        let output = results.into_iter().next().ok_or(TranslationState::Failed)?.0;
        if output.trim().is_empty() { return Err(TranslationState::Failed); }
        Ok(output)
    }
}

fn classify_pack(error: ModelError) -> TranslationState {
    match error {
        ModelError::Missing => TranslationState::MissingModel,
        _ => TranslationState::InvalidModel,
    }
}

pub struct TranslationEngine {
    sender: SyncSender<TranslationRequest>,
    telemetry: Arc<Mutex<Telemetry>>,
}
impl TranslationEngine {
    pub fn start(on_result: Arc<dyn Fn(TranslationResult) + Send + Sync>) -> Result<Self, String> {
        let (sender, receiver) = mpsc::sync_channel::<TranslationRequest>(1);
        let telemetry = Arc::new(Mutex::new(Telemetry::default()));
        let state = telemetry.clone();
        thread::Builder::new().name("cliplingo-local-model".into()).spawn(move || {
            #[cfg(feature="offline-ct2")]
            let mut loaded: Option<Loaded> = None;
            #[cfg(feature="offline-ct2")]
            let mut last_used = Instant::now();
            loop {
                let next = receiver.recv_timeout(Duration::from_secs(10));
                let request = match next {
                    Ok(value) => value,
                    Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    Err(mpsc::RecvTimeoutError::Timeout) => {
                        #[cfg(feature="offline-ct2")]
                        if loaded.is_some() && last_used.elapsed() >= MODEL_IDLE_EVICTION {
                            loaded = None;
                            state.lock().unwrap().state = TranslationState::Idle;
                        }
                        continue;
                    }
                };
                #[cfg(not(feature="offline-ct2"))]
                {
                    // Feature-disabled build must NEVER claim it translated input.
                    drop(request);
                    state.lock().unwrap().state = TranslationState::BackendUnavailable;
                    continue;
                }
                #[cfg(feature="offline-ct2")]
                {
                    let model_path = match default_pack_dir() {
                        Ok(path) => path,
                        Err(error) => {
                            state.lock().unwrap().state = classify_pack(error);
                            continue;
                        }
                    };
                    if loaded.is_none() {
                        state.lock().unwrap().state = TranslationState::Loading;
                        match Loaded::load(&model_path) {
                            Ok(value) => loaded = Some(value),
                            Err(reason) => {
                                state.lock().unwrap().state = reason;
                                continue;
                            }
                        }
                    }
                    state.lock().unwrap().state = TranslationState::Translating;
                    let result = loaded.as_ref().expect("loaded above").translate(&request.text);
                    // Translation input is dropped here. No source text in events.
                    last_used = Instant::now();
                    match result {
                        Ok(target) => {
                            state.lock().unwrap().successful += 1;
                            state.lock().unwrap().state = TranslationState::Ready;
                            on_result(TranslationResult {
                                generation: request.generation,
                                anchor: request.anchor,
                                source_hwnd: request.source_hwnd,
                                target,
                            });
                        }
                        Err(reason) => state.lock().unwrap().state = reason,
                    }
                }
            }
        }).map_err(|e| format!("Could not start local model worker: {e}"))?;
        Ok(Self { sender, telemetry })
    }
    pub fn submit(&self, request: TranslationRequest) {
        // Caller must enforce newest-generation presentation after result.
        match self.sender.try_send(request) {
            Ok(()) => self.telemetry.lock().unwrap().state = TranslationState::Translating,
            Err(TrySendError::Full(_)) => self.telemetry.lock().unwrap().state = TranslationState::QueueFull,
            Err(TrySendError::Disconnected(_)) => self.telemetry.lock().unwrap().state = TranslationState::Failed,
        }
    }
    pub fn status(&self) -> TranslationStatus {
        let state = self.telemetry.lock().unwrap();
        TranslationStatus { state: state.state, successful: state.successful }
    }
    pub fn pack_dir(&self) -> Result<String, ModelError> {
        default_pack_dir().map(|p| p.display().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn model_errors_never_create_fake_success() {
        assert_eq!(classify_pack(ModelError::Missing), TranslationState::MissingModel);
        assert_eq!(classify_pack(ModelError::CorruptFile), TranslationState::InvalidModel);
    }
}
