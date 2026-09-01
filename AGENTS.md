# ClipLingo Agent Entry Point

ClipLingo uses Devland with a deliberately lean project-local operating layer.

Read in this order:
1. `.devland/project.yaml` — product facts and hard constraints.
2. `.devland/state.yaml` — current work, branch/PR, blocker, next action.
3. Only the relevant `.agents/*` file for the task.

Do not preload the whole `.agents` directory. Repository evidence wins over stale prose; if state and Git disagree, fix the state before expanding work.

## Operating model

Optimize for **small production-shaped slices, fast feedback, and frequent integration**.

```text
problem -> acceptance example -> smallest slice -> implement/test -> focused CI -> merge -> observe -> next slice
```

Rules:
- WIP limit: one active implementation slice per agent.
- Prefer slices that can merge the same day; split anything that grows beyond one reviewable behavior.
- Branches are short-lived. `master` is the integration truth; no iteration/develop branch.
- One task stays in one PR through review and CI fixes.
- TDD is preferred for deterministic behavior, but ceremony is not a goal. Use the cheapest executable evidence that catches the relevant failure.
- Verification is risk-based. Do not make broad compatibility/performance research block an alpha slice unless that property is the slice's purpose.
- YAGNI first. Add abstractions only for an observed duplication, volatile external boundary, or concrete testability need.
- Delete dead/obsolete code when replacement is proven and references/tests show it is unused. Do not keep parallel legacy paths "just in case".
- Release early through explicit maturity channels (`alpha` -> `beta` -> stable) instead of demanding stable-grade evidence from every early slice.

## Project-local components

- System design: `.agents/DESIGN.md`
- Requirement slicing / iteration: `.agents/REQUIREMENTS.md`
- Engineering + code rules: `.agents/RULES.md`
- Code patterns: `.agents/CODE_PATTERNS.md`
- Git/CI/DoD: `.agents/SDLC.md`
- Release strategy: `.agents/RELEASE.md`
- Delivery metrics: `.agents/METRICS.md`
- Task-specific skills: `.agents/skills/*`

Metrics diagnose flow, not people. Primary signals: product cycle time, PR age/size, CI feedback time, rework, change failure, escaped defects, WIP age, and release frequency.

Never claim tests, CI, releases, benchmarks, deployments, or manual acceptance that were not actually observed.
