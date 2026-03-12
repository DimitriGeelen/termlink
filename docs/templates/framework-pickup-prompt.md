# Framework Agent Pickup: {Title}

> Task: T-XXX | Generated: {YYYY-MM-DD} | Project: {project-name}

## What You Need To Do

{1-3 sentence summary of what the framework agent should create/modify in the
agentic-engineering-framework repo.}

## OneDev Location

- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: create `feature/{descriptive-branch-name}`

## Files to Include in the PR

### New files (copy verbatim from this project):

{For each file:}
**`{path-in-framework-repo}`**
{Brief description of what the file does, key behaviors, any assumptions.}

### Modified files:

{For each file:}
**`{path-in-framework-repo}`** — {What to add/change and where.}

## PR Description Template

**Title:** `{type}: {short description}`

**Body:**
```
## Problem

{Why this change is needed. Root cause if applicable.}

## Solution

{What was built/changed. Key design decisions.}

## Validation

{How it was tested. Artifact references from the source project.}

## Limitations

{Known issues, platform assumptions, edge cases.}
```

## Validation Evidence

{Reference the test artifact, e.g.:}
- Tested in a real session ({date}). Artifact at: `docs/reports/T-XXX-*.md`
- {Key metrics: turns captured, tests passed, etc.}

## After Creating the PR

1. Post the PR URL in the project's T-XXX task as a comment
2. Update T-XXX status to work-completed
