# ClipLingo Agent Entry Point

This repository uses Devland.

Canonical project facts: `.devland/project.yaml`  
Current work state: `.devland/state.yaml`  
Project architecture: `.agents/DESIGN.md`

Read the current plan or other project-local artifacts only when referenced by `.devland/state.yaml` or required by the task. Apply only the relevant Devland core policies, profiles, and workflow; do not load the entire Devland catalog by default.

Repository source and configuration evidence describe what currently exists. Active approved work artifacts describe what should change. When those differ, treat the difference as planned work or drift and verify before rewriting canonical state.

The existing `.agents/` directory is retained as supporting legacy context during the Devland migration. Read only the relevant file when a task needs detail that is not yet represented canonically, such as code patterns, release constraints, platform-specific guidance, or a referenced implementation plan. Do not treat duplicated universal Git/TDD/engineering prose there as a second canonical policy source.

This file is a router, not an independent source of product, stack, architecture, or work-state truth.

Never claim repository, version-control, CI, release, or deployment actions that the current runtime did not actually perform.
