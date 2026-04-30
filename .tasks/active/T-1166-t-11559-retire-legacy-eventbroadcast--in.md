---
id: T-1166
name: "T-1155/9 Retire legacy event.broadcast + inbox + file.send/receive primitives"
description: >
  After N months of parallel operation + deprecation warnings (T-1155 S-5 phase 4). Remove hub router handlers for event.broadcast, inbox.*, file.* once all callers migrated. Protocol bump + version diversity check (T-1132) gates removal.

status: started-work
workflow_type: decommission
owner: agent
horizon: now
tags: [T-1155, bus, deprecation]
components: []
related_tasks: [T-1155, T-1158]
created: 2026-04-20T14:12:20Z
last_update: 2026-04-30T07:11:16Z
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

### 2026-04-30T07:18Z — T-1417 staged: pre-cut migration of event.broadcast `--targets` fanout [agent autonomous pass]
- **Pre-cut gap discovered:** Reading `crates/termlink-mcp/src/tools.rs::termlink_broadcast` (line 1852) and `crates/termlink-cli/src/commands/events.rs` (line 320), both still call legacy `event.broadcast` when `--targets a,b,c` is non-empty. The migration doc explicitly flags this: "Per-target fan-out still uses event.broadcast until T-1166 cuts the router method — at which point the CLI will need a separate replacement (planned: parallel emit_to calls)."
- **Risk if not migrated pre-cut:** Post-cut, callers using `termlink event broadcast --targets ...` get -32601 method-not-found from the hub. The empty-targets case is already migrated (T-1401/T-1403 channel.post(broadcast:global)). Most callers don't use --targets, so blast radius is limited but real.
- **T-1417 created (horizon: next, captured):** Detailed implementation spec — `event.emit_to` already exists in protocol + router (not retired), so the fix is a fan-out loop with per-target result aggregation. ACs cover both call sites (CLI + MCP), result-shape preservation, partial-failure semantics (succeeded/failed counters, not hard error), and migration-doc update. Implementation sketch included for the next agent to pick up cleanly.
- **Pre-bake checklist:** 16 shipped + 1 staged (T-1415 post-cut, T-1417 pre-cut). The arc is now: ship T-1417 → re-verify cut-ready → operator authorizes Tier-2 cut → bake 7d → fire T-1415 cleanup.

### 2026-04-30T07:14Z — T-1416 api-usage `--cut-ready` flag: binary gate on attributable-only legacy [agent autonomous pass]
- **Why now:** The T-1166 entry gate is statistical (legacy_pct over rolling window) — useful for trend, but the wrong gate for the actual cut decision. Operator's real question: "is ANYONE still hitting legacy methods, ignoring the pre-deploy backlog?" That's a binary check on `legacy_attributable == 0`.
- **Patch:** `--cut-ready` flag added to `api-usage.sh` (additive, no existing-behavior change). Exit 0 iff `legacy_attributable == 0` in chosen window (default 7d). Composes with `--json` for compact CI output.
- **Verified live on .107:** 575 attributable + 3401 pre-T-1409 → `--cut-ready` returns NOT READY (exit=1). Once .143 is migrated, attributable drops to 0 and the gate flips to READY (exit=0). The pre-T-1409 backlog is ignored — it ages out of the 60d window naturally.
- **Mirrored upstream:** `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh` commit 616ea2cb6 → onedev master pushed.
- **Pre-bake checklist now 16/16 shipped** — T-1400 through T-1414 + T-1416. T-1415 (post-cut source cleanup) drafted with horizon=later and detailed inventory; fires after Tier-2 cut + ≥7d bake.
- **Use cases:** T-1415 prelude verification, CI gate for the post-cut binary build, future watchtower page rendering "X hubs cut-ready, Y not yet" status.

### 2026-04-30T07:10Z — Holdout .143 IDENTIFIED: ring20-dashboard re-numbered (TLS fingerprint match) [agent autonomous pass]
- **The mystery is solved.** TLS fingerprint of `192.168.10.143:9100` is `sha256:53de15ec8b33b4e87abd57d6...` — matches `~/.termlink/known_hubs` line for `192.168.10.121:9100` (`sha256:53de15ec8b33b4e87abd57d6e9700553d68382d66a105cf0c14690bf452b6fe4`). The dashboard container has been renumbered from .121 → .143 since last pin update (last_seen .121 = 2026-04-29T11:30Z). Same persistent TLS cert (T-985 / T-1028 persist-if-present), so cert-pin still trusted.
- **ARP confirms Proxmox VE container:** MAC `bc:24:11:15:62:d1` — `bc:24:11` is the Proxmox vNIC OUI. Consistent with the ring20-dashboard container topology recorded in the operator's reference memory.
- **Why it polls inbox.status:** The legacy-fallback shim (T-1235, `inbox_channel::status_with_fallback`) DOES prefer `channel.list` if the hub advertises it. So the dashboard binary on .143 is either (a) pre-T-1235 termlink-cli that never had the dual-read shim, or (b) bypassing the shim and calling inbox.status directly. Cadence (~60s) is consistent with a `termlink doctor` loop or a custom dashboard-poll script.
- **Operator action — single migration step closes the gate:** Upgrade termlink-cli on the ring20-dashboard container to a binary that includes T-1235 (the dual-read shim). The hub on .107 already advertises channel.list; once the caller picks up the shim, polls switch over and legacy traffic from .143 drops to zero within one polling interval. No hub-side change needed.
- **Why the agent can't fix it directly:** The hubs.toml profile `ring20-dashboard` still points at .121 (stale), and probing .143:9100 returns `Authentication required` on both `hub.version` and `hub.capabilities` — no way to determine the running binary version without the OOB secret. Out of agent autonomous-mode scope.
- **Post-cut still gated on:** (1) operator does the upgrade above, OR (2) Tier-2 authorization to flip the const + rebuild + deploy regardless (rejecting the .143 caller on hub side, breaking its inbox.status loop until the dashboard is fixed). PL-094 destructive-cut staging (T-1411 + T-1413) made path (2) safe and reversible.

### 2026-04-30T07:03Z — T-1414 api-usage agent: split attributable vs pre-T-1409 unattributable legacy [agent autonomous pass]
- **Why now:** Post-T-1409 deploy (2026-04-29 21:49 UTC on .122) the audit captures peer_addr for every TCP caller. But the rolling 7d/30d/60d windows still include pre-deploy lines that have no `from`, no `peer_pid`, no `peer_addr` — these surface as "(unknown)" in legacy_callers and inflate the bake-fail picture. Live snapshot today: 7d window shows 6.21% legacy / FAIL, but 3401/3964 of those legacy lines are pre-deploy backlog. Of the 563 attributable, 552 (~98%) trace cleanly to a single IP: **192.168.10.143** polling `inbox.status` on a ~60s cadence.
- **Patch (additive, gate logic unchanged):**
  - `stats_for_window()` now also returns `legacy_unattributable` (count of legacy lines with no `from` AND no `peer_pid` AND no `peer_addr` — definitionally pre-T-1409 backlog on this hub).
  - JSON: new fields `legacy_attributable` and `legacy_unattributable_pre_t1409` at root level + per-window-row level. Both single-window and trend modes covered.
  - Human text: clarifying split line under "Legacy primitives:" so operator sees "563 attributable, 3401 pre-T-1409" instead of one muddled aggregate.
- **Verified live on .122:** `legacy=3964 = legacy_attributable=563 + legacy_unattributable_pre_t1409=3401`. Math holds. `legacy_callers_by_ip` shows the holdout unambiguously.
- **Mirrored upstream:** `/opt/999-Agentic-Engineering-Framework/agents/metrics/api-usage.sh` commit 3c5ed476c → onedev master pushed.
- **Why this matters for the cut:** With this split, the operator's mental model switches from "we have 6.21% legacy across some window" to "we have ONE host left to migrate and a ~60-day backlog that ages out on its own". The decision-gate becomes binary, not statistical.
- **Pre-bake checklist now 15/15 shipped** — T-1400 through T-1414. Cut still gated on .143 decom + Tier-2 authorization (both outside agent scope per CLAUDE.md autonomous-mode boundaries).
- **Backlog rollover ETA:** Pre-T-1409 lines age out of the 60d gate window naturally by ~2026-06-28; from then on the gate metric reflects current reality without the split needing to be consulted.

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

### 2026-04-30T00:45Z — T-1413 cargo-feature-driven const + OFF-path test suite [agent autonomous pass]
- **T-1413 closed** — `crates/termlink-hub/Cargo.toml`: new `[features]` section with `legacy_primitives_disabled = []` (empty deps, pure cfg switch). `crates/termlink-hub/src/router.rs`: const becomes `pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = !cfg!(feature = "legacy_primitives_disabled");` — default-feature-off preserves byte-identical production behavior.
- **5-test `cut_path` module** (gated by `#[cfg(feature = "legacy_primitives_disabled")]`) covers: const-is-false invariant, capabilities-advertises-false, methods-array-excludes-retired-names, route returns -32601 for event.broadcast, route returns -32601 for each inbox.{list,status,clear}. 3 existing tests gated to default-only (T-1215, T-1405, tcp_broadcast happy path) because they assert legacy-on behavior.
- **CI verification path live:**
  - `cargo test -p termlink-hub --lib`: 291 PASS
  - `cargo test -p termlink-hub --lib --features legacy_primitives_disabled`: 293 PASS
- **Migration doc updated:** new step 3 in `## Operator Cut Procedure` runs the OFF-feature test suite as a pre-flip verification gate — green means "the cut works, ship". References list extended with T-1413.
- **The cut is now CI-proven, not just code-reviewed.** Operator running the cut sees concrete green tests for the post-cut behavior before flipping the const in production.
- **Pre-bake checklist now 14/14 shipped** — T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408, T-1409, T-1410, T-1411, T-1412, T-1413. Cut still gated on .143 caller migration + Tier-2 authorization.

### 2026-04-30T00:30Z — T-1412 migration doc updated for one-flag-flip cut + PL-094 pattern captured [agent autonomous pass]
- **T-1412 closed** — `docs/migrations/T-1166-retire-legacy-primitives.md`: new "## Operator Cut Procedure" section (file path, line, build/install/restart commands, capabilities-flip smoke test via raw socket probe, rejection smoke test). Roll-Back rewritten — the flag-flip is reversible until source-cleanup follow-up ships; recommend ≥7-day flag-off bake. References list extended with T-1406..T-1411.
- **PL-094 captured (Level D operational reflection)** — generalized the T-1166 arc pattern: stage a destructive cut into a single-character flip via (1) forensics, (2) regression guard, (3) feature flag exposed, (4) flag-gated rejection pre-staged, (5) source-cleanup as no-risk follow-up. Reusable for future destructive-API cuts.
- **T-1166 pre-bake checklist now 13/13 shipped** — T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408, T-1409, T-1410, T-1411, T-1412. The cut itself is now: edit one line in router.rs, recompile, restart hub. Still Tier-2 gated.

### 2026-04-30T00:20Z — T-1411 hub-side flag-gated rejection pre-staged; cut becomes one-character flip [agent autonomous pass]
- **T-1411 closed** — `crates/termlink-hub/src/router.rs`: introduced `pub(crate) const LEGACY_PRIMITIVES_ENABLED: bool = true;` as the single source of truth for the T-1166 cut. Wired into both (a) `features.legacy_primitives` value in `handle_hub_capabilities` and (b) guarded match arms `<METHOD> if !LEGACY_PRIMITIVES_ENABLED => legacy_method_retired_response(id, ...)` above each of the 4 router-handled legacy methods (event.broadcast, inbox.list/status/clear). Helper returns JSON-RPC -32601 with message naming T-1166 + the migration doc.
- **Cut now atomic at the hub layer:** flipping the const from `true` to `false`, recompiling, restarting hub produces post-retirement behavior in one commit. The actual source-cleanup (deleting `handle_event_broadcast` + inbox handlers + 6 client-side fallback paths) becomes a follow-up at zero risk because flag-off behavior is test-proven.
- **Tests (3 new, all PASS):** `legacy_method_retired_response_shape`, `hub_capabilities_flag_value_matches_const` (proves single-source-of-truth invariant), `is_retired_legacy_method_predicate`. Total 291 hub lib tests pass (288 prior + 3).
- **Live verification:** Hub PID 2574661 (post-restart with new binary): probed via raw Unix socket — `features.legacy_primitives:true`, all 4 legacy method names present in `methods[]`, `.143` inbox.status traffic continues unaffected. Flag-on path is byte-identical to pre-T-1411.
- **T-1166 cut sequence simplified.** When authorized: change `LEGACY_PRIMITIVES_ENABLED` to `false`, build, restart hub. Capabilities flips. Source-cleanup PR follows separately.
- **Pre-bake checklist now 12/12.** Forensics surface complete (T-1407+T-1409+T-1410), regression guard up (T-1406), capability flag exposed (T-1405), evidence telemetry live (T-1408+T-1409 by-IP), cut infrastructure pre-staged (T-1411). Cut still gated on the .143 caller decommissioning + Tier-2 authorization.

### 2026-04-29T22:00Z — T-1410 IP rollup shipped (api-usage agent UX) [agent autonomous pass]
- **T-1410 closed** — `agents/metrics/api-usage.sh` (upstream commit b663ef781): `legacy_callers_by_addr` → `legacy_callers_by_ip`, ports stripped via new `addr_to_ip(addr)` helper using `rsplit(':', 1)`. IPv4 + IPv6 (bracket form) both handled. Section heading is now "Legacy callers by IP (last Nd)".
- **Why:** T-1409's by-addr breakdown grouped per (method, "ip:port"). Each TCP connection draws a fresh ephemeral port so a single host hammering inbox.status 60×/min would fragment into N rows of count=1 — operator's question is "which host?" not "which connection?".
- **Live verification:** .143 (the mystery poller) collapsed from N rows → 1 row showing cumulative count for the host. Test: `fw metrics api-usage --last-Nd 1 --json` returns `{"method":"inbox.status","peer_ip":"192.168.10.143","count":8}`.
- **Schema bump risk:** breaking JSON-field rename (`legacy_callers_by_addr` → `legacy_callers_by_ip`), but T-1409 was 30 minutes old — no consumers on it yet. Better to land the right shape now.
- **T-1166 pre-bake checklist now 11/11.** Forensics surface complete; UX surface clean; ready for cut once .143 caller migrates or is decommissioned.

### 2026-04-29T21:55Z — T-1409 closes TCP-side forensics gap; mystery poller identified as 192.168.10.143 [agent autonomous pass]
- **T-1409 closed** — `crates/termlink-hub/src/{rpc_audit,server}.rs`: hub now threads `peer_addr: Option<String>` from the TCP+TLS / TCP-no-TLS accept paths through `handle_connection` → `record()` / `warn_if_legacy()` → audit line. Mirror of T-1407 for the network side: peer_pid is None for TCP by construction, so peer_addr fills the "who is this anonymous TCP caller" gap.
- **Schema additive:** `{"ts":...,"method":"X","peer_addr":"ip:port"}` — non-empty peer_addr only. Unix path passes `None`. 4 new unit tests (peer_addr only, with from, all-three-fields, empty-omitted). 21 rpc_audit + 288 hub lib tests pass.
- **Agent mirrored upstream:** `agents/metrics/api-usage.sh` (commit b381a53f9 on /opt/999-AEF master, pushed to OneDev) now parses peer_addr per JSONL entry and prints `Legacy callers by addr (last Nd):` block in trend + single-window + JSON modes. Stable shape — `legacy_callers_by_addr` field added to JSON output.
- **Live verification — bake mystery solved.** Previous session diagnosed the bake-window legacy floor as a 60s anonymous `inbox.status` poller stopping at session-end. THIS session re-checked and found the poller still firing every 60s. With the T-1409 hub binary (built + installed + restarted as PID 1470670), the very next poll appeared in audit as `{"ts":1777499373875,"method":"inbox.status","peer_addr":"192.168.10.143:35852"}`. The fw agent immediately surfaces it under "Legacy callers by addr". Caller is **192.168.10.143** — a LAN host running its own termlink hub (rcgen self-signed CN), MAC bc:24:11:15:62:d1 (Proxmox VE vNIC), connecting 11x/min. Not in our hubs.toml fleet config.
- **Forensics surface complete:** Unix callers identified by peer_pid (T-1407+T-1408), TCP callers by peer_addr (T-1409). Anonymous-caller blind spot closed end-to-end.
- **T-1166 pre-bake checklist: 10/10 shipped** — T-1400, T-1401, T-1402, T-1403, T-1404, T-1405, T-1406, T-1407, T-1408, T-1409. Cut still gated on .143 poller migration (or hub-side decommission) + Tier-2 authorization.

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


### 2026-04-30T07:11:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
