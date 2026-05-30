---
id: T-1874
name: "/check-arc skill: replace whoami self-fp path with recent-local-post sender_id read (shared-host fix)"
description: >
  PL-195 mitigation. /check-arc Step 1 reads whoami.session.identity_fingerprint which isnt the envelope sender_id. On shared hosts whoami is ambiguous. Fix: read sender_id from any recent local-hub post. See PL-195 for full context.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-30T07:53:19Z
last_update: 2026-05-30T08:13:47Z
date_finished: null
---

# T-1874: /check-arc skill: replace whoami self-fp path with recent-local-post sender_id read (shared-host fix)

## Context

PL-195 (captured 2026-05-30 from a session on shared host .107): `/check-arc` Step 1 reads `session.identity_fingerprint` from `termlink whoami --json`, but (a) that field is not the wire-level envelope `sender_id`, and (b) on shared hosts `whoami` returns `{ambiguous: true, candidates:[...]}` with no session block at all. Result: the skill cannot resolve self-fp on any shared host, so `dm:<self-fp>:*` topic enumeration in Step 2 finds nothing and the operator's inbox appears empty.

The workaround is mechanical: read the envelope `sender_id` from any recent post this session signed on the local hub. `termlink channel subscribe agent-presence --cursor 0 --limit 1 --json | jq -r .envelopes[0].sender_id` returns the host's signing key on a single round-trip.

Tactical scope: edit `.claude/commands/check-arc.md` (project-level skill) — replace Step 1's whoami path with the subscribe-based read, document shared-host semantics (all sessions on the same host share one envelope identity until T-1693 ships per-agent keys), keep the failure mode actionable (no silent degradation).

## Acceptance Criteria

### Agent
- [x] `.claude/commands/check-arc.md` Step 1 reads sender_id from `termlink channel subscribe agent-presence --cursor 0 --limit 1 --json` (or another recent local-hub post) instead of `termlink whoami --json`
- [x] Step 1 documents that on shared hosts (multiple sessions co-resident) the resolved fp is the host's signing key, shared across sessions until T-1693
- [x] Step 1 has an explicit fallback: if `agent-presence` has zero posts on the local hub, try `agent-chat-arc --cursor 0 --limit 1` then `--limit 5` (any recent post)
- [x] Step 1 fail-fast path is preserved: if no local post can be read, print an actionable error naming the alternate diagnostic command
- [x] The skill's "Related" or "Rules" section references PL-195 and T-1693 (per-agent keys, structural fix)
- [x] Smoke test: invoke the new Step 1 sequence manually on this host and verify it returns a 16-hex sender_id matching what envelopes carry

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

grep -qE "channel info agent-presence|channel subscribe agent-presence" .claude/commands/check-arc.md
! awk '/^```/{p=!p; next} p && /termlink whoami/' .claude/commands/check-arc.md | grep -q .
grep -qE "PL-195" .claude/commands/check-arc.md
grep -qE "T-1693" .claude/commands/check-arc.md
test -n "$(timeout 8 termlink channel info agent-presence --json 2>/dev/null | jq -r '.senders[0].sender_id // empty')"

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

### 2026-05-30T07:53:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1874-check-arc-skill-replace-whoami-self-fp-p.md
- **Context:** Initial task creation

### 2026-05-30T08:13:47Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-05-30T08:13:47Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
