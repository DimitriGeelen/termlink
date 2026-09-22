---
id: T-3077
name: "Repair arc-011 slice register drift and dispose the duplicate task pair"
description: >
  arc-011's slice register records S6 (sender ledger) as status:unbuilt with a note saying a restarted sender loses all knowledge of what it is waiting on. T-3070 shipped that slice and is in .tasks/completed/. The register that exists specifically to make slice status a structural query rather than archaeology is currently asserting something false about its own arc. Also dispose T-3073/T-3074, the orphan duplicates created when two fw task create --type inception calls failed silently and were re-run under fresh IDs; the register points at T-3071/T-3072.

status: work-completed
workflow_type: build
owner: claude-code
horizon: null
tags: [arc:arc-011]
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
created: 2026-09-22T16:06:55Z
last_update: 2026-09-22T16:13:48Z
date_finished: 2026-09-22T16:13:48Z
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

# T-3077: Repair arc-011 slice register drift and dispose the duplicate task pair

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] arc-011 slice S6 reads `status: built` and its note states what T-3070 actually
      shipped, replacing the note claiming a restarted sender loses all knowledge of
      what it is waiting on — which stopped being true when T-3070 landed
- [x] Every slice in `.context/arcs/arc-011.yaml` whose `task:` is in `.tasks/completed/`
      reads `built` or `partial`, never `unbuilt`. Checked mechanically, not by eye
- [x] T-3073 and T-3074 are disposed as duplicates: each names its canonical twin
      (T-3071 / T-3072) in-file, and neither remains a live candidate for work
- [x] Every `task:` reference in the arc's slice register resolves to a task file that
      exists in either `.tasks/active/` or `.tasks/completed/`
- [x] `bash scripts/check-arc-claim-drift.sh` exits the same as it did before this
      change — no regression in the existing arc guard. Its exit 1 is the pre-existing
      arc-004 UNBOUND firing; the guard evaluates CLOSED arcs only (`closed: 2` —
      arc-004 and arc-003), and arc-011 is `in-progress`, so this edit provably cannot
      reach its verdict. `git status .context/arcs/` shows arc-011.yaml as the only
      changed file in that directory

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

# AC1 — S6 reads built, and its note no longer carries the claim T-3070 falsified.
python3 -c "import yaml,sys; s=[x for x in yaml.safe_load(open('.context/arcs/arc-011.yaml'))['slices'] if x['id']=='S6'][0]; sys.exit(0 if s['status']=='built' and 'notify-ledger.sh' in s['note'] and 'loses all' not in s['note'] else 1)"

# AC2 + AC4 — no slice whose task is COMPLETED still reads 'unbuilt', and every
# task: reference in the register resolves to a real file. Fail-closed: a register
# that does not parse, or that yields zero slices, exits non-zero rather than
# vacuously passing (T-2831 — "nothing to check" must not read as "all clear").
python3 -c "import yaml,glob,sys; D=lambda t: bool(glob.glob('.tasks/completed/%s-*.md'%t)); L=lambda t: bool(glob.glob('.tasks/active/%s-*.md'%t)); sl=yaml.safe_load(open('.context/arcs/arc-011.yaml'))['slices']; sys.exit(2) if not sl else None; bad=[s['id']+':'+s['task'] for s in sl if (D(s['task']) and s['status']=='unbuilt') or not (D(s['task']) or L(s['task']))]; print('slice-drift: '+(', '.join(bad) or 'none')); sys.exit(1 if bad else 0)"

# AC3 — both duplicates are retired out of the live candidate set and each names
# its canonical twin. Checks absence from active/ as well as presence in completed/,
# because presence alone would pass while a stray copy still sat in the backlog.
test -z "$(ls .tasks/active/T-3073-*.md .tasks/active/T-3074-*.md 2>/dev/null)"
grep -q 'related_tasks: \[T-3071\]' .tasks/completed/T-3073-message-queue-priority-field---flat-fifo.md
grep -q 'related_tasks: \[T-3072\]' .tasks/completed/T-3074-urgent-flag-bypass-the-prompt-free-wait-.md

# AC5 — the existing arc guard still runs and still reports arc-011 outside its
# scope. Asserted on the SCOPE claim, not on the exit code: the guard exits 1 on a
# pre-existing arc-004 finding that has nothing to do with this change, so gating on
# rc would be a permanently-red check (T-2818).
bash scripts/check-arc-claim-drift.sh > /tmp/.t3077-arcguard.out 2>&1 || true
grep -q 'closed: 2' /tmp/.t3077-arcguard.out
grep -q 'says nothing about in-progress arcs' /tmp/.t3077-arcguard.out

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

### 2026-09-22 — the register that exists to stop archaeology needed archaeology

- **What changed:** The repair itself was one field. What the repair exposed is that
  **nothing detects this class of drift**, and the omission sits precisely where it
  costs most. arc-011's slice register was introduced (T-3066) with an explicit
  rationale: *"A slice with no task is now a structural query, not an archaeology
  exercise."* It was built because arc-003 closed asserting "no silent loss" and a
  silent-loss path was proven 82 days later, and because a deferred slice with no
  field to live in survived only as the word "future" inside a task that completed,
  and vanished. The register answers "which slices exist and what is their status"
  — but its status field is **hand-maintained**, and a slice goes stale the moment
  its task completes without someone remembering to come back. T-3070 completed and
  S6 kept reading `unbuilt`, with a note asserting something T-3070 had made false.
- **Why that is the expensive direction:** the drift under-claims. A register that
  says `unbuilt` about shipped work invites the work to be done twice — the exact
  duplicate-effort class T-2800 documents, reached by a different route. And it is
  silent: `check-arc-claim-drift.sh` exists and passes over this cleanly, because it
  judges CLOSED arcs for prover bindings and says so in its own scope disclaimer —
  *"it says nothing about in-progress arcs."* The guard was right; it was simply
  never asked this question. That is the T-2680 shape: a green that is narrower than
  it reads.
- **Plan impact:** none to the arc's slice ordering. The correction is factual —
  arc-011 is now 3 built (S3, S4, S6) + 1 partial (S5), not 2 built. It does mean
  the register cannot be trusted as a *current* status read without a cross-check,
  which is what the verification block on this task now performs.
- **Where the check lives, and its honest limit:** T-3077's `## Verification` carries
  the cross-check (every slice whose `task:` is in `.tasks/completed/` must not read
  `unbuilt`; every `task:` must resolve; zero slices is a refusal, not a pass). It is
  mutation-proven — reverting S6 to `unbuilt` fires it. But it runs **only when this
  task completes**, which is a one-shot, not a standing guard. The next arc to drift
  drifts unwatched. Promoting it to a `scripts/check-arc-slice-drift.sh` sibling that
  walks every in-progress arc is the real G-019 close, and is deliberately NOT done
  here: it is new guard-layer surface and a separate deliverable, offered rather than
  assumed.
- **Triggered:** T-3073 and T-3074 retired as duplicates (see their own Evolution
  entries for the `fw task create --type inception` silent-failure mechanism that
  produced them). Follow-up candidate: standing slice-drift check across all
  in-progress arcs.

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

### 2026-09-22T16:06:55Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3077-repair-arc-011-slice-register-drift-and-.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-f5d1b9e9
- **Timestamp:** 2026-09-22T16:13:50Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 80
     - evidence: `bash scripts/check-arc-claim-drift.sh > /tmp/.t3077-arcguard.out 2>&1 || true`

### 2026-09-22T16:13:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** arc-011 slice S6 repaired to built with an accurate note; T-3073/T-3074 retired as duplicates of T-3071/T-3072; slice/task cross-check added and mutation-proven.
