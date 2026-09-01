# Requirement and Iteration Strategy

## Purpose

Turn user/product intent into the smallest independently verifiable behavior that can reach `master` quickly without allowing the implementation agent to redefine product scope.

`AGENT_FLOW.md` defines authority and lifecycle. This file defines how to slice accepted intent into executable work.

## Product authority

The user owns:
- product behavior and scope;
- acceptance criteria;
- public contracts;
- architecture boundaries;
- data ownership and security boundaries;
- material technical decisions.

The agent may clarify ambiguity, surface trade-offs, and propose acceptance criteria, but must not silently add features, narrow intended behavior, or change a material boundary to simplify implementation.

## Slice rule

A slice must have one observable outcome, one primary failure mode, and a clear rollback/revert path. Prefer a slice that can be implemented, reviewed, and merged in one working day.

Good:
- hotkey opens popup with fake text
- capture selected text from one supported path
- preserve clipboard on fallback
- reject stale translation result

Too large:
- "complete Windows interaction"
- "support all representative Windows apps + DPI + latency + memory + privacy"
- "finish translation architecture"

If acceptance contains several independently useful `and`s, split it while preserving the original product intent.

## Requirement template

```text
Problem:
User-visible behavior:
Acceptance example/criteria:
Non-goals:
Risk: low | medium | high
Cheapest useful verification:
Rollback/revert:
```

No separate long-form spec is required for a local, reversible task with clear acceptance. Write a deeper plan only when ambiguity, uncertainty, or blast radius justifies it.

## Bounding rule

Before implementation, identify:

- what is in scope;
- explicit non-goals;
- touched architecture/platform boundaries;
- material risks;
- decisions that require user authority.

Missing details that only affect local implementation may be resolved by the agent using existing repository conventions and the lowest-blast-radius option.

Missing details that change observable behavior, public contracts, architecture, data/state ownership, consistency, permissions, security, or infrastructure must be surfaced rather than guessed.

## Iterations

Iterations are planning labels, not branches or release gates. An iteration may contain multiple independently mergeable slices. Do not wait for every iteration objective before integrating finished slices.

`master` should continuously contain the latest verified product state.

Live iteration/work status belongs in `.devland/state.yaml`; do not duplicate it in narrative state documents.

## Scope control

While implementing a slice:
- required to satisfy acceptance -> keep in the same task;
- defect exposed by the change -> fix if necessary for acceptance, otherwise record follow-up;
- cleanup that directly reduces touched-code complexity -> allowed;
- unrelated refactor, speculative abstraction, future feature, compatibility expansion -> defer;
- architecture/public-contract/security/infrastructure change -> stop and surface the decision.

Do not expand scope merely because a broader design would be cleaner in theory.

## Acceptance maturity

Use product maturity to choose gates:
- **alpha:** prove the core path works; targeted smoke/manual evidence is enough for platform-specific behavior.
- **beta:** broaden compatibility, reliability, observability, and performance evidence.
- **stable:** require documented supported-platform matrix, release/signing/update checks, and regression confidence.

Do not apply stable-release acceptance to every alpha development slice.
