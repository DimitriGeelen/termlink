---
id: T-1369
name: "docs/operations/agent-conversations.md — T-1365/1366/1367/1368 wave (threads, edits-of, forwards-of, topic-stats)"
description: >
  Documentation wave for the latest 4 agent-conversation deliverables —
  channel threads (T-1365), edits-of (T-1366), forwards-of (T-1367), and
  topic-stats (T-1368). Add a section per command with synopsis, purpose,
  example output, and JSON shape. Update the e2e step counter and the
  related-tasks list.

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [docs, agent-conversation, channel-cli]
components: []
related_tasks: [T-1365, T-1366, T-1367, T-1368, T-1364]
created: 2026-04-28T10:33:00Z
last_update: 2026-04-28T10:35:59Z
date_finished: 2026-04-28T10:35:59Z
---

# T-1369: docs wave for T-1365..T-1368

## Context

T-1364 was the last docs wave (covered T-1361/1362/1363). Four new commands
have shipped since: threads, edits-of, forwards-of, topic-stats. This wave
brings the operator-facing doc up to current state.

## Acceptance Criteria

### Agent
- [x] Section added for each: `channel threads`, `channel edits-of`, `channel forwards-of`, `channel topic-stats`
- [x] Each section has: synopsis line, one-paragraph purpose, real example output, JSON shape comment
- [x] e2e step count updated from 37 to 41 (the prior count was stale across two waves)
- [x] Related-tasks list extended with T-1365..T-1368
- [x] `bash tests/e2e/agent-conversation.sh` still green (no regression from doc edits — guard against accidental code touches)

## Verification

bash /opt/termlink/tests/e2e/agent-conversation.sh 2>&1 | grep -q "END-TO-END WALKTHROUGH PASSED"

## Decisions

## Updates

### 2026-04-28T10:33:00Z — task scoped
- Real ACs filled (G-020 build-readiness gate).

### 2026-04-28T10:35:59Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
