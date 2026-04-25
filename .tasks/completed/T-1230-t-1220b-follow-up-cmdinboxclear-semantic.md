---
id: T-1230
name: "T-1220b/d follow-up: inbox.clear semantic split (Q4 spool deletion)"
description: >
  Migrate inbox.clear callers (4 sites) with the Q4 semantic split: legacy inbox.clear deletes spool files on hub disk; channel-backed clear advances local cursor only (no hub mutation). Two competing semantics. **Inception-style task: design first, build later.**

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-b-followup, inception]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/router.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-protocol/src/control.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: [T-1220, T-1225, T-1231, T-1226, T-1227, T-1228, T-1232]
created: 2026-04-25T08:24:33Z
last_update: 2026-04-25T10:44:27Z
date_finished: 2026-04-25T10:44:27Z
---

# T-1230: inbox.clear channel-aware migration (4 sites, design-first)

## Context

After T-1226/27/28/32 the `inbox.list` receiver migration is complete.
The parallel migration for `inbox.clear` raises a fundamental semantic
question per inception Q4:

- **Legacy `inbox.clear`**: removes spool files on the hub disk. Affects
  ALL subscribers (the data is gone).
- **Channel-backed clear**: each subscriber owns its own cursor.
  "Clearing" means advancing my cursor past the latest offset; the data
  remains on the hub for other subscribers and for retention to GC.

These are different operations. Conflating them under one verb risks
operator surprise.

**Affected call sites (4 sites, ~80 LOC):**
- `cmd_inbox_clear` — `crates/termlink-cli/src/commands/infrastructure.rs:802`
- `termlink_inbox_clear` MCP — `crates/termlink-mcp/src/tools.rs:4537`
- `cmd_remote_inbox_inner` Clear arm — `crates/termlink-cli/src/commands/remote.rs:1319`
- `termlink_remote_inbox_clear` MCP — `crates/termlink-mcp/src/tools.rs:~4754`

## Open Design Questions (Inception Phase)

1. **Verb split.** Three options:
   - **A.** Keep `inbox.clear` legacy (deletes spool); add new
     `channel.cursor.advance` for the per-subscriber semantic. Two verbs,
     clear distinction.
   - **B.** Add hub-side `channel.trim(topic, before_offset)` RPC that
     drops messages from the channel buffer (analogous to legacy delete).
     Single verb, but requires hub work + retention coordination.
   - **C.** Make `clear` advance local cursor only and warn-on-CLI when
     spool delete is desired. Confusing.
2. **Backwards compat.** Existing scripts call `inbox.clear` expecting
   spool deletion. Any change to semantics needs a deprecation cycle or
   a new verb name.
3. **T-1166 alignment.** If `inbox.*` retires, then `inbox.clear` retires
   too — only the channel verb remains. Need to confirm with T-1166's
   timeline whether the new semantic is sufficient.
4. **Multi-subscriber risk.** If two clients subscribe to
   `inbox:<target>` (e.g. CLI + Watchtower), and one calls "clear",
   should the other still see the messages? Channel semantics say yes
   (cursors are per-subscriber). Operator expectation may say no
   (someone explicitly cleared it).

## Acceptance Criteria

### Agent
- [x] Inception phase: `docs/reports/T-1230-inception.md` written with
      answers to Q1-Q4 above
- [x] Go/No-Go decision recorded: `fw inception decide T-1230 go|no-go`
- [x] If GO: create concrete build sub-tasks per decided wedge split
- [x] No source edits under this task ID beyond the inception artifact

## Verification

test -f docs/reports/T-1230-inception.md
grep -q "Go.*No-Go\|GO\|NO-GO" docs/reports/T-1230-inception.md

## Recommendation

**Recommendation:** GO

**Rationale:** Option A — two distinct verbs, no semantic conflation. Add `channel.trim(topic, before_offset?)` (destructive hub-side delete, mirrors legacy `inbox.clear` 1:1) and `channel.cursor.advance(topic, offset)` (per-subscriber cursor advance, new semantic for new use cases). Legacy migration uses `channel.trim` exclusively — operator behaviour preserved. Multi-subscriber risk resolved by verb choice: `trim` = everyone loses, `cursor.advance` = subscriber-local.

**Evidence:**
- Hub `inbox::clear_target` / `clear_all` (`crates/termlink-hub/src/inbox.rs:449-493`) does `std::fs::remove_dir_all(&target_dir)` — affects ALL subscribers. Channel-backed cursor-advance is a fundamentally different operation. The two cannot be conflated under one verb without operator surprise.
- T-1166 explicitly retires `inbox.clear` (`crates/termlink-hub/src/router.rs:75,756` + T-1166 §"Router methods removed"). Replacement is mandatory; keep-legacy is not viable.
- Option B (single `channel.trim`) is adequate for legacy migration but punts the multi-subscriber design question (Q4) — re-opens this inception as soon as the first new broadcast consumer needs subscriber-local "mark as read" semantics.
- Option C (single `clear` = cursor-advance only) is backwards-incompatible without a clear failure mode — a script that ran `inbox clear orphaned-target` to free disk would silently no-op, disk fills, G-009 fires.
- Full analysis + 7 build sub-task breakdown: `docs/reports/T-1230-inception.md`

## Go/No-Go Criteria

**GO if:**
- Verb split makes operator intent unambiguous (destructive vs subscriber-local)
- Legacy `inbox.clear` retirement under T-1166 has a 1:1 replacement (`channel.trim`)
- Sub-task scope is bounded (<400 LOC, breakable into ≤8 wedges)

**NO-GO if:**
- Hub spool storage and channel storage diverge such that `channel.trim` cannot reuse `inbox::clear_*`
- T-1166 retains `inbox.clear` indefinitely (no replacement needed)
- A simpler single-verb design preserves all four design properties

## Decisions

(Pending inception.)

## Updates

### 2026-04-25T08:24:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1230-t-1220b-follow-up-cmdinboxclear-semantic.md
- **Context:** Initial task creation

### 2026-04-25T08:55:00Z — convert-to-inception [agent]
- **Change:** Reclassified workflow_type build → inception
- **Change:** Captured 4 affected call sites and Q1-Q4 design questions
- **Reason:** Clear semantics differ fundamentally between spool-delete (legacy) and per-subscriber-cursor (channel). Needs explicit design + verb discussion before any source edit.

### 2026-04-25T09:28:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T10:09:26Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Option A — two distinct verbs, no semantic conflation. Add `channel.trim(topic, before_offset?)` (destructive hub-side delete, mirrors legacy `inbox.clear` 1:1) and `channel.cursor.advance(topic, offset)` (per-subscriber cursor advance, new semantic for new use cases). Legacy migration uses `channel.trim` exclusively — operator behaviour preserved. Multi-subscriber risk resolved by verb choice: `trim` = everyone loses, `cursor.advance` = subscriber-local.

### 2026-04-25T10:44:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
