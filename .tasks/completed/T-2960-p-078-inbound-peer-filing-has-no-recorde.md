---
id: T-2960
name: "P-078 inbound peer filing has no recorded disposition"
description: >
  P-078 is a genuine INBOUND filing from the opencode project (T-064 wave-156): a
  cross-check of the T-1976 bare-IP --hub class across their estate, reporting zero
  exposure and corroborating our fix. It sits in .context/pickup/auto-deferred/ as
  one of the four STRANDED envelopes, but unlike the other three it is not one of
  our own outbound records. It is referenced in T-2954 and in the arc-008 census only
  as the TRIGGER that exposed the blind canary; its own contents were never triaged
  and no disposition was recorded. Deliberately not folded into T-2951 (one finding,
  one task). Note the envelope is stamped source.project: termlink despite originating
  at opencode, which is the T-2955 attribution defect seen from the receiving side.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-008]
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
created: 2026-09-11T20:42:16Z
last_update: 2026-09-21T20:48:13Z
date_finished: 2026-09-21T20:48:13Z
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
  - ts: '2026-09-18T18:42:31Z'
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
cost_estimate_proposed:
  - ts: '2026-09-18T18:42:32Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=204,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2960: P-078 inbound peer filing has no recorded disposition

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

P-078 is the one genuinely INBOUND envelope left in `.context/pickup/auto-deferred/`
(T-2951 drained the three that were our own outbound records). It was cited in T-2954
and the arc-008 census only as the TRIGGER that exposed the blind pickup canary — its
own payload was never read and no disposition was ever recorded. This task reads it and
records one. The deliverable is the disposition, not a code change.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] P-078's payload is read in full and its ask-of-us is classified (action-required vs
      acknowledge-only), with the classification and the evidence for it recorded in a
      `## Disposition` section of this task file
- [x] The disposition names the originating project and task (opencode, T-064 wave-156),
      states the cross-check result it reports, and gives the explicit reason why no
      downstream task is filed from it
- [x] The receive-side attribution artefact (`source.project: termlink` on an envelope that
      originated at opencode) is recorded as corroborating the already-filed T-2955, and is
      NOT re-filed as a new defect
- [x] Whether `check-pickup-deferred-freshness.sh` still counts P-078 as STRANDED is measured
      and the verdict recorded with its reason — the checker is neither weakened nor the
      envelope drained to make it green (T-2801: it detects and never drains)

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

## Disposition

**Classification: ACKNOWLEDGE — no action required of us. No downstream task filed.**

**Origin.** opencode project, task T-064, wave-156, thread `inbound-learnings-crosscheck`,
dated 2026-09-10. Contact recorded in the payload as an opencode agent (model glm-5.3-flash,
user dimitri-mint-dev), delivered via `termlink exec` → `fw pickup send`.

**What it reports.** opencode ingested three of our filings (T-1975 harness-instability,
T-1850/PL-136 double-run technique, T-1976 `--hub` colon-discrimination) and cross-checked the
T-1976 bare-IP class across their own estate. Result: **zero instances**. Their app repo carries
no `--hub` literal (templates default to `hub=""`, the local hub); 005-Deco-m4r-provisioning uses
`DHCP_OWNER_HUB="192.168.10.122:9100"` — `host:port`, with a config.env comment forbidding the
bare and profile-name forms. They also recorded adopting two of our lessons.

**Why no task is filed.** The envelope asks nothing of us. It is a corroborating NEGATIVE result
plus an adoption report: it reports no defect, requests no change, and names no unmet need. Its
value to this project is evidentiary, and it is recorded here rather than actioned:

1. It independently confirms that fixing the T-1976 class at the **termlink layer**
   (the `termlink-hub.sh` guard) was the correct scope, rather than per-consumer — the
   conclusion the payload states explicitly.
2. It is direct evidence that our outbound filings are being **read and acted on** by a peer
   project, which is the thing G-063 exists to make observable.

Filing a task to "respond" would manufacture work from a report whose content is that nothing
is wrong.

**Receive-side attribution artefact.** The envelope is stamped `source.project: termlink` despite
originating at opencode. This is the **T-2955** attribution defect (project label falling back to
a path-derived slug) seen from the RECEIVING side. T-2955 is already filed and work-completed;
this is recorded as corroboration and is deliberately **NOT re-filed** as a new defect.

**Stranded-checker verdict (measured, not assumed).** `bash scripts/check-pickup-deferred-freshness.sh`
→ **rc=1, still firing**, reporting P-078 as the single STRANDED envelope. That is correct and
expected: the class is "no breadcrumb, so `fw pickup promote-deferred` can never promote it", and
recording a disposition in this task file does not mint a breadcrumb. The checker was **not**
weakened and the envelope was **not** drained to make it green — per T-2801 it detects and never
drains, and discarding is a human judgement. The envelope is left in place, now with its contents
read and its disposition recorded, which is what was actually missing.

**Byproduct finding, filed separately as T-3045.** The checker's own age-resolution is defective:
`TS_RE` accepts an optional double quote but not a single quote, and the pickup pipeline writes
RFC3339 values single-quoted, so `recorded_time()` returns `None` and the age silently falls back
to file mtime — the exact PL-213 environment-dependence its docstring says it prevents. Proven by
running the regex against all three quoting styles (single NO-MATCH, double MATCH, bare MATCH).
It closes none of this task's acceptance criteria, so it was registered as its own task rather
than folded in here.

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

# AC1+AC2: a Disposition section exists, classifies the ask, and names the origin.
grep -q '^## Disposition' .tasks/active/T-2960-p-078-inbound-peer-filing-has-no-recorde.md
grep -q 'ACKNOWLEDGE — no action required' .tasks/active/T-2960-p-078-inbound-peer-filing-has-no-recorde.md
grep -q 'opencode project, task T-064, wave-156' .tasks/active/T-2960-p-078-inbound-peer-filing-has-no-recorde.md
grep -q 'zero instances' .tasks/active/T-2960-p-078-inbound-peer-filing-has-no-recorde.md
# AC3: T-2955 recorded as corroboration, explicitly not re-filed.
grep -q 'NOT re-filed' .tasks/active/T-2960-p-078-inbound-peer-filing-has-no-recorde.md
# AC4: the envelope is still on disk — the checker was not made green by draining it (T-2801).
test -f .context/pickup/auto-deferred/P-078-learning.yaml
# AC4: and the checker genuinely still fires on it, exactly as the disposition records.
rc=0; bash scripts/check-pickup-deferred-freshness.sh > /tmp/.t2960-chk 2>&1 || rc=$?; test "$rc" = 1
grep -q 'STRANDED: P-078-learning.yaml' /tmp/.t2960-chk
# The byproduct finding was registered as its own task, not folded into this one.
test -f .tasks/active/T-3045-stranded-envelope-checker-falls-back-to-.md

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

### 2026-09-21 — a disposition is not a breadcrumb, and the checker cannot tell them apart

- **What changed:** The task was filed as "nobody recorded a disposition", which implied that
  recording one would resolve the finding. It does not, and should not. `check-pickup-deferred-
  freshness.sh` fires on the STRANDED class — *no breadcrumb, therefore `fw pickup
  promote-deferred` can never promote it* — which is a statement about **promotability**, not
  about whether a human has read the thing. A disposition and a breadcrumb are different
  artefacts. So P-078 is now fully triaged and the checker still reports it, correctly, at rc=1.
- **Plan impact:** The obvious-looking success condition ("record disposition, checker goes
  green") was wrong and was deliberately not pursued. Making it green would have required either
  weakening the checker or draining the envelope — and T-2801 is explicit that it detects and
  never drains, because turning a visible backlog into a silent one is the exact trade it exists
  to reverse. The AC was therefore written to *measure and record* the verdict rather than to
  flip it, and `## Verification` now asserts that the checker still fires.
- **Open hazard this leaves:** P-078 will now fire on this check forever, for a reason that has
  been fully addressed. That is the T-2818 permanently-red-guard shape — a guard nobody reads —
  and the resolution is a human judgement (drop the envelope, or mint a breadcrumb for it), not
  something an agent should take unilaterally. Surfaced here rather than decided.
- **Triggered:** T-3045 (filed, `captured`). While measuring the checker's verdict for AC4 its
  own age-resolution proved defective: `TS_RE` accepts an optional double quote but not a single
  quote, and the pickup pipeline writes RFC3339 values single-quoted, so `recorded_time()`
  returns `None` and every envelope's age silently falls back to mtime — the precise PL-213
  environment-dependence the function's docstring says it exists to prevent. Masked on this host
  only because mtime happens to agree (11 days either way) for P-078. Not folded in here (one
  bug, one task), and not fixed under this task, since it closes none of these ACs.

### 2026-09-21 — the gate that refused me reported a second finding while doing it

- **What changed:** G-020 blocked the first command of this task because the ACs were still
  template placeholders. Answering it (writing real ACs) was the documented unblock, not a
  bypass. But its refusal message also named something unrelated and true: it could not prove
  `bash scripts/check-pickup-deferred-freshness.sh` is a read, because that script is absent
  from the P-002 read-only allowlist.
- **Plan impact:** None to this task's scope — recorded rather than actioned.
- **Triggered:** Nothing new filed. This is a fresh instance of the class **T-3030** already
  closed for other pure-read guard scripts, so it is logged as a recurrence of a known class
  rather than re-filed as a novel defect.

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

### 2026-09-11T20:42:16Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2960-p-078-inbound-peer-filing-has-no-recorde.md
- **Context:** Initial task creation

### 2026-09-21T20:43:03Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-267fdcca
- **Timestamp:** 2026-09-21T20:48:14Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-21T20:48:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
