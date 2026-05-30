---
id: T-1871
name: "/pulse: surface failed hubs from both halves in the rendered digest (T-1870 follow-on)"
description: >
  /pulse: surface failed hubs from both halves in the rendered digest (T-1870 follow-on)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-29T23:22:34Z
last_update: 2026-05-29T23:22:34Z
date_finished: null
---

# T-1871: /pulse: surface failed hubs from both halves in the rendered digest (T-1870 follow-on)

## Context

T-1870 added `summary.failed_hubs: [{hub, reason}]` to `agent-chat-arc-recent.sh`.
`agent-listeners-fleet.sh` already emits `hubs_failed: [{name, address, error}]`
(richer — names + addresses + first 200 chars of stderr). Both wrappers now
expose actionable failure data in their JSON envelopes.

Real gap: the `/pulse` skill markdown
(`.claude/commands/pulse.md`) instructs Claude to render counts only —
"PEERS (LIVE / total)" and "RECENT (last N in HOURSh window, ...)". The
underlying failure arrays are passed through verbatim in `--json` mode
but never surface in the default human-format digest. So an operator
running `/pulse` sees the conversation arc but not "ring20-dashboard
unreachable" — exactly the page-respond opacity the directive is
asking us to close.

Scope: doc-only change to the /pulse skill markdown. Add one extra line
each to the PEERS and RECENT sections when their respective failure
arrays are non-empty. No behavior change in the underlying scripts;
no JSON envelope change.

Originally filed as "enrich listener-fleet hubs_failed" — discovery
during T-1871 build showed listener-fleet was already correctly
shaped. Task re-scoped to the actually-blocking layer (rendering).

## Acceptance Criteria

### Agent
- [x] `.claude/commands/pulse.md` Step 4 (default human-format render) instructs Claude to print a one-line "failed: <name1> (<reason1>), <name2> (<reason2>)" footer under each section when the respective failure array is non-empty
- [x] Render instruction reads from `.peers.hubs_failed[].name` + `.address` (listener-fleet shape) for PEERS and `.recent.summary.failed_hubs[].hub` + `.reason` (chat-arc shape) for RECENT — both shapes documented inline so Claude doesn't conflate them
- [x] When both failure arrays are empty, no extra lines are rendered (silent on the good path; consistent with current behavior)
- [x] `--json` mode is unchanged (passthrough as today; the skill's Step 4 `--json` branch carries no rendering)
- [x] At least one example block in the skill shows the failed-hubs render so Claude has a concrete template to follow

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

grep -q "failed_hubs" .claude/commands/pulse.md
grep -q "hubs_failed" .claude/commands/pulse.md
grep -q ".name.*.address\|.hub.*.reason" .claude/commands/pulse.md
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

### 2026-05-29T23:22:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1871-agent-listeners-fleet-enrich-hubsfailed-.md
- **Context:** Initial task creation
