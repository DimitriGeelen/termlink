---
id: T-2616
name: "Drain-loop permanent-failure policy — poison-drop-delete and head-of-line wedge"
description: >
  bus_client flush loop: (a) when dead_letter write fails but pop succeeds the poison
  post is DELETEd with only an error log and no durable record (T-2452 head-of-line
  tradeoff); (b) a permanent transport fault (bad addr/TLS) is indistinguishable from
  a transient outage and wedges the whole FIFO at debug! forever. Both are design
  tensions needing a deliberate policy.

status: captured
workflow_type: design
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T17:07:11Z
last_update: '2026-08-18T18:58:39Z'
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
  - ts: '2026-08-18T18:55:35Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 3
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=3 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2616: Drain-loop permanent-failure policy — poison-drop-delete and head-of-line wedge

## Context

The offline-queue drain loop (`crates/termlink-session/src/bus_client.rs`) has two
permanent-failure handling tensions, both currently loud-ish but imperfect:

**(a) Poison-drop-delete (bus_client.rs:322-353).** When a post crosses
`POISON_THRESHOLD` and `dead_letter(id)` (the durable MOVE) FAILS but the fallback
`pop(id)` DELETE then succeeds, the post is permanently removed from BOTH
`pending_posts` AND `dead_letters` — `dropped_poison += 1; continue`. It is logged at
`error!` (T-2452 deliberately made it loud, not silent), and the comment explains the
fallback exists to avoid an unbounded re-POST busy-loop / head-of-line block. So the
loss is LOUD but real: the exact data the T-2243 dead-letter store exists to preserve
is gone in the (dead_letter-fails ∧ pop-succeeds) window.

**(b) Permanent-transport head-of-line wedge (bus_client.rs:375-378).** A transport
`Err` breaks the flush pass at `debug!` with no attempt bump — correct for a transient
outage (retry forever is the point), but a PERMANENT fault (misconfigured addr, TLS/cert
mismatch that always fails) is indistinguishable: the head row never bumps attempts,
never reaches poison, never dead-letters, and the whole FIFO behind it never drains,
signalled only by a `debug!` line + growing `queue-status` depth.

These are the same design question — **how the drain loop should distinguish and handle
a PERMANENT failure vs a transient one without silent loss or a silent wedge** — hence
one task. NOT a clean bug: naive "never drop" (a) reinstates the head-of-line block
T-2452 removed; naive "give up after N" (b) drops legitimately-retryable outage traffic.

## Acceptance Criteria

### Agent
- [ ] A deliberate policy is chosen (recorded in Decisions) for: (a) what happens when dead_letter fails at the poison threshold — e.g. leave in pending + surface via a distinct dead-letter-write-failed counter/canary rather than delete; (b) how a permanent transport fault is distinguished from a transient outage and surfaced above `debug!` (e.g. an error-after-K-consecutive-identical-failures signal)
- [ ] Neither branch can silently lose a guaranteed post nor silently wedge the FIFO indefinitely (loss/wedge is always surfaced at `warn!`/`error!` or a canary)
- [ ] Tests cover the dead_letter-fails ∧ pop-succeeds path and the permanent-transport path (using the queue's existing fault-injection test seams)
- [ ] `cargo test -p termlink-session` passes

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

## RCA

**Symptom:** (a) A guaranteed post can be silently lost (loud-logged but gone) when the
dead-letter write fails at the poison threshold; (b) a permanently-misconfigured hub
addr wedges the entire offline queue with only a `debug!` line.

**Root cause:** The drain loop treats all failures on the "permanent" spectrum with
mechanisms designed for transient ones — a fallback DELETE to avoid head-of-line block,
and infinite retry — neither of which has a durable/loud terminal state for a genuinely
permanent condition.

**Why structurally allowed:** Each individual guard (T-2439 attempt-bump, T-2452
anti-busy-loop, T-2497 pop-guard) correctly fixed its own failure mode, but no single
owner defined the end-to-end permanent-vs-transient policy, so the seams between them
leak.

**Prevention:** A single explicit permanent-failure policy + tests at the seams, plus
a canary on dead-letter-write-failure and on stuck-head-of-line queue depth (extends
the existing queue-depth observability).


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

### 2026-08-11T17:07:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2616-drain-loop-permanent-failure-policy--poi.md
- **Context:** Initial task creation
