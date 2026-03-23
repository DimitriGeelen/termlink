# T-233: Specialist Agent Orchestration — Research Artifact

## Problem Statement

Today, a single Claude Code agent handles everything: research, coding, infrastructure, design, testing. This creates two problems:

1. **Context pollution** — a coding agent's context fills up with research findings, design exploration, and infra commands that dilute its core task
2. **No specialization** — each agent starts from zero; there's no way to pre-load domain context (e.g., "you're the infrastructure agent, here's what you know about our servers")

The vision: an **orchestrator agent** that recognizes "I need research" or "I need infrastructure work" and delegates to **specialist agents** that are pre-loaded with relevant context, running as TermLink sessions.

## What Exists Today

### TermLink primitives that could support this:
- **`termlink spawn`** — start a new session with name/roles/tags
- **`termlink agent ask`** — typed request-response between agents (ask/listen protocol)
- **`termlink interact`** — inject a command into a session and capture output
- **`termlink inject`** — send keystrokes to a session
- **`termlink mirror`** (NEW) — observe what an agent is doing
- **Hub** — central routing for multi-agent coordination
- **Events** — pub/sub for agent-to-agent signaling

### Framework primitives:
- **Sub-Agent Dispatch Protocol** (CLAUDE.md) — rules for using Claude Code's Task tool
- **`fw bus`** — result ledger for sub-agent outputs
- **Episodic memory** — completed task histories for context

## Dialogue Log

*(To be filled during inception dialogue with human)*

