//! One bounded translation worker; model is never constructed on the hotkey or
//! Tauri event loop. Only verified translated *target* text reaches PopupOwner.
use crate::model_pack::{default_pack_dir, ModelError};
use crate::latest_slot::{LatestSlot, Next};
#[cfg(feature = "offline-ct2")]
use crate::sentence_pipeline::{self, PipelineFailure};
#[cfg(feature = "offline-ct2")]
use crate::model_pack::verify_pack;
#[cfg(feature = "offline-ct2")]
use ct2rs::Tokenizer as _;
use std::{
    sync::{Arc, Mutex},
    thread,
    time::Duration,
};

#[cfg(feature = "offline-ct2")]
use std::{path::Path, time::Instant};

#[cfg(feature = "offline-ct2")]
const MODEL_IDLE_EVICTION: Duration = Duration::from_secs(90);
// Marian model advertises 512 positions; reserve headroom for special tokens.
#[cfg(feature = "offline-ct2")]
const MAX_SOURCE_TOKENS: usize = 480;

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
    InputTooLong,
    Superseded,
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
            Self::InputTooLong => "input_too_long",
            Self::Superseded => "superseded_by_new_request",
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

/// A restricted CPU inference profile. The default matches F2.5 production
/// behavior until verified benchmark+translation quality evidence supports
/// selecting another profile. Benchmark profiles are not user Settings.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct OfflineProfile {
    pub beam_size: usize,
    pub cpu_threads: usize,
}
impl Default for OfflineProfile {
    fn default() -> Self {
        Self { beam_size: 2, cpu_threads: 2 }
    }
}
impl OfflineProfile {
    pub fn validate(self) -> bool {
        matches!(self.beam_size, 1 | 2) && matches!(self.cpu_threads, 1 | 2 | 4)
    }
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
    source_tokenizer: ct2rs::tokenizers::sentencepiece::Tokenizer,
}
#[cfg(feature = "offline-ct2")]
impl Loaded {
    fn load(model_dir: &Path, profile: OfflineProfile) -> Result<Self, TranslationState> {
        verify_pack(model_dir).map_err(classify_pack)?;
        let config = ct2rs::Config {
            device: ct2rs::Device::CPU,
            num_threads_per_replica: profile.cpu_threads,
            ..Default::default()
        };
        let tokenizer = ct2rs::tokenizers::sentencepiece::Tokenizer::from_file(
            model_dir.join("source.spm"), model_dir.join("target.spm")
        ).map_err(|_| TranslationState::InvalidModel)?;
        let source_tokenizer = ct2rs::tokenizers::sentencepiece::Tokenizer::from_file(
            model_dir.join("source.spm"), model_dir.join("target.spm")
        ).map_err(|_| TranslationState::InvalidModel)?;
        let translator = ct2rs::Translator::with_tokenizer(model_dir, tokenizer, &config)
            .map_err(|_| TranslationState::InvalidModel)?;
        Ok(Self { translator, source_tokenizer })
    }
    fn translate(&self, text: &str, profile: OfflineProfile, is_superseded: impl Fn() -> bool) -> Result<String, TranslationState> {
        sentence_pipeline::translate_complete(
            text,
            |segment| self.translate_one(segment, profile),
            is_superseded,
        ).map_err(|error| match error {
            PipelineFailure::Empty | PipelineFailure::EmptyTarget | PipelineFailure::Model(TranslationState::Failed) => TranslationState::Failed,
            PipelineFailure::TooManySentences | PipelineFailure::OutputTooLong => TranslationState::InputTooLong,
            PipelineFailure::Model(reason) => reason,
            PipelineFailure::Superseded => TranslationState::Superseded,
        })
    }

    fn translate_one(&self, text: &str, profile: OfflineProfile) -> Result<String, TranslationState> {
        // Enforce model token budget for EVERY sentence; no silent truncation.
        let source_tokens = self.source_tokenizer.encode(text)
            .map_err(|_| TranslationState::Failed)?;
        if source_tokens.len() > MAX_SOURCE_TOKENS {
            return Err(TranslationState::InputTooLong);
        }
        let options = ct2rs::TranslationOptions::<String, String> {
            beam_size: profile.beam_size,
            max_input_length: 0, // disable silent input truncation
            max_decoding_length: 512,
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
    pending: Arc<LatestSlot<TranslationRequest>>,
    telemetry: Arc<Mutex<Telemetry>>,
}
impl TranslationEngine {
    pub fn start(on_result: Arc<dyn Fn(TranslationResult) + Send + Sync>) -> Result<Self, String> {
        Self::start_with_profile(on_result, OfflineProfile::default())
    }

    /// Separately exercised by the headless native benchmark harness.
    /// Never accept arbitrary unbounded thread counts or decoding modes.
    pub fn start_with_profile(
        on_result: Arc<dyn Fn(TranslationResult) + Send + Sync>,
        profile: OfflineProfile,
    ) -> Result<Self, String> {
        if !profile.validate() {
            return Err("Unsupported offline CPU benchmark profile".into());
        }
        #[cfg(not(feature = "offline-ct2"))]
        let _ = (&on_result, profile);
        let pending = Arc::new(LatestSlot::<TranslationRequest>::new());
        let worker_pending = pending.clone();
        let telemetry = Arc::new(Mutex::new(Telemetry::default()));
        let state = telemetry.clone();
        thread::Builder::new().name("cliplingo-local-model".into()).spawn(move || {
            #[cfg(feature="offline-ct2")]
            let mut loaded: Option<Loaded> = None;
            #[cfg(feature="offline-ct2")]
            let mut last_used = Instant::now();
            loop {
                let request = match worker_pending.take_timeout(Duration::from_secs(10)) {
                    Next::Item(value) => value,
                    Next::Closed => break,
                    Next::Timeout => {
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
                        match Loaded::load(&model_path, profile) {
                            Ok(value) => loaded = Some(value),
                            Err(reason) => {
                                state.lock().unwrap().state = reason;
                                continue;
                            }
                        }
                    }
                    state.lock().unwrap().state = TranslationState::Translating;
                    let result = loaded.as_ref().expect("loaded above").translate(&request.text, profile, || worker_pending.has_pending());
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
                        Err(TranslationState::Superseded) => {
                            // A newer pending request owns presentation; it is not
                            // a model failure and should be processed next.
                            continue;
                        }
                        Err(reason) => state.lock().unwrap().state = reason,
                    }
                }
            }
        }).map_err(|e| format!("Could not start local model worker: {e}"))?;
        Ok(Self { pending, telemetry })
    }
    pub fn submit(&self, request: TranslationRequest) {
        // Always replace older pending text; never reject newest hotkey simply
        // because obsolete work occupies the single pending slot.
        match self.pending.publish(request) {
            Ok(_) => self.telemetry.lock().unwrap().state = TranslationState::Translating,
            Err(_) => self.telemetry.lock().unwrap().state = TranslationState::Failed,
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

impl Drop for TranslationEngine {
    fn drop(&mut self) {
        self.pending.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn inference_profile_is_bounded_and_preserves_existing_default() {
        assert_eq!(OfflineProfile::default(), OfflineProfile {beam_size: 2, cpu_threads: 2});
        for beam in [1, 2] {
            for threads in [1, 2, 4] {
                assert!(OfflineProfile {beam_size:beam, cpu_threads:threads}.validate());
            }
        }
        assert!(!OfflineProfile {beam_size:0, cpu_threads:2}.validate());
        assert!(!OfflineProfile {beam_size:3, cpu_threads:2}.validate());
        assert!(!OfflineProfile {beam_size:1, cpu_threads:64}.validate());
    }

    #[test]
    fn model_errors_never_create_fake_success() {
        assert_eq!(classify_pack(ModelError::Missing), TranslationState::MissingModel);
        assert_eq!(classify_pack(ModelError::CorruptFile), TranslationState::InvalidModel);
    }
}
