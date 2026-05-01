---
id: T-1441
name: "remote-list shows identity_fingerprint"
description: >
  remote-list shows identity_fingerprint

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-hub/src/router.rs]
related_tasks: []
created: 2026-05-01T20:07:38Z
last_update: 2026-05-01T20:31:34Z
date_finished: 2026-05-01T20:31:34Z
---

# T-1441: remote-list shows identity_fingerprint

## Context

The chat-arc cross-host contact pattern (T-1429 / T-1431 `/agent-handoff`) requires a
`--target-fp <hex>` argument to address peers across hubs. Today
`termlink remote list <hub>` shows session ID, name, state, PID, tags — but
NOT the identity_fingerprint that a sender would copy-paste into `--target-fp`.
Operators end up doing a 3-step dance: list sessions, ssh to that host, run
`termlink whoami` there to extract the fp, paste back. T-1436 already
plumbed `identity_fingerprint` into `SessionMetadata`; T-1440 surfaces it in
local `whoami`. This task surfaces it on the *remote* side: hub-side
`session.discover` includes `identity_fingerprint`, CLI `remote list` adds
an FP column. Single-step lookup → cuts onboarding friction for vendored
field agents using the chat arc.

## Acceptance Criteria

### Agent
- [x] Hub `handle_discover` (router.rs:217) includes `identity_fingerprint` in each session JSON entry — sourced from `s.metadata.identity_fingerprint`. Field omitted (not null) when absent for pre-T-1436 sessions to keep JSON shape minimal
- [x] CLI `cmd_remote_list_inner` (remote.rs:586-615) adds an `FP` column to text output between NAME and STATE; renders the first 16 hex chars (the canonical fingerprint width) or `-` when absent
- [x] `--json` output unchanged in shape — clients who deserialize the response see the new optional field naturally (it's just an extra key on each session entry)
- [x] Header alignment preserved — header reads `ID NAME FP STATE PID TAGS` with FP column 17 chars wide; row separator widened from 64 to 80 dashes
- [x] Live verification on local hub: `termlink remote list local-test` shows the `FP` column header and per-row values; pre-T-1681 hub returns no fp field so all rows show `-` until the .107 hub gets upgraded (header itself proves the CLI side)
- [x] Workspace builds clean: cargo build for cli + hub exits 0; full hub suite passes 297/297 incl. new test
- [x] New unit test `discover_includes_identity_fingerprint_when_present` verifies hub-side JSON shape: registers a test session, calls handle_discover, asserts `sessions[0].identity_fingerprint == r.metadata.identity_fingerprint`. Test passes locally.

## Verification

cargo build --workspace --release 2>&1 | tail -3 | grep -qv "error"
./target/release/termlink remote list local-test 2>&1 | head -1 | grep -q "FP"

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

### 2026-05-01T20:07:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1441-remote-list-shows-identityfingerprint.md
- **Context:** Initial task creation

### 2026-05-01T20:31:34Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
