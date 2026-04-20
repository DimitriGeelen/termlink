---
id: T-1168
name: "T-1074 follow-up: channel:learnings publisher + subscriber on T-1155 bus"
description: >
  Cross-agent learning exchange on top of T-1155 channel bus. Publish learnings to channel:learnings on fw context add-learning; subscribe daemon writes to received-learnings.yaml; Watchtower fleet-insights panel. Depends on T-1158 (bus crate), T-1159 (ed25519 identity), T-1160 (channel API). Replaces the 15-min cron design from T-1074 inception — see docs/reports/T-1074-cross-agent-learning-exchange-inception.md for rationale.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [T-1074, T-1155, bus, learnings-exchange]
components: []
related_tasks: [T-1074, T-1155, T-1158, T-1159, T-1160, T-1161]
created: 2026-04-20T14:43:26Z
last_update: 2026-04-20T14:43:26Z
date_finished: null
---

# T-1168: T-1074 follow-up: channel:learnings publisher + subscriber on T-1155 bus

## Context

Cross-agent learning exchange on top of the T-1155 channel bus. Replaces the 15-min cron design from T-1074 inception — see `docs/reports/T-1074-cross-agent-learning-exchange-inception.md` for the full spike evidence and the bus-pivot rationale.

**Dependencies:** T-1158 (bus crate), T-1159 (ed25519 identity), T-1160 (channel API). This task cannot start until those three land.

**Scope:** one topic + one publisher hook + one subscriber daemon + one Watchtower panel. Designed to fit one session once dependencies are in place.

## Acceptance Criteria

### Agent
- [ ] `channel:learnings` topic defined in bus schema with `(origin_project, L-id)` dedup key
- [ ] `fw context add-learning` publishes the new entry to `channel:learnings` via `channel.post` (T-1160 API)
- [ ] Subscriber daemon writes incoming entries to `.context/project/received-learnings.yaml` (separate file to preserve origin authorship per T-1074 scope fence)
- [ ] Dedup on `(origin_project, id)` — re-receiving the same entry is idempotent
- [ ] Envelope includes `origin_project` + `origin_hub_fingerprint` (T-1052 R1 — lets receivers spot pre-rotation learnings)
- [ ] Watchtower "fleet insights" panel renders `received-learnings.yaml`
- [ ] Never auto-applies received learnings — humans decide promotion to local rules

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

### 2026-04-20T14:43:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1168-t-1074-follow-up-channellearnings-publis.md
- **Context:** Initial task creation
