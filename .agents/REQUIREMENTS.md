# Requirement and Iteration Strategy

## Purpose

Turn product intent into the smallest independently verifiable behavior that can reach `master` quickly.

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

If acceptance contains several independent `and`s, split it.

## Requirement template

```text
Problem:
User-visible behavior:
Acceptance example:
Non-goals:
Risk: low | medium | high
Cheapest useful verification:
Rollback/revert:
```

No separate long-form spec is required for a local, reversible task with clear acceptance. Write a deeper plan only when uncertainty or blast radius justifies it.

## Iterations

Iterations are planning labels, not branches or release gates. An iteration may contain multiple independently mergeable slices. Do not wait for every iteration objective before integrating finished slices.

`master` should continuously contain the latest verified product state.

## Scope control

While implementing a slice:
- required to satisfy acceptance -> keep in the same task;
- defect exposed by the change -> fix if necessary for acceptance, otherwise record follow-up;
- cleanup that directly reduces touched-code complexity -> allowed;
- unrelated refactor, speculative abstraction, future feature, compatibility expansion -> defer.

## Acceptance maturity

Use product maturity to choose gates:
- **alpha:** prove the core path works; targeted smoke/manual evidence is enough for platform-specific behavior.
- **beta:** broaden compatibility, reliability, observability, and performance evidence.
- **stable:** require documented supported-platform matrix, release/signing/update checks, and regression confidence.

Do not apply stable-release acceptance to every alpha development slice.
