---
id: T-3042
name: "Pickup consumer discards envelope payload — content-free tasks rank BVP 70
  hv-lc"
description: >
  pickup_create_inception (lib/pickup.sh:431) passes only name/type/owner/description/horizon/tags
  to fw task create, discarding payload.detail, priority and tags. Every auto-created
  task is a 167-line empty template; the BVP estimator then scores all-2s no-signal
  = BVP 70, tier 4 inception, cost 3.6 = hv-lc, so content-free tasks top the Q1 quadrant.
  Vendored (G-062) — file upstream, do not patch.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [bug, pickup, framework-upstream, bvp]
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
created: 2026-09-21T14:04:04Z
last_update: 2026-09-21T14:21:07Z
date_finished: 2026-09-21T14:21:07Z
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
  - ts: '2026-09-21T14:06:42Z'
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

# T-3042: Pickup consumer discards envelope payload — content-free tasks rank BVP 70 hv-lc

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Reproduced before filing** (PL-367): a pickup envelope carrying a non-empty
      `payload.detail` is processed, and the resulting task file is confirmed to contain
      none of it — measured, not read off `lib/pickup.sh`. Use a fixture envelope, never a
      live inbound one.
- [x] The BVP consequence is measured, not asserted: the created task's proposed scores are
      shown to be the all-2s `no-signal` default (BVP 70) and to land in `hv-lc`.
- [x] Filed upstream at `framework:pickup` (G-062 — `lib/pickup.sh` is vendored and a local
      patch is erased by the next `fw upgrade`), naming both halves: the consumer dropping
      `payload.detail`/`priority`/`tags`, AND the estimator's no-signal default being HIGH
      rather than neutral, so an unreadable task outranks a readable one.
- [x] The filing cites T-2687 explicitly — that task diagnosed the identical displacement
      ("ranked BVP 70 / hv-hc, displacing real work") and its guard only refuses a
      literally-empty `summary`, so a good summary over an empty body still passes.
- [x] Filing is recorded here with its `framework:pickup` offset, so this task cannot close
      on an intention to file (the T-2812 lesson: two `lib/upgrade.sh` defects sat 76 days
      behind ACs that said "report written for operator copy-paste", ticked and never sent).
- [x] Local detection, if any is added, is a CHECK not a patch — and must not fire on the
      32 pre-existing shells that T-3043 is draining, or it is red on arrival (T-2818).


<!-- ── Measured evidence (T-3042 execution, 2026-09-21) ────────────────────── -->

**Reproduction (AC 1).** Fixture envelope with `payload.detail` carrying sentinel
`SENTINEL-DETAIL-9F2A` + a file:line + a repro command, `priority: high`, and
`tags: [SENTINEL-TAG-RINGBUF, widget, performance]`. Isolated project root,
`fw pickup process`, rc=0, task created. In the resulting task file: detail
sentinel `grep -c` = **0**, tag sentinel = **0**, file path = **0**, priority =
**0**. `tags:` read `[pickup, bug-report]`. 243 lines, all template. The producer
side accepts all three fields (`fw pickup send --detail --priority --tags`), so
the asymmetry is consumer-only.

**BVP consequence (AC 2), measured on the live register — and it corrects the
figure this task was filed with.** `fw bvp --quadrant hv-lc --include-proposed`:
28 tasks in the quadrant, **25 tied at BVP 70**, **18 of those 25 are `Pickup:`
shells**. The first task carrying real content (T-2938, a live cron-install
drift) ranks below all 18. 35 `Pickup:` shells in `.tasks/active` total. The
filing description said "8 of 12" — that was read off a truncated listing and is
wrong; 18 of 25 is the counted figure.

Two content-free scoring shapes coexist and the distinction matters: the 18
legacy shells sit at the all-2s no-signal default (2x35 = **BVP 70**), while a
shell created TODAY scores D1=4 D2=0 D3=3 D4=2 = **BVP 57** from template
boilerplate matching keyword heuristics. Both are content-free; the no-signal
reading is the HIGHER of the two. So the claim to carry forward is not "empty
tasks score 70" but "the reading that admits least knowledge ranks best".

**Filed upstream (ACs 3-5): `framework:pickup` offset 138**, msg_type
`pickup-bug-report`, attributed `(010-termlink)` so T-2816's self-filter
suppresses it from our own canary. Read back from the hub at that offset and
confirmed to contain both named halves and the T-2687 citation. `delivered` was
NOT treated as proof (T-2876); the read-back is.

**No local detection was added (AC 6).** The constraint binds nothing: draining
the 35 shells is T-3043's scope, and a checker written now would be red on
arrival against them (T-2818).

**Side finding — the reproduction seam is incomplete.** `pickup_process_one`
mirrors every processed envelope to the LIVE `framework:pickup` topic via
`lib/pickup-channel-bridge.sh` (T-1165), resolving the bridge from
`FRAMEWORK_ROOT`, not `PROJECT_ROOT`. A `PROJECT_ROOT`-isolated reproduction
therefore still posts to the shared cross-project rail. It did here: the fixture
reached offset 136 and was redacted at 137 with an explicit "not a real filing"
reason. Included in the upstream filing as a testability gap. Also observed:
`--payload-from-file` does not exist on the shipping `termlink channel post`, so
the bridge's first form always fails and silently falls through to its second.

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

# ── Real verification (T-3042) ──
termlink channel subscribe framework:pickup --cursor 138 --limit 1 > /tmp/.t3042rb 2>&1 && grep -q 'pickup_id: P-TL-3042' /tmp/.t3042rb
grep -q 'T-2687' /tmp/.t3042rb
grep -q 'HALF 1: the consumer drops the payload' /tmp/.t3042rb
grep -q 'HALF 2: the empty body then scores HIGH' /tmp/.t3042rb
grep -rq 'framework:pickup offset 138' .tasks/
grep -A9 'fw task create' .agentic-framework/lib/pickup.sh > /tmp/.t3042src 2>&1 && ! grep -q 'payload' /tmp/.t3042src

## RCA

**Symptom:** 35 `Pickup:` tasks in `.tasks/active` are 243-line untouched
templates. 18 of them occupy the top BVP-70 band of the hv-lc quadrant, ranking
above every task in that quadrant that carries real content.

**Root cause:** two independent gaps that compose.
(1) `lib/pickup.sh::pickup_create_inception` passes only name/type/owner/
description/horizon/tags to `fw task create`. `payload.detail`, `payload.priority`
and `payload.tags` are read from the envelope by the producer (`fw pickup send`
accepts all three) and then never written anywhere by the consumer.
(2) The BVP estimator's no-signal default is 2 on every driver = BVP 70, which is
a mid-high value claim rather than an abstention. An unreadable task therefore
competes, and wins, against readable ones.

Neither alone would surface. A dropped payload with an abstaining estimator would
leave the shells at the bottom of the queue where their emptiness is harmless. A
high no-signal default with a payload-preserving consumer would have real content
to score. Together they invert the queue.

**Why structurally allowed:** T-2687 had already diagnosed this exact displacement
— its own comment says the content-free tasks "ranked BVP 70 / hv-hc, displacing
real work at the top of `fw bvp --quadrant hv-hc`" — and added a guard. But the
guard refuses only when `summary` or `source.project` extract *empty*. It was
written against the malformed-envelope case that produced tasks literally named
"Pickup:  (from )". A well-formed envelope with a good summary and a rich body
passes it cleanly and produces an equally empty task. The guard tested the
symptom it had in front of it rather than the property it wanted, which is that a
created task carries the content that justified creating it.

The blindness compounded because nothing measures the quadrant's *composition*.
`fw bvp --quadrant hv-lc` reports scores, not whether the top band is made of
tasks nobody can read; and 161 of 200 tasks have no cost at all, so the
thresholds are computed over 39. A queue can be 64% content-free at the top and
every surface still reads green.

**Prevention:** filed upstream at `framework:pickup` offset 138 (G-062 — vendored,
not patched here), naming both halves so a consumer fix that leaves the estimator
default alone is not mistaken for a complete one. Deliberately NOT paired with a
local checker: the 35 existing shells are T-3043's scope and any detector written
before that drain would be red on arrival, which is how a guard teaches its
operator to stop reading it (T-2818).

## Evolution

### 2026-09-21 — the filed premise was half wrong, and the measurement is what caught it

- **What changed:** this task was filed asserting the estimator's no-signal
  default (all-2s, BVP 70) is what pins the shells at the top. Reproducing it
  showed a shell created *today* scores D1=4 D2=0 D3=3 D4=2 = BVP 57 — the
  estimator now matches keyword heuristics against template boilerplate. Both
  outcomes are content-free; the 70 is simply the higher one. The accurate claim
  is "the reading that admits least knowledge ranks best", not "empty tasks score
  70". The filing carries the corrected version.
- **Plan impact:** AC 2 as written ("shown to be the all-2s no-signal default")
  would have been satisfied by the 18 legacy shells alone and would have missed
  that fresh shells score differently. It was answered on the live register
  rather than on the fixture, which is what surfaced the divergence.
- **Triggered:** the displacement figure was also corrected — 18 of 25, not the
  "8 of 12" in this task's own description, which had been read off a truncated
  listing.

### 2026-09-21 — PROJECT_ROOT does not isolate the pickup pipeline

- **What changed:** the reproduction was scoped with `PROJECT_ROOT` pointed at a
  throwaway project, which correctly isolated the task register, the pickup dirs
  and the dedup log. It did not isolate the hub. `pickup_process_one` mirrors
  every processed envelope to the live `framework:pickup` topic through
  `lib/pickup-channel-bridge.sh`, resolved from `FRAMEWORK_ROOT`. The fixture
  reached a shared cross-project rail at offset 136.
- **Plan impact:** none to the deliverable, but the reproduction was not as
  side-effect-free as it was designed to be, and the design error was invisible
  until after the fact. Redacted at offset 137 with an explicit "not a real
  filing" reason naming the cause, so a peer reading the topic is not left
  guessing.
- **Triggered:** included in the upstream filing as a testability gap — anyone
  testing this pipeline hits it. Also observed en route: `--payload-from-file`
  does not exist on the shipping `termlink channel post`, so the bridge's first
  post form always fails and falls through to its second, silently.

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

### 2026-09-21T14:04:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3042-pickup-consumer-discards-envelope-payloa.md
- **Context:** Initial task creation

### 2026-09-21T14:06:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e43200bb
- **Timestamp:** 2026-09-21T14:21:08Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `cross-project`

### 2026-09-21T14:21:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
