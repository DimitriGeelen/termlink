---
id: T-2671
name: "termlink_dispatch event.collect loop Err arm has no backoff — silent CPU busy-spin"
description: >
  termlink_dispatch's event.collect retry loop (tools.rs:13675-13678) does Err(_)
  => continue with no backoff and discards the error entirely; a dead/half-open hub
  errors instantly so the loop pins a CPU core silently for the whole collect_timeout
  window. Same T-2658/T-2670 busy-spin class; the register loop directly above already
  sleeps 250ms.

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
created: 2026-08-12T22:36:19Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-12T22:38:35Z
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
  - ts: '2026-08-18T18:56:56Z'
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
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2671: termlink_dispatch event.collect loop Err arm has no backoff — silent CPU busy-spin

## Context

Found during the T-2468 critical-review mandate (round 16), same busy-spin probe that
produced T-2670. `termlink_dispatch` (mcp/tools.rs:13473) runs a `loop {}` collecting
worker events via `event.collect`. Its call site (tools.rs:13675-13678) is
`Err(_) => continue` — no backoff sleep AND the error is fully discarded. On a live hub
`event.collect` long-polls up to `subscribe_timeout_ms` (500ms), pacing the loop; on a
dead/half-open hub `client::rpc_call` returns Err instantly so the loop busy-spins for the
whole `collect_timeout` window. The register loop directly above (tools.rs:13637) already
sleeps 250ms — the collect loop is the un-swept sibling.

## Acceptance Criteria

### Agent
- [x] `termlink_dispatch`'s `event.collect` `Err` arm sleeps `Duration::from_millis(500)` (matching `subscribe_timeout_ms`) before `continue`, mirroring the sibling register loop (tools.rs:13637) and the T-2658/T-2670 backoff convention.
- [x] A `// T-2671` comment cites the divergence + names the convention so the sleep reads as load-bearing.
- [x] `cargo build -p termlink-mcp` clean.
- [x] Structural check (temp-revert-provable): the `event.collect` `Err` arm in `termlink_dispatch` contains a `tokio::time::sleep(` — removing it makes the check fail, restoring passes.

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
cargo build -p termlink-mcp 2>&1 | tail -1
# The event.collect Err arm must carry a backoff sleep (anchored on the call site)
out=$(grep -A14 '"event.collect", params' crates/termlink-mcp/src/tools.rs 2>&1); echo "$out" | grep -q "tokio::time::sleep"

## RCA

**Symptom:** an MCP `termlink_dispatch` call pins a CPU core at 100% for the whole
`collect_timeout` window when the hub is down/half-open — completely silently (the error is
discarded by `Err(_) => continue`, not even logged).

**Root cause:** the `event.collect` retry loop re-dispatches with no delay on error. Live hub
long-polls (paced); dead hub errors instantly → zero-delay busy-spin.

**Why structurally allowed:** the same "sibling loops diverged, only one got the guard"
pattern as T-2658/T-2670. The register loop two blocks up (tools.rs:13637) sleeps 250ms, but
the collect loop was never given the same backoff. Nothing enforces "an error-arm continue
in a poll loop must back off."

**Prevention:** the fix's structural check (AC #4) proves the sleep present + load-bearing.
The general regression-guard (a static check for error-arm `continue` with no backoff in a
`loop {}`) remains the broader round-16 candidate; this fixes the concrete third site. After
this, the busy-spin probe surface (agent.rs ×2, tools.rs ×1) is tree-clean, making that check
buildable-clean in a future window.

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

### 2026-08-12T22:36:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2671-termlinkdispatch-eventcollect-loop-err-a.md
- **Context:** Initial task creation

### 2026-08-12T22:37:15Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-0c176d1b
- **Timestamp:** 2026-08-12T22:38:54Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-12T22:38:35Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
