---
id: T-2876
name: "Cross-session message prover: assert delivery on the receiver, not the sender"
description: >
  Cross-session message prover: assert delivery on the receiver, not the sender

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
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-01T18:47:39Z
last_update: 2026-09-01T18:52:17Z
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

# T-2876: Cross-session message prover: assert delivery on the receiver, not the sender

## Context

Asked for by the operator after T-2875: "test everything end to end, and also build that
as a testing harness for any future development work."

The rail it covers is the one no existing prover touches — Claude-session to
Claude-session messaging. `comms-selftest.sh` (T-2482) covers TermLink discover +
exchange, `session-selftest.sh` (T-2485) covers PTY control, `substrate-smoke.sh`
(T-2151) covers claim work. This is the fourth.

Design driven by what T-2875 actually measured, not by what the docs claim:

- **Assert on the receiver, never the sender.** `SendMessage` returns `success:true` at
  enqueue time. Three findings in two days share that shape — a hub said `injected` while
  the PTY got nothing (T-2873), a config looked authoritative and was never read (T-2874),
  a send succeeded while the target sat blocked (T-2875).
- **Separate the outcomes that look identical from outside:** DELIVERED (sentinel is a
  `user` turn), ENQUEUED (only a `queue-operation` — accepted, not drained), BLOCKED
  (absent, target on a permission prompt — *delivery was fine*), UNDELIVERED (absent,
  target idle). Collapsing BLOCKED into UNDELIVERED is exactly the misdiagnosis that made
  a working rail look impossible.
- **The target is told to do nothing.** Assertion is on its transcript, so the prover
  measures the rail rather than the target's obedience — and cannot be defeated by a
  target that lacks permission to act.
- **Send is agent-driven.** `claude --help` has no send/message/peer verb (verified), so
  `prepare`/`assert`/`cleanup` compose *around* the send instead of pretending to do it.

## What the live pass measured (and what it corrected)

The script was written but never executed. Running it turned three of its stated
premises into measured facts, two of which were false:

**1. `waitingFor` does not exist.** The BLOCKED branch keyed off
`claude agents --json`'s `waitingFor` field. Measured across the live fleet: 0 of 56
agent records carried it. The entire branch was dead code, and the AC below was
written from the same false premise.

**2. `state:"blocked"` is the RESTING state, not a permission signal.** 43 of those
56 agents were `blocked`, 40 of them stale records with no `status` at all. The
fallback would have classified almost every broken rail as "target merely stuck" —
the exact inversion of the bug this prover exists to prevent. It is now used only to
ANNOTATE an already-queued message, never to decide whether delivery happened.

**3. The real observable is the receiver's own transcript.** A `queue-operation`
carrying the sentinel with no matching `user` turn means accepted-but-never-drained.
That is the T-2875 shape, and it is readable without consulting the sender at all.

**4. `prepare` handed back an address that could not receive anything.** `claude --bg`
produces a record that appears in `claude agents --json` but carries no `status`,
never writes a transcript, and is refused by SendMessage ("No agent named ... is
reachable"). The reachability predicate turned out to be the presence of a `status`
field — that set matched the SendMessage peer list exactly (15 of 58 records).
`prepare` now gates on it and gained `--existing MATCH` to target a session that is
already reachable, which is how the live pass was actually run.

The fail-closed design held up under all of this: pointed at the unreachable probe,
the script exited 2 ("cannot observe the receiver") rather than inventing a verdict.

**Live end-to-end result** (peer `resume workflow state aggregation`, 2026-09-01):
DELIVERED, rc 0 — sentinel present as a `user` turn in the receiver's transcript.
Control with a sentinel that was never sent: UNDELIVERED, rc 1. Unobservable
receiver: TOOLING, rc 2. The full exit-code contract is proven against real data,
not fixtures.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/session-message-selftest.sh` exists and proves the one rail no existing prover covers: Claude-session → Claude-session messaging. (`comms-selftest.sh` = TermLink discover/exchange, `session-selftest.sh` = PTY control, `substrate-smoke.sh` = claim work; none touch this.)
- [x] It asserts on the **receiver's** transcript, never on the sender's return value — the T-2873/T-2874/T-2875 pattern is that a layer reporting success says nothing about the next layer acting.
- [x] It distinguishes the outcomes that are byte-identical from the sender's side: DELIVERED (sentinel is a `user` turn), BLOCKED (sentinel queued, target at rest — delivery was fine), ENQUEUED (sentinel queued, target still busy — may yet drain), UNDELIVERED (sentinel absent). A prover that collapsed these would reproduce the exact confusion it exists to remove. **Corrected during implementation:** this AC originally named `waitingFor` in `claude agents --json` as the BLOCKED signal. That field does not exist (0 of 56 agents), so the split is derived from the receiver's transcript instead — see the measurement block above.
- [x] Subcommands `prepare` / `assert` / `cleanup` compose around the send, because the send leg has no CLI verb and must be an agent tool call. The script does not pretend to perform it.
- [x] Exit codes follow the repo convention: 0 proven, 1 broken (naming the stage), 2 tooling — **fail-closed**, so a missing `claude`, missing `python3`, or unreadable transcript exits 2 and never a false "proven".
- [x] `--json` for scripting, and test seams (PL-213) `SESSION_MSG_TEST_AGENTS_JSON` + `SESSION_MSG_TEST_TRANSCRIPT_DIR` so the fixtures run with no live session and no network.
- [x] `tests/session-message-selftest-fixtures.sh` covers all three classifications plus the fail-closed paths, and is weighted toward the FIRING cases — a prover that only ever goes green is not a prover.
- [x] Load-bearing: the fixtures fail if the BLOCKED classification is removed, proving that branch is doing work rather than decorating the output.
- [x] It carries the `# guard-layer: source` marker only if it is safe to run with no live session; if it is not, it is deliberately left out of the runner and the reason is stated in the script header.

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
#
# Every criterion for this task is agent-verifiable, so the Human AC section is
# omitted per the template's own instruction rather than left as empty boilerplate.
#
# Grep markers are read from files, never through a pipe (L-387): under the gate's
# `set -eo pipefail`, `cmd | grep -q PAT` returns 141 when the pattern MATCHES.

# The prover itself must be present and executable.
test -x scripts/session-message-selftest.sh

# The fixture suite must be fully green. `&&` keeps the suite's own exit code in the
# verdict; the marker is grepped from a file, never through a pipe (L-387).
bash tests/session-message-selftest-fixtures.sh > /tmp/.t2876-fixtures.out 2>&1 && grep -q "0 failed" /tmp/.t2876-fixtures.out

# Fail-closed: an unobservable receiver must exit 2, never a delivery verdict.
rc=0; SESSION_MSG_TEST_TRANSCRIPT_DIR=/tmp bash scripts/session-message-selftest.sh assert --session-id no-such-session --sentinel x --timeout-secs 0 > /tmp/.t2876-fc.out 2>&1 || rc=$?; test "$rc" -eq 2

# Fail-closed: prepare must refuse to run without an explicit mode.
rc=0; bash scripts/session-message-selftest.sh prepare --sentinel x > /tmp/.t2876-mode.out 2>&1 || rc=$?; test "$rc" -eq 2

# It must NOT claim guard-layer membership: prepare/cleanup spawn and stop real sessions.
! grep -q "^# guard-layer: source" scripts/session-message-selftest.sh

# The phantom field must not come back undocumented (0 of 56 agents carry it).
bash -c 'if grep -q waitingFor scripts/session-message-selftest.sh; then grep -q "DOES NOT EXIST" scripts/session-message-selftest.sh; fi'

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

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
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

### 2026-09-01T18:47:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2876-cross-session-message-prover-assert-deli.md
- **Context:** Initial task creation
