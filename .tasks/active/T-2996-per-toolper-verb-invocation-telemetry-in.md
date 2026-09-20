---
id: T-2996
name: "Per-tool/per-verb invocation telemetry instrument"
description: >
  S-22/C-45,C-30: no per-tool/per-verb invocation counts exist (incl. kv.* session-daemon
  blind spot); non-use judgements were capped at reading D (UNMEASURED). Instrument
  invocation counts so the next review can judge usage. Evidence: docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md
  C-45, C-30.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [value-review, arc:arc-009]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-19T22:19:23Z
last_update: 2026-09-20T22:49:54Z
date_finished:
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
bvp_scores_proposed:
  - ts: '2026-09-20T08:45:11Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=2 (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-20T22:49:55Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 3
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=3 (body:component-discoverability); 
      D4=3 (body:portability-abstraction); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T08:45:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2996: Per-tool/per-verb invocation telemetry instrument

## Context

C-45 is the value review's highest-value ADD, HIGH across all three runs, and the
prerequisite for C-01/IW-1 (the 28 off-charter tools), C-02, C-30, C-31 and C-38.

**The task description is wrong in a way that matters.** It says "no per-tool/per-verb
invocation counts exist". Hub-side RPC telemetry DOES exist: `rpc_audit::record` is
called for every authenticated dispatch (`termlink-hub/src/server.rs:1610`), which is
where C-34's `channel.create` = 52,695 and C-36's "108 calls/34.2d" came from.

The real gap is one of RESOLUTION, not absence: `rpc_audit` counts **RPC methods**,
while every usage question is about **tools and verbs**. `termlink_agent_top_reacted`,
`termlink_agent_top_repliers` and a plain CLI `channel post` all arrive at the hub as
the same `channel.post`. So the 260-tool surface is invisible not because nothing
counts, but because the counter aggregates above the level the decision needs.

**Design is determined by T-2982, closed immediately before this task.** The repo
contains both patterns for this exact problem:
  - `rpc_audit` serializes its critical section under a Mutex (`AUDIT_WRITE_LOCK`)
    explicitly "so concurrent authenticated dispatches cannot race" — CORRECT.
  - `lib/hook-telemetry.sh::_fw_telemetry_increment` does an unlocked
    read-modify-truncate-write and loses 477-479 of 480 increments under 8 writers
    (measured, T-2982) — WRONG, and currently corrupting `.hook-counter`.
This instrument follows the first. The direction of the error is why that is not a
style preference: these counts are the evidence for DELETING tools, so an undercount
makes a used tool read as unused and get deleted. Silent undercount is the one
failure mode this instrument must not have.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] Gap is demonstrated, not asserted: a recorded check shows `rpc_audit` cannot
      distinguish two named MCP tools from the off-charter set that dispatch to the
      same RPC method (or that resolve entirely client-side and reach the hub not at
      all) — establishing that per-method counts cannot answer the per-tool question
- [ ] An invocation sink lives in `termlink-hub` beside `rpc_audit` (both
      `termlink-mcp` and `termlink-cli` already depend on that crate — no new
      dependency, no T-2069 duplication), recording `{ts, surface, name}` append-only,
      serialized under a mutex, size-bounded with rotation — the `rpc_audit` shape,
      explicitly NOT the unlocked `.hook-counter` shape
- [ ] Every MCP tool call is recorded by TOOL NAME at the dispatch choke point
      (`#[tool_handler]` on `termlink-mcp/src/server.rs:16` replaced by an explicit
      `call_tool` that records then delegates to the generated router)
- [ ] Recording is best-effort and cannot fail a tool call: a sink write error leaves
      the tool's own result unchanged (pinned by a test, not by inspection)
- [ ] A concurrency fixture proves the sink loses no records under >=8 concurrent
      writers — the direct T-2982 regression guard, and the reason this design was
      chosen over the framework's existing counter
- [ ] A reader reports per-tool invocation counts over a window, so the next review
      judges the 260-tool surface on measurement instead of inference
- [ ] `cargo build` and the touched crates' tests are green
- [ ] Coverage scope is stated in the reader's own output and in this task: this
      instruments the MCP TOOL surface. CLI verbs and the session-daemon `kv.*` /
      `session.*` blind spot (C-30/C-31) remain UNMEASURED and are filed as
      follow-ups. A reader must not be able to mistake partial coverage for a clean
      full-surface census (T-2680: a guard reporting green is why nobody looks)

## Verification

# T-2996 (C-45). Safe redirect form only (L-387): never `cmd | grep -q PAT`.
cargo build -p termlink-hub -p termlink-mcp > /tmp/.t2996-build 2>&1
cargo test -p termlink-hub invocation_audit > /tmp/.t2996-unit 2>&1 && grep -q "6 passed" /tmp/.t2996-unit
bash tests/invocation-audit-concurrency-fixtures.sh > /tmp/.t2996-fix 2>&1 && grep -q "ALL ASSERTIONS PASSED" /tmp/.t2996-fix
# The fixture must be able to go RED on the rejected shape, or it proves nothing (T-2982 standard).
grep -q "the defect reproduces" /tmp/.t2996-fix
# list_tools must stay implemented alongside call_tool — the default is an EMPTY list.
cargo test -p termlink-mcp --test mcp_integration test_list_tools > /tmp/.t2996-lt 2>&1 && grep -q "1 passed" /tmp/.t2996-lt
# The instrument must resolve runtime_dir, not hardcode /tmp (T-2729).
grep -q "discovery::runtime_dir" crates/termlink-hub/src/invocation_audit.rs
# The reader must carry its scope disclaimer on every output path (T-2680).
bash scripts/invocation-usage.sh > /tmp/.t2996-rd 2>&1 && grep -q "SCOPE:" /tmp/.t2996-rd

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

### 2026-09-21 — the task description was wrong, and the correction is the task

- **What changed:** The filing says "no per-tool/per-verb invocation counts
  exist". Hub-side RPC telemetry *does* exist (`rpc_audit::record`,
  `server.rs:1610`) — it is the source of C-34's 52,695 and C-36's 108 calls.
  The gap is RESOLUTION, not absence: it counts RPC METHODS, and every usage
  question is about TOOLS. Measured: `top_reacted`, `top_replied` and
  `top_repliers` all dispatch the identical `channel.subscribe`, which is this
  host's largest method at 403,555 dispatches, and they PAGE — so the count is
  not even proportional to tool usage.
- **Plan impact:** The work is not "add telemetry where there is none"; it is
  "record at the one place in the process where tool identity still exists"
  (the MCP dispatch choke point), because every tool body below it dissolves
  its identity into a shared RPC method.
- **Triggered:** Context section rewritten to carry the correction so the next
  reader does not re-derive it.

### 2026-09-21 — T-2982 determined the design, and the fixture proves it

- **What changed:** This repo contains both the right and the wrong pattern for
  this exact problem. Measured side by side under 8 concurrent writers × 60:
  `O_APPEND` line writes 480/480/480 with zero torn lines; the unlocked
  read-modify-TRUNCATE-write used by `lib/hook-telemetry.sh` scored 3, 9, 6.
- **Plan impact:** The sink is append-only by requirement, not by taste. The
  error direction decides it: these counts are the warrant for DELETING tools,
  so an undercount makes a used tool read as zero and get deleted. Also
  corrected a claim in my own module doc — the process-local `Mutex` does NOT
  serialise across processes (MCP/CLI are separate processes); cross-process
  safety comes from `O_APPEND` atomicity. The fixture tests PROCESSES, not
  threads, because the unit test structurally cannot substantiate that claim.
- **Triggered:** `tests/invocation-audit-concurrency-fixtures.sh`, whose leg 2
  runs the rejected shape so the harness can still go red.

### 2026-09-21 — replacing a macro silently dropped half its behaviour

- **What changed:** `#[tool_handler]` generates `call_tool` AND `list_tools`.
  Hand-writing only `call_tool` left `list_tools` on its default, which returns
  an EMPTY list: the server would have advertised ZERO tools while compiling
  cleanly and dispatching correctly. Caught by
  `mcp_integration.rs::test_list_tools` ("missing tool: termlink_ping").
- **Plan impact:** Both methods must move together; recorded in a comment at
  the site because the failure is silent at compile time and total at runtime.
- **Triggered:** Nothing new filed — the existing test was sufficient, which is
  itself the argument for T-2686 having wired the suite into CI.

### 2026-09-21 — shipped is not live

- **What changed:** The reader reports no sink on this host, correctly: the
  running MCP server is an older binary, so nothing records until it is rebuilt
  and restarted. This is the repo's own G-069 condition (T-2480).
- **Plan impact:** This task delivers the instrument, NOT a populated dataset.
  No usage verdict — in particular nothing about C-01's 28 tools — may be drawn
  until the server restarts onto a binary carrying this and time passes.
- **Triggered:** Stated in the Recommendation rather than left implicit.

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
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

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-09-19T22:19:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2996-per-toolper-verb-invocation-telemetry-in.md
- **Context:** Initial task creation

### 2026-09-19T22:35:32Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-009

### 2026-09-20T22:49:54Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
