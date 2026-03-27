---
id: T-212
name: "Create Homebrew tap for TermLink distribution"
description: >
  Create Homebrew tap repo and formula for TermLink distribution — enables `brew install` on macOS without requiring Rust toolchain. Depends on release.yml CI and GitHub release artifacts.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [homebrew]
components: []
related_tasks: []
created: 2026-03-21T15:43:22Z
last_update: 2026-03-27T17:57:12Z
date_finished: null
---

# T-212: Create Homebrew tap for TermLink distribution

## Context

Build task from T-208 inception (GO). Homebrew tap enables `brew install termlink` on macOS — solving Gatekeeper, PATH, and Rust toolchain barriers. Requires GitHub repo creation (human action) and formula authoring.

## Acceptance Criteria

### Agent
- [x] Homebrew formula file created (homebrew/Formula/termlink.rb)
- [x] Formula downloads pre-built binaries from GitHub Releases (aarch64 + x86_64 + linux)
- [x] Formula includes SHA256 verification of downloaded binaries
- [x] README documents `brew install` usage (homebrew/README.md)
- [x] SHA256 update script for releases (scripts/update-homebrew-sha.sh)

### Human
- [ ] [RUBBER-STAMP] Create GitHub repo `DimitriGeelen/homebrew-termlink`
  **Steps:**
  1. Go to github.com/new
  2. Create repo named `homebrew-termlink` (public)
  3. Clone the new repo, copy `homebrew/Formula/` and `homebrew/README.md` into it
  4. Push to GitHub
  **Expected:** Repo exists and contains Formula/termlink.rb
  **If not:** Check repo name matches exactly `homebrew-termlink`
- [ ] [RUBBER-STAMP] Create a release to generate binaries with real SHA256s
  **Steps:**
  1. `git push github main --tags` (pushes v0.9.0 tag to GitHub)
  2. Wait for GitHub Actions to complete (~5 min)
  3. `./scripts/update-homebrew-sha.sh v0.9.0`
  4. Commit and push updated formula to the tap repo
  **Expected:** Formula has real SHA256 hashes (not PLACEHOLDER)
  **If not:** Check release artifacts exist at GitHub releases page
- [ ] [REVIEW] Test `brew install DimitriGeelen/termlink/termlink` on macOS
  **Steps:**
  1. `brew tap DimitriGeelen/termlink`
  2. `brew install termlink`
  3. `termlink --version`
  **Expected:** termlink installed, no Gatekeeper warning
  **If not:** Check formula URL points to valid release artifact

## Verification

# Formula file exists with correct structure
test -f homebrew/Formula/termlink.rb
grep -q 'class Termlink < Formula' homebrew/Formula/termlink.rb
grep -q 'aarch64' homebrew/Formula/termlink.rb
grep -q 'x86_64' homebrew/Formula/termlink.rb
# README exists
test -f homebrew/README.md
grep -q 'brew install' homebrew/README.md
# SHA update script exists and is executable
test -x scripts/update-homebrew-sha.sh

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

### 2026-03-22T21:09:22Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-03-22T21:09:22Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-23T07:48:24Z — status-update [task-update-agent]
- **Change:** owner: agent → human
