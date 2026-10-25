# AGENTS.md — ClipLingo AI Engineering Rules

This is the **repository-wide instruction and skill router** for every AI coding agent (and human contributor). Read this file before modifying the repo. Do not replace the accepted product or design contract with framework preference or speculative architecture.

## 0. Authoritative documents and precedence

```text
User's explicit request and safety constraints
  ├── DESIGN.md                       → approved visual/design tokens and UI composition
  ├── .agents/product.md              → job, observable behavior, scope, acceptance proof
  ├── .agents/engineering-design.md   → Rust-first migration target, boundaries, state owners & guarantees
  └── AGENTS.md                       → engineering workflow / AI skill routing / code quality
```

Read only the docs relevant to the actual change. The Rust-first target in `engineering-design.md` and its ADR are **not a verified Rust implementation**; open backend/ABI decisions and unmeasured performance claims stay open until tested. Avoid duplicated rules or shadow copies of these contracts.

**Baseline:** ClipLingo is a low-resource, low-latency, offline Windows translation utility. Tray as shell, Settings as separate window, auto-fit target-text-only popup anchored above cursor. **No controls or source text inside translation popup.** **Target architecture: Rust-first native Windows** ([ADR-0001](.agents/decisions/0001-rust-first-native-windows.md)). **Current repository: documentation-only clean slate with no runnable application or CI**. Archived C++ source, build and tests are recoverable from `archive/pre-rust-clean-slate-20261026`. Do not claim any Rust binary exists, that tests passed, or that release/OS support and native OCR ABI are verified.

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

### Rust-first native Windows safety defaults (target implementation)

- Prefer Rust ownership/borrowing and explicit native-resource guards for HWND, HANDLE, COM, DirectWrite and buffers. Every `unsafe` block/function must identify its platform invariant; do not scatter raw pointers through coordinator/domain logic.
- Expected absence (`NoSelection`) and recoverable failures use explicit Rust results/errors; never invent translation success or panic for routine missing selection.
- Rust panics and C++ exceptions must not cross Win32/COM/C callbacks or an FFI ABI. Define ownership, allocator, lifetime, UTF conversion, HRESULT/error mapping and thread affinity at foreign boundaries; Rust does **not** automatically make FFI safe.
- UI thread owns HWND and popup lifecycle. UIA calls run on correctly initialized COM MTA workers or bounded helper processes. Late generations never present stale output; a timeout does not cancel an arbitrary hung COM call.
- Validate model packs, OCR regions and capture/clipboard data; deny excluded or unknown live app sources; avoid captured-text logs, automatic clipboard access, default history and privilege bypass.
- For new Rust code use `cargo fmt`, `cargo clippy`, unit/integration tests and appropriate platform checks. Run actual Windows native/UIA tests and analyze any unsafe/FFI boundary; do not substitute lints for sanitizer/handle-soak or fresh-VM packaging proof.
- C++ sources, CMake/MSVC tests and release workflow are **archived, not active** after the user's explicit clean-slate request. Consult the archive for previous behavior and recovery; do not claim its CI still runs on the default branch.
- Low idle use and hotkey-to-result latency **require measured evidence**. Prefer event-driven behavior and on-demand OCR; compare measured cold/warm latency, RAM, native resource ownership and fault containment before changing process/model topology.

## 4. Module boundary direction (conceptual only)

```text
WindowsHost (Rust-owned Win32 tray/hotkey, lifecycle)
      → RequestCoordinator (request generation & cursor snapshot)
          → Capture (UIA / optional OCR)
          → Translation (offline model lifecycle; native CTranslate2 FFI only if needed)
      → Popup (text layout/measurement, HWND only)
      → Settings (authoritative local preferences)
```

This is **not a prescribed directory tree** and does not require an interface for each arrow. Prefer a direct function call when two responsibilities share a legitimate owner and no independently meaningful boundary exists. New modules must preserve one-way ownership; don't let UI rendering call capture/inference APIs directly.

## 5. Verification matrix — risk matched to tests

| Change | Minimum faithful proof |
| --- | --- |
| Visual text/popup change | Real Windows native rendering, intrinsic size, DPI/edges, focus and accessibility |
| Selection/hover capture | Valid/empty/denied cases on real target apps; MTA/timeout handling |
| Coordinator/concurrency | Rapid hotkey/reordering/stale response tests |
| Model/worker | Offline fixture, invalid/missing model, load/unload and error lifecycle |
| Native resources | Leak/handle soak, sanitizer/static findings, shutdown/restart |
| Installer/security boundary | Fresh OS VM, DLL load path, signature/update and permission constraints |

Run the smallest fast tests repeatedly. For an affected native boundary, an integration test on Windows is necessary; mocks cannot prove HWND focus or actual UIA provider semantics. For docs-only changes, check links, paths, contradictions and complete contracts; do not pretend an application binary exists.

## 6. Documentation discipline

- If changing **how it looks or behaves on screen**, update `DESIGN.md` after explicit UI decision and product behavior when relevant.
- If changing **what outcome the user receives**, update `.agents/product.md`.
- If changing **ownership, concurrency, runtime topology, resource lifecycle, error policy or package constraints**, update `.agents/engineering-design.md`.
- Create concise ADRs only for consequential **accepted** choices, under `.agents/decisions/`; do not use them for naming, cosmetic refactors or undecided experiments.
- Root `AGENTS.md` is the AI workflow router, **not** a second architecture document or a copy of the saved `RTK.md`.
- Keep experimental assumptions explicitly labeled; preserve approved baseline across implementation iterations.

## 7. Completion report

For implementation work, report: behavior delivered, affected owners/boundaries, files changed, checks **actually executed**, failure paths checked, remaining unverified Windows acceptance, and any ADR/contract updates. For the Rust implementation, distinguish the documented *target* from any genuinely built executable and from archived C++ reference code. Be factual and concise.
