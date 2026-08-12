# ClipLingo Agent Entry Point

This file is the mandatory entry point for any coding agent or engineer working in this repository.

## Read order

Before changing code, read only what is needed, in this order:

1. `.agents/PROJECT.md` — product goal, scope, non-goals.
2. `.agents/ITERATION_STATE.md` — what is being built now.
3. `.agents/RULES.md` — non-negotiable engineering and Git rules.
4. `.agents/DESIGN.md` — architecture and system invariants.
5. `.agents/CODE_PATTERNS.md` — approved implementation patterns.
6. `.agents/SDLC.md` — TDD, CI, PR, merge, and environment workflow.
7. `.agents/skills/<relevant-skill>/SKILL.md` — only the skills relevant to the task.

Read `.agents/RELEASE.md` for packaging/release work and `.agents/DOCUMENTATION_PLAN.md` when behavior or architecture changes.

## Operating principles

- Build the smallest correct solution for the current iteration.
- Prefer boring, explicit code over abstractions with no demonstrated value.
- Apply TDD for behavior changes: RED → GREEN → REFACTOR.
- Keep external/volatile dependencies behind narrow boundaries; do not abstract stable internal code just to make it look architectural.
- The native core owns product behavior. Svelte renders view models; it is not the application core.
- Translation is offline by default. Never send selected text to a remote service unless a future feature explicitly requires it and the user opts in.
- Optimize from measurements, not assumptions.
- Do not add a server, database, GPU requirement, OCR pipeline, or broad language support unless the current iteration requires it.

## Task discipline

For one task or bugfix, use at most one working branch. Continue the same branch for small follow-ups and failed tests. Intermediate RED/GREEN commits are allowed while developing, but normal tasks must reach `master` through a squash merge so history contains one clean commit per task.

Do not create formatting-only, typo-only, CI-retry, or "fix previous commit" commits when the change belongs to the same task. Delete the task branch after merge when tooling allows it.

Update `.agents/ITERATION_STATE.md` when a completed task changes the current capability, next step, blocker, or architectural evidence.

## Skill routing

- Architecture/dependency decisions: `principal-engineering`
- Tauri shell and IPC exposure: `tauri`
- Rust core: `rust`
- Svelte UI: `svelte`
- Win32/UI Automation/desktop behavior: `windows-desktop`
- Core workflows and ports/adapters: `backend-core`
- ML inference/model packs: `translation-inference`
- Tests: `tdd`
- Latency/memory/CPU: `performance`
- Privacy/capabilities/supply chain: `security-privacy`
- CI, packaging, versioning, distribution: `release-engineering`

If a task crosses several areas, combine only the skills required by the actual change. Do not invoke every skill by default.
