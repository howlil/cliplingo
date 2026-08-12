# Code Patterns

Patterns here are defaults, not a demand for ceremony.

## 1. Boundary-first traits, concrete internals

Use traits where there is a real external/volatile boundary:

```rust
pub trait SelectionProvider {
    fn capture(&self) -> Result<Selection, SelectionError>;
}

pub trait Translator {
    fn translate(&self, request: TranslationRequest)
        -> Result<Translation, TranslationError>;
}
```

Good candidates: Windows selection provider, worker translator client, model catalog/download source.

Do **not** create traits for every service, repository, helper, formatter, or state object. Concrete internal types are preferred until substitution is required.

## 2. Use-case owns workflow

Keep one cohesive use case rather than micro-use-cases:

```text
TranslateSelection
  capture
  normalize
  detect
  resolve route
  translate
  present
```

Do not split this into `GetTextUseCase`, `DetectLanguageUseCase`, `LoadModelUseCase`, etc. unless independent reuse emerges.

## 3. Explicit domain errors

Prefer actionable enums over stringly errors:

```text
NoSelection
SelectionUnsupported
ClipboardUnavailable
LanguageUnknown
RouteUnavailable
ModelMissing
ModelCorrupt
WorkerUnavailable
WorkerCrashed
TranslationTimeout
```

Map them to user-facing messages only at the presentation boundary.

## 4. State machines over boolean combinations

Popup:

```text
Hidden -> Capturing -> Translating -> Ready
                         \-> Error
```

Worker:

```text
Stopped -> Starting -> Ready <-> Busy
                     \-> Failed
```

Avoid combinations such as `loading=true`, `ready=true`, `error=false` that can represent impossible states.

## 5. Latest request wins

Every interactive request has an identity. When request `N+1` becomes current, results from `N` are ignored. Keep at most one active request and one latest pending request if cancellation is not immediately possible.

## 6. Thin Tauri adapters

Good:

```text
command copy_translation
  -> validate transport args
  -> call application method
  -> map result
```

Bad:

```text
command translate_selection
  -> UI Automation
  -> language detection
  -> model selection
  -> process spawn
  -> config writes
```

## 7. View models across the UI boundary

Svelte should receive presentation data, not Rust internals:

```text
PopupViewModel {
  status,
  source_text,
  translated_text,
  source_language,
  target_language,
  can_copy,
  error
}
```

Rust remains the source of truth. UI reload must not become application-state loss.

## 8. IPC protocol stays small

Start with a versioned, length-delimited request/response protocol over Named Pipe. JSON is acceptable for the first working protocol because messages are tiny; replace serialization only if profiling proves it matters.

Conceptual messages:

```text
TranslateRequest { protocol_version, request_id, source, target, text }
TranslateResponse { protocol_version, request_id, translation, inference_ms }
```

No REST server, no gRPC, no GraphQL.

## 9. Atomic model installation

```text
download -> temp file/dir -> checksum + manifest verification -> atomic rename
```

Never write a downloading model directly into the installed path.

## 10. Structured privacy-safe logging

Good:

```text
request=491 capture=uia chars=43 route=ja-en>en-id worker=warm inference_ms=186 status=ok
```

Bad:

```text
text="confidential contract ..."
translation="..."
```

## 11. Configuration

Start with a schema-versioned JSON configuration. Migrate old schemas explicitly when needed. Do not add SQLite merely for preferences.

## 12. Unsafe/native code

Keep `unsafe` and COM/Win32 details inside focused Windows adapter modules. Document safety assumptions next to the unsafe block. Safe core code should not need to understand raw HWND/COM lifetime details.

## 13. Dependency replacement

Agnosticism is achieved at meaningful seams, not by hiding every library:

```text
core -> Translator port -> Worker client -> CTranslate2 worker
core -> SelectionProvider -> Windows UIA implementation
UI -> application commands/view models -> Tauri transport
```

If CTranslate2 or Tauri is replaced later, the core workflow should survive. Do not add a second implementation today merely to prove replaceability.
