---
id: T-106
name: "Inception — Streamline framework agent PR handoff process"
description: >
  The current process for handing work to the framework agent is manual and ad-hoc:
  write a research artifact, create a OneDev PR, generate a console prompt, copy-paste
  it to the framework agent running on a different system. This works today but is
  fragile and will become a bottleneck as we produce more framework PRs. Explore
  making this a skill or agent — structured, repeatable, eventually TermLink-routed.
status: captured
workflow_type: inception
owner: human
horizon: later
tags: [framework, workflow, skill, agent-mesh, pr-handoff]
components: []
related_tasks: [T-094, T-099, T-101, T-103, T-104, T-105]
created: 2026-03-11T13:30:00Z
last_update: 2026-03-11T13:30:00Z
date_finished: null
---

# T-106: Inception — Streamline Framework Agent PR Handoff Process

## Problem Statement

Every time we produce work that belongs in the framework (not just this project),
we need to:
1. Write a research artifact with full context, dialogue, options, decisions
2. Create a OneDev PR in the framework repo
3. Generate a pickup prompt for the framework agent
4. Print it to console (for copy-paste — framework agent runs on a different system)
5. Write it to `docs/framework-agent-pickups/` (for persistence)

This is currently done manually per PR, with no template, no enforcement, and high
risk of steps being skipped or content being lost.

**Future state (with Agent Mesh):** the pickup prompt becomes a TermLink message
routed directly to the framework agent — no copy-paste, no different-system problem.
But that's Phase 1 of Agent Mesh. Today we need a process that works without it.

## Questions to Explore

1. **Skill or agent?**
   - Skill (`/fw-pr`): lightweight, invoked manually, prompt-driven
   - Agent (dedicated script): structured, runs checks, validates all steps complete
   - Which fits better into the current workflow?

2. **What does the skill/agent do?**
   - Read the completed build task and its related inception
   - Pull the research artifact content
   - Scaffold the OneDev PR body (title, description, acceptance criteria)
   - Generate the framework agent pickup prompt from a template
   - Write prompt to `docs/framework-agent-pickups/T-XXX-<name>.md`
   - Print to console
   - Open OneDev PR via API (if credentials available) or output the URL + body for manual creation

3. **Template for pickup prompts**
   - What sections does every framework agent pickup need?
   - Problem statement, evidence, proposed implementation, reference files, task IDs
   - How much is project-specific vs. reusable template?

4. **TermLink migration path**
   - When Agent Mesh Phase 1 exists: replace "print to console" with `termlink emit framework-agent pr.pickup --payload <file>`
   - The skill/agent should have a flag: `--via termlink` vs `--via console`
   - Design the interface now so the migration is a one-line change later

5. **Enforcement**
   - Should the build task completion gate check that a framework PR task exists?
   - Or is the AC checkbox in the build template sufficient?

## Relationship to Other Tasks

- **T-099, T-101, T-103, T-104, T-105:** All will produce framework PRs.
  This task designs the repeatable process they all use.
- **Agent Mesh (T-100 area):** TermLink routing replaces console copy-paste in Phase 1.
- **T-098:** The ad-hoc version of this — done manually, no template, no enforcement.
  T-106 makes T-098's approach repeatable.

## Scope Fence

**IN:** Design the skill/agent, define the template, map the TermLink migration path
**OUT:** Implementation before GO decision

## Acceptance Criteria

### Agent
- [ ] Skill vs. agent decision made with rationale
- [ ] Pickup prompt template defined
- [ ] TermLink migration path designed
- [ ] GO/NO-GO framed for discussion

### Human
- [ ] Design reviewed and direction decided

## Decisions

## Decision

## Updates
