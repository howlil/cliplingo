//! Windows CI smoke: real native CTranslate2 + verified on-device EN->ID pack.
//! No UIA or WebView involved. Interactive Windows focus E2E remains separate.
#[cfg(feature = "offline-ct2")]
#[path = "../model_pack.rs"]
mod model_pack;
#[cfg(feature = "offline-ct2")]
#[path = "../translation.rs"]
mod translation;

#[cfg(feature = "offline-ct2")]
fn main() {
    use std::{sync::{mpsc, Arc}, time::{Duration, Instant}};
    let pack = model_pack::default_pack_dir().expect("LOCALAPPDATA model directory");
    model_pack::verify_pack(&pack).expect("actual converted model pack must verify");

    let (sender, receiver) = mpsc::channel();
    let callback = Arc::new(move |result: translation::TranslationResult| {
        let _ = sender.send(result);
    });
    let engine = translation::TranslationEngine::start(callback)
        .expect("spawn CPU translation worker");
    let started = Instant::now();
    let source = "Good morning.";
    engine.submit(translation::TranslationRequest {
        generation: 1,
        anchor: (0, 0),
        source_hwnd: 0,
        text: source.into(),
    });
    let target = receiver.recv_timeout(Duration::from_secs(150))
        .unwrap_or_else(|err| panic!("real model produced no translation within 150s: {err}; state={:?}", engine.status()));
    assert_eq!(target.generation, 1, "generation must survive inference");
    assert!(!target.target.trim().is_empty(), "real model returned blank translation");
    assert_ne!(target.target.trim().to_lowercase(), source.to_lowercase(), "inference returned unmodified English input");
    println!("ClipLingo real model smoke: success; target chars={}, elapsed_ms={}",
        target.target.chars().count(), started.elapsed().as_millis());
}

#[cfg(not(feature = "offline-ct2"))]
fn main() {
    eprintln!("offline_model_smoke requires --features offline-ct2");
    std::process::exit(2);
}
