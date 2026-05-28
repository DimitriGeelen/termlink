---
id: T-1849
name: "agent-chat-arc-recent.sh — fleet-wide 'what's been said?' verb"
description: >
  Walks hubs.toml, scans recent agent-chat-arc posts across the fleet, merges chronologically, surfaces per-post: ts, hub, sender (metadata.agent_id), msg_type, payload preview. Closes the third leg of the discovery triangle: who's there (agent-listeners-fleet, T-1837), is rail healthy (fleet-doctor + canary), what's been said (THIS). Gives a fresh-session agent context to respond.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [doorbell-mail, discovery, t-1830-arc]
components: [scripts/agent-chat-arc-recent.sh, scripts/test-agent-chat-arc-recent.sh]
related_tasks: []
created: 2026-05-28T19:30:39Z
last_update: 2026-05-28T19:34:29Z
date_finished: 2026-05-28T19:34:29Z
---

# T-1849: agent-chat-arc-recent.sh — fleet-wide 'what's been said?' verb

## Context

Third leg of the discovery triangle for the T-1830 arc:
  1. **Who's there?**         → `agent-listeners-fleet.sh` (T-1837)
  2. **Is the rail healthy?** → `fleet-doctor` + `check-fleet-doorbell-mail-health.sh` (T-1831)
  3. **What's been said?**    → **this task** (T-1849)

Without #3, an agent landing on a fresh session can see WHO is reachable but has no context to RESPOND to. The arc needs context-before-reply, not just discovery-before-send.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-chat-arc-recent.sh` (NEW) — walks `~/.termlink/hubs.toml`, scans recent `agent-chat-arc` posts on each hub, merges chronologically. Reuses seek-to-tail (PL-188) + timeout wrap (PL-189).
- [x] Args: `--limit N` (default 20, clamp 1..=200), `--since <hours>` (default 24, clamp 1..=720), `--hub <addr>` (single-hub override), `--filter-sender ID`, `--filter-msg-type T` (default `chat`), `--all-msg-types`, `--hubs-file P`, `--json`, `--help`.
- [x] Output (text): header + 5-column table `TS HUB SENDER TYPE PREVIEW`. Preview truncated to ~80 chars with ellipsis.
- [x] Output (--json): envelope `{ok, window_hours, limit, summary: {total_posts, hubs_scanned, hubs_failed, unique_speakers}, posts: [...]}`.
- [x] Sender resolution: `.metadata.agent_id` → `.metadata._from` → `.sender_id` (priority). Closes a gap that mis-counted vendored-arc heartbeat posters (they use `_from`, not `agent_id`).
- [x] Payload decode: prefer `.payload`; fall back to `.payload_b64 | @base64d`.
- [x] Test script: 8 tests, 8/8 pass.
- [x] Fabric: both cards registered.
- [x] Live: `state=4 unique_speakers` in 24h window — finds vendored-arc heartbeats from 3 distinct hosts + my own posts.

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

test -x scripts/agent-chat-arc-recent.sh
test -x scripts/test-agent-chat-arc-recent.sh
bash scripts/agent-chat-arc-recent.sh --json | jq -e '.ok and (.posts | type == "array")' >/dev/null
bash scripts/test-agent-chat-arc-recent.sh >/dev/null

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

### 2026-05-28T19:30:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1849-agent-chat-arc-recentsh--fleet-wide-what.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-5de86d96
- **Timestamp:** 2026-05-28T19:34:34Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 3

**Per-AC findings:**

- **AC#1 (Agent)** — `scripts/agent-chat-arc-recent.sh` (NEW) — walks `~/.termlink/hubs.toml`, scans recent `agent-chat-arc` posts on each hub, merges chronologically. Reuses seek-to-tail (PL-188) + timeout wrap (PL-189).
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/hubs.toml in: `scripts/agent-chat-arc-recent.sh` (NEW) — walks `~/.termlink/hubs.toml`, scans recent `agent-chat-arc` posts on each hub, merges chronologically. Reu`

**Verification-level findings:**

  1. **empty-output-success** (partial, heuristic) @ Verification:line 3
     - evidence: `bash scripts/agent-chat-arc-recent.sh --json | jq -e '.ok and (.posts | type == "array")' >/dev/null`
  2. **empty-output-success** (partial, heuristic) @ Verification:line 4
     - evidence: `bash scripts/test-agent-chat-arc-recent.sh >/dev/null`

### 2026-05-28T19:34:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
