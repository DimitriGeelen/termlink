---
id: T-2933
name: "Wire the 12 self-declared-hermetic dormant guards into the guard layer"
description: >
  T-2929 measured 57 dormant guard scripts. Twelve of them declare hermeticity in
  their own headers (no live hub, no live PTY) and their disposition is unambiguous:
  guard-layer, missing only the marker. Run each standalone to confirm it is hermetic,
  green and fast, then add the marker so it runs on every push and PR. The 18 deploy-time
  and 27 provisional rows are explicitly NOT in scope.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/test-comms-selftest.sh, scripts/test-diagnose-unconsumed.sh, scripts/test-fleet-rearm-wakers.sh, scripts/test-pushwaker-ready-loop.sh, tests/agent-send-idle-gate.sh, tests/relay-b2-send-hops.sh, tests/relay-b3-hop-budget.sh, tests/relay-wake-confirm.sh, tests/stale-waker-code-canary.sh, tests/tl-claude-identity-binding.sh, tests/wake-confirm-reply-match.sh]
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
created: 2026-09-08T21:47:10Z
last_update: 2026-09-09T07:43:13Z
date_finished: 2026-09-09T07:43:13Z
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
  - ts: '2026-09-08T21:49:24Z'
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
  - ts: '2026-09-08T21:49:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-2933: Wire the 12 self-declared-hermetic dormant guards into the guard layer

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## State at park (session S-2026-0908-2240, parked on context budget at ~96%)

**11 of 12 marked. Work is done and committed; verification is incomplete.**

| step | result |
|---|---|
| pre-screen for live send paths | 3 flagged, all cleared by reading: 2 only `bash -n`/grep `agent-send.sh`; `relay-b2-send-hops.sh` executes it under `TERMLINK=/bin/true`, a real stub seam |
| all 12 run standalone | **12/12 rc=0**; 11 completed in ≤1241 ms |
| marker added | **11** — `mutate-2783.sh` withheld |
| unclassified | 86 → **75** (−11) ✓ |
| dormant (`check-guard-runner-coverage.sh`) | 57 → **46** (−11) ✓ |
| guard-layer members | 100 → **112**; +11 markers plus `tests/guard-runner-coverage-fixtures.sh`, which joins by naming convention (authored under T-2929) |

**`mutate-2783.sh` was withheld on its own AC, not waved through.** It passes (rc 0) but takes
**58.6 s**. The guard-layer CI job is documented as "runs on every push and PR — no Rust build,
seconds"; one 59 s member contradicts that. Bound set at **10 s** — the other 11 are all under
1.3 s, so the bound separates cleanly rather than being fitted to the answer. It needs either a
fast mode or a slower tier; that is a runner decision, so it is surfaced, not decided.

### Why this is parked and not closed

AC3 is **unverified**. `run-guard-layer.sh` did not finish inside a 580 s bound (it already
exceeded 300 s last session, so this is pre-existing — 11 members at ~4 s total cannot explain
it), and the run stopped before reaching any of the 11. Their verdicts are not in doubt — the
layer executes each member as `bash <script>` and records the rc, which is exactly the
measurement taken standalone, 12/12 green — but the *layer-level* assertion has not been made,
and asserting it from the standalone runs would be inferring a check I did not run.

### Finding: the guard layer is RED on main, and CI probably does not see it

3 FAILs, none in a file this task touches: `check-installed-binary-drift.sh`,
`check-receiver-ack-lag.sh`, `cron-drift-firing-fixtures.sh`. All three read **host state** —
an installed binary, ack lag, `/etc/cron.d`. CI has none of it, so this is plausibly
locally-red / CI-green, which would mean the push/PR gate and the developer's own run disagree
about whether the tree is sound. That is the shipped≠live shape (G-069) inside the guard layer.
Not investigated — surfaced as a Sovereign question.

### Also unresolved: the layer's runtime

>580 s locally. CLAUDE.md says "seconds". Either the documented claim is stale or the layer has
grown past what a per-push gate should carry. A decision either way is out of this task's scope.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Every candidate is executed standalone before it is trusted, and the header is not taken as evidence.** Each of the 12 scripts T-2929 classified `guard-layer` on the strength of its own "hermetic / no live hub" header is run in isolation and its exit code, wall-clock duration, and any network or host-state side effect recorded. A self-declaration is a claim by the author, not a measurement — T-2929's own hermeticity screen was wrong twice, once posting real broadcasts to three live fleet hubs. A script whose header says hermetic but whose behaviour disagrees is NOT marked.
- [x] **Only scripts that pass on their own merits get the marker.** A candidate earns `# guard-layer: source` iff it exits 0, completes within a stated bound, and shows no live-hub or host-state dependency when run. Any candidate that fails, hangs, or touches the fleet is left unmarked with the reason recorded — a failing script added to the layer would make every push and PR red, which is how a guard layer gets switched off.
- [x] **The layer is demonstrably larger, and this change introduces no new failure.** Member count has grown by exactly the number of scripts marked and unclassified has fallen by the same number, both stated before and after — "the layer still passes" is not the same claim as "the layer now covers more". *(AC AMENDED mid-task, openly: it originally read "runs to completion with no FAIL and no ERROR". That baseline was assumed and is false — the layer is already RED on main with 3 pre-existing FAILs, none in a file this task touches. Amending an AC to something passable is exactly the producer-not-judge hazard, so the amendment is recorded here rather than made silently, and it does not weaken the real claim: no NEW failure. The 3 pre-existing FAILs are a separate finding, below.)*
- [x] **`check-guard-runner-coverage.sh` reflects the change, and the residue is named.** Re-running it shows the dormant count reduced by exactly the number marked, and the remaining dormant scripts are reported with their buckets intact (deploy-time / provisional) so nothing is silently absorbed. The check must still FIRE — this task does not claim to end dormancy, only to close the unambiguous part of it.
- [x] **No deploy-time or provisional script is marked.** The 18 `deploy-time` and 27 `guard-layer?` rows are out of scope by construction: the first would read host state in CI, the second is unverified. Verified mechanically, not by intent.

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

## Close-out (session S-2026-0909, AC3 resolved)

Parked last session with AC3 unverified: `run-guard-layer.sh` had not finished inside a
580 s bound and never reached the 11 newly-marked members. Their verdicts were not in
doubt — the layer runs each member as `bash <script>` and records the rc, the same
measurement taken standalone — but the *layer-level* assertion had not been made, and
asserting it from the standalone runs would have been inferring a check never run. It
has now been run to completion.

**AC3, measured.** Full layer run, unbounded, ~9 min wall clock:

    guard layer: FIRING - 3 guard(s) found something (109 passed, 0 errored)

- 112 members executed, 0 ERROR, 0 SKIP.
- All **11** newly-marked members PASS (`suite` tier).
- 3 FAIL — `check-installed-binary-drift.sh`, `check-receiver-ack-lag.sh`,
  `cron-drift-firing-fixtures.sh`: the pre-existing host-state failures recorded at
  park, none in a file this task touched. **No new failure.**

**Member delta, measured exactly.** Rebuilt the pre-change member set from `540fc51d4`
via `git archive`: **102 to 113**, delta **11**, and the delta set is byte-identical to
the 11 scripts marked. The park note's "100 to 112" was loose recollection; the
discrepancy it implied does not exist.

**AC4.** unclassified 86 to **75** (-11); dormant + fixtures-only 57 to **46** (-11);
buckets intact; check still FIRES (rc 1). The task never claimed to end dormancy.

**AC5.** All 11 verified mechanically against T-2929's disposition table as
guard-layer rows — 0 deploy-time, 0 provisional.

### Finding: a marked script the runner never runs (pre-existing, filed onward)

Reconciling 113 computed members against 112 executed surfaced a real gap, not an
off-by-one. `scripts/fabric-workflow-link.sh` carries the guard-layer marker on line 2,
but the runner's inventory globs are `scripts/check-*.sh`, `scripts/test-*.sh`,
`tests/*.sh` — it matches none. It is therefore **never run, never listed by `--list`,
and never reported as unclassified**: it declares membership and is silently excluded.

`check-guard-runner-coverage.sh` inherits the same globs, so the script is invisible to
the detector as well as the runner — neither covered nor flagged. Introduced by T-2839
(2026-08-27), so it predates this task and does not affect the no-new-failure claim.
Filed separately rather than fixed here: widening the globs changes the runner's
contract and belongs behind its own gate (one lock at a time).

### Process note — the same error, twice

The verification block was twice inserted into the wrong section, because
`str.index('## Verification')` matched a *mention* of that heading inside the Human-AC
template comment rather than the heading itself. The first attempt put commands under
`## Acceptance Criteria` (the T-2831 misfile defect — P-011 never runs them, so the gate
passes vacuously). The second spliced a block mid-line into the template comment, which
the completion gate caught and refused with "the ## Verification block contains line(s)
bash cannot parse".

The gate was right both times and was not bypassed, though
`FW_ALLOW_UNPARSEABLE_VERIFICATION=1` was offered. Fixed by restoring the file from HEAD
and re-applying every edit through an exact-line matcher that asserts a unique match and
fails loudly otherwise. Recorded because the error was made by the author of a check for
that exact error class — a self-declaration is a claim, not a measurement, which is this
task's own thesis turned back on itself.

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


# ── T-2933 verification (added at close) ─────────────────────────────────────
# Asserts RELATIONSHIPS, not pinned totals: unclassified/dormant counts move
# whenever any guard is added anywhere in the repo, and a gate hard-coding 75/46
# would go red on unrelated work and get switched off.
# AC1+AC2 — each of the 11 marked scripts still passes standalone.
for s in scripts/test-comms-selftest.sh scripts/test-diagnose-unconsumed.sh scripts/test-fleet-rearm-wakers.sh scripts/test-pushwaker-ready-loop.sh tests/agent-send-idle-gate.sh tests/relay-b2-send-hops.sh tests/relay-b3-hop-budget.sh tests/relay-wake-confirm.sh tests/stale-waker-code-canary.sh tests/tl-claude-identity-binding.sh tests/wake-confirm-reply-match.sh; do bash "$s" >/dev/null 2>&1 || { echo "FAILED standalone: $s"; exit 1; }; done
# AC2 — all 11 carry the marker.
test "$(grep -lE '^# guard-layer: source' scripts/test-comms-selftest.sh scripts/test-diagnose-unconsumed.sh scripts/test-fleet-rearm-wakers.sh scripts/test-pushwaker-ready-loop.sh tests/agent-send-idle-gate.sh tests/relay-b2-send-hops.sh tests/relay-b3-hop-budget.sh tests/relay-wake-confirm.sh tests/stale-waker-code-canary.sh tests/tl-claude-identity-binding.sh tests/wake-confirm-reply-match.sh | wc -l)" = "11"
# AC3 — the runner enumerates all 11 as members (membership, not just the marker).
bash scripts/run-guard-layer.sh --list > /tmp/.t2933-list.txt 2>&1
test "$(grep -cE 'test-comms-selftest\.sh|test-diagnose-unconsumed\.sh|test-fleet-rearm-wakers\.sh|test-pushwaker-ready-loop\.sh|agent-send-idle-gate\.sh|relay-b2-send-hops\.sh|relay-b3-hop-budget\.sh|relay-wake-confirm\.sh|stale-waker-code-canary\.sh|tl-claude-identity-binding\.sh|wake-confirm-reply-match\.sh' /tmp/.t2933-list.txt)" = "11"
# AC4 — coverage check still FIRES, and none of the 11 is still reported dormant.
bash scripts/check-guard-runner-coverage.sh --json > /tmp/.t2933-cov.json 2>&1 || true
python3 -c "import json;d=json.load(open('/tmp/.t2933-cov.json'));assert d['ok'] is False,'coverage must still fire';w={'test-comms-selftest.sh','test-diagnose-unconsumed.sh','test-fleet-rearm-wakers.sh','test-pushwaker-ready-loop.sh','agent-send-idle-gate.sh','relay-b2-send-hops.sh','relay-b3-hop-budget.sh','relay-wake-confirm.sh','stale-waker-code-canary.sh','tl-claude-identity-binding.sh','wake-confirm-reply-match.sh'};f={e['script'] for e in d['firing']};assert not (w&f),sorted(w&f);print('AC4 ok')"
# AC5 — none of the 11 was a deploy-time or provisional row in T-2929's table.
python3 -c "rows=open('.tasks/completed/T-2929-85-guard-scripts-carry-no-guard-layer-ma.md').read().splitlines(); want=['test-comms-selftest','test-diagnose-unconsumed','test-fleet-rearm-wakers','test-pushwaker-ready-loop','agent-send-idle-gate','relay-b2-send-hops','relay-b3-hop-budget','relay-wake-confirm','stale-waker-code-canary','tl-claude-identity-binding','wake-confirm-reply-match']; g=lambda s:[l for l in rows if l.startswith('| \`%s.sh\`'%s)]; bad=[s for s in want if not g(s) or 'deploy-time' in g(s)[0] or 'guard-layer?' in g(s)[0]]; assert not bad, bad; print('AC5 ok')"
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

### 2026-09-08T21:47:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2933-wire-the-12-self-declared-hermetic-dorma.md
- **Context:** Initial task creation

### 2026-09-08T21:49:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-98ee2af8
- **Timestamp:** 2026-09-09T07:43:25Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 73
     - evidence: `bash scripts/check-guard-runner-coverage.sh --json > /tmp/.t2933-cov.json 2>&1 || true`

### 2026-09-09T07:43:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
