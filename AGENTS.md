# ClipLingo — Agent Rules

## Authority and project memory
- Use the installed **Agentflow** skill as the sole SWE workflow router. Load only the relevant specialist when a decision is unresolved; deterministic edits go directly to implementation.
- Follow the user's explicit request, this file, and the existing code conventions; do not replace a verified project decision with an invented one.
- Product scope and acceptance: `.agents/product.md`; architecture, state, contracts, and verification stages: `.agents/engineering.md`; UI tokens, states, and interaction rules: `.agents/DESIGN.md`.
- Preserve the v6 visual reference `design/reference/cliplingo-v6.html` as the canonical **non-shipping** HTML prototype. Do not treat simulated behavior as implemented native functionality.

## Stack and commands
- Windows 10/11 native app; C++20, CMake, Win32 shell. UI Automation, translation provider, SQLite, and Slint are planned, **not yet implemented**.
- Configure (VS 2022 on Windows): `cmake --preset windows-msvc`
- Build: `cmake --build --preset windows-msvc`
- Run: `out/build/windows-msvc/Debug/cliplingo.exe`
- Tests: TBD until the first behavior slice installs a real test target; do not invent commands or assert they passed.
- CI: not configured. Add only necessary gates with implementation; avoid redundant runs, CI spam, and AI-invoking CI.

## Intake and scope (nontrivial changes)
- Inspect the relevant code, tests, Git HEAD/diff, architectural contracts, and observed behavior first. For bugs, reproduce and find the root cause before modifying code.
- State a compact contract: **Goal / Scope / Non-goals / 2–5 observable Acceptance criteria / Risk / cheapest faithful Proof**. Prefer direct action over long planning documents.
- Identify the single smallest complete vertical slice. Do not split one outcome into artificial milestones, layers, Issues, or PRs.
- Don't add abstractions, features, checks, dependencies, or refactors unless tied to a current requirement, defect, or invariant.

## Implementation discipline
- Work **sequentially**, not via overlapping or competing changes. One active PR by default; reuse it for the same outcome. Commit logically, not after each edit; avoid commit spam.
- Direct changes on `master` only when the user expressly requests them; no force-push, destructive migration, publish, or release without explicit authorization.
- Finish a vertical slice end-to-end: UI, state, logic, persistence, external effects, recovery, and proof **where the behavior requires them**. Do not label partial work complete.
- Favor simple, bounded modules and public contracts. Business logic must not import UI/Win32/network/SQL adapters; UI must not own text capture, credentials, provider calls, or storage.
- Enforce explicit state ownership and request lifecycle: ordering, concurrency, cancellation, timeout, stale-response suppression, cleanup, and idempotency when applicable.
- Treat OS, clipboard, UIA, HTTP/provider, and persistence interfaces as real external contracts; verify behavior on the actual target environment, not only mocks.
- Maintain v6 component fidelity and `DESIGN.md` tokens/states; component-first, reusable only when a responsibility is genuinely shared. Separate presentation, feature state, logic, and data access.
- Keep native popup passive without stealing source focus unless the user explicitly interacts. Never ship HTML demo text or fake macOS window controls as Windows behavior.
- Default to consent-denied before sending selected text; honor excluded apps. Never expose credentials, selected text, or raw context through logs, metrics, or unapproved persistence.

## Risk-based verification
- Prioritize high-impact/low-effort proofs; reserve high-effort tests for critical risks. No ceremonial tests or coverage targets disconnected from failure impact.
- Apply test-first/TDD to **critical core paths** when it makes correctness clearer (capture policy, latest-wins, consent, vocabulary persistence); reproduce regressions before fixing.
- For critical behavior, cover a **happy path, relevant failure path, and meaningful edge case**, at the cheapest faithful level (unit, integration, component, OS acceptance).
- Include frontend interactions when critical, not just backend logic. Use full browser E2E only when simpler proofs cannot establish the required behavior; avoid gratuitous harnesses.
- `iter`: affected format/build/unit checks. `pr`: integration and risk checks. `merge`: final head proof. `release`: one real Windows desktop compatibility, focus, DPI, clipboard, memory, and latency acceptance.
- Do not run full CI or release jobs on every file change. Re-run failed or affected checks first; keep the full safety gate before integration.
- Report commands, results, failures, and gaps accurately. A mocked HTML preview, a successful commit, or an old green SHA does **not** prove native functionality.

## Review, integration, and status
- Inspect the final diff for scope, architecture/design conformance, correctness, security, error paths, race conditions, cleanup, maintainability, and regression evidence.
- Assign an evidence-backed **Approve / Approve with Minor / Block** verdict. **Block** on missing critical proof, failed required checks, unsafe behavior, or known requirement gaps.
- Target implementation quality **≥8/10**, assessed from actual correctness, readability, maintainability, UX fidelity, and risk-appropriate verification; never invent a score.
- Integrate only when all scoped acceptance criteria and relevant proof pass, the final HEAD is reviewed, and no blocker remains. If not, report the precise blocker and next action.
- Distinguish **IMPLEMENTED** (code committed), **VERIFIED** (required proof passed for exact HEAD), **PUBLISHED/DEPLOYED**, and **RELEASED** (actual observed distribution); merge is not release.
- Keep the project releasable and report what remains unimplemented. Preserve still-valid code/contracts; no speculative rewrites, unrelated changes, or status ledgers.

## Current safety boundaries
- `src/` is native production code; `design/reference/` is a browser-only design reference.
- Respect `.agents/engineering.md` fitness rules (F1–F4) when modules are introduced. Use real CI gates only once the corresponding targets exist.
- Do not claim UIA capture, global hotkeys, translation quality, native focus behavior, memory targets, or release success without appropriate Windows/runtime evidence.
