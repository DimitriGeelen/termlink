---
id: T-1445
name: "Channel-1 upstream PR — adopt --ensure-topic in framework lib/publish-learning-to-bus.sh + lib/pickup-channel-bridge.sh (G-051 long-term)"
description: >
  Channel-1 upstream PR — adopt --ensure-topic in framework lib/publish-learning-to-bus.sh + lib/pickup-channel-bridge.sh (G-051 long-term)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-02T05:03:31Z
last_update: 2026-05-02T05:32:07Z
date_finished: 2026-05-02T05:32:07Z
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
- [x] Probe-first pattern in `lib/publish-learning-to-bus.sh`: probe via `termlink channel post --help | grep -q -- '--ensure-topic'`, gate the flag conditionally. Old binaries get the no-flag path; new binaries get the heal-on-restart path.
  **Evidence:** Upstream commit `7090c6082` + vendored copy lines 88-95.
- [x] Same probe pattern applied in `lib/pickup-channel-bridge.sh` for `framework:pickup` post.
  **Evidence:** Upstream commit `7090c6082` + vendored copy lines 73-81.
- [x] Upstream commit on `/opt/999-Agentic-Engineering-Framework` master with T-1445 message; pushed to onedev — NOT github.
  **Evidence:** `7090c6082 T-1445: probe for --ensure-topic and pass conditionally (G-051 long-term)`; push `ae516b761..7090c6082  master -> master` to `https://onedev.docker.ring20.geelenandcompany.com/agentic-engineering-framework.git`. NOTE: the upstream repo's onedev remote is named `origin` (github is `github`) — opposite of /opt/termlink.
- [x] Vendored copy in `/opt/termlink/.agentic-framework/lib/publish-learning-to-bus.sh` reflects the patch (vendored is a manual copy per PL-022, not auto-mirror — applied directly via Edit).
  **Evidence:** `grep -n T-1445 .agentic-framework/lib/{publish-learning-to-bus,pickup-channel-bridge}.sh` returns 4 matches.
- [x] Verification on local hub: `fw context add-learning` exercised Tier-A path successfully.
  **Evidence:** `.publish-learning-bus.log` last line: `2026-05-02T05:31:07Z posted via=channel.post topic=channel:learnings msg_type=learning-P-009 id=PL-113 origin=termlink`. No `channel.post-failed`.
- [x] No-flag fallback verified mechanically via simulated old binary.
  **Evidence:** simulated `/tmp/fake-termlink-old` (no `--ensure-topic` in help): probe returns empty `ENSURE_TOPIC_FLAG=[]`, invocation identical to pre-T-1445. Real 0.9.1701 returns `ENSURE_TOPIC_FLAG=[--ensure-topic]`. Both paths exercise correctly.

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

### 2026-05-02T05:24:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-05-02T05:32:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
