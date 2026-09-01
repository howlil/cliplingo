# Delivery Metrics

Use metrics to find queueing, oversized work, and unreliable feedback. Do not score contributors or agents with activity counts.

## Core flow metrics

| Metric | Healthy direction | Investigate when |
|---|---|---|
| Product cycle time | down | a product slice remains open for days without new user value |
| PR age | down | a normal feature PR exceeds ~2 working days |
| PR size | small | one PR contains multiple independently releasable behaviors |
| CI feedback time | down | developers wait long enough to context-switch repeatedly |
| Rework rate | down | repeated fixes come from unclear acceptance or oversized scope |
| Change failure rate | low | merged/released changes repeatedly need urgent correction |
| Flaky-test rate | near zero | reruns are used as a normal path to green |
| WIP age/count | low | several unfinished branches compete for attention |
| Release frequency | up when useful | verified product changes accumulate on branches instead of shipping |

## ClipLingo baseline audit — 2026-08-26

Observed repository signals:
- PR #1: governance bootstrap; merged quickly.
- PR #2: iteration plan; merged quickly.
- PR #4/#5: governance/Devland changes; merged quickly.
- PR #3: the only product implementation PR; opened 2026-08-12, still open/draft on 2026-08-26, with 38 commits.
- `master` contains governance/context only; product code remains on the long-lived feature branch.
- canonical `.devland/state.yaml` on `master` still described the work as `planned` with no branch/PR, despite the implementation PR existing.

Interpretation: the dominant bottleneck is **batch size and merge acceptance**, not coding throughput. Governance work flows rapidly while product value queues behind one large high-risk iteration gate.

## Improvement target

For new work:
- slice one independently reviewable user-visible behavior at a time;
- normal PR target: same day, hard review trigger after 2 working days;
- one active implementation PR per agent;
- CI should return the first actionable failure quickly; expensive native/release checks run only where risk requires them;
- merge finished slices instead of accumulating an entire iteration on a branch;
- track manual/platform evidence as follow-up beta/stable hardening unless it is essential to the current slice's correctness.

Revisit thresholds using observed data; they are guardrails, not performance quotas.
