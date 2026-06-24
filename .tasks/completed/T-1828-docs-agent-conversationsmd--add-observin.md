---
id: T-1828
name: "docs: agent-conversations.md — add Observing Autonomous Threads section (T-1826/T-1827)"
description: >
  docs: agent-conversations.md — add Observing Autonomous Threads section (T-1826/T-1827)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-28T09:51:14Z
last_update: 2026-05-28T09:52:57Z
date_finished: 2026-05-28T09:52:57Z
---

# T-1828: docs: agent-conversations.md — add Observing Autonomous Threads section (T-1826/T-1827)

## Context

T-1826 (`agent-conversation-status.sh`) + T-1827 (`agent-conversation-list.sh`) shipped
as the read-side observability pair for the doorbell+mail loop (T-1800 arc). They live
in `scripts/` and are discoverable only by `ls scripts/agent-*` — no recipe link, no
mention in the canonical `docs/operations/agent-conversations.md` reference.

Add a short "Observing autonomous threads" section to `agent-conversations.md` (before
"Limits and next steps") that lists both verbs, links to the doorbell+mail primitives
they complement (`agent-send.sh`, `agent-respond.sh`), and shows a minimal
copy-pasteable example. Discoverability fix; no code change.

## Acceptance Criteria

### Agent
- [x] `docs/operations/agent-conversations.md` gains a new section "Observing autonomous threads (T-1826/T-1827)" before the "Limits and next steps" section.
- [x] Section mentions both `scripts/agent-conversation-status.sh` (single-cid detail) and `scripts/agent-conversation-list.sh` (all-cids on a topic).
- [x] Includes at least one copy-pasteable example for EACH verb with realistic flags.
- [x] Cross-references `scripts/agent-send.sh` and `scripts/agent-respond.sh` so a reader finding the docs via "what observes my agent threads?" can trace back to the loop.
- [x] Notes the unknown-topic exit behavior (`channel subscribe` exits 1, both verbs exit 3) so automation scripts know what to expect.
- [x] `grep -F "T-1826" docs/operations/agent-conversations.md` and `grep -F "T-1827" docs/operations/agent-conversations.md` both find at least one match.
- [x] No other sections are removed or rewritten; this is purely additive.

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

grep -qF "T-1826" docs/operations/agent-conversations.md
grep -qF "T-1827" docs/operations/agent-conversations.md
grep -qF "agent-conversation-status.sh" docs/operations/agent-conversations.md
grep -qF "agent-conversation-list.sh" docs/operations/agent-conversations.md
grep -qF "Observing autonomous threads" docs/operations/agent-conversations.md

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

### 2026-05-28T09:51:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1828-docs-agent-conversationsmd--add-observin.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-7407cfb9
- **Timestamp:** 2026-05-28T09:52:58Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#2 (Agent)** — Section mentions both `scripts/agent-conversation-status.sh` (single-cid detail) and `scripts/agent-conversation-list.sh` (all-cids on a topic).
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/agent-conversation-status.sh in: Section mentions both `scripts/agent-conversation-status.sh` (single-cid detail) and `scripts/agent-conversation-list.sh` (all-cids on a topic).`
- **AC#4 (Agent)** — Cross-references `scripts/agent-send.sh` and `scripts/agent-respond.sh` so a reader finding the docs via "what observes my agent threads?" can trace back to the loop.
  - **AC-verify-mismatch** (narrow, heuristic) — `path=scripts/agent-send.sh in: Cross-references `scripts/agent-send.sh` and `scripts/agent-respond.sh` so a reader finding the docs via "what observes my agent threads?" can trace b`

### 2026-05-28T09:52:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
