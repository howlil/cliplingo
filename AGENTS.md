# ClipLingo Agent Entry Point

This repository uses Devland.

Canonical project facts and project-specific delivery constraints: `.devland/project.yaml`  
Current work state: `.devland/state.yaml`  
Project architecture: `.agents/DESIGN.md`

Read the current plan or other project-local artifacts only when referenced by `.devland/state.yaml` or required by the task. Apply only the relevant Devland core policies, profiles, and workflow; do not load the entire Devland catalog by default.

Repository source and configuration evidence describe what currently exists. Active approved work artifacts describe what should change. When those differ, treat the difference as planned work or drift and verify before rewriting canonical state.

The existing `.agents/` directory is retained as supporting project-local/legacy context during the Devland migration. Read only the relevant file when a task needs detail that is not yet represented canonically, such as code patterns, release constraints, platform-specific guidance, or a referenced implementation plan.

For universal engineering-process rules such as Git, testing, dependency discipline, verification, security baseline, and documentation discipline, Devland core is the baseline. ClipLingo's deliberate project-specific delivery constraints are recorded under `constraints` in `.devland/project.yaml`; these constraints apply to this repository even when they are stricter or more specific than a generic baseline. `.agents/RULES.md` and `.agents/SDLC.md` explain those constraints in implementation terms.

ClipLingo's project-specific delivery model is **fast verified delivery**: executable behavior uses RED → GREEN → REFACTOR, work stays in small coherent vertical slices with low WIP, same-task CI/review feedback stays on the same branch/PR, and verification expands with risk. Native Windows behavior, IPC/process lifecycle, privacy/security, updater/release, model integration, concurrency, and performance-sensitive changes require broader evidence than low-risk local work.

Delivery metrics diagnose the system rather than score activity. Prefer cycle time, PR lead time, CI feedback time, rework, escaped defects, change failure, flaky-test rate, WIP age, and release frequency. Never optimize commit count, branch count, PR count, or lines changed as productivity targets.

Iteration grouping never changes the Git unit: one logical task uses at most one working branch by default. Do not advance `.devland/state.yaml` merely because maintenance/policy work merged; active work state changes only when the approved work itself changes.

This file is a router, not an independent source of product, stack, architecture, or work-state truth.

Never claim repository, version-control, CI, release, benchmark, or deployment actions that the current runtime did not actually perform or observe.
