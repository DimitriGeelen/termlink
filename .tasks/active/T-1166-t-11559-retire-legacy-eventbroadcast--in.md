---
id: T-1166
name: "T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives"
description: >
  After N months of parallel operation + deprecation warnings (T-1155 S-5 phase 4). Remove hub router handlers for event.broadcast, inbox.*, file.* once all callers migrated. Protocol bump + version diversity check (T-1132) gates removal.

status: captured
workflow_type: decommission
owner: agent
horizon: later
tags: [T-1155, bus, deprecation]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:20Z
last_update: 2026-04-20T14:12:20Z
date_finished: null
---

# T-1166: T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives

## Context

Final migration phase per T-1155 §"Migration strategy Phase 4": retire the legacy primitives after N months of parallel operation. **Decommission workflow** — do NOT start until all three migrations (T-1162, T-1163, T-1164) have been in production for at least 60 days AND telemetry shows <1% legacy-API call volume.

This task is deliberately gated: it has entry criteria that block starting too early. Framework sovereignty (R-033) applies — final retirement is a Tier-2 authorized action.

## Acceptance Criteria

### Agent
- [ ] **Entry gate check:** `fw metrics api-usage --last-60d` shows `event.broadcast + inbox.* + file.*` ≤ 1% of total RPC volume. If >1%, stop and open a task to hunt down the remaining callers.
- [ ] Zero live callers in repo: `grep -rn "event\.broadcast\|event_broadcast\|inbox\.\(list\|status\|clear\)\|file\.send\|file\.receive" crates/ lib/ skills/` returns 0 hits (excluding deprecation shims themselves and test fixtures)
- [ ] Router methods removed from `crates/termlink-hub/src/router.rs`: `event.broadcast`, `inbox.list`, `inbox.status`, `inbox.clear`, `file.send`, `file.receive`, and their chunked variants
- [ ] CLI commands removed: `termlink inbox *`, `termlink file send`, `termlink file receive` — OR rewritten as thin wrappers over `termlink channel *` (keep the verb, change the impl). Choose per UX review.
- [ ] MCP tools updated: remove `termlink_inbox_list/status/clear`, `termlink_file_send/receive` OR rewrite as channel shims. `termlink doctor` tool count reflects the removal.
- [ ] Protocol version bumped; new major version per the `PROTOCOL_VERSION` enforcement from T-1131
- [ ] Migration guide published at `docs/migrations/T-1166-retire-legacy-primitives.md` — for downstream consumers (ring20, ntb-atc-plugin, skills-manager, etc.)
- [ ] Blast radius check (`fw fabric blast-radius HEAD`) shows no unregistered downstream surprises
- [ ] Full workspace build + tests pass: `cargo build && cargo test && cargo clippy -- -D warnings`
- [ ] Capability handshake update: hub advertises `legacy_primitives = false`; older clients fail fast with a clear error pointing at the migration doc

### Human
- [ ] [REVIEW] Approve retirement timing
  **Steps:**
  1. Run `fw metrics api-usage --last-60d` and verify ≤1% legacy traffic
  2. Scan `.context/project/concerns.yaml` for any open gap that depends on a legacy API
  3. Notify downstream consumer operators via their termlink sessions (ring20-dashboard, ntb-atc-plugin) — 1 week grace period
  4. After grace, authorize this task to proceed (Tier-2: `fw task update T-1166 --status started-work` is not enough — the human must explicitly confirm in this AC)
  **Expected:** Explicit retirement approval
  **If not:** Extend the parallel operation period and re-check in 30 days

## Verification

cargo build
cargo test
cargo clippy -- -D warnings
! grep -rn "event\.broadcast\|event_broadcast" crates/ --include='*.rs' | grep -v "deprecated\|test\|fixture"
! grep -rn "inbox\.\(list\|status\|clear\)" crates/ --include='*.rs' | grep -v "deprecated\|test\|fixture"
test -f docs/migrations/T-1166-retire-legacy-primitives.md

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

### 2026-04-20T14:12:20Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1166-t-11559-retire-legacy-eventbroadcast--in.md
- **Context:** Initial task creation
