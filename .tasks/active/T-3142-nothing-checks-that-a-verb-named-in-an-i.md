---
id: T-3142
name: "Nothing checks that a verb named in an instruction actually resolves — and
  a naive checker reports checkout defects as framework defects"
description: >
  Two surfaces verified from the main checkout name a verb this build does not have:
  checkpoint.sh budget (prescribed by the /resume skill as the G-087-safe budget read;
  usage at checkpoint.sh:458 is post-tool|reset|status) and fw sidecar send (prescribed
  by AEF's SIDECAR-E2E run ab947312; not in the verb table). In both the DOCUMENTED
  path is absent while an undocumented one works, so the failure is invisible to anyone
  following the documentation - and in the checkpoint case the absent path is the
  SAFE one, making the docs strictly worse than ignoring them. A third candidate,
  fw integrate run, was filed and then RETRACTED (framework:pickup offset 160): it
  exists and runs, with 28 refs in bin/fw. Every measurement behind that claim had
  been taken from inside a git worktree whose vendored tree differed from main's,
  so the readings were true of where I stood and false of the framework - the T-2817
  dangling-reference class biting the report rather than the code. That retraction
  is the design constraint for this check: it MUST declare which framework root it
  resolved against on every output path, because a verb-resolution check that does
  not is one worktree away from confidently reporting a checkout defect as a framework
  defect. Filed upstream; the check is the local detection.

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
created: 2026-09-25T11:25:00Z
last_update: 2026-09-25T16:42:17Z
date_finished:
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
  - ts: '2026-09-25T11:25:18Z'
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

# T-3142: Nothing checks that a verb named in an instruction actually resolves — and a naive checker reports checkout defects as framework defects

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] **UNPARKED 2026-09-25.** The `# guard-layer: source` marker is restored and the runner
      adopts it as a `static-check`; it no longer appears under "unclassified". The park
      held while it stood: the previous AC text read "FAILED — PARKED … marker withheld",
      and it is replaced rather than quietly reworded, because the parked state was real
      and the record of it is the point.
- [x] **Every output path declares the framework root it resolved against** — the retraction
      at offset 160 happened because measurements taken inside a worktree were reported as
      framework facts; a check without this repeats it
- [x] It resolves `fw <verb>` references found in instructional surfaces against the live
      verb table, and fires on one that does not resolve
- [x] **Load-bearing:** it fires on a fixture instructing `fw sidecar` (genuinely absent,
      verified today) and does NOT fire on `fw integrate` (genuinely present, 28 refs — the
      one I wrongly retracted). Both measured: `fw sidecar` → rc 1 naming the verb,
      `fw integrate` → rc 0. This is the assertion that failed and caused the park.
- [x] Fail-closed: an unresolvable `fw` binary, or a verb table that comes back empty,
      exits 2 — never a clean bill (an empty table would clear every reference vacuously)
- [x] **Reference floor (the parked defect).** Too few references on the default surface
      exits **2**, not 0. "Found nothing to check" and "everything checks out" no longer
      share an exit code. Proven by a mutant that kills the anchor: it drops to 4
      references — the exact number the broken script reported as *clean* — and the floor
      now refuses instead.
- [x] **The anchor defect is fixed and pinned.** The backtick branch was written `` \` ``,
      which GNU ERE reads as the start-of-buffer anchor, so it never matched. Corrected to
      a literal backtick; the real surface went from **4 references to 10** — the check had
      been passing because it was reading 40% of its subject.
- [x] Fixtures cover firing, clean, the false-positive guard, both fail-closed paths, the
      floor, the ROOT line on both output paths, the JSON envelope, and the dead-anchor
      mutant. 12/12.

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

# The suite, including the dead-anchor mutant and both fail-closed paths.
bash tests/instructed-verb-resolves-fixtures.sh > /tmp/.t3142-fix 2>&1 && grep -q "12 passed, 0 failed" /tmp/.t3142-fix

# UNPARKED: the marker is restored in the COMMITTED script and the runner adopts it.
git show HEAD:scripts/check-instructed-verb-resolves.sh > /tmp/.t3142-head 2>&1 && grep -q "^# guard-layer: source" /tmp/.t3142-head
bash scripts/run-guard-layer.sh --list > /tmp/.t3142-list 2>&1 && grep -q "static-check   check-instructed-verb-resolves.sh" /tmp/.t3142-list
grep -q "fixture-suite  instructed-verb-resolves-fixtures.sh" /tmp/.t3142-list

# The backtick is LITERAL, not the GNU start-of-buffer anchor. One character; the whole defect.
grep -q "grep -oE '(\`fw |bin/fw |" /tmp/.t3142-head

# The anchor sees the surface it was blind to: 10 references, not the 4 it reported broken.
bash scripts/check-instructed-verb-resolves.sh --json > /tmp/.t3142-json 2>&1
python3 -c "import json; d=json.load(open('/tmp/.t3142-json')); assert d['checked']==10, d['checked']; assert d['ok'] is True, d; print('10 references, all resolve')"

# THE LOAD-BEARING ONE, run against the real GNU grep a script gets, not the shell's ugrep alias:
# a surface naming an absent verb must FIRE. This is the assertion that failed and got it parked.
printf 'Run `fw sidecar` now.\n' > /tmp/.t3142-fx/probe.md 2>/dev/null || { mkdir -p /tmp/.t3142-fx && printf 'Run `fw sidecar` now.\n' > /tmp/.t3142-fx/probe.md; }
bash scripts/check-instructed-verb-resolves.sh --dirs /tmp/.t3142-fx > /tmp/.t3142-probe 2>&1; test $? -eq 1
grep -q "sidecar" /tmp/.t3142-probe

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

### 2026-09-25T11:25:00Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3142-nothing-checks-that-a-verb-named-in-an-i.md
- **Context:** Initial task creation

### 2026-09-25T11:25:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work


## Failure record — parked after the load-bearing test failed

**Two defects, both found by this task's own load-bearing AC rather than by review.**

1. **VACUOUS PASS.** Given a fixture naming `fw sidecar` — verified absent from the
   verb table the same day — the check reported `clean — 0 reference(s) all resolve`
   and exited **0**. There is a floor on the VERB TABLE size (≥10, else exit 2) but
   **none on the REFERENCE count**, so "found nothing to check" and "everything
   checks out" share an exit code. That is precisely the T-2831 shape this repo has
   a dozen guards against — reproduced inside a check whose entire subject is things
   that look right and are not.
2. **`VERB_CHECK_DIRS` override does not take effect.** The fixture directory was not
   scanned at all, which is what produced (1).

   **CORRECTION (2026-09-25, on unparking) — defect 2 as written above is WRONG.** The
   override works, and always did; `--dirs` and `VERB_CHECK_DIRS` both reach the scan
   loop, which was confirmed by tracing the script rather than inferring from the
   symptom. The real cause of the vacuous pass was **one character in the anchor**: the
   backtick branch was written `` \` ``, and GNU ERE reads `\`` as the **start-of-buffer
   anchor**, not a literal backtick. So the branch could never match anything, and every
   backticked `` `fw x` `` reference in the corpus was invisible.

   That is why the fixture reported zero: its only reference was backticked. It is also
   why the real surface reported `clean — 4 references` when the true figure is **10** —
   the check was passing because it was looking at 40% of its subject.

   **The diagnosis was wrong because of a tooling accident worth remembering.** This
   host's interactive shell has `grep` aliased to **ugrep**, which treats `` \` `` as a
   literal backtick. Every by-hand verification of the anchor therefore PASSED while the
   script — which gets `/usr/bin/grep` — was broken. Three hypotheses were wrong in a row
   here (backtick regex, then heredoc re-expansion, then back to the backtick) before
   printing the pattern bytes and the engine name settled it. A check verified by hand in
   a shell whose tools differ from the script's is not verified.

**Why it is parked and not patched:** the acceptance criteria failed twice — three
anchor tightenings (13 → 4 → 6 findings, all prose) and then the vacuous pass. The
rule this repo applies to everything else applies here: stop, record the failure mode,
move on. A third round on my own check, at the end of a long session, is how a
permanently-green guard gets shipped.

**What is worth keeping when this resumes**

- The **ROOT line** works and is the real contribution: every output path names the
  `fw` binary and cwd it resolved against. It exists because the upstream retraction
  at `framework:pickup` offset 160 was caused by measurements taken inside a worktree
  being reported as framework facts.
- The **surface narrowing** is sound: `.claude/commands` are instructions; shell-script
  comments are documentation, where `fw` appears as a discussed PATH, not a cited
  COMMAND. Widening back to `scripts/` needs a way to tell those apart.
- **The fix for (1) is known and small:** a reference-count floor, mirroring the
  verb-table floor already present — zero references must exit 2, never 0.
