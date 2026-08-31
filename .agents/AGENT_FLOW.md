# SWE Agent Flow

## Purpose

Define how an AI engineering agent operates inside ClipLingo without adding process ceremony or changing product/architecture authority.

This file controls **agent behavior**. Current project truth and work state remain in `.devland/project.yaml` and `.devland/state.yaml`.

## Precedence

When guidance conflicts, use this order:

1. the user's current explicit intent and approved decisions;
2. canonical current project/state in `.devland/project.yaml` and `.devland/state.yaml`;
3. this operating flow;
4. project-specific engineering docs in `.agents`;
5. task-specific skills relevant to the touched boundary;
6. historical/legacy documents.

Do not silently reinterpret an explicit user decision through an older document.

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

The lifecycle is canonical, not ceremonial. Fuse stages for small, reversible, unambiguous work. Expand them only when uncertainty, blast radius, or risk requires it.

## Authority model

### User owns

- WHY and WHAT;
- product behavior and scope;
- architecture boundaries;
- acceptance criteria;
- public contracts;
- data ownership;
- security boundaries;
- material technical decisions that are expensive to reverse or alter system boundaries.

### Agent owns

Within those approved boundaries, the agent owns:

- repository inspection;
- implementation design;
- coding;
- testing and debugging;
- verification;
- implementation-level decisions;
- local refactoring required by the change;
- quality gates;
- evidence collection and concise reporting.

High autonomy for local engineering. Low autonomy for product or architecture changes.

## Stage behavior

### UNDERSTAND

Inspect current code, docs, state, tests, and relevant history before proposing new structure. Restate only the material problem, constraints, and observable outcome.

### BOUND

Define the smallest coherent vertical slice.

Explicitly separate:

- in scope;
- non-goals;
- touched boundaries;
- material risks;
- decisions that require user authority.

Do not expand scope for generic best practices.

### SPECIFY

For ordinary work, specify only what is needed to remove ambiguity:

- problem;
- user-visible behavior;
- acceptance example/criteria;
- non-goals;
- risk;
- cheapest useful verification;
- rollback/revert path.

A separate long-form spec is not required for a local, reversible task with clear acceptance.

### DESIGN

Choose the lowest-blast-radius design in this order:

```text
reuse existing pattern
  -> extend existing component
  -> small local abstraction
  -> new component
  -> architecture change
```

Prefer the first option that satisfies the requirement cleanly.

A material change to architecture boundaries, public contracts, data ownership, consistency model, security boundary, or infrastructure requires explicit user approval before implementation.

### IMPLEMENT

Implement the smallest correct change. Follow repository conventions and `.agents/RULES.md`, `.agents/CODE_PATTERNS.md`, and boundary-specific skills.

Executable behavior changes use RED -> GREEN -> REFACTOR where deterministic automation is practical. Refactor only what the change directly requires.

### VERIFY

Verify observable behavior and likely failure modes using the cheapest evidence with sufficient signal. Match verification depth to risk and maturity.

Do not substitute code inspection for required native/manual evidence, and do not turn alpha verification into stable-grade certification.

### QUALITY GATES

Run only gates relevant to the touched risk, plus mandatory repository gates. A valid failing regression test is a defect signal, not a reason to weaken the test.

### RELEASE READY

A change is release ready when acceptance is satisfied, required evidence exists, relevant quality gates are green, rollback is understood, and no known blocker prevents the intended maturity-level use.

Release ready does not authorize unrelated hardening or follow-up scope.

### STOP

Stop when the requested slice is release ready or when a stop condition is reached. Do not continue into speculative cleanup, the next feature, or architecture work without new user intent.

## Minimum-change rule

Make the smallest coherent change that fully satisfies acceptance.

Allowed:

- required implementation;
- regression tests for changed behavior;
- local cleanup directly required for clarity/correctness;
- deletion of replaced dead code when repository evidence proves it safe.

Defer:

- unrelated refactors;
- renames/reorganizations not required by the slice;
- speculative abstractions or future-proofing;
- unrelated dependency upgrades;
- compatibility expansion not required at the current maturity stage;
- behavior changes outside acceptance.

## Stop conditions

Stop implementation and surface the decision when work would require:

- contradictory requirements;
- destructive or irreversible migration;
- public contract change;
- security boundary change;
- major architecture/boundary change;
- unclear ownership of data/state whose choice materially changes behavior.

Do not invent a decision merely to keep moving.

## Feature Compass

Use `.devland/state.yaml` as the source for current position. When orientation is useful, report only:

```text
Feature Shape -> Current Position -> Delta -> Next Move
```

It should answer:

- what the feature should look/behave like;
- what changes from current behavior;
- where the active work is in the lifecycle;
- what is already done;
- what is blocked/in progress;
- the single next meaningful action.

Do not reproduce the full plan or specification unless requested.

## Retrospective

Run a retrospective only when an iteration/release finishes, meaningful rework/failure occurs, repeated friction appears, or the user requests one.

Use:

```text
Evidence -> Bottleneck -> Root Cause -> Small Improvement -> Verify
```

Base conclusions on repository and delivery evidence: requirement churn, diff/PR size, review cycles, CI/test failures, build/deploy failures, manual blockers, repeated debugging, rework, waiting time, duplicated work, unnecessary abstraction/dependencies, escaped defects, and user corrections.

Choose the smallest process or engineering improvement that addresses the dominant bottleneck. Do not create a retrospective ceremony or large improvement backlog.

## ClipLingo-specific constraints

This flow does not replace project architecture. Preserve the established boundaries unless the user explicitly changes them:

- Rust owns application state/workflows;
- Svelte owns presentation;
- Tauri commands remain thin adapters;
- Windows APIs stay in platform-specific modules;
- inference stays behind the isolated worker boundary when introduced;
- normal translation remains offline;
- selected/translated text must not appear in normal logs or telemetry.

Use `.agents/DESIGN.md` for architecture details and `.agents/SDLC.md` for Git/CI mechanics.
