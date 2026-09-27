---
id: T-3095
name: "D8 regression: handover 5 TODO sections persist despite T-3015 mechanism fix"
description: >
  arc-008 cycle-3 audit re-run: D8 still FAILs identically (5 [TODO] sections) even
  though T-3015 (owner: agentagent, work-completed 2026-09-20) was filed specifically
  as the MECHANISM fix distinguishing this from the T-2941 instance-only close. Per
  arc-008's own rule ('verification of a task is the re-run of the audit, not self-assertion'),
  this re-run says T-3015's fix did not hold. Link: T-2941, T-2942, T-3015.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-008]
components: [tests/bvp-derived-blast-radius-fixtures.sh]
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
created: 2026-09-24T23:53:29Z
last_update: 2026-09-27T22:26:43Z
date_finished: 2026-09-27T22:26:43Z
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
  - ts: '2026-09-25T00:06:50Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 3
      D4: 2
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=3
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-25T00:07:07Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-27T21:34:08Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (1-file-ref-derived-T-3189); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3095: D8 regression: handover 5 TODO sections persist despite T-3015 mechanism fix

## Context

arc-008 cycle-2 census already diagnosed this exact FAIL and filed T-3015 as the mechanism fix (not merely T-2941's instance-only close). This cycle-3 re-run is the verification arc-008's own rule calls for -- and it says T-3015 did not hold.

## Diagnosis (AC1)

**T-3015 was not a mechanism fix.** Its four commits (`17191511d`, `64523bdf5`,
`a6f79ffb1`, and its share of `10580e6c5`) touched **21 files and not one of them is the
generator or the check**. Enumerated from git rather than from its Updates section, as this
AC requires: task files, `.context/pickup/` filings, `learnings.yaml`, one episodic, and
**exactly one handover** — `S-2026-0920-1235.md`. That is an instance repair plus an
upstream filing (`framework:pickup@126`). The mechanism was correctly identified and
correctly routed; it was never locally changed, because both files are vendored (G-062).
So "T-3015 did not hold" is not a regression — **nothing was ever in place to hold.**

**The mechanism itself has three layers, and only the third is new.**

1. **The generator emits 4 unfilled placeholder sections, by design and correctly.**
   T-2882's rule is that the generator cannot know the session narrative, so an unfilled
   section must read as unfilled rather than be fabricated. This is right, and it is not
   the defect.
2. **The check counts the generator's own meta-comment.** `handover.sh:712` contains an
   instructional line that itself carries the literal marker, and `audit.sh:5086` is a raw
   `grep -c` over the whole file. So every generated handover starts at a floor of 1 —
   T-2943 measured this and concluded D8's `pass` branch is unreachable dead code.
   4 sections + 1 comment = **5**, against a FAIL threshold of `>3`. **Every freshly
   generated handover FAILs D8 by construction.**
3. **PreCompact mints a fresh handover and repoints `LATEST.md` at it.** This is the layer
   that makes enrichment futile rather than merely manual. `LATEST.md` is a *symlink*, so
   the moment a new handover is generated the enriched one stops being the file D8 reads.

**Live evidence from this session, not inference.** A previous session enriched
`S-2026-0927-2110` — commit `85d2c0f36`, "all four sections filled". `/compact` then fired,
PreCompact generated and auto-committed `S-2026-0927-2222` un-enriched (`5e0a6045a`),
`LATEST.md` followed it, and D8 was red again within the minute. **Enrichment is Sisyphean
by construction**, and that is the finding T-3015 could not have had, because it enriched
and then stopped watching.

**A fourth, smaller trap, reproduced here while writing this up.** Prose *about* the marker
counts too. The first draft of the Gotchas section in the enriched handover pushed the
tally from 1 back to 3 by quoting the marker twice. T-2943 documented exactly this; it is
the same comment-vs-code shape that has now defeated six detectors in this repo. The
enriched handover therefore describes the marker without spelling it.

**Disposition of the mechanism.** It is upstream's (filed `@126`, not landed). `T-3021`
nominally carries the PreCompact half — its title is precisely "Handover generator +
PreCompact auto-commit mint a fresh D8/D8b FAIL every…" — but it is an **empty stub with no
Context and no ACs**, one of the three unwanted pickup round-trip artifacts T-3015's own
evolution log flagged as "needing disposition". So the mechanism is currently owned by
nothing. Raised as a Sovereign question rather than resolved here, because every available
fix is an architectural change to vendored code: change PreCompact's behaviour, make
enrichment automatic, or change what D8 considers a failure.

**Scope note.** This task is D8. D8b still FAILs (6/10 recent handovers) and is **T-3096**,
deliberately untouched here.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Diagnose why T-3015's change did not prevent this handover's TODO count from being 5 again (read T-3015's actual diff, not just its Updates section)
      → See `## Diagnosis (AC1)`. T-3015's 4 commits touched 21 files, **none** of them the
      generator or the check — enumerated from `git show --name-only`, not its Updates
      section. It was an instance repair plus an upstream filing. Three-layer mechanism
      identified, the third layer (PreCompact repoints the `LATEST.md` symlink) being the
      one that makes enrichment futile rather than merely manual, evidenced live this
      session: `85d2c0f36` enriched, `5e0a6045a` replaced it, D8 red again within the minute.
- [x] fw audit's D8 check no longer FAILs (handover LATEST.md TODO-section count is at or below the check's threshold-floor, per T-2943)
      → `fw audit --section discovery` now reports
      `[WARN] D8: Handover quality — LATEST.md has 1 [TODO] section(s)`. Was `[FAIL] … 5`.
      **1 is exactly the floor T-2943 identified** (the generator's own meta-comment at
      `handover.sh:712`), so this is the lowest value reachable without patching vendored
      code. FAIL cleared; the residual WARN is structurally unfixable locally.
      Scope: D8b still FAILs (6/10) and belongs to T-3096, untouched here.

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

# T-3095: the original pair here ran a FULL `fw audit` into this same temp path and then
# asserted `! grep -q '[FAIL] D8:'`. It is REMOVED, and that is a strengthening rather than a
# weakening — the assertion it made is strictly implied by the scoped pair below, which adds
# two guards it lacked: `test -s` (the audit actually produced output, so an audit that could
# not run fails instead of passing vacuously — T-3144) and a positive companion proving D8
# itself was evaluated. Both wrote the SAME file, so they were duplicates, not independent
# evidence.
#
# The full audit is also what made this gate unrunnable: it exceeds 300s on this host and the
# closure attempt of 2026-09-28 was OOM-killed by the kernel partway through P-011. D8 is
# emitted only by the `discovery` section, so the scoped run is sufficient as well as cheaper.

# AC2: the count D8 reads must be at or below T-2943's floor of 1. This asserts the
# NUMBER, not the absence of a FAIL string, so it cannot pass because a grep missed.
test "$(grep -c '\[TODO' .context/handovers/LATEST.md)" -le 1

# AC2, through the check itself rather than my own count: D8 must not be a FAIL.
# `|| true` is deliberate and is NOT the T-3144 anti-pattern. The audit exits 2 whenever
# ANY check fails, and D8b legitimately fails here (6/10, owned by T-3096) — so the
# producer's exit code cannot be the signal for a D8-specific assertion. The guard T-3144
# asks for is supplied instead by `test -s` (the run produced output) plus the positive
# companion on the next line (D8 itself was evaluated), so "no FAIL" can never mean
# "the check never ran".
.agentic-framework/bin/fw audit --section discovery > /tmp/.t3095-audit.out 2>&1 || true
test -s /tmp/.t3095-audit.out && ! grep -q '^\[FAIL\] D8:' /tmp/.t3095-audit.out

# Positive companion for the absence assertion above: prove D8 actually ran and was
# evaluated, so "no FAIL" cannot mean "the check never executed".
grep -q '^\[WARN\] D8: Handover quality' /tmp/.t3095-audit.out

# AC1 is a written diagnosis; assert the load-bearing claim is recorded, excising this
# Verification block first so the command's own text cannot satisfy the pattern
# (the vacuous self-match T-3015 caught in its own draft check).
sed '/^## Verification/,/^## RCA/d' .tasks/active/T-3095-d8-regression-handover-5-todo-sections-p.md > /tmp/.t3095-body.md && grep -q 'not one of them is the' /tmp/.t3095-body.md

## RCA

**Symptom:** `fw audit` D8 reported `[FAIL] D8: Handover quality — LATEST.md has 5 [TODO]
sections` on the arc-008 cycle-3 re-run, identically to cycle-2, despite T-3015 having been
filed and closed specifically as the *mechanism* fix distinguishing it from T-2941's
instance-only close.

**Root cause:** T-3015 never changed the mechanism. Its four commits touched 21 files, none
of them `agents/handover/handover.sh` or `agents/audit/audit.sh`; it repaired one handover
instance and filed the defect upstream at `framework:pickup@126`. Both files are vendored,
so a local fix was correctly out of scope — but the task was recorded as the mechanism fix
while the mechanism had only been *reported*. Underneath that, the defect is three-layered:
the generator emits 4 unfilled placeholder sections by design (T-2882, correct); the check
additionally counts the generator's own instructional comment at `handover.sh:712`, putting
the floor at 1 (T-2943); and PreCompact generates a fresh handover and repoints the
`LATEST.md` **symlink** at it, so 4+1=5 against a `>3` threshold means every freshly
generated handover FAILs, and any enrichment stops being the file D8 reads at the next
`/compact`.

**Why structurally allowed:** arc-008's own rule — "verification of a task is the re-run of
the audit in the next cycle, not self-assertion" — worked exactly as designed: the re-run
caught it. What the framework has no state for is **"defect reported upstream, not yet
landed here."** `.vendor-divergence.yaml` tracks that lifecycle (`local-only` →
`filed-upstream` → `landed-upstream`) but only for divergences we *did* patch locally. A
defect we deliberately did **not** patch has no register entry, so when it closes as a task
it becomes invisible (CLAUDE.md: "completed tasks archive and become invisible"). The next
audit cycle then sees a FAIL with no open task and files a fresh one. **This defect has now
been filed four times for one cause: T-2941 (instance) → T-2943 (mechanism, WARN leg) →
T-3015 (mechanism, filed upstream) → T-3095 (this).** The re-filing is not sloppiness; it
is the register behaving as built.

**Prevention:** registered as **G-094** in `.context/project/concerns.yaml` — a gap persists
in the register, is visible in Watchtower and is checked by the audit, which is precisely the
property a closed task lacks. That converts the next cycle's rediscovery into a known
watched item instead of a fifth task. Note the narrower prevention that is *not* available:
raising D8's threshold or excluding the generator's comment would make the check pass, but
the first weakens a gate and the second still leaves 4 > 3 — so there is no local code change
that makes D8 green, which is the substance of the Sovereign question this task raises.

**Finding recorded en route (not fixed here):** the audit's own mitigation string for the
handover section tells the operator to "Register via `fw gaps add`". **There is no `fw gaps
add` verb** — `bin/fw` routes only bare `gaps` (show) and `gaps close`. So the register had
to be edited directly, which is the documented path in CLAUDE.md §"When discovering
structural flaws" but means the audit prescribes an unactionable command. Same class as
T-2958 (`fw task review` hardcodes `go`) and the missing `checkpoint.sh budget` subcommand
the `/resume` skill calls: the framework advising a verb it does not ship.

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

### 2026-09-27 — the mechanism has a third layer, and it is the one that matters

- **What changed:** at filing this looked like "T-3015's fix regressed". It had not regressed,
  because it was never a code change: 21 files across four commits, none of them the generator
  or the check. The layer nobody had named is that **PreCompact repoints the `LATEST.md`
  symlink** at each freshly generated handover, so enrichment stops being the file D8 reads.
  T-2943 had the floor-of-1 arithmetic and T-3015 had the upstream filing; neither had the
  symlink, which is why "enrich it" kept looking like a fix and kept not being one.
- **Plan impact:** AC2 ("D8 no longer FAILs") is satisfiable, but only transiently, and the
  task must say so rather than imply a mechanism close. Closing this without that sentence
  would have reproduced T-3015's error one cycle later.
- **Triggered:** G-094 (no register state for "reported upstream, not yet landed"), and the
  observation that T-3021 — which nominally owns the PreCompact half — is an empty stub.

### 2026-09-28 — the closure was OOM-killed mid-gate, and the near-miss is the finding

- **What changed:** the first `--status work-completed` run was killed by the kernel partway
  through P-011 (host low on memory; this box runs ~450 concurrent agent processes). Cause: a
  pre-existing verification line invoking a **full** `fw audit`, which exceeds 300s here.
- **Plan impact:** that line was removed as a strictly weaker, same-temp-path duplicate of the
  scoped `--section discovery` pair, which additionally guards `test -s` and carries a positive
  companion. Recorded in the block itself so the removal is auditable rather than silent.
- **Triggered:** a concrete near-miss for **T-2833**. `update-task.sh` writes
  `status: work-completed` unconditionally ~221 lines *before* the finalize block, and the
  guard then reads the value the first write committed — so a process killed *between* them
  latches the task permanently out of finalization. Verified after the kill: T-3095 was still
  `started-work` with `date_finished: null` and
  `check-stranded-finalized-tasks.sh` reported 0 stranded of 328, i.e. **the kill landed inside
  the gate rather than inside the window, by timing alone.** T-2833's detector is local and its
  fix is upstream's; this is the first observed instance of the killing condition actually
  occurring, as opposed to being reasoned about.

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

### 2026-09-24T23:53:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3095-d8-regression-handover-5-todo-sections-p.md
- **Context:** Initial task creation

### 2026-09-27T21:50:40Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-18f2c905
- **Timestamp:** 2026-09-27T22:27:06Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 83
     - evidence: `.agentic-framework/bin/fw audit --section discovery > /tmp/.t3095-audit.out 2>&1 || true`

### 2026-09-27T22:26:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
