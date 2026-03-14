# T-109: /capture Skill — Framework PR Research Artifact

> Task: T-109 | Date: 2026-03-14

## What /capture Is

`/capture` is an emergency conversation capture skill for Claude Code. When a
valuable conversation is about to be lost (context exhaustion, session crash,
topic shift), the user types `/capture` and the agent saves the conversation
to a structured Markdown artifact on disk.

## Why It Exists

Conversations are ephemeral — context compaction, crashes, and session limits
destroy them. The C-002 rule (Exploratory Conversation Guard) tells agents to
invoke `/capture` when untracked dialogue exceeds 3 exchanges. Without the
skill itself, the rule has no implementation.

## Components

### `agents/capture/read-transcript.py`
JSONL transcript reader. Finds the current Claude Code session transcript at
`~/.claude/projects/<project-encoded>/`, extracts conversation turns since the
last topic boundary (5-minute gap heuristic), outputs JSON.

Key details:
- Path encoding: `project_root.replace('/', '-')` — no lstrip
- Format canary validates `user`/`assistant` event types
- Flags: `--dry-run`, `--last-n N`, `--all`
- Python 3.6+ (stdlib only, no dependencies)

### `.claude/commands/capture.md`
The /capture skill prompt. Workflow:
1. Read `focus.yaml` for current task
2. Run `read-transcript.py` to extract turns
3. Write structured artifact to `docs/reports/{task}-capture-{YYYY-MM-DD-HH}.md`
4. Commit via `fw git commit`

Artifact sections: Topic, Key Insights, Options Explored, Decisions Made,
Open Questions, Conversation Log (verbatim turns).

### Fabric cards
- `.fabric/components/capture-reader.yaml` — for read-transcript.py
- `.fabric/components/capture-skill.yaml` — for the skill prompt

## Files for PR

| File | Action | Notes |
|------|--------|-------|
| `agents/capture/read-transcript.py` | New | Copy from 010-termlink |
| `.claude/commands/capture.md` | New | Copy from 010-termlink |
| `.fabric/components/capture-reader.yaml` | New | Component card |
| `.fabric/components/capture-skill.yaml` | New | Component card |

No existing files modified. The CLAUDE.md C-002 rule change was covered by T-098.

## Validation

Tested 2026-03-12 in a real session (T-108). 24 turns captured, topic boundary
correctly identified via 5-minute gap heuristic. Key Insights section was
accurate and non-hallucinated. Artifact committed and persisted.

Second real-world use: T-125 retrospective capture (2026-03-13), 14 turns.

## Integration Notes

- **Claude Code only** — reads JSONL transcript format specific to Claude Code
- **macOS validated** — JSONL path encoding tested on macOS; Linux testing recommended
- **Python 3 required** — uses only stdlib (json, os, re, sys, argparse, datetime)
- **Topic boundary heuristic** — 5-minute gap between turns; works well in practice

## Limitations

- `/capture` captures its own skill invocation text in the conversation log (noisy but acceptable)
- Topic boundary is heuristic — edge cases possible with long pauses mid-topic
- No Windows support (Claude Code limitation, not /capture-specific)
