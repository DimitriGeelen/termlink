---
id: T-1877
name: "Doc: canonical self-fp resolution recipe in operator runbook + e2e doc (PL-195 closure)"
description: >
  PL-195 fixed at 4 code sites (T-1874/1875/1876) but no doc tells operators or future arc-tool authors WHY whoami doesn't work and what the canonical path is. Add: (1) Failure-modes entry in doorbell-mail-operator-runbook.md describing the symptom and the channel-info path; (2) Short identity-resolution recipe in agent-conversations.md or arc-e2e.md for skill/script authors writing new arc tooling. Prevents the next PL-195 recurrence.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T12:18:23Z
last_update: 2026-05-30T12:20:13Z
date_finished: 2026-05-30T12:20:13Z
---

# T-1877: Doc: canonical self-fp resolution recipe in operator runbook + e2e doc (PL-195 closure)

## Context

PL-195 fixed at four sites today (T-1874 /check-arc skill, T-1875 /agent-handoff skill, T-1876 agent-send.sh + agent-respond.sh). The fix is consistent across all four — `termlink channel info agent-presence --json | jq -r .senders[0].sender_id`, with agent-chat-arc fallback — but no doc tells operators (failure path) or future arc-tool authors (greenfield path) what the canonical resolution is.

Two landing zones:
1. `docs/operations/doorbell-mail-operator-runbook.md` "Failure modes" — operator-facing entry for "I see 'cannot resolve own identity_fingerprint'"
2. Either `docs/operations/agent-conversation-arc-e2e.md` or `docs/operations/agent-conversations.md` — author-facing recipe with the exact bash incantation and the `agent-presence → agent-chat-arc → die` decision tree.

Both bounded text additions. No code changes; doc-only.

## Acceptance Criteria

### Agent
- [x] `docs/operations/doorbell-mail-operator-runbook.md` Failure modes section gains an entry titled something like "Self-fp resolution fails" or "'could not resolve own identity_fingerprint' error" with: symptom string the operator sees, root cause one-liner (whoami doesn't expose envelope sender_id, PL-195), and the working incantation
- [x] An author-facing recipe is added to either `agent-conversation-arc-e2e.md` or `agent-conversations.md` — a short titled subsection labeled "Resolving self-fp in new arc tooling" or similar, showing the channel info path with fallback, and pointing at T-1874/T-1875/T-1876 as the four call-sites that use the pattern
- [x] Both additions reference PL-195 and T-1693 (structural fix that will eventually remove the shared-host caveat)
- [x] No regression: existing sender_id/whoami references in these docs are NOT contradicted by the new content
- [x] Smoke test: grep both docs for the new sections and confirm the channel-info command appears verbatim (so a future agent searching by command string can find it)

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

grep -q "channel info agent-presence" docs/operations/doorbell-mail-operator-runbook.md
grep -qE "channel info agent-presence|self-fp.*resolution|resolving self.fp" docs/operations/agent-conversation-arc-e2e.md
grep -q "PL-195" docs/operations/doorbell-mail-operator-runbook.md
grep -qE "T-1693" docs/operations/doorbell-mail-operator-runbook.md

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

### 2026-05-30T12:18:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1877-doc-canonical-self-fp-resolution-recipe-.md
- **Context:** Initial task creation

### 2026-05-30T12:18:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.4)

- **Scan ID:** R-b7012925
- **Timestamp:** 2026-05-30T12:20:13Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-30T12:20:13Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
