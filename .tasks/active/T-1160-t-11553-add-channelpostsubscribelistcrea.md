---
id: T-1160
name: "T-1155/3 Add channel.{post,subscribe,list,create} API — CLI + MCP + hub router"
description: >
  Public surface for T-1155 bus. CLI: termlink channel post/subscribe/list/create. MCP: termlink_channel_* tools. Hub router: channel.* methods. JSON wire format (MVP). Typed message: {channel, sender_id, type, payload, artifact?, seq}.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: [T-1155, bus, api]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:06Z
last_update: 2026-04-20T14:12:06Z
date_finished: null
---

# T-1160: T-1155/3 Add channel.{post,subscribe,list,create} API — CLI + MCP + hub router

## Context

API surface for the T-1155 bus: wires T-1158 (`termlink-bus` core) + T-1159 (identity) into the hub's JSON-RPC router, the CLI, and the MCP tool registry. Per R-033 (T-922), every new CLI verb must be MCP-reachable.

Depends on: T-1158 (bus library), T-1159 (signing for `sender_id` + signature on posts). R-033 + G-020 gate this task — keep scope to the 4 verbs.

## Acceptance Criteria

### Agent
- [ ] Hub router adds 4 RPC methods in `crates/termlink-hub/src/router.rs`:
  - `channel.create(name, retention)` → `{topic_id}`; idempotent on name
  - `channel.post(topic, msg_type, payload, artifact_ref?, sender_pubkey, signature)` → `{offset, ts}`; hub verifies signature against sender_pubkey over `topic||msg_type||payload||artifact_ref||ts` before append
  - `channel.subscribe(topic, cursor?, limit?)` → `{messages: [...], next_cursor}`; `cursor=null` starts from 0
  - `channel.list(prefix?)` → `{topics: [{name, message_count, last_offset, retention}]}`
- [ ] CLI subcommand `termlink channel` with `create`, `post` (stdin payload), `subscribe` (streaming, polls every 1s), `list` — arg parsing mirrors `termlink inbox` shape
- [ ] MCP tools registered in `crates/termlink-mcp`: `termlink_channel_create`, `termlink_channel_post`, `termlink_channel_subscribe`, `termlink_channel_list` — `termlink doctor` reports updated tool count
- [ ] Protocol version bumped in `crates/termlink-protocol/src/lib.rs::PROTOCOL_VERSION` and the new methods are tagged Tier-A (opaque) per T-1133 taxonomy
- [ ] Router tests in `crates/termlink-hub/tests/`: post+subscribe roundtrip, signature-verify failure rejects post with typed error, cursor advances correctly on repeated subscribe, list returns created topics
- [ ] CLI integration tests: `termlink channel create` then `post` then `subscribe` on a local hub; all exit 0; stdout JSON parses
- [ ] Backward compat: `event.broadcast` / `inbox.*` / `file.*` continue to work unchanged (migration is T-1162..T-1164's job)
- [ ] `cargo build` + `cargo test` + `cargo clippy -- -D warnings` pass workspace-wide
- [ ] Design doc updated: `docs/reports/T-1155-agent-communication-bus.md` gets a new "Build log — T-1160" section with protocol wire format frozen

### Human
- [ ] [REVIEW] Validate the 4-verb surface is complete
  **Steps:**
  1. Read the method signatures in router.rs
  2. Compare against T-1155 §"Subsumption mapping" table — does this cover `event.broadcast`, `inbox.*`, `file.*` migration needs?
  3. If a migration target is missing a verb (e.g., `channel.delete`, `channel.ack`), flag it now rather than post-migration
  **Expected:** Approval or one follow-up task for missing verbs
  **If not:** List missing verbs

## Verification

cargo build
cargo test -p termlink-hub router::tests::channel
cargo test -p termlink-cli channel
cargo clippy -- -D warnings
grep -q "channel.post" crates/termlink-hub/src/router.rs
grep -q "termlink_channel_post" crates/termlink-mcp/src/lib.rs

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
