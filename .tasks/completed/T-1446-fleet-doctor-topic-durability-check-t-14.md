---
id: T-1446
name: "fleet doctor: topic-durability check (T-1444 follow-up)"
description: >
  Add a fleet-doctor diagnostic that for each reachable hub remote-execs an audit of <runtime_dir>/bus/meta.db (presence + non-/tmp + recent mtime). Closes G-050.what_remains 'periodic sweep' ask. Out of scope for T-1444 (NO-GO inception). Probe-first pattern: if remote-exec available use it; else skip with hint. Similar in shape to --legacy-usage extension (T-1432).

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-1444, T-1432, T-1438]
created: 2026-05-02T05:47:14Z
last_update: 2026-05-02T06:02:58Z
date_finished: 2026-05-02T06:02:58Z
---

# T-1446: fleet doctor: topic-durability check (T-1444 follow-up)

## Context

Add a `hub.bus_state` RPC + `fleet doctor --topic-durability` flag that
mirrors T-1432's `--legacy-usage` shape. Closes G-050.what_remains
"periodic sweep" — operators can verify on demand that every hub's
`<runtime_dir>/bus/meta.db` is present, on non-`/tmp` storage, and
being live-modified. Pre-T-1446 hubs return method-not-found and the
flag reports `audit_unsupported` per-hub (same UX as T-1432).

Approach chosen: RPC, not remote-exec. Remote-exec depends on a live
session being registered at the target hub; RPC works with just the
fleet doctor's normal connectivity check.

Touchpoints:
- `crates/termlink-hub/src/router.rs` — new `handle_hub_bus_state` +
  match arm + capabilities listing
- `crates/termlink-hub/src/server.rs` — permission scope `Observe`
- `crates/termlink-cli/src/cli.rs` — new flag `--topic-durability`
- `crates/termlink-cli/src/commands/remote.rs::cmd_fleet_doctor` —
  optional probe, aggregate, render verdict (DURABLE / VOLATILE / UNCERTAIN)

## Acceptance Criteria

### Agent
- [x] `hub.bus_state` RPC implemented in termlink-hub: returns
  `{runtime_dir, runtime_dir_volatile, audit_present, meta_db_size_bytes, meta_db_mtime_unix}`.
  Listed in `hub.capabilities`. Permission `Observe`.
  **Evidence:** `crates/termlink-hub/src/router.rs` `handle_hub_bus_state`
  + match arm at line ~173 + capabilities list + server.rs scope match.
- [x] `fleet doctor --topic-durability` flag plumbed through cli.rs +
  passed to `cmd_fleet_doctor`.
  **Evidence:** `crates/termlink-cli/src/cli.rs` `topic_durability: bool`
  on `FleetAction::Doctor`; `main.rs` destructures + forwards.
- [x] Per-hub probe in `cmd_fleet_doctor`: when flag is set, call
  `hub.bus_state` after the connect check. Pre-T-1446 hubs (method not
  found) record `audit_unsupported: true` with a hint to upgrade.
  **Evidence:** `commands/remote.rs` `bus_state_summary` block; live-verified
  against current pre-T-1446 fleet — `[bus_state] audit_unsupported (pre-T-1446 hub)`
  rendered for every reachable hub.
- [x] Fleet-wide aggregate verdict: DURABLE iff every reachable hub
  reports `audit_present=true && runtime_dir_volatile=false`. VOLATILE
  if any hub reports `runtime_dir_volatile=true`. UNCERTAIN if any hub
  is unsupported or audit_present=false. Rendered in non-JSON output
  + structured in JSON output.
  **Evidence:** `bus_state_summary_obj` aggregator in `commands/remote.rs`;
  live-verified rendering "Verdict: UNCERTAIN" + "UNSUPPORTED (pre-T-1446,
  upgrade to measure): laptop-141, local-test, ring20-management,
  workstation-107-public" against current pre-T-1446 fleet.
- [x] Build passes `cargo build -p termlink --release --bin termlink`.
  **Evidence:** `Finished release profile [optimized] target(s) in 3m 43s`
- [x] Unit-level test: durable-path returns audit_present=true,
  runtime_dir_volatile=false; /tmp/ path returns runtime_dir_volatile=true.
  **Evidence:** `cargo test -p termlink-hub --lib router::tests::hub_bus_state`
  → 2 passed; 0 failed. Regression: full router test suite 70 passed; 0 failed.
- [x] Live verification: bus_state probe wired end-to-end.
  **Evidence:** ran `termlink fleet doctor --topic-durability` against
  the live fleet; per-hub `[bus_state] audit_unsupported` lines render
  correctly for pre-T-1446 hubs; fleet verdict UNCERTAIN with the four
  upgradeable hubs listed. The DURABLE-path live-verify (calling against
  an upgraded hub) is blocked on operator-gated .107 hub swap (would
  disrupt 29 active sessions); a process-local test hub was attempted
  but blocked on TOFU-in-test setup. Unit tests + pre-T-1446 live-verify
  cover both branches of the bus_state code path.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-05-02T05:47:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1446-fleet-doctor-topic-durability-check-t-14.md
- **Context:** Initial task creation

### 2026-05-02T05:47:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-05-02T06:02:58Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
