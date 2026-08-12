---
name: backend-core
description: Use for application workflows, ports/adapters, configuration, model registry, worker client, and non-UI product logic.
---

# Backend/Core Engineering Skill

ClipLingo has an application core, not a web backend. Do not introduce an HTTP service merely because the work is "backend".

## Responsibilities

- application use cases
- request/state machines
- language routing
- model metadata and lifecycle orchestration
- worker supervision/client
- configuration
- privacy-safe diagnostics

## Patterns

- Cohesive use case first (`TranslateSelection`), not chains of tiny service classes.
- Ports/adapters only at external or replacement-worthy boundaries.
- Concrete internal modules by default.
- Structured errors at domain boundaries.
- Latest-request-wins for interactive translation.
- Timeouts around operations that can hang the UX.
- Version configuration and IPC/model schemas from the start if persisted/cross-process.

## Avoid

- controller/service/repository layers copied from server frameworks
- DI container
- database for simple settings
- local REST API
- event bus for ordinary direct calls
- generic plugin system before a second implementation exists
