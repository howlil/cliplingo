# Engineering Rules

## Goal

Maximize verified product throughput with the least process and architecture necessary for the current maturity stage.

`AGENT_FLOW.md` governs authority, lifecycle, scope, stop conditions, Feature Compass, and retrospective behavior. These rules govern implementation quality inside those boundaries.

## Core rules

- **KISS:** prefer the smallest design that is clear, testable, and reversible.
- **YAGNI:** do not implement future subsystems, generic frameworks, or speculative extensibility.
- **DRY pragmatically:** tolerate small local duplication until a stable repeated concept is visible. Do not invent abstractions to eliminate two similar lines.
- **SOLID selectively:** use boundaries where change or testing pressure exists; do not create interface layers by default.
- **Vertical slices:** one task should produce one observable behavior or remove one concrete risk.
- **Low WIP:** finish and merge before starting unrelated implementation.
- **Trunk-centered:** `master` is the integration truth. No permanent develop/iteration branches.
- **Risk-based verification:** match evidence to blast radius and maturity; do not block alpha work on stable-grade evidence.
- **Minimum change:** modify only what is required for the accepted slice and directly necessary local cleanup.

## Product and architecture authority

The agent may choose implementation details inside approved boundaries. It must not silently change product behavior, scope, acceptance criteria, public contracts, data ownership, security boundaries, or major architecture.

For design choices, prefer:

```text
reuse existing pattern
  -> extend existing component
  -> small local abstraction
  -> new component
  -> architecture change
```

Use the first option that satisfies the requirement cleanly. Architecture changes and other material boundary decisions require explicit user approval before implementation.

## System design

Current intentional boundaries:
- Rust owns application state/workflows.
- Svelte owns presentation.
- Tauri commands remain thin adapters.
- Windows APIs stay in platform-specific modules.
- Real inference, when introduced, stays behind a narrow worker boundary.
- No local HTTP server, database, broker, DI framework, OCR stack, GPU stack, or extra service without a demonstrated requirement.

Prefer direct function/module calls inside one process until isolation is justified by fault containment, language/runtime constraints, security, or measured performance needs.

## Code rules

- Optimize for readability of the current behavior, not architectural impressiveness.
- Prefer pure functions for decisions and state transitions; isolate OS side effects.
- Keep modules cohesive and dependencies one-directional.
- New dependency requires a concrete complexity/risk reduction and must not change a material boundary without approval.
- Bug fixes add the smallest useful regression test.
- Refactor only touched code or code directly blocking the change.
- Comments explain non-obvious why/trade-offs, not obvious syntax.
- Follow repository conventions before introducing a new local pattern.

## Legacy deletion policy

Legacy code is a liability, not an archive.

Delete a path in the same task when:
1. its replacement is active;
2. repository search shows no required callers/config references;
3. relevant tests/build still pass; and
4. rollback can be done with Git.

Do not keep duplicate implementations behind dead flags, commented blocks, `old/`, `backup/`, `v1/`, or unused compatibility adapters without a current supported consumer.

If safety cannot be established from repository evidence, record the candidate and defer deletion instead of guessing.

## Testing

Use the cheapest evidence that detects the likely regression:
- deterministic logic: focused unit/component test;
- boundary interaction: focused integration test;
- native Windows behavior: targeted executable/manual smoke where automation is impractical;
- release path: release-specific smoke only when releasing.

RED -> GREEN -> REFACTOR is preferred for deterministic behavior and is required for executable behavior changes where practical under current project constraints. Do not create low-value tests solely to satisfy ceremony.

Do not weaken valid regression tests to obtain green CI. Treat flakes and slow feedback as delivery defects.

## Security and privacy invariants

These are not maturity-dependent shortcuts:
- never log selected/translated text by default;
- normal translation path remains offline;
- keep Tauri capabilities narrow;
- never commit secrets/signing keys;
- verify downloaded model/update artifacts before use.

A change to these boundaries is a stop condition, not an implementation detail.

## Performance

Measure before optimizing. Optimize user-visible p95 latency and idle resource use when those become actual bottlenecks. Avoid benchmark gates for unrelated alpha slices.

## Decision rule

When two solutions are correct within approved boundaries, choose the one with:
1. fewer moving parts;
2. smaller blast radius;
3. shorter feedback loop;
4. easier deletion/reversion;
5. less code and operational burden.

Complexity needs evidence.
