---
id: T-1445
name: "Channel-1 upstream PR — adopt --ensure-topic in framework lib/publish-learning-to-bus.sh + lib/pickup-channel-bridge.sh (G-051 long-term)"
description: >
  Channel-1 upstream PR — adopt --ensure-topic in framework lib/publish-learning-to-bus.sh + lib/pickup-channel-bridge.sh (G-051 long-term)

status: captured
workflow_type: build
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
created: 2026-05-02T05:03:31Z
last_update: 2026-05-02T05:04:15Z
date_finished: null
---

# T-1445: Channel-1 upstream PR — adopt --ensure-topic in framework lib/publish-learning-to-bus.sh + lib/pickup-channel-bridge.sh (G-051 long-term)

## Context

Follow-up to G-051 (gap, watching). Both `lib/publish-learning-to-bus.sh` and
`lib/pickup-channel-bridge.sh` in vendored framework code post to known-canon
topics (`channel:learnings`, `framework:pickup`) without `--ensure-topic`. When
the topics don't exist (post-restart loss, fresh hub) the post fails with -32013
and the script silently falls back to T-1166-deprecated `event.broadcast`.

Workflow: Channel-1 upstream-mirror pattern — `fw termlink dispatch --workdir`
to /opt/999-AEF, patch the two scripts, commit + push to onedev (NOT github).
Remote in 999-AEF is `onedev`, not `origin`. Then verify the patch lands in
this consumer project's vendored copy via the auto-mirror.

Adoption must be backward-compatible: scripts run on peer projects whose
vendored CLI predates T-1443 (no `--ensure-topic` flag) and would error on
unknown flag. Probe for flag support first OR keep a no-flag fallback path.

## Acceptance Criteria

### Agent
- [ ] Probe-first pattern in `lib/publish-learning-to-bus.sh`: check `termlink channel post --help | grep -q ensure-topic` once and gate the flag. Old binaries get the no-flag path; new binaries get the heal-on-restart path.
- [ ] Same probe pattern applied in `lib/pickup-channel-bridge.sh` for `framework:pickup` post.
- [ ] Upstream commit on `/opt/999-AEF` master with T-1445 message; pushed to onedev (not github).
- [ ] Vendored copy in `/opt/termlink/.agentic-framework/lib/publish-learning-to-bus.sh` reflects the patch after auto-mirror — verified via grep.
- [ ] Verification on local hub: drop `channel:learnings` topic, then `fw context add-learning "..."` — Tier-A path succeeds (log shows `posted via=channel.post`, no `channel.post-failed`).
- [ ] Run on a host with old CLI (e.g., .141 still on 0.9.1640) — the script's no-flag fallback exercises and doesn't error.

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

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.

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

### 2026-05-02T05:03:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1445-channel-1-upstream-pr--adopt---ensure-to.md
- **Context:** Initial task creation

### 2026-05-02T05:04:15Z — status-update [task-update-agent]
- **Change:** status: started-work → captured
- **Change:** horizon: now → next
