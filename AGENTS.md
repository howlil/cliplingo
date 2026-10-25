# AGENTS.md — ClipLingo AI Engineering Rules

This is the **repository-wide instruction and skill router** for every AI coding agent (and human contributor). Read this file before modifying the repo. Do not replace the accepted product or design contract with framework preference or speculative architecture.

## 0. Authoritative documents and precedence

```text
User's explicit request and safety constraints
  ├── DESIGN.md                       → approved visual/design tokens and UI composition
  ├── .agents/product.md              → job, observable behavior, scope, acceptance proof
  ├── .agents/engineering-design.md   → Tauri v2 + Rust native Windows contract, owners & evidence gates
  └── AGENTS.md                       → engineering workflow / AI skill routing / code quality
```

Read only the docs relevant to the actual change. The Tauri v2 + Rust/Win32 hybrid target in `engineering-design.md` and ADR-0002 are **not a verified implementation**; open backend/ABI decisions and unmeasured performance claims stay open until tested. Avoid duplicated rules or shadow copies of these contracts.

**Baseline:** ClipLingo is a low-resource, low-latency, offline Windows translation utility. Tray as shell, Settings as separate window, auto-fit target-text-only popup anchored above cursor. **No controls or source text inside translation popup.** **Target architecture: Tauri v2 + Rust-owned Windows capture/coordinator + native Win32/DirectWrite result popup** ([ADR-0003](.agents/decisions/0003-latency-first-dual-mode-translation.md) refines [ADR-0002](.agents/decisions/0002-tauri-rust-windows-hybrid.md)). Settings uses WebView2 on demand; translation is offline by default with an optional explicitly consented online provider. **Current branch F0: experimental Rust/Tauri source, Win32/GDI preview tooltip, static on-demand Settings and Windows CI exist; no functional translator, DirectWrite final renderer, native E2E or release**. Archived C++ source, build and tests are recoverable from `archive/pre-rust-clean-slate-20261026`. Do not claim completed translation functionality, a passing Windows build before CI evidence, native focus/DPI acceptance, release readiness or a validated OCR/model ABI.

## F0 implementation boundary

The `src-tauri/` experimental shell registers Ctrl+Alt+T but **does not translate**. Only explicit tray preview / debug Ctrl+Alt+P displays fixed sample text on a native Win32 GDI HWND. Source capture, offline/online engines, cache, DirectWrite accessibility, update/installer and persisted settings remain unimplemented. Native F0 preview timing measures dispatch-to-WM_PAINT, not visible translation latency. See [README](README.md) and `.agents/f0-acceptance.md`.

## 1. Default router: `$agentflow`

Use the user's **saved `$agentflow` Graph-first Agentic Development workflow**, sourced from `RTK.md`, when available. The notation `$skill-name` explicitly names the user's own saved skill, not a generic brainstorming technique. This file defines the local **routing contract**, not a replacement for the full saved skill. If a specialist skill/source cannot be found, report that limitation rather than invent its contents.

**Choose the smallest useful subset by unresolved decision, not a fixed checklist:**

| Task / unresolved decision | Route |
| --- | --- |
| Job, behavior, scope, acceptance proof | `$product-design` |
| Ownership, boundaries, concurrency, technology trade-offs | `$engineering-design` |
| Nontrivial screen, navigation, focus, accessible states | `$design-graph` |
| Actual implementation shapes, state transitions and failure paths | `$design-thinking` |
| Nontrivial verification strategy, concurrency/fault proof | `$test-engineering` |
| Independent assessment of a substantial implementation diff | `$code-review` |
| Privileged native APIs, unsafe input, secrets, trust boundaries | `$security-review` |
| Production health, recovery, crash dumps, packaging/release readiness | `$production-ops` |
| Meaningful delegation/subagent parallelism | `$graph-protocol` |
| Presenting an established runtime/dependency graph | `$call-graph-output` |

Do **not** invoke every skill mechanically. Do **not** invent subagents or mandatory ceremony for trivial changes. If the request contains an explicit skill invocation, use the corresponding saved skill instructions, including spelling variants with clear intent.

## 2. Delivery workflow

```text
Understand the request and affected contract
  → inspect existing code, state ownership and failure surfaces
  → classify risk (mechanical / local behavior / boundary-state / high risk)
  → choose only needed specialist skills
  → define smallest shippable, verifiable vertical slice
  → implement with explicit state ownership
  → run focused behavioral and native integration checks
  → inspect actual diff for new complexity/broken invariants
  → integrate only after verified acceptance
```

- Prefer one coherent short-lived branch for nontrivial changes; keep integration branch releasable. Avoid long-lived `develop`, `staging`, `release-development` branches and per-skill branches.
- No broad refactor while adding a feature unless it is a prerequisite and separately verified.
- Do not claim code compiled, a test passed, Windows APIs worked, or performance improved without actually running the check.
- If testing on real Windows is unavailable, say so and preserve the relevant acceptance gate as pending.
- Tests prove **observable behavior and failure semantics**, not private method call counts or structural snapshots.

## 3. Engineering quality: SOLID with restraint

**SOLID is a design diagnostic, not a target for number of classes.**

- **S — Single Responsibility:** every module has one coherent *reason to change* and one authoritative mutable-state owner. Keep closely coupled domain behavior together.
- **O — Open/Closed:** extend through a stable boundary **when a second real implementation or requirement exists**. Do not create speculative plugin points.
- **L — Liskov:** only introduce polymorphism where implementations actually preserve a meaningful result/error/lifecycle contract; no misleading interface inheritance.
- **I — Interface Segregation:** prefer narrow ports at OS/worker boundaries over god interfaces, inheritance trees or bloated module APIs.
- **D — Dependency Inversion:** coordinator/application behavior should not know UIA COM detail or HWND drawing; provider/worker boundary details stay behind small owned contracts **when separation gives real value**.

**Prohibited default patterns unless justified by immediate product needs:**
- Over-abstraction, wrappers that only forward to another wrapper, one-class-per-file/one-interface-per-class by ritual.
- Premature factories, generic repositories, service locators, global mutable singleton state, dependency-injection frameworks, internal message buses, microservices or local HTTP backends.
- Duplicate state in Win32 UI, SettingsStore, worker and Coordinator; cross-owner reach-through; circular dependencies.
- Splitting files solely by LOC count or applying SOLID as a replacement for understanding resource lifetime.
- Speculative support for platforms, languages, models, capture backends or install/update services that product does not need.

**Rule of admission:** every new class, interface, process, thread, dependency or abstraction must answer: *what current state invariant, lifecycle boundary, external contract or testability risk justifies it?* If none, keep the code direct and local.

### Tauri + Rust native Windows safety defaults (target implementation)

- Prefer Rust ownership/borrowing and explicit native-resource guards for HWND, HANDLE, COM, DirectWrite and buffers. Every `unsafe` block/function must identify its platform invariant; do not scatter raw pointers through coordinator/domain logic.
- Expected absence (`NoSelection`) and recoverable failures use explicit Rust results/errors; never invent translation success or panic for routine missing selection.
- Rust panics and C++ exceptions must not cross Win32/COM/C callbacks or an FFI ABI. Define ownership, allocator, lifetime, UTF conversion, HRESULT/error mapping and thread affinity at foreign boundaries; Rust does **not** automatically make FFI safe.
- UI thread owns HWND and popup lifecycle. UIA calls run on correctly initialized COM MTA workers or bounded helper processes. Late generations never present stale output; a timeout does not cancel an arbitrary hung COM call.
- Validate model packs, OCR regions and capture/clipboard data; deny excluded or unknown live app sources; avoid captured-text logs, automatic clipboard access, default history and privilege bypass.
- Keep captured source text and OCR pixels in Rust backend only. Settings must never receive them through frontend events or logs. Online transmission is allowed ONLY in explicit per-provider user-consented mode; never silently fall back to cloud, including on model corruption or missing model. Apply Tauri v2 per-window capabilities, restrictive CSP, and explicit Rust command input/authorization checks; permissions do not sandbox Rust.
- For new Rust code use `cargo fmt`, `cargo clippy`, unit/integration tests and appropriate platform checks. Run actual Windows native/UIA tests and analyze any unsafe/FFI boundary; do not substitute lints for sanitizer/handle-soak or fresh-VM packaging proof.
- C++ sources, CMake/MSVC tests and release workflow are **archived, not active** after the user's explicit clean-slate request. Consult the archive for previous behavior and recovery; do not claim its CI still runs on the default branch.
- Low idle use and hotkey-to-result latency **require measured evidence**. Prefer event-driven behavior, on-demand OCR, bounded RAM-only translation cache, lazy Settings WebView2 and on-demand/offline model load. Include WebView2 children in process-tree memory measurements; compare cold/warm latency, native resource ownership and fault containment.

## 4. Module boundary direction (conceptual only)

```text
TauriHost (Rust: tray, hotkey, lifecycle, single instance)
      → RequestCoordinator (generation & frozen cursor/foreground snapshot)
          → NativeCapture (Windows UIA MTA, optional explicit OCR/clipboard)
          → TranslationRouter (offline model lifecycle OR consented official HTTPS provider; cache)
      → PopupOwner (Rust Win32/DirectWrite HWND default; WebView prototype comparison only)
      → SettingsStore (authoritative Rust preferences)
          → SettingsUI (Tauri WebView2 React/TS; validated IPC projection)
```

This is **not a prescribed directory tree** and does not require an interface for each arrow. Prefer a direct function call when two responsibilities share a legitimate owner and no independently meaningful boundary exists. New modules must preserve one-way ownership; don't let UI rendering call capture/inference APIs directly.

## 5. Verification matrix — risk matched to tests

| Change | Minimum faithful proof |
| --- | --- |
| Visual text/popup change | Real Windows Tauri/Win32 popup, intrinsic size, mixed-DPI edges, no-focus-steal and accessibility |
| Selection/hover capture | Valid/empty/denied cases on real target apps; MTA/timeout handling |
| Coordinator/concurrency | Rapid hotkey/reordering/stale response tests |
| Model/worker | Offline fixture/network-off, quality + tokenizer parity, invalid model, load/unload, online consent/auth/429/timeout, cache isolation |
| Native resources | Leak/handle soak, sanitizer/static findings, shutdown/restart |
| Installer/security boundary | Fresh Windows VM, WebView2 runtime, DLL paths, Tauri IPC permissions, signed updater and actual release assets |

Run the smallest fast tests repeatedly. For an affected native boundary, an integration test on Windows is necessary; mocks cannot prove HWND focus or actual UIA provider semantics. For docs-only changes, check links, paths, contradictions and complete contracts; do not pretend an application binary exists.

## 6. Documentation discipline

- If changing **how it looks or behaves on screen**, update `DESIGN.md` after explicit UI decision and product behavior when relevant.
- If changing **what outcome the user receives**, update `.agents/product.md`.
- If changing **ownership, concurrency, runtime topology, Tauri IPC/capabilities, popup rendering choice, resource lifecycle, error policy or package constraints**, update `.agents/engineering-design.md`.
- Create concise ADRs only for consequential **accepted** choices, under `.agents/decisions/`; do not use them for naming, cosmetic refactors or undecided experiments.
- Root `AGENTS.md` is the AI workflow router, **not** a second architecture document or a copy of the saved `RTK.md`.
- Keep experimental assumptions explicitly labeled; preserve approved baseline across implementation iterations.

## 7. Completion report

For implementation work, report: behavior delivered, affected owners/boundaries, files changed, checks **actually executed**, failure paths checked, remaining unverified Windows acceptance, and any ADR/contract updates. For the Rust implementation, distinguish the documented *target* from any genuinely built executable and from archived C++ reference code. Be factual and concise.
