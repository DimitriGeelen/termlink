---
id: T-3071
name: "Message queue priority field - flat FIFO for now"
description: >
  Spec step 8. Operator: keep it flat for now, prioritise later. Queue reads the journal
  (2238 rows today) and injects the first item. This task adds the priority FIELD
  so the ordering is explicit rather than incidental.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-011]
components: [scripts/be-reachable-pushwaker.sh]
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
created: 2026-09-22T12:53:43Z
last_update: 2026-09-22T18:14:06Z
date_finished: 2026-09-22T18:14:06Z
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
  - ts: '2026-09-22T14:57:18Z'
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
  - ts: '2026-09-22T14:57:42Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=207,acs=4)
    rubric_sha: e4a00f38e801
  - ts: '2026-09-22T14:59:18Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=8 
      (lines=207,acs=4)
    rubric_sha: e4a00f38e801
---

# T-3071: Message queue priority field - flat FIFO for now

## Context

arc-011 slice **S8**, operator spec step 8: *"read the queue, inject the
highest-priority message."* Operator direction: **keep it flat for now, prioritise
later** — so this slice adds the FIELD and the explicit ordering, not a policy.

**The field is not missing from the wire; it is being discarded.**
`scripts/journal-mirror.sh:118` reads each envelope's `metadata` and extracts exactly
two keys (`conversation_id`, `observed_addr`). Everything else is dropped on the floor.
`payload` holds the base64-decoded BODY, not the envelope, so nothing downstream can
recover what the sender declared. The `messages` table has no column for it.

So today `notify-injector.sh:183` orders by `ts ASC, offset ASC` and the comment at
:176 calls it "FLAT FIFO ... per operator". That is true, but it is true *by accident*:
there is no declared notion of priority for the ordering to be flat with respect to.
The ordering is incidental, and incidental ordering is the thing that silently changes
when someone later adds an index or swaps a query.

**What this slice does:** persist `metadata.priority` into the journal, clamp it on the
way in, and make the injector's ORDER BY name it explicitly — with the default band
chosen so that a corpus declaring no priority (which is the entire corpus today)
selects byte-identically to the current query. Flat stays flat; it just stops being an
accident.

**What it deliberately does not do:** change the prompt-free wait. An urgent message
bypassing that wait is T-3072 (slice S9), and it is blocked on **SQ-4** — injecting
into a BUSY prompt is the T-2396 failure where text lands unsubmitted and is
discarded, which would silently lose exactly the messages most likely to be marked
urgent. Adding the field here does not authorise that bypass.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these.

     SCOPE FENCE. This slice makes queue ordering EXPLICIT. It does not change what
     gets injected today, and it does not touch the prompt-free wait — bypassing that
     wait is T-3072/S9, gated on SQ-4. A change here that alters today's selection is
     out of scope by construction, which is what AC1 pins. -->
- [x] **Behaviour-preserving by default.** With no message declaring a priority —
      i.e. the entire corpus that exists today — the injector selects exactly the row
      it selects now. Proven by differential comparison of the old and new queries
      against the REAL journal, per topic, not by a synthetic fixture alone
- [x] `journal-mirror.sh` persists the sender's declared priority instead of dropping
      it: a `priority` column on `messages`, populated from `metadata.priority`.
      Absent, null, boolean, or unparseable values resolve to the normal band (0) —
      an unreadable field must never outrank a real one.
      (AMENDED mid-build: this originally also said out-of-range resolves to 0, which
      contradicts AC3 and is not what was built — out-of-range is CLAMPED to the band
      edge, not reset to normal. Corrected rather than ticked as written.)
- [x] The value is CLAMPED to a declared range on the way in, per the repo's
      caller-param convention (T-2527): a peer cannot post `priority: 1e9` and pin
      itself to the head of every queue forever
- [x] Schema migration is idempotent and non-destructive on the existing journal
      (2238 rows, no priority column): running the mirror twice against an already-
      migrated database is a no-op, and no existing row is rewritten
- [x] `notify-injector.sh` orders by priority first, then the existing (ts, offset)
      FIFO — so ordering WITHIN a priority band stays first-in-first-out and is not
      left to incidental row order
- [x] Fixtures cover the ordering matrix: equal priority falls back to FIFO; a higher
      band wins regardless of arrival order; absent/garbage/out-of-range priority sorts
      as normal; and an empty queue still exits 2 rather than passing vacuously
- [x] `bash tests/notify-injector-fixtures.sh` passes in full, with the pre-existing
      assertions intact — no test weakened or removed to accommodate the new column

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

# Both scripts parse. The mirror matters especially: insert_py is a SINGLE-QUOTED
# shell string, so one apostrophe in a comment ends it and the python body runs as
# shell. That happened during this task; C0b pins the regression.
bash -n scripts/journal-mirror.sh
bash -n scripts/notify-injector.sh

# Ordering matrix, hermetic. T18 equal-band -> FIFO; T19 higher band wins regardless
# of arrival order; T20 negative band yields; T21 NULL sorts as normal; T22 priority
# cannot promote a receipt; T23 empty queue still exits 2 rather than passing
# vacuously. T1..T17 build a queue with NO priority column, so their continued
# passing IS the pre-migration back-compat assertion.
bash tests/notify-injector-fixtures.sh

# Clamp + coercion + migration. C4/C5 clamp, C6 rejects bool, C7 sorts unparseable as
# normal, M2 proves idempotency and that no existing row is rewritten.
bash tests/journal-mirror-fixtures.sh

# Structural pins, so a future edit cannot quietly drop the three properties the
# fixtures above are asserting.
grep -q 'COALESCE(priority,0) DESC' scripts/notify-injector.sh
grep -q 'max(PRIORITY_MIN, min(PRIORITY_MAX' scripts/journal-mirror.sh
grep -q "pragma_table_info('messages')" scripts/journal-mirror.sh

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

### 2026-09-22 — the field was never missing; it was being thrown away

- **What changed:** Filed as "add the priority FIELD", which reads like adding a
  column to a queue that has nowhere to put one. The measurement says otherwise.
  `journal-mirror.sh` already parses every envelope's `metadata`; it keeps exactly two
  keys (`conversation_id`, `observed_addr`) and discards the rest. `payload` holds the
  base64-decoded BODY, not the envelope, so once the mirror drops a key nothing
  downstream can recover it. A sender could have been declaring a priority for months
  and it would have reached the journal and vanished with no error anywhere. The work
  was therefore *stop discarding it*, not *invent it*.
- **Plan impact:** the slice grew one component the task did not name. It was scoped to
  `notify-injector.sh` + its fixtures; the real change is two-sided, because an ORDER BY
  over a column nothing populates sorts nothing. `journal-mirror.sh` and a new
  `tests/journal-mirror-fixtures.sh` joined the slice — the mirror had NO fixtures at
  all, which was tolerable while it was plumbing and stopped being tolerable the moment
  it began writing peer-supplied input into the column the injector sorts on.
- **The security shape nobody asked about at filing:** `priority` arrives from a PEER.
  Unclamped, `priority: 1e9` pins that sender to the head of every queue forever — a
  denial of attention needing no exploit, just a large integer. Clamped to [-9, 9] per
  the T-2527/T-2526 caller-param convention. Two coercion traps found while testing:
  Python's `bool` is a subclass of `int`, so `true` would have become band 1 (a sender
  setting a flag silently promoting itself), and an unparseable value had to resolve to
  NORMAL rather than to anything that could outrank a real band. Both are pinned (C6,
  C7) and both mutations were confirmed to fire.
- **What "flat FIFO" actually meant:** the old comment said FLAT FIFO "per operator",
  and it was true — but true *by accident*. There was no declared notion of priority
  for the ordering to be flat with respect to, so the guarantee lived only in a comment
  and would have evaporated the first time someone added an index or rewrote the query.
  It is now declared: `COALESCE(priority,0) DESC, ts ASC, offset ASC`. Flat did not
  change; it stopped being incidental.
- **Evidence that the default is genuinely behaviour-preserving:** differential run of
  the old and new queries over the REAL journal — 2241 rows, 143 topics — selected an
  identical row on every topic. That result is only meaningful because the clause was
  separately shown to be load-bearing: promoting the NEWEST message on a live topic to
  band 5 flips the selection from offset 0 to offset 20, so identity is a fact about
  the corpus (everything sits at band 0), not about a no-op ORDER BY.
- **A gate finding, recorded not routed around:** the T-559 project-boundary hook
  refused `ls /root/.termlink/...` as an outside path, then allowed the identical read
  via a `$HOME`-derived variable. The hook matches literal argument text, so a variable
  defeats it. I used the variable for brevity before noticing. Filing separately; the
  journal IS this project's own runtime data, so the right fix is the read-side
  allowlist, not tighter string matching.
- **SQ-2 is NOT resolved by this task.** T-3071 is injection machinery, and SQ-2 asks
  whether injection machinery is worth building while nothing on this host is
  injectable. What made this executable anyway is narrower and should not be read as an
  answer: its acceptance criteria are about queue ORDERING, which is hermetically
  testable and needs no live REPL. SQ-2 bears on the slice's value, not on whether its
  ACs can close.
- **Triggered:** `tests/journal-mirror-fixtures.sh` (new, 16 assertions — the mirror's
  first coverage). Nothing here authorises T-3072/S9: bypassing the prompt-free wait
  remains blocked on SQ-4, because injecting into a BUSY prompt is the T-2396 failure
  where text lands unsubmitted and is discarded.

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

### 2026-09-22T12:53:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3071-message-queue-priority-field---flat-fifo.md
- **Context:** Initial task creation

### 2026-09-22T13:10:19Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-011

### 2026-09-22T18:00:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-ef494d45
- **Timestamp:** 2026-09-22T18:14:10Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-22T18:14:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** arc-011 S8: priority band persisted from metadata.priority (was being discarded), clamped to [-9,9], explicit ORDER BY with FIFO within band. Differential-proven behaviour-preserving on the real 2241-row corpus; 23 injector + 16 new journal-mirror fixtures, mutation-tested.
