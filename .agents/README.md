# `.agents` Engineering Context

This directory contains project-local engineering context retained during the Devland migration. It is intentionally small enough to read selectively.

Canonical project facts and project-specific delivery constraints live in `.devland/project.yaml`. Current approved work state lives in `.devland/state.yaml`. Root `AGENTS.md` is the routing entry point.

The documents here provide deeper project-specific design, implementation, release, and historical context. Where universal process text is duplicated, Devland core is the baseline; deliberate ClipLingo-specific constraints recorded in `.devland/project.yaml` take precedence for this repository and are mirrored in `RULES.md` / `SDLC.md` for implementation clarity.

## Core documents

| File | Purpose |
|---|---|
| `PROJECT.md` | Legacy/deeper product description, scope, priorities, non-goals |
| `DESIGN.md` | Runtime architecture, boundaries, constraints, data flow |
| `RULES.md` | ClipLingo-specific engineering and risk constraints |
| `CODE_PATTERNS.md` | Preferred code patterns and anti-patterns |
| `SDLC.md` | TDD, fast verified delivery, Git, CI, review, risk-based verification, definition of done |
| `RELEASE.md` | Versioning, signing, packaging, updater, distribution |
| `DOCUMENTATION_PLAN.md` | What documentation exists and when to update it |
| `ITERATION_STATE.md` | Legacy iteration context; canonical active work is `.devland/state.yaml` |

## Delivery posture

ClipLingo optimizes for **fast verified delivery**:

```text
problem/evidence
  -> acceptance behavior
  -> RED
  -> GREEN
  -> REFACTOR
  -> focused verification
  -> broader risk-based verification
  -> PR / CI
  -> merge
  -> observe / measure
```

Keep vertical slices small, WIP low, and feedback fast. Native Windows behavior, IPC/process lifecycle, privacy/security, updater/release, real-model integration, concurrency, and performance-sensitive changes require broader evidence than low-risk local changes.

Delivery health is evaluated using cycle time, PR lead time, CI feedback time, change failures, escaped defects, rework, flaky tests, WIP age, and release frequency when measurable. Commit count, branch count, PR count, and lines changed are not productivity KPIs.

## Skills

Skills are focused checklists for work in a particular area. They are not permission to introduce new architecture.

- `skills/principal-engineering/SKILL.md`
- `skills/tauri/SKILL.md`
- `skills/rust/SKILL.md`
- `skills/svelte/SKILL.md`
- `skills/windows-desktop/SKILL.md`
- `skills/backend-core/SKILL.md`
- `skills/translation-inference/SKILL.md`
- `skills/tdd/SKILL.md`
- `skills/performance/SKILL.md`
- `skills/security-privacy/SKILL.md`
- `skills/release-engineering/SKILL.md`

## Maintenance rule

Documentation must follow working software. Do not create speculative documents for subsystems that do not exist. Update canonical `.devland` state/facts when they change; update the relevant `.agents` document in the same task when deeper architecture, implementation, release, or compatibility guidance changes.
