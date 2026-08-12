# SDLC and Git Workflow

## Delivery model

ClipLingo uses short, testable vertical iterations. Each iteration should produce observable behavior or remove a measured risk.

```text
problem/evidence
  -> smallest design
  -> failing test or reproducible check
  -> minimal implementation
  -> refactor
  -> verification
  -> documentation update
  -> PR
  -> squash merge
```

## Before coding

1. Read `PROJECT.md`, `ITERATION_STATE.md`, `RULES.md`, and the relevant design/skill sections.
2. Define the smallest acceptance behavior.
3. Identify the boundary that can be faked in a test.
4. Confirm the change belongs to the current iteration; otherwise record it, do not implement it opportunistically.
5. Use the existing task branch if the task is already in progress.

## TDD

For behavior:

1. **RED:** add one focused failing test or deterministic reproduction.
2. Run it and confirm failure for the expected reason.
3. **GREEN:** implement only enough to pass.
4. Run focused tests.
5. **REFACTOR:** improve names/structure without changing behavior.
6. Run the broader relevant suite.

Native behavior that cannot be meaningfully unit tested still needs a deterministic integration/E2E reproduction and must keep pure logic separated for unit tests.

## Branch policy

- Default branch: `master`.
- One task = at most one branch.
- Naming: `feat/<topic>`, `fix/<topic>`, `perf/<topic>`, `docs/<topic>`, `chore/<topic>`.
- No permanent `develop` or iteration branches.
- Small task follow-ups stay on the same branch.
- Experimental work must be deleted if abandoned.

## Commit policy

Intermediate TDD commits are permitted when useful. Do not manufacture commits for every tiny edit. Before integration, use a squash merge for normal work so `master` gets one clean commit with a meaningful title.

Suggested squash titles:

```text
feat: capture selected text through UI Automation
fix: preserve clipboard during fallback capture
perf: keep translation worker warm after first request
docs: define release engineering workflow
```

## PR policy

A PR is the review/integration unit. It should contain:

- problem and intended behavior
- important design trade-offs
- tests/verification performed
- performance evidence when relevant
- screenshots for meaningful UI changes
- privacy/security impact when applicable
- docs updated when required

Do not create a second PR for revisions to the same task.

## CI gates

As components appear, CI should grow only with them. The intended gates are:

- formatting/lint: Rust, TypeScript/Svelte, C++
- Rust unit/integration tests
- Svelte component/unit tests
- C++ worker tests
- build on a Windows runner
- focused Windows integration/E2E tests when stable enough
- dependency/security/license checks where practical
- release build smoke test for release-related changes

Do not require expensive model downloads for every unit-test job; use a tiny fixture/fake worker and reserve real-model smoke tests for dedicated jobs.

## Environments/channels

Desktop software has release channels rather than server environments:

### Development

Local debug build, fake/tiny models allowed, unsigned, verbose privacy-safe diagnostic logs. No expectation of upgrade compatibility.

### Staging / prerelease

Git tags such as `v0.2.0-rc.1`. GitHub **prerelease** assets. Uses production-like packaging and updater metadata; signing should be exercised as soon as credentials exist. Package-manager community channels are not updated to prereleases unless a separate prerelease package is intentionally created.

### Stable

Tag such as `v0.2.0` from verified `master`. Signed installer/update artifacts, immutable release assets, checksums, release notes, updater metadata, then downstream package-manager updates.

## Definition of Done

A task is done when:

- acceptance behavior is met
- relevant tests pass
- mandatory CI is green
- no unresolved blocker/review thread remains
- logging/privacy rules are preserved
- performance-sensitive work includes evidence
- docs/state are updated where behavior changed
- branch is squash-merged and removed when possible

Do not delay an already-approved task solely for another confirmation after all required gates are satisfied.
