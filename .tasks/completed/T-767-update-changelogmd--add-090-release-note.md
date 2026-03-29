---
id: T-767
name: "Update CHANGELOG.md — add 0.9.0 release notes for features since 0.8.0"
description: >
  Update CHANGELOG.md — add 0.9.0 release notes for features since 0.8.0

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-29T23:05:03Z
last_update: 2026-03-29T23:06:48Z
date_finished: 2026-03-29T23:06:48Z
---

# T-767: Update CHANGELOG.md — add 0.9.0 release notes for features since 0.8.0

## Context

CHANGELOG.md has entries for 0.8.0, 0.7.0, etc. but is missing a 0.9.0 section (307 commits, major features: vendor, push, file transfer, agent protocol, dispatch improvements, release optimization).

## Acceptance Criteria

### Agent
- [x] Add 0.9.0 section to CHANGELOG.md with Added/Changed/Fixed subsections
- [x] Cover major features: vendor, push, file transfer, agent protocol, dispatch, release optimization, Linux aarch64
- [x] Follow Keep a Changelog format consistent with existing entries
- [x] Include test count update

## Verification

grep -q "0.9.0" CHANGELOG.md

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

### 2026-03-29T23:05:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-767-update-changelogmd--add-090-release-note.md
- **Context:** Initial task creation

### 2026-03-29T23:06:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
