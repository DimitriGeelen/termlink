# Framework Agent Pickup: Conversation Guard + /capture Skill PR

> Task: T-098 | Generated: 2026-03-12

## What You Need To Do

Create a PR in the agentic-engineering-framework repo on OneDev to contribute:
1. The `/capture` skill (emergency conversation capture)
2. The C-002 Exploratory Conversation Guard rule

These were built in project 010-termlink to solve a real problem: pure conversation
sessions fire zero hooks in Claude Code, so valuable research conversations can be
lost silently on context exhaustion.

## OneDev Location

- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: create `feature/conversation-guard-capture-skill`

## Files to Include in the PR

### New files (copy verbatim from 010-termlink project):

**`agents/capture/read-transcript.py`**
JSONL transcript reader. Finds the current Claude Code session JSONL at
`~/.claude/projects/<project-encoded>/`, extracts conversation turns since the
last topic boundary (5-minute gap heuristic), outputs JSON.
- Path encoding: `project_root.replace('/', '-')` — no lstrip
- Format canary validates `user`/`assistant` event types
- Flags: `--dry-run`, `--last-n N`, `--all`

**`.claude/commands/capture.md`**
The /capture skill prompt. Reads focus.yaml for current task, runs the reader,
writes structured artifact to `docs/reports/{task}-capture-{YYYY-MM-DD-HH}.md`,
commits. Six sections: Topic, Key Insights, Options Explored, Decisions Made,
Open Questions, Conversation Log (verbatim).

**`.fabric/components/capture-reader.yaml`** and **`capture-skill.yaml`**
Fabric cards for the two new components.

### Modified files:

**`CLAUDE.md`** — Add to Inception Discipline section:
```
8. **Exploratory Conversation Guard (C-002)** — If a substantive conversation
   on an untracked topic reaches 3+ exchanges without an active task, STOP and:
   1. Create an inception task for the topic
   2. Invoke `/capture` to save the prior dialogue to disk
   3. Continue the conversation under the new task
   Trigger: 3 substantive exchanges (exclude greetings, one-word replies, status
   checks). Enforcement: agent self-governs; no hook coverage (G-005).
```

## PR Description Template

**Title:** `feat: /capture skill + C-002 Exploratory Conversation Guard`

**Body:**
```
## Problem

Claude Code fires zero hooks for pure conversation sessions. A session that never
touches a file can run indefinitely and lose all content on context exhaustion.
This is registered as G-005 in affected projects.

Root cause: A 45-minute Agent Mesh architecture research session was completely
lost when context exhausted mid-conversation. No hooks fired. No recovery possible.

## Solution

Two layers:
1. **C-002 rule** (CLAUDE.md): Self-governing protocol — stop at 3 substantive
   exchanges on an untracked topic, create inception task, invoke /capture.
2. **/capture skill**: Reads the live JSONL transcript, extracts current topic,
   writes structured artifact, commits. Emergency ejector seat.

## Validation

Tested in a real session (2026-03-12). 24 turns captured, topic boundary correctly
identified, Key Insights accurate and non-hallucinated.
Artifact: docs/reports/T-108-capture-2026-03-11-23.md

## Limitations

- JSONL path encoding validated on macOS — test on Linux before merging
- Topic boundary is heuristic (5-min gap) — works well in practice
- /capture captures skill invocation text in log (noisy but acceptable)
```

## After Creating the PR

1. Post the PR URL in the project's T-098 task as a comment
2. Update T-098 status to work-completed
