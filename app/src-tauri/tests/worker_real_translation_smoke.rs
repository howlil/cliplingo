use std::env;
use std::time::Instant;

use cliplingo_lib::application::WorkerTranslator;
use cliplingo_lib::core::{TranslationRequest, Translator};

#[test]
#[ignore = "release qualification requires the built worker and production OPUS model pack"]
fn real_worker_translates_english_without_test_mode() {
    let executable = env::var_os("CLIPLINGO_WORKER_EXE")
        .expect("CLIPLINGO_WORKER_EXE must point to the production worker executable");
    let model_pack = env::var_os("CLIPLINGO_MODEL_PACK")
        .expect("CLIPLINGO_MODEL_PACK must point to the production OPUS model pack");

    assert!(env::var_os("CLIPLINGO_WORKER_TEST_MODE").is_none());
    let mut translator = WorkerTranslator::with_executable_and_model(executable, model_pack);

    let cold_started = Instant::now();
    let first = translator
        .translate(&TranslationRequest {
            text: "The deployment failed yesterday.".into(),
        })
        .expect("production worker should translate English directly to Indonesian");
    let cold_ms = cold_started.elapsed().as_millis();

    assert!(!first.text.trim().is_empty());
    assert!(!first.text.starts_with("[FAKE]"));
    assert_ne!(first.text.trim(), "The deployment failed yesterday.");

    let warm_started = Instant::now();
    let second = translator
        .translate(&TranslationRequest {
            text: "Thank you for your help.".into(),
        })
        .expect("warm production worker should translate a second English request");
    let warm_ms = warm_started.elapsed().as_millis();

    assert!(!second.text.trim().is_empty());
    assert!(!second.text.starts_with("[FAKE]"));
    eprintln!(
        "event=real_translation_smoke route=en-id cold_ms={cold_ms} warm_ms={warm_ms} output_chars={}",
        second.text.chars().count()
    );
}
