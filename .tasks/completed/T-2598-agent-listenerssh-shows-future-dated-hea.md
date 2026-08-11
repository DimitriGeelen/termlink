---
id: T-2598
name: "agent-listeners.sh shows future-dated heartbeat as LIVE forever (missing upper clock bound)"
description: >
  T-2468 verb-1 (discover) adversarial hunt (2026-08-10, CONFIRMED + self-verified). scripts/agent-listeners.sh ~284-291: heartbeats are filtered on msg_type+agent_id, grouped, max_by(.ts) per agent, then age=((now_ms-.ts)/1000)|floor and 'LIVE if age<=2*interval'. A FUTURE-dated ts (clock skew or corrupt final heartbeat) makes now_ms-.ts NEGATIVE -> negative age -> age<=2*interval trivially true -> the agent shows LIVE FOREVER in /peers, /agent-listeners, and (via merge) agent-listeners-fleet.sh. The Rust find_idle path already guards this (T-2536 future_cutoff_ms) and chat-arc-stats drops ts>now_ms (tools.rs:4029 'future-clock safety'), but this shell surface never got the guard -> discovery surfaces DISAGREE on the same agent; orchestrator dispatches work to a corpse /find-idle already retired. Violates 'no silent failures'. Fix: drop future-dated envelopes before group_by (add '(.ts // 0) <= $now_ms' to the heartbeat select) so max_by picks a valid recent envelope, or the agent falls to OFFLINE/absent. Mirrors T-2536. Load-bearing test via TERMLINK_LISTENERS_TEST_JSON seam: a future-dated heartbeat must NOT classify LIVE.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [bug, verb1, silent-failure]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-10T20:21:13Z
last_update: 2026-08-10T20:23:44Z
date_finished: 2026-08-10T20:23:44Z
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

# T-2598: agent-listeners.sh shows future-dated heartbeat as LIVE forever (missing upper clock bound)

## Context

`scripts/agent-listeners.sh` classifies presence via `age = ((now_ms - .ts)/1000)`
then `LIVE if age <= 2*interval`. A future-dated `ts` (clock skew or a corrupt
final heartbeat) yields a NEGATIVE age → trivially `<= 2*interval` → the agent
shows LIVE forever across `/peers`, `/agent-listeners`, and the fleet merge. The
Rust `find_idle` path already guards this (T-2536 future cutoff) and chat-arc
stats drop `ts > now_ms` (tools.rs:4029) — this shell surface never got the
guard, so discovery surfaces disagree and an orchestrator can dispatch to a
corpse `/find-idle` already retired. Fix mirrors T-2536: drop future-dated
envelopes before the group_by/max_by so a future heartbeat cannot win or mark
LIVE.

## Acceptance Criteria

### Agent
- [x] A heartbeat whose `ts` is in the future (now + 1h) does NOT classify as `LIVE` (it is dropped → agent absent, or falls to a real classification from an older valid envelope).
- [x] A normal recent heartbeat (age within `2*interval`) still classifies `LIVE` (fix does not over-drop).
- [x] When an agent has BOTH a future-dated envelope and an older valid recent one, the valid one wins and the agent classifies `LIVE` at the real age (max_by no longer picks the future envelope).
- [x] Load-bearing test (`tests/agent-listeners-liveness-fixtures.sh`) via the `TERMLINK_LISTENERS_TEST_JSON` seam asserts the above and FAILS against the pre-fix script (temp-revert proven).

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
bash tests/agent-listeners-liveness-fixtures.sh
grep -q '(.ts // 0) <= $now_ms' scripts/agent-listeners.sh

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

**Symptom:** A dead agent whose final heartbeat carried a future `ts` shows
`LIVE` indefinitely in `/peers` / `/agent-listeners` / fleet merge, while the
Rust `/find-idle` correctly retires it — the surfaces disagree and an
orchestrator dispatches work to a corpse.

**Root cause:** `age = (now_ms - .ts)/1000` has no lower bound. A future `ts`
makes `age` negative; `age <= 2*interval` is then trivially true → `LIVE`.

**Why structurally allowed:** the future-clock-safety guard exists in the
codebase (T-2536 `find_idle` future cutoff; tools.rs:4029 chat-arc-stats drops
`ts > now_ms`) but was never applied to the shell listeners surface. T-2585
unified only the `msg_type=="heartbeat"` predicate across surfaces, not the
window/clock logic — so the guard's absence here was invisible.

**Prevention:** `tests/agent-listeners-liveness-fixtures.sh` asserts a
future-dated heartbeat is not `LIVE` (load-bearing: FAILS pre-fix). The fix
adds `(.ts // 0) <= $now_ms` to the heartbeat select, mirroring T-2536.

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

### 2026-08-10T20:21:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2598-agent-listenerssh-shows-future-dated-hea.md
- **Context:** Initial task creation

### 2026-08-10T20:21:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-c3c1dea4
- **Timestamp:** 2026-08-10T20:23:45Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-10T20:23:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
