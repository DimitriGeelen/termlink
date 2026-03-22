---
id: T-219
name: "Fix GitHub repo URL in Cargo.toml and add github remote"
description: >
  Fix GitHub repo URL in Cargo.toml and add github remote

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-22T21:22:23Z
last_update: 2026-03-22T21:22:23Z
date_finished: null
---

# T-219: Fix GitHub repo URL in Cargo.toml and add github remote

## Context

Cargo.toml has `repository = "https://github.com/dimidev32/termlink"` but all docs, specs, README, and Homebrew formula reference `DimitriGeelen/termlink`. Also, no `github` remote is configured — only OneDev origin exists. The release workflow and Homebrew formula depend on the GitHub mirror.

## Acceptance Criteria

### Agent
- [x] Cargo.toml repository URL matches docs (`DimitriGeelen/termlink`)
- [x] `github` remote added pointing to `DimitriGeelen/termlink`

## Verification

grep -q 'DimitriGeelen/termlink' Cargo.toml
git remote get-url github

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-03-22T21:22:23Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-219-fix-github-repo-url-in-cargotoml-and-add.md
- **Context:** Initial task creation
