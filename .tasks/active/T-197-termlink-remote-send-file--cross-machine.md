---
id: T-197
name: "termlink remote send-file — cross-machine file transfer via TCP hub"
description: >
  Add `termlink remote send-file` command that transfers files to sessions on
  remote hubs via TOFU TLS + auth. Reuses existing FileInit/FileChunk/FileComplete
  event protocol (T-039) with remote hub routing (like remote inject).

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [cli, cross-machine, file-transfer]
components: []
related_tasks: [T-163, T-186, T-187]
created: 2026-03-20T22:24:41Z
last_update: 2026-03-20T22:31:49Z
date_finished: 2026-03-20T22:31:49Z
---

# T-197: termlink remote send-file — cross-machine file transfer via TCP hub

## Context

Local `termlink file send/receive` exists (T-039) using FileInit/FileChunk/FileComplete
events with SHA-256 verification. But it uses `manager::find_session()` (local filesystem
only). Need a `remote send-file` that routes through the TCP hub like `remote inject` does.
Discovered during T-196: 13KB pickup prompt too large for PTY inject, needed file transfer.

## Acceptance Criteria

### Agent
- [x] `RemoteAction::SendFile` variant added to CLI enum with args: hub, session, path, secret-file/secret, chunk-size, scope, json
- [x] `cmd_remote_send_file()` implements: read file → TOFU connect → auth → emit FileInit/FileChunk/FileComplete via hub
- [x] Events use `target` param for hub routing (same pattern as remote inject)
- [x] SHA-256 verification included in FileComplete
- [x] Error handling for: file not found, auth failure, connection refused, session not found
- [x] `cargo build --package termlink` compiles clean
- [x] `cargo test --package termlink` passes (0 failed, 4 ignored integration tests)
- [x] Help text: `termlink remote send-file --help` shows usage

### Human
- [ ] [REVIEW] Test cross-machine: send file from macOS to .107 session
  **Steps:**
  1. Ensure hub running on .107 with `--tcp`
  2. Start a receiver: `termlink file receive fw-agent --output-dir /tmp`
  3. From macOS: `termlink remote send-file 192.168.10.107:9100 fw-agent ./test-file.txt --secret-file /tmp/termlink-107-secret.txt`
  4. Check /tmp on .107 for received file
  **Expected:** File appears in /tmp with matching content and SHA-256
  **If not:** Check hub logs, verify auth, check event routing

## Verification

/Users/dimidev32/.cargo/bin/cargo build --package termlink
grep -q "SendFile" crates/termlink-cli/src/main.rs
grep -q "cmd_remote_send_file" crates/termlink-cli/src/main.rs

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

### 2026-03-20T22:24:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-197-termlink-remote-send-file--cross-machine.md
- **Context:** Initial task creation

### 2026-03-20T22:31:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
