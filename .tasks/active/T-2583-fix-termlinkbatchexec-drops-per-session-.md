---
id: T-2583
name: "fix: termlink_batch_exec drops per-session truncated flag (T-2578 twin, fleet path)"
description: >
  termlink_batch_exec (tools.rs:14568 Ok(val) per-session rebuild arm) selects only stdout/stderr/exit_code and OMITS truncated, unlike its correct sibling batch_run (tools.rs:14884 which forwards it, T-2537). A capped session returns exit_code:-1,truncated:true but the fleet rollup reports ok:true with partial stdout as complete. Fix: add truncated forwarding line, mirror batch_run / mcp_exec_result_json. From T-2468 MCP-flattening hunt, twin of T-2578.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T21:57:36Z
last_update: 2026-08-09T22:09:47Z
date_finished: null
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
---

# T-2583: fix: termlink_batch_exec drops per-session truncated flag (T-2578 twin, fleet path)

## Context

From the T-2468 MCP-flattening hunt (the class T-2578 + T-2580 fixed). This is the
**T-2578 twin on the fleet path**. `termlink_batch_exec`'s per-session rebuild arm
(`crates/termlink-mcp/src/tools.rs:14567-14575`, the `Ok(val)` arm) selects only
`stdout`/`stderr`/`exit_code` and OMITS `truncated`. The RPC (`command.exec` →
`command.execute` → `ExecResult::to_json`, executor.rs:30-33) always carries
`truncated`, and a cap-hit emits the ambiguous pair `{exit_code:-1, truncated:true}`
(T-2529/T-2537). Proof it's an oversight not a design choice: the direct-executor
sibling `termlink_batch_run` DOES forward it (tools.rs:14884, `"truncated":
result.truncated` with a T-2537 comment). So of three exec surfaces, `termlink_exec`
(fixed T-2578) and `batch_run` carry `truncated`; only RPC-routed `batch_exec` drops
it. Failure: `batch_exec` runs `cat big.log` fleet-wide; a capped session returns
`{exit_code:-1, truncated:true, stdout:<partial>}` but the tool reports
`{ok:true, exit_code:-1, stdout:<partial>}` and the `succeeded`/`failed` rollup
(computed on `ok`) looks clean — partial output read as complete.

**Fix (SMALL, mechanical):** add one line to the `Ok(val)` arm —
`"truncated": val.get("truncated").and_then(|v| v.as_bool()).unwrap_or(false)` —
identical to the `batch_run` sibling / `mcp_exec_result_json` (T-2578). Add a
load-bearing test that the per-session rebuild carries `truncated`.

## Acceptance Criteria

### Agent
- [x] `termlink_batch_exec`'s per-session result rebuild (tools.rs ~14568) includes
      `truncated` forwarded from the RPC result, matching the `batch_run` sibling.
      Done via pure helper `mcp_batch_exec_session_json` (tools.rs ~2978).
- [x] A load-bearing unit test asserts the per-session rebuild carries `truncated`
      (extract a pure helper if needed for testability) — proven to FAIL on
      temp-revert. Test: `batch_exec_session_forwards_truncated` (FAILS when the
      `truncated` line is stripped from the helper; confirmed).
- [x] `cargo test -p termlink-mcp --lib` passes (no regressions). 892 passed, 0 failed.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.
cargo test -p termlink-mcp --lib batch_exec_session_forwards_truncated

## RCA

**Symptom:** `termlink_batch_exec` runs a command fleet-wide; a session whose
output hit the 16 MiB cap returns `{exit_code:-1, truncated:true, stdout:<partial>}`
from the hub, but the MCP tool reports `{ok:true, exit_code:-1, stdout:<partial>}`
with no `truncated` field — the fleet `succeeded`/`failed` rollup looks clean and
the partial output is read as complete.

**Root cause:** the per-session result rebuild arm (`Ok(val)`, tools.rs ~14568)
hand-selected only `stdout`/`stderr`/`exit_code` from the `command.exec` RPC
result and omitted `truncated`. The RPC (`ExecResult::to_json`) always carries the
field; the rebuild dropped it. A classic MCP-flattening omission — a handler
collapses a structured RPC result and silently drops a load-bearing field.

**Why structurally allowed:** the direct-executor sibling `termlink_batch_run`
DOES forward `truncated` (tools.rs ~14884, T-2537), and the single-session
`termlink_exec` was fixed in T-2578 — so two of three exec surfaces carried the
flag and only the RPC-routed batch path dropped it. There was no test pinning the
batch_exec per-session shape, so the divergence was invisible.

**Prevention:** extracted the rebuild into a pure helper
`mcp_batch_exec_session_json` (a single testable choke-point) + the load-bearing
test `batch_exec_session_forwards_truncated`, which FAILS if the `truncated` line
is removed. This closes the last of the three exec surfaces (single/direct-batch/
RPC-batch) — the T-2578 twin on the fleet path. The recurring MCP-flattening
class itself is captured as a learning (found 4× across T-2578/2580/2583/2584).

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

### 2026-08-09T21:57:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2583-fix-termlinkbatchexec-drops-per-session-.md
- **Context:** Initial task creation

### 2026-08-09T21:58:12Z — status-update [task-update-agent]
- **Change:** status: started-work → captured

### 2026-08-09T22:09:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
