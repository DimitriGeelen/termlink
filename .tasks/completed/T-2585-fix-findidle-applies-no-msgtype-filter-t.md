---
id: T-2585
name: "fix: find_idle applies no msg_type filter, trusts any agent-presence envelope (disagrees with peers/listeners liveness)"
description: >
  find_idle (both hint and walk paths, lib.rs:583-609 and 685-713) applies NO msg_type filter — it trusts any envelope on agent-presence carrying metadata.agent_id, disagreeing with the listeners/peers path which requires msg_type==heartbeat (agent-listeners.sh jq ~line 85). A later non-heartbeat post to agent-presence from a known agent_id is taken as the last heartbeat, so /peers (heartbeat-only) and /find-idle (any-envelope) disagree about who is LIVE; also a trust-the-topic reliability gap. Fix candidate: add msg_type==heartbeat predicate to both find_idle paths so all three discovery surfaces share one liveness definition. VERIFY heartbeat envelopes carry msg_type==heartbeat (heartbeat_env sets it) and that no legit producer posts liveness via a different msg_type before building. From T-2468 verb-1 hunt.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/lib.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T21:59:09Z
last_update: 2026-08-09T22:24:16Z
date_finished: 2026-08-09T22:24:16Z
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

# T-2585: fix: find_idle applies no msg_type filter, trusts any agent-presence envelope (disagrees with peers/listeners liveness)

## Context

From the T-2468 verb-1 ("discover peers") hunt. `find_idle` (both the walk path
`find_idle_agents` and the cv_index fast path `find_idle_agents_from_hint`) applied
NO `msg_type` filter — it trusted any envelope on `agent-presence` carrying
`metadata.agent_id`, disagreeing with the listeners/peers path
(`agent-listeners.sh` line ~284: `select(.msg_type == "heartbeat" ...)`). A later
non-heartbeat post to `agent-presence` from a known `agent_id` was taken as that
agent's latest heartbeat, so `/peers` (heartbeat-only) and `/find-idle`
(any-envelope) could disagree about who is LIVE — a trust-the-topic reliability gap.

**Verified before building (go decision):** (1) the heartbeat producer
`listener-heartbeat.sh:165` posts `--msg-type heartbeat`, and the test helper
`heartbeat_env` sets `msg_type:"heartbeat"`; (2) the listeners consumer requires
`msg_type == "heartbeat"`; (3) find_idle both paths filtered neither; (4) the ONLY
production writer to `agent-presence` is `listener-heartbeat.sh` (msg_type=heartbeat)
— the MCP `termlink_listener_heartbeat` tool shells out to it, and every other
reference is a test. So no legit producer posts liveness via a different msg_type;
the filter is safe and does not exclude any real heartbeat.

## Acceptance Criteria

### Agent
- [x] Both `find_idle_agents` (walk) and `find_idle_agents_from_hint` (cv_index
      fast path) skip any `agent-presence` envelope whose `msg_type != "heartbeat"`
      — matching the listeners/peers liveness definition. One predicate, both paths.
      (lib.rs ~582 walk, ~690 hint.)
- [x] A load-bearing unit test posts a heartbeat envelope AND a non-heartbeat
      envelope carrying an `agent_id` to `agent-presence`, and asserts find_idle
      counts only the heartbeat — proven to FAIL on temp-revert. Test:
      `find_idle_ignores_non_heartbeat_presence_envelope` (FAILS with both guards
      neutralised; confirmed, asserts walk+hint agree).
- [x] `cargo test -p termlink-bus --lib` passes (no regressions). 110 passed, 0 failed.

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
cargo test -p termlink-bus --lib find_idle_ignores_non_heartbeat_presence_envelope

## RCA

**Symptom:** `/peers` (heartbeat-only) and `/find-idle` (any-envelope) could
disagree about who is LIVE. A non-heartbeat post to `agent-presence` from a known
`agent_id` was taken by find_idle as that agent's latest heartbeat, so an agent
that had gone quiet (last real heartbeat old) but had a recent non-heartbeat post
could show as idle/available to a dispatcher while `/peers` correctly showed it
STALE/OFFLINE.

**Root cause:** find_idle (both `find_idle_agents` walk and
`find_idle_agents_from_hint`) deduped/ranked liveness on `metadata.agent_id` +
`ts_unix_ms` with NO `msg_type` predicate — it trusted the topic. The
listeners/peers path (`agent-listeners.sh`) had always required
`msg_type == "heartbeat"`. Two consumers of the same topic used two different
liveness definitions.

**Why structurally allowed:** the three discovery surfaces (listeners/peers,
find_idle walk, find_idle hint) grew independently; only the shell listeners path
encoded the heartbeat predicate. There was no shared definition and no test
asserting cross-surface agreement on liveness, so the divergence was invisible.
(Distinct from T-2582, which closed an identity fork between the two find_idle
paths; this closes a msg_type/liveness fork between find_idle and the peers path.)

**Prevention:** added the `msg_type != "heartbeat"` guard to BOTH find_idle paths
(one predicate, both paths) + the load-bearing test
`find_idle_ignores_non_heartbeat_presence_envelope`, which posts a heartbeat AND a
non-heartbeat envelope and asserts (a) only the heartbeat counts and (b) walk and
hint agree — failing if either guard is removed. All three discovery surfaces now
share one liveness definition.

<!-- template guidance retained below -->
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

### 2026-08-09T21:59:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2585-fix-findidle-applies-no-msgtype-filter-t.md
- **Context:** Initial task creation

### 2026-08-09T21:59:20Z — status-update [task-update-agent]
- **Change:** status: started-work → captured

### 2026-08-09T22:21:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-fdeb4a68
- **Timestamp:** 2026-08-09T22:24:17Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-09T22:24:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
