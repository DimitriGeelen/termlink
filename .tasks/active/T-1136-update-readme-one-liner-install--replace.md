---
id: T-1136
name: "Update README one-liner install — replace 'cargo install --git' hint with install.sh curl-pipe (from T-1070 GO)"
description: >
  From T-1070 inception GO. README currently points consumers to 'brew install termlink' (macOS-centric) or 'cargo install --git' (requires toolchain — the failure mode for LXCs). After T-1070-install-sh lands, update README Install section to lead with the curl-pipe one-liner (cross-platform, no toolchain). Keep brew as the macOS preferred path, cargo as the 'from source' path, but de-emphasize. Small, text-only change that consolidates the install UX behind the bootstrap script.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [readme, docs, ux, T-1070]
components: []
related_tasks: []
created: 2026-04-18T23:02:47Z
last_update: 2026-04-19T13:56:06Z
date_finished: 2026-04-19T13:56:06Z
---

# T-1136: Update README one-liner install — replace 'cargo install --git' hint with install.sh curl-pipe (from T-1070 GO)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] README Quick Start lead block is the `curl | sh` one-liner pointing at the T-1134 installer
- [x] Homebrew path retained as a secondary option (macOS-preferred)
- [x] `cargo install --git` demoted to "from source" with a clear label
- [x] No references remain recommending `cargo install --git` as a default install

### Human
- [ ] [RUBBER-STAMP] README renders correctly on GitHub
  **Steps:** Open https://github.com/DimitriGeelen/termlink and read the Quick Start block
  **Expected:** `curl -fsSL ... install.sh | sh` appears first; brew second; cargo labeled "from source"
  **If not:** Copy the Quick Start block and note what looks wrong

## Verification

grep -q "curl -fsSL https://raw.githubusercontent.com/DimitriGeelen/termlink/main/install.sh" /opt/termlink/README.md
grep -q "from source" /opt/termlink/README.md
bash -c 'head -40 /opt/termlink/README.md | grep -n "install.sh\|brew install\|cargo install" | head -1 | grep -q "install.sh"'

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

### 2026-04-18T23:02:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1136-update-readme-one-liner-install--replace.md
- **Context:** Initial task creation

### 2026-04-19T13:55:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now (auto-sync)

### 2026-04-19T13:56:06Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
