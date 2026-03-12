# Framework Agent Pickup: /capture Skill and JSONL Transcript Reader

> Task: T-109 | Generated: 2026-03-12

## What You Need To Do

Create a PR in the agentic-engineering-framework repo on OneDev to contribute the
`/capture` skill and its JSONL transcript reader. This is the implementation side
of the conversation guard work (T-098 covered the CLAUDE.md rule change; this
covers the tooling).

## OneDev Location

- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: create `feature/capture-skill`

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
commits via `fw git commit`. Six sections: Topic, Key Insights, Options Explored,
Decisions Made, Open Questions, Conversation Log (verbatim).

**`.fabric/components/capture-reader.yaml`** and **`capture-skill.yaml`**
Fabric cards for the two new components.

### No modified files

The CLAUDE.md rule change (C-002) was covered by T-098's PR. This PR is tooling only.

## PR Description Template

**Title:** `feat: /capture skill — emergency conversation capture`

**Body:**
```
## Problem

When a valuable conversation is about to be lost (context exhaustion, session crash),
there is no way to save it to disk. The C-002 rule (added separately) tells agents
to invoke /capture, but the skill itself needs to exist.

## Solution

- `agents/capture/read-transcript.py` — reads Claude Code's live JSONL transcript,
  extracts turns since topic boundary, outputs structured JSON
- `.claude/commands/capture.md` — skill prompt that reads focus.yaml, runs the reader,
  writes a structured artifact to docs/reports/, and commits

## Validation

Tested 2026-03-12 in a real session. 24 turns captured, topic boundary correctly
identified, Key Insights accurate and non-hallucinated.

## Limitations

- JSONL path encoding validated on macOS — test on Linux before merging
- Topic boundary is heuristic (5-min gap) — works well in practice
- /capture captures skill invocation text in log (noisy but acceptable)
```

## After Creating the PR

1. Post the PR URL in the project's T-109 task as a comment
2. Update T-109 status to work-completed
