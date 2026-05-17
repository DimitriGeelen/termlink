---
id: T-1668
name: "Document T-1666/T-1667 in CLAUDE.md rotation-protocol detection table"
description: >
  Document T-1666/T-1667 in CLAUDE.md rotation-protocol detection table

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T19:58:27Z
last_update: 2026-05-17T20:00:52Z
date_finished: 2026-05-17T20:00:52Z
---

# T-1668: Document T-1666/T-1667 in CLAUDE.md rotation-protocol detection table

## Context

T-1662 added the "Detection — primitive verbs" table for the original six
(T-1656–T-1661) but T-1663 (MCP parity), T-1666 (`fleet doctor
--include-pin-check`), and T-1667 (`fleet doctor --watch`) are not in the
operator's canonical reference. Closes the discoverability gap so
operators find the unified single-shot verb and continuous-monitor mode
from CLAUDE.md without needing to read git log or task files.

## Acceptance Criteria

### Agent
- [x] CLAUDE.md "Detection — primitive verbs" table extended with 4 rows: `termlink_hub_probe` (MCP), `termlink_tofu_verify` (MCP), `fleet doctor --include-pin-check`, `fleet doctor --watch <secs>`
- [x] Section heading updated to reference T-1663/1666/1667 alongside T-1656-61
- [x] No other CLAUDE.md sections modified (focused doc change)
- [x] `grep -c "fleet doctor --include-pin-check\|fleet doctor --watch" CLAUDE.md` reports >= 2 occurrences

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

grep -q "include-pin-check" CLAUDE.md
grep -q "fleet doctor --watch" CLAUDE.md
grep -q "T-1663/1666" CLAUDE.md
grep -q "termlink_hub_probe" CLAUDE.md
grep -q "termlink_tofu_verify" CLAUDE.md

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

### 2026-05-17T19:58:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1668-document-t-1666t-1667-in-claudemd-rotati.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1a1369f1
- **Timestamp:** 2026-05-17T20:00:52Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **cross-project-blast** (medium) — Cross-project or cross-repo change
     - matched: `fleet doctor`

### 2026-05-17T20:00:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
