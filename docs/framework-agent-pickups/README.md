# Framework Agent Pickup Prompts

This directory contains pickup prompts for the framework agent — one file per
framework PR. Each file is written here for persistence (never lost) and also
printed to console (for copy-paste while Agent Mesh doesn't exist yet).

## Future state

When Agent Mesh Phase 1 is complete, these prompts will be routed directly to
the framework agent via:
  termlink emit framework-agent pr.pickup --payload <file>

No copy-paste. No different-system problem.

## Files

| File | Task | Status |
|---|---|---|
| T-094-conversation-guard-capture-skill.md | T-094 | Research artifact |
| T-098-conversation-guard-pr.md | T-098 | PR pickup (CLAUDE.md rule + /capture) |
| T-109-capture-skill-pr.md | T-109 | PR pickup (/capture skill tooling only) |
| T-118-fabric-context-llm-enrichment.md | T-118 | PR pickup (fw fabric context) |

## How to use

1. Open the relevant `.md` file
2. Copy the entire content
3. Paste into a fresh framework agent session
4. The agent will create a task, read the context, and start working
