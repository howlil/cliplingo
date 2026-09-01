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

## Canonical delivery lifecycle

```text
USER INTENT
  -> UNDERSTAND
  -> BOUND
  -> MILESTONE PLAN
  -> EXECUTE SLICES CONTINUOUSLY
  -> MILESTONE GATE
  -> RELEASE READY
  -> STOP
```

Plan at milestone boundaries. Execute continuously at slice boundaries. Integrate at logical-change boundaries.

Increase the planning horizon, not the integration batch size.

The lifecycle is canonical, not ceremonial. A small task may enter an existing milestone directly and use a tight execution loop. Do not create a milestone merely to rename a trivial change.

## Work hierarchy

```text
Milestone
  -> Slice
    -> Logical Change
      -> Commit
```

### Milestone

A bounded, meaningful product, engineering, reliability, migration, or release outcome worth planning as a whole.

A milestone should define:

- why the outcome matters;
- desired end state;
- explicit scope and non-goals;
- major slices or capability increments;
- architecture/product decisions already fixed;
- milestone-level acceptance/gate conditions;
- important risks or unresolved decisions.

Do not turn milestones into long-lived integration branches or all-or-nothing implementation batches.

### Slice

The smallest coherent vertical increment that advances the milestone and can be independently verified and integrated.

A slice should have clear observable acceptance and stay narrow enough to review, test, revert, and merge without waiting for the rest of the milestone.

### Logical change

A focused implementation change inside a slice. Integrate when the change is coherent and independently safe rather than accumulating unrelated edits for the sake of one branch or one large PR.

### Commit

A repository history unit. Commit boundaries support clarity and rollback; they are not planning units or productivity metrics.

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

## Outer lifecycle behavior

### UNDERSTAND

Inspect current code, docs, state, tests, relevant history, and active delivery context before proposing new structure. Restate only the material problem, constraints, and observable outcome.

### BOUND

Decide whether the user intent belongs to an existing milestone or requires a new milestone boundary.

Explicitly separate:

- in scope;
- non-goals;
- touched boundaries;
- material risks;
- decisions that require user authority.

Do not expand scope for generic best practices.

### MILESTONE PLAN

Plan the meaningful outcome once, at the milestone boundary. Keep the plan compact enough to remain useful during execution.

A milestone plan should provide orientation, not task-by-task ceremony. Prefer a small ordered set of slices with explicit gates and dependencies over repeated sprint planning.

Do not pre-plan implementation trivia that can be decided safely inside a slice.

### EXECUTE SLICES CONTINUOUSLY

Execute one bounded slice at a time. Within each slice use the inner engineering loop:

```text
SPECIFY
  -> DESIGN
  -> IMPLEMENT
  -> VERIFY
  -> QUALITY GATES
  -> INTEGRATE
```

Stages may be fused for small, reversible, unambiguous work.

Do not wait for the entire milestone before integration. Merge each release-ready logical change or slice as soon as its required evidence and gates are satisfied.

### MILESTONE GATE

After the planned milestone slices are integrated, verify the milestone-level desired outcome and cross-slice acceptance.

The milestone gate should answer whether the bounded outcome is complete enough for its intended maturity. It must not become an excuse to repeat every slice-level check or add unrequested hardening.

### RELEASE READY

A milestone or release is release ready when its agreed acceptance is satisfied, required evidence exists, relevant gates are green, rollback is understood, and no known blocker prevents intended use.

Release ready does not authorize unrelated hardening or future scope.

### STOP

Stop when the requested milestone/release outcome is ready or when a stop condition is reached. Do not continue into speculative cleanup, the next milestone, or architecture work without new user intent.

## Inner slice engineering loop

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

### INTEGRATE

Integrate at the logical-change boundary when the change is coherent, accepted, verified, and mergeable.

Do not keep work unmerged merely because sibling slices in the milestone are unfinished. Prefer short-lived branches and small PRs. If a change is too coupled to integrate independently, reduce or reshape the slice rather than normalizing a large batch.

## Planning and integration rules

- Plan milestones, not recurring sprints by default.
- A milestone may contain several slices; slices may contain several logical changes.
- Do not create a branch simply because a new slice, iteration label, or planning stage exists.
- One branch/PR should represent one coherent logical change or tightly coupled slice, not an entire milestone by default.
- CI/review fixes for the same logical change stay on the same branch/PR.
- Integrate finished work continuously instead of waiting for milestone completion.
- Keep WIP low; finish or explicitly block the active slice before expanding implementation scope.
- Historical iteration/sprint labels may remain for traceability, but they do not control branch lifetime or integration cadence.

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

- what the milestone/feature should look and behave like;
- what changes from current behavior;
- which milestone and slice are active;
- what is already integrated;
- what is blocked or in progress;
- the single next meaningful action.

Do not reproduce the full milestone plan or specification unless requested.

## Iteration state

Conversation history is context, not the source of truth for active engineering work.

`.devland/state.yaml` remains the canonical current work-state representation for ClipLingo. It should expose enough information to recover the active milestone/slice position, acceptance, evidence, blocker, branch/PR when relevant, and next meaningful action.

Legacy `iteration` identifiers may remain while existing work completes. Treat them as grouping/traceability metadata, not as mandatory planning cycles or branch boundaries.

## Retrospective

Run a retrospective only when a milestone/release finishes, meaningful rework/failure occurs, repeated friction appears, or the user requests one.

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
