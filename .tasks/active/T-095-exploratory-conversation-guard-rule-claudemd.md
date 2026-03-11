---
id: T-095
name: "Exploratory Conversation Guard — add rule to CLAUDE.md"
description: >
  Add the "Exploratory Conversation Guard" rule to CLAUDE.md's Inception Discipline
  section. This closes the trigger gap in C-001: conversations that begin organically
  (via /resume or autonomous mode) without a task currently bypass all enforcement.
  The new rule requires the agent to stop at 3 substantive exchanges on an untracked
  topic and create an inception task + artifact before continuing.
status: captured
workflow_type: build
owner: agent
horizon: now
tags: [framework, governance, claude-md, session-capture]
components: []
related_tasks: [T-094]
created: 2026-03-11T11:30:00Z
last_update: 2026-03-11T11:30:00Z
date_finished: null
---

# T-095: Exploratory Conversation Guard — Add Rule to CLAUDE.md

## Context

Spawned from T-094 inception. The 5-agent exploration confirmed that CLAUDE.md has no
rule preventing organic conversations from running without tasks. C-001 only fires when
an inception task already exists. This rule closes that gap.

See: `docs/reports/T-094-volatile-conversation-prevention.md` (Agent 3 findings + draft rule)

## Acceptance Criteria

### Agent
- [ ] Exploratory Conversation Guard rule added to CLAUDE.md Inception Discipline section
- [ ] Rule includes: trigger condition (3+ exchanges), required actions (create task, create artifact, log prior dialogue), commit cadence
- [ ] Dialogue Log requirement explicitly stated
- [ ] Rule is positioned after C-001 (Research artifact first) as C-002

## Verification

grep -q "Exploratory Conversation Guard" /Users/dimidev32/001-projects/010-termlink/CLAUDE.md

## Decisions

### 2026-03-11 — Rule placement
- **Chose:** Add as C-002 in Inception Discipline section (after C-001)
- **Why:** Logically follows "research artifact first" — this is the trigger that makes C-001 relevant for organic conversations
- **Rejected:** New top-level section — would break existing structure

## Updates
