# `.agents` Project Engineering Context

`.devland/project.yaml` and `.devland/state.yaml` are canonical. `.agents` contains project-specific implementation guidance only. Read selectively; do not load the whole directory by default.

## Required components

| Concern | Source |
|---|---|
| System design / boundaries | `DESIGN.md` |
| Requirement slicing / iteration | `REQUIREMENTS.md` |
| Engineering rules / legacy policy | `RULES.md` |
| Code patterns / anti-patterns | `CODE_PATTERNS.md` |
| Git / CI / testing / Definition of Done | `SDLC.md` |
| Release strategy | `RELEASE.md` |
| Delivery health | `METRICS.md` |
| Task-specific execution checklists | `skills/*/SKILL.md` |

## Operating principle

```text
small requirement
  -> smallest vertical slice
  -> focused implementation + test
  -> fast CI
  -> merge to master
  -> observe
  -> next slice
```

Iterations group intent; they do **not** create long-lived branches or all-or-nothing merge gates. Stable-grade compatibility, performance, signing, and release evidence belongs at the maturity stage that actually needs it.

## Legacy context

`PROJECT.md`, `ITERATION_STATE.md`, `DOCUMENTATION_PLAN.md`, and large historical plans are not canonical process inputs. Keep historical artifacts only when they preserve useful rationale. New agents must not treat them as blockers or duplicate state from them.

When a historical document becomes misleading and its useful decisions already exist in canonical/current docs, delete it rather than maintaining two truths.

## Skills

Load only skills relevant to the touched boundary. Skills guide execution; they do not authorize new architecture or dependencies.

## Maintenance

Prefer fewer durable documents over duplicated policy. Working software and repository evidence lead; documentation follows observable behavior.
