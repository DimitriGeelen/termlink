---
id: T-2882
name: "Handover generator fabricates narrative sections and a lexicographic-constant Suggested First Action"
description: >
  Handover generator fabricates narrative sections and a lexicographic-constant Suggested First Action

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
created: 2026-09-03T05:31:38Z
last_update: 2026-09-03T05:53:52Z
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

# T-2882: Handover generator fabricates narrative sections and a lexicographic-constant Suggested First Action

## Context

`.context/handovers/LATEST.md` is the first artefact read at every session start
(CLAUDE.md §Session Start Protocol steps 2-3) and at every post-compaction recovery
(SessionStart hook). Measured 2026-09-02: every narrative section in it is a
generator-supplied constant — `Decisions`/`Things Tried`/`Open Questions` say `None`,
`Gotchas` says `See gaps register above.`, and **Suggested First Action is
`Continue T-1457`, byte-identical across the last 68 handovers** (the 916 before that
said `Continue T-1166`). The generator at `handover.sh:1307-1345` sorts started-work
candidates by task id **compared as a string** and prints the first agent-owned one;
the session's actual focus (T-2871) ranks 17th of 123 and is never named. Enrichment
is only an advisory `echo` at `handover.sh:1450`, and the T-136 auto-handover
(`checkpoint.sh:123-161`) runs the script non-interactively and commits, so no agent
is ever in the loop to fill it. The defect is not that the line is wrong but that a
fabricated constant is indistinguishable from a reasoned answer — Directive #2, in
the one line every session is told to act on.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] **The generator stops emitting fabricated narrative.** The five narrative sections
  (`Where We Are` beyond its commit list, `Decisions Made This Session`, `Things Tried
  That Failed`, `Open Questions / Blockers`, `Gotchas / Warnings`) emit an explicit
  `[TODO: ...]` marker instead of `None` / `See gaps register above.`. An unfilled
  section must look unfilled — a reader skimming LATEST.md cannot mistake it for a
  finding.
- [ ] **Handovers carry `enrichment_status`.** Both generation paths — the main block
  (`handover.sh` ~line 704) and the `--checkpoint` block (~line 291) — write
  `enrichment_status: pending` into the frontmatter. This reuses the convention the
  framework already applies to episodic summaries and already reads at
  `handover.sh:495`; it is not a new mechanism.
- [ ] **Suggested First Action is derived from real signal, not string order.** Ranking
  keys on `.context/working/focus.yaml` `current_task` first, then `last_update`
  descending — never lexicographic task id. Run against the live tree, the generator
  must name **whatever `focus.yaml` currently holds**, never `T-1457`.
  (This AC originally pinned the literal `T-2871`, which was the focus when it was
  written; focus has since moved to the tasks doing this work, so the literal would now
  fail for the right behaviour. The invariant — focus wins — is what is asserted, and
  the fixture suite pins it independently of whatever focus happens to be.)
- [ ] **It is labelled as mechanical while unenriched.** While `enrichment_status:
  pending`, the line states it is a mechanical fallback rather than a reasoned
  recommendation, so a session that acts on it knows what it is acting on.
- [ ] **A fixture suite pins all three behaviours and fails against the pre-fix script.**
  Cases: focus present wins; focus absent falls back to most-recently-updated; the
  lexicographic-first task is NOT chosen when it is neither focused nor recent. The
  suite must be red against the current `handover.sh` — a fixture that passes both
  before and after proves nothing (T-2814 lesson).
- [ ] **The local divergence is registered so a re-vendor cannot silently delete it.**
  `.vendor-divergence.yaml` gains a `divergences:` entry for this change with
  `status: local-only` and a cited reason, and the file still parses.
- [ ] **It is filed upstream**, since `handover.sh` is vendored (G-062) and a local fix
  has a ~2-month half-life (T-2812). Post the defect + the three-part fix to the
  `framework:pickup` topic and record the offset in this task.
  → Filed 2026-09-03 at **`framework:pickup` offset 83**, carrying the root cause, the
  three-part fix, and the fixture evidence including the pre-fix mutant case.
  `.vendor-divergence.yaml` moved `local-only` → `filed-upstream`. Not yet confirmed
  carried, so the entry stays at risk on a re-vendor until upstream acknowledges.

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
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
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
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

# Both generation paths stamp the enrichment marker (main block + --checkpoint block).
test "$(grep -c 'enrichment_status: pending' .agentic-framework/agents/handover/handover.sh)" -ge 2
# Ranking + placeholder behaviour pinned; suite must be red against the pre-fix script.
bash tests/handover-suggested-action-fixtures.sh > /tmp/.t2882-fix.out 2>&1 && grep -q "ALL PASS" /tmp/.t2882-fix.out
# The local divergence is registered and the register still parses.
python3 -c "import yaml,sys; d=yaml.safe_load(open('.vendor-divergence.yaml')); sys.exit(0 if any('T-2882' in str(e) for e in (d.get('divergences') or [])) else 1)"
# The divergence checker does not report a tooling error (exit 2) on the edited register.
# Condition-context capture: `cmd; test $?` would abort under the gate's `set -e`
# whenever cmd is non-zero — which is exactly the case this line exists to distinguish.
rc=0; bash scripts/check-vendor-divergence.sh > /tmp/.t2882-vd.out 2>&1 || rc=$?; test "$rc" -ne 2

## RCA

**Symptom:** `/resume` and post-compaction recovery report no usable state — every
narrative section of LATEST.md is a constant, and Suggested First Action has read
`Continue T-1457` for 68 consecutive handovers while the real focus was T-2871.

**Root cause:** `handover.sh:1307-1345` ranks started-work candidates by task id
compared as a **string**, so the winner is whichever agent-owned `horizon: now` task
carries the lexicographically smallest id — a constant until that task closes. The
other narrative sections are hard-coded literals (`None`, `See gaps register above.`)
emitted unconditionally.

**Why structurally allowed:** enrichment was never a step, only an advisory `echo`
after generation (`handover.sh:1450`). The T-136 auto-handover
(`checkpoint.sh:123-161`) invokes the script non-interactively and commits the result,
so that advice is printed to nobody. And nothing in the framework compares one
handover to the next, so a value repeating 916 times produced no signal — the
framework had no way to see a constant as a constant.

**Prevention:** distinct from the fix — a guard that fires when LATEST.md is
`enrichment_status: pending` and when Suggested First Action is byte-identical across
N consecutive handovers. Filed as a separate task so it ships and is verified on its
own merits.

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

### 2026-09-03T05:31:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2882-handover-generator-fabricates-narrative-.md
- **Context:** Initial task creation
