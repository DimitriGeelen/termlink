---
id: T-1220
name: "T-1163 follow-up: CLI/MCP inbox receiver migration to channel.{subscribe,list}"
description: >
  Receiver-side migration following T-1163's hub dual-write shim. CLI verbs 'inbox {list,status,clear}' + MCP tools termlink_inbox_* + remote inbox verbs switch to channel.{subscribe,list} on topic 'inbox:<target>' with capabilities fallback to legacy inbox.* when peer lacks channel API.

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1163-followup]
components: []
related_tasks: []
created: 2026-04-24T15:10:01Z
last_update: 2026-04-24T15:36:42Z
date_finished: null
---

# T-1220: T-1163 follow-up: CLI/MCP inbox receiver migration to channel.{subscribe,list}

## Context

Receiver-side migration following T-1163's hub dual-write shim
(`channel::mirror_inbox_deposit` lands every `inbox::deposit` into the
per-target `inbox:<target>` channel topic). Legacy `inbox.list / inbox.status
/ inbox.clear` still work via the existing router handlers — this task
rewrites consumers to read via the channel surface instead, with a
capabilities-gated fallback to legacy for peers that predate `channel.*`.

**Call sites to migrate** (from T-1163 audit):
- CLI local: `crates/termlink-cli/src/commands/infrastructure.rs` — `cmd_inbox_status @766`, `cmd_inbox_clear @802`, `cmd_inbox_list @839`
- CLI remote: `crates/termlink-cli/src/commands/remote.rs` — `@1255/1288/1328` + fleet doctor `@2810`
- MCP local: `crates/termlink-mcp/src/tools.rs` — `termlink_inbox_{status,clear,list} @4518/4537/4564`
- MCP remote: `crates/termlink-mcp/src/tools.rs` — `termlink_remote_inbox_{status,list,clear} @4684/4719/4754`

**Open design questions (blocks build — inception first):**
1. **Cursor persistence.** Per-caller read cursor needs storage — options: `~/.termlink/cursors/<caller>-<target>.seq`, SQLite table, process-memory-only (loses cursor across invocations). Which survives a CLI restart? Does remote exec share cursor state with local?
2. **Capabilities probe timing.** T-1215 shipped `HubCapabilitiesCache`. Do we probe per-invocation (cheap, always-fresh) or per-session-per-target (cheaper, risk of stale cache after hub upgrade)?
3. **Fallback semantics.** Peer's hub returns `method-not-found` on `channel.subscribe` — do we fall back silently, warn once, or flag the peer as legacy-only in cache?
4. **`inbox.clear` semantics.** Task says "advance cursor to latest_offset". Does this delete nothing on the hub (retention handles GC) or actively trigger retention sweep? Current `inbox.clear` removes spool files on disk — the channel-backed version cannot (subscribers' cursors are independent).
5. **Mixed-mode rollout.** During transition, some deposits are only on legacy (pre-T-1163 hubs) and some are on both. A channel-only reader misses legacy-only deposits. Is this acceptable until T-1166 retires legacy, or do we need a merging layer?

**Decision gate before proceeding:** this task should be taken through inception discipline — produce `docs/reports/T-1220-inception.md`, answer the 5 questions above, THEN create concrete build sub-tasks.

## Acceptance Criteria

### Agent
- [ ] Inception phase: `docs/reports/T-1220-inception.md` written with answers to the 5 open design questions above
- [ ] Go/No-Go decision recorded: `fw inception decide T-1220 go|no-go --rationale "..."`
- [ ] If GO: create concrete build sub-tasks per decided wedge split (likely: local CLI inbox verbs → MCP local → remote CLI → remote MCP, with a dedicated capabilities-fallback helper task in termlink-session)
- [ ] No source edits under this task ID beyond the inception artifact — implementation lands under the build sub-tasks

## Verification

test -f docs/reports/T-1220-inception.md
grep -q "Go.*No-Go" docs/reports/T-1220-inception.md

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

### 2026-04-24T15:10:01Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1220-t-1163-follow-up-climcp-inbox-receiver-m.md
- **Context:** Initial task creation

### 2026-04-24T15:36:42Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
