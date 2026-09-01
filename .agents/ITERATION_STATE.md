# Iteration State — Legacy Redirect

Live iteration/work state is **not maintained in this file**.

Use:

- `.devland/state.yaml` for active work, lifecycle position, blockers, recently completed work, and open decisions;
- `.devland/project.yaml` for canonical project purpose, constraints, stack, and delivery model;
- `.agents/AGENT_FLOW.md` for Feature Compass and agent operating behavior;
- `.agents/plans/*` for task-specific implementation detail when a plan is justified.

## Feature Compass

When a compact orientation is needed, derive it from `.devland/state.yaml`:

```text
Feature Shape -> Current Position -> Delta -> Next Move
```

Do not copy live status, blocker lists, acceptance criteria, or next-iteration state into this file. Maintaining two representations of current state creates drift and is explicitly avoided.

This path remains only as a compatibility pointer for older references and may be deleted once repository search shows no useful consumers.
