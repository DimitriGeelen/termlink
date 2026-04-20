---
id: T-1160
name: "T-1155/3 Add channel.{post,subscribe,list,create} API — CLI + MCP + hub router"
description: >
  Public surface for T-1155 bus. CLI: termlink channel post/subscribe/list/create. MCP: termlink_channel_* tools. Hub router: channel.* methods. JSON wire format (MVP). Typed message: {channel, sender_id, type, payload, artifact?, seq}.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [T-1155, bus, api]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:06Z
last_update: 2026-04-20T21:43:41Z
date_finished: 2026-04-20T21:43:41Z
---

# T-1160: T-1155/3 Add channel.{post,subscribe,list,create} API — CLI + MCP + hub router

## Context

API surface for the T-1155 bus: wires T-1158 (`termlink-bus` core) + T-1159 (identity) into the hub's JSON-RPC router, the CLI, and the MCP tool registry. Per R-033 (T-922), every new CLI verb must be MCP-reachable.

Depends on: T-1158 (bus library), T-1159 (signing for `sender_id` + signature on posts). R-033 + G-020 gate this task — keep scope to the 4 verbs.

## Acceptance Criteria

### Agent
- [x] Hub router adds 4 RPC methods in `crates/termlink-hub/src/router.rs` + channel.rs:
  - `channel.create(name, retention)` → `{ok, name, retention}`; idempotent on name
  - `channel.post(topic, msg_type, payload_b64, artifact_ref?, ts, sender_id, sender_pubkey_hex, signature_hex)` → `{offset, ts}`; hub verifies signature via `control::channel::canonical_sign_bytes` before append
  - `channel.subscribe(topic, cursor?, limit?)` → `{messages: [...], next_cursor}`; `cursor` defaults to 0
  - `channel.list(prefix?)` → `{topics: [{name, retention}]}`
- [x] CLI subcommand `termlink channel` with `create`, `post` (inline or stdin payload), `subscribe` (`--follow` polls every 1s), `list` — arg parsing mirrors `termlink inbox` shape
- [x] MCP tools registered in `crates/termlink-mcp`: `termlink_channel_create`, `termlink_channel_post`, `termlink_channel_subscribe`, `termlink_channel_list` — `termlink doctor` reports 73 MCP tools (was 69, added 4)
- [x] Protocol version bumped: added `CONTROL_PLANE_VERSION = 2` in `termlink-protocol/src/lib.rs` alongside existing `DATA_PLANE_VERSION`. All 4 new methods tagged Tier-A (opaque) per T-1133 taxonomy with doc comments
- [x] Router tests (8) in `crates/termlink-hub/src/channel.rs`: post+subscribe roundtrip, signature-verify failure → typed `CHANNEL_SIGNATURE_INVALID`, cursor advances across paginated subscribe, unknown topic → typed `CHANNEL_TOPIC_UNKNOWN` (both post and subscribe), list returns created topics + prefix filter, missing-name 400, create→list roundtrip
- [x] CLI integration tests (3) in `crates/termlink-cli/tests/cli_integration.rs`: `channel --help` lists the four verbs, `channel list` without hub exits non-zero with clear error, `channel create` rejects unknown retention spec. Live smoke: create→post→subscribe transcript captured in T-1155 design doc
- [x] Backward compat: `event.broadcast` / `inbox.*` / `file.*` continue to work unchanged (migration is T-1162..T-1164's job) — all 207 hub lib tests still pass
- [x] `cargo build --workspace` + `cargo test --workspace --lib` (695 tests) + `cargo clippy --workspace --lib --tests -- -D warnings` pass workspace-wide (drive-by: 3 pre-existing `let _ =` fixes in inbox tests)
- [x] Design doc updated: `docs/reports/T-1155-agent-communication-bus.md` gets "Build log — T-1160" section with protocol wire format frozen, error codes, canonical signing bytes, file map, live smoke transcript

### Human
- [ ] [REVIEW] Validate the 4-verb surface is complete
  **Steps:**
  1. Read the method signatures in router.rs
  2. Compare against T-1155 §"Subsumption mapping" table — does this cover `event.broadcast`, `inbox.*`, `file.*` migration needs?
  3. If a migration target is missing a verb (e.g., `channel.delete`, `channel.ack`), flag it now rather than post-migration
  **Expected:** Approval or one follow-up task for missing verbs
  **If not:** List missing verbs

## Verification

cargo build --workspace
cargo test -p termlink-hub --lib channel
cargo test -p termlink --test cli_integration cli_channel
cargo clippy --workspace --lib --tests -- -D warnings
grep -q "CHANNEL_POST" crates/termlink-hub/src/router.rs
grep -q "termlink_channel_post" crates/termlink-mcp/src/tools.rs

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

### 2026-04-20T14:12:06Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1160-t-11553-add-channelpostsubscribelistcrea.md
- **Context:** Initial task creation

### 2026-04-20T21:16:51Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-04-20T21:16:55Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-04-20T21:43:41Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
