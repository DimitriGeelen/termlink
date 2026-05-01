---
id: T-1431
name: "/agent-handoff claude-code skill — wraps T-1429 verb (T-1425 pick #5)"
description: >
  From T-1425 fast-forward synthesis. Plugin-level skill, NOT in CLAUDE.md. Lives in .claude/skills/agent-handoff.md (or in a plugin file if we have one). Sequence: verify task exists → termlink whoami (lock identity) → termlink agent contact <target> --thread <task-id> --message <summary> → verify offset returned → update task with posted=offset, status hint=awaiting-reply. CLAUDE.md cost: ONE line — 'for cross-host handoffs use /agent-handoff'. Depends on T-1429 (the verb being wrapped) and T-1427 (identity binding the skill enforces). Independent of T-1430 (topic self-doc).

status: captured
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T07:02:51Z
last_update: 2026-05-01T07:02:51Z
date_finished: null
---

# T-1431: /agent-handoff claude-code skill — wraps T-1429 verb (T-1425 pick #5)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [ ] T-1429 (`agent contact` verb) has shipped — skill wraps it, so verb must exist first
- [ ] T-1427 (whoami + binding) has shipped — skill calls `whoami` to lock identity before posting
- [ ] Skill file exists at `.claude/skills/agent-handoff.md` (or in the project's plugin dir if the repo uses one — confirm location during build) with frontmatter declaring `argument-hint: <target> <task-id>`
- [ ] Skill body executes the canonical sequence: (1) verify task `<task-id>` exists in `.tasks/active/`; (2) `termlink whoami` and capture self sender_id; (3) `termlink agent contact <target> --thread <task-id> --message "$(extract task summary)" --json`; (4) parse offset from JSON output; (5) update task with `posted=offset, status_hint=awaiting-reply` in Updates section; (6) print summary to user
- [ ] Skill fails fast (exit non-zero with a clear error) if any step fails — task missing, whoami fails, contact returns non-zero, etc. No silent fallbacks
- [ ] CLAUDE.md gains exactly ONE line under "Quick Reference" or equivalent — "for cross-host handoffs use `/agent-handoff <target> <task-id>`". No prose elsewhere. The skill's own description carries the canon
- [ ] Skill is invocable via `/agent-handoff` slash command and listed in the available-skills surface
- [ ] Skill prompt explicitly disallows: improvising sender labels, using `remote push` or `inbox.push`, posting to `agent.reply` or any other invented topic
- [ ] Smoke test in skill body or accompanying test: invoke against a known target with a known task ID, verify offset returned and task updated

### Human
- [ ] [RUBBER-STAMP] Verify the skill works end-to-end from a real session
  **Steps:**
  1. From a fresh Claude Code session in /opt/termlink: `/agent-handoff ring20-management-agent T-1429`
  2. Watch the output — should see whoami → contact → offset → task update sequence
  3. `cat .tasks/active/T-1429-*.md | grep -A2 "posted="` — see the update entry
  4. `termlink channel subscribe agent-chat-arc --cursor <last-known> --limit 5` — confirm post landed with correct metadata.thread=T-1429
  **Expected:** end-to-end works without prompts, manual fallbacks, or improvisation
  **If not:** capture failure point in Updates and re-scope which step broke

## Verification

test -f .claude/skills/agent-handoff.md
grep -q "agent contact" .claude/skills/agent-handoff.md
grep -qi "whoami" .claude/skills/agent-handoff.md
! grep -qi "remote push\|inbox.push\|agent.reply" .claude/skills/agent-handoff.md

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

### 2026-05-01T07:02:51Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1431-agent-handoff-claude-code-skill--wraps-t.md
- **Context:** Initial task creation
