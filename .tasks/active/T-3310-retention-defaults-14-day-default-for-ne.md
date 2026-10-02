---
id: T-3310
name: "Retention defaults: 14-day default for new topics, forever only with owner+reason,
  ceiling checked on post"
description: >
  arc-012 step 5 (T-3304 IW-2 C): new topics default to 14 days (forever-by-omission
  ends); forever requires an owner and a reason (the four operator-durable topics
  keep it); a bounded topic past 2x its limit trims oldest on post and logs it loudly;
  forever topics get a size warning, never deletion.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-012]
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
created: 2026-10-01T19:22:54Z
last_update: 2026-10-02T14:44:45Z
date_finished:
revisit_at: 2026-11-15
revisit_evidence_needed: enforcement auto-flipped on every hub, or the backstop canary named who still sends bare forever
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
  - ts: '2026-10-02T14:16:04Z'
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

# T-3310: Retention defaults: 14-day default for new topics, forever only with owner+reason, ceiling checked on post

## Context

arc-012 step 5 (T-3304 IW-2, operator ruling C): bound topic growth by default. Design and
evidence: `docs/reports/T-3304-hub-storage-model.md`. Sub-decisions are walked one at a time
(D1 ruled 2026-10-02, D2/D3 open; see ## Decisions).

Measured 2026-10-02 (local hub, `channel list --json`): 120 topics, 81 forever (32 `inbox:*`,
18 `sidecar:*`, 4 `dm:*`), 39 bounded. Every client asks for forever EXPLICITLY by default
(`cli.rs:1848`, CLI ensure_topic `channel.rs:2941`, MCP `tools.rs:18888`), so a hub-side default
change alone changes almost nothing, and old binaries cannot send owner/reason.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] Hub `channel.create` accepts optional `owner` + `reason`; stores them; the four operator-durable topics keep forever without them
- [ ] A forever create without owner+reason is accepted, labelled `unowned_forever`, warned in the response and hub log, and recorded (time, sender identity) (D1 layer C)
- [ ] `channel list` / `info` (CLI + MCP) show owner, reason and the `unowned_forever` label
- [ ] Enforcement switches on by itself after 14 days with no bare-forever create; after that a bare forever create is refused (-32602) with a message naming the owner/reason flags; a bare-forever create restarts the clock (D1 layer 1)
- [ ] `TERMLINK_FOREVER_REQUIRES_OWNER` = auto (default) / on / never; `never` is reported in `hub status --governor` and `fleet governor-status` (D1 layer 3)
- [ ] Backstop canary: fires if enforcement is still off on 2026-11-15 or any hub runs `never`, names the identities still sending bare forever, and files a task via the T-3267 filer; crontab installed by runme (D1 layer 2)
- [ ] Fixture/unit tests: flips at 14 quiet days, not at 13, clock resets on a bare create; a mutant removing the auto-flip turns them red (D1 layer 4)
- [ ] One shared default-retention table used by CLI `channel create`, CLI auto-create (ensure_topic) and MCP create: `inbox:*` and `dm:*` -> Messages(1000); `state:*` -> Latest; presence/chat-arc/agent-listeners-*/agent-conv-* -> Messages(1000); debris -> Days(7); everything else (incl. `sidecar:*`) -> Days(14) (D2)
- [ ] Ceiling checked on post per D3 (open): a bounded topic past 2x its limit trims oldest on post and logs it; forever topics get a size warning, never deletion
- [ ] A create that omits retention gets Days(14) (debris namespaces keep Days(7))


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
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Measured in this corpus, not invented.
# Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# T-3144 censused 2853 task files: 71 absence assertions, 41 already correct, 30
# not. The convention mostly works — this note is here so the next one is written
# right, because a vacuous leg is invisible until the day the path moves.
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

### 2026-10-02 — D1: hub handling of a bare "forever" create (operator ruling)
- **Chose:** D+ — accept and label as `unowned_forever` now; enforcement (refuse) switches on
  automatically after 14 days with no bare-forever create; backstop canary on 2026-11-15 that
  files a task if enforcement is still off or a hub opts out; opt-out
  `TERMLINK_FOREVER_REQUIRES_OWNER=never` stays possible but is reported daily; fixture tests and
  a mutant pin the auto-flip. revisit_at 2026-11-15 as the human reminder.
- **Why:** every current client and every old binary sends forever explicitly and cannot send an
  owner, so a strict refusal breaks unattended agents' posts to new topics on day one; a silent
  downgrade loses mail older than 14 days without telling the sender (Directive #2). The operator
  accepted D but called the "switch stays off forever" strawman valid, so the flip is driven by
  measured fleet behaviour, not by memory, with an action-filing backstop.
- **Rejected:** A refuse now (breaks fleet, score -44); B silent downgrade to 14 d (-9, silent
  loss); C label only (+37, no path to enforcement). D scored +46 before the layers.
- **Assumed (overturnable):** the 14-day quiet window and the 2026-11-15 backstop date were agent
  proposals; the operator accepted them unchanged.
- **Left open:** D2 new-client defaults (incl. `inbox:*` / `dm:*`), D3 meaning of "2x its limit"
  for day-based topics.

### 2026-10-02 — D2: what new clients ask for by default (operator ruling)
- **Chose:** B — mail by count, everything else by age: `inbox:*` and `dm:*` default to
  Messages(1000) (the bound the hub inbox mirror, `hub channel.rs:243`, and the CLI `dm:*`
  path, `channel.rs:2806`, already apply); every other new topic, including `sidecar:*`,
  defaults to Days(14); existing exceptions kept (`state:*` Latest, debris Days(7),
  presence/chat topics Messages(1000)). One shared table for CLI create, auto-create and MCP create.
- **Why:** bounds every topic while mail is never deleted for being unread, only when outnumbered,
  and a reader that falls behind a trim is told (T-3307/T-3308 gap signal).
- **Rejected:** A uniform 14 d (-11: deletes unread mail, hides rail outages); C mail forever with
  automatic owner (+21: rubber-stamp ownership reopens forever-by-default); D keep-until-read (+11:
  depends on T-3309 read data that is a lead, not a verdict, and is the most code). B scored +46.
- **Left open:** D3, the meaning of "2x its limit" for the post-time ceiling (sets the real margin
  before a mail trim).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-10-01T19:22:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3310-retention-defaults-14-day-default-for-ne.md
- **Context:** Initial task creation

### 2026-10-01T19:23:06Z — status-update [task-update-agent]
- **Change:** tags: +arc:arc-012

### 2026-10-02T14:16:04Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
