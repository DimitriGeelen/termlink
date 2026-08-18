---
id: T-2611
name: "clamp event.subscribe timeout_ms and max_events caller params"
description: >
  clamp event.subscribe timeout_ms and max_events caller params

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-session/src/handler.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T16:47:53Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-11T16:51:16Z
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
  - ts: '2026-08-18T18:56:53Z'
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
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2611: clamp event.subscribe timeout_ms and max_events caller params

## Context

`handle_event_subscribe` (crates/termlink-session/src/handler.rs:754,764,803) reads
two caller-supplied numeric params — `timeout_ms` (u64, unwrap_or 5000) and
`max_events` (u64, unwrap_or 100) — from the RPC boundary and applies NEITHER a
clamp nor a ceiling. `timeout_ms` flows into `tokio::time::Instant::now() +
Duration::from_millis(timeout_ms)` (line 803): a caller passing `u64::MAX` triggers
an `Instant + Duration` overflow and **panics the handler task**; a merely-large
value pins the detached long-poll task open indefinitely. `max_events` bounds both
the historical-replay `.take(max_events)` (line 787) and the live-collect loop
(line 809), so a large value lets `collected: Vec<Value>` grow one heap JSON value
per delivered event with no ceiling — an unbounded drain sink on a busy topic.
This is the exact class the sibling executor path already guards: `executor.rs:65`
`MAX_EXEC_TIMEOUT_SECS = 3_600` + `effective_exec_timeout` clamp (T-2530), whose
doc literally cites the "`Instant + Duration` overflow panic" + "clamp every caller
numeric param" convention. Reachable via `event.subscribe` / `kv.watch` /
`remote_call`.

## Acceptance Criteria

### Agent
- [x] A pure `effective_subscribe_timeout_ms(Option<u64>) -> u64` helper clamps the caller timeout to `[0, MAX_SUBSCRIBE_TIMEOUT_MS]` (ceiling = `MAX_EXEC_TIMEOUT_SECS * 1000` = 3_600_000, mirroring the exec ceiling), applying the 5000 default when absent
- [x] A pure `effective_subscribe_max_events(Option<u64>) -> usize` helper clamps to `[0, MAX_SUBSCRIBE_MAX_EVENTS]` (10_000), applying the 100 default when absent
- [x] `handle_event_subscribe` uses both helpers instead of the bare `unwrap_or(...)`
- [x] A load-bearing unit test asserts `effective_subscribe_timeout_ms(Some(u64::MAX))` returns the ceiling AND that `Instant::now() + Duration::from_millis(ceiling)` does not panic; temp-reverting the clamp makes it fail
- [x] A unit test asserts `effective_subscribe_max_events(Some(u64::MAX))` returns 10_000 and the absent/default cases return 5000 / 100
- [x] Full `cargo test -p termlink-session` suite passes

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

cargo test -p termlink-session --lib effective_subscribe 2>&1 | tail -3 | grep -q "test result: ok"
cargo test -p termlink-session 2>&1 | tail -3 | grep -q "test result: ok"

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

**Symptom:** A client (or peer via `remote_call`) issuing `event.subscribe` with
`timeout_ms: 18446744073709551615` (u64::MAX) panics the handler task at
`Instant::now() + Duration::from_millis(timeout_ms)`; a merely-large timeout or
`max_events` pins the detached long-poll open / grows an unbounded `Vec<Value>`.

**Root cause:** Two caller-supplied numeric params reach a time-arithmetic sink and
a collection bound with no clamp — the "unclamped caller-param → allocation/loop
sink" class the T-2527 static check exists to prevent, but that check scopes
`with_capacity`/`Semaphore`/`vec!`/`repeat`, not `Instant + Duration` time math, so
this instance was invisible to it.

**Why structurally allowed:** The sibling exec path was hardened in T-2530
(`MAX_EXEC_TIMEOUT_SECS`) and the MCP wrapper's *doc* even advertises `timeout_ms`
"clamped [100, 30_000]" — but the doc is aspirational: the wrapper passes the value
verbatim and the core handler applies nothing. A documented contract with no
enforcement point is a silent gap; nothing tied the handler to the convention its
own sibling and its own doc string already assumed.

**Prevention:** Pure clamp helpers with the same shape as `effective_exec_timeout`,
plus a load-bearing test that fails if the timeout clamp is removed (asserts
`u64::MAX` → ceiling AND that the ceiling does not overflow `Instant + Duration`).
Follow-up candidate (not this task): extend the T-2527 static check to flag
`Instant/Duration` arithmetic fed a bare caller param.

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

### 2026-08-11T16:47:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2611-clamp-eventsubscribe-timeoutms-and-maxev.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-24d11ef1
- **Timestamp:** 2026-08-11T16:51:23Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 1
     - evidence: `cargo test -p termlink-session --lib effective_subscribe 2>&1 | tail -3 | grep -q "test result: ok"`
  2. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 2
     - evidence: `cargo test -p termlink-session 2>&1 | tail -3 | grep -q "test result: ok"`

### 2026-08-11T16:51:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
