---
name: performance
description: Use for latency, memory, CPU, battery/power, worker lifecycle, startup, and benchmark-driven optimization.
---

# Performance Engineering Skill

## Principle

Measure the user path. Do not optimize from intuition.

## Metrics

For relevant changes capture:

- hotkey → popup visible p50/p95
- selection capture latency
- warm/cold inference p50/p95
- worker startup/model-load time
- private working set / commit / peak memory
- idle CPU
- CPU during short translation

Record hardware/OS/model/build mode with results.

## Rules

- Release-mode measurements only for performance conclusions.
- Compare before/after when claiming an optimization.
- Do not use all CPU cores by default for a short interactive request.
- Prefer OS events over polling.
- Avoid loading models at application startup without data.
- Keep popup feedback immediate even when inference is cold.
- Treat WebView residency strategy as an empirical decision.

Reject micro-optimizations that complicate ownership/safety without moving a product metric.
