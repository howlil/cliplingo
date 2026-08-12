---
name: tdd
description: Use for every feature/bugfix test strategy and before implementing behavior changes.
---

# TDD Skill

## Loop

1. Write one failing test for the next observable behavior.
2. Run it and verify it fails for the intended reason.
3. Write the minimum implementation to pass.
4. Run the focused test.
5. Refactor without changing behavior.
6. Run the relevant wider suite.

## Test boundaries

- Pure Rust core: unit tests with fakes.
- Windows adapters: targeted integration tests plus safe pure helpers.
- Tauri/Svelte: component/transport behavior only where useful.
- C++ worker: protocol/model lifecycle tests; real-model tests belong in dedicated smoke/benchmark jobs.
- Bugs: add a regression test before/with the fix.

## Test quality

Test behavior, invariants and error cases. Avoid mocking private implementation sequences. A test suite that prevents refactoring is not a success.

## Native exceptions

If a Windows behavior cannot be reproduced in an ordinary unit test, create the smallest deterministic integration/E2E scenario and keep non-native decision logic separately testable.
