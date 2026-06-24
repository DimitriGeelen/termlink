---
id: T-1873
name: "/pulse: surface fallback_hubs hint when chat-arc-recent uses no-seek fallback (T-1872 render follow-on)"
description: >
  /pulse: surface fallback_hubs hint when chat-arc-recent uses no-seek fallback (T-1872 render follow-on)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-30T06:50:58Z
last_update: 2026-05-30T06:52:21Z
date_finished: 2026-05-30T06:52:21Z
---

# T-1873: /pulse: surface fallback_hubs hint when chat-arc-recent uses no-seek fallback (T-1872 render follow-on)

## Context

T-1872 added `summary.fallback_hubs: [<name>]` to `agent-chat-arc-recent.sh`
+ a human-format `fallback: ... (seek-to-tail unavailable — data may be
partial)` line. T-1871 already taught `/pulse` to surface failed_hubs;
this is the symmetric instruction for fallback_hubs.

Why surface separately. A failed hub is "did not return data". A
fallback hub is "returned data via the degraded path". An operator
sees them differently — failed = poke the hub, fallback = the data
you're seeing from this hub may miss recent activity past
SCAN_LIMIT from offset 0. Conflating them hides the partial-data
signal.

Scope: instruction-only update to `.claude/commands/pulse.md`.
No script change.

## Acceptance Criteria

### Agent
- [x] `.claude/commands/pulse.md` Step 4 default human-format render documents the `fallback:` line under the RECENT section
- [x] Concrete example block in the skill shows both `failed:` AND `fallback:` lines so Claude knows they can co-exist
- [x] Read instruction names `.recent.summary.fallback_hubs[]` as the data source (single string per entry, not an object) — different shape from `failed_hubs` so docstring needs to call this out
- [x] When fallback_hubs is empty, no line is rendered (consistent with failed_hubs handling)
- [x] Skill's Related section grows a T-1872 reference

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

grep -q "fallback_hubs" .claude/commands/pulse.md
grep -q "T-1872" .claude/commands/pulse.md
test -f .claude/commands/pulse.md

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

### 2026-05-30T06:50:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1873-pulse-surface-fallbackhubs-hint-when-cha.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-0ba277fc
- **Timestamp:** 2026-05-30T06:52:22Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-30T06:52:21Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
