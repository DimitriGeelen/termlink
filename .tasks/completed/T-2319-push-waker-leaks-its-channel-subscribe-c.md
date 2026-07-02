---
id: T-2319
name: "Push-waker leaks its channel-subscribe child on be-reachable stop (orphan reconnect loop)"
description: >
  Push-waker leaks its channel-subscribe child on be-reachable stop (orphan reconnect loop)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: ["arc:push-transport", "bug", "resource-leak"]
components: [scripts/be-reachable.sh]
related_tasks: [T-2318, T-2316, T-2314]
arc_id: push-transport
created: 2026-07-02T22:34:38Z
last_update: 2026-07-02T22:51:56Z
date_finished: 2026-07-02T22:51:56Z
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

# T-2319: Push-waker leaks its channel-subscribe child on be-reachable stop (orphan reconnect loop)

## Context

Surfaced by the T-2318 live E2E. `scripts/be-reachable-pushwaker.sh` holds its
`channel subscribe inbox.queued --push` child via `done < <(… --push)` process
substitution with **no trap**. `be-reachable.sh cmd_stop` SIGTERMs the waker
*script* (its recorded `pushwaker_pid`), but the subscribe child is a separate
process that is left **orphaned** — and because it inherits the T-2314 active
reconnect, it loops trying to re-subscribe forever (degrading to a poll that never
exits). Every `/be-reachable stop` therefore leaks one live `channel subscribe`
process. Fix: `cmd_stop` kills the waker's **process group** (setsid makes it a
group leader), reaping the subscribe child atomically; a waker-side trap is kept
as defense-in-depth. (The obvious "trap in the waker" alone does NOT work — bash
defers the trap while the waker is blocked in `read` on the idle stream; see
Evolution.)

## Acceptance Criteria

### Agent
- [x] `be-reachable.sh cmd_stop` reaps the waker's `channel subscribe … --push`
      child: when the waker is a setsid process-group leader it kills the whole
      group (`kill -TERM -<pgid>` then `-KILL`), falling back to a plain pid-kill
      otherwise. The waker also carries a `TERM`/`INT`/`EXIT` trap as defense-in-depth
      (foreground Ctrl-C / stream-death), documented as NOT sufficient alone because
      bash defers a trapped signal while blocked in `read` on the idle push stream
- [x] New regression `scripts/test-pushwaker-reap.sh`: starts the waker via the real
      `be-reachable.sh start`, captures both the `pushwaker_pid` and its subscribe
      child pid, runs `be-reachable.sh stop`, and asserts **both** are gone; prints
      `RESULT: PASS`
- [x] The T-2318 live E2E (`scripts/demo-pushwaker-e2e.sh`) still prints `RESULT: PASS`
      after the change (fix does not regress wake behaviour) AND leaves no orphan
      `channel subscribe inbox.queued --push` process behind
- [x] `bash -n` clean on the modified waker + new regression script

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

bash -n scripts/be-reachable.sh
bash -n scripts/be-reachable-pushwaker.sh
bash -n scripts/test-pushwaker-reap.sh
# Regression: stop must reap the subscribe child (no orphan).
out=$(bash scripts/test-pushwaker-reap.sh 2>&1); echo "$out" | grep -q "RESULT: PASS"
# Wake behaviour still works end-to-end after the fix.
out2=$(bash scripts/demo-pushwaker-e2e.sh 2>&1); echo "$out2" | grep -q "RESULT: PASS"

## RCA

**Symptom:** After `/be-reachable stop`, a `termlink channel subscribe
inbox.queued --push` process remains alive on the host, looping against the hub
(T-2314 active reconnect) — one leaked process per start/stop cycle.

**Root cause:** `be-reachable-pushwaker.sh` reads its push stream via
`done < <("$TERMLINK" channel subscribe … --push)` (process substitution) and
installs **no signal trap**. `be-reachable.sh cmd_stop` terminates only the waker
script's recorded `pushwaker_pid`; the process-substitution child is a separate
PID with no supervisor, so it is orphaned rather than reaped.

**Why structurally allowed:** the WP1/WP2 demos (T-2316/T-2317) invoke the waker
directly and stub `termlink inject`; neither drives `be-reachable.sh cmd_stop`, so
the stop-path child-reaping was never exercised until the T-2318 live E2E ran the
real lifecycle and left an observable orphan.

**Prevention:** `scripts/test-pushwaker-reap.sh` asserts, through the real
`be-reachable start`/`stop`, that BOTH the waker pid and its subscribe child are
gone after stop — added to this task's Verification so the completion gate runs it
and any future regression re-blocks completion.

## Evolution

### 2026-07-03 — the obvious fix (waker trap) does NOT work; group-kill does

- **First attempt (wrong):** put a `trap … TERM INT EXIT` in the waker to reap its
  own `channel subscribe --push` child. Empirically **failed** — the regression
  still left a live orphan. Diagnosis (instrumented direct + setsid runs): the
  subscribe IS a direct child of the waker, but **bash defers a trapped signal
  while blocked in an un-timed `read`**, and the waker sits in
  `while read … done < <(subscribe)` with no data on an idle stream. cmd_stop's
  SIGTERM is delivered but the trap never runs until the stream unblocks — so the
  child is orphaned regardless of the trap.
- **Working fix:** reap from `cmd_stop` by killing the waker's **process group**.
  cmd_start spawns the waker under `setsid`, so it is its own group leader
  (`pgid == pushwaker_pid`, confirmed via `ps -o pgid=`); `kill -TERM -<pgid>`
  (then `-KILL`) takes down the waker AND its subscribe child atomically, with a
  guarded fallback to plain pid-kill when setsid is unavailable (never signals an
  unrelated group). The waker trap is kept as defense-in-depth for the foreground
  Ctrl-C / stream-death cases where `read` is not blocking.
- **Plan impact:** the fix moved from the waker (where the AC first assumed it) to
  cmd_stop; AC-1 updated to describe the delivered mechanism. Wake path untouched
  (T-2318 E2E re-run PASSes, before=0→after-self=2, no false wake, no orphan left).
- **Triggered:** `scripts/test-pushwaker-reap.sh` regression added + wired into
  Verification; also tightened its subscribe match to the binary path (`/termlink
  channel subscribe …`) so it never matches an unrelated shell command line on a
  busy host. No further sub-tasks.

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

### 2026-07-02T22:34:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2319-push-waker-leaks-its-channel-subscribe-c.md
- **Context:** Initial task creation

### 2026-07-02T22:51:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
