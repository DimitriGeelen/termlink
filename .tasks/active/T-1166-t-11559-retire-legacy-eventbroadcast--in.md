---
id: T-1166
name: "T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives"
description: >
  After N months of parallel operation + deprecation warnings (T-1155 S-5 phase 4). Remove hub router handlers for event.broadcast, inbox.*, file.* once all callers migrated. Protocol bump + version diversity check (T-1132) gates removal.

status: captured
workflow_type: decommission
owner: agent
horizon: next
tags: [T-1155, bus, deprecation]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:20Z
last_update: 2026-04-29T20:35:17Z
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
- [x] [REVIEW] Approve retirement timing — ticked by user direction 2026-04-23. Evidence: User direction 2026-04-23 — legacy primitive retirement timing approved.
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

### 2026-04-22T04:52:49Z — status-update [task-update-agent]
- **Change:** horizon: later → next

### 2026-04-29T07:55Z — telemetry-driven re-audit; gate FAILS, two surgical migrations identified [agent autonomous pass]
- **Telemetry surface NOW EXISTS:** `<runtime_dir>/rpc-audit.jsonl` (T-1304) + `fw metrics api-usage` agent (T-1311). The previous audit's "telemetry gate untestable" line is stale — both shipped before this session.
- **Live numbers (60d window, /var/lib/termlink/rpc-audit.jsonl, 48,543 records):**
  - Legacy traffic: **5.46%** (2,651 calls) — gate threshold is 1.0%. **GATE FAILS.**
  - Top legacy method: `inbox.status` — 2,453 calls (5.1%), 100% from `(unknown)` caller
  - Second: `event.broadcast` — 193 calls (0.4%), 184 unknown + 9 from named sessions
  - `inbox.list`: 5 calls. `inbox.clear`, `file.send`, `file.receive`: ZERO. Effectively retired already.
- **Source-map of the two real blockers:**
  - `inbox.status (unknown)` source: `crates/termlink-cli/src/commands/infrastructure.rs:434` (`fw doctor` step 7) + `crates/termlink-mcp/src/tools.rs:5166` (`termlink_doctor` MCP tool step 3). Both call `rpc_call("inbox.status", ...)` directly, bypassing the existing `inbox_channel::status_with_fallback` shim. Each `fw doctor` run emits one inbox.status; the 2453-call total reflects ~2453 doctor invocations over the audit window.
  - `event.broadcast (unknown)` source: `crates/termlink-cli/src/commands/events.rs:211` (`cmd_broadcast`). This caller IS already env-var aware (T-1310 injects `from = $TERMLINK_SESSION_ID`), but ad-hoc shells running `termlink event broadcast` without setting the var produce the 184 unknowns. Migrating cmd_broadcast to call `channel.post` against `broadcast:global` (the same topic the hub-side mirror already writes to per T-1162) eliminates the legacy method dispatch entirely.
- **Decomposition:** spawned T-1400 (doctor inbox.status migration — eliminates 2453 calls / 5.1% in one shot). The event.broadcast migration (a `cmd_broadcast` rewrite) is the second sub-task — to be spawned as T-1401 once T-1400 ships and bakes.
- **Forecast:** T-1400 alone drops legacy% from 5.46% to ~0.4% (under the 1% gate). T-1401 brings it to <0.05%. Together they unblock T-1166 entry gate and allow the actual decommission to schedule.
- **Status:** stays `captured` — preconditions in flight, not yet ready to start. Will re-audit after T-1400+T-1401 land + bake 24h.

### 2026-04-26T22:42Z — entry-gate audit (no AC ticks; status stays captured) [agent autonomous pass]
- **Telemetry gate (AC line 30):** UNTESTABLE — `fw metrics api-usage --last-60d` is not an implemented subcommand (only `dashboard`, `predict` exist). The gate references a tool that was assumed but not built. Either (a) build the telemetry, or (b) replace the gate with a different signal before retirement can proceed.
- **Code gate (AC line 31):** PARTIAL.
  - `file.send` / `file.receive` — **0 live callsites in `crates/`.** Router constants gone (`FILE_SEND`/`FILE_RECEIVE` not in `control.rs` or `router.rs`). User-facing `termlink_file_send`/`termlink_file_receive` MCP tools at `tools.rs:3109/3318` survive but operate on the post-migration event protocol (`file.init`/`file.chunk`/`file.complete` topics), not the legacy RPCs. Surface is effectively retired; only the verb-name remains for UX continuity.
  - `event.broadcast` — **~30 hits.** One is a direct CLI caller (`cmd_broadcast` in `commands/events.rs:201` does `rpc_call(... "event.broadcast" ...)`); others are protocol const, auth scope rule, MCP tool description, and the T-1162 hub-side mirror shim (`hub/src/channel.rs::mirror_event_broadcast`). The CLI command would need to be rewritten as a `channel.post` thin wrapper before the router method can be removed — that's a user-visible UX change (per T-1166 line 33 "Choose per UX review") so it stays gated on operator decision.
  - `inbox.{list,status,clear}` — **~20 hits.** Migration shim in `session/src/inbox_channel.rs` does probe → channel-aware → fallback with `warn_once` deprecation messages. CLI `remote.rs` and MCP `tools.rs` route through the shim. Real fallback rate is unknown (back to telemetry gate). Not safe to retire without proving the warn_once never fires for real callers.
- **Recommendation:** keep status `captured`. To proceed:
  1. Build a telemetry surface (`fw metrics api-usage` or equivalent over the existing observability log) — own follow-up task
  2. UX-review the `termlink broadcast` rewrite — own follow-up task (cmd_broadcast → channel.post wrapper)
  3. Run the inbox shim's warn_once stats for ~7 days to confirm no live fallbacks
- **No ACs ticked.** This is an audit log entry; the structural gates remain unchanged.

### 2026-04-29T~time~ — both sub-migrations shipped; bake window starts [agent autonomous pass]
- **T-1400 closed earlier today** — `fw doctor` and `termlink_doctor` MCP tool now use `channel.list(prefix="inbox:")` with inbox.status fallback. Live-verified.
- **T-1401 closed minutes ago** — `cmd_broadcast` routes to `channel.post(broadcast:global)` when `--targets` is empty (the dominant case). Live-verified: zero new event.broadcast audit lines per broadcast, msg_type matches hub-side T-1162 mirror shape.
- **Operator binary refresh:** `target/release/termlink 0.9.1567` installed to `/root/.cargo/bin/termlink`. Post-install verification: `termlink event broadcast` emits one `channel.post`, `termlink doctor` emits one `channel.list`, neither emits a legacy method.
- **Trend (post-binary-install snapshot):** 1d=4.91% / 7d=5.42%. The drop will materialize as the audit log accumulates new entries from the migrated binary and old entries age out (60d window). Forecast: 1d <0.5% within 24h, <0.05% within 7d if no new legacy callers appear.
- **Outstanding cohort to migrate:** 9 named-session callers of event.broadcast remain in 60d window (7+1+1 from `tl-bkfp6hqt`, `tl-ismotg7j`, `tl-bubfbc3w`). These are remote sessions on other hosts whose termlink binaries are independent — fleet rollout, not a code change. Will surface as a follow-up task if they continue past 24h bake.
- **Outstanding sub-system:** the MCP server process spawned by Claude Code may still hold the pre-T-1400 binary in memory; will refresh on next Claude Code launch / MCP restart. Not blocking.
- **Status:** stays `captured`. Re-check entry gate after 24h bake at 2026-04-30T~10:30Z. If 1d <1%, the gate has effectively passed and we can promote to `started-work` to begin the actual retirement (router method removal, protocol bump, migration doc).

### 2026-04-29T~time2~ — sibling-migration audit + migration guide pre-stage [agent autonomous pass]
- **In-repo audit (post-T-1401, post-T-1403):** `grep -rn 'rpc_call.*"event\.broadcast"' crates/ lib/ skills/` returns exactly 2 lines, both intentional fallback paths (events.rs:320 in cmd_broadcast, tools.rs:1852 in termlink_broadcast MCP tool). `inbox.{list,status,clear}` direct callers: 2 lines, both inside the T-1400 migration shim's fallback. `file.send/receive`: zero. The "Zero live callers in repo" AC effectively means "no callers bypass the migration shims" — this state is reached.
- **T-1402 shipped** — migration guide published at `docs/migrations/T-1166-retire-legacy-primitives.md` (244 lines, ticks AC line 35). Cross-links T-1162/T-1163/T-1164/T-1300/T-1304/T-1311/T-1400/T-1401, includes per-method side-by-side recipes, capability handshake plan, diagnostic queries, roll-forward checklist.
- **T-1403 shipped** — sibling migration of MCP `termlink_broadcast` tool that T-1401 missed (CLI cmd_broadcast was migrated; the MCP tool was a separate code path). Same channel.post-then-fallback pattern.
- **Pre-existing workspace test compilation issue** — `crates/termlink-session/src/bus_client.rs` lib tests fail to compile due to a stale `TransportAddr` API in the test module (`connect_with_interval` takes `TransportAddr` now but tests pass `PathBuf`). NOT introduced by my changes — `git stash && cargo test --no-run` pre-stash also fails. Likely fallout from the T-1385 TransportAddr migration. Worth its own task; doesn't block T-1166.
- **Updated in-repo readiness:** the only remaining work for T-1166 entry is the bake window. All structural code work is done. Pre-staged: migration guide. Awaiting: telemetry to drop below 1% (24h–7d).

### 2026-04-29T~time3~ — bus_client test fix (T-1404) + bake re-audit scheduled [agent autonomous pass]
- **T-1404 closed** — fixed the pre-existing T-1385 test-callsite fallout (3 sites in `bus_client.rs` lib tests + 1 in `tests/bus_client_integration.rs`). Workspace test build now green (336 tests pass for termlink-session). Recorded learning **PL-093**: cargo build does NOT compile `#[cfg(test)]` code, so public-API signature changes must include `cargo test --no-run` as workspace check.
- **Bake re-audit scheduled** — Claude Code cron `ba9d9f2b` set to fire 2026-04-30 at 11:17Z. The job runs `fw metrics api-usage` and reports the 1d/7d/30d/60d trend; if 1d <1% it recommends T-1166 promotion to `started-work`. Job is session-only (Claude Code cron limitation), so /resume tomorrow will see this Updates entry as a manual-fallback reminder if the cron didn't fire.
- **T-1166 status:** still `captured`. Code surface complete; awaiting time. Per-target broadcast (`--targets`) replacement remains a UX-review decision per task line 35 — zero in-repo callers, so safest path is keep-and-reimplement (parallel `event.emit_to`) as a drop-in. Recommend tackling that decision when T-1166 promotes.

### 2026-04-30 (scheduled) — bake-window re-audit pickup checklist
1. Run `fw metrics api-usage` and inspect the 1d window
2. If 1d <1.0%: prepare to promote T-1166 to `started-work` (Tier-2 — needs human authorization). Migration guide already pre-staged; the actual cut work is router method removal + protocol bump + capability handshake flip
3. If 1d >=1.0%: hunt the remaining caller. Likely sources:
   - MCP server processes still holding pre-T-1401 binary (running 4× at session start; will refresh on Claude Code restart)
   - Remote sessions on other hosts running stale termlink binary (binary refresh is per-host)
4. Re-stage cron for next-day re-check if needed

### 2026-04-29T20:55Z — T-1407 audit log enriched with peer_pid + T-1408 agent surfaces it [agent autonomous pass]
- **T-1407 closed** — `crates/termlink-hub/src/rpc_audit.rs` + `server.rs`: hub now threads `peer_pid` from `getsockopt(SO_PEERCRED)` (already extracted at connect time for the same-UID check, previously discarded post-check) into the audit log JSONL line + the `tracing::warn!` line for legacy methods. Schema is additive (`{ts, method, from?, peer_pid?}`); existing readers ignore unknown keys. TCP/TLS connections pass `None`. Pid 0 treated as absent. Tests: 17 rpc_audit unit tests (3 new), 284 hub lib + 3 integration. Live-verified by injecting an `event.broadcast` and observing `peer_pid:723266` in `/var/lib/termlink/rpc-audit.jsonl` plus matching `peer_pid=Some(723266)` in `journalctl -u termlink-hub`. Binary 0.9.1579 installed; hub PID 713361 is the verifying process.
- **T-1408 closed (cross-repo)** — `agents/metrics/api-usage.sh` (upstream framework, commit 1e184dd5b on origin/master) gained a parallel "Legacy callers by PID (last Nd)" section in trend + single-window + JSON modes. Builds on T-1407's enriched JSONL. Live-verified the agent now prints `1  event.broadcast  pid=723266` in the new section.
- **Forensics blind spot closed.** Future incidents like the 60s mystery poller can be diagnosed in one query: the agent prints the offending PID, then `ps -p <pid>` identifies the process. Separately, the `tracing::warn!` line carries `peer_pid` for live-tail operator awareness.
- **T-1166 cut sequence remains the same** — when authorized: router method removal, capability flag flip, protocol bump, fallback path removal in 6 allowlisted files, T-1406 allowlist shrinks to zero. The cut is one commit. Pre-bake prep complete: T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408 — all shipped.

### 2026-04-29T20:35Z — T-1406 regression-guard test shipped + bake-metric anomaly diagnosed [agent autonomous pass]
- **T-1406 closed** — `crates/termlink-hub/tests/no_legacy_callers.rs`: a structural integration test that walks `crates/**/src/**/*.rs` and fails if a quoted legacy-method literal appears at a caller-shaped use-site outside the 6-file allowlist (router, audit-list, CLI broadcast/doctor fallbacks, MCP broadcast/doctor fallbacks, session inbox_channel.rs). A line classifier skips comments, `const X: &str = "..."`, match arms, and `#[cfg(test)] / #[test]` blocks so the allowlist stays tight. Three sub-tests including a rename-rot guard (`allowlist_entries_exist`) and a predicate smoke test. Negative control verified.
- **Effect on T-1166:** pre-emptive. Any PR that adds a new direct caller during the bake window now fails CI with a clear file:line message and a pointer to `docs/migrations/T-1166-retire-legacy-primitives.md`. Bake stays clean.
- **Bake-metric anomaly diagnosed.** Live `fw metrics api-usage` showed 1d=7.08% (1398/19750) and 60d=6.01%. Root cause: a single anonymous `inbox.status` poller was firing every 60s with empty params (`{}`) and no `from` field, generating ~1440 calls/day. Last hit at 1777490734735 — the poller stopped 62 minutes ago (likely tied to the T-1405 hub restart at session-end). Audit log only captures `{ts, method, from?}` so the originating process is unattributable, but the cessation correlates exactly with the hub restart. Going forward the 1d window will drain quickly (next 24h should drop ~1440 anonymous calls, dragging 1d well below the 1.0% gate).
- **Forecast revised:** with the mystery poller dead and T-1406 protecting against new in-repo callers, 1d should reach <1.0% within ~24h (was contaminated by ~70 calls/h). 60d will lag because of the 3145 backlog, but the rolling-window math means it falls to <1% as the older entries age out (ETA 14–21 days based on current rate).
- **T-1166 status:** still `captured`. Pre-bake structural prep is now: T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406 — ALL shipped. The cut itself is gated on time + telemetry only.

### 2026-04-29T~time4~ — T-1405 capability flag pre-staged + binary install [agent autonomous pass]
- **T-1405 closed** — `hub.capabilities` response now includes `features: {"legacy_primitives": true}`. Live-verified against the running hub (PID 4049739, v0.9.1574) over Unix-socket JSON-RPC. Forward-compatible (clients not reading the field are unaffected).
- **Binary refresh:** termlink 0.9.1574 installed to `/root/.cargo/bin/termlink`; hub restarted (old PID 2430771 → new PID 4049739). The new hub is the one currently serving — every fresh broadcast/doctor invocation since the restart is on the migrated code.
- **Migration guide corrected** — was using placeholder `capabilities.legacy_primitives`, now matches actual wire shape `features.legacy_primitives`. Added T-1403 + T-1405 to references.
- **Consumer side now wirable:** downstream consumers (ring20-mgmt, ring20-dashboard, ntb-atc-plugin, framework-agent, skills-manager) can land their `hub.capabilities` startup check in their next deployment cycle. The check passes (returns `true`) until T-1166 cuts; it then flips automatically.
- **T-1166 cut sequence on flag side:** when T-1166 lands, `handle_hub_capabilities` flips the literal `true` to `false` AND removes the listed legacy methods from the `methods` array AND the actual router match arms. One commit, all three changes.

