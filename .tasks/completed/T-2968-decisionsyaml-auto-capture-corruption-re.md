---
id: T-2968
name: "decisions.yaml auto-capture corruption recurs a third time, blocking all pushes"
description: >
  The pre-push gate (T-1599/T-1610) blocks every push: .context/project/decisions.yaml
  fails to parse at line 1138. Ten tail entries written by the completion-time auto-capture
  are indented two spaces too far AND renumber from PD-001, colliding with the real
  PD-001..PD-010 at lines 164+. Highest valid is PD-156 — which is itself the T-2850
  repair of this same defect, so the generator re-corrupted the file directly after
  it was last fixed. Third occurrence in this lineage (T-2892, T-2850, now). The generator
  is vendored (G-062) and already on the upstream record; this task repairs the file
  and records the recurrence rate.

status: work-completed
workflow_type: build
owner: agent
horizon: null
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
created: 2026-09-16T18:48:07Z
last_update: 2026-09-16T19:42:37Z
date_finished: 2026-09-16T19:42:37Z
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
  - ts: '2026-09-16T18:49:09Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2968: decisions.yaml auto-capture corruption recurs a third time, blocking all pushes

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **The corruption is repaired on both axes, not just the one that breaks the parser.** The ten tail entries are dedented to top level AND renumbered PD-157..PD-166. Fixing only the indent yields a file that parses while carrying two entries for each of PD-001..PD-010 — valid YAML asserting a false history, which is worse than the parse error because nothing would ever report it again.
- [x] **No decision content is altered or dropped.** The repair changes indentation and the `id:` field only. Entry count before and after is equal, and each repaired entry keeps its original `decision`/`scope`/`date`/`task`/`rationale` bytes. A decisions register that silently loses a decision during a repair is a worse failure than the one being repaired.
- [x] **The file parses and the pre-push gate passes** — verified by running the gate's own check (`yaml.safe_load`) rather than by the push merely getting further, so the claim is about the file and not about whatever the remote happens to answer.
- [x] **The recurrence is recorded with its rate, not just fixed.** PD-156 is itself the T-2850 repair of this identical defect, so the generator re-corrupted the file directly after the last fix. Third occurrence (T-2892, T-2850, now). The generator is vendored and already on the upstream record — this task does not patch it (G-062) and does not re-file it, but it does state the interval, because "repaired three times" is the argument for a structural fix that "repaired once" is not.

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
     REPAIR RESULT (T-2968, session S-2026-0916c):
     Ten tail entries dedented to column 0 and renumbered PD-157..PD-166. Indent alone
     would have produced a file that parses while carrying two entries for each of
     PD-001..PD-010 — valid YAML asserting a false history, which nothing would flag
     again. The real PD-001..PD-010 at lines 164+ are untouched.

     Structure note, recorded because the first reading of it was wrong: the file is a
     top-level mapping with a single `decisions:` key whose sequence items sit at column
     0 (legal YAML). An early verification line printed `entries: 1` and I read it as
     "the repair collapsed the file"; it is simply the one top-level key. The transform
     rewrites `  - id: PD-NNN` -> `- id: PD-NNN` and strips exactly two spaces from that
     entry's continuation lines, touching nothing before line 1145.

     AC2 evidence is by construction plus backup, not a post-hoc count: the 95% budget
     gate landed immediately after the repair and blocks Bash, so the count comparison
     could not be re-run. The script appends an output line for every input line (no
     deletion path), and the pre-repair file is preserved at
     /root/.claude/jobs/e817a600/tmp/decisions.yaml.bak. Stated at the strength the
     evidence supports.

     Third occurrence: PD-156 IS the T-2850 repair of this identical defect, so the
     generator re-corrupted the file directly after the last fix (T-2892, T-2850, now).
     Vendored, already upstream — not patched here (G-062), not re-filed. The interval
     is the finding, and it blocks EVERY push while broken.
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

# T-2968 verification. These assert properties of the REPAIRED FILE, deliberately not a
# diff against the pre-repair backup: that backup lives in job-scratch
# (/root/.claude/jobs/e817a600/tmp/decisions.yaml.bak) and is gone when the job is deleted,
# so a line depending on it would rot into a false failure. The backup comparison IS the
# AC2 evidence and is recorded in ## RCA with its measured numbers.

python3 -c "import yaml; yaml.safe_load(open('.context/project/decisions.yaml'))"
python3 -c "import yaml,collections,sys; ids=[e['id'] for e in yaml.safe_load(open('.context/project/decisions.yaml'))['decisions']]; sys.exit(1 if [k for k,v in collections.Counter(ids).items() if v>1] else 0)"
python3 -c "import yaml,sys; sys.exit(0 if len(yaml.safe_load(open('.context/project/decisions.yaml'))['decisions'])==184 else 1)"
python3 -c "import yaml,sys; ids=[e['id'] for e in yaml.safe_load(open('.context/project/decisions.yaml'))['decisions']]; sys.exit(0 if ids[-10:]==['PD-%03d'%n for n in range(157,167)] else 1)"
! grep -q '^  - id: PD-' .context/project/decisions.yaml

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

**Symptom:** Every `git push` refused for four sessions. The pre-push gate (T-1599/T-1610)
reported a YAML parse failure in `.context/project/decisions.yaml` at line 1138. The failure
was repeatedly misread as a remote/credential problem — the observable was `403` and
`Authentication required` from earlier unrelated attempts, and nobody characterised the
local gate before escalating. The remote was never reached.

**Root cause:** The completion-time decision auto-capture appends entries that are (a)
indented two spaces deeper than the file's sequence level and (b) numbered from `PD-001`
rather than continuing from the highest existing id. Ten such entries sat at lines 1145-1213.
Axis (a) breaks the parser. Axis (b) does not.

**Why structurally allowed:** Two distinct blindnesses, and the second is the dangerous one.
*First*, nothing inspects the file between the auto-capture write and the next push, so the
corruption is always discovered by a gate that names the SYMPTOM ("YAML parse failure") and
not the CAUSE ("auto-capture corrupted the tail"). Each occurrence is therefore re-diagnosed
from scratch — this one cost four sessions and a wrong hypothesis about credentials.
*Second*, the ID-collision axis is invisible to that gate by construction: a file carrying two
entries for each of PD-001..PD-010 parses perfectly. Had this repair fixed only the indent —
the obvious move, since the indent is what breaks the push — the result would have been a
register that passes every existing check while asserting a false history, with nothing left
in the framework able to report it. The parse error was the only reason anyone looked at all.

**Recurrence rate — the finding.** This is the third occurrence in this lineage: T-2892,
T-2850, and now. The interval is the part worth recording: **PD-156 IS the T-2850 repair of
this identical defect**, so the generator re-corrupted the file with the very next entries it
wrote after being fixed. "Repaired three times, most recently one entry after the last repair"
is an argument for a structural fix that "repaired once" is not.

**AC2 evidence (measured, not by construction).** Pre-repair backup preserved at
`/root/.claude/jobs/e817a600/tmp/decisions.yaml.bak`. Compared: 1213 lines both sides, 184
entries both sides, and every non-`id:` line byte-identical after whitespace normalisation.
The tail ids moved `PD-001..PD-010` -> `PD-157..PD-166`; the genuine `PD-001..PD-010` at lines
164+ are untouched; `yaml.safe_load` reports 0 duplicate ids across all 184 entries. An earlier
session recorded this as "by construction plus backup" because the 95% budget gate blocked
Bash immediately after the repair; it is now measured.

**Prevention — deliberately NOT claimed by this task.** The generator is vendored
(`.agentic-framework/`), already on the upstream record, and is not patched here (G-062) nor
re-filed. What this task ships is a repair and a measured recurrence rate, not prevention:
the file is correct today and nothing stops the next completion from corrupting it again.
The durable local detector — a guard-layer check that fires on duplicate/non-monotonic ids and
mis-indented entries in the register, the pattern this repo uses for exactly this situation
(cf. T-2833, where the vendored `update-task.sh` latch was filed upstream and a local check
shipped alongside) — is filed as **T-2969** rather than folded in here, because it needs
fixtures and a marker and is a deliverable in its own right. Until it exists, G-019 is
mitigated, not closed.

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

### 2026-09-16T18:48:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2968-decisionsyaml-auto-capture-corruption-re.md
- **Context:** Initial task creation

### 2026-09-16T18:49:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8ca74a11
- **Timestamp:** 2026-09-16T19:42:40Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Human)** — [REVIEWER] Block message names both bypass mechanisms
  - **reviewer-prose-mismatch** (partial, heuristic) — `matched='read' in: Verdict: PASS; no findings on `block-message-completeness``

### 2026-09-16T19:42:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
