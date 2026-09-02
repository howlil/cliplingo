use std::env;
use std::time::Instant;

use cliplingo_lib::application::WorkerTranslator;
use cliplingo_lib::core::{TranslationRequest, Translator};

#[test]
#[ignore = "release qualification requires the built worker and production OPUS model pack"]
fn real_worker_translates_japanese_without_test_mode() {
    let executable = env::var_os("CLIPLINGO_WORKER_EXE")
        .expect("CLIPLINGO_WORKER_EXE must point to the production worker executable");
    let model_pack = env::var_os("CLIPLINGO_MODEL_PACK")
        .expect("CLIPLINGO_MODEL_PACK must point to the production OPUS model pack");

    assert!(env::var_os("CLIPLINGO_WORKER_TEST_MODE").is_none());
    let mut translator = WorkerTranslator::with_executable_and_model(executable, model_pack);

    let cold_started = Instant::now();
    let first = translator
        .translate(&TranslationRequest {
            text: "こんにちは。今日は良い天気です。".into(),
        })
        .expect("production worker should translate Japanese through both OPUS stages");
    let cold_ms = cold_started.elapsed().as_millis();

    assert!(!first.text.trim().is_empty());
    assert!(!first.text.starts_with("[FAKE]"));
    assert_ne!(first.text.trim(), "こんにちは。今日は良い天気です。");

    let warm_started = Instant::now();
    let second = translator
        .translate(&TranslationRequest {
            text: "ありがとうございます。".into(),
        })
        .expect("warm production worker should translate a second request");
    let warm_ms = warm_started.elapsed().as_millis();

    assert!(!second.text.trim().is_empty());
    assert!(!second.text.starts_with("[FAKE]"));
    eprintln!(
        "event=real_translation_smoke cold_ms={cold_ms} warm_ms={warm_ms} output_chars={}",
        second.text.chars().count()
    );
}
