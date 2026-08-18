---
id: T-107
name: "Inception — Sub-agent transcript persistence"
description: >
  Background agents (spawned via Claude Code's Agent tool) write their transcripts
  to /tmp output files with isSidechain: true, NOT to the project JSONL. These files
  are ephemeral — cleared on reboot or tmp cleanup. Sub-agent reasoning trails (how
  they reached conclusions, what they tried, what failed) are lost. Results are
  captured via fw bus, but the thinking behind them is not. Explore options.
status: work-completed
workflow_type: inception
owner: human
horizon:
tags: [sub-agents, transcript, persistence, ephemeral, sidechain]
components: []
related_tasks: [T-094, T-101, T-104]
created: 2026-03-11T14:00:00Z
last_update: '2026-08-18T18:58:43Z'
date_finished: 2026-03-11T23:31:48Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:44Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 2
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 2
    rationale: D1=2 (no-signal); D2=2 (no-signal); D3=2 (no-signal); D4=2 
      (no-signal); F-RECALL=2 (no-signal); F-ORCH=2 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:43Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 4
      effort: 6
    rationale: blast_radius=0 (no-signal); tier=4 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

# T-107: Inception — Sub-agent Transcript Persistence

## Problem Statement

Background agents write their full transcripts (reasoning, tool calls, errors,
findings) to `/tmp/claude-501/.../tasks/<agent-id>.output` with `isSidechain: true`.
These files are NOT written to the project JSONL. They are ephemeral.

What we currently capture from sub-agents:
- **Results** → `fw bus post` → persistent ✓
- **Reasoning trails** → `/tmp` → ephemeral ✗
- **Tool calls + errors** → `/tmp` → ephemeral ✗

Today's 5 research agents did valuable investigative work. Their tool calls,
hypotheses, and intermediate findings exist only in `/tmp` right now.

## What We Know

From investigation:
- Output files live at: `/tmp/claude-501/<project-encoded>/tasks/<agent-id>.output`
- Format: JSONL lines, same structure as main transcript, with `isSidechain: true`
  and `agentId: <id>` fields
- Session ID matches the parent session
- File size example: agent a8a624b1 produced 170KB of transcript

## Questions to Explore

1. **Is the `/tmp` path stable?**
   - Is `/tmp/claude-501/` always `claude-501` or does it encode the UID?
   - Is the path structure consistent across sessions?

2. **When should we copy/archive them?**
   - Option A: At PostToolUse after agent completes (we know the agent ID from the result)
   - Option B: At PreCompact (sweep all `/tmp` agent files before session ends)
   - Option C: Part of `fw handover` — archive sub-agent transcripts as part of session wrap

3. **Where should they be stored?**
   - `.context/sub-agents/<session-id>/<agent-id>.jsonl` — alongside episodic memory
   - `.context/telemetry/sub-agents/` — alongside T-104's tool call store
   - Compressed? These files can be large (170KB each, 5 agents = 850KB/session)

4. **What's the value vs. cost tradeoff?**
   - Value: full reasoning trail available for debugging, pattern mining (T-103/T-104)
   - Cost: storage (manageable), processing (low — just copy, not parse)
   - Risk: `/tmp` might be cleared before we get to it

5. **Relationship to T-104 (tool call capture):**
   - Sub-agent tool calls and errors should feed T-104's store
   - T-104's parser needs to handle both main JSONL and sidechain files
   - Design T-107 storage with T-104 consumption in mind

## Scope Fence

**IN:** Understand the file structure, design the archival approach
**OUT:** Implementation — explore and decide only

## Acceptance Criteria

### Agent
- [x] `/tmp` path structure confirmed stable (symlinks to ~/.claude/ — uid 501 is actual unix uid)
- [x] Archival timing decided — not needed, already persisted by Claude Code
- [x] Storage location decided — `~/.claude/projects/<project>/<session>/subagents/` native
- [x] GO/NO-GO framed — NO-GO on original scope; T-110 spawned for retention policy

### Human
- [x] Approach reviewed and direction decided — user requested deep-dive before closing

## Decisions

**Decision**: NO-GO on original scope (ephemerality already solved). Two follow-on actions:
1. T-110 (new task): Transcript retention policy + `fw transcripts clean` command
2. T-104 design note: unified parser must consume sidechain files as primary error source

## Decision
<!-- inception-decision -->

**Decision**: NO-GO — sub-agent transcripts already persisted durably at `~/.claude/`. Original problem doesn't exist. Unbounded growth (65 MB/day, no cleanup) is a real concern — addressed via T-110.

## Updates

### 2026-03-11T23:23:10Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-11T23:31:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
