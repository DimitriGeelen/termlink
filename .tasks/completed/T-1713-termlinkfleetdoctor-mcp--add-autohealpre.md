---
id: T-1713
name: "termlink_fleet_doctor MCP — add auto_heal_preview (dry-run shape, G-057 punch-list #4)"
description: >
  termlink_fleet_doctor MCP — add auto_heal_preview (dry-run shape, G-057 punch-list #4)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-19T14:09:38Z
last_update: 2026-05-19T14:14:47Z
date_finished: 2026-05-19T14:14:47Z
---

# T-1713: termlink_fleet_doctor MCP — add auto_heal_preview (dry-run shape, G-057 punch-list #4)

## Context

G-057 punch-list #4 (silent-strip parity for `termlink_fleet_doctor`).

CLI `fleet doctor --auto-heal --dry-run` (T-1684) classifies each hub's
current state and prints what `fleet reauth --bootstrap-from auto` WOULD
fire — without spawning any heal subprocesses. Safe by construction:
no live remediation, just a preview.

MCP `termlink_fleet_doctor` currently has no parity for `--auto-heal`
at all. Live `--auto-heal` is too dangerous to expose via one-shot MCP
(agents can't oversee the heal subprocess after-the-fact). But the
**dry-run preview** is the ideal MCP-shaped operation: the agent gets
a structured "these N hubs would heal, these M would skip-no-anchor"
answer in a single call, no side effects.

This task adds `auto_heal_preview: bool` to `FleetDoctorParams`.
When true and after the normal doctor sweep, the response gains an
`auto_heal_preview` field with `would_fire[]`, `skipped_no_anchor[]`,
`no_action[]`, and `total_would_fire` count. Mirrors T-1684 stderr
header semantics as structured JSON.

Same G-057 parallel-implementation pattern as T-1710/T-1711/T-1712:
inline the classification logic (auth_mismatch_class, derive_watch_conn,
plus a new auto-heal classifier) in MCP rather than sharing a libcli
crate — the project doesn't have that abstraction yet.

## Acceptance Criteria

### Agent
- [x] `FleetDoctorParams` gains optional `auto_heal_preview: Option<bool>` field with default false (response shape unchanged when unset)
- [x] When `auto_heal_preview=true`, response gains `auto_heal_preview` object with `would_fire[]`, `skipped_no_anchor[]`, `no_action[]`, `total_would_fire`, and `header` fields
- [x] Cert-drift trigger detected (pin_check.status=="drift") AND auth-mismatch trigger detected (status==error with auth-mismatch error class) — mirrors CLI T-1681 OR-gate
- [x] Gates on declared `bootstrap_from` per profile (read from hubs.toml) — same R2 rule as CLI; profiles without anchors land in `skipped_no_anchor` with a hint
- [x] When `auto_heal_preview=true` AND `include_pin_check=false`, response includes `missing_pin_check_warning: true` (only auth-mismatch path can fire) — mirrors CLI T-1683 stderr info hint
- [x] Pure helpers `auth_mismatch_class_mcp`, `derive_watch_conn_mcp`, `classify_auto_heal_preview` testable without infrastructure
- [x] ≥7 new unit tests covering: param deserialization (default + true), auth-mismatch detection (≥3 cases), watch-conn derivation (≥2 cases), classifier branches (clean / pin-drift-with-anchor / auth-mismatch-with-anchor / both-triggers / no-anchor-skip) — **delivered 19 tests** (189 passing total, was 170)
- [x] All existing tests still pass (`cargo test -p termlink-mcp`)
- [x] G-057 parity convention documented in code comments referencing T-1710/T-1711/T-1712 precedent

## Verification

cargo build -p termlink-mcp
cargo test -p termlink-mcp --quiet
grep -q "auto_heal_preview" crates/termlink-mcp/src/tools.rs
grep -q "classify_auto_heal_preview" crates/termlink-mcp/src/tools.rs

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-19T14:09:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1713-termlinkfleetdoctor-mcp--add-autohealpre.md
- **Context:** Initial task creation

### 2026-05-19T14:30Z — implementation-complete [agent]
- **Changes:**
  - `crates/termlink-mcp/src/tools.rs`: added `auto_heal_preview: Option<bool>` to `FleetDoctorParams`; added 4 helper functions (`auth_mismatch_class_mcp`, `derive_watch_conn_mcp`, `classify_auto_heal_preview`, `read_bootstrap_from_map`); added 2 types (`AutoHealAction`, `AutoHealTrigger`); wired preview emission at end of `termlink_fleet_doctor` handler.
  - Output shape: `auto_heal_preview` object with `would_fire[]`, `skipped_no_anchor[]`, `no_action[]`, `total_would_fire`, `missing_pin_check_warning`, `header`.
  - Each `would_fire[i]` carries the exact CLI invocation (`termlink fleet reauth <hub> --bootstrap-from auto`) so the agent can act on the preview directly.
- **Tests:** 19 new tests, suite 170 → 189 passing, 0 failures.
- **Build:** clean (one pre-existing `unused_assignments` warning at 14351, untouched).
- **G-057 parity status:** punch-list items #1 (T-1710 fleet_history analyze), #2 (T-1711 fleet_status verbose), #3 (T-1712 doctor strict), #4 (this task, fleet_doctor auto_heal_preview) all shipped. Live `--auto-heal` deliberately NOT exposed via MCP — agents can preview but cannot fire from a one-shot RPC.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-ed200679
- **Timestamp:** 2026-05-19T14:15:31Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-19T14:14:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
