---
id: T-2485
name: "session-selftest — affirmative prover for the control-terminal-sessions charter
  verb (4th verb, sibling of comms-selftest)"
description: >
  session-selftest — affirmative prover for the control-terminal-sessions charter
  verb (4th verb, sibling of comms-selftest)

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T00:24:31Z
last_update: '2026-08-18T18:59:11Z'
date_finished: 2026-08-02T00:31:16Z
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
  - ts: '2026-08-18T18:56:47Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 1
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=1 (body:hand-wired-dispatch)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:11Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2485: session-selftest — affirmative prover for the control-terminal-sessions charter verb (4th verb, sibling of comms-selftest)

## Context

Fresh critical-review gap (T-2468 mandate, 7th re-issue). The charter names FOUR core
verbs: discover / exchange durable messages / claim work / **control terminal
sessions**. Affirmative on-demand provers exist for three: comms-selftest (T-2482, P7)
proves discover+exchange; substrate-smoke (T-2151) proves claim work. But the FOURTH —
"control terminal sessions", the FOUNDING verb ("TermLink began as a cross-terminal
session-control tool", CHARTER.md) — has **no affirmative prover**. The gap is
explicitly acknowledged in-code: `scripts/agent-conversation-selftest.sh` says "What it
does NOT validate: PTY inject … requires a live peer session", and comms-selftest STAGE
1 only checks the `pty_session` presence FLAG, never that a command actually injects and
executes. So nothing proves, on demand, that you can register a terminal session and
inject/exec into it right now (G-069 shipped≠live class, applied to the PTY verb).

Unlike doorbell-wake (needs a live peer), `termlink exec <session> <cmd> --json` makes
this **deterministic and local**: spawn a tmux-backed scratch session, exec a sentinel,
assert stdout+exit_code. This is the direct 4th sibling of comms-selftest — an on-demand
PROVER (not a cron canary, so it does NOT add to the canary breadth). Completes
affirmative-prover coverage across all four charter verbs (deepen-the-core, not add).

## Acceptance Criteria

### Agent
- [x] `scripts/session-selftest.sh` exists, is executable, and `--help` documents purpose
      (affirmative prover for the control-terminal-sessions charter verb), `--json`,
      `--ttl <secs>`, and exit codes (0 proven / 1 broken-names-stage / 2 tooling).
- [x] Runs three staged PASS/FAIL checks against a real session: **SPAWN** (a tmux-backed
      scratch session registers), **EXEC** (`termlink exec <session> 'echo <sentinel>'
      --json` returns ok+exit_code 0 and stdout containing the unique sentinel — the
      inject→run→capture round-trip), **CLEANUP** (best-effort `signal TERM` + `clean`;
      never fatal, since a self-reaped session makes signal legitimately fail).
- [x] Exit 0 only when SPAWN and EXEC both PASS; exit 1 naming the broken stage when
      either fails; exit 2 (fail-closed) on tooling error (missing termlink/jq, hub down).
- [x] `--json` emits `{ok, proven, broken_stage, stages:{spawn,exec,cleanup}, session,
      sentinel}` (mirrors comms-selftest's envelope shape).
- [x] Host-independent test hooks (PL-213): `TERMLINK_SESSION_SELFTEST_TEST_SPAWN_RC` and
      `TERMLINK_SESSION_SELFTEST_TEST_EXEC_JSON` let the test suite drive every path with
      no live hub.
- [x] `scripts/test-session-selftest.sh` covers proven / broken-at-SPAWN / broken-at-EXEC
      (bad exit_code) / broken-at-EXEC (missing sentinel) / tooling via hooks; prints
      `PASS`/`FAIL`; all pass.
- [x] Live-validated once this session against the real local hub: full run exits 0
      (PROVEN) — recorded in the task Updates with the sentinel echoed back.
- [x] `docs/operations/session-selftest.md` written, and a CLAUDE.md mention added
      placing it in the affirmative-prover family alongside comms-selftest (the two
      complete prover coverage of the four charter verbs). NOT a cron canary.

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

## Updates

### 2026-08-02 — build + verification COMPLETE
- **Gap (7th mandate re-issue):** of the four charter verbs, three had affirmative
  provers (comms-selftest T-2482 = discover+exchange; substrate-smoke T-2151 = claim)
  but the 4th — "control terminal sessions", the FOUNDING verb — had none. Confirmed by
  reading the substrate scripts (substrate-smoke already covers claim, so no dup) and
  the explicit in-code note in agent-conversation-selftest.sh ("does NOT validate: PTY
  inject"). comms-selftest only checks the pty_session presence FLAG.
- **Built:** `scripts/session-selftest.sh` — SPAWN (tmux-backed session registers) →
  EXEC (`termlink exec <s> 'echo <sentinel>' --json` → ok+exit0+sentinel, the
  inject→run→capture round-trip) → CLEANUP (best-effort signal+clean). Deterministic +
  local via `exec --json` (no live peer needed). Exit 0 proven / 1 broken-names-stage /
  2 tooling (fail-closed on hub-down).
- **Robustness fix found during live-validation:** two back-to-back live runs disagreed
  (one PROVEN, one BROKEN@EXEC) — `spawn` returns rc 0 before the tmux shell is
  exec-ready (intermittent). Added a bounded EXEC retry (~5s) that absorbs the slow
  start; a ready session still passes on attempt 1, a broken verb still FAILs after the
  budget. Stress-tested 5/5 proven, 0 leaked sessions.
- **Tests:** `scripts/test-session-selftest.sh` → `test-session-selftest: PASS` (9/9:
  proven / broken@SPAWN / broken@EXEC bad-code / broken@EXEC missing-sentinel /
  broken@EXEC ok:false-no-PTY / unknown-arg-tooling / json-proven-envelope /
  json-broken-names-EXEC / --help). Live real-hub run → exit 0 PROVEN (sentinel
  `SESSION-SELFTEST-OK-<nonce>` echoed back through `exec`).
- **Docs:** `docs/operations/session-selftest.md` + CLAUDE.md §"Session self-test"
  placing it in the affirmative-prover family (4/4 charter verbs now have a prover).
  NOT a cron canary — an on-demand prover, so zero canary-breadth added.

## Verification

test -x scripts/session-selftest.sh
out=$(bash scripts/session-selftest.sh --help 2>&1); echo "$out" | grep -q "control terminal sessions"
out=$(bash scripts/test-session-selftest.sh 2>&1); echo "$out" | grep -q "^test-session-selftest: PASS"
test -f docs/operations/session-selftest.md

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

**Symptom:** The "control terminal sessions" charter verb (register→inject/exec into a
PTY) had no affirmative on-demand prover, while the other three charter verbs did — so
"can I actually exec into a session right now?" was unanswerable except by ad-hoc manual
attempts. Separately, during live-validation the new prover itself flaked (two
back-to-back runs disagreed: PROVEN then BROKEN@EXEC).

**Root cause:** (1) Prover coverage grew verb-by-verb (comms-selftest, substrate-smoke)
without anyone tracking that the founding PTY verb was the odd one out — the gap was even
acknowledged in-code ("does NOT validate: PTY inject") but never closed. (2) `termlink
spawn` returns rc 0 before the tmux-backed shell is guaranteed exec-ready, so an
immediately-following `exec` intermittently hits a not-yet-ready session.

**Why structurally allowed:** no inventory tied "affirmative prover" to "each charter
verb", so a missing one was invisible. The race was invisible because nothing exercised
spawn→exec back-to-back at speed before.

**Prevention:** the prover itself (`scripts/session-selftest.sh`) is the standing answer
for the coverage gap — the four verbs now each have a prover. The race is prevented by
the bounded EXEC readiness-retry (~5s) baked into the prover, covered by the test suite
+ a 5/5 live stress run. Learning captured on prover-coverage-per-promise.

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

### 2026-08-02T00:24:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2485-session-selftest--affirmative-prover-for.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-b4fe63a2
- **Timestamp:** 2026-08-02T00:31:18Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T00:31:16Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
