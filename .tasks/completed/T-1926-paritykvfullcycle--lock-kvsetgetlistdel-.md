---
id: T-1926
name: "parity_kv_full_cycle — lock kv_set/get/list/del MCP/CLI shapes"
description: >
  Add parity_kv_full_cycle test exercising the full kv RPC cycle (set → get → list → del). Locks the shapes against future drift. Uses multi_thread runtime per PL-199 since CLI subprocess calls hit the in-process session over unix socket.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-02T12:59:02Z
last_update: 2026-06-02T12:59:02Z
date_finished: null
---

# T-1926: parity_kv_full_cycle — lock kv_set/get/list/del MCP/CLI shapes

## Context

PL-198's MCP/CLI envelope convergence arc shipped 7 tool fixes (T-1919..T-1925) plus T-1911 closure. Parity harness now has 14 tests covering ~5% of 251 MCP tools. KV is heavily used by callers and was scouted to be shape-equivalent already (MCP and CLI both wrap with `{ok:true, ...result}`). A parity test would lock that against future drift.

PL-199 (T-1911) established that socket-roundtrip parity tests need `flavor = "multi_thread"`; KV calls hit the session over the unix socket so this applies here.

## Acceptance Criteria

### Agent
- [x] `parity_kv_full_cycle` test added to `crates/termlink-mcp/tests/parity.rs` exercising the full kv RPC cycle (set → get → list → del). All four phases compared MCP-side vs CLI-side via the existing `diff_json` helper.
- [x] Test uses `#[tokio::test(flavor = "multi_thread", worker_threads = 2)]` per PL-199 (socket roundtrip needed).
- [x] All assertions: each MCP call returns `{ok:true, ...}`; each CLI call returns `{ok:true, ...}`; the field sets match modulo any documented divergence captured in an ignore list (`ts_ms`, `timestamp`).
- [x] Full parity suite: 15 passed; 0 failed; 0 ignored (was 14 passed).
- [x] No regression of any other parity test.

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

cargo test --release --test parity -p termlink-mcp parity_kv_full_cycle -- --test-threads=1 2>&1 | tail -2 | grep -qE "test result: ok\. 1 passed; 0 failed; 0 ignored"

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

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

### 2026-06-02 — test ordering: MCP-cycle-then-CLI-cycle vs interleaved
- **Chose:** Run the FULL MCP cycle (set→get→list→del) first, then the
  full CLI cycle. The intermediate del wipes the kv store, so the CLI
  set starts from empty and returns `replaced=false` (matching MCP set).
- **Why:** Interleaved (MCP-set then CLI-set against the same key)
  returns `replaced=true` on the second call — a real value divergence
  driven by call order, not by code shape. The cycle-then-cycle layout
  makes both sides see identical input state, so value parity holds
  alongside shape parity. Confirmed via the first failing-then-passing
  iteration: first run hit `replaced: false vs true`, restructured run
  passes cleanly.
- **Rejected:** Strip `replaced` / `deleted` from the diff ignore list —
  would let shape pass when those fields silently change type or get
  removed. Keeping them in the diff catches real regressions.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-02T12:59:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1926-paritykvfullcycle--lock-kvsetgetlistdel-.md
- **Context:** Initial task creation
