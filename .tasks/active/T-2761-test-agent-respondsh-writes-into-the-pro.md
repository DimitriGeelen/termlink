---
id: T-2761
name: "test-agent-respond.sh writes into the production woken-but-silent canary log"
description: >
  test-agent-respond.sh invokes the real agent-send.sh without exporting TERMLINK_WOKEN_SILENT_LOG, so its by-design give-up path appends to the real .woken-but-silent-canary.log. Sibling test-agent-send.sh got that redirect in T-2402 Stage 5; this one was not migrated. Because empty-log = healthy is a one-bit channel, test residue leaves the canary permanently FIRING and therefore deaf to genuine events (T-2685 harm). Observed: entry cid=cidB-962839 topic=agent-respond-test-962839 dated 2026-08-15T22:19:24Z.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-16T11:59:02Z
last_update: 2026-08-16T12:00:26Z
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

# T-2761: test-agent-respond.sh writes into the production woken-but-silent canary log

## Context

`scripts/agent-send.sh` escalates a never-acked send by appending a framed entry
to the woken-but-silent canary log (T-2402 Stage 5 — a loud canary instead of a
silent `exit 3`). The destination defaults to the real
`.context/working/.woken-but-silent-canary.log` and is overridable via
`TERMLINK_WOKEN_SILENT_LOG`.

`scripts/test-agent-send.sh` exports that override (line 33) precisely so its
give-up assertions do not pollute the operator's log. **`scripts/test-agent-respond.sh`
does not** — it drives the same real `agent-send.sh` (line 16) against a
deliberately-absent session, so its give-up path is guaranteed to fire, and the
entry lands in the production log.

This is the "hardened in one place, siblings not migrated" divergence that
`check-busy-spin.sh`, T-2667 and T-2673 each exist to catch, reproduced in the
test layer where no static check is looking.

**Why this is worse than ordinary test noise.** `empty log = healthy` is a
one-bit channel. Once test residue is in the file the canary reads FIRING
forever, so a subsequent genuine woken-but-silent event appends to an
already-non-empty file and changes nothing an operator can see. The canary is
not merely noisy — it is **deaf** until someone truncates it by hand. That is
exactly the harm T-2685 documents for the stderr-merge mistake, arriving here
through a different door.

Observed residue (from a test run, not real traffic):

```
=== 2026-08-15T22:19:24Z ===
woken-but-silent: no receipt for cid=cidB-962839 on topic=agent-respond-test-962839
  recipient= session=no-such-session-962839
```

`agent-respond-test-$$` is `test-agent-respond.sh:24`; `no-such-session-…` is its
deliberately-absent session.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `test-agent-respond.sh` exports `TERMLINK_WOKEN_SILENT_LOG` into its own temp dir before invoking `agent-send.sh`, mirroring `test-agent-send.sh`
- [x] Running `test-agent-respond.sh` leaves the real `.context/working/.woken-but-silent-canary.log` byte-identical
- [x] The existing test-fixture residue is cleared from the production log, so the canary is readable again
- [x] A check fires if any `tests/` or `scripts/test-*` script invokes `agent-send.sh` without redirecting the canary log — so the next sibling cannot regress the same way
- [x] The new check is a `# guard-layer: source` member, so `run-guard-layer.sh` executes it (a check nothing runs is the T-2683 defect)
- [x] The check is load-bearing: removing the export from `test-agent-respond.sh` makes it fire; restoring returns it to clean

**Scope note — the hand-found instance was 1 of 10.** The task was filed from a
single observed log entry pointing at `test-agent-respond.sh`. On its first run
the check found **nine more** unredirected callers:
`test-agent-send-auto-discover.sh`, `test-agent-send-orchestration.sh`,
`test-agent-send-transport.sh`, `test-sidecar-auto-confirm.sh`,
`agent-send-grace-window.sh`, `agent-send-idle-gate.sh`,
`relay-b1-doorbell-rail.sh`, `relay-b2-send-hops.sh`,
`wake-confirm-reply-match.sh`. All ten now redirect; the census reads 12 of 81
test scripts invoking `agent-send.sh`, all redirecting.

`agent-send-grace-window.sh` is the sharpest of them: it exists specifically to
pin the confirmation window against a false `"receiver never acked"` escalation,
and it was writing that very escalation into the operator's log.

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

# --- The guard itself, and its hermetic fixtures ---
bash scripts/check-canary-log-isolation.sh
bash tests/canary-log-isolation-fixtures.sh
#
# --- The originally-reported offender now redirects ---
grep -q 'TERMLINK_WOKEN_SILENT_LOG=' scripts/test-agent-respond.sh
#
# --- Every caller redirects, not just the one found by hand ---
# The check found 9 more beyond the reported one; this asserts the whole
# population is clean rather than the single instance.
out=$(bash scripts/check-canary-log-isolation.sh 2>&1); echo "$out" | grep -q "all redirect"
#
# --- The guard is RUN, not merely present (the T-2683 defect) ---
out=$(bash scripts/run-guard-layer.sh --list 2>&1); echo "$out" | grep -q "check-canary-log-isolation.sh"
out=$(bash scripts/run-guard-layer.sh --list 2>&1); echo "$out" | grep -q "canary-log-isolation-fixtures.sh"
#
# --- The production log is readable again (empty = healthy) ---
test ! -s .context/working/.woken-but-silent-canary.log
#
# --- Whole guard layer still clean ---
bash scripts/run-guard-layer.sh

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

**Symptom:** `/canaries` reported `woken-but-silent-canary` FIRING. The entry named
`cid=cidB-962839` on `topic=agent-respond-test-962839` with
`session=no-such-session-962839` — synthetic test fixtures, not real traffic. No
peer had actually gone silent.

**Root cause:** `scripts/test-agent-respond.sh` invokes the real
`scripts/agent-send.sh` against a deliberately-absent session, so agent-send's
T-2402 Stage 5 give-up escalation is guaranteed to fire. That escalation appends
to `TERMLINK_WOKEN_SILENT_LOG`, which defaults to the operator's real canary log.
The script never set the variable, so its test output went into the production
signal channel. Nine sibling test scripts had the same omission.

**Why structurally allowed:** The hazard was already understood — T-2402 Stage 5
added exactly this redirect to `scripts/test-agent-send.sh` at the moment the
escalation was introduced. It was applied to the one script being edited and to
no other caller, and nothing checked the rest. This is the same "hardened in one
place, siblings not migrated" divergence that `check-busy-spin.sh` (T-2672),
T-2667 and T-2673 each exist to catch, occurring in the test layer where no
static check was looking. The guard layer had eleven members and not one of them
knew that a test writing into a canary log is a defect.

The harm is disproportionate to the cause because `empty log = healthy` is a
**one-bit** channel. Test residue does not merely add noise: it saturates the
signal. A genuine woken-but-silent event afterwards appends to an already-FIRING
log and changes nothing an operator can see, so the canary is **deaf**, not just
dirty — until someone truncates it by hand. That is the same failure T-2685
documents for merged stderr, reached through a different door.

**Prevention:** `scripts/check-canary-log-isolation.sh` scans every
`scripts/test-*.sh` and `tests/*.sh`, selects those that invoke `agent-send.sh`
(comments stripped — prose about the script is not a call to it), and fires on
any that never assign `TERMLINK_WOKEN_SILENT_LOG`. It carries the
`# guard-layer: source` marker so `run-guard-layer.sh` — and therefore CI —
executes it, rather than joining the set of checks nothing runs (T-2683). An
empty candidate set is a tooling error, never a vacuous clean census (T-2747).
Load-bearing: removing the export from `test-agent-respond.sh` fires the check by
name; restoring returns it to clean. 20 hermetic fixture assertions pin both
directions plus a PL-219 control that the real tree scans clean.

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

### 2026-08-16T11:59:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2761-test-agent-respondsh-writes-into-the-pro.md
- **Context:** Initial task creation

### 2026-08-16T12:00:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
