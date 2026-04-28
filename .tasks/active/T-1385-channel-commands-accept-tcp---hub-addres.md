---
id: T-1385
name: "channel commands accept TCP --hub addresses for cross-hub RPC"
description: >
  channel commands accept TCP --hub addresses for cross-hub RPC

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-28T18:21:09Z
last_update: 2026-04-28T18:21:09Z
date_finished: null
---

# T-1385: channel commands accept TCP --hub addresses for cross-hub RPC

## Context

T-1384 inception discovered: `channel.*` commands' `--hub` flag accepts only Unix socket paths because `hub_socket()` in `crates/termlink-cli/src/commands/channel.rs:52` hardcodes `PathBuf` and `client::rpc_call(&Path, ...)`. The runtime layer (`Client::connect_addr` in `crates/termlink-session/src/client.rs:19`) already supports both Unix and TCP via `TransportAddr`. Fix is purely client-side: parse `host:port`, return `TransportAddr`, swap `rpc_call` → `rpc_call_addr`. Unblocks cross-hub multi-agent conversations.

## Acceptance Criteria

### Agent
- [x] `parse_hub_addr()` helper added — returns `TransportAddr::Tcp` for `host:port` (no `/`, trailing u16), else `TransportAddr::Unix`
- [x] `hub_socket()` and `hub_socket_soft()` return `TransportAddr` instead of `PathBuf`
- [x] All 4 helper signatures using `sock: &Path` updated to `sock: &TransportAddr` (ensure_topic, compute_dm_inbox_row, resolve_latest_offset, walk_topic_full)
- [x] All 20 `client::rpc_call(` call sites in channel.rs migrated (now go through new `rpc_call_authed` helper that pre-auths on TCP, delegates to `client::rpc_call_addr` on Unix)
- [x] `BusClient` (termlink-session) migrated to `TransportAddr` + `rpc_call_addr` for TCP-capable wire format; `cmd_channel_post` bypasses BusClient on TCP and uses authed direct RPC (offline queue stays Unix-only this iteration)
- [x] Workspace builds clean: `cargo check --release -p termlink`
- [x] Existing channel.rs unit tests pass: 306 tests in `channel::` module
- [x] Unit tests added: `parse_hub_addr_recognizes_tcp_host_port`, `parse_hub_addr_recognizes_localhost_tcp`, `parse_hub_addr_falls_back_to_unix_path`, `parse_hub_addr_path_with_colon_treated_as_unix`, `parse_hub_addr_invalid_port_falls_back_to_unix`
- [x] Cross-hub e2e: `termlink channel create xhub-real-... --hub 192.168.10.122:9100` succeeds and appears via `channel list --hub 192.168.10.122:9100` (NOT on local)
- [x] Cross-hub `channel post --hub 192.168.10.122:9100` succeeds with auth (signature verified by remote hub)
- [x] Multi-agent e2e (`tests/e2e/multi-agent-conversation.sh`) passes all 10 steps: 6 agents post to both hubs, byte-identical canonical state confirmed, local edit/redact does not leak to remote

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
cargo check --release -p termlink 2>&1 | tail -5
grep -q "parse_hub_addr" crates/termlink-cli/src/commands/channel.rs
grep -q "rpc_call_authed" crates/termlink-cli/src/commands/channel.rs
test -x tests/e2e/multi-agent-conversation.sh

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

### 2026-04-28T18:21:09Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1385-channel-commands-accept-tcp---hub-addres.md
- **Context:** Initial task creation
