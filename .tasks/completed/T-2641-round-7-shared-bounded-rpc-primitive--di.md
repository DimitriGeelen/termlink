---
id: T-2641
name: "Round-7 shared bounded-RPC primitive + divergence batch close"
description: >
  Tracker for round-7 of the T-2468 divergence-class sweep: built the shared rpc_call_addr_with_timeout
  bounded-RPC primitive and batch-closed the three unbounded-RPC consumers (T-2639
  unix branch, T-2640 dispatch collect, T-2635 BusClient flush).

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/dispatch.rs, 
      crates/termlink-session/src/bus_client.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T14:27:48Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T14:30:40Z
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
  - ts: '2026-08-18T18:56:55Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 4
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=4 
      (body:framework-level-ux); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2641: Round-7 shared bounded-RPC primitive + divergence batch close

## Context

Round-7 of the T-2468 "subtract-and-deepen" divergence-class sweep (successor to
the round-6 tracker T-2637). The round-6 hunt found that a bounded/hardened
primitive repeatedly landed on ONE caller while sibling callers doing the same
operation kept the old unbounded shape (the "divergence class"). Round-7 closed
that class for the unbounded-RPC-await instances as a batch:

1. **T-2641/T-2639** — built the shared `rpc_call_addr_with_timeout` in
   `termlink-session/src/client.rs` (bounds connect via `connect_addr_with_timeout`
   + read via `call_with_timeout`), the missing bounded twin of `rpc_call_addr`.
2. **T-2639** — channel.rs `rpc_call_authed` unix branch (was unbounded while the
   TCP branch bounded) — routed through the primitive; extracted shared
   `rpc_read_timeout()` so both branches use one convention.
3. **T-2640** — dispatch `event.collect` loop (unbounded await defeated `--timeout`
   on a half-open hub + zero-delay Err micro-spin) — bounded via `collect_call_bound`
   + `COLLECT_ERR_BACKOFF`.
4. **T-2635** — BusClient `post`/`flush` (unbounded → wedged flush task never
   drained the offline queue) — bounded + inner `select!` so shutdown interrupts a
   stuck flush.

All four proven load-bearing via temp-revert. Commits: 5806c5fc8, 833fe9776,
d3b77ecc4.

### Round-8 next lenses (un-swept — for the next fresh-budget window)
- **Directive #3 Usability** — STILL un-swept across all seven rounds (actionable
  errors / sensible defaults / copy-pasteable remediation). Highest-priority next
  sweep.
- **Session-control verbs' PTY/tmux/signal semantics** — the T-2612–2616 PTY
  cluster is filed; the tmux/signal (non-path, non-RPC) surface is unswept.
- **Other divergence instances** — the shared-primitive pattern (a bounded/paced
  primitive whose siblings were never migrated) may recur beyond RPC-await; a
  fresh hunter pass keyed on "hardened primitive + un-migrated sibling caller"
  across other operation classes (allocation, drain, retry) is warranted.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The shared `rpc_call_addr_with_timeout` bounded-RPC primitive exists in `termlink-session/src/client.rs` (bounds connect + read), with load-bearing black-hole tests. **Done in T-2639** (commit 5806c5fc8).
- [x] All three unbounded-RPC divergence consumers route through the shared primitive: T-2639 (channel.rs `rpc_call_authed` unix branch), T-2640 (dispatch `event.collect` loop), T-2635 (BusClient `post`/`flush`). Each with its own load-bearing test proven via temp-revert. **Done** (commits 5806c5fc8, 833fe9776, d3b77ecc4).
- [x] `cargo build` (workspace) succeeds; `cargo test -p termlink-session` (427) + the three per-consumer test filters green.

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

cargo test -p termlink-session --lib client::tests::rpc_call_addr_with_timeout
cargo build -p termlink

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

### 2026-08-12T14:27:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2641-round-7-shared-bounded-rpc-primitive--di.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-dab1e84c
- **Timestamp:** 2026-08-12T14:31:20Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — The shared `rpc_call_addr_with_timeout` bounded-RPC primitive exists in `termlink-session/src/client.rs` (bounds connect + read), with load-bearing black-hole tests. **Done in T-2639** (commit 5806c5f
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink-session/src/client.rs in: The shared `rpc_call_addr_with_timeout` bounded-RPC primitive exists in `termlink-session/src/client.rs` (bounds connect + read), with load-bearing bl`

### 2026-08-12T14:30:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
