---
id: T-110
name: "Transcript retention policy — fw transcripts clean command"
description: >
  ~/.claude/projects/ grows unbounded at ~65 MB/day with zero cleanup mechanism.
  Build a retention policy (30-day default TTL) and fw transcripts clean command
  to prevent multi-GB accumulation. Discovered via T-107 deep-dive.
status: captured
workflow_type: build
owner: agent
horizon: next
tags: [transcripts, retention, cleanup, storage, maintenance]
components: []
related_tasks: [T-107]
created: 2026-03-12T00:00:00Z
last_update: 2026-03-12T00:00:00Z
date_finished: null
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
- [ ] `fw transcripts clean` command implemented
- [ ] `--dry-run` mode works (no deletions)
- [ ] `--older-than N` flag works
- [ ] Current session is never deleted
- [ ] `fw doctor` or `fw transcripts size` shows current usage

## Verification

fw transcripts clean --dry-run 2>&1 | grep -q "dry-run"
