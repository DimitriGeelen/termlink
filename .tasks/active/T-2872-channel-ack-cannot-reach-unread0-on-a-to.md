---
id: T-2872
name: "channel ack cannot reach unread=0 on a topic the acking identity also posts
  to"
description: >
  unread is computed as latest - receipt_up_to, and 'channel ack' posts the receipt
  as a normal envelope, which advances latest. So each ack manufactures exactly the
  one unread item it was meant to clear. Measured deterministically: three consecutive
  acks on dm:d1993c2c3ec44c94:deadbeefdeadbeef moved latest 7->8->9 and receipt_up_to
  6->7->8 with unread pinned at 1 throughout. Affects every topic the local identity
  posts to, including agent-chat-arc, so the T-2838 receiver-ack-lag guard decays
  back to non-zero after every broadcast.

status: started-work
workflow_type: build
owner: claude-code
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
created: 2026-08-31T19:46:32Z
last_update: 2026-08-31T19:52:39Z
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
  - ts: '2026-08-31T19:47:36Z'
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

# T-2872: channel ack cannot reach unread=0 on a topic the acking identity also posts to

## Context

> **CORRECTED 2026-09-01 — the headline defect does not reproduce against the
> shipping verb.** The filing below is preserved verbatim because the record of a
> wrong call is worth more than a quiet edit.

`channel unread` was measured on the **same topic**, driving the same three
consecutive acks:

```
ack #1 -> last_offset=10  up_to=9   unread_count=0
ack #2 -> last_offset=11  up_to=10  unread_count=0
ack #3 -> last_offset=12  up_to=11  unread_count=0
```

The `latest - up_to = 1` gap reproduces exactly as filed. `unread_count` is **0**
every time. Both are true, and only one of them is the product's answer.

`count_unread` (`crates/termlink-cli/src/commands/channel.rs`) has skipped
`receipt` via `UNREAD_META_TYPES` since **T-1332**, so `latest - receipt_up_to` is
not the definition of unread — it is a subtraction this task performed by hand and
then attributed to the system. The convergence the task demanded was already the
shipping behaviour.

This is the failure mode the filing itself names one paragraph later — *"the
assertion is adjacent to the property"* — committed by the filing. The measurement
was real; the inference from it was not checked against the verb.

**What survives, and it is the useful half.** *"The defect is only visible when you
ask for convergence, and no test asks"* was **correct**. The six existing
`count_unread` tests are static-slice tests; `count_unread_excludes_meta_envelopes`
places a receipt mid-list but never exercises the ack LOOP, where each ack appends a
receipt and advances the tail. Nothing asserted that repeated acking terminates.
That test now exists and is load-bearing: removing `"receipt"` from
`UNREAD_META_TYPES` turns it red, so had the code carried the filed defect this test
would have caught it.

**Prior art nobody joined to this filing.** `PL-317` (T-2589, 2026-08-10) describes
this mechanism precisely — *"a receipt acking up_to=N is ITSELF an envelope at an
offset > N included in the topic total, so the difference never reaches 0"* — and
records the fix. `check-outbox.sh` was repaired there; `check-receiver-ack-lag.sh`
carries its own note (line 34) that `lag: 0` was once unreachable and now uses
`latest_content_offset`. Three surfaces had already been through this class. The
learning existed and did not reach the re-discovery, which is the more durable
finding than the one filed.

### As filed (2026-08-31)

`unread` is `latest - receipt_up_to`. `channel ack` posts the receipt as an ordinary
envelope on the topic, which advances `latest`. So an ack sets `receipt_up_to` to the
pre-post latest and simultaneously creates one new envelope above it: the receipt itself.
The count it was issued to clear is reproduced by the act of clearing it.

Measured on `dm:d1993c2c3ec44c94:deadbeefdeadbeef`, three consecutive acks:

```
ack #1 -> latest=7  receipt_up_to=6  unread=1
ack #2 -> latest=8  receipt_up_to=7  unread=1
ack #3 -> latest=9  receipt_up_to=8  unread=1
```

Deterministic, not a race — the gap is exactly 1 every time and does not decay.

**Why nothing caught it.** A single-ack test passes: unread *does* drop, from whatever it
was to 1. The defect is only visible when you ask for convergence, and no test asks. This
is the same shape as a guard that proves its own copy rather than the live path — the
assertion is adjacent to the property.

**Scope is wider than one DM topic.** It applies to any topic the local identity also
posts to, which includes `agent-chat-arc`. That has a direct consequence for work
recorded as complete: the T-2838 receiver-ack-lag guard was brought to lag=0 earlier
today and is already back to lag=7 after two broadcasts from this session. The remediation
is self-defeating on precisely the topics we participate in — acking is not a fix there,
it is a reset that the next post undoes. `framework:pickup` stays at 0 only because we
have not posted to it since.

**The second-order harm is the reason to prioritise it.** An unread count that cannot
reach zero is an alert that is always on, and an alert that is always on stops being
read. `/check-arc` surfaces this number to the operator. A peer made the general form of
this argument to us about flaky gates training people into `--force`; this is the same
mechanism in the inbox rather than the gate.

**Deliberately not decided here:** whether the receipt should be excluded from the count
or should carry its own offset in `up_to`. Those differ in what a PEER sees of our read
state, which is a protocol-visible choice and belongs in the first AC, not in a hurried
fix.


## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] **Establish where the off-by-one belongs.** Answered: **neither (a) nor (b) — there is no off-by-one in `unread`.** `count_unread` already excludes `receipt` (`UNREAD_META_TYPES`, T-1332), so option (a) would move an offset that nothing reads and option (b) would re-exclude what is already excluded. The gap lives only in the hand-computed `latest - up_to`, which no shipping surface uses. Recorded rather than implemented, because implementing either would have been a change with no defect under it.
- [x] **A repeated ack converges.** Verified twice, independently. Live: three consecutive acks on `dm:d1993c2c3ec44c94:deadbeefdeadbeef` hold `unread_count=0` while `last_offset - up_to` sits at 1 throughout. In-tree: `count_unread_converges_across_repeated_acks` models the ack loop (ack through tail, append receipt above it, ×3) and asserts both the reproduced gap of 1 and the true count of 0 — so the discrepancy is legible at the point of failure instead of being re-discovered.
- [x] **The regression test is load-bearing.** Removing `"receipt"` from `UNREAD_META_TYPES` fails it (`MUTANT_RC=101`); restoring returns 7/7 green. A test that cannot go red is not a test, and this one goes red on precisely the defect that was filed.
- [ ] **Decide whether self-authored posts should count toward one's OWN lag.** This is the residual, and it is a genuine judgement rather than a bug. `check-receiver-ack-lag.sh` currently reports `agent-chat-arc lag=7` for our own identity — classified `ok` (threshold 25), guard rc=0, and it *does* reach 0 when acked; it re-grows because we posted, which for a frontier metric is correct, not self-defeating. The open question is narrower than the filing: should envelopes an identity authored itself count as that identity's own unacked backlog? Arguably not — you have read what you wrote — but it is protocol-visible to peers reading our receipt frontier, so it is not a change to make in passing. **Owner: human** (see Human AC).

### Human
- [ ] [REVIEW] Decide whether an identity's OWN posts should count toward its own ack-lag / unread frontier.
  **Steps:**
  1. `timeout 240 bash scripts/check-receiver-ack-lag.sh`
  2. Read the `agent-chat-arc` row. Today it shows `ok  d1993c2c3ec44c94  lag=7` — that identity is us, and those 7 are largely our own broadcasts.
  3. Decide one of: **(A) leave as-is** — lag counts all unacked content regardless of author, so posting raises your own lag until you ack; simple, and already correct against its own definition. **(B) exclude self-authored envelopes** from an identity's own lag/unread — "you have read what you wrote"; quieter, but changes what a peer infers from our receipt frontier, so it is protocol-visible.
  **Expected:** A or B recorded here, with one line of reasoning. If **B**, a follow-up build task is filed against `check-receiver-ack-lag.sh` and `count_unread`; if **A**, no code changes and this task closes.
  **If not:** leave unchecked — the guard is green at threshold 25 and nothing is degrading while this sits.


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

grep -q "count_unread_converges_across_repeated_acks" crates/termlink-cli/src/commands/channel.rs
grep -q "\"receipt\", \"reaction\", \"redaction\", \"edit\", \"topic_metadata\"" crates/termlink-cli/src/commands/channel.rs
timeout 600 cargo test -p termlink count_unread > /tmp/.t2872-tests.txt 2>&1 && grep -q "count_unread_converges_across_repeated_acks ... ok" /tmp/.t2872-tests.txt
timeout 240 bash scripts/check-receiver-ack-lag.sh > /tmp/.t2872-lag.txt 2>&1

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

### 2026-08-31T19:46:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2872-channel-ack-cannot-reach-unread0-on-a-to.md
- **Context:** Initial task creation

### 2026-08-31T19:47:36Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
