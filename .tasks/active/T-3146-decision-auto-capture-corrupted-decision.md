---
id: T-3146
name: "Decision auto-capture corrupted decisions.yaml — unquoted scalar carrying a colon-space (T-3084 class)"
description: >
  Decision auto-capture corrupted decisions.yaml — unquoted scalar carrying a colon-space (T-3084 class)

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
created: 2026-09-25T14:17:14Z
last_update: 2026-09-25T14:17:14Z
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

# T-3146: Decision auto-capture corrupted decisions.yaml — unquoted scalar carrying a colon-space (T-3084 class)

## Context

Completing T-3145 auto-captured its `## Decisions` section into
`.context/project/decisions.yaml` and left the file unparseable. `fw audit` FAILed and the
pre-push hook refused the push — **the detection worked**; this task is about the write.

**The task title is wrong and is kept as filed for traceability.** It says "unquoted scalar
carrying a colon-space (T-3084 class)". That was my hypothesis before I looked, formed
because my decision text contains `# guard-layer: source`. It is **not** what happened: the
emitted scalar is correctly double-quoted. Three different defects are involved, and I would
have "fixed" a quoting bug that was not there.

Measured in the emitted entry:

1. **Indentation.** 168 entries sit at column 0 (`- id: PD-167`); the new one is indented two
   spaces (`  - id: PD-001`). That is the parse error — a block mapping cannot suddenly
   become a nested sequence.
2. **ID counter reset.** It emitted `PD-001` while the file already holds PD-001 (line 164)
   through PD-168. A duplicate ID is the quieter defect of the two: once the indentation is
   repaired the file parses, and two different decisions answer to one name.
3. **Truncation.** The captured text is `"acknowledge \`lib/build.sh\` in a git-tracked
   ledger, counted and reported on the"` — cut mid-sentence at the end of the first physical
   line of a wrapped `- **Chose:**` bullet. The rest of the decision was silently dropped.

Defect 3 is why this is not merely cosmetic. A decisions register that truncates at the first
newline records a *plausible* decision rather than the decision — the reader has no signal
that anything is missing, which is the Directive #2 shape.

All three are in **vendored** code (the capture path under `.agentic-framework/`), so per
G-062 the data is repaired here and the code defect is filed upstream, not patched.

## Acceptance Criteria

### Agent
- [x] `.context/project/decisions.yaml` parses as YAML again, and `fw audit --section
      structure` no longer reports the parse FAIL.
- [x] The repaired entry is at column 0 like its 168 siblings, carries the next free id
      (`PD-169`, not a duplicate `PD-001`), and holds the FULL decision text rather than the
      truncated first line.
- [x] No other entry is altered — the repair touches exactly the one broken record, verified
      by diffing entry ids before and after.
- [x] The duplicate-id condition is checked explicitly after the repair (no id appears twice),
      since defect 2 survives a successful parse and would otherwise go unnoticed.
- [x] All three defects are filed upstream on `framework:pickup` with the measured evidence,
      and the filing states plainly that the colon-space/quoting hypothesis was WRONG — a
      filing that sends the maintainer after the wrong defect is worse than none.
- [x] The filing is verified by read-back from the topic, not by the sender's own
      `delivered` status (T-2876: delivered is not received).

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

# Parses, no duplicate ids, and the repaired entry carries the FULL text not the
# truncated 79 chars. Asserted against the COMMITTED file (T-3086), not the worktree.
git show HEAD:.context/project/decisions.yaml > /tmp/.t3146-dec.yaml 2>&1
python3 -c "import yaml,collections,sys; d=yaml.safe_load(open('/tmp/.t3146-dec.yaml'))['decisions']; ids=[e['id'] for e in d]; dup=[k for k,v in collections.Counter(ids).items() if v>1]; assert not dup, dup; e=[x for x in d if x['id']=='PD-169'][0]; assert e['task']=='T-3145', e; assert len(e['decision'])>400, len(e['decision']); print('register ok:',len(d),'entries, no dupes, PD-169 len',len(e['decision']))"

# The broken shape is gone: no entry is indented under the top-level sequence.
test "$(grep -c '^  - id: PD-' /tmp/.t3146-dec.yaml)" -eq 0

# The audit no longer reports the parse failure.
.agentic-framework/bin/fw audit --section structure > /tmp/.t3146-audit 2>&1; grep -q "Fail: 0" /tmp/.t3146-audit

# The filing is on the topic — read back, not trusted from the sender (T-2876).
termlink channel subscribe framework:pickup --cursor 164 --limit 1 --json > /tmp/.t3146-rb 2>&1 && grep -q "T-3146" /tmp/.t3146-rb

## RCA

**Symptom:** `.context/project/decisions.yaml` unparseable after a task completed; `fw audit`
FAIL, pre-push refused.

**Root cause:** three independent defects in the vendored decision auto-capture write path —
the entry is emitted indented two spaces where its 168 siblings are at column 0 (the parse
error), the id counter restarts at `PD-001` instead of continuing past `PD-168` (a duplicate
id), and the captured text stops at the first physical line of a soft-wrapped markdown bullet
(79 characters of a 556-character decision).

**Why structurally allowed:** the capture appends to a YAML file and never reads it back. A
two-line parse assertion at the write site would have converted all three into a refusal at
the moment of writing, naming the cause. Instead the failure surfaces one step later, at
someone else's push, as "fix the YAML" with no indication of what wrote it. Note the split:
defect 1 is loud and got caught; defect 3 is silent and would NOT have been — a decision whose
first line happens to be a complete sentence truncates invisibly and no gate ever fires.

**Prevention:** the data repair is not prevention and is not claimed as such. The code is
vendored (G-062), so prevention is upstream's: filed at `framework:pickup` offset 164 with R4
proposing the post-append parse assertion at the write site. What exists locally is the
detection that already worked — `fw audit`'s YAML check plus the pre-push gate — which is why
this was a blocked push rather than a corrupt register discovered weeks later.

**Correction recorded deliberately:** the task title says "unquoted scalar carrying a
colon-space (T-3084 class)". That hypothesis was formed before looking and is WRONG — the
emitted scalar is correctly quoted. The title is kept as filed so the wrong guess stays
visible next to the measurement, and the upstream filing leads with the correction so nobody
hunts a quoting bug that does not exist.

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

### 2026-09-25T14:17:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3146-decision-auto-capture-corrupted-decision.md
- **Context:** Initial task creation
