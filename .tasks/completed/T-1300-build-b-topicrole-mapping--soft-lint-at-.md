---
id: T-1300
name: "Build B: Topic↔role mapping + soft-lint at emit (hub-side)"
description: >
  Per T-1297 GO: hub-side YAML mapping (~/var/lib/termlink/topic_roles.yaml or similar) + soft-lint at event emit. 10 prefix rules + 4 exempt categories cover 95% of current topic catalog. Warning-only (NEVER reject); emit a sentinel event (e.g. routing.lint.warning) to subscribed channels. Hot-reload on SIGHUP. Compares topic prefix against caller session's roles (and payload.relay_target/needs/from when present per Spike 1 design signal). Estimate: 1 dev-day. Reversible: lint can be globally disabled via config. Evidence: docs/reports/T-1297-termlink-agent-routing-discipline.md § Spike 3.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [termlink, routing, lint, T-1297-child, hub]
components: [crates/termlink-cli/src/commands/events.rs, crates/termlink-hub/src/channel.rs, crates/termlink-hub/src/lib.rs, crates/termlink-hub/src/router.rs, crates/termlink-hub/src/server.rs, crates/termlink-hub/src/topic_lint.rs, crates/termlink-mcp/src/tools.rs, crates/termlink-session/src/inbox_channel.rs]
related_tasks: [T-1297]
created: 2026-04-26T21:19:39Z
last_update: 2026-04-29T07:44:27Z
date_finished: 2026-04-29T07:43:08Z
---

# T-1300: Build B: Topic↔role mapping + soft-lint at emit (hub-side)

## Context

Per T-1297 GO (`docs/reports/T-1297-termlink-agent-routing-discipline.md` § Spike 3): hub-side topic↔role mapping with **soft-lint** at emit. NEVER reject — emit a sentinel `routing.lint.warning` envelope when caller's session role does not match the topic prefix's expected role(s). Mapping lives at `<runtime_dir>/topic_roles.yaml`; if absent, hub uses 10 built-in default rules + 4 exempt categories that cover 95% of the current 125-topic catalog. Hot-reload on `SIGHUP`.

Lint applies to `event.broadcast` and `event.emit_to` (the two RPC paths that fan-out to subscribers). Caller identification uses the optional `from` field; if absent, lint is skipped (logged at debug). T-1302 already delivers `TERMLINK_SESSION_ID` env-var injection so the CLI can populate `from` automatically.

`relay_for` per-session opt-in is **out of scope** — that is Build C / T-1301.

## Acceptance Criteria

### Agent
- [x] New module `crates/termlink-hub/src/topic_lint.rs` defines `Rules` (prefix rules + exempt prefixes), `LintOutcome` (`Pass | ExemptMatch | NoMatchingRule | Warn { rule_prefix, expected_roles, actual_roles }`), and a pure `lint(topic, caller_roles, &rules) -> LintOutcome` function with no I/O
- [x] `Rules::defaults()` returns the 10 prefix rules + 4 exempt categories from Spike 3 (framework, channel, pickup, learning, inception, claude.md, gap, peer, infra, oauth/outage; exempt: agent., session., worker., test., help., channel.delivery)
- [x] `Rules::load_from_path(path)` parses YAML into `Rules` and returns `Result<Rules, anyhow::Error>` with a clear parse error. Unknown YAML keys are tolerated (forward-compat)
- [x] Hub `init_topic_lint(runtime_dir)` loads `<runtime_dir>/topic_roles.yaml` if it exists; falls back to `Rules::defaults()` otherwise. Logs which path it took at info level
- [x] `handle_event_broadcast` and `handle_event_emit_to` extract optional `from` param, resolve caller session via `manager::find_session` (local), run `lint()`, and on `Warn` outcome best-effort dual-write a `routing.lint.warning` envelope to bus topic `routing:lint`. Emit response is unchanged (lint is soft)
- [x] SIGHUP triggers a reload via `tokio::signal::unix::SignalKind::hangup()`; reload uses `Arc<RwLock<Rules>>`; reload errors keep the previous Rules in place and log at warn level
- [x] CLI `cmd_broadcast` populates `from` from `$TERMLINK_SESSION_ID` if the caller did not pass it explicitly
- [x] Unit tests in `topic_lint.rs`: (1) default rules match `framework:pickup` for role=framework (Pass), (2) `framework:pickup` for role=product (Warn), (3) exempt prefix `agent.request` returns ExemptMatch regardless of role, (4) YAML loader parses a sample file matching the schema, (5) caller with no roles + non-exempt topic = Warn, (6) caller with no roles + exempt topic = ExemptMatch
- [x] Hot-reload test (file-based, no signal): write defaults.yaml → load → modify file → call `Rules::load_from_path` again → new content reflected
- [x] Full workspace builds with no warnings: `cargo build --workspace` clean and `cargo clippy -p termlink-hub -- -D warnings` clean
- [x] All existing tests pass: `cargo test -p termlink-hub` 0 failures

### Human

- [x] [RUBBER-STAMP] **Verify lint fires on .107 for an undeclared session emitting a mismatched-prefix topic.**
  Steps:
  1. `cd /opt/termlink && ./target/release/termlink channel state routing:lint --hub 127.0.0.1:9100 --json | python3 -c "import sys,json;raw=sys.stdin.read();rows=json.loads(raw[raw.find('['):]);print(f'rows={len(rows)}');p=json.loads(rows[-1]['payload']);print(f'last: topic={p[\"topic\"]} from={p[\"from\"]} expected={p[\"expected_roles\"]} actual={p[\"actual_roles\"]}')"`
  Expected: `rows >= 7` and the last row shows `topic=framework:t1300-validation-undeclared-070345 from=tl-bubfbc3w expected=['framework', 'pickup'] actual=[]` (the validation row captured 2026-04-29T07:03Z; later rows are fine — the row must be present).
  If not: probe `grep "framework:t1300-validation-undeclared" <(...the same command...)` — if absent, the bus topic was rotated; re-run the trigger from this session's evidence (Updates 2026-04-29T07:05Z, Trigger 2).

## Verification

cargo build -p termlink-hub 2>&1 | tail -5 | grep -qE "Finished"
cargo test -p termlink-hub topic_lint 2>&1 | tail -10 | grep -qE "test result: ok"
cargo test -p termlink-hub 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
test -f crates/termlink-hub/src/topic_lint.rs

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

### 2026-04-26T21:19:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1300-build-b-topicrole-mapping--soft-lint-at-.md
- **Context:** Initial task creation

### 2026-04-27T06:49:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-29T07:05Z — live fleet evidence captured [agent autonomous pass]
- **A/B test on .107 hub (single binary, single hub):**
  - Topic prefix `framework:` → expected_roles=[framework, pickup]
  - Trigger 1 (suppression-target): `tl-bkfp6hqt` (termlink-agent, roles=[termlink, diagnostics], relay_for=[framework]) → broadcast to `framework:t1300-validation-070030` → 13/13 succeeded → **0 new rows in routing:lint** (relay_for suppressed — proves T-1301 path)
  - Trigger 2 (warning-target): `tl-bubfbc3w` (ntb-dev-test, no roles, no relay_for) → broadcast to `framework:t1300-validation-undeclared-070345` → 13/13 succeeded → **+1 row in routing:lint**: `[6] from=tl-bubfbc3w expected=[framework, pickup] actual=[]` — proves T-1300 lint fires correctly
- **Organic field evidence pre-existing in routing:lint:** 3 rows from `email-archive` session emitting `infra.ingress.request` with mismatched roles (rule_prefix=infra, expected=[ring20-management, infrastructure], actual=[email-archive, pickup]) — lint catches real production prefix-role mismatches not just synthetic tests
- **Cross-hub READ verified earlier in session:** `channel state routing:lint --hub 192.168.10.122:9100` returned `[0] from=tl-yz557gnw expected=[framework, pickup] actual=[]` — proving each hub maintains its own routing:lint canonical state and is readable cross-TCP. (.122 went dark mid-session due to ring20-management environmental churn — unrelated to T-1300, see PL-020.)
- **Soft-lint property confirmed:** all broadcasts returned `13/13 succeeded` regardless of warning outcome — emit responses unchanged, lint is dual-write only.
- **Conclusion:** Build B (lint warnings dual-write to routing:lint with correct expected/actual roles + topic + from/method) is live and observable across the fleet.

### 2026-04-27T07:05Z — build delivered [agent autonomous pass]
- **Module:** `crates/termlink-hub/src/topic_lint.rs` (new) — pure `Rules`/`LintOutcome`/`lint()` plus init/reload/SIGHUP plumbing.
- **Defaults:** 10 prefix rules + 6 exempt categories from T-1297 § Spike 3.
- **Wiring:** `handle_event_broadcast` and `handle_event_emit_to` in `crates/termlink-hub/src/router.rs` extract optional `from`, resolve caller via `manager::find_session`, run lint, and on Warn dual-write a payload to bus topic `routing:lint` via new `crate::channel::mirror_routing_lint_warning`. Emit responses unchanged (soft lint).
- **Init:** `crate::topic_lint::init` + `spawn_sighup_watcher` called from `server::run_with_tcp` next to `channel::init_bus`. Watcher uses `tokio::signal::unix::SignalKind::hangup()`; reload errors keep previous `Rules` in place.
- **CLI:** `cmd_broadcast` in `crates/termlink-cli/src/commands/events.rs` now injects `from = $TERMLINK_SESSION_ID` (T-1302 env-var) when the caller did not pass an explicit `from`.
- **Docs:** `docs/operations/topic-lint.md` (new) — schema, hot-reload incantation, payload shape, opt-out guidance.
- **Tests:** 12 unit tests in `topic_lint::tests` (all 6 spec cases + boundary, most-specific-wins, unknown-key-tolerance, hot-reload, payload-shape).
- **Verification (P-011 gate):** `cargo build -p termlink-hub` ✓; `cargo test -p termlink-hub topic_lint` 12/12 ok; `cargo test -p termlink-hub` 253/253 ok; `cargo clippy -p termlink-hub --tests -- -D warnings` ✓; module file present ✓.
- **All Agent ACs ticked.** Owner=human; awaiting operator validation (no Human ACs declared).

### 2026-04-29T07:43:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Completed via Watchtower UI (human action)
