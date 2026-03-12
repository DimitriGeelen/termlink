# Framework Agent Pickup: Conversation Guard + /capture Skill

> Generated: 2026-03-12 | Task: T-094 | Project: 010-termlink

## What This Is

This project has solved the problem of volatile conversation loss in Claude Code.
Two deliverables are ready for contribution to the agentic-engineering-framework:

### 1. /capture Skill (`.claude/commands/capture.md`)
An emergency conversation capture skill. When invoked, it:
- Reads the live session JSONL transcript
- Extracts the current topic's conversation (5-minute gap heuristic)
- Writes a structured research artifact to `docs/reports/`
- Commits the artifact

**Files:**
- `.claude/commands/capture.md` — the skill prompt
- `agents/capture/read-transcript.py` — JSONL transcript reader
- `.fabric/components/capture-reader.yaml` — fabric card
- `.fabric/components/capture-skill.yaml` — fabric card

### 2. C-002 Exploratory Conversation Guard (CLAUDE.md rule)
A new rule in the Inception Discipline section:

> If a substantive conversation on an untracked topic reaches 3+ exchanges without
> an active task, STOP: create an inception task, invoke /capture, continue under
> the task.

**Location:** CLAUDE.md, Inception Discipline section, item 8 (C-002).

## Why It Matters

Claude Code fires zero hooks for pure conversation sessions. The task gate,
commit cadence, and session capture checklist only trigger on tool use. A session
that never touches a file can run indefinitely and lose all content on context
exhaustion (registered as G-005 in this project's gaps.yaml).

**Root cause of the original problem:** A 45-minute Agent Mesh research session
(multi-agent coordination architecture) was completely lost when context exhausted
mid-conversation. No hooks fired. No artifacts written. No recovery possible.

## PR Scope

To contribute to the framework, create a PR with:

1. **`agents/capture/read-transcript.py`** — copy verbatim from this project
2. **`.claude/commands/capture.md`** — copy verbatim
3. **CLAUDE.md** — add C-002 rule to Inception Discipline section (item 8)
4. **`.fabric/components/`** — add capture-reader.yaml and capture-skill.yaml

**Note:** The JSONL path encoding (`project_root.replace('/', '-')`) has been
validated on macOS. Test on Linux before merging.

## Known Limitations

- G-005: No hook coverage for pure conversation sessions — C-002 is self-governed
- /capture includes skill invocation text in the transcript (noisy but acceptable)
- Topic boundary detection is heuristic (5-min gap) — works well in practice

## Validation Evidence

Tested 2026-03-12 in a real session. Artifact at:
`docs/reports/T-108-capture-2026-03-11-23.md`
24 turns captured, topic boundary correctly identified, Key Insights accurate.
