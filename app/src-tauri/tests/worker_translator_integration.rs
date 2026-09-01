use std::env;

use cliplingo_lib::application::WorkerTranslator;
use cliplingo_lib::core::{TranslationRequest, Translator};

#[test]
#[ignore = "requires CLIPLINGO_WORKER_EXE built by the Windows CI worker step"]
fn worker_translator_spawns_worker_and_translates_over_named_pipe() {
    let executable = env::var_os("CLIPLINGO_WORKER_EXE")
        .expect("CLIPLINGO_WORKER_EXE must point to the deterministic worker executable");
    let mut translator = WorkerTranslator::with_executable(executable);

    let first = translator
        .translate(&TranslationRequest {
            text: "こんにちは dunia 🌏".into(),
        })
        .unwrap();
    assert_eq!(first.text, "[FAKE] こんにちは dunia 🌏");

    let second = translator
        .translate(&TranslationRequest {
            text: "selamat malam".into(),
        })
        .unwrap();
    assert_eq!(second.text, "[FAKE] selamat malam");
}
