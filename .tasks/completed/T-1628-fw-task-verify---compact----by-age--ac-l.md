---
id: T-1628
name: "fw task verify: --compact + --by-age + AC label visibility (G-008 triage UX)"
description: >
  fw task verify: --compact + --by-age + AC label visibility (G-008 triage UX)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-06T19:04:11Z
last_update: 2026-05-06T19:13:43Z
date_finished: 2026-05-06T19:13:43Z
---

# T-1628: fw task verify: --compact + --by-age + AC label visibility (G-008 triage UX)

## Context

`fw task verify` (no-arg, T-193 mode) currently emits 740 lines for the
146 tasks awaiting Human AC review — unscannable. G-008 (medium severity)
captures the structural gap: human-owned tasks accumulate because no
triage workflow surfaces them.

Three additive UX flags address this without breaking the verbose default:

1. `--compact` — one line per task: `T-XXXX  Nd  H:U/V  TYPE  short-name`
   where Nd = days since last_update, U/V = unchecked/total Human ACs,
   TYPE = `[RUBBER-STAMP]` / `[REVIEW]` / `[MIXED]` based on prefix
   markers in the unchecked items.
2. `--by-age` — sort by age desc (oldest first); applies in both verbose
   and compact modes. Default remains alphabetical for backward compat.
3. `--rubber-stamp-only` / `--review-only` — filter to a single AC type.
   RUBBER-STAMP items are mechanical and operator-clearable in seconds;
   surfacing them separately accelerates the queue drain.

Out of scope: changing the verbose output shape, adding a Watchtower
review page, auto-actioning anything (R-033 sovereignty — agent cannot
tick Human ACs).

## Acceptance Criteria

### Agent
- [x] `fw task verify --compact` emits one line per task in the format `T-XXXX <Nd> <U/V> [TYPE] <name>` (no per-AC body text)
- [x] `fw task verify --by-age` sorts by `last_update` desc (oldest first); default remains alphabetical
- [x] `fw task verify --rubber-stamp-only` includes only tasks where ≥1 unchecked Human AC begins with `[RUBBER-STAMP]`
- [x] `fw task verify --review-only` includes only tasks where ≥1 unchecked Human AC begins with `[REVIEW]` (filters are additive — a mixed task appears under both)
- [x] Flags compose: `fw task verify --compact --by-age --rubber-stamp-only` works
- [x] Default `fw task verify` (no args) output is unchanged from pre-T-1628 (verbose, alphabetical) — backward compatible
- [x] `fw task --help` lists the new flags
- [x] Mirrored to upstream `/opt/999-Agentic-Engineering-Framework/bin/fw` per channel-1 protocol

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

bash tests/test_t1628_task_verify_flags.sh
out=$(.agentic-framework/bin/fw task verify --compact 2>&1 | sed 's/\x1b\[[0-9;]*m//g'); echo "$out" | grep -qE '^[[:space:]]+T-[0-9]+ +[0-9]+d +[0-9]+/[0-9]+ +\[(RUBBER-STAMP|REVIEW|MIXED|UNTAGGED)\]'
.agentic-framework/bin/fw task --help 2>&1 | grep -q -- '--compact'
# Upstream mirror verified manually via termlink dispatch (commit 68f516908 on /opt/999-AEF master).

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

## Updates

### 2026-05-06T19:04:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1628-fw-task-verify---compact----by-age--ac-l.md
- **Context:** Initial task creation

### 2026-05-06T19:13:43Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
