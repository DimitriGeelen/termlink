---
id: T-2318
name: "arc-004 WP1/WP2 live end-to-end push-waker proof (real spawn+inject, no stub)"
description: >
  arc-004 WP1/WP2 live end-to-end push-waker proof (real spawn+inject, no stub)

status: started-work
workflow_type: test
owner: agent
horizon: now
tags: ["arc:push-transport", "e2e", "verification"]
components: []
related_tasks: [T-2316, T-2317, T-2315]
arc_id: push-transport
created: 2026-07-02T22:26:34Z
last_update: 2026-07-02T22:26:34Z
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

# T-2318: arc-004 WP1/WP2 live end-to-end push-waker proof (real spawn+inject, no stub)

## Context

The T-2316 (WP1) and T-2317 (WP2) push-waker demos prove the *logic* — filter,
dedup, blip-resume — but they invoke `be-reachable-pushwaker.sh` **directly** and
replace `termlink inject` with a **stub** that only logs the command. Two seams
are therefore unproven end-to-end: (1) the `be-reachable.sh cmd_start` wiring that
actually spawns the waker as a detached process and records `pushwaker_pid`; and
(2) a **real** `termlink inject` landing in a **real** PTY-backed session on an
inbox deposit. This task closes that gap with a live proof against an isolated
hub + HOME: `be-reachable start` → waker spawned (pid recorded, alive) → real
`termlink spawn` session → real `inbox:<id>` deposit → real inject observed in the
session's own output → `be-reachable stop` reaps the waker. Verification, not new
feature — strengthens the arc-close evidence for the human's sovereignty-gated
`fw arc close push-transport`. See docs/reports/T-2318-arc-004-pushwaker-e2e-demo.md.

## Acceptance Criteria

### Agent
- [ ] `scripts/demo-pushwaker-e2e.sh` exists, is executable, and passes `bash -n`
- [ ] Demo drives the **real** operator path: `be-reachable.sh start` spawns the
      waker; the state file records a non-null `pushwaker_pid` whose process is alive
- [ ] Demo uses a **real** `termlink spawn` PTY session and the **real** waker (NO
      stub inject); a deposit to `inbox:<self>` produces a real inject observable in
      the spawned session's own output (verified via `termlink output`, not a log stub)
- [ ] A deposit to a **different** inbox does NOT ring the spawned session (no false wake)
- [ ] `be-reachable.sh stop` terminates the recorded `pushwaker_pid` (process gone after stop)
- [ ] Demo is hermetic: isolated `TERMLINK_RUNTIME_DIR` + `HOME` + loopback hub port,
      no writes to the real fleet/agent-presence; prints `RESULT: PASS` on success
- [ ] Evidence report `docs/reports/T-2318-arc-004-pushwaker-e2e-demo.md` captures the
      run transcript and states the honest scope (what this adds over T-2316/T-2317)

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

# Syntax-check the new demo + its dependencies.
bash -n scripts/demo-pushwaker-e2e.sh
bash -n scripts/be-reachable-pushwaker.sh
bash -n scripts/be-reachable.sh
# Run the live E2E demo hermetically and require PASS (capture-then-grep per L-387).
out=$(bash scripts/demo-pushwaker-e2e.sh 2>&1); echo "$out" | grep -q "RESULT: PASS"
# Evidence report present.
test -f docs/reports/T-2318-arc-004-pushwaker-e2e-demo.md

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

### 2026-07-03 — building the harness surfaced the real seams (and a real leak)

- **What changed (harness):** a faithful E2E needed three things the stub demos
  never used. (1) `termlink spawn` alone gives `PTY: no`; a REAL injectable PTY
  needs `spawn --shell --backend tmux`. (2) The injected keystrokes are NOT visible
  via `tmux capture-pane` (that pane shows the termlink session-SERVER log) nor via
  a `cat >> file` pane (block-buffered, never flushes a short line) — the faithful
  read is `termlink output <session> --strip-ansi`, which reads the inner shell's
  own terminal through the data plane. (3) `be-reachable start` spawns the waker
  WITHOUT `--hub`, so it rides the local socket in `TERMLINK_RUNTIME_DIR` — the
  isolated hub is reached simply by exporting that var.
- **What changed (finding):** the E2E caught a **stop-path resource leak** the WP1/WP2
  stub demos structurally could not. `be-reachable-pushwaker.sh` holds its
  `channel subscribe inbox.queued --push` child via `done < <(… --push)` process
  substitution with **no trap**. `cmd_stop` SIGTERMs the waker *script*, but the
  subscribe child is orphaned — and with the T-2314 active reconnect it loops
  against the hub forever. The recorded `pushwaker_pid` IS reaped (so this demo's
  AC — "stop reaps the recorded pid" — still passes), but a real operator's
  `/be-reachable stop` leaves a live orphan.
- **Plan impact:** none to T-2318's scope — the demo asserts exactly the shipped
  contract (real spawn → real inject → real filter → recorded-pid reaped) and PASSes.
  The leak is a distinct bug, not a demo failure.
- **Triggered:** filed **T-2319** (build) to add a child-reaping trap to the waker +
  a regression assertion; this E2E demo is the reproduction harness for it (the
  orphan is visible as a lingering `channel subscribe inbox.queued --push` after the
  run's cleanup kills the isolated hub).

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

### 2026-07-02T22:26:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2318-arc-004-wp1wp2-live-end-to-end-push-wake.md
- **Context:** Initial task creation
