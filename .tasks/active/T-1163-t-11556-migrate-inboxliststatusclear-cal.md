---
id: T-1163
name: "T-1155/6 Migrate inbox.{list,status,clear} callers → channel.{post,subscribe}"
description: >
  18 call sites across infrastructure.rs, remote.rs, tools.rs, router.rs. inbox.target becomes recipient channel; inbox.list becomes subscribe-since-cursor; inbox.clear becomes cursor advance. See T-1155 S-5.

status: captured
workflow_type: refactor
owner: agent
horizon: next
tags: [T-1155, bus, migration]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:13Z
last_update: 2026-04-24T09:00:19Z
date_finished: null
---

# T-1163: T-1155/6 Migrate inbox.{list,status,clear} callers → channel.{post,subscribe}

## Context

Second migration in the T-1155 bus rollout: `inbox.{list, status, clear}` move to `channel.{subscribe, list}` semantics on a per-recipient topic. Largest migration surface (~18 call sites across 4 files per T-1155 §"Subsumption mapping"). Follows T-1162 (broadcast).

Depends on: T-1162 done (proves migration pattern). Legacy `inbox.*` stays working until T-1166 retires it.

**Call sites (audited 2026-04-24, 82 raw matches across 6 files):**

*Hub-side handlers (where dual-write shim goes — follow T-1162 pattern):*
- `crates/termlink-hub/src/router.rs` dispatcher `@73-75`
- `handle_inbox_list @1425`, `handle_inbox_status @1448`, `handle_inbox_clear @1467`

*CLI producers (local hub):*
- `crates/termlink-cli/src/commands/infrastructure.rs::cmd_inbox_status @766`, `cmd_inbox_clear @802`, `cmd_inbox_list @839`
- `crates/termlink-cli/src/main.rs @332-334` (CLI dispatch only)

*CLI producers (remote hub via rpc_client):*
- `crates/termlink-cli/src/commands/remote.rs @1255/1288/1328` (remote inbox verbs)
- `@2810` (fleet doctor inbox check)

*MCP producers (local + remote):*
- `crates/termlink-mcp/src/tools.rs::termlink_inbox_{status,clear,list} @4518/4537/4564`
- `termlink_remote_inbox_{status,list,clear} @4684/4719/4754`

*Tests (verification surface):*
- `crates/termlink-hub/src/router.rs` tests `@3047/3063/3080/3111/3123/3234/3254/3264/3285/3295` (10)
- `crates/termlink-cli/tests/cli_integration.rs` (6)

**Summary:** 12 real entry points (3 hub handlers + 3 CLI local + 3 CLI remote + 3 MCP local + 3 MCP remote) + 1 dispatcher + 16 tests. Matches original T-1155 §Subsumption estimate of ~18 producer/handler sites.

**Migration pattern (from T-1162):** T-1162 was scope-reduced to "hub-side dual-write shim only — zero producer/receiver churn; pure additive". Recommend applying same discipline here: split this task into (a) hub shim in router.rs handlers (mirror into `channel:inbox:<target>`), (b) receiver rewrite follow-up (CLI/MCP switch to channel.subscribe), (c) capabilities handshake. ACs below currently bundle all three — suggest scope-reduction at task-start time.

## Acceptance Criteria

### Agent
- [x] Audit all current callers of `inbox.list`, `inbox.status`, `inbox.clear` — captured above under "Call sites (audited 2026-04-24)". 82 raw matches, 12 real entry points + 1 dispatcher + 16 tests.
- [ ] Topic naming convention: per-recipient topic `inbox:<session-id>` — auto-created when first message is posted to it
- [ ] Each `inbox.list(target)` caller rewritten to `channel.subscribe(topic="inbox:"+target, cursor=last_cursor_for_this_caller)` — read cursor stored client-side
- [ ] `inbox.status(target)` → `channel.list(prefix="inbox:"+target)` + count since cursor
- [ ] `inbox.clear(target)` → advance cursor to `latest_offset` (do NOT delete from log — retention policy handles eviction)
- [ ] Legacy `inbox.*` router methods remain operational as shims forwarding to channel semantics; `#[deprecated(note = "migrate to channel.{subscribe,list} with topic=inbox:<target> (T-1163)")]`
- [ ] Remote inbox commands (T-1009, T-1010, T-1020) — `termlink remote inbox` — also dual-mode: prefer `channel.subscribe`, fall back to `inbox.list` if channel API absent (capabilities check)
- [ ] Integration test: post via legacy `inbox` semantics and new `channel.post`, read back via both APIs — content identical
- [ ] `cargo build && cargo test && cargo clippy -- -D warnings` pass workspace-wide
- [ ] No user-visible behavioral change: `termlink inbox list`, `termlink remote inbox list` produce the same output as before

### Human
- [x] [REVIEW] Confirm per-recipient topic naming (`inbox:<session-id>`) — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — per-recipient topic naming `inbox:<session-id>` confirmed.
  **Steps:**
  1. Consider whether session-id (ephemeral) vs peer-pubkey-fingerprint (stable across restarts) is the right recipient identifier
  2. Ephemeral session-id loses messages across restart; stable identity does not
  3. Decide: cut over to stable-identity-as-recipient now, or defer to post-migration task?
  **Expected:** Decision recorded
  **If not:** Open a follow-up task for recipient identity migration

## Verification

cargo build
cargo test -p termlink-hub inbox
cargo test -p termlink-cli inbox
cargo clippy -- -D warnings
grep -rn "inbox\.\(list\|status\|clear\)" crates/ | tee /tmp/T-1163-callsites.txt
grep -q "inbox:" crates/termlink-hub/src/router.rs

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

### 2026-04-20T14:12:13Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1163-t-11556-migrate-inboxliststatusclear-cal.md
- **Context:** Initial task creation

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next
