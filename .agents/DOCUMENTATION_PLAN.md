# Documentation Plan

Documentation should describe working behavior and durable decisions, not speculative architecture inventories.

## Repository documentation layers

### `README.md`

Audience: users and new contributors.

Keep: product value, supported platform/status, install/run commands once available, screenshots, basic development entry point.

### `AGENTS.md` and `.agents/*`

Audience: coding agents and engineers.

Keep: constraints, architecture, code patterns, workflow, current iteration, skill checklists.

### Future `docs/`

Create only when implementation exists and user-facing/developer detail no longer fits README. Likely future areas:

- architecture/runtime flows
- model-pack format
- Windows compatibility matrix
- contributor setup
- troubleshooting

### ADRs

Create an ADR only for decisions that are expensive to reverse or likely to be challenged later, e.g. changing inference runtime, IPC transport, model-pack trust model, or supported Windows baseline. Do not write ADRs for ordinary library choices or obvious implementation details.

## Documentation update triggers

Update in the same task when changing:

- product scope or supported behavior → `PROJECT.md`
- architecture/boundaries/data flow → `DESIGN.md`
- engineering invariant → `RULES.md`
- recurring implementation convention → `CODE_PATTERNS.md`
- Git/CI/testing/environment policy → `SDLC.md`
- versioning/package/release procedure → `RELEASE.md`
- current capability/next step/blocker → `ITERATION_STATE.md`
- user install/run behavior → root `README.md`

## Documentation quality rules

- Prefer concrete examples and commands over vague statements.
- Mark measured facts separately from targets/hypotheses.
- Delete obsolete instructions instead of appending contradictory notes.
- Do not copy the same rule into many files unless one location is a short index/summary.
- Keep secrets, private keys, tokens, user-selected text, and private model URLs out of docs.
