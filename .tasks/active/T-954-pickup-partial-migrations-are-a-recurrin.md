---
id: T-954
name: "Pickup: Partial migrations are a recurring bug class — audit should detect incomplete migrations (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-940. Type: pattern.

status: work-completed
workflow_type: inception
owner: human
horizon: now
tags: [pickup, pattern]
components: []
related_tasks: []
created: 2026-04-12T08:40:31Z
last_update: 2026-04-12T17:16:34Z
date_finished: 2026-04-12T17:16:34Z
---

# T-954: Pickup: Partial migrations are a recurring bug class — audit should detect incomplete migrations (from termlink)

## Problem Statement

`fw upgrade` can partially complete — some files get updated, others retain old versions. This leaves the framework in an inconsistent state where some components expect new APIs/patterns while others have old code. Observed in this session (T-984): 6 local patches reverted, T-978 backup code overwritten. The audit system doesn't detect this class of inconsistency.

Related: T-984 (fw upgrade local patch reversion), T-978 (checksum manifest).

## Assumptions

1. Partial migrations leave detectable signatures (version mismatches between files, missing functions referenced by callers)
2. The `.upstream-checksums` manifest (T-978) could serve as a basis for consistency checking
3. This is a framework-side change (audit.sh), not a termlink Rust change

## Exploration Plan

1. Assess what "incomplete migration" looks like in practice — DONE (T-984 evidence)
2. Determine if `.upstream-checksums` can detect inconsistency — DONE (it can compare current vs expected)
3. Decide: build audit check or subsume into T-984's `.local-patches` manifest

## Technical Constraints

- Framework audit runs as cron job (every 30 min)
- Must be fast (no `sha256sum` of 100+ files per run — cache results)
- Changes go in framework repo (audit.sh), not termlink

## Scope Fence

**IN:** Assessment of feasibility and relationship to T-984
**OUT:** Implementation (belongs in framework repo)

## Acceptance Criteria

### Agent
- [x] Problem statement validated (T-984 session evidence of partial migration)
- [x] Assumptions tested (upstream-checksums provides basis; audit change is framework-side)
- [x] Recommendation written with rationale (DEFER: subsume into T-984)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-954, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

## Go/No-Go Criteria

**GO if:**
- T-984 does NOT include audit detection as part of its build scope
- Partial migration incidents continue after T-984 is deployed

**NO-GO if:**
- T-984's `.local-patches` manifest makes partial migrations detectable by design
- This is a framework-side change better handled in the framework repo

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** DEFER

**Rationale:** This is subsumed by T-984 (fw upgrade local patch reversion). If T-984's `.local-patches` manifest is built, partial migrations become structurally detectable — the manifest tracks which files have local modifications, and `fw upgrade` would preserve them. A separate audit check adds complexity for a problem that T-984 prevents at the source.

Additionally, this is a framework-side change (audit.sh) that belongs in the framework repo, not the termlink consumer project.

**Evidence:**
- T-984 addresses the root cause (silent overwriting) rather than the symptom (detecting inconsistency)
- `.upstream-checksums` already exists (108 entries) as a consistency baseline
- Framework audit runs every 30 min but adding sha256sum for 100+ files is expensive

## Decisions

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: This is subsumed by T-984 (fw upgrade local patch reversion). If T-984's `.local-patches` manifest is built, partial migrations become structurally detectable — th...

**Date**: 2026-04-12T17:16:34Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: DEFER

Rationale: This is subsumed by T-984 (fw upgrade local patch reversion). If T-984's `.local-patches` manifest is built, partial migrations become structurally detectable — th...

**Date**: 2026-04-12T17:16:34Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:16:34Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: DEFER

Rationale: This is subsumed by T-984 (fw upgrade local patch reversion). If T-984's `.local-patches` manifest is built, partial migrations become structurally detectable — th...

### 2026-04-12T17:16:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
