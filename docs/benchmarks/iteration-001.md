# Iteration 001 — Windows Interaction Evidence

## Status

The planned application slice is implemented on `feat/iteration-001-windows-interaction`. Automated verification is green; interactive Windows acceptance remains pending.

This document deliberately separates evidence that GitHub Actions can verify from evidence that requires a real interactive Windows desktop session. An unobserved GUI check must never be recorded as passed.

## Implemented slice

```text
select text in another Windows application
 -> Ctrl+Alt+T
 -> pre-created popup shows Capturing state
 -> UI Automation selection capture
 -> clipboard fallback only when UIA provider is unsupported and clipboard preservation is safe
 -> selection bounds or cursor fallback positioning
 -> deterministic [FAKE] translation
 -> latest request wins
 -> second shortcut / close action dismisses popup
```

Real translation inference is not part of Iteration 001.

## Automated verification

The repository commits both `app/package-lock.json` and `app/src-tauri/Cargo.lock`. The final CI workflow is read-only and uses the committed dependency graph.

GitHub Actions run `31677315164` (run #33) on commit `0122dbcc2edab610ec4168c709c963d9c9f42a10` completed successfully on `windows-latest` with:

```text
npm ci
npm run check
npm test
npm run build
cargo fmt --check
cargo clippy --locked --all-targets --all-features -- -D warnings
cargo test --locked --all
```

Automated result: **PASS**.

The native hardening covered by this gate includes:

- UI Automation falls back to clipboard only for `UIA_E_NOTSUPPORTED`; other UIA failures remain native capture errors;
- clipboard snapshots retain raw UTF-16 units instead of round-tripping through a lossy Rust `String` representation;
- restoration is attempted after clipboard-copy failure/timeout and restore failure takes precedence over reporting capture success;
- unsupported existing clipboard formats are refused before mutation;
- clipboard fallback waits for the `Ctrl+Alt+T` keys to be released before synthesizing `Ctrl+C`, with a bounded timeout;
- GitHub Actions use current Node-24-based majors for checkout, Node setup, and diagnostic artifact upload.

## Privacy review

Implemented logging is metadata-only. Timing records may contain:

- request ID
- metric name
- duration
- capture source (`uia` or `clipboard`)
- structured error category

Selected source text, fake translated text, and clipboard contents must not be logged.

Manual log inspection: **PENDING — requires an interactive run with representative selected text.**

## Compatibility matrix

| Application | Version | Capture path | Bounds | Placement | Result |
|---|---|---|---|---|---|
| Notepad | NOT RUN | NOT RUN | NOT RUN | NOT RUN | PENDING |
| Chromium browser | NOT RUN | NOT RUN | NOT RUN | NOT RUN | PENDING |
| VS Code | NOT RUN | NOT RUN | NOT RUN | NOT RUN | PENDING |
| Selectable PDF reader | NOT RUN | NOT RUN | NOT RUN | NOT RUN | PENDING |

Reason: GitHub Actions runners do not provide the interactive desktop/user selection session required by this acceptance check.

## Clipboard safety checks

Automated policy/unit tests cover:

- clipboard fallback is allowed only after `UIA_E_NOTSUPPORTED`;
- `NoSelection` and other UIA/native failures do not trigger clipboard fallback;
- the Iteration 001 mutation policy accepts only empty or Unicode-text-only clipboard formats;
- other existing clipboard formats are refused rather than intentionally discarded;
- the snapshot representation retains raw UTF-16 units;
- restore failure overrides a nominal capture result;
- pressed-key detection uses the high-order bit from Windows key state.

Interactive proof that an existing Unicode-text clipboard is restored exactly after a real Ctrl+C fallback: **PENDING**.

## Display / DPI checks

| Environment | Result |
|---|---|
| Primary monitor at 100% scale | PENDING |
| Non-100% scaling | PENDING |
| Secondary monitor, if available | PENDING |
| Right/bottom/top-left work-area edges | PENDING |
| Negative-coordinate monitor, if available | PENDING |

The pure placement policy is unit-tested, including flipping above near the bottom edge, right-edge clamping, and negative-coordinate work areas. Real DPI/WebView behavior still requires an interactive Windows session.

## Latency

The application records privacy-safe timings using `std::time::Instant` for:

- hotkey received -> popup show request;
- selection capture duration;
- fake translation duration;
- hotkey received -> ready state request.

These are application-side timing points. They do not claim to measure the exact pixel-present time of WebView2.

| Metric | Samples | p50 | p95 |
|---|---:|---:|---:|
| Hotkey -> popup show request | 0 | NOT MEASURED | NOT MEASURED |
| Capture duration | 0 | NOT MEASURED | NOT MEASURED |
| Hotkey -> ready request | 0 | NOT MEASURED | NOT MEASURED |

Acceptance collection requires at least 20 warm real interactions.

## Idle resources

Measure after 60 seconds idle with the popup hidden and devtools closed:

```powershell
Get-Process cliplingo | Select-Object ProcessName, Id, WorkingSet64, PrivateMemorySize64, CPU
```

| Build | Working set | Private memory | CPU observation |
|---|---:|---:|---|
| Dev | NOT MEASURED | NOT MEASURED | NOT MEASURED |
| Local non-debug | NOT MEASURED | NOT MEASURED | NOT MEASURED |

## Interactive validation procedure

1. Build/run ClipLingo on Windows 11 x64.
2. Put a known plain Unicode string in the clipboard and record it.
3. In each compatibility application, select text and press `Ctrl+Alt+T`.
4. Confirm the popup appears promptly without stealing focus.
5. Record whether capture used UIA or clipboard fallback and whether selection bounds were available.
6. When clipboard fallback is used, verify the original plain-text clipboard is restored exactly.
7. Repeat near monitor edges and under a non-100% display scale if available.
8. Trigger the shortcut repeatedly and confirm an older result never replaces the newest request.
9. Inspect logs and confirm selected/translated text does not appear.
10. Collect at least 20 warm timing samples and the idle process measurements above.

## Exit decision

Automated verification is complete. Iteration 001 remains **IN PROGRESS** until the interactive evidence above is recorded. Iteration 002 must not start based only on compilation/unit tests.
