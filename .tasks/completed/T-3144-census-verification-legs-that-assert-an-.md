---
id: T-3144
name: "Census verification legs that assert an absence with nothing proving the search
  could succeed"
description: >
  832-Workflow-designer (framework:pickup 152/153, task T-843) measured 122 verification
  legs where a leg asserts an ABSENCE with nothing establishing the search could have
  succeeded — '! grep -q PATTERN FILE' passes vacuously once FILE is gone. Red since
  2026-09-01, count rose 78 -> 122 in that window. We have NO count of the shape here.
  Same class as T-2831 and the reason T-3142 was parked.

status: work-completed
workflow_type: inception
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-09-25T11:49:26Z
last_update: 2026-09-25T14:52:00Z
date_finished: 2026-09-25T14:52:00Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── Inception scoring exception (T-2186 Slice 2 / T-2188). See 050-Inceptions.md §Scoring Exception. ──
target_blast_radius: 3            # int 0..9. Anticipated component count of the build work this inception would authorise on GO.
                                  # Substitutes for the absent components: list in the F8 cost formula (040). Required.
                                  # Guide: 0=docs only, 1=single file, 3=small subsystem (S), 5=cross-subsystem (M), 7=multi-arc (L), 9=framework-wide (XL).
voi_score: 0.5                    # float 0..1. Value of Information — expected value of resolving this question,
                                  # independent of build cost. Higher when answer affects many tasks or unblocks a strategic decision. Required.
bvp_scores_proposed:
  - ts: '2026-09-25T14:22:12Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3144: Census verification legs that assert an absence with nothing proving the search could succeed

## Problem Statement

A verification leg that asserts an ABSENCE — `! grep -q "PATTERN" file` — cannot distinguish
"the bad thing is not there" from "I could not look". Both exit 0. If the file is renamed,
deleted, or empty, the leg passes and the gate reports green over a check that never ran.

**For whom:** anyone reading a green `## Verification` block as evidence. That is the whole
point of P-011 — the framework runs the commands mechanically so the agent is not
self-assessing — and this shape hands back a mechanical pass that means nothing.

**Why now:** a peer measured it. `832-Workflow-designer` posted 122 instances in their tree
(framework:pickup 152/153, their T-843), red since 2026-09-01, having grown 78 → 122. They
explicitly marked it "not a request". We have no count of our own, and two reasons to expect
a non-zero one: T-2831 found the sibling defect in this corpus, and T-3142 was parked one
commit before this task was filed because its own load-bearing test passed vacuously.

## Assumptions

- **A1:** The `## Verification` block is the right and sufficient scope. Rejected alternative:
  scanning `scripts/` too — those are guarded by their own fixture suites, and the defect
  under study is specifically about the P-011 gate's evidence.
- **A2:** A companion `test -f` / positive `grep -q` / `&&`-joined producer genuinely clears
  the risk. Tested by hand on two instances rather than assumed (T-1417, T-2873) — both
  confirmed correct.

## Open Questions

<!-- T-2190 (T-2186 Slice 4): every IW-N question must be disposed before
     --status work-completed. Disposition gate (agents/task-create/update-task.sh
     check_disposition_gate) refuses on under-disposed inceptions.

     Per-question shape:

       - **IW-1: <question text>**
         confidence: 0-3      (your confidence in your current answer; 0=guess, 3=verified)
         disposition: answered | deferred | dissolved
         rationale: <one-line evidence — file:line, decision id, dialogue ref>

     Never bare yes/no — the gate refuses bare checkboxes. See 050-Inceptions.md
     §Disposition Gate. Bypass: --skip-disposition-gate "rationale" (direct) or
     FW_SKIP_DISPOSITION_GATE=1 (env-var, T-1890 producer/consumer parity).
-->

- **IW-1: How many verification legs in this corpus assert an absence with nothing
  establishing the search could have succeeded?** This is the whole inception. 832 measured
  122 in their tree; a number here is what makes every other question answerable.
  confidence: 3
  disposition: answered
  rationale: 30 firing of 71 candidates across 2853 task files — docs/reports/T-3144-vacuous-absence-census.md §The number. 832 measured 122.

- **IW-2: Is the shape endemic or concentrated?** A number alone cannot choose a remedy.
  If the instances cluster in a few tasks or a few copy-pasted idioms, the fix is a template
  change and a note; if they are spread thinly across hundreds of authors and years, only a
  structural check reaches them.
  confidence: 3
  disposition: answered
  rationale: Endemic, not concentrated — 23 distinct tasks, max 2 legs each, no single idiom carries the count. But 41/71 (58%) already carry the correct companion, so the convention is working.

- **IW-3: Is any firing leg live risk, or is it all latent?** A vacuous leg over a path that
  exists today is a trap waiting for a rename; a vacuous leg over a path that is ALREADY gone
  is a gate reporting green right now. Those are different severities and the census must
  distinguish them rather than reporting one total.
  confidence: 3
  disposition: answered
  rationale: All latent, none live — 14 of 14 resolvable literal paths exist today. 28 of 30 firing legs are in .tasks/completed/; only 2 are active, both T-1415 over crates/.

- **IW-4: Would a guard-layer member actually pay for itself here?** Explicitly askable in
  the negative. The layer is already ~16 minutes and T-2483 documents breadth accretion as a
  real cost; "the shape exists" is not sufficient grounds. NO-GO is an acceptable outcome and
  the census is worth running even if that is where it lands.
  confidence: 3
  disposition: answered
  rationale: NO — 28 of 30 findings are in closed tasks and structurally un-actionable, so the member would need a 28-entry allowlist to ever be green (the T-2833 permanently-red disease) on a layer already ~16min (T-3090). Cheaper remedy proposed instead: an absence-assertion rule in the task template beside the existing Pipefail section. Offered, not performed — a template change is the operator's call.

## Exploration Plan

**Spike 1 — count (time-box: 30 min).** Scan `## Verification` blocks across
`.tasks/active/` and `.tasks/completed/` for negated searches over a named path. Classify
each as FIRING (nothing in the block proves the path was readable) or CLEARED (a companion
`test -f` / `-s` / positive `grep` / `&&`-joined producer exists). Answers IW-1.

**Spike 2 — distribution (time-box: 15 min).** Group the firing set by task and by idiom.
Answers IW-2.

**Spike 3 — liveness (time-box: 15 min).** For each firing leg, resolve the path and test
whether it exists in the tree TODAY. A firing leg over a missing path is a gate that is
green right now and should not be. Answers IW-3.

IW-4 is answered from the three results plus judgement, and is presented — not decided.

## Technical Constraints

<!-- What platform, browser, network, or hardware constraints apply?
     For web apps: HTTPS requirements, browser API restrictions, CORS, device support.
     For hardware APIs (mic, camera, GPS, Bluetooth): access requirements, permissions model.
     For infrastructure: network topology, firewall rules, latency bounds.
     Fill this BEFORE building. Discovering constraints after implementation wastes sessions. -->

## Scope Fence

**IN:** counting the shape in `## Verification` blocks across `.tasks/active/` and
`.tasks/completed/`; classifying firing vs cleared; measuring distribution and path liveness;
producing a recommendation.

**OUT, deliberately:** building a guard-layer member (that is what the recommendation is
*about*, and pre-building it would make the census decorative); editing the 28 historical
firing legs in closed tasks (rewrites history for no behavioural gain); amending the task
template (a convention change for every future task — operator's decision, offered not taken);
scanning anything outside the task corpus.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [x] Problem statement validated
<!-- @auto-tick-on-decide -->
- [x] Assumptions tested
<!-- @auto-tick-on-decide -->
- [x] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [x] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
These are the criteria stated in the research artifact's "What this inception will NOT do"
section, written and committed BEFORE any measurement was taken, so the verdict is not
back-fitted to the number that came out.

**GO (build a guard-layer member) if:**
- The firing count is 832-scale (their 122) rather than a handful, AND
- the instances are actionable — i.e. mostly in ACTIVE tasks, where a fix changes what a
  future gate actually does, AND
- a meaningful share are vacuous TODAY (path already gone), not merely latent, AND
- the existing `## Verification` convention is demonstrably NOT closing the shape on its own.

**NO-GO if:**
- The count is small, or dominated by closed tasks whose verification blocks will never run
  again — a member that must ship with a large allowlist to be green is the T-2833
  permanently-red disease, and the guard layer is already ~16 minutes (T-3090), so breadth
  has a measured price (T-2483).
- Authors are already writing the companion check at a healthy rate, which makes the leverage
  point the TEMPLATE (read while a block is being written), not a detector (read after).

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** NO-GO on a guard-layer member. The census ran and answered the question;
the answer is that a check is not warranted. A template amendment is offered instead.

**Rationale (post-census, 2026-09-25):** Measured 30 firing legs of 71 candidates across 2853
task files — real, and a quarter of 832's 122. The count is not what decides it; the
composition is. **28 of the 30 sit in `.tasks/completed/`**, whose verification blocks will
never execute again, so a new member would report 30 findings on day one of which 28 are
structurally un-actionable and would need a 28-entry allowlist before it could ever go green.
That is the T-2833 shape exactly — a draft that fired on 58 legitimate-but-unfixable
instances and had to be re-scoped because a permanently-red check is one nobody reads — on a
layer already costing ~16 minutes (T-3090). **Zero firing legs are vacuous today**: 14 of 14
resolvable literal paths exist, so every instance is a latent trap rather than a gate
currently reporting green over nothing. The 2 live instances are a single task searching
`crates/`, the repository's core source directory.

Against the pre-stated GO criteria: the count is not 832-scale, the instances are not
actionable, none are vacuous today, and **the convention is demonstrably working** — 41 of 71
candidates (58%) already carry the correct companion, verified by hand on two of them
(T-1417 pairs its negated grep with `test -f` on the same path; T-2873 with a positive
`grep -q` on the same file). Four of four GO conditions fail; two of two NO-GO conditions hold.

The leverage point is therefore the task template, not a detector: the template is read while
a verification block is being written, which is where 832's 78 → 122 growth would come from.
The proposed three-line addition sits beside the existing Pipefail/SIGPIPE guidance and is
written out in `docs/reports/T-3144-vacuous-absence-census.md` § Recommendation. It is
**offered, not performed** — amending the task template changes the shape of every future
task in the project, which is a convention decision for the operator, not agent initiative.

**Evidence:**
- `docs/reports/T-3144-vacuous-absence-census.md` — method, counts, distribution, liveness.
- 71 candidates / 41 cleared / 30 firing across 2853 task files.
- 23 distinct tasks, max 2 legs each — endemic, not concentrated in one copy-pasted idiom.
- 28/30 in `completed/`; 2 in `active/`, both T-1415, both over `crates/`.
- 14/14 resolvable literal paths present — no live vacuity.
- Sharpest single instance found: `[ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]`
  — a build gate that goes green precisely when the build could not run. In a closed task.

**Rationale (at filing, retained for the record):** Measure before deciding. The count is the whole question and we do not have it: 832 measured 122 in their corpus and ours is 2850 task files, so the shape is either widespread here or our '## Verification' convention already closes it, and those two worlds want opposite actions. The census itself is cheap — one pass over .tasks/ for a negated absence-assertion with no companion existence check — and it is the evidence gap, not a confidence gap. Two independent reasons to think the rate is non-zero here: T-2831 found the sibling defect (commands filed under the wrong heading) in this corpus, and T-3142 was parked THIS SESSION because its own load-bearing test passed vacuously. GO is to run the census and report a number, explicitly NOT to build a guard-layer member before that number exists.

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

**Decision**: GO

**Rationale**: Measured 30 firing legs of 71 candidates across 2853
task files — real, and a quarter of 832's 122. The count is not what decides it; the
composition is. **28 of the 30 sit in `.tasks/completed/`**, whose verification blocks will
never execute again, so a new member would report 30 findings on day one of which 28 are
structurally un-actionable and would need a 28-entry allowlist before it could ever go green.
That is the T-2833 shape exactly — a draft that fired on 58 legitimate-but-unfixable
instances and had to be re-scoped because a permanently-red check is one nobody reads — on a
layer already costing ~16 minutes (T-3090). **Zero firing legs are vacuous today**: 14 of 14
resolvable literal paths exist, so every instance is a latent trap rather than a gate
currently reporting green over nothing. The 2 live instances are a single task searching
`crates/`, the repository's core source directory.

Against the pre-stated GO criteria: the count is not 832-scale, the instances are not
actionable, none are vacuous today, and **the convention is demonstrably working** — 41 of 71
candidates (58%) already carry the correct companion, verified by hand on two of them
(T-1417 pairs its negated grep with `test -f` on the same path; T-2873 with a positive
`grep -q` on the same file). Four of four GO conditions fail; two of two NO-GO conditions hold.

The leverage point is therefore the task template, not a detector: the template is read while
a verification block is being written, which is where 832's 78 → 122 growth would come from.
The proposed three-line addition sits beside the existing Pipefail/SIGPIPE guidance and is
written out in `docs/reports/T-3144-vacuous-absence-census.md` § Recommendation. It is

Measure before deciding. The count is the whole question and we do not have it: 832 measured 122 in their corpus and ours is 2850 task files, so the shape is either widespread here or our '## Verification' convention already closes it, and those two worlds want opposite actions. The census itself is cheap — one pass over .tasks/ for a negated absence-assertion with no companion existence check — and it is the evidence gap, not a confidence gap. Two independent reasons to think the rate is non-zero here: T-2831 found the sibling defect (commands filed under the wrong heading) in this corpus, and T-3142 was parked THIS SESSION because its own load-bearing test passed vacuously. GO is to run the census and report a number, explicitly NOT to build a guard-layer member before that number exists.

**Date**: 2026-09-25T14:52:00Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-09-25T14:22:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-09-25T14:52:00Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Measured 30 firing legs of 71 candidates across 2853
task files — real, and a quarter of 832's 122. The count is not what decides it; the
composition is. **28 of the 30 sit in `.tasks/completed/`**, whose verification blocks will
never execute again, so a new member would report 30 findings on day one of which 28 are
structurally un-actionable and would need a 28-entry allowlist before it could ever go green.
That is the T-2833 shape exactly — a draft that fired on 58 legitimate-but-unfixable
instances and had to be re-scoped because a permanently-red check is one nobody reads — on a
layer already costing ~16 minutes (T-3090). **Zero firing legs are vacuous today**: 14 of 14
resolvable literal paths exist, so every instance is a latent trap rather than a gate
currently reporting green over nothing. The 2 live instances are a single task searching
`crates/`, the repository's core source directory.

Against the pre-stated GO criteria: the count is not 832-scale, the instances are not
actionable, none are vacuous today, and **the convention is demonstrably working** — 41 of 71
candidates (58%) already carry the correct companion, verified by hand on two of them
(T-1417 pairs its negated grep with `test -f` on the same path; T-2873 with a positive
`grep -q` on the same file). Four of four GO conditions fail; two of two NO-GO conditions hold.

The leverage point is therefore the task template, not a detector: the template is read while
a verification block is being written, which is where 832's 78 → 122 growth would come from.
The proposed three-line addition sits beside the existing Pipefail/SIGPIPE guidance and is
written out in `docs/reports/T-3144-vacuous-absence-census.md` § Recommendation. It is

Measure before deciding. The count is the whole question and we do not have it: 832 measured 122 in their corpus and ours is 2850 task files, so the shape is either widespread here or our '## Verification' convention already closes it, and those two worlds want opposite actions. The census itself is cheap — one pass over .tasks/ for a negated absence-assertion with no companion existence check — and it is the evidence gap, not a confidence gap. Two independent reasons to think the rate is non-zero here: T-2831 found the sibling defect (commands filed under the wrong heading) in this corpus, and T-3142 was parked THIS SESSION because its own load-bearing test passed vacuously. GO is to run the census and report a number, explicitly NOT to build a guard-layer member before that number exists.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-f3a2ddf2
- **Timestamp:** 2026-09-25T14:52:01Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **disposition-incomplete** (partial, heuristic) @ ## Open Questions: IW-2
     - evidence: `IW-2 disposition='answered' but rationale has no evidence citation (T-NNNN, file:line, docs/reports/, G-/L-/D-id, dialogue-log, or commit hash)`

## Recommendation Verdict (v1.0)

- **Scan ID:** RC-715cf9d4
- **Timestamp:** 2026-09-25T14:52:01Z
- **Overall:** CONFIRMED
- **Claims:** 8

| Claim | Type | Status |
|-------|------|--------|
| `docs/reports/T-3144-vacuous-absence-census.md` | file | ✓ pass |
| `T-2833` | task | ✓ pass |
| `T-3090` | task | ✓ pass |
| `T-1417` | task | ✓ pass |
| `T-2873` | task | ✓ pass |
| `T-1415` | task | ✓ pass |
| `T-2831` | task | ✓ pass |
| `T-3142` | task | ✓ pass |

### 2026-09-25T14:52:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Inception decision: GO
