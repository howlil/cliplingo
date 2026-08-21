# SDLC and Git Workflow

## Delivery model

ClipLingo uses **fast verified delivery** through short, testable vertical slices. Each task should produce observable behavior, remove a measured risk, or improve the delivery system itself.

```text
problem/evidence
  -> smallest acceptance behavior
  -> RED
  -> GREEN
  -> REFACTOR
  -> focused verification
  -> broader risk-based verification
  -> PR / CI
  -> review/fix on the same branch
  -> squash merge
  -> observe/measure
```

The goal is low cycle time with low rework and low escaped-defect risk, not maximum code or commit throughput.

## Before coding

1. Read `.devland/project.yaml` and `.devland/state.yaml` first.
2. Read only the relevant `.agents` design/rules/skill/plan needed by the current task.
3. Define the smallest acceptance behavior.
4. Identify the boundary that can be faked or deterministically reproduced in a test.
5. Confirm the change belongs to current approved work; otherwise record it rather than implementing it opportunistically.
6. Check whether the task already has a working branch or PR and continue it instead of creating duplicates.
7. Decide verification depth from risk before implementation so high-risk work does not discover required evidence only at merge time.

Do not create a heavyweight plan for a trivial local change. Planning depth should increase with uncertainty, native/platform risk, concurrency, IPC/process lifecycle, privacy/security impact, updater/release impact, or performance-sensitive architecture.

## TDD

For executable behavior:

1. **RED:** add one focused failing test or deterministic reproduction.
2. Run it and confirm failure for the expected missing/incorrect behavior.
3. **GREEN:** implement only enough to pass.
4. Run the smallest relevant focused tests immediately.
5. **REFACTOR:** improve names/structure without broadening behavior.
6. Keep tests green during refactor.
7. Widen verification according to risk.

A bug fix requires the regression test first at the lowest useful level.

Native behavior that cannot be meaningfully unit tested still needs a deterministic integration/E2E reproduction or executable harness. Keep pure decision logic separated so the difficult native boundary does not make the whole system untestable.

Do not weaken or delete a valid test because implementation or CI is inconvenient. Flaky tests must be diagnosed and fixed; repeated reruns until green are not valid verification.

## Feedback-loop strategy

Use nested verification loops:

### Inner loop

Target seconds where practical:

- one focused Rust/TypeScript/Svelte/C++ test or small test subset
- type/lint check for the touched component when useful
- deterministic fake worker/native-boundary fixture

### Integration loop

Run when a coherent behavior slice is green:

- relevant component/module suite
- affected Tauri command/worker/native integration checks
- relevant UI/component tests

### Merge loop

Before merge, run mandatory CI and any additional gates required by risk. Do not run every expensive Windows/model/release check after every tiny edit when a focused deterministic loop provides the same signal.

If feedback is routinely slow, fix the delivery system: split oversized jobs, remove unnecessary work, introduce safe caching where evidence supports it, isolate expensive native/model checks, and remove flaky gates.

## Verification by risk

### Low risk

Examples: docs, metadata, formatting, behavior-preserving local refactor.

Expected evidence:

- smallest relevant static/focused checks
- no artificial E2E merely for ceremony

### Medium risk

Examples: Svelte interaction, Rust application workflow, Tauri command contracts, deterministic worker protocol logic, local state transitions.

Expected evidence:

- RED/GREEN focused tests
- relevant component/module suite
- affected integration boundary when runtime behavior crosses one

### High risk

Examples:

- Windows UI Automation/clipboard/selection behavior
- native COM/Win32 integration
- IPC/process lifecycle and timeouts
- latest-request-wins/concurrency behavior
- privacy/security capability changes
- updater/signing/release behavior
- real model/runtime integration
- performance-sensitive architecture or resource-lifetime change

Expected evidence as relevant:

- RED/GREEN regression/acceptance coverage
- negative/error-path tests
- relevant Windows/native integration or E2E evidence
- privacy/log inspection
- benchmark before/after evidence for performance claims
- release smoke/signing/update evidence for release-path changes
- full mandatory CI

## WIP discipline

- One agent normally owns one coherent implementation task end-to-end before starting unrelated work.
- An iteration may contain several tasks, but iteration grouping never creates a long-lived iteration branch.
- A follow-up found during the task remains on the same branch/PR if it is necessary for the same acceptance behavior.
- Genuinely independent work becomes a separate task, not an excuse to enlarge the current PR indefinitely.
- Prefer finishing, verifying, and merging a small slice over opening several partially complete slices.

## Branch policy

- Default branch: `master`.
- One task = at most one branch.
- Naming: `feat/<topic>`, `fix/<topic>`, `perf/<topic>`, `docs/<topic>`, `chore/<topic>`.
- No permanent `develop` or iteration branches.
- Test failures, CI failures, review corrections, formatting, and small task follow-ups stay on the same branch.
- Experimental work must be deleted if abandoned after preserving anything intentionally valuable.

## Commit policy

Intermediate TDD commits are permitted when they provide useful diagnosis or checkpoints. Do not manufacture commits for every tiny edit. Before integration, use a squash merge for normal work so `master` gets one clean logical task commit.

Commit count is not a productivity metric. A retained working commit should exist because it is useful, not because a process target requires more commits.

Suggested squash titles:

```text
feat: capture selected text through UI Automation
fix: preserve clipboard during fallback capture
perf: keep translation worker warm after first request
docs: define release engineering workflow
```

## PR policy

A PR is the review/integration unit. Keep it small enough that a reviewer can reason about failure modes and rollback.

It should contain:

- problem and intended behavior
- important design trade-offs
- tests/verification actually performed
- performance evidence when relevant
- screenshots for meaningful UI changes
- privacy/security impact when applicable
- docs/state updated when required

Do not create a second PR for revisions to the same task. Push revisions to the existing branch/PR.

## CI gates

As components appear, CI should grow only with them. Intended gates include:

- formatting/lint: Rust, TypeScript/Svelte, C++
- Rust unit/integration tests
- Svelte component/unit tests
- C++ worker tests
- build on a Windows runner
- focused Windows integration/E2E tests when stable enough
- dependency/security/license checks where practical
- release build smoke test for release-related changes

Do not require expensive model downloads for every unit-test job; use a tiny fixture/fake worker and reserve real-model smoke tests for dedicated jobs.

CI design should optimize **actionable feedback time** without reducing coverage of meaningful risks. Parallelize independent jobs where practical and keep expensive/high-variance checks separated when that makes failures easier to diagnose.

## Delivery metrics

Track trends when measurement is available:

- cycle time: task start to merge-ready/merged
- PR lead time: PR open to merge
- CI feedback time: push to actionable result
- change failure rate
- escaped defect rate
- rework rate
- flaky-test rate
- WIP age
- release frequency

Do not use commit count, branch count, PR count, lines changed, or generated code as productivity KPIs. Metrics are diagnostic signals for the delivery system, not targets to game.

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
- TDD/verification evidence exists at the appropriate level for executable behavior
- relevant tests pass
- mandatory CI is green
- no unresolved blocker/review thread remains
- logging/privacy rules are preserved
- performance-sensitive work includes measured evidence
- docs/state are updated where behavior or canonical work state changed
- the task is squash-merged and temporary branch state is removed when tooling permits

Do not delay an already-approved normal-risk task solely for another confirmation after all required gates are satisfied. High-impact destructive, credential/signing, privacy/security, irreversible state, or similarly sensitive operations still require the appropriate explicit approval/review gate.
