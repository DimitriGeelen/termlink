---
id: T-2320
name: "arc-004 push-wake latency benchmark — measure push-wake vs poll floor"
description: >
  arc-004 push-wake latency benchmark — measure push-wake vs poll floor

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: ["push-transport", "benchmark", "verification"]
components: []
related_tasks: [T-2303, T-2316, T-2318]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-02T23:21:30Z
last_update: 2026-07-02T23:32:37Z
date_finished: 2026-07-02T23:32:37Z
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

# T-2320: arc-004 push-wake latency benchmark — measure push-wake vs poll floor

## Context

Follow-on **verification** of the closed arc-004 `push-transport` (WS live push,
shipped). The T-2303 inception GO'd on the value claim that hub→client WS push
replaces the ~15 s doorbell-then-poll wake floor with a **sub-second** wake — but
§10 (lines 201–205) flagged the one honest gap: *"the 15 s latency floor is read
from code constants, not a live end-to-end measurement… a 30-min live baseline
would confirm the delta is worth it. This is the one reason a reasonable reviewer
might choose DEFER over GO."* The arc shipped on the code-constant basis; this task
retires that gap by **measuring** the push-delivery latency rigorously (N trials,
percentiles) against an isolated hub and putting it side-by-side with the
documented pre-push floor. Measurement, not new build — no inception, no arc reopen.
Related: T-2316 (WP1 push-waker, single 172 ms full-E2E point), T-2318 (E2E
functional proof), T-2303 (inception §10 gap).

## Acceptance Criteria

### Agent
- [x] `scripts/bench-pushwake-latency.sh` exists and is `bash -n` clean; runs
      hermetically (isolated `TERMLINK_RUNTIME_DIR` + `HOME` + loopback TCP hub +
      real PTY spawn), pins the real release binary via `TERMLINK_BIN`, and
      self-cleans (be-reachable stop + tmux PTY + hub + temp dirs) on exit.
- [x] The benchmark performs **N ≥ 10** trials (default 12), each measuring the
      latency from `channel post inbox:<self>` to the real doorbell inject being
      observed in a live PTY session, and reports **min / median / p95 / max** in
      milliseconds. *(12/12 trials rang in both runs.)*
- [x] Measured **median** wake latency is **sub-second (< 1000 ms)** — the arc's
      core value claim, now measured rather than assumed (asserted by the script,
      exit 6 if violated). *(Run 1 median 111 ms, run 2 median 85 ms.)*
- [x] `docs/reports/T-2320-arc-004-pushwake-latency-benchmark.md` exists with the
      per-trial table, the min/median/p95/max summary (both runs), and a
      side-by-side vs the documented pre-push doorbell-then-poll floor (floor cited
      to T-2303 §10; `--follow` 1 s poll floor cited to CLI help).
- [x] The report states the **honest measurement scope**: what is timed (full wake
      path: post → push → inject → shell echo) as an **upper bound** (observation
      cost counted in), cross-referenced to T-2316's ~172 ms full-E2E point and
      T-2313's 31 ms delivery-only point, so the number is not over-claimed.

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

bash -n scripts/bench-pushwake-latency.sh
out=$(bash scripts/bench-pushwake-latency.sh 2>&1); echo "$out" | grep -q "RESULT: PASS"
test -f docs/reports/T-2320-arc-004-pushwake-latency-benchmark.md
grep -q "p95" docs/reports/T-2320-arc-004-pushwake-latency-benchmark.md

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

### 2026-07-03 — measurement approach pivoted from synthetic proxy to real path
- **What changed:** The first design measured a synthetic proxy — post to a
  normal topic → receipt at a live `subscribe` stream. That path does NOT work:
  (a) plain `subscribe inbox.queued` returns `unknown topic` (the aggregator
  frame is not a real topic); (b) `--push` only delivers aggregator `hub.event`
  frames and requires a hubs.toml profile + a **registered session** as the
  aggregator sink — a bare `--push` subscribe on a normal topic receives nothing;
  (c) `channel post` to a non-existent `inbox:<id>` errors before the
  `inbox.queued` emit ever fires (channel.rs:752 guards the emit behind a
  successful append). So there is no clean synthetic shortcut to the push frame.
- **Plan impact:** Rather than fight the aggregator/registration plumbing, the
  benchmark was rebuilt on T-2318's **proven** hermetic harness — real
  `spawn --shell` PTY + real `be-reachable start` (which registers the session
  and spawns the waker) + real `termlink inject` observed in the PTY. This
  measures the ACTUAL full production wake path, which is strictly more honest
  than any proxy would have been.
- **Triggered:** Reframed the metric as an explicit **upper bound** (the
  `termlink output` poll used to detect the ring costs one RPC per iteration, so
  observation cost is counted into the latency, never out). Result: 85–111 ms
  median, sub-100 ms, vs the documented 15 s floor. No new sub-task; the pivot
  stayed within this task's scope.

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

### 2026-07-02T23:21:30Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2320-arc-004-push-wake-latency-benchmark--mea.md
- **Context:** Initial task creation

### 2026-07-02T23:32:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
