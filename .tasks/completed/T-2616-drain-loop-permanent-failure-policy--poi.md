---
id: T-2616
name: "Drain-loop permanent-failure policy — poison-drop-delete and head-of-line wedge"
description: >
  bus_client flush loop: (a) when dead_letter write fails but pop succeeds the poison
  post is DELETEd with only an error log and no durable record (T-2452 head-of-line
  tradeoff); (b) a permanent transport fault (bad addr/TLS) is indistinguishable from
  a transient outage and wedges the whole FIFO at debug! forever. Both are design
  tensions needing a deliberate policy.

status: work-completed
workflow_type: design
owner: agent
horizon: null
tags: []
components: [crates/termlink-session/src/bus_client.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T17:07:11Z
last_update: 2026-09-29T16:23:41Z
date_finished: 2026-09-29T16:23:41Z
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
  - ts: '2026-09-08T21:30:29Z'
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
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-08T21:30:39Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 3
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=3 
      (workflow:design); effort=8 (lines=173,acs=6)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-27T21:34:05Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 3
      effort: 8
    rationale: blast_radius=1 (1-file-ref-derived-T-3189); tier=3 
      (workflow:design); effort=8 (lines=173,acs=6)
    rubric_sha: e4a00f38e801
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
- [x] A deliberate policy is chosen (recorded in Decisions) for: (a) what happens when dead_letter fails at the poison threshold — e.g. leave in pending + surface via a distinct dead-letter-write-failed counter/canary rather than delete; (b) how a permanent transport fault is distinguished from a transient outage and surfaced above `debug!` (e.g. an error-after-K-consecutive-identical-failures signal)
- [x] Neither branch can silently lose a guaranteed post nor silently wedge the FIFO indefinitely (loss/wedge is always surfaced at `warn!`/`error!` or a canary)
- [x] Tests cover the dead_letter-fails ∧ pop-succeeds path and the permanent-transport path (using the queue's existing fault-injection test seams)
- [x] `cargo test -p termlink-session` passes

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

cargo test -q -p termlink-session --lib bus_client
grep -q 'pub dead_letter_failed: u64' crates/termlink-session/src/bus_client.rs
grep -q 'pub const TRANSPORT_STREAK_WARN' crates/termlink-session/src/bus_client.rs

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

**Symptom:** (a) when a poison post's dead-letter move failed but its DELETE succeeded, the guaranteed post vanished from both `pending_posts` and `dead_letters` (logged once at `error!`). (b) A transport fault that never clears (wrong addr, cert) kept the queue from draining forever, logging only at `debug!`.
**Root cause:** (a) T-2452's fallback chose "unwedge the FIFO" over "preserve the post" when the store that preserves posts is broken. (b) The flush pass is stateless across calls, so it could not tell the 1st refused connect from the 10,000th, and it logged every one at the transient level.
**Why structurally allowed:** both paths were loud-ish (T-2452 made (a) an `error!`), so the no-silent-failure reviews read them as handled. Nothing asked whether "logged once" still means "lost". The dead-letter canary (T-2558) counts rows IN `dead_letters` and structurally cannot see a post that never arrived there. No fault-injection test existed for the move-fails path.
**Prevention:** fault-injected test for the move-fails ∧ delete-succeeds window (the mutant restoring the delete is red); `dead_letter_failed` and `transport_fail_streak` in `FlushReport` for any observer; the loud schedule is unit-pinned.

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

### 2026-09-29 — drain-loop permanent-failure policy (T-3211 R6)
- **(a) dead-letter MOVE fails at the poison threshold → keep the post, never delete.** It stays at the head of `pending_posts`, the pass breaks (still no re-POST busy loop, T-2452), `error!` fires on every tick, and a new `FlushReport::dead_letter_failed` counts it. **Trade-off accepted:** the FIFO wedges behind it until the dead-letter store accepts the move. That wedge is loud; the old fallback `pop()` was a silent-to-the-queue permanent loss of exactly the post T-2243 exists to preserve. A move failing while a delete succeeds usually means the dead-letter table or its disk is broken. Posts behind it would hit the same broken store at their own poison threshold anyway.
- **(b) permanent vs transient transport fault → distinguished by duration, surfaced by a streak.** Each pass can only see one error, and "connection refused" looks the same whether it is a blip or a wrong address. So `BusClient` keeps a cross-pass `transport_fail_streak` (reset by ANY hub answer, accept or reject), exposed as `FlushReport::transport_fail_streak`. At `TRANSPORT_STREAK_WARN` = 12 consecutive passes (~1 min at the 5s tick) the log line escalates from `debug!` to `warn!` (with pending depth), repeating every further 12 passes rather than every pass. Nothing is dropped: FIFO is preserved, as the retry-forever design intends.

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

### 2026-09-29 — T-3211 R6 evidence
- `flush_keeps_poison_at_head_when_dead_letter_write_fails`: real fault injection. `DROP TABLE dead_letters` under the live queue makes the INSERT fail while a DELETE would succeed, which is the exact (dead_letter-fails ∧ pop-succeeds) window. After POISON_THRESHOLD+2 passes: dropped_poison 0, dead_letter_failed ≥ 1, queue_size 1. **Mutant** restoring the fallback delete: red.
- `flush_transport_streak_grows_across_passes_and_is_loud_on_schedule`: the streak equals the pass number 1..12 against an unreachable hub, the queue is intact, and the loud schedule fires at 12 and 24, not 11 or 13.
- `cargo test -p termlink-session`: 529 passed, 0 failed. `cargo check --workspace`: rc 0. 0 warnings.
- Tests use real fault injection rather than a pre-existing seam: none existed for dead_letter failure, and the file's own `pop_action` comment says so.

### 2026-08-11T17:07:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2616-drain-loop-permanent-failure-policy--poi.md
- **Context:** Initial task creation

### 2026-09-29T16:20:32Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c96d394f
- **Timestamp:** 2026-09-29T16:23:49Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-09-29T16:23:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
