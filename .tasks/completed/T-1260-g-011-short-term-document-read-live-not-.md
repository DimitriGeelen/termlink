---
id: T-1260
name: "G-011 short-term: document read-live-not-cache pattern for own-hub secret in CLAUDE.md"
description: >
  G-011 short-term: document read-live-not-cache pattern for own-hub secret in CLAUDE.md

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-25T18:09:23Z
last_update: 2026-04-25T18:11:02Z
date_finished: 2026-04-25T18:11:02Z
---

# T-1260: G-011 short-term: document read-live-not-cache pattern for own-hub secret in CLAUDE.md

## Context

G-011 short-term mitigation. Mirror-image of T-1051 (receiving-end drift):
when an agent shares its OWN hub's secret with a peer, it should read the
authoritative `<runtime_dir>/hub.secret` directly, NOT the IP-keyed cache
at `~/.termlink/secrets/<ip>.hex` which is never invalidated on hub
restart. Failure mode is silent — peer auth-mismatch with no diagnostic
pointing back at the giver. Document in CLAUDE.md §Hub Auth Rotation
Protocol as rule R3.

## Acceptance Criteria

### Agent
- [x] R3 paragraph added to CLAUDE.md §Hub Auth Rotation Protocol explaining the read-live-not-cache rule for own-hub secret access.
- [x] R3 references the source incident (2026-04-20 peer-share, stale cache).
- [x] R3 distinguishes self-hub access from remote-hub caching.
- [x] G-011 status updated to record short-term-mitigation-shipped.

## Verification

grep -q "R3 — read-live, not cache" /opt/termlink/CLAUDE.md
grep -q "G-011" /opt/termlink/CLAUDE.md
grep -q "Short-term mitigation shipped" /opt/termlink/.context/project/concerns.yaml

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

### 2026-04-25T18:09:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1260-g-011-short-term-document-read-live-not-.md
- **Context:** Initial task creation

### 2026-04-25T18:11:02Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
