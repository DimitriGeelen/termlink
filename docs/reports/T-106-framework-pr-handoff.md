# T-106: Framework PR Handoff — Inception Research

> Task: T-106 | Started: 2026-03-12 | Type: inception

## Evidence Base

Two framework PR handoffs completed:
- `docs/framework-agent-pickups/T-094-conversation-guard-capture-skill.md`
- `docs/framework-agent-pickups/T-098-conversation-guard-pr.md`

Both were written ad-hoc. Analysis of common structure reveals a stable template.

## Inception Answers

### 1. Skill vs. Agent
**Decision: Neither (yet).** A Markdown template is sufficient at current volume (~2 PRs so far).
A `/fw-pr` skill would be premature — template + AC checkbox in build tasks provides enough structure.
Revisit when volume reaches 5+ PRs.

### 2. Pickup Prompt Template
Extracted from the two existing prompts:

```markdown
# Framework Agent Pickup: {Title}

> Task: T-XXX | Generated: {date} | Project: {project}

## What You Need To Do
{1-3 sentence summary of what the framework agent should do}

## OneDev Location
- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: `feature/{branch-name}`

## Files to Include
### New files:
{list with paths and brief descriptions}

### Modified files:
{list with paths and what to change}

## PR Description Template
**Title:** `{PR title}`
**Body:**
{full PR body text}

## Validation Evidence
{how this was tested, artifact references}

## After Creating the PR
1. Post the PR URL in the project task
2. Update task status to work-completed
```

### 3. TermLink Migration Path
When Agent Mesh exists:
- Replace "copy-paste to console" with `termlink emit framework-agent pr.pickup --payload <file>`
- The pickup prompt file format stays the same — it's the transport that changes
- No design changes needed now — the template is transport-agnostic

### 4. Enforcement
The build task template already has an AC: "If this work belongs in the framework: framework PR task created."
This is sufficient. No additional gate needed.

## Decision

**GO (lightweight)** — Codify the template at `docs/templates/framework-pickup-prompt.md`.
No skill/agent build needed yet. The template + existing AC checkbox is the right level of structure.
