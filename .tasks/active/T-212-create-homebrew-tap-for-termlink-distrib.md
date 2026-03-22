---
id: T-212
name: "Create Homebrew tap for TermLink distribution"
description: >
  Create Homebrew tap repo and formula for TermLink distribution — enables `brew install` on macOS without requiring Rust toolchain. Depends on release.yml CI and GitHub release artifacts.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [homebrew]
components: []
related_tasks: []
created: 2026-03-21T15:43:22Z
last_update: 2026-03-22T17:23:51Z
date_finished: null
---

# T-212: Create Homebrew tap for TermLink distribution

## Context

Build task from T-208 inception (GO). Homebrew tap enables `brew install termlink` on macOS — solving Gatekeeper, PATH, and Rust toolchain barriers. Requires GitHub repo creation (human action) and formula authoring.

## Acceptance Criteria

### Agent
- [ ] Homebrew formula file created (Formula/termlink.rb)
- [ ] Formula downloads pre-built binaries from GitHub Releases (aarch64 + x86_64)
- [ ] Formula includes SHA256 verification of downloaded binaries
- [ ] README documents `brew install` usage

### Human
- [ ] [RUBBER-STAMP] Create GitHub repo `DimitriGeelen/homebrew-termlink`
  **Steps:**
  1. Go to github.com/new
  2. Create repo named `homebrew-termlink` (public)
  3. Push the formula from this task
  **Expected:** Repo exists and contains Formula/termlink.rb
  **If not:** Check repo name matches exactly `homebrew-termlink`
- [ ] [REVIEW] Test `brew install DimitriGeelen/termlink/termlink` on macOS
  **Steps:**
  1. `brew tap DimitriGeelen/termlink`
  2. `brew install termlink`
  3. `termlink --version`
  **Expected:** termlink 0.1.0 installed, no Gatekeeper warning
  **If not:** Check formula URL points to valid release artifact

## Verification

# Formula file exists (will be created when building)
# Skipped until formula is written — human-blocked on repo creation

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

### 2026-03-21T15:43:22Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimitri/.termlink/.tasks/active/T-212-create-homebrew-tap-for-termlink-distrib.md
- **Context:** Initial task creation

### 2026-03-21T15:43:46Z — status-update [task-update-agent]
- **Change:** tags: +homebrew

### 2026-03-22T17:23:11Z — status-update [task-update-agent]
- **Change:** horizon: now → later
