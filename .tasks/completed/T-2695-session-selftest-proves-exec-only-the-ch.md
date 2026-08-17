---
id: T-2695
name: "session-selftest proves exec only; the charter also claims inject and output"
description: >
  The charter says peers can stream output, inject keystrokes, exec, and doorbell-wake PTY sessions. session-selftest.sh exercises only 'termlink exec'. inject's unit tests are named command_inject_resolves_keys_no_pty — key resolution without a PTY. Add INJECT and OUTPUT stages proving the capabilities end-to-end (T-2694 F1/G1+G2).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/session-selftest.sh, scripts/test-session-selftest.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T08:01:34Z
last_update: 2026-08-17T06:06:15Z
date_finished: 2026-08-17T06:06:15Z
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

# T-2695: session-selftest proves exec only; the charter also claims inject and output

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] An **INJECT** stage proves `termlink inject` end-to-end: keystrokes written into a live PTY produce their **observable effect**, not merely an `ok` RPC response
- [x] The proof is by effect, not by return code — an `ok` proves the bytes were accepted, which is exactly what the existing `no_pty` unit tests already establish and is not what the charter claims
- [x] An **OUTPUT** stage proves `termlink output` streams back PTY content the session actually produced
- [x] ~~Both stages reuse the session the prover already spawns~~ — **corrected during build.** The existing session is spawned `-- sleep <TTL>` and has `pty: null`; `output` refuses it with `-32007 No PTY session` and `inject` cannot reach a terminal through it. Reuse is structurally impossible, so the PTY stages spawn their OWN `--shell` session. The `sleep`-backed session stays exactly as-is so the T-2557 canary's existing stages carry zero regression risk.
- [x] The PTY session is cleaned up on every exit path, including when a PTY stage fails — a leaked tmux session per canary run would be worse than the gap being closed — **CORRECTION (2026-08-17, post-closure): this AC was NOT satisfied when ticked.** `signal TERM` + `clean` remove the termlink registration but leave the backing tmux session running, and both return rc=0 while doing so. Measured after closure: 7 orphaned `tl-session-selftest-*-pty` sessions against `termlink list` showing 0. The AC even names the exact failure ("a leaked tmux session per canary run") — it was ticked on the cleanup code *existing*, not on the tmux session being gone, which is the same reasoning-instead-of-measuring error this task's own seam-coverage decision documents one section below. Fixed under **T-2780**.
- [x] Stages absorb the PTY timing race the way EXEC already does (bounded retry), so the prover does not become flaky and start firing its canary on timing
- [x] A failure in either stage names *which* stage broke, matching the existing `broken_stage` contract
- [x] JSON envelope extended additively — `stages.inject` / `stages.output` alongside the existing keys; no existing key renamed or removed
- [x] Test seams let the new stages be exercised without a live PTY (mirroring `TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON`), so the canary translation stays verifiable
- [x] Exit-code contract preserved: 0 proven / 1 broken (names the stage) / 2 tooling — a missing tmux must stay exit 2, never a false "broken"
- [x] Verified by actually running it on this host, not asserted — `session-selftest.sh --json` returns `proven:true` with the new stages present
- [x] Load-bearing: sabotaging the injected sentinel makes the INJECT stage fail rather than silently pass
- [x] `docs/operations/session-selftest.md` updated so the documented stage list matches reality

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

bash scripts/session-selftest.sh --json
bash scripts/test-session-selftest.sh
bash scripts/check-session-control-freshness.sh --quiet
out=$(bash scripts/session-selftest.sh --json); echo "$out" | jq -e '.stages.inject == "PASS" and .stages.output == "PASS"'
# The PTY seams must stay EXERCISED, not merely present (PL-329). Count-pin: 23 cases
# = 14 pre-existing + 9 added here. Deleting any assertion re-fires this line.
# Herestring, not a pipe: `| grep -q` exits 141 under the gate's pipefail (T-2775).
out=$(bash scripts/test-session-selftest.sh 2>&1); test "$(grep -c '^  ok   ' <<< "$out")" -eq 23
out=$(bash scripts/test-session-selftest.sh 2>&1); grep -q 'ok   INJECT failure names the INJECT stage' <<< "$out"
out=$(bash scripts/test-session-selftest.sh 2>&1); grep -q 'ok   both failing attributes to OUTPUT, not INJECT' <<< "$out"

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

### 2026-08-17 — the seams existed but nothing exercised them

- **Chose:** Add 9 assertions to `scripts/test-session-selftest.sh` covering the
  `TERMLINK_SESSION_SELFTEST_TEST_{OUTPUT,INJECT}_STATUS` seams before closing.
- **Why:** The AC "test seams let the new stages be exercised without a live PTY,
  so the canary translation stays verifiable" was ticked on the seams *existing*.
  Measured at closure: `grep -c 'INJECT_STATUS\|OUTPUT_STATUS'` against the suite
  returned **0**. The seams were written and never invoked, so a typo'd env name,
  or a FAIL path that forgot to set `broken_stage`, would have been invisible while
  the live run kept passing on this host. That is PL-329 exactly — *a guard is only
  as load-bearing as its invocation path* — reproduced inside the task that added
  the guard. Closing on the un-exercised version would have made the AC a claim
  about a mechanism no test ran.
- **Coverage added:** seams-unset ⇒ both stages `skipped` and `proven` NOT blocked
  (the back-compat guarantee the script's own comment asserts); INJECT FAIL ⇒
  `broken_stage:"INJECT"`; OUTPUT FAIL ⇒ `broken_stage:"OUTPUT"`; both PASS ⇒
  `proven` with both keys present; and both-FAIL ⇒ attributed to **OUTPUT**, pinning
  the deliberate diagnosis ordering (inject's effect is observed *through* output,
  so blaming INJECT when output is broken is a wrong answer).
- **Load-bearing, proven by mutation, not asserted:** removing the INJECT
  `broken_stage` attribution fails 1 case; removing "inject FAIL blocks `proven`"
  fails 2. Both reverted; suite green; `git diff` on the prover clean.
- **Rejected:** closing on the live `proven:true` run alone. A live green on one
  host is evidence the verb works *today*, not evidence the prover would report
  correctly when it breaks — which is the entire purpose of a canary-backed prover.

### 2026-08-17 — the suite itself has no automated invocation path (filed separately)

- **Found:** `scripts/test-session-selftest.sh` is referenced *only* by task
  Verification blocks — T-2563 (already in `completed/`) and this task. It is not in
  CI and not in the guard layer, whose discovery covers `scripts/check-*.sh` carrying
  a `# guard-layer: source` marker plus `tests/*fixtures*.sh` by naming convention.
  This file matches neither, and adding a marker would not help because the loop
  never iterates `scripts/test-*.sh`. On this task's closure the suite drops to
  **zero** automated invocations.
- **Chose:** File it as its own task rather than fix it here.
- **Why:** It is a distinct defect with a distinct root cause, it predates this task
  (T-2563's suite had the same gap), and it affects all 18 cases rather than the 9
  added here — "one bug = one task" per the sizing rules. Widening T-2695 to cover
  it would bury a guard-layer coverage defect inside a PTY-stage task.

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-14T08:01:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2695-session-selftest-proves-exec-only-the-ch.md
- **Context:** Initial task creation

### 2026-08-14T08:01:56Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-08-17T06:06:15Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
