---
id: T-1880
name: "/reply <peer> '<text>' — one-keystroke ad-hoc reply skill (SEND/RECEIVE symmetry)"
description: >
  Operator has /agent-handoff for first-contact send but no one-keystroke verb for ad-hoc reply to an existing thread. Currently must invoke bash scripts/agent-respond.sh --topic dm:... --conversation-id <??> --reply '...' which requires manual cid extraction from envelope metadata. Build (a) scripts/agent-reply.sh wrapper that resolves topic via peer-substring match + auto-extracts cid from the topic's latest envelope, delegates to agent-respond.sh for the actual ack+reply; (b) /reply slash skill that surfaces this with one-keystroke invocation. Complements /check-arc respond (which is the batch-iterate-unread pattern) — /reply is the targeted single-thread case.

status: work-completed
workflow_type: build
owner: claude-code
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-30T15:21:10Z
last_update: 2026-05-30T15:25:47Z
date_finished: 2026-05-30T15:25:47Z
---

# T-1880: /reply <peer> '<text>' — one-keystroke ad-hoc reply skill (SEND/RECEIVE symmetry)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] `scripts/agent-reply.sh` exists with: required positional `<peer-substring>` + `<text>`, optional flags `--self`, `--hub`, `--ensure-cid` (allow new cid when none on topic), `--dry-run`, `--json`
- [x] Self-fp resolution uses the PL-195 canonical chain (`channel info agent-presence`, chat-arc fallback) — same as agent-send.sh/agent-respond.sh
- [x] Topic resolution: substring-match peer against `dm:*` topics where self-fp appears in either slot. Zero matches → exit 2 with "use /agent-handoff to start a thread" hint. Multiple matches → exit 2, list candidates, refuse
- [x] cid extraction: read latest envelope on the resolved topic, pull `metadata.conversation_id`. Missing cid + no `--ensure-cid` → exit 2 with hint to use /agent-handoff (structured threads). With `--ensure-cid` → mint a new cid (`reply-<utc-iso>` form) and proceed
- [x] Delegates to `scripts/agent-respond.sh --topic <T> --conversation-id <cid> --reply <text>` for the actual ack+reply (no protocol duplication)
- [x] `.claude/commands/reply.md` slash skill exists, wraps the script with one-keystroke invocation: `/reply <peer-short> "<text>"`
- [x] CLAUDE.md Quick Reference gains exactly ONE line for `/reply`
- [x] Smoke test executed live: live invocation `bash scripts/agent-reply.sh "d1993c2c3ec44c94:d1993c2c3ec44c94" "[T-1880 smoke - please ignore]" --ensure-cid --json` posted receipt (offset 9) + reply turn (offset 10) on `dm:d1993c2c3ec44c94:d1993c2c3ec44c94` with minted cid `reply-20260530T152332Z`. Topic envelope count 9→11.
- [x] `--help` is operator-readable and documents the multi-match refusal behavior

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

test -x scripts/agent-reply.sh
bash scripts/agent-reply.sh --help 2>&1 | grep -q "peer-substring"
bash scripts/agent-reply.sh --help 2>&1 | grep -q "ensure-cid"
grep -q "channel info agent-presence" scripts/agent-reply.sh
grep -q "scripts/agent-respond.sh" scripts/agent-reply.sh
test -f .claude/commands/reply.md
grep -q "/reply" CLAUDE.md
bash -n scripts/agent-reply.sh

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

### 2026-05-30T15:21:10Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1880-reply-peer-text--one-keystroke-ad-hoc-re.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-11d19cbc
- **Timestamp:** 2026-05-30T15:25:47Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — Self-fp resolution uses the PL-195 canonical chain (`channel info agent-presence`, chat-arc fallback) — same as agent-send.sh/agent-respond.sh
  - **AC-verify-mismatch** (narrow, heuristic) — `path=agent-send.sh/agent-respond.sh in: Self-fp resolution uses the PL-195 canonical chain (`channel info agent-presence`, chat-arc fallback) — same as agent-send.sh/agent-respond.sh`

### 2026-05-30T15:25:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
