use std::env;
use std::path::PathBuf;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use cliplingo_lib::application::{InteractionCoordinator, WorkerTranslator};
use cliplingo_lib::core::{
    CaptureError, PopupPort, PopupViewModel, Selection, SelectionProvider, SelectionSource,
};

const SOURCE_TEXT: &str = "おはようございます。";
const READY_TIMEOUT: Duration = Duration::from_secs(180);

struct FixedJapaneseSelection;

impl SelectionProvider for FixedJapaneseSelection {
    fn capture(&mut self) -> Result<Selection, CaptureError> {
        Ok(Selection {
            text: SOURCE_TEXT.into(),
            source: SelectionSource::UiAutomation,
            bounds: None,
            work_area: None,
        })
    }
}

struct NoopPopup;

impl PopupPort for NoopPopup {
    fn show(&self, _state: PopupViewModel) {}
    fn update(&self, _state: PopupViewModel) {}
    fn move_to(&self, _x: f64, _y: f64) {}
    fn hide(&self) {}
}

fn wait_until_ready(coordinator: &InteractionCoordinator) -> PopupViewModel {
    let deadline = Instant::now() + READY_TIMEOUT;
    loop {
        let snapshot = coordinator.snapshot();
        match snapshot.status {
            "ready" => return snapshot,
            "error" => panic!("offline translation reached popup error state"),
            _ if Instant::now() >= deadline => {
                panic!("offline translation did not reach ready popup state before timeout")
            }
            _ => thread::sleep(Duration::from_millis(10)),
        }
    }
}

fn assert_real_translation(snapshot: &PopupViewModel) {
    assert!(snapshot.status == "ready", "popup must be ready");
    assert!(
        snapshot.source_text.as_deref() == Some(SOURCE_TEXT),
        "popup source text must match the captured selection"
    );

    let translated = snapshot
        .translated_text
        .as_deref()
        .expect("ready popup must contain translated text");
    assert!(!translated.is_empty(), "real translation must not be empty");
    assert!(
        translated != SOURCE_TEXT,
        "real translation must differ from the Japanese source"
    );
    assert!(
        !translated.starts_with("[FAKE]"),
        "qualification must not use deterministic test mode"
    );
    assert!(
        translated
            .chars()
            .any(|character| character.is_ascii_alphabetic()),
        "translated result must contain Latin-script output"
    );
}

#[test]
#[ignore = "requires a built worker and real CLIPLINGO_MODEL_PACK"]
fn selected_japanese_reaches_popup_through_real_offline_worker() {
    let executable = PathBuf::from(
        env::var_os("CLIPLINGO_WORKER_EXE")
            .expect("CLIPLINGO_WORKER_EXE must point to the built worker executable"),
    );
    let model_pack = PathBuf::from(
        env::var_os("CLIPLINGO_MODEL_PACK")
            .expect("CLIPLINGO_MODEL_PACK must point to the built OPUS model pack"),
    );
    assert!(executable.is_file(), "worker executable must exist");
    assert!(model_pack.is_dir(), "model pack directory must exist");
    assert!(
        env::var_os("CLIPLINGO_WORKER_TEST_MODE").is_none(),
        "real qualification must not enable deterministic worker mode"
    );

    let coordinator = InteractionCoordinator::start(
        Box::new(FixedJapaneseSelection),
        Box::new(WorkerTranslator::with_executable(executable)),
        Arc::new(NoopPopup),
    );

    let cold_started = Instant::now();
    coordinator.trigger(cold_started);
    let cold_snapshot = wait_until_ready(coordinator.as_ref());
    let cold_elapsed = cold_started.elapsed();
    assert_real_translation(&cold_snapshot);

    let warm_started = Instant::now();
    coordinator.trigger(warm_started);
    let warm_snapshot = wait_until_ready(coordinator.as_ref());
    let warm_elapsed = warm_started.elapsed();
    assert_real_translation(&warm_snapshot);

    let translated_chars = warm_snapshot
        .translated_text
        .as_deref()
        .map(|text| text.chars().count())
        .unwrap_or(0);
    println!(
        "offline_alpha cold_ms={} warm_ms={} translated_chars={translated_chars}",
        cold_elapsed.as_millis(),
        warm_elapsed.as_millis()
    );
}
