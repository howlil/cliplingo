---
name: principal-engineering
description: Use for architecture, dependency, cross-cutting risk, decomposition, and major technical trade-off decisions.
---

# Principal Engineering Skill

## Objective

Keep ClipLingo simple enough to ship and structured enough to change safely without letting engineering preference override product or architecture authority.

`../../AGENT_FLOW.md` governs lifecycle and authority. This skill supports design analysis; it does not authorize material boundary changes.

## Method

1. Restate the user-visible behavior or measured problem.
2. Identify hard constraints and which ones conflict.
3. Inspect existing repository patterns and boundaries before proposing new structure.
4. Choose the smallest reversible design using: reuse -> extend -> small local abstraction -> new component -> architecture change.
5. Identify only the boundaries likely to change: OS integration, inference implementation, model source, UI transport.
6. Reject abstractions whose only justification is hypothetical future reuse.
7. Define evidence: test, benchmark, compatibility check, or failure mode.
8. Record expensive-to-reverse decisions; leave cheap implementation decisions in code.
9. If the preferred solution changes a public contract, security boundary, data/state ownership, infrastructure, or major architecture boundary, stop and request explicit user approval before implementation.

## Review questions

- Is this solving a current accepted problem or a hypothetical one?
- Can the existing pattern/component satisfy it before adding a new abstraction?
- Can one process/module do this safely before adding another?
- Does a new dependency materially reduce risk/complexity?
- Is the abstraction at a real volatility boundary?
- What is the failure domain?
- What happens when the worker dies, model is missing, UI reloads, or Windows API is unsupported?
- Are latency/memory/privacy claims measured?
- Can a future dependency swap happen without rewriting the application workflow?
- Does this decision belong to local implementation authority or user/product/architecture authority?

## Reject

- architecture for architecture's sake
- speculative cross-platform layers before a port exists
- generic repository/service/factory stacks with one implementation and no volatility
- server-style infrastructure for local in-process behavior
- broad refactors unrelated to the task
- silently changing product behavior or acceptance to fit an implementation
- crossing an architecture/security/public-contract boundary without explicit approval
