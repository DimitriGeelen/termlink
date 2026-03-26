# T-280: Dispatch Readiness — Structural Completion Signaling

**Status:** In progress (horizon: now)

## Problem

TermLink's collect-based dispatch convention (T-257) works in E2E tests but fails in real agent sessions. Observed: orchestrating agents fall back to manual `fw termlink status` polling or ask the human when workers finish — despite the convention existing on paper.

The gap is between convention (documented) and structure (enforced). Workers complete without signaling, orchestrators can't detect completion without polling.

## Research Areas

- Completion signaling: how does a worker announce "I'm done" structurally?
- Collection patterns: how does an orchestrator know all N workers finished?
- Failure detection: how does the system detect a worker that died without signaling?
- Integration with `termlink dispatch` and `termlink collect` commands

## Status

Active inception, investigating structural completion signaling patterns.
