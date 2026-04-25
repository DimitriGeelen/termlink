---
id: T-1229
name: "T-1220b/d follow-up: inbox.status migration via channel aggregation"
description: >
  Migrate inbox.status callers (cmd_inbox_status local CLI, termlink_inbox_status MCP, cmd_remote_inbox_status arm, termlink_remote_inbox_status MCP, fleet-doctor inbox check) to channel surface. Needs a new aggregation entry point in inbox_channel — channel.* is per-topic but status returns {total, targets[]}. **Inception-style task: design first, build later.**

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [T-1155, bus, migration, T-1220, wedge-b-followup, inception]
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-hub/src/channel.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: [T-1220, T-1225, T-1231, T-1226, T-1227, T-1228, T-1232]
created: 2026-04-25T08:24:26Z
last_update: 2026-04-25T10:44:24Z
date_finished: 2026-04-25T10:44:24Z
---

# T-1229: inbox.status channel-aware migration (5 sites, design-first)

## Context

After T-1226/27/28/32 the `inbox.list` receiver migration is complete. The
parallel migration for `inbox.status` is *not* a drop-in: status returns
an aggregate `{total_transfers, targets: [{target, pending}]}` while the
channel surface is per-topic (`channel.subscribe(topic="inbox:<target>")`).

**Affected call sites (5 sites, ~150 LOC):**
- `cmd_inbox_status` — `crates/termlink-cli/src/commands/infrastructure.rs:766`
- `termlink_inbox_status` MCP — `crates/termlink-mcp/src/tools.rs:4518`
- `cmd_remote_inbox_inner` Status arm — `crates/termlink-cli/src/commands/remote.rs:1253`
- `termlink_remote_inbox_status` MCP — `crates/termlink-mcp/src/tools.rs:~4684`
- Fleet-doctor inbox check — `crates/termlink-cli/src/commands/remote.rs:2810`

## Open Design Questions (Inception Phase)

1. **Aggregation source.** Three options:
   - **A.** Add hub-side `channel.list_topics(prefix="inbox:")` RPC that
     returns counts per topic. Single round-trip, but adds new server
     surface (T-1166 retirement scope grows).
   - **B.** Have client iterate: enumerate known sessions via
     `session.discover`, then `channel.subscribe(topic="inbox:<each>",
     limit=0)` for counts only. N+1 round-trips per call.
   - **C.** Keep `inbox.status` legacy forever — it's a read-only
     aggregation that hub can compute cheaply from spool. Migrating it
     buys nothing operationally.
2. **Fleet-doctor sensitivity.** The check at remote.rs:2810 emits
   `warn` on any pending transfer. If migrated to per-topic enumeration,
   it might miss transfers for targets not in `session.discover` results
   (offline sessions). Risk: regression vs current behavior.
3. **Migration urgency.** T-1166 retires `inbox.*` entirely. If
   `inbox.status` retires *with* `inbox.list`, then Option C is wrong.
   Need to confirm T-1166 scope.

## Acceptance Criteria

### Agent
- [x] Inception phase: `docs/reports/T-1229-inception.md` written with
      answers to Q1-Q3 above, particularly Option A vs B vs C decision
- [x] Go/No-Go decision recorded: `fw inception decide T-1229 go|no-go`
- [x] If GO: create concrete build sub-tasks per decided wedge split
- [x] No source edits under this task ID beyond the inception artifact

## Verification

test -f docs/reports/T-1229-inception.md
grep -q "Go.*No-Go\|GO\|NO-GO" docs/reports/T-1229-inception.md

## Recommendation

**Recommendation:** GO

**Rationale:** Option A — add hub-side `channel.list_topics(prefix="inbox:")` RPC. Single round-trip aggregation that mirrors the existing `inbox::list_all_targets()` spool walk on the channel surface. Preserves fleet-doctor correctness invariant (sees all pending transfers regardless of subscriber online status). Aligns with T-1166 retirement scope: when legacy `inbox.status` is removed, `channel.list_topics` stays.

**Evidence:**
- Hub `inbox::list_all_targets()` walks the spool directory directly (`crates/termlink-hub/src/inbox.rs`) — this is the data source `inbox.status` exposes today; Option A re-uses it via the channel surface.
- T-1166 explicitly retires `inbox.status` (`crates/termlink-hub/src/router.rs:74,755` + T-1166 §"Router methods removed"). Option C (keep legacy) is not viable for the long horizon.
- Option B (client iteration via `session.discover` + `channel.subscribe(limit=0)`) regresses on offline-target visibility — `session.discover` returns active sessions only, but inbox spool holds transfers for any target name including offline / never-registered ones. Fleet-doctor would silently miss the exact failure mode it exists to surface.
- Full analysis + 7 build sub-task breakdown: `docs/reports/T-1229-inception.md`

## Go/No-Go Criteria

**GO if:**
- Aggregation source preserves fleet-doctor correctness invariant (no offline-target blind spot)
- Migration path aligns with T-1166 `inbox.*` retirement scope
- Implementation cost is bounded (<400 LOC, breakable into ≤8 wedges)

**NO-GO if:**
- Hub-side aggregation would require unbounded server complexity
- T-1166 timeline shifts to keep `inbox.status` indefinitely
- A simpler client-side path emerges that doesn't lose offline-target transfers

## Decisions

(Pending inception.)

## Updates

### 2026-04-25T08:24:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1229-t-1220b-follow-up-cmdinboxstatus-migrati.md
- **Context:** Initial task creation

### 2026-04-25T08:55:00Z — convert-to-inception [agent]
- **Change:** Reclassified workflow_type build → inception
- **Change:** Captured 5 affected call sites and Q1-Q3 design questions
- **Reason:** Status aggregation does not map cleanly to per-topic channel surface; needs design discussion before any source edit. Three options (server-side aggregation, client-side enumeration, keep legacy) have different trade-offs.

### 2026-04-25T09:27:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-25T10:08:58Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** Option A — add hub-side `channel.list_topics(prefix="inbox:")` RPC. Single round-trip aggregation that mirrors the existing `inbox::list_all_targets()` spool walk on the channel surface. Preserves fleet-doctor correctness invariant (sees all pending transfers regardless of subscriber online status). Aligns with T-1166 retirement scope: when legacy `inbox.status` is removed, `channel.list_topics` stays.

### 2026-04-25T10:44:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
