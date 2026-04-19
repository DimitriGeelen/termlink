---
id: T-1132
name: "fleet doctor reports fleet-wide version diversity (piggyback on query.capabilities) (from T-1071 GO)"
description: >
  From T-1071 inception GO. fleet doctor / fleet status should report fleet-wide version diversity, e.g. 'Versions in fleet: 0.9.815 (1 hub), 0.9.99 (1 hub), 0.9.844 (1 hub)'. Cheap — reuses the query.capabilities ping already in fleet doctor probe path. Lets operators see at a glance whether a fleet is homogenous or skewed before a Tier-B typed RPC fails.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [termlink, fleet-doctor, diagnostics, T-1071]
components: []
related_tasks: []
created: 2026-04-18T23:00:06Z
last_update: 2026-04-19T14:08:31Z
date_finished: null
---

# T-1132: fleet doctor reports fleet-wide version diversity (piggyback on query.capabilities) (from T-1071 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] New hub-level RPC `hub.version` returns `{hub_version, protocol_version}` (no params; no auth scope beyond Observe)
- [x] Hub router dispatches `hub.version` to a handler that reads `env!("CARGO_PKG_VERSION")` + `DATA_PLANE_VERSION`
- [x] Hub unit test: `hub.version` returns both fields and their values match the build
- [x] `fleet doctor` calls `hub.version` after successful connect, captures the returned version, and prints a fleet-wide diversity summary line on human output (e.g. `Versions in fleet: 0.9.169 (2 hubs), 0.9.99 (1 hub)`)
- [x] Hubs that fail connectivity are noted as `unknown` in the diversity summary
- [x] JSON output gains `fleet_versions: {"0.9.169": 2, "0.9.99": 1, "unknown": 1}` (BTreeMap; "unknown" entry only present when at least one hub is unreachable)
- [x] `cargo test -p termlink-hub` passes (199 tests); `cargo check -p termlink` builds

### Scope Fence
**IN:** hub.version RPC, fleet doctor diversity summary. **OUT:** Watchtower UI display, per-session version skew, auto-remediation.

### Decisions

### 2026-04-19 — hub.version vs session query.capabilities
- **Chose:** New hub-level `hub.version` RPC (no auth scope beyond observe).
- **Why:** `query.capabilities` is session-level and assumes at least one registered session at the remote. The hub-level probe must work even on an empty fleet, which is exactly when operators care most about version skew.
- **Rejected:** Piggyback on session.discover + aggregate per-session `protocol_version` (T-1131) — gives wrong granularity (session declared, not hub binary) and doesn't work for empty hubs. The task description said "piggyback on query.capabilities" but the probe path doesn't actually call it today, so a dedicated hub method is cheaper to build right.

### Human
- [ ] [REVIEW] Run `termlink fleet doctor` against your real fleet and confirm the diversity summary matches your expectation
  **Steps:** `termlink fleet doctor`
  **Expected:** at the end, a `Versions in fleet: …` line appears; counts match what you have deployed
  **If not:** note the hub name, its expected version, and what fleet doctor reports; upgrade the lagging hub (T-1134 install.sh)

## Verification

grep -q 'fn handle_hub_version' /opt/termlink/crates/termlink-hub/src/router.rs
grep -q 'hub.version' /opt/termlink/crates/termlink-hub/src/router.rs
grep -q 'fleet_versions\|Versions in fleet' /opt/termlink/crates/termlink-cli/src/commands/remote.rs
bash -c 'cd /opt/termlink && cargo check -p termlink-hub -p termlink-cli 2>&1 | tail -2 | grep -qE "Finished|Compiling"'

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-18T23:00:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1132-fleet-doctor-reports-fleet-wide-version-.md
- **Context:** Initial task creation

### 2026-04-19T14:08:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)
