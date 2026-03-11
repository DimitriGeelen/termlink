---
id: T-110
name: "Transcript retention policy — fw transcripts clean command"
description: >
  ~/.claude/projects/ grows unbounded at ~65 MB/day with zero cleanup mechanism.
  Build a retention policy (30-day default TTL) and fw transcripts clean command
  to prevent multi-GB accumulation. Discovered via T-107 deep-dive.
status: work-completed
workflow_type: build
owner: agent
horizon: next
tags: [transcripts, retention, cleanup, storage, maintenance]
components: []
related_tasks: [T-107]
created: 2026-03-12T00:00:00Z
last_update: 2026-03-11T23:40:10Z
date_finished: 2026-03-11T23:40:10Z
---

# T-110: Transcript Retention Policy — fw transcripts clean

## Context

From T-107 deep-dive: `~/.claude/projects/<project>/` is 261 MB after 4 days,
growing at ~65 MB/day. One heavy multi-agent session = 201 MB. No cleanup mechanism
exists. 30-day trajectory: 1.8 GB. 6-month: 8+ GB.

Meta files contain only `{"agentType":"Explore"}` — no timestamps for selective cleanup.

## What to Build

`fw transcripts clean` command (shell script or Python):
- Default: delete session dirs older than 30 days
- `--older-than N` flag (days)
- `--dry-run` flag (show what would be deleted, no action)
- `--size-report` flag (show current usage breakdown by session)
- Safety: never delete current session

Optionally: add `~/.claude/` size to `fw doctor` health output.

## Acceptance Criteria

### Agent
- [x] `fw transcripts clean` command implemented (at agents/transcripts/transcripts.sh)
- [x] `--dry-run` mode works (no deletions)
- [x] `--older-than N` flag works
- [x] Current session is never deleted
- [x] `fw doctor` or `fw transcripts size` shows current usage (size command)

## Verification

test -x agents/transcripts/transcripts.sh
PROJECT_ROOT=/Users/dimidev32/001-projects/010-termlink bash agents/transcripts/transcripts.sh clean --dry-run > /tmp/t110-clean.txt 2>&1; grep -q "DRY RUN" /tmp/t110-clean.txt
PROJECT_ROOT=/Users/dimidev32/001-projects/010-termlink bash agents/transcripts/transcripts.sh size > /tmp/t110-size.txt 2>&1; grep -q "Total" /tmp/t110-size.txt

### 2026-03-11T23:33:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T23:40:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
