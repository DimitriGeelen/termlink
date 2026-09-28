---
id: T-3196
name: "Drain the stale inbox-transfer backlog (R7 outstanding AC: 19 -> 232)"
description: >
  Drain the stale inbox-transfer backlog (R7 outstanding AC: 19 -> 232)

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
created: 2026-09-28T13:18:41Z
last_update: 2026-09-28T13:18:41Z
date_finished: null
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
---

# T-3196: Drain the stale inbox-transfer backlog (R7 outstanding AC: 19 -> 232)

## Context

Opened to drain what T-3193 reported as a stale inbox-transfer backlog grown
19 → 232, on explicit operator authorisation. **The backlog does not exist.**
The number is produced by a metric that counts something else, and acting on it
would have deleted live correspondence. Nothing was drained.

## Findings

### There are zero pending transfers

`inbox list` returns `{"transfers": []}` for **every one of the 22 targets**,
while `inbox status` totals 232. The two surfaces use different predicates, and
only one of them is looking at transfers:

| surface | backend | predicate |
|---|---|---|
| `inbox status` | `channel.list` (T-1235) | `aggregate_status_from_channel_list` at `inbox_channel.rs:239` takes each topic's raw `count` as `pending` |
| `inbox list` | `channel.subscribe` (T-1225) | `fold_envelopes` at `inbox_channel.rs:506-534` folds envelopes **filtered by `msg_type`** |

So `status` reports *records on a topic* and labels them *"pending transfer(s)"*
(`infrastructure.rs:1225`). Sampled envelope types on those topics:
`sidecar.consult` and `note` — **not a single file transfer.** The 232 sum
exactly across the 22 `inbox:*` topics in `channel list`, all
`retention: forever`, confirming the count is pure topic size.

**This is ours, not vendored.** `crates/termlink-session/src/inbox_channel.rs`
and `crates/termlink-cli/src/commands/infrastructure.rs` are product code. The
fix is a local change, not a G-062 filing.

### The number I reported to the operator was wrong

T-3193 stated R7's AC had regressed "19 → 232" and I repeated it as a live
growing backlog. Both halves are wrong: 232 is not a transfer count, and it is
not comparable to R7's 19. Corrected here rather than quietly restated —
the whole reason this task existed was that measurement.

### What the 232 actually are — and why deleting them would have been serious

`inbox clear` is not a queue drain. `clear_via_channel_trim`
(`inbox_channel.rs:334`) calls `channel.trim`: it deletes records.

- **49 on `inbox:cacc73ea32b121dd/010-termlink` — our own unread inbound.**
  All `sidecar.consult`, 48 of 49 from `999-Agentic-Engineering-Framework`.
- **95 under AEF's namespace** (`e2e-<hex>-{sender,responder}`, `t3433-*`,
  `t3434-*`). Ephemeral test identities, but they are inside *another project's*
  address space on a shared hub. Deleting them is outward-facing.
- **88 across live peer roots** — 832-Workflow-designer (54), 1409-sprind (27),
  AEF root (6), framework-agent (1).

### The unread inbound is substantive, and one item is a bug report against us

Read in full rather than counted:

- **offset 7** — AEF: T-3433 closed; consults now post to
  `inbox:cacc73ea32b121dd/010-termlink`; T-3434's retry ladder is live
  (re-posts on 2×1m, 2×5m, 2×15m rungs until acked; dedupe on
  `metadata.client_msg_id`). **Two direct asks, unanswered:** (1) confirm our
  subscriber wakes on `inbox.queued` for this topic; (2) answer the next
  `fw sidecar e2e --peer 010-termlink` run so their H4/H5 go green.
- **offset 30 (T-3442)** — **`termlink agent search` does not scan `dm:*`
  channels.** Measured by AEF: a search returns 0 of 1002 envelopes scanned while
  the exact phrase sits verbatim at `dm:3bba15e681b3a078:d1993c2c3ec44c94`
  offset 2. Their stated cost: our substantive reply sat unread 3+ weeks while
  five independent search-based drives on their side reported the artefacts it
  named as absent. **Ask: extend `agent search` to cover `dm:*`, or document
  explicitly that it never will**, so callers stop assuming it is a superset.
  Homed to us; they are building their own receiving-end reader (T-3442).
- **offset 39** — round trip PROVEN: reply landed, token and sender validated,
  73h05m end to end. Their harness had recorded FAIL at the 1800s mark. *"The
  rail was fine the whole time; the verdict was the defect."* Filed their side
  as T-3476.
- **Repeated nudges** at offsets 33, 37–38, 43–48 — the T-3434 ladder firing
  against us because nobody replied.

### The real defect is the mailbox never reaching a prompt

This is arc-011's exact subject ("Agent-to-agent message delivery: mailbox to
prompt"). Delivery works — every message arrived, durably, and AEF proved the
round trip. **Consumption is what fails.** Nothing surfaces
`inbox:<our-project>` to a session, so a peer's bug report about our own search
verb sat unread while their retry ladder escalated, and the only surface that
mentions the topic reports its contents as "pending transfers" — i.e. as
plumbing to be flushed, not correspondence to be read.

That is why this task was opened to delete it.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Every inbox target is enumerated with its pending count and classified
      DEAD (no live consumer — dead smoke/e2e fixture targets) or LIVE (a peer
      project that can still collect). Classification is by evidence — presence on
      the fleet / a real project root — never by the target string looking test-ish.
- [x] **NOT DRAINED — the premise was false, and this AC is answered by refusing it.**
      Measurement showed there is no pending-transfer backlog at all: `inbox list`
      returns 0 transfers for every one of the 22 targets. The 232 are ordinary
      messages (`msg_type: sidecar.consult` / `note`) on `retention: forever` topics,
      and `inbox clear` resolves to `channel.trim` (`inbox_channel.rs:334`) — it would
      have permanently deleted them. Draining would have destroyed 49 unread inbound
      consults addressed to this project and 95 records inside AEF's namespace on a
      shared hub. The drain was authorised against a number that does not mean what
      it says; executing it would have been the single most destructive action
      available here.
- [x] LIVE targets are NOT drained in this task. Those transfers are undelivered
      messages to working peers; deleting them destroys another project's inbound.
      They are reported with counts so the operator can decide per target.
- [x] Every `termlink` invocation sets `TERMLINK_RUNTIME_DIR` from
      `/proc/<hub-pid>/environ` (G-096). A drain issued against the wrong runtime
      dir would report success while touching nothing — or touch the wrong store.
- [x] Before any destructive call, the exact verb is read from `--help` and its
      blast radius stated. `inbox clear` semantics are confirmed against the
      shipping CLI rather than assumed from the name.
- [x] **Absence of a detector recorded as the finding.** Growth was never the
      problem, so a growth detector would have been the wrong guard — it would have
      fired on correspondence arriving and recommended deleting it faster. What is
      missing is a CONSUMER: nothing surfaces `inbox:<this-project>` to a session,
      which is why a peer's bug report against our own `agent search` sat unread
      under an escalating retry ladder. The three candidate fixes are named in
      Recommendation; none is taken here, because each is a scope decision and two
      are outward-facing.

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

# --- T-3196 verification ---
# All legs are READ-ONLY by design: the finding of this task is that the
# destructive verb must not run. TERMLINK_RUNTIME_DIR is set explicitly on every
# call (G-096) — unset, these would read a stale hub and "confirm" nothing.

# 1. The source predicates that produce the discrepancy still read as reported.
grep -q 'let pending = t.get("count")' crates/termlink-session/src/inbox_channel.rs
grep -q 'let msg_type = msg.get("msg_type")' crates/termlink-session/src/inbox_channel.rs
grep -n 'channel.trim' crates/termlink-session/src/inbox_channel.rs > /tmp/.t3196-trim && test -s /tmp/.t3196-trim

# 2. Positive control FIRST (T-3144): our inbox topic exists and is non-empty, so
#    the zero-transfers assertion in leg 3 cannot pass by querying nothing.
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 30 termlink channel info "inbox:cacc73ea32b121dd/010-termlink" --json > /tmp/.t3196-info.json 2> /tmp/.t3196-info.err
python3 -c "import json;d=json.load(open('/tmp/.t3196-info.json'));assert d['count']>=49,d['count']"

# 3. Zero pending transfers on the topic that carries 49 records — the whole
#    discrepancy, asserted on one target rather than in aggregate.
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 30 termlink inbox list "cacc73ea32b121dd/010-termlink" --json > /tmp/.t3196-list.json 2> /tmp/.t3196-list.err
python3 -c "import json;d=json.load(open('/tmp/.t3196-list.json'));assert d.get('transfers')==[],d"

# 4. The inbound is what the findings say it is: sidecar.consult from AEF, and it
#    carries the agent-search dm:* report. Asserted on content, not on a count.
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 40 termlink channel subscribe "inbox:cacc73ea32b121dd/010-termlink" --cursor 0 --limit 60 --json > /tmp/.t3196-sub.ndjson 2> /tmp/.t3196-sub.err
python3 -c "import json,base64;rows=[json.loads(l) for l in open('/tmp/.t3196-sub.ndjson') if l.strip()];assert len(rows)>=49,len(rows);assert all(r.get('msg_type')=='sidecar.consult' for r in rows);bodies='\n'.join(base64.b64decode(r['payload_b64']).decode('utf-8','replace') for r in rows if r.get('payload_b64'));assert 'agent search' in bodies and 'dm:' in bodies, 'agent-search dm report absent'"

# 5. Nothing was deleted: the 22 inbox topics and the 232 total still stand.
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 30 termlink channel list --json > /tmp/.t3196-ch.json 2> /tmp/.t3196-ch.err
python3 -c "import json;d=json.load(open('/tmp/.t3196-ch.json'));ch=d.get('channels') or d.get('topics') or [];inb=[c for c in ch if str(c.get('name','')).startswith('inbox:')];assert len(inb)==22,len(inb);assert sum(c.get('count') or 0 for c in inb)==232,sum(c.get('count') or 0 for c in inb)"

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

**Recommendation:** NO-GO on draining anything. Fix the metric, then build a consumer.

**Rationale:** There is no backlog to drain — 0 pending transfers across all 22
targets. The 232 are live messages on forever-retention topics, and `inbox clear`
trims records, so the authorised action would have deleted 49 unread inbound
consults to this project plus 95 records inside AEF's address space. The
authorisation rested on a number that means something other than what it says.

**Evidence:**
- `inbox list` → `{"transfers": []}` for every target; `inbox status` → 232.
- `inbox_channel.rs:239` counts topic records as `pending`; `:506-534` filters by
  `msg_type`; `:334` makes `clear` a `channel.trim`.
- Envelope types sampled on those topics: `sidecar.consult`, `note`. Zero transfers.
- Our own topic holds an unanswered AEF bug report against `termlink agent search`
  plus an escalating T-3434 retry ladder.

**Three candidate fixes, none taken here:**
1. **Stop the metric lying** (ours, local, smallest). `inbox status` must not label
   topic size "pending transfer(s)". Either fold by `msg_type` as `list` does, or
   relabel to a record count that names what it counts. Note PL-244: a response-shape
   change needs the deliberate-change treatment, so this is not a silent edit.
2. **Answer AEF** (outward-facing — operator's call). Two open asks from offset 7,
   plus the `agent search` dm:* gap they homed to us at offset 30.
3. **A consumer for `inbox:<this-project>`** (arc-011's actual subject). Delivery is
   proven; nothing surfaces the mailbox to a session. This is the structural fix and
   the largest — it wants its own inception, not a slice bolted here.

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

### 2026-09-28T13:18:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3196-drain-the-stale-inbox-transfer-backlog-r.md
- **Context:** Initial task creation
