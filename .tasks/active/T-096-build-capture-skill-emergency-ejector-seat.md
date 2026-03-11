---
id: T-096
name: "Build /capture skill — emergency ejector seat for untracked conversations"
description: >
  Create .claude/commands/capture.md — a skill that acts as an emergency rescue tool
  when a conversation has progressed without a task or research artifact. User types
  /capture, provides a summary, and the skill writes it to disk, creates a proper
  research artifact, and commits. Cannot access conversation history automatically
  (Claude Code platform limit), but provides a fast, structured path from "nothing
  written" to "committed artifact" in one command.
status: captured
workflow_type: build
owner: agent
horizon: now
tags: [framework, skills, session-capture, tooling]
components: []
related_tasks: [T-094, T-095]
created: 2026-03-11T11:30:00Z
last_update: 2026-03-11T11:30:00Z
date_finished: null
---

# T-096: Build `/capture` Skill

## Context

Spawned from T-094 inception. Agent 4 confirmed full feasibility.
See: `docs/reports/T-094-volatile-conversation-prevention.md` (Agent 4 findings + design)

## Skill Design

When user types `/capture`:

1. Read `.context/working/focus.yaml` — get current task (or prompt to create one)
2. Ask: "Briefly describe what we've been discussing and key insights/decisions so far."
3. User provides summary (can paste prior conversation excerpts)
4. Skill writes `docs/reports/T-XXX-capture-{timestamp}.md` with structured sections:
   - Topic / Problem Statement
   - Key Insights
   - Options Explored + Decisions
   - Open Questions
   - Dialogue Log (user-provided content)
5. Run `fw git commit -m "T-XXX: /capture — [topic]"`
6. Report: file path + commit hash

## Acceptance Criteria

### Agent
- [ ] `.claude/commands/capture.md` created with full prompt
- [ ] Skill handles case where no active task exists (prompts to create one first)
- [ ] Output artifact follows C-001 structure (sections: Topic, Insights, Options, Decisions, Open Questions, Dialogue Log)
- [ ] Skill runs `git commit` on completion
- [ ] Tested manually: `/capture` invocation produces committed file

### Human
- [ ] Tested `/capture` in a real session and found it useful

## Verification

test -f /Users/dimidev32/001-projects/010-termlink/.claude/commands/capture.md

## Decisions

### 2026-03-11 — Cannot auto-capture conversation history
- **Chose:** User-provided summary (interactive)
- **Why:** Claude Code hooks only fire on tool calls; conversation content is not accessible to skills or hooks
- **Rejected:** Automatic background capture — architecturally impossible per Agent 1 + 2 findings
- **Implication:** Skill requires user discipline to invoke, but once invoked, the write + commit is automatic

## Updates
