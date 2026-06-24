---
id: T-1794
name: "PL-177 skip-rationale docstrings on channel_inbox / channel_dm / channel_dm_list"
description: >
  PL-177 skip-rationale docstrings on channel_inbox / channel_dm / channel_dm_list

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/channel.rs]
related_tasks: []
created: 2026-05-21T22:54:03Z
last_update: 2026-05-21T22:56:17Z
date_finished: 2026-05-21T22:56:17Z
---

# T-1794: PL-177 skip-rationale docstrings on channel_inbox / channel_dm / channel_dm_list

## Context

PL-177 (recorded 2026-05-21 via T-1166) names three `channel_*` CLI verbs that were intentionally excluded from the MCP-parity arc closure (T-1789 + T-1790) because the closest `agent_*` MCP wedge supersedes them: `channel_inbox` → `agent_inbox`, `channel_dm` → `agent_contact`, `channel_dm_list` → `agent_dms`. The skip is real-but-undiscoverable — a future agent looking at the CLI handlers would have no signal that omission was deliberate. Add the rationale as a docstring on each function so the next "should I wedge this?" investigation lands on the answer in two seconds (grep / hover) instead of reconstructing PL-177 from scratch.

## Acceptance Criteria

### Agent
- [x] `cmd_channel_dm` doc comment names PL-177 + the `agent_contact` supersedence + the selection criterion
- [x] `cmd_channel_dm_list` doc comment names PL-177 + the `agent_dms` supersedence
- [x] `cmd_channel_inbox` doc comment names PL-177 + the `agent_inbox` supersedence
- [x] `cargo check -p termlink` passes (no compile regression from docstring edits)

## Verification

cargo check -p termlink 2>&1 | tail -3
grep -q "MCP-PARITY SKIP (PL-177" crates/termlink-cli/src/commands/channel.rs
test "$(grep -c 'MCP-PARITY SKIP (PL-177' crates/termlink-cli/src/commands/channel.rs)" = "3"

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

### 2026-05-21T22:54:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1794-pl-177-skip-rationale-docstrings-on-chan.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-041a45da
- **Timestamp:** 2026-05-21T22:56:28Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-21T22:56:17Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
