---
id: T-180
name: "Build fw upstream report command for consumer-to-framework bug reports"
description: >
  Two-path upstream reporting for consumer-to-framework feedback. PRIMARY: TermLink inject-remote — directly inject improvement prompts into framework agent Claude session on another machine (proven in T-184/T-185: 5 prompts, 7.4KB injected). FALLBACK: fw upstream report --title ... --attach-doctor — create a task file with evidence when TermLink not available. Both scenarios need clear docs. Discovered 2026-03-18.
status: started-work
workflow_type: inception
owner: human
horizon: later
tags: [framework, cli, upstream]
components: []
related_tasks: []
created: 2026-03-18T22:25:12Z
last_update: 2026-03-22T17:22:24Z
date_finished: null
---

# T-180: Build fw upstream report command for consumer-to-framework bug reports

## Problem Statement

Consumer projects discovering framework improvements have no standard upstream reporting path. Ad-hoc prompt injection was proven in T-184/T-185 but wasn't repeatable. Need a dual-path approach: TermLink inject-remote (primary, interactive) and `fw upstream report` (fallback, file-based).

**For whom:** Any consumer project agent that finds framework bugs or improvements.
**Why now:** TermLink inject-remote is now a standard CLI command (T-187). The primary path is ready — need to document both paths and propose the fallback to the framework.

## Assumptions

- A-001: TermLink inject-remote is sufficient as primary path (VALIDATED — T-187 implemented, T-184/T-185 proven)
- A-002: A file-based fallback adds value when TermLink is unavailable (VALIDATED — air-gapped, offline, or no hub scenarios)
- A-003: `fw upstream report` belongs in the framework, not in TermLink (VALIDATED — it's a framework workflow, TermLink is just transport)
- A-004: Structured prompt templates improve quality of upstream reports (VALIDATED — T-185 showed structured prompts with governance instructions work well)

## Exploration Plan

1. ~~Design dual-path approach~~ — DONE (see research artifact)
2. ~~Document TermLink primary path~~ — DONE (in research artifact)
3. ~~Design fw upstream fallback~~ — DONE (in research artifact)
4. Present for GO/NO-GO decision
5. If GO: send `fw upstream` proposal to framework agent via TermLink, document both paths

See full research: `docs/reports/T-180-upstream-reporting-design.md`

## Technical Constraints

- Primary path requires: TermLink hub with TCP, shared secret, network connectivity
- Fallback path requires: only the framework CLI (no TermLink, no network)
- `fw upstream report` is a framework enhancement — must be proposed upstream, not built in TermLink
- Prompt templates should be project-agnostic (usable by any consumer project)

## Scope Fence

**IN scope:**
- Design and documentation of dual-path upstream reporting
- Prompt template for TermLink-based improvements
- `fw upstream report` command specification (for framework team)
- Documenting both scenarios with clear instructions

**OUT of scope:**
- Implementing `fw upstream report` in the framework (that's the framework team's job)
- Auto-detection of which path to use
- Batch injection tools
- `fw harvest` integration

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (A-001 through A-004 all validated)
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Primary path (TermLink) is implemented and proven — YES (T-187)
- Fallback path design is clear and implementable — YES
- Both paths documented with instructions — YES (research artifact)

**NO-GO if:**
- TermLink path proves unreliable or too complex — NOT THE CASE
- Framework team rejects upstream reporting concept — UNKNOWN (need to propose)

## Verification

# Research artifact exists
test -f docs/reports/T-180-upstream-reporting-design.md

## Decisions

**Decision**: GO

**Rationale**: Primary path (TermLink inject-remote) implemented and proven. Fallback (fw upstream report) design clear. Both documented in research artifact.

**Date**: 2026-03-19T06:04:09Z
## Decision

**Decision**: GO

**Rationale**: Primary path (TermLink inject-remote) implemented and proven. Fallback (fw upstream report) design clear. Both documented in research artifact.

**Date**: 2026-03-19T06:04:09Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-19T06:02:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-19T06:04:09Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Primary path (TermLink inject-remote) implemented and proven. Fallback (fw upstream report) design clear. Both documented in research artifact.

### 2026-03-22T17:22:24Z — status-update [task-update-agent]
- **Change:** horizon: now → later
