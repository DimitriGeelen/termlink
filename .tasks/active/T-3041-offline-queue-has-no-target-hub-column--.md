---
id: T-3041
name: "Offline queue has no target-hub column — a queued cross-hub post flushes to
  the wrong hub"
description: >
  channel post --hub <addr> queues to a SHARED outbound.sqlite whose pending_posts
  table carries no target-hub column, and BusClient.flush() drains every row to the
  single addr fixed at its construction. A cross-post queued while hub B is down is
  therefore delivered to whichever hub the next post targets — a misdelivery that
  reports success and pops the row, not a drop. Found by source read during AEF T-3397
  design review; NOT yet reproduced (PL-367).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [bug, substrate, comms]
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
created: 2026-09-21T12:45:56Z
last_update: '2026-09-21T13:49:24Z'
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
  - ts: '2026-09-21T12:47:10Z'
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
  - ts: '2026-09-21T13:49:24Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 8
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=8 (lines=265,acs=8)
    rubric_sha: e4a00f38e801
---

# T-3041: Offline queue has no target-hub column — a queued cross-hub post flushes to the wrong hub

## Context

`channel post --hub <addr>` queues to a SHARED queue whose rows carry no destination,
and the flush sends every row to whichever hub the current client points at. A
cross-post queued while the target hub is down is therefore delivered to a DIFFERENT
hub — a misdelivery that reports success and pops the row, not a drop.

Three facts, read in source:

1. `default_queue_path()` (`crates/termlink-session/src/offline_queue.rs:118`) resolves
   to ONE file per identity dir. It does not vary with `--hub`;
   `crates/termlink-cli/src/commands/channel.rs:1072` computes the target socket and the
   queue path independently of each other.
2. `pending_posts` is `(id, post_json, enqueued_ms, attempts)`
   (`offline_queue.rs:123-128`). There is NO target-hub column.
3. `BusClient` holds a single `addr` fixed at construction
   (`crates/termlink-session/src/bus_client.rs:128`) and `flush()` drains EVERY queued
   row to `self.addr` (`bus_client.rs:234`, `295`).

### THIS IS NOT A NEW FINDING — PL-109, 2026-05-01

**Prior art, found before writing any code here.** PL-109 (task T-1429) recorded this
same defect nearly five months ago and verified it MORE strongly than the source read
above — by inspecting the live `/root/.termlink/outbound.sqlite` and confirming
`post_json` carries topic + metadata + signature and **no `hub_addr` field**. Verbatim:

> "Outbound queue serialization gap: channel post --hub <REMOTE> with TCP target queues
> to outbound.sqlite WITHOUT preserving hub_addr in the post_json envelope. Queue flush
> retries against local hub, never the intended remote. … Should either bypass queue for
> TCP (per T-1385 design intent) or persist destination. Bug not fix for this session —
> design gap for T-1429 follow-up."

It has sat at `application: TBD` since. The learning is accurate, specific, names both
remediation options, and **prevented nothing for five months** — the same argument
T-2746 made for converting a precise-but-inert learning into a structural check.

Re-surfaced 2026-09-21 during design review of AEF's T-3397 cross-post proposal, which
is about to specify `channel post --hub <addr>` as the G-060 cross-hub mechanism. That
makes this defect load-bearing for a consumer, not just latent.

**Scope note:** PL-109 says flush "retries against local hub"; the source read says it
flushes to whichever `addr` the current `BusClient` was built with. These agree — a
later `channel post` with no `--hub` builds a local client and drains the remote-destined
rows into the local hub. The local case is simply the common one.

## Acceptance Criteria

### Agent
- [x] **Reproduced against the shipping verb before any fix** (PL-367) — **NEGATIVE RESULT.**
      The defect this task was filed for does NOT reproduce on the TCP cross-hub path.
      Two variants, both against the shipping verb, 2026-09-21:
      `127.0.0.1:9199` (nothing listening, absent from `hubs.toml`) →
      `Error: cross-hub channel.post failed: I/O error: Connection refused (os error 111)`,
      true exit code **1**; and `192.168.10.141:9100` (laptop-141, present in `hubs.toml`,
      host down) → `... No route to host (os error 113)`, true exit code **1**. Queue before
      and after: `pending=0 dead_letters=0`. The scratch topic `smoke:t3041-135102` on the
      LOCAL hub received nothing — `channel delete` reported **0 record(s) removed**, so the
      canary was not misdelivered there either. Nothing was queued, nothing was misdelivered,
      and the refusal was loud.
- [x] **Mechanism confirmed in source, not inferred from two trials.**
      `crates/termlink-cli/src/commands/channel.rs:1096` branches on `if sock.is_tcp()` into a
      direct authed RPC whose error is propagated, under the comment: *"T-1385: TCP cross-hub
      posts bypass the offline queue (BusClient is Unix-only at the wire level). Direct authed
      RPC; no queueing on failure."* The offline-queue path is the `else` branch and is
      Unix-socket-only. A TCP cross-hub post therefore cannot reach the queue, so it cannot be
      misdelivered by a queue row that carries no destination.
- [x] **The task's own premise is recorded as falsified.** This task's `description` asserts a
      queued cross-post "is delivered to whichever hub the next post targets"; PL-109 asserts it
      "retries against local hub, never the intended remote". Those were already two different
      failure shapes, and the measurement shows **neither** occurs — the post never queues.
- [x] **PL-109 updated** from `application: TBD` to name this task and carry the measurement,
      so a five-month-old learning stops reading as an unactioned live defect.
- [x] **Dating the divergence.** The bypass landed **2026-04-28** in commit `175096726`
      (T-1385) — three days BEFORE PL-109 was recorded on 2026-05-01. This is deliberately NOT
      claimed as "the learning was false when written": a binary predating that commit would
      still have shown the old behaviour, which is this project's own shipped-not-live class
      (G-069). The claim is only that it is not true now.
- [x] **Residual named and explicitly NOT claimed.** `queue_path` is a single shared
      `default_queue_path()` and the queueing branch is taken for any non-TCP `--hub`, so two
      local hubs addressed by different UNIX SOCKET paths on one host would share one queue with
      no destination column. **Not reproduced, not tested, not asserted in either direction.**
      One bug = one task: if it is ever worth chasing it is a separate task, not a re-scope of
      this one.

<!-- WITHDRAWN as not applicable. These four ACs presupposed a reproducible defect:
       - hermetic regression test capturing the reproduction
       - remediation chosen between PL-109's two named options
       - cross-hub post can never be delivered to a hub other than the one addressed
       - refusal is loud and names the target hub (PL-373)
     There is nothing to remediate on the measured path, and the last one is already
     satisfied by the shipping behaviour (named cause + exit 1), not by work done here.
     They are withdrawn rather than ticked, because ticking them would claim work that
     was never performed. -->

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

# T-3041 closes on a NEGATIVE reproduction, so these pin the EVIDENCE for that
# negative, not a fix. All are pure reads; all use the L-387-safe no-pipe form.
grep -q 'TCP cross-hub posts bypass the offline queue' crates/termlink-cli/src/commands/channel.rs
grep -q 'if sock.is_tcp()' crates/termlink-cli/src/commands/channel.rs
git cat-file -e 175096726^{commit}
python3 -c "import yaml; yaml.safe_load(open('.context/project/learnings.yaml'))"
python3 -c "import yaml,sys; d=yaml.safe_load(open('.context/project/learnings.yaml')); ls=d['learnings'] if isinstance(d,dict) and 'learnings' in d else d; e=[x for x in ls if x.get('id')=='PL-109'][0]; sys.exit(0 if str(e.get('application','')).startswith('SUPERSEDED') else 1)"
#
# DELIBERATELY NOT VERIFIED HERE: the live two-variant post to an unreachable hub.
# `cmd_channel_post` calls ensure_topic BEFORE the is_tcp branch, so running it under
# P-011 would CREATE a topic on the local hub on every completion attempt. A gate with
# a side effect on shared hub state is the wrong trade. The live measurement is recorded
# as evidence in the Agent ACs above, with the exact addresses, errors and exit codes.

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

## RCA

**Symptom:** A learning recorded five months earlier (PL-109, 2026-05-01) was read as a
live defect, filed as this task, and relayed to a peer project as a **standing constraint
for their spec** — "treat cross-hub post as unreliable under a hub blip". Measured against
the shipping verb, the defect does not exist on that path: the post never queues, it
refuses loudly with a named cause and exit 1. The claim had to be retracted to the peer
twenty minutes after it was sent.

**Root cause:** The learning register cannot express supersession. PL-109 sat at
`application: TBD`, which reads identically whether nobody got round to it or whether it
stopped being true. The bypass that falsifies it landed 2026-04-28 in `175096726` (T-1385)
— three days BEFORE the learning was written — and nothing re-measured it in the five
months since. Note this is not "the learning was wrong": a binary predating that commit
would still have shown the old behaviour, which is this project's own shipped-not-live
class (G-069). It was recorded against reality and then reality moved.

**Why structurally allowed:** three gaps, none of which is the code.
1. **PL-367 is discipline, not a gate.** "A filed defect must be measured against the
   shipping verb, not accepted on the filing's say-so" is exactly the rule that would have
   caught this, and nothing asked whether it had been applied. This is the same condition
   the alloc-sink / drain-sink / silent-exit / busy-spin checks exist to fix one layer
   down — a convention held by memory rather than by a check.
2. **Nothing reads the learning register for staleness.** The guard layer runs eighteen
   cron canaries and eleven source-level static checks; not one asks "does this learning
   still reproduce?" A learning is written once and never re-measured, so its half-life is
   invisible. The register is the one memory type with no freshness signal at all.
3. **A cross-project assertion has no gate between reading and sending.** I moved from
   "the register says X" to telling another project to design around X with no measurement
   step in between. The blast radius of an unmeasured claim is largest precisely when it
   crosses a project boundary, and that is where the least checking happens.

**Prevention:**
- *Instance:* PL-109 now carries the measurement, the addresses, the exit codes and the
  commit that superseded it, so the next reader cannot repeat this. The retraction was sent
  to the peer with the evidence rather than another assertion.
- *Class:* the structural fix — learnings carrying a `re_measured_on:` / supersession field,
  and something that surfaces learnings never re-measured — is **a Sovereign question,
  surfaced here and deliberately not resolved.** Adding a field to the learning schema
  unilaterally, inside a bug task, is exactly the kind of quiet policy change that should be
  a human decision; and a checker that guesses whether a learning is stale would be worse
  than none (PL-373). Recorded as an open question in the handback, not decided here.

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

### The rule existed and I did not apply it (2026-09-21)

PL-367 says a filed defect must be measured against the shipping verb, not accepted on the
filing's say-so. I read PL-109, found it matched a source read, and relayed it to a peer
project as a **standing constraint for their spec** — "treat cross-hub post as unreliable
under a blip" — before running a single command. Twenty minutes later I had to retract it.
The rule was not missing, not unclear, and not hard to apply; the reproduction took four
commands. Having the learning is not the same as reaching for it.

### A superseded learning is indistinguishable from a live one

PL-109 sat at `application: TBD` for five months. The register has no way to express "this
was true, and then someone fixed it" — `TBD` reads identically whether nobody got to it or
whether it stopped being true three days after it was written. The fix landed 2026-04-28
and the learning was recorded 2026-05-01, so for essentially its whole life it described
behaviour the code no longer had, and anyone reading the register would have concluded, as
I did, that it was a live defect. That is the same shape as this project's stale-guard
lessons (T-2818: a guard nobody can trust; T-2680: a green whose scope is unstated) applied
to project memory rather than to a check. Whether learnings should carry a re-measured-on
date is a question for the human, not one to settle by inventing a field here.

### Cost of relaying unmeasured claims across projects

The retraction was cheap because the peer had not shipped anything yet. It would not have
been cheap if they had designed around a constraint that does not exist — a retry/buffer
layer for a path that already fails loudly. The asymmetry argues for measuring BEFORE the
cross-project send, not before the fix.

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

### Why no remediation was chosen (2026-09-21)

Nothing to remediate on the measured path. PL-109 named two options — (a) persist the
destination on the queue row, (b) refuse to queue a cross-hub post and fail loudly. The
shipping binary already does (b), and has since commit `175096726` (2026-04-28, T-1385):
the TCP branch never touches the queue and propagates the connection error with a named
cause and exit 1. Implementing (a) now would add a destination column to a queue that the
cross-hub path cannot reach.

### Why the four unmet ACs were WITHDRAWN rather than ticked

Ticking "remediation chosen" or "regression test written" would claim work nobody did.
P-010 gates on checked boxes, so the cheap path was to tick them and let the task close
clean. They are struck through in an HTML comment naming each one and why it no longer
applies, which leaves the gate honest and the record readable.

### Why the UNIX-socket residual is not pursued here

`queue_path` is one shared `default_queue_path()` and the queueing branch is taken for any
non-TCP `--hub`, so two local hubs addressed by different unix socket paths would share a
queue with no destination column. It is unreproduced and untested. One bug = one task: it
is a separate task if it is ever worth chasing, not a re-scope of this one. Recording it
as an untested residual rather than silently dropping it, and equally not inflating it
into a finding I have not measured.

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

### 2026-09-21T12:45:56Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3041-offline-queue-has-no-target-hub-column--.md
- **Context:** Initial task creation

### 2026-09-21T12:47:09Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
