---
id: T-1714
name: "termlink_fleet_doctor MCP — validate_anchors extension for auto_heal_preview"
description: >
  termlink_fleet_doctor MCP — validate_anchors extension for auto_heal_preview

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
created: 2026-05-19T14:24:36Z
last_update: 2026-05-19T14:29:44Z
date_finished: 2026-05-19T14:29:44Z
---

# T-1714: termlink_fleet_doctor MCP — validate_anchors extension for auto_heal_preview

## Context

Follow-up to T-1713 (`fleet_doctor.auto_heal_preview`). Currently the
preview tells an agent which hubs WOULD heal and with which declared
`bootstrap_from`, but does NOT validate that the anchor is actually
reachable. The agent has to make a second MCP call to
`termlink_fleet_bootstrap_check` and cross-reference manually.

This task adds `validate_anchors: Option<bool>` to `FleetDoctorParams`.
When true AND `auto_heal_preview=true`, the handler subprocess-calls
`termlink fleet bootstrap-check --all --json` ONCE (reusing the
T-1688/T-1689 path) and decorates each `would_fire[i]` with
`anchor_status: "ok" | "fetch-fail" | "invalid-format" | "no-anchor"`
plus an optional `anchor_error` string. Adds an
`anchor_validation_summary: {validated, ok, broken}` block.

Net result: an agent gets a single-call answer to "for the hubs that
would heal, is the anchor I'd use actually working?" — preventing
heal-fires that would error at the anchor-fetch step.

Same parallel-implementation pattern as T-1707..T-1713 (PL-172 recipe).
Pure decoration helper is testable; subprocess wrapper is integration-only.

## Acceptance Criteria

### Agent
- [x] `FleetDoctorParams` gains optional `validate_anchors: Option<bool>` with default false; response shape unchanged when unset
- [x] When `auto_heal_preview=true AND validate_anchors=true`, exactly one `termlink fleet bootstrap-check --all --json` subprocess is invoked
- [x] Each `would_fire[i]` gains `anchor_status` field with one of: `ok` / `no-anchor` / `fetch-fail` / `invalid-format` (mirrors CLI taxonomy from T-1688) — plus defensive `unknown` for missing-profile case
- [x] On anchor errors, `anchor_error: <string>` is also injected per entry
- [x] Response gains `auto_heal_preview.anchor_validation_summary: {validated, ok, broken}` count summary
- [x] When `validate_anchors=true` but `auto_heal_preview=false`, the param is silently ignored (no surprise subprocess call) — anchor validation only makes sense WITHIN preview (gated by `let validate_anchors = auto_heal_preview && ...`)
- [x] Subprocess failures (spawn-fail / timeout / non-JSON) degrade gracefully: response gains `auto_heal_preview.anchor_validation_error: <string>` but the preview itself is still returned
- [x] Pure decoration helper `decorate_preview_with_anchors` testable without subprocess
- [x] ≥5 new unit tests: param deserialization (default + true), decoration of all 4 status branches, missing-anchor-key handling, empty-anchor-map graceful path — **delivered 10 tests** (199 passing total, was 189)
- [x] All existing tests still pass (`cargo test -p termlink-mcp`)

## Verification

cargo build -p termlink-mcp
cargo test -p termlink-mcp --quiet
grep -q "validate_anchors" crates/termlink-mcp/src/tools.rs
grep -q "decorate_preview_with_anchors" crates/termlink-mcp/src/tools.rs
grep -q "anchor_validation_summary" crates/termlink-mcp/src/tools.rs

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

### 2026-05-19T14:55Z — implementation-complete [agent]
- **Changes:**
  - `crates/termlink-mcp/src/tools.rs`: added `validate_anchors: Option<bool>` to `FleetDoctorParams`; added 3 helpers (`parse_bootstrap_check_json`, `decorate_preview_with_anchors`, `fetch_anchor_validation_map`); added `AnchorValidation` type; wired anchor-validation path into `termlink_fleet_doctor` handler — folded into the `auto_heal_preview` block after `would_fire` is built.
  - Output shape: each `would_fire[i]` gains `anchor_status` (`ok` / `no-anchor` / `fetch-fail` / `invalid-format` / `unknown`) and optional `anchor_error`. Preview gains `anchor_validation_summary: {validated, ok, broken}` on success, or `anchor_validation_error` on subprocess failure.
  - Graceful degradation: subprocess spawn-fail / timeout / non-JSON returns the preview unchanged plus an error string — never blocks the response.
  - Silent no-op when caller passes `validate_anchors=true` without `auto_heal_preview=true` (composability rule).
- **Tests:** 10 new tests (199 passing total, was 189). Covers all 4 anchor status branches, missing-from-map defensive path, empty-input, mixed-summary, parse JSON shape, and FleetDoctorParams deserialization.
- **Build:** clean (one pre-existing `unused_assignments` warning at 14549, untouched).
- **Value:** an agent can now make ONE MCP call (`termlink_fleet_doctor` with `include_pin_check + auto_heal_preview + validate_anchors`) to answer the full safety-gate question: "which hubs need heal, would the heal target a working anchor, and which hubs lack an anchor entirely?" — replacing what was previously a 2-call cross-reference.

### 2026-05-19T14:24:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1714-termlinkfleetdoctor-mcp--validateanchors.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-b4b876d5
- **Timestamp:** 2026-05-19T14:30:23Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-19T14:29:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
