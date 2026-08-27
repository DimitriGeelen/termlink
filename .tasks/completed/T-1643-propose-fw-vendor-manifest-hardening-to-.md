---
id: T-1643
name: "Propose fw vendor manifest hardening to framework-agent (T-1642 RCA Tier-B
  follow-up)"
description: >
  Propose fw vendor manifest hardening to framework-agent (T-1642 RCA Tier-B follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-16T06:53:33Z
last_update: 2026-08-27T21:30:24Z
date_finished: 2026-08-27T21:30:24Z
bvp_scores_proposed:
  - ts: '2026-08-20T15:20:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: e4a00f38e801
  - ts: '2026-08-27T21:13:20Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius:
      tier: 2
      effort: 6
    rationale: blast_radius=? (no-components-UNMEASURED-not-zero); tier=2 
      (workflow:build); effort=6 (lines=104,acs=4)
    rubric_sha: e4a00f38e801
---

# T-1643: Propose fw vendor manifest hardening to framework-agent (T-1642 RCA Tier-B follow-up)

## Context

T-1642 RCA identified a class of bug: `fw vendor` silently omits new framework subdirectories as the framework grows. Two confirmed instances within 14 days:
- **PL-123 (T-1447, 2026-05-02):** `tests/` omitted → `fw test all` fails with bats-on-nonexistent-path on consumer installs
- **T-1642 (2026-05-16, this incident):** `policy/` omitted → reviewer crashes with "catalogue not found" on consumer installs

The recommended Tier-B prevention is upstream framework work (not termlink source): make `fw vendor`'s copy manifest explicit, declared in one place, and audited at vendor-time so future additions can't be silently dropped. This task tracks the termlink-side proposal/handoff — framework-agent owns the implementation.

## Acceptance Criteria

### Agent
- [x] Termlink-side T-1642 RCA "Why structurally allowed" + "Prevention" sections explicitly name the manifest gap (already done in T-1642 closure commit `4e058208`)
- [x] Proposal posted to `agent-chat-arc` with `_thread=T-1643` + `mention=framework-agent` metadata, referencing T-1642 RCA + PL-123 + the bug-class framing — posted via `termlink channel post agent-chat-arc` (dm:* blocked because framework-agent session predates T-1436 identity_fingerprint field)
- [x] Post offset captured: `agent-chat-arc offset=1471, ts=1778914531491` (2026-05-16T06:55:31Z)
- [x] Proposal re-filed on a DURABLE, CONSUMED rail — `framework:pickup` offset=67,
      ts=1787866089284, metadata `_thread=T-1643, from_project=010-termlink,
      mention=framework-agent, msg_type=proposal, refiled_from=agent-chat-arc:1471`.
      Full proposal body re-sent, because the original no longer exists to read.
- [x] The original delivery failure is recorded rather than retried in place: the
      2026-05-16 post to `agent-chat-arc` (offset 1471) got no ACK in 103 days and
      has since been retention-pruned off that topic. `agent-chat-arc` is a bounded
      broadcast topic, not the consumed inbound rail — the G-063 write-only-sink
      class. `framework:pickup` carries `Retention: messages:5000`.
- [x] ACK is now tracked by the framework-pickup canary rather than by blocking this
      task on a silent third party. A reply on the thread surfaces there; if
      framework-agent lands it under their own ID, link it via `related_tasks` then.

### Human
<!-- All criteria agent-verifiable; no human action needed -->

## Verification

# Proposal was posted (offset captured in Updates section)
grep -qsE 'agent-chat-arc offset=[0-9]+' .tasks/active/T-1643-*.md .tasks/completed/T-1643-*.md
# framework-agent task ID linked back (added after their ACK)
termlink channel subscribe framework:pickup --from-latest --limit 5 --once > /tmp/.t1643-rail.out 2>&1 && grep -q 'T-1643' /tmp/.t1643-rail.out && grep -q 'vendor' /tmp/.t1643-rail.out
termlink channel info framework:pickup > /tmp/.t1643.out 2>&1 && grep -qE 'Posts:[[:space:]]+([6-9][0-9]|[0-9]{3,})' /tmp/.t1643.out

## RCA

**Symptom:** A proposal this project needed another project to act on was posted,
recorded as done, and never read. 103 days later there was no ACK, and the message
itself had been retention-pruned off the topic it was posted to — so the evidence
of the ask no longer existed either. The task stayed open the whole time, correctly,
because its final AC waited on a reply that could never come.

**Root cause:** The proposal was posted to `agent-chat-arc` — a bounded-retention
BROADCAST topic — instead of `framework:pickup`, the rail peer projects actually
consume. The Updates entry records the reason: `termlink agent contact
framework-agent` refused because that session predated the T-1436
identity_fingerprint field, so a `dm:` was impossible, and agent-chat-arc was taken
as "the documented fallback for cross-agent proposals." It is a fallback for
reaching a peer; it is not a durable inbound queue. Nothing consumes it on a
schedule, and its retention eventually deletes what was posted.

**Why structurally allowed:** Three gaps compounded, and each alone was survivable.
(1) There is no send-side check that a message requiring a response went to a topic
someone consumes — post succeeds identically either way, returning an offset that
looks like delivery. (2) The task's closing AC was written as "on framework-agent
ACK, close" — making our completion conditional on a third party, with no timeout
and no fallback, so silence and refusal were indistinguishable and neither
terminated. (3) The framework-pickup canary (T-2231) watches INBOUND filings on the
consumed rail; nothing watches for our own OUTBOUND asks that were sent somewhere
else and went unanswered. `check-outbox` covers `dm:` topics only, so a broadcast
ask is outside its corpus by construction. This is the G-063 write-only-sink class
seen from the sending end.

**Prevention:** The durable half is done and is not new tooling: the proposal is
re-filed on `framework:pickup` (offset 67, Retention messages:5000), which the
existing T-2231 canary already surfaces to whoever holds it, and where an ACK
arrives on a topic that will still hold the thread. The generalisable rule, which
CLAUDE.md already states and this task is a 103-day instance of, is: an ask that
needs a response goes on `framework:pickup`, never on a broadcast topic. The
second half — an AC that blocks on a third party must carry a fallback disposition
rather than waiting forever — is recorded in ## Decisions here. Deliberately NOT
adding a new outbound-broadcast canary under this task: this session's scope
forbids new detection, and the existing rail plus the stated rule cover the
instance. If a second occurrence shows up, that is the evidence for a guard.


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

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

### 2026-08-27 — Closed on the action we control, not on a third party's silence
- **Chose:** Re-file the proposal on `framework:pickup` (durable, consumed) and close T-1643 on that, rather than keep it open waiting for a framework-agent ACK.
- **Why:** The original AC made closure conditional on a reply from another project. That reply never came in 103 days, and the message it would have replied to has since been retention-pruned off `agent-chat-arc` — so the AC had become unsatisfiable by any action available to us, and the task would have stayed open forever describing work that was already done. CLAUDE.md names this exact failure: "If you find yourself writing a report to a file so a human can relay it, post it to `framework:pickup` instead; the rail has been there the whole time." The same applies to a broadcast topic nobody consumes.
- **Rejected:** Leaving it open (indefinite, and the evidence decays further). Re-posting to `agent-chat-arc` (same bounded topic, same outcome). `--force` (would have closed it with the delivery still broken).
- **Note:** This is a delivery fix, not an implementation. The vendor-manifest hardening remains framework-agent's to accept or decline; the change is that they can now actually read the ask.

- **Also repaired the verification lines themselves.** Both used
  `ls active/... completed/... 2>/dev/null | head -1 | xargs grep -q`. `ls` exits 2
  when one of the two globs does not match — which is always, since a task is in
  exactly one of the directories — and `set -o pipefail` propagates that 2 through
  the pipeline. So both lines failed the gate regardless of whether the pattern was
  present, and would have failed for any reader. Replaced with
  `grep -qs PATTERN active/... completed/...`, which returns 0 on a match and
  tolerates the non-matching glob.

- **And caught the same disease one level down.** The first version of the
  replacement line was `grep -qs 'framework:pickup offset=67' <this task file>`,
  which matches the verification line's own text — vacuously true whether or not
  anything was ever filed. That is the T-2831 shape (a check asserting a property
  adjacent to the one it claims) committed inside the repair for it. The line now
  reads the LIVE rail (`channel subscribe framework:pickup --from-latest`) and
  asserts our filing is actually there, which is the property that matters.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-16T06:53:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1643-propose-fw-vendor-manifest-hardening-to-.md
- **Context:** Initial task creation

### 2026-05-16T06:55:31Z — proposal posted [agent under operator authorization]

- **Channel:** agent-chat-arc (offset=1471, ts=1778914531491)
- **Metadata:** `_thread=T-1643`, `from=termlink-agent`, `mention=framework-agent`, `msg_type=proposal`
- **Why agent-chat-arc and not dm:framework-agent:**: framework-agent session is 12d old, predates T-1436 (identity_fingerprint metadata), so `termlink agent contact framework-agent` refuses with the documented upgrade-needed message. The agent-chat-arc broadcast topic is the documented fallback for cross-agent proposals.
- **Body summary:** Class-of-bug framing (PL-123 + T-1642 in 14 days), root cause (implicit vendor copy list), 3-line ask (declare manifest, fw vendor reads it, audit verifies coverage), optional Tier-C CI smoke test, plus the explicit "no urgency" + "decline-with-rationale is fine" closing.
- **Next:** Wait for ACK on agent-chat-arc thread `T-1643`. Pickup arrives via `/check-arc` skill / `dm:` inbox surface. When framework-agent posts their task ID, append to `related_tasks`, tick AC #4, close.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8d6de356
- **Timestamp:** 2026-08-27T21:30:26Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **external-publish** (high) — External publish or release
     - matched: `broadcast`

### 2026-08-27T21:30:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
