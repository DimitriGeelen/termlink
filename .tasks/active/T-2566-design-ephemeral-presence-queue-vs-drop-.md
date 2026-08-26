---
id: T-2566
name: "DESIGN: ephemeral-presence queue-vs-drop plus heartbeat exit-after-N-queued policy"
description: >
  Design decision from T-2564/T-2468 verb-1 review

status: captured
workflow_type: build
owner: human
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T11:21:41Z
last_update: 2026-08-09T11:21:41Z
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

# T-2566: DESIGN: ephemeral-presence queue-vs-drop plus heartbeat exit-after-N-queued policy

## Context

Filed from T-2564 (T-2468 verb-1 silent-failure review). T-2564 shipped the
MECHANICAL half — the heartbeat loop now warns loudly when a presence post is queued
instead of delivered. But two DESIGN questions need human judgment before any further
code:

1. **Should ephemeral presence even use the offline queue?** A heartbeat is a
   liveness signal with a natural TTL — queuing a batch of stale heartbeats to replay
   on reconnect arguably delivers no value (by the time they flush, they're old) and
   consumes queue capacity that guaranteed messages need. Options: (a) keep queuing
   (status quo), (b) drop presence posts on hub-unreachable (fire-and-forget), (c)
   coalesce — queue at most the latest presence post per agent.

2. **Should the heartbeat loop exit non-zero after N consecutive queued cycles?**
   Right now it warns but runs forever. A supervisor (systemd, tl-claude launcher)
   could restart / re-arm on a non-zero exit, but exiting also risks flapping. What
   is the right N, and should it be configurable / opt-in?

These interact with the offline-queue design (T-2051) and presence semantics
(T-1841/T-2107 cv_key), so they are a human/design decision, not an autonomous build.

## Acceptance Criteria

### Human
- [ ] Decision recorded on Q1 (queue / drop / coalesce presence on hub-unreachable),
      with rationale tied to the offline-queue capacity + presence-TTL trade-off.
- [ ] Decision recorded on Q2 (loop exit-after-N-queued: yes/no, N value, opt-in?),
      with rationale tied to supervisor restart behaviour vs flap risk.
- [ ] If either decision is "change behaviour", a follow-up build task is filed with
      concrete ACs; if "keep status quo", the T-2564 warn is confirmed as sufficient.

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

## Recommendation

**Recommendation:** GO on Q1 — stop queuing ephemeral presence (drop or
coalesce). NO change on Q2 — the T-2564 warn is sufficient; do not add an
exit-after-N policy yet.

**Rationale:** Q1 has an asymmetry the design discussion did not name. A queued
heartbeat delivers almost nothing on flush, because presence is derived from
RECENCY, not from record count: `listener-heartbeat.sh:23-25` defines LIVE as
"newer than 2×interval" and OFFLINE as "older than 5×interval" — 60s and 150s at
the 30s default. A batch of heartbeats replayed after an outage is a batch of
records that all read OFFLINE except possibly the newest, and the agent's next
live heartbeat arrives within 30s anyway. Meanwhile the cost is real and falls on
someone else: the outbound queue is a single shared per-host FIFO
(`~/.termlink/outbound.sqlite`) with `cap: 1000`, and on QueueFull the R3 loud-fail
(T-2051) rejects everything — including the guaranteed messages the queue exists
to protect. Ephemeral traffic evicting durable traffic from a shared bounded
resource is the trade to fix, and it is a bad one in both directions.

**Evidence:** Measured on this host 2026-08-27 — `channel queue-status --json`
reports `{"cap":1000,"pending":0}`, so nothing is currently backed up. Heartbeat
interval defaults to 30s (`listener-heartbeat.sh:95`, min 5). At 30s, one agent
emits 120 posts/hour, so a single agent alone fills the entire shared 1000-row
cap in ~8.3 hours of hub outage; three agents on one host do it in under three.
`agent-presence` retention here is `{"kind":"messages","value":1000}`, and
CLAUDE.md documents cv_index as last-write-wins per `cv_key` — so even on a
successful flush the replayed batch collapses to one current value. **Not
measured:** any real outage on this host long enough to have hit the cap; the
`pending:0` reading means this is arithmetic on measured constants, not an
observed incident.

**On Q2 specifically.** The mechanism is already half-built and that is the
argument against finishing it: `queued_run` is already computed and incremented
at `listener-heartbeat.sh:221-223`, so adding an exit threshold is a one-line
change on an existing variable. It has not been done because the *value* is
unproven, not because the code is hard. Exiting non-zero only helps if a
supervisor is actually watching — and the failure it would respond to is "the
hub is down", which restarting the heartbeat does not fix. That is the flap the
task itself flags. Q1 is worth doing; Q2 is a change looking for evidence.

**What you are actually deciding.**

| Q1 option | Behaviour | Cost |
|---|---|---|
| (a) keep queuing (status quo) | replay on reconnect | ephemeral traffic can starve guaranteed traffic out of a shared 1000-cap queue |
| (b) drop on unreachable | fire-and-forget | a ~30s window where nothing records the attempt; the T-2564 warn is the only trace |
| (c) coalesce to latest-per-agent | one row per agent survives | needs enqueue-time dedupe in the queue layer — real work, and the surviving row still reads OFFLINE by flush time |

Between (b) and (c) the evidence does not separate them cleanly: (c) is strictly
more code for a record that the staleness math will mostly discard anyway. I
lean (b) and would not argue hard against (c).

**Why I should not decide this.** Q1 changes what "presence" promises across the
whole fleet — whether an agent that was unreachable has a queued record of having
tried, or nothing at all. That is a semantic about the discovery verb, and it
interacts with the offline-queue contract (T-2051) and cv_key presence
(T-1841/T-2107). Q2 is a supervisor-policy question about a supervisor whose
behaviour I have not observed.

**If you say GO on Q1:** the third AC then needs a follow-up build task with the
(b)-or-(c) choice named in its ACs. If you keep the status quo instead, the
honest close is confirming the T-2564 warn as the whole answer — which is a
defensible outcome and should be recorded as one, not left open.

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

### 2026-08-09T11:21:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2566-design-ephemeral-presence-queue-vs-drop-.md
- **Context:** Initial task creation
