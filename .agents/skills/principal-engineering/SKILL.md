---
name: principal-engineering
description: Use for architecture, dependency, cross-cutting risk, decomposition, and major technical trade-off decisions.
---

# Principal Engineering Skill

## Objective

Keep ClipLingo simple enough to ship and structured enough to change safely.

## Method

1. Restate the user-visible behavior or measured problem.
2. Identify hard constraints and which ones conflict.
3. Choose the smallest reversible design that satisfies the current iteration.
4. Identify only the boundaries likely to change: OS integration, inference implementation, model source, UI transport.
5. Reject abstractions whose only justification is hypothetical future reuse.
6. Define evidence: test, benchmark, compatibility check, or failure mode.
7. Record expensive-to-reverse decisions; leave cheap decisions in code.

## Review questions

- Is this solving a current problem or a hypothetical one?
- Can one process/module do this safely before adding another?
- Does a new dependency materially reduce risk/complexity?
- Is the abstraction at a real volatility boundary?
- What is the failure domain?
- What happens when the worker dies, model is missing, UI reloads, or Windows API is unsupported?
- Are latency/memory/privacy claims measured?
- Can a future dependency swap happen without rewriting the application workflow?

## Reject

- architecture for architecture's sake
- speculative cross-platform layers before a port exists
- generic repository/service/factory stacks with one implementation and no volatility
- server-style infrastructure for local in-process behavior
- broad refactors unrelated to the task
