---
id: T-1879
name: "/check-arc Step 4+5 peek hints — point at /recent-dm and /recent-chat (skill cross-link)"
description: >
  /check-arc Steps 4 and 5 currently emit verbose 'termlink channel subscribe <topic> --since-offset <last-acked> --limit <count>' as the peek hint per topic, requiring the operator to do cursor arithmetic. The canonical drill-in tools already exist: /recent-dm <peer-short> (T-1862, just got its PL-195 fix in T-1878) for DM topics, and /recent-chat (T-1851) for agent-chat-arc. Replace the verbose hints with the slash-skill calls to lower the floor on every check-arc → respond cycle.

status: work-completed
workflow_type: build
owner: claude-code
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T14:38:38Z
last_update: 2026-05-30T14:41:32Z
date_finished: 2026-05-30T14:41:32Z
---

# T-1879: /check-arc Step 4+5 peek hints — point at /recent-dm and /recent-chat (skill cross-link)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `.claude/commands/check-arc.md` Step 4 per-DM-topic peek hint replaced with `/recent-dm <peer-short> --since 720` (the canonical drill-in for a single DM thread, post-T-1862/T-1878)
- [x] `.claude/commands/check-arc.md` Step 5 agent-chat-arc peek hint replaced with `/recent-chat <unread-count>` (the canonical drill-in for fleet broadcasts, T-1851)
- [x] Ack hint (`termlink channel ack ...`) preserved verbatim — that's the only operator action for which there is no slash-skill yet
- [x] Related footer (bottom of skill body) gains references to T-1862 and T-1851 if not already present (T-1862 was already present; T-1851 added)
- [x] No behavior change in Steps 1-3 (self-fp resolution, topic discovery, unread computation all unchanged); only the rendered hint strings move from verbose-CLI to slash-skill form

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

grep -q "peek: /recent-dm" .claude/commands/check-arc.md
grep -q "peek: /recent-chat" .claude/commands/check-arc.md
grep -q "termlink channel ack" .claude/commands/check-arc.md
# Note: the verbose "subscribe --since-offset" form remains in Step 6 (respond
# mode programmatic read) by design — Step 6 is the agent's read path, not the
# operator's peek hint. Only Steps 4/5 (the operator-facing peek hints) were
# rewritten; verify they no longer carry that form.
! awk '/^## Step 4/,/^## Step 6/' .claude/commands/check-arc.md | grep -q "channel subscribe.*--since-offset"

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

### 2026-05-30T14:38:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1879-check-arc-step-45-peek-hints--point-at-r.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e1d24492
- **Timestamp:** 2026-05-30T14:41:32Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-30T14:41:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
