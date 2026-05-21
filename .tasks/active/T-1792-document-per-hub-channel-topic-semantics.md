---
id: T-1792
name: "Document per-hub channel-topic semantics — close G-060 (T-1791 follow-up #2)"
description: >
  Add explicit documentation that channel topics are per-hub; cross-hub message visibility requires explicit --hub <addr> posting or remote_call channel.post. Topics with the same name on different hubs are independent state. Close the operator-facing portion of G-060.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [docs, G-060, T-1791]
components: []
related_tasks: [T-1791, T-1166]
created: 2026-05-21T19:14:24Z
last_update: 2026-05-21T22:29:11Z
date_finished: null
---

# T-1792: Document per-hub channel-topic semantics — close G-060 (T-1791 follow-up #2)

## Context

T-1791 inception established that TermLink has NO inter-hub channel-topic federation primitive: channel topics on different hubs are independent state. This was the structural cause of G-060 (the 1800 vs 486 chat-arc disparity that triggered the inception). Close the operator-facing portion of G-060 by writing explicit operator documentation so future agents/operators don't assume auto-federation that doesn't exist.

## Acceptance Criteria

### Agent
- [x] `docs/operations/channel-topic-semantics.md` exists
- [x] Doc covers: per-hub independence, client-driven cross-hub posting, diagnostic recipe for "topic out of sync" investigations, implications for T-1166 retirement
- [x] Doc references PL-176, T-1791, G-060 so the trail is traceable
- [x] CLAUDE.md "Project-Specific Rules" links to the new doc (one-line pointer, not full inlined content)

## Verification

# Shell commands that MUST pass before work-completed.
test -f docs/operations/channel-topic-semantics.md
grep -q "PL-176" docs/operations/channel-topic-semantics.md
grep -q "T-1791" docs/operations/channel-topic-semantics.md
grep -q "G-060" docs/operations/channel-topic-semantics.md
grep -q "channel-topic-semantics" CLAUDE.md

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

### 2026-05-21T19:14:24Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1792-document-per-hub-channel-topic-semantics.md
- **Context:** Initial task creation

### 2026-05-21T22:29:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
