# Milestone — Real Offline Translation Alpha

## Why

ClipLingo has proven the isolated Rust -> Named Pipe -> C++ worker boundary with deterministic translation. The next risk is replacing the deterministic worker behavior with a real offline translation runtime without coupling the shell to model-specific details.

## Product decision

The first real route is **Japanese -> Indonesian** using an **English pivot**:

`Japanese -> English -> Indonesian`

The alpha baseline uses two Helsinki-NLP OPUS-MT Marian models:

1. `Helsinki-NLP/opus-mt-ja-en`
2. `Helsinki-NLP/opus-mt-en-id`

Both are Apache-2.0 licensed and are supported by CTranslate2 through its MarianMT/Transformers conversion path.

The initial runtime format is CTranslate2 **int8** on CPU. This is a benchmarkable alpha decision, not a permanent claim that pivot routing is superior to every direct model.

NLLB-200 distilled 600M was evaluated as an architectural candidate but is **not** selected for the distributable baseline because its model license is CC-BY-NC-4.0.

## Desired end state

With a valid local Japanese -> Indonesian language pack installed, the existing popup workflow sends selected Japanese text to the isolated worker and receives a real Indonesian translation while remaining offline during normal translation.

## Scope

### Slice A — Model-pack foundation

- define a versioned model-pack catalog entry for Japanese -> Indonesian;
- pin upstream model IDs and revisions;
- pin conversion tool versions;
- produce reproducible CTranslate2 int8 stage directories;
- validate pack layout without downloading models in normal CI;
- keep generated model binaries out of Git.

Acceptance:
- route explicitly records `ja -> en -> id`;
- both upstream models are Apache-2.0;
- source revisions are pinned, not floating `main`;
- CTranslate2 conversion uses int8;
- generated pack layout has two ordered stages with tokenizer files;
- normal CI validates the catalog/build plan without downloading model weights.

### Slice B — CTranslate2 worker runtime

- add CTranslate2 + SentencePiece behind a worker-local translation engine interface;
- load one stage from a pack directory;
- tokenize, translate, and detokenize locally;
- map model-load/inference failures to protocol errors without payload logging;
- keep the deterministic engine available only as a test fixture.

Acceptance:
- worker can execute a real converted Marian stage on CPU;
- no Python runtime is required in the shipped worker;
- model files are loaded from an external pack directory;
- inference failure does not crash the shell.

### Slice C — Japanese -> Indonesian pivot

- load the pinned `ja-en` and `en-id` stages;
- translate stage 1 output directly into stage 2;
- preserve request correlation and current worker lifecycle semantics;
- keep the route fixed to Japanese -> Indonesian for this alpha.

Acceptance:
- a Japanese sentence produces non-fake Indonesian output through both real stages;
- the worker still communicates through protocol v1;
- selected/intermediate/translated text is absent from normal logs;
- worker restart behavior remains bounded.

### Slice D — Benchmark and qualification

- record cold/warm latency and peak/RSS memory on the CI/development Windows baseline;
- record pack disk size;
- run a small fixed Japanese -> Indonesian quality corpus with human-readable expected intent notes;
- decide whether this pivot baseline is good enough to keep or whether a direct/per-language alternative should replace it.

Acceptance:
- benchmark evidence is recorded in-repo;
- quality limitations are explicit;
- the next CJK route is not added until this first route is qualified.

## Non-goals

- automatic language detection;
- Chinese or Korean runtime packs in this milestone;
- arbitrary language-to-language routing;
- model download/settings UI;
- installer/signing/package-manager release work;
- GPU inference;
- OCR;
- manual interactive Windows testing as a merge blocker.

## Fixed constraints

- Indonesian remains the primary target language.
- Runtime inference stays in the isolated C++ worker.
- Rust remains application/workflow owner.
- Normal translation must be offline after the language pack is installed.
- Model binaries are external artifacts, not Git blobs.
- Model/runtime licensing must permit intended distribution; non-commercial-only weights are not the default distributable baseline.

## Milestone gate

The milestone is complete when automated Windows evidence proves:

`selected Japanese text -> Rust shell -> isolated C++ worker -> OPUS ja-en -> OPUS en-id -> Indonesian result -> popup contract`

with bounded failure handling, offline inference, and no text payload logging.
