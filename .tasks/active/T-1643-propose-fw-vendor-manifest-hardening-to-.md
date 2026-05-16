---
id: T-1643
name: "Propose fw vendor manifest hardening to framework-agent (T-1642 RCA Tier-B follow-up)"
description: >
  Propose fw vendor manifest hardening to framework-agent (T-1642 RCA Tier-B follow-up)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-16T06:53:33Z
last_update: 2026-05-16T08:59:13Z
date_finished: null
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
- [ ] On framework-agent ACK with their task ID (or refusal with rationale), append their task ID to `related_tasks` and close T-1643

### Human
<!-- All criteria agent-verifiable; no human action needed -->

## Verification

# Proposal was posted (offset captured in Updates section)
ls .tasks/active/T-1643-*.md .tasks/completed/T-1643-*.md 2>/dev/null | head -1 | xargs grep -qE 'agent-chat-arc offset=[0-9]+'
# framework-agent task ID linked back (added after their ACK)
ls .tasks/active/T-1643-*.md .tasks/completed/T-1643-*.md 2>/dev/null | head -1 | xargs grep -q 'framework-agent ACK:'

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
