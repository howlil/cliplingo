# `.agents` Engineering System

This directory is the repository-local source of truth for agent behavior and engineering decisions. It is intentionally small enough to read selectively.

## Core documents

| File | Purpose |
|---|---|
| `PROJECT.md` | Product description, scope, priorities, non-goals |
| `DESIGN.md` | Runtime architecture, boundaries, constraints, data flow |
| `RULES.md` | Non-negotiable engineering rules |
| `CODE_PATTERNS.md` | Preferred code patterns and anti-patterns |
| `SDLC.md` | TDD, Git, CI, review, environments, definition of done |
| `RELEASE.md` | Versioning, signing, packaging, updater, distribution |
| `DOCUMENTATION_PLAN.md` | What documentation exists and when to update it |
| `ITERATION_STATE.md` | Current capability, next iteration, risks and decisions |

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

Documentation must follow working software. Do not create speculative documents for subsystems that do not exist. When implementation invalidates an architectural statement, update the relevant document in the same task.
