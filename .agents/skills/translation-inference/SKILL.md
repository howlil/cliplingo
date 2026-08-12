---
name: translation-inference
description: Use for CTranslate2/C++ worker, tokenization, model evaluation, quantization, routing, model packs, and inference benchmarks.
---

# Translation Inference Skill

## Baseline

- Isolated C++ worker process.
- CTranslate2 + SentencePiece as the current candidate runtime stack.
- CPU-first and INT8 as the initial benchmark baseline.
- One or two loaded models maximum initially.
- Worker can exit after inactivity to reclaim memory.

These are design choices to validate, not permission to lock a model before quality/license evaluation.

## Model selection

Evaluate at least:

- translation quality on representative CJK/English→Indonesian text
- short-text warm/cold latency
- memory and disk size
- tokenizer/runtime compatibility
- license, commercial/redistribution rights, attribution
- maintenance/source provenance

Do not choose a model solely because it supports many languages.

## Routing

Prefer direct routes when measured quality justifies them. A pivot route such as source→English→Indonesian is a fallback strategy, not an assumption that it is always better.

## Worker contract

Worker knows only model load/translate/unload/health/shutdown concerns. It must not access UI, clipboard, hotkeys, Tauri state, or user settings directly.

## Model packs

Version, checksum and license metadata are mandatory. Partial downloads are never installed. Model update lifecycle is independent from app binary versioning.

## Performance

Optimize interactive p95 latency, memory and power rather than server throughput. Beam/thread settings must come from benchmarks on target-class hardware.

## Reference

- https://opennmt.net/CTranslate2/
