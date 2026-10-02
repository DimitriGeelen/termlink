---
id: T-3310
name: "Retention defaults: 14-day default for new topics, forever only with owner+reason,
  ceiling checked on post"
description: >
  arc-012 step 5 (T-3304 IW-2 C): new topics default to 14 days (forever-by-omission
  ends); forever requires an owner and a reason (the four operator-durable topics
  keep it); a bounded topic past 2x its limit trims oldest on post and logs it loudly;
  forever topics get a size warning, never deletion.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:arc-012]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/limits.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/retention_sweeper.rs, crates/termlink-hub/src/router.rs, crates/termlink-hub/src/topic_policy.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/lib.rs, crates/termlink-protocol/src/retention_defaults.rs, runme.sh, scripts/check-forever-owner-freshness.sh, tests/forever-owner-canary-fixtures.sh]
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
last_update: 2026-10-02T16:34:07Z
date_finished: 2026-10-02T16:34:07Z
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
- [x] Hub `channel.create` accepts optional `owner` + `reason`; stores them; the four operator-durable topics keep forever without them
- [x] A forever create without owner+reason is accepted, labelled `unowned_forever`, warned in the response and hub log, and recorded (time, sender identity) (D1 layer C)
- [x] `channel list` / `info` (CLI + MCP) show owner, reason and the `unowned_forever` label
- [x] Enforcement switches on by itself after 14 days with no bare-forever create; after that a bare forever create is refused (-32602) with a message naming the owner/reason flags; a bare-forever create restarts the clock (D1 layer 1)
- [x] `TERMLINK_FOREVER_REQUIRES_OWNER` = auto (default) / on / never; `never` is reported in `hub status --governor` and `fleet governor-status` (D1 layer 3)
- [x] Backstop canary: fires if enforcement is still off on 2026-11-15 or any hub runs `never`, names the identities still sending bare forever, and files ONE task (same filing contract as the T-3267 filer: body-marker de-dup, never under CI, pending-commit manifest); crontab wired into runme (D1 layer 2)
- [x] Fixture/unit tests: flips at 14 quiet days, not at 13, clock resets on a bare create; a mutant removing the auto-flip turns them red (D1 layer 4)
- [x] One shared default-retention table used by CLI `channel create`, CLI auto-create (ensure_topic) and MCP create: `inbox:*` and `dm:*` -> Messages(1000); `state:*` -> Latest; presence/chat-arc/agent-listeners-*/agent-conv-* -> Messages(1000); debris -> Days(7); everything else (incl. `sidecar:*`) -> Days(14) (D2)
- [x] Hub config file `<runtime_dir>/retention.yaml` (+ env overrides) holds the D3 caps and the D1 mode/quiet window; absent file = built-in defaults; invalid file is loud and falls back; per-topic retention still via `channel set-retention` (D3). The D2 per-name defaults stay a shared code table: clients send them explicitly, so a hub file could not change them (see Evolution; configurable defaults -> T-3320)
- [x] Ceilings checked on post (D3): Messages(N) trims at 2N back to N; Days topics trim at 10,000 records and when the oldest is past 2x the window (back to the window); bounded topics trim at 64 MB live; Latest/LatestPerCvKey compact past 10,000; every trim logged and counted
- [x] Forever topics: warn only (hub log + counter + `over_ceiling` in channel list) at 10,000 records or 64 MB live; never deleted (D3)
- [x] Trim/warn counters in `hub status --governor` and `fleet governor-status`
- [x] A create that omits retention gets Days(14) (debris namespaces keep Days(7))


## Verification

cargo test -p termlink-protocol retention_defaults > /tmp/.t3310-proto 2>&1 && grep -q "3 passed" /tmp/.t3310-proto
cargo test -p termlink-bus > /tmp/.t3310-bus 2>&1 && grep -q "test result: ok" /tmp/.t3310-bus && ! grep -q "FAILED" /tmp/.t3310-bus
cargo test -p termlink-hub --lib topic_policy > /tmp/.t3310-pol 2>&1 && grep -q "5 passed" /tmp/.t3310-pol
cargo test -p termlink-hub --lib t3310 > /tmp/.t3310-hub 2>&1 && grep -q "2 passed" /tmp/.t3310-hub
cargo test -p termlink --bin termlink t3310 > /tmp/.t3310-cli 2>&1 && grep -q "test result: ok" /tmp/.t3310-cli && ! grep -q "FAILED" /tmp/.t3310-cli
bash tests/forever-owner-canary-fixtures.sh > /tmp/.t3310-fx 2>&1 && grep -q "16 passed, 0 failed" /tmp/.t3310-fx
bash tests/runme-fixtures.sh > /tmp/.t3310-runme 2>&1 && grep -q "0 failed" /tmp/.t3310-runme
bash scripts/check-canary-log-hygiene.sh > /tmp/.t3310-hyg 2>&1
grep -q "forever-owner-canary.crontab" runme.sh

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

### 2026-10-02 — client defaults, not hub defaults, were the forever source
- **What changed:** every client (CLI create, CLI --ensure-topic, MCP create) asked for
  `forever` EXPLICITLY, so the IW-2 plan ("flip the hub default") would have changed almost
  nothing, and a strict "forever needs an owner" would have broken every old binary.
- **Plan impact:** D1 became accept-label-record now with evidence-driven auto-enforcement
  (operator ruling D+); the default table moved to `termlink-protocol` and clients send it
  explicitly (so an older hub gets bounded topics too).
- **Triggered:** D1/D2/D3 operator rulings (see Decisions).

### 2026-10-02 — sweeps never shrink log files
- **What changed:** measured `agent-presence` at 31.6 MB on disk vs 0.55 MB live; bus 49 MB vs
  12.4 MB. Index-only trims bound the record count, not the disk.
- **Triggered:** T-3322 (rewrite mostly-dead logs, offsets unchanged), per D3 = C.

### 2026-10-02 — ceiling check cost
- **What changed:** checking count/bytes/oldest on every post is O(records) per post; it made
  the bus suite 71 s and pushed the T-2258 concurrency test past its 10 s bound under load.
- **Plan impact:** checks run every Nth post (latest 1, messages N/8 capped at 64, else 64);
  the 2x margin absorbs the delay. Bus suite back to ~11 s.

### 2026-10-02 — D2 table is code, not config
- **What changed:** the AC said the hub config file holds the D2 table. It cannot usefully:
  clients compute the per-name default and send it explicitly (needed so old hubs get it).
  The hub file holds what the hub decides: D3 caps, D1 mode and quiet window.
- **Plan impact:** AC reworded; making the per-name defaults configurable fleet-wide belongs
  with the settings surface (T-3320), which needs clients to read shared config.

### 2026-10-02 — sender clocks
- **What changed:** age trims use the record's sender-supplied timestamp (as the sweeper always
  did), so a sender with a badly wrong clock has its records aged out at once. Unchanged risk,
  now also on post; noted, not fixed here.

### 2026-10-02 — operator side-points became inceptions
- T-3319 learn from message traffic (now), T-3320 settings surface (next), T-3321 compaction
  (later, DEFER behind T-3319). T-3319 research confirmed T-3310 does not trim the raw material
  it wants: framework:pickup and channel:learnings are operator-durable forever.

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

### 2026-10-02 — D3: ceilings checked on post (operator ruling)
- **Chose:** C — count, age and live-size ceilings on post, read from a hub config file and
  changeable per topic; 2x hysteresis (trim at 2x, back to 1x). Defaults: Messages(N) 2N; Days
  topics 10,000 records and oldest past 2x window; 64 MB live for bounded topics; Latest/LPCK
  compact past 10,000; Forever warn-only at 10,000 / 64 MB, never deleted. Disk reclamation
  (rewrite a log over 8 MB and more than half dead, offsets unchanged) is its own build task.
- **Why:** operator asked for count + age + size, configurable. Measured 2026-10-02: sweeps delete
  index rows but log files never shrink (agent-presence 31.6 MB on disk vs 0.55 MB live, ~98%
  dead; bus 49 MB on disk vs 12.4 MB live), so only C bounds disk too.
- **Rejected:** A count only (+28: large payloads evade it); B no reclamation (+37: disk still
  grows); D rely on sweeper (-20: off on .122/.121). C +41, reliability scored +1 for rewrite risk.
- **Assumed (overturnable):** all numbers are agent proposals accepted unchanged.
- **Operator side-points recorded as inceptions:** T-3319 learn from message traffic (now),
  T-3320 settings surface (next), T-3321 compaction (later, DEFER behind T-3319).

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

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1abcbd99
- **Timestamp:** 2026-10-02T16:37:21Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Reviewer:** inline
- **Findings:** none

### 2026-10-02T16:34:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
