---
id: T-2578
name: "fix: termlink_exec MCP tool drops truncated flag (silent output-truncation,
  T-2537 boundary miss)"
description: >
  T-2578

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T21:25:18Z
last_update: '2026-08-18T18:59:13Z'
date_finished: 2026-08-09T21:31:29Z
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
  - ts: '2026-08-18T18:56:52Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 3
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=3 (body:portability-abstraction); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:13Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2578: fix: termlink_exec MCP tool drops truncated flag (silent output-truncation, T-2537 boundary miss)

## Context

From the T-2468 charter verb-4 ("control terminal sessions") adversarial hunt. The
`termlink_exec` MCP tool (`crates/termlink-mcp/src/tools.rs:11328`) builds its
response with only `ok / exit_code / stdout / stderr / target` — it silently drops
the `truncated` flag that the underlying `command.execute` RPC already carries
(`executor.rs:33` `ExecResult::to_json`, `handler.rs:456-457`). T-2537 established
that `truncated` MUST be emitted at EVERY exec boundary — a cap-hit renders
`exit_code:-1` (indistinguishable from a signal kill) and in the ~64 KiB band around
the 16 MiB cap can even read `exit_code:0`. The sibling `termlink_run` forwards it
via `mcp_run_result_json` (tools.rs:2933) with a load-bearing test
(`run_result_truncated_is_emitted`, tools.rs:30997); `termlink_exec` was simply
missed. Net: an MCP caller running `exec` against a command whose output exceeds the
cap receives truncated stdout with NO signal that data was lost — a silent-failure
Reliability violation on the charter-core output-capture path.

## Acceptance Criteria

### Agent
- [x] `termlink_exec`'s response JSON (tools.rs ~11328) includes a `truncated` field
      forwarded from the RPC result (`result["truncated"].as_bool().unwrap_or(false)`).
- [x] A load-bearing unit test asserts the forwarding, mirroring the
      `run_result_truncated_is_emitted` convention — removing the `truncated` line
      from the `termlink_exec` response makes the test FAIL (proven via temp-revert).
- [x] `cargo test -p termlink-mcp` passes (no regressions).

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
out=$(cargo test -p termlink-mcp --lib exec_result_truncated_is_emitted 2>&1); echo "$out" | grep -q "1 passed"
out=$(grep -n '"truncated": result\["truncated"\]' crates/termlink-mcp/src/tools.rs); echo "$out" | grep -q truncated
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

## RCA

**Symptom:** An MCP `termlink_exec` whose command produces more than the 16 MiB
output cap is silently truncated (child killed, output cut) but the response carries
no `truncated` flag — the caller cannot tell a cap-hit (`exit_code:-1`) from a signal
kill, and in the ~64 KiB band around the cap a truncated result can even read
`exit_code:0`. The caller acts on incomplete output believing it complete.

**Root cause:** `termlink_exec` rebuilds its response inline from the `command.execute`
RPC result, copying only `ok/exit_code/stdout/stderr/target` and dropping the
`truncated` field the RPC already provides (`ExecResult::to_json`, executor.rs:33).

**Why structurally allowed:** T-2537 added `truncated` at the exec boundaries it
enumerated (`termlink_run`, `termlink_batch_run`, `command.execute`) but did NOT
enumerate `termlink_exec` — the field-forwarding was inline (no shared helper), so
there was no single choke-point and no per-tool test asserting the field survives.
The convention existed by discipline, unenforced for this one tool.

**Prevention:** (1) extracted a pure `mcp_exec_result_json` helper so the forwarding
is a single testable choke-point (mirrors `mcp_run_result_json`); (2) added the
load-bearing test `exec_result_truncated_is_emitted` — proven to FAIL when the
`truncated` line is removed (temp-revert verified). Any future rebuild of the exec
response that drops the field now fails a test.

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

### 2026-08-09T21:25:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2578-fix-termlinkexec-mcp-tool-drops-truncate.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-4fc8ad9d
- **Timestamp:** 2026-08-09T21:31:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-09T21:31:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
