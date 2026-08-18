---
id: T-168
name: "File transfer via base64 chunked events"
description: >
  Implement file.send.init/chunk/complete event protocol for transferring files between
  machines via base64 chunked events. CLI commands: termlink file send/receive.

status: work-completed
workflow_type: build
owner: human
horizon:
tags: [file-transfer, agent-comms]
components: []
related_tasks: []
created: 2026-03-18T10:08:38Z
last_update: '2026-08-18T18:58:54Z'
date_finished: 2026-03-18T18:31:32Z
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:08Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 0
      D2: 0
      D3: 0
      D4: 4
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=0 (no-signal); D2=0 (no-signal); D3=0 (no-signal); D4=4 
      (body:cross-machine); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:54Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-168: File transfer via base64 chunked events

## Context

Transfers files between sessions via base64-encoded chunked events. Uses the event bus so it works over TCP hub (cross-machine). Protocol: `file.init` → N × `file.chunk` → `file.complete`. SHA-256 integrity check on completion.

## Acceptance Criteria

### Agent
- [x] Event schemas added to `termlink-protocol/src/events.rs`: `FileInit`, `FileChunk`, `FileComplete`, `FileError` with `file_topic` constants
- [x] Schema roundtrip tests for all file transfer event types
- [x] `termlink file send <target> <path>` reads file, chunks to base64, emits events to target session
- [x] `termlink file receive <target> [--output-dir <dir>] [--timeout <secs>]` polls for file events, reassembles, writes to disk
- [x] SHA-256 checksum verified on receive, error printed on mismatch
- [x] Integration test: send file between two sessions, verify content matches
- [x] All existing tests pass (`cargo test --workspace`)

### Human
- [x] [REVIEW] Transfer a binary file between two local sessions and verify integrity
  **Steps:**
  1. Register two sessions: `termlink register --name sender --shell` and `termlink register --name receiver --shell`
  2. Create a test file: `dd if=/dev/urandom of=/tmp/test-transfer.bin bs=1024 count=100`
  3. In one terminal: `termlink file receive receiver --output-dir /tmp/received --timeout 30`
  4. In another: `termlink file send receiver /tmp/test-transfer.bin`
  5. Compare: `shasum -a 256 /tmp/test-transfer.bin /tmp/received/test-transfer.bin`
  **Expected:** SHA-256 checksums match
  **If not:** Check for chunk ordering or base64 decode errors in output

## Verification

bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink-protocol 2>&1); echo "$out" | grep -q "0 failed"'
bash -c 'out=$(/Users/dimidev32/.cargo/bin/cargo test --package termlink 2>&1); echo "$out" | grep -q "0 failed"'
grep -q "FileInit" crates/termlink-protocol/src/events.rs
grep -q "cmd_file_send" crates/termlink-cli/src/main.rs
grep -q "cmd_file_receive" crates/termlink-cli/src/main.rs

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

### 2026-03-18T10:08:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /Users/dimidev32/001-projects/010-termlink/.tasks/active/T-168-file-transfer-via-base64-chunked-events.md
- **Context:** Initial task creation

### 2026-03-18T18:09:38Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T18:31:32Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
