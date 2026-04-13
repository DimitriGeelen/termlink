---
id: T-1013
name: "Create termlink deploy script for remote hosts"
description: >
  Create termlink deploy script for remote hosts

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-13T10:20:14Z
last_update: 2026-04-13T10:20:14Z
date_finished: null
---

# T-1013: Create termlink deploy script for remote hosts

## Context

Create a reusable deploy script for deploying termlink binary + hub to remote Debian/Linux hosts via SSH. Automates: binary copy, systemd service setup, secret generation, profile creation, TOFU init.

## Acceptance Criteria

### Agent
- [x] Script at scripts/deploy-remote.sh
- [x] Accepts HOST, PROFILE_NAME, and optional PORT (default 9100) args
- [x] Copies the local termlink binary via scp
- [x] Creates systemd service on remote (matching .109 pattern)
- [x] Generates hub secret on remote
- [x] Copies secret back and creates local profile in hubs.toml
- [x] Runs termlink remote ping to verify connectivity
- [x] Script is idempotent (can re-run to update binary)
- [x] Script passes shellcheck (1 intentional SC2087 info — client-side heredoc expansion is desired)

### Human
- [ ] [REVIEW] Run script against .121 after authorizing SSH key
  **Steps:**
  1. Authorize SSH key on .121: `ssh-copy-id -i ~/.ssh/id_ed25519.pub root@192.168.10.121`
  2. `cd /opt/termlink && bash scripts/deploy-remote.sh 192.168.10.121 ring20-dev`
  **Expected:** Hub starts on .121:9100, profile created, ping succeeds
  **If not:** Check SSH connectivity and systemd logs on .121

## Verification

test -f scripts/deploy-remote.sh
bash -n scripts/deploy-remote.sh

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

### 2026-04-13T10:20:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1013-create-termlink-deploy-script-for-remote.md
- **Context:** Initial task creation
