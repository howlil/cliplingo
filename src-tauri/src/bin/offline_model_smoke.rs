//! Windows CI smoke: real native CTranslate2 + verified on-device EN->ID pack.
//! No UIA or WebView involved. Interactive Windows focus E2E remains separate.
#[cfg(feature = "offline-ct2")]
#[path = "../latest_slot.rs"]
mod latest_slot;
#[cfg(feature = "offline-ct2")]
#[path = "../model_pack.rs"]
mod model_pack;
#[cfg(feature = "offline-ct2")]
#[path = "../translation.rs"]
mod translation;

#[cfg(feature = "offline-ct2")]
fn main() {
    use std::{sync::{mpsc, Arc}, time::{Duration, Instant}};
    use sha2::{Digest, Sha256};
    let mut beam_size = 2usize;
    let mut cpu_threads = 2usize;
    let mut benchmark = false;
    let mut quality_suite: Option<std::path::PathBuf> = None;
    let mut quality_output: Option<std::path::PathBuf> = None;
    let mut arguments = std::env::args().skip(1);
    while let Some(flag) = arguments.next() {
        match flag.as_str() {
            "--benchmark" => benchmark = true,
            "--quality-suite" => quality_suite = Some(arguments.next().expect("--quality-suite path required").into()),
            "--quality-output" => quality_output = Some(arguments.next().expect("--quality-output path required").into()),
            "--beam" => {
                beam_size = arguments.next().expect("--beam requires integer")
                    .parse::<usize>().expect("invalid --beam");
            }
            "--threads" => {
                cpu_threads = arguments.next().expect("--threads requires integer")
                    .parse::<usize>().expect("invalid --threads");
            }
            _ => panic!("Unknown benchmark argument: {flag}"),
        }
    }
    if quality_suite.is_some() != quality_output.is_some() {
        panic!("--quality-suite and --quality-output must be used together");
    }
    let profile = translation::OfflineProfile { beam_size, cpu_threads };
    assert!(profile.validate(), "Unsupported model profile");
    let pack = model_pack::default_pack_dir().expect("LOCALAPPDATA model directory");
    model_pack::verify_pack(&pack).expect("actual converted model pack must verify");

    let (sender, receiver) = mpsc::channel();
    let callback = Arc::new(move |result: translation::TranslationResult| {
        let _ = sender.send(result);
    });
    let engine = translation::TranslationEngine::start_with_profile(callback, profile)
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
    let cold_ms = started.elapsed().as_millis();
    println!("ClipLingo real model smoke: success; target chars={}, elapsed_ms={}",
        target.target.chars().count(), cold_ms);

    if let (Some(suite), Some(output_path)) = (quality_suite, quality_output) {
        #[derive(serde::Deserialize)]
        struct PublicFixture {
            id: String,
            source: String,
        }
        let input = std::fs::read(&suite).expect("read public quality fixture JSON");
        let fixtures: Vec<PublicFixture> =
            serde_json::from_slice(&input).expect("valid public quality fixture JSON");
        assert!(!fixtures.is_empty() && fixtures.len() <= 100,
            "quality suite must contain 1 to 100 public fixtures");
        let mut seen = std::collections::HashSet::new();
        let mut rows = Vec::with_capacity(fixtures.len());
        for (index, fixture) in fixtures.into_iter().enumerate() {
            assert!(!fixture.id.is_empty() && !fixture.source.trim().is_empty()
                    && seen.insert(fixture.id.clone()), "invalid/duplicate quality fixture");
            let begun = Instant::now();
            let generation = (index + 2) as u64;
            engine.submit(translation::TranslationRequest {
                generation,
                anchor: (0, 0),
                source_hwnd: 0,
                text: fixture.source,
            });
            let outcome = receiver.recv_timeout(Duration::from_secs(90))
                .unwrap_or_else(|err| panic!("quality inference {index} failed: {err}; state={:?}",engine.status()));
            assert_eq!(outcome.generation, generation, "quality output ordering mismatch");
            assert!(!outcome.target.trim().is_empty() && !outcome.target.contains('\0'),
                "invalid translated target");
            rows.push(serde_json::json!({
                "id": fixture.id,
                "target": outcome.target,
                "elapsed_ms": begun.elapsed().as_millis(),
            }));
        }
        let data = serde_json::json!({
            "schema": 1,
            "boundary": "native_offline_fixed_public_fixtures",
            "beam_size": profile.beam_size,
            "cpu_threads": profile.cpu_threads,
            "results": rows,
        });
        std::fs::write(&output_path, serde_json::to_vec_pretty(&data).unwrap())
            .expect("write native quality output");
        println!("cliplingo_quality_suite_done count={} beam={} threads={}",
            data["results"].as_array().unwrap().len(), beam_size, cpu_threads);
    }

    if benchmark {
        // Same verified model, same native worker, sequential fixture runs.
        // This is an inference-only diagnostic: no UIA, hotkey or DWM paint.
        let fixtures = [
            "Thank you.", "Good morning.", "Please open the file.",
            "This document is useful.", "The application works offline.",
            "Could you help me?", "I will read it tomorrow.",
            "The setting is available.", "Select one paragraph.",
            "The model is ready.", "Close the window.", "Keep it private.",
        ];
        let mut latency_ms = Vec::with_capacity(fixtures.len());
        let mut output_digest = Sha256::new();
        for (i, fixture) in fixtures.iter().enumerate() {
            let started = Instant::now();
            engine.submit(translation::TranslationRequest {
                generation: (i + 2) as u64,
                anchor: (0, 0),
                source_hwnd: 0,
                text: (*fixture).to_owned(),
            });
            let output = receiver.recv_timeout(Duration::from_secs(60))
                .unwrap_or_else(|err| panic!("warm inference {} failed: {err}; state={:?}", i, engine.status()));
            assert_eq!(output.generation, (i + 2) as u64);
            assert!(!output.target.trim().is_empty());
            assert!(!output.target.contains('\0'));
            // Compare output digests across profiles without revealing strings.
            output_digest.update((output.target.len() as u64).to_le_bytes());
            output_digest.update(output.target.as_bytes());
            latency_ms.push(started.elapsed().as_millis());
        }
        latency_ms.sort_unstable();
        let percentile = |p: f64| -> u128 {
            let rank = ((latency_ms.len() as f64) * p).ceil() as usize;
            latency_ms[rank.saturating_sub(1).min(latency_ms.len()-1)]
        };
        // No sensitive user text, host details or model output is logged.
        println!("cliplingo_headless_benchmark={{\"schema\":2,\"boundary\":\"native_inference_only\",\"sample_type\":\"fixed_public_fixtures\",\"beam_size\":{},\"cpu_threads\":{},\"cold_ms\":{},\"warm_n\":{},\"warm_p50_ms\":{},\"warm_p95_ms\":{},\"warm_max_ms\":{},\"target_sha256\":\"{:x}\"}}",
            beam_size, cpu_threads, cold_ms, latency_ms.len(), percentile(0.50), percentile(0.95), latency_ms[latency_ms.len()-1], output_digest.finalize());
    }
}

#[cfg(not(feature = "offline-ct2"))]
fn main() {
    eprintln!("offline_model_smoke requires --features offline-ct2");
    std::process::exit(2);
}
