---
id: T-2965
name: "Three stranded pickup envelopes have no breadcrumb and no disposition (P-074,
  P-075, P-077)"
description: >
  check-pickup-deferred-freshness.sh (T-2801) fires: four envelopes sit in .context/pickup/auto-deferred/
  with no breadcrumb, so fw pickup promote-deferred can never promote them and fw
  pickup auto-deferred list shows blocked-by=? while saying nothing is wrong. P-078
  is already tracked by T-2960; P-074, P-075 and P-077 are not. Surfaced by the T-2935
  AC5 full-layer run as the one FAIL beyond the T-2933 baseline of three.

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
created: 2026-09-16T17:39:46Z
last_update: '2026-09-16T17:44:11Z'
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
  - ts: '2026-09-16T17:43:02Z'
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
  - ts: '2026-09-16T17:44:11Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=210,acs=7)
    rubric_sha: e4a00f38e801
---

# T-2965: Three stranded pickup envelopes have no breadcrumb and no disposition (P-074, P-075, P-077)

## Context

Surfaced by T-2935's AC5 full-layer run as the single FAIL beyond the T-2933 baseline of
three — and it was only visible because that AC compared **per-member verdicts** rather than
the summary counts. The headline arithmetic concealed it perfectly: adding one passing member
took PASS to 110 while this check regressed 109, netting back to 109 against a baseline of 109.

**Measured.** All four stranded envelopes carry `source.project: termlink` — they are this
project's OWN outbound filings, deposited into its own inbox by `lib/pickup.sh:643` (already
measured and filed upstream at `framework:pickup` offset 124 under T-2953), then auto-deferred
with no breadcrumb. A breadcrumb names the blocking task; with none, `fw pickup
promote-deferred` has nothing to resolve and `fw pickup auto-deferred list` prints
`blocked-by=?` while reporting nothing wrong. Not delayed — lost.

| envelope | origin task | origin status | upstream offset |
|---|---|---|---|
| P-074 | T-2948 | work-completed | **117** |
| P-075 | T-2949 | work-completed | **118** (and 122, the later correction) |
| P-077 | T-2942 | work-completed | **121** |
| P-078 | T-064 | — | **not in scope — T-2960** |

Every one has a confirmed upstream counterpart, so all three are echoes of delivered work,
not unsent work. That distinction is the whole task: it is what separates "safe to close" from
"silently discarding a report nobody ever read".


## Result (session S-2026-0916c)

**AC1 — provenance, per envelope.** Read individually rather than generalised from the first
sample, because the correct disposition inverts on the answer. All three resolve to completed
local tasks (table above). P-078 is deliberately excluded: its `source.task_id` is `T-064`,
outside this project's ID range, and its body concerns the opencode / 005-Deco estate — so it
is plausibly genuine inbound peer work and is T-2960's to judge, not this task's. Attributing
it by the same `source.project: termlink` field the others carry would have been the easy and
wrong move.

**AC2 — delivery confirmed before treating any of them as an echo.** Matched each envelope's
`payload.summary` against decoded bodies on `framework:pickup` offsets 108-125. Three hits,
one per envelope. Had any returned NONE, that envelope would have been unsent work and the
correct action would have been to file it, not move it.

**AC3 — dispositioned by moving to `processed/`, deliberately and reversibly.** `git mv` into
`.context/pickup/processed/`, which is git-tracked and already holds **P-073 and P-076** — the
siblings from this same cluster that took the processed path instead of the deferred one. So
the three land exactly where their peers already are. Not deleted: the bytes are the only
local record of what was filed, and T-2801's entire argument is that turning a visible backlog
into an invisible one is the trade to avoid.

**AC4 — residual stated, not assumed.** Re-run: **4 → 1 STRANDED**, the remainder being
P-078, exactly as predicted. The check still exits 1, and that is the correct outcome rather
than an incomplete one — reporting "clean" here would require absorbing another task's scope.

**AC5 — the write-location defect was not re-filed.** Already upstream at offset 124; vendored,
so not patched here (G-062).

**A gap found while doing this, filed as T-2966.** The check tells the operator to "drop it
deliberately", and `fw pickup` has no verb that does so — only `send`/`process`/`status`/`list`/
`auto-deferred`/`promote-deferred`. The disposal above therefore happened *outside* the
pipeline's own accounting, via `git mv`. `fw pickup status` cannot distinguish a dispositioned
envelope from one that never existed, and the reason for the drop survives only in this task
file. Recorded rather than worked around silently.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Provenance is measured per envelope, not assumed from one sample.** For each of P-074, P-075 and P-077 the `source.project` / `source.task_id` is read and the originating local task's status recorded. A stranded envelope that turns out to be genuine inbound peer work has the opposite disposition to one that is an echo of our own outbound filing, so the class must be established per file before any of them is dispositioned.
- [x] **Each envelope is confirmed actually delivered upstream before it is treated as an echo.** "It is our own filing" only makes it safe to discard if the filing genuinely reached `framework:pickup`. For each, the corresponding hub offset is located and its body matched against the envelope's `payload.summary`. An envelope with no upstream counterpart is NOT an echo — it is unsent work, and must be filed rather than dropped. This is the AC that decides between "already delivered, safe to close" and "silently lost".
- [x] **The disposition is recorded in the register, and the envelopes are not silently drained.** Each of the three gets a stated outcome with its evidence. Per T-2801 the checker detects and never drains, and auto-draining would convert a visible backlog into an invisible one — the exact trade the check exists to reverse — so any removal from `auto-deferred/` is a deliberate, recorded act, not a cleanup.
- [x] **`check-pickup-deferred-freshness.sh` is re-run and its residual explained rather than assumed clean.** After disposition the check is run again and the remaining STRANDED set is stated explicitly. P-078 is expected to remain and is NOT in this task's scope (it is T-2960's), so a non-zero exit is the predicted outcome, not a failure — the number is reported and attributed instead of being read as "not done".
- [x] **The write-location defect is not re-filed.** `lib/pickup.sh:643` writing outbound envelopes into the project's own inbox is already measured and filed upstream at `framework:pickup` offset 124 (T-2953). This task dispositions the four envelopes that defect stranded here; it does not re-report the defect, and it does not patch the vendored file (G-062).

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

# ---- T-2965 ----
# The three echoes left auto-deferred/ and are preserved in processed/, not deleted.
test -f .context/pickup/processed/P-074-bug-report.yaml
test -f .context/pickup/processed/P-075-bug-report.yaml
test -f .context/pickup/processed/P-077-bug-report.yaml
test ! -e .context/pickup/auto-deferred/P-074-bug-report.yaml
test ! -e .context/pickup/auto-deferred/P-075-bug-report.yaml
test ! -e .context/pickup/auto-deferred/P-077-bug-report.yaml
# Residual is exactly P-078 (T-2960's scope) — stated, not assumed clean.
ls .context/pickup/auto-deferred/ > /tmp/.t2965-resid.txt
test "$(wc -l < /tmp/.t2965-resid.txt)" = "1"
grep -q "^P-078-learning.yaml$" /tmp/.t2965-resid.txt
# The load-bearing claim: each really was delivered upstream. Without this the move is a
# silent discard of three unread reports.
termlink channel subscribe framework:pickup --cursor 108 --limit 40 --json > /tmp/.t2965-v-hub.ndjson 2>&1
python3 -c "import json,base64,sys; rows=[base64.b64decode(json.loads(l)['payload_b64']).decode('utf-8','replace') for l in open('/tmp/.t2965-v-hub.ndjson') if l.strip()]; need=['SUMMARY block omits the section scope','mints local tasks from a project','CORRECTION to P-076']; sys.exit(0 if all(any(n in b for b in rows) for n in need) else 1)"
# Origin tasks are completed, so these are echoes of finished work.
grep -q "^status: work-completed" .tasks/completed/T-2948-pre-push-audit-runs-only-the-structure-s.md
grep -q "^status: work-completed" .tasks/completed/T-2949-pickup-processor-mints-local-tasks-from-.md
grep -q "^status: work-completed" .tasks/completed/T-2942-d8b-10-of-10-recent-handovers-carry-unfi.md

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

**Symptom:** four envelopes sat unpromotable in `.context/pickup/auto-deferred/` for 6-7 days.
The two surfaces that should have shown it both reported normally: `fw pickup auto-deferred
list` printed `blocked-by=? reason=? at=?` without calling it an error, and `fw pickup status`
counted them as ordinary deferred items, indistinguishable from ones deferred yesterday for a
good reason.

**Root cause:** they are this project's own outbound filings, written into its own inbox by
`lib/pickup.sh:643`, then auto-deferred with no breadcrumb. `pickup_write_breadcrumb()`
(T-1425) is what names the blocking task, and both consumers depend on it — `promote-deferred`
resolves the blocker from it (T-2072), `auto-deferred list` displays it — but **neither checks
that it exists**. With no breadcrumb there is no blocker to re-evaluate, so the envelope is not
delayed; it is permanently stranded.

**Why structurally allowed:** the degradation is silent in both directions. A missing breadcrumb
renders as `?` rather than as a fault, and the count surface has no category for "unpromotable",
so a stranded envelope and a healthy deferred one are the same row. The T-2801 checker exists
precisely because of this, and it did fire — but nothing was reading it: it is a deploy-time
check, not a cron canary, and it reached attention only as a side effect of T-2935's AC5
comparing per-member verdicts instead of summary counts. Had that AC been written against the
totals, the regression would have been invisible, because the counts cancelled exactly.

**Prevention:** the breadcrumb hole is vendored (`lib/pickup.sh`) and already filed upstream, as
is the write-location defect (offset 124) — per G-062 neither is patched locally. What is added
here is T-2966: the pipeline offers no verb to dispose a stranded envelope, so every future
disposition must also happen outside its accounting, leaving no durable record of why. Until
that exists, the reason a stranded envelope was dropped lives only in a task file.

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

### 2026-09-16T17:39:46Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2965-three-stranded-pickup-envelopes-have-no-.md
- **Context:** Initial task creation

### 2026-09-16T17:43:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
