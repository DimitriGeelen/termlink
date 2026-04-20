---
id: T-1163
name: "T-1155/6 Migrate inbox.{list,status,clear} callers → channel.{post,subscribe}"
description: >
  18 call sites across infrastructure.rs, remote.rs, tools.rs, router.rs. inbox.target becomes recipient channel; inbox.list becomes subscribe-since-cursor; inbox.clear becomes cursor advance. See T-1155 S-5.

status: captured
workflow_type: refactor
owner: agent
horizon: later
tags: [T-1155, bus, migration]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:13Z
last_update: 2026-04-20T14:12:13Z
date_finished: null
---

# T-1163: T-1155/6 Migrate inbox.{list,status,clear} callers → channel.{post,subscribe}

## Context

Second migration in the T-1155 bus rollout: `inbox.{list, status, clear}` move to `channel.{subscribe, list}` semantics on a per-recipient topic. Largest migration surface (~18 call sites across 4 files per T-1155 §"Subsumption mapping"). Follows T-1162 (broadcast).

Depends on: T-1162 done (proves migration pattern). Legacy `inbox.*` stays working until T-1166 retires it.

## Acceptance Criteria

### Agent
- [ ] Audit all current callers of `inbox.list`, `inbox.status`, `inbox.clear` — `grep -rn "inbox\.\(list\|status\|clear\)\|inbox_list\|inbox_status\|inbox_clear" crates/ lib/` produces the exhaustive list; capture in this task file under "Call sites"
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
- [ ] [REVIEW] Confirm per-recipient topic naming (`inbox:<session-id>`)
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
