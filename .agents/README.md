# `.agents` Project Engineering Context

`.devland/project.yaml` and `.devland/state.yaml` are canonical for current project truth and work state. `.agents` contains the operating rules, architecture guidance, engineering conventions, and task-specific skills needed to execute that state.

Read selectively; do not load the whole directory by default.

## Precedence and agent behavior

Start with `AGENT_FLOW.md` for authority, lifecycle, scope control, Feature Compass, stop conditions, and retrospective behavior.

Current explicit user intent overrides older repository guidance. Do not let a historical document silently redefine product scope, architecture authority, or acceptance criteria.

## Required components

| Concern | Source |
|---|---|
| Agent authority / lifecycle / scope / Feature Compass | `AGENT_FLOW.md` |
| System design / boundaries | `DESIGN.md` |
| Requirement slicing / iteration | `REQUIREMENTS.md` |
| Engineering rules / legacy policy | `RULES.md` |
| Code patterns / anti-patterns | `CODE_PATTERNS.md` |
| Git / CI / testing / Definition of Done | `SDLC.md` |
| Release strategy | `RELEASE.md` |
| Delivery health | `METRICS.md` |
| Task-specific execution checklists | `skills/*/SKILL.md` |

## Canonical lifecycle

```text
USER INTENT
  -> UNDERSTAND
  -> BOUND
  -> SPECIFY
  -> DESIGN
  -> IMPLEMENT
  -> VERIFY
  -> QUALITY GATES
  -> RELEASE READY
  -> STOP
```

Fuse stages for small, reversible, unambiguous tasks. The lifecycle is an engineering control model, not mandatory ceremony.

The unit of delivery remains a **small coherent vertical slice**. Iterations group intent; they do not create long-lived branches or all-or-nothing merge gates. Stable-grade compatibility, performance, signing, and release evidence belongs at the maturity stage that actually needs it.

## Current-state orientation

Use `.devland/state.yaml` as the single source of truth for active work, blockers, recent completion, and open decisions.

When orientation is useful, use the compact Feature Compass:

```text
Feature Shape -> Current Position -> Delta -> Next Move
```

Do not duplicate live status into narrative documents.

## Legacy context

`PROJECT.md`, `DOCUMENTATION_PLAN.md`, and large historical plans are not canonical process inputs. Keep historical artifacts only when they preserve useful rationale. New agents must not treat them as blockers or duplicate state from them.

When a historical document becomes misleading and its useful decisions already exist in canonical/current docs, delete it rather than maintaining two truths.

## Skills

Load only skills relevant to the touched boundary. Skills guide execution; they do not authorize new product behavior, architecture boundaries, public contracts, dependencies, or infrastructure.

## Maintenance

Prefer fewer durable documents over duplicated policy. Working software and repository evidence lead; documentation follows observable behavior.
