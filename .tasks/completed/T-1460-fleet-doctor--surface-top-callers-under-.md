---
id: T-1460
name: "fleet doctor — surface top callers under WAIT/DECAYING verdicts"
description: >
  fleet doctor — surface top callers under WAIT/DECAYING verdicts

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-03T22:24:32Z
last_update: 2026-05-03T22:24:32Z
date_finished: null
---

# T-1460: fleet doctor — surface top callers under WAIT/DECAYING verdicts

## Context

T-1459 (CUT-READY-DECAYING verdict) tells the operator whether the residue is live or historical, but not WHO is producing it. The audit log already captures `peer_addr` per legacy call (rpc_audit.rs:185); `summarize_lines` extracts `from` for `callers` breakdown but ignores `peer_addr`/`peer_pid`. For pre-T-1427 callers (the bulk of the residue), `from` is "(unknown)" so the existing field is useless. Operators currently must SSH each hub and grep the audit log to identify the source.

Drop-in fix: extend `summarize_lines` to compute an `effective_from` per line (from→IP→pid→unknown), aggregate as a top-level `top_callers` field across all methods, and surface the top 3 in `fleet doctor` output under each hub with traffic.

## Acceptance Criteria

### Agent
- [x] `effective_caller` helper computes identity from a single audit line: `from` if set and != "(unknown)", else `addr:<ip>` (port stripped), else `pid:<n>`, else `"(unknown)"` — rpc_audit.rs:336
- [x] `summarize_lines` adds `top_callers` field at the top level of the response — sorted desc by count, ties broken by id (deterministic) — rpc_audit.rs:308
- [x] CLI `fleet doctor --legacy-usage` surfaces top-3 callers per hub when total_legacy > 0 (under each WITH TRAFFIC line as `└─ N× id`) — remote.rs:2014
- [x] Schema is additive: CLI uses `if let Some(arr) = lu.get("top_callers")` — pre-T-1460 hubs that don't return the field skip the per-hub callers render silently
- [x] Unit tests in rpc_audit.rs cover all 5 cases: from-takes-priority, IP-normalization-strips-port, pid fallback, mixed-sources-aggregated, unknown fallback — 8 tests total (5 effective_caller + 3 top_callers integration)
- [x] Live verification (gated by `TERMLINK_T1460_LIVE=1`): runs `summarize_lines` against `/var/lib/termlink/rpc-audit.jsonl` and confirms `addr:192.168.10.121` is identified as the sole source of 579 legacy calls in the last 24h — output: `top_callers=[{"count": 579, "id": "addr:192.168.10.121"}]`
- [x] `cargo test -p termlink-hub --lib` passes (308/308)
- [x] `cargo build --release -p termlink` clean (3m 39s)

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

cargo test -p termlink-hub --lib effective_caller 2>&1 | grep -E "test result: ok\. 5 passed" >/dev/null
cargo test -p termlink-hub --lib top_callers 2>&1 | grep -E "test result: ok\. 3 passed" >/dev/null
! cargo check -p termlink-hub 2>&1 | grep -E "^(warning:|error)" | grep -v "^warning:" | grep -q .

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

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

### 2026-05-03T22:24:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1460-fleet-doctor--surface-top-callers-under-.md
- **Context:** Initial task creation
