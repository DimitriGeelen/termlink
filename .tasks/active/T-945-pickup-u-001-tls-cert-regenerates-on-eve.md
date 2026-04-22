---
id: T-945
name: "Pickup: U-001: TLS cert regenerates on every hub restart, breaking all client TOFU trust (from 999-Agentic-Engineering-Framework)"
description: >
  Auto-created from pickup envelope. Source: 999-Agentic-Engineering-Framework, task T-1121. Type: bug-report.

status: work-completed
workflow_type: inception
owner: human
horizon: next
tags: [pickup, bug-report]
components: []
related_tasks: []
created: 2026-04-12T08:10:01Z
last_update: 2026-04-22T04:52:52Z
date_finished: 2026-04-12T17:14:58Z
---

# T-945: Pickup: U-001: TLS cert regenerates on every hub restart, breaking all client TOFU trust (from 999-Agentic-Engineering-Framework)

## Problem Statement

TermLink hub generates a new self-signed TLS certificate on every restart (`tls.rs:29-65`), and `tls::cleanup()` deletes cert files on shutdown (`tls.rs:106-109`). Client-side TOFU (`tofu.rs:152-190`) correctly stores fingerprints in `~/.termlink/known_hubs` and rejects changed certs. Combined effect: every hub restart breaks all existing client trust, requiring manual fingerprint acceptance.

T-933 added "persist-if-present" for hub secret (`server.rs:45-71`) but NOT for TLS certs.

## Assumptions

1. The T-933 persist-if-present pattern can be directly applied to TLS certs
2. Cert files (`hub.cert.pem`, `hub.key.pem`) survive hub shutdown if cleanup is changed
3. Self-signed certs don't have expiry concerns for LAN-only use (reasonable default: 1 year)

## Exploration Plan

1. Spike 1: Read tls.rs and server.rs — DONE, clear fix path
2. Spike 2: Verify cert file paths match shutdown cleanup — DONE, `tls::cleanup()` removes them

## Technical Constraints

- Self-signed certs only (no CA infrastructure)
- `rcgen` crate used for generation
- Cert files stored alongside hub socket/secret in runtime dir

## Scope Fence

**IN:** Persist-if-present for certs, don't delete on shutdown, load-or-generate
**OUT:** CA-signed certs, cert rotation, cert distribution to clients

## Acceptance Criteria

### Agent
- [x] Problem statement validated (tls.rs regenerates unconditionally, cleanup deletes on shutdown)
- [x] Assumptions tested (T-933 pattern confirmed applicable; cert files at known paths)
- [x] Recommendation written with rationale (GO: 2-file fix, directly follows T-933 pattern)

### Human
- [ ] [RUBBER-STAMP] Record go/no-go decision
  **Steps:**
  1. Open: http://192.168.10.107:3002/approvals (Inception Decisions section)
  2. Find T-945, select GO / NO-GO / DEFER, click Record Decision
  **Expected:** Decision recorded, task completed

  **Agent evidence (2026-04-15T19:52Z):** `fw inception status` reports decision
  **GO** recorded on 2026-04-12T17:14:58Z. Rationale: Recommendation: GO...
  The inception decision is captured in the task's `## Decisions` section
  and in the Updates log. The Human AC "Record go/no-go decision" is
  literally satisfied — all that remains is ticking the box. Human may
  tick and close.

## Go/No-Go Criteria

**GO if:**
- Fix is a 2-file change following an existing pattern (T-933)
- TOFU breakage on every restart is a real usability problem for cross-host agents

**NO-GO if:**
- TLS is being removed or replaced with a different auth mechanism
- Certs need to be rotated frequently (self-signed LAN certs don't)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

**Recommendation:** GO

**Rationale:** Every hub restart breaks all client TOFU trust, requiring manual fingerprint acceptance. This is a significant usability barrier for cross-host agent communication. The fix directly follows the T-933 "persist-if-present" pattern already implemented for hub secrets — a 2-file change with proven precedent.

**Evidence:**
- `tls.rs:29-65` — always calls `generate_simple_self_signed()`, no load logic
- `tls.rs:106-109` — `cleanup()` deletes cert files on shutdown
- `server.rs:45-71` — T-933 persist-if-present pattern exists for hub secret (proven approach)
- `tofu.rs:152-190` — client TOFU works correctly, will accept persistent certs

**Build scope (if GO):**
1. `tls.rs`: Add `load_existing_cert_and_key()` — load from disk if exists, else generate
2. `tls.rs`: Remove cert deletion from `cleanup()` (preserve across restarts)
3. `server.rs`: Change cert init call to use load-or-generate
4. Unit test: verify cert persistence across simulated restart

## Decisions

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: Every hub restart breaks all client TOFU trust, requiring manual fingerprint acceptance. This is a significant usability barrier for cross-host agent communication. T...

**Date**: 2026-04-12T17:14:58Z
## Decision

**Decision**: GO

**Rationale**: Recommendation: GO

Rationale: Every hub restart breaks all client TOFU trust, requiring manual fingerprint acceptance. This is a significant usability barrier for cross-host agent communication. T...

**Date**: 2026-04-12T17:14:58Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-12T17:14:58Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Recommendation: GO

Rationale: Every hub restart breaks all client TOFU trust, requiring manual fingerprint acceptance. This is a significant usability barrier for cross-host agent communication. T...

### 2026-04-12T17:14:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO

### 2026-04-16T05:39:43Z — status-update [task-update-agent]
- **Change:** horizon: now → later

### 2026-04-16T21:05:40Z — programmatic-evidence [T-1090]
- **Evidence:** hub.cert.pem persisted at /var/lib/termlink/hub.cert.pem; hub.key.pem at runtime dir /tmp/termlink-0/
- **Verified by:** automated command execution

### 2026-04-22T04:52:52Z — status-update [task-update-agent]
- **Change:** horizon: later → next
