---
id: T-1301
name: "Build C: relay_for per-session opt-in (hubs.toml integration with B)"
description: >
  Per T-1297 GO: per-session relay_for TOML declaration in hubs.toml (e.g. [session.framework-agent] relay_for = ["channel.delivery", "learning.*"]). Build B's lint suppresses warnings when the declared session emits declared prefixes. Required to keep framework-agent (multi-purpose: governance + cross-project relay) lint-clean per A1 assumption. Estimate: ½ dev-day. Depends on Build B. Reversible: declarations are additive, missing relay_for == empty list. Evidence: docs/reports/T-1297-termlink-agent-routing-discipline.md § Spike 3.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [termlink, routing, relay, T-1297-child, config]
components: []
related_tasks: [T-1297]
created: 2026-04-26T21:19:42Z
last_update: 2026-04-27T07:05:06Z
date_finished: null
---

# T-1301: Build C: relay_for per-session opt-in (hubs.toml integration with B)

## Context

Per T-1297 GO § Spike 3: per-session opt-in declarations that suppress Build B lint warnings for caller-declared prefixes. Without it, the `framework-agent` (multi-purpose: governance + cross-project relay) generates lint noise on every legitimate `channel.delivery` / `task.complete` / `learning.*` emit, defeating the purpose of B.

**Hub-side declaration file** `<runtime_dir>/relay_declarations.yaml` (sibling to `topic_roles.yaml` — same hot-reload surface, same operator audit point). Per-session entries keyed by display_name. Hot-reload on SIGHUP. Pragmatically chosen over per-client `~/.termlink/hubs.toml` because:

1. The lint runs on the hub, not the client — keeping declarations hub-side avoids a register-time schema bump just to thread one field through.
2. Operators already manage `topic_roles.yaml`; one more file in the same directory is zero new cognitive cost.
3. The inception's "session config (cwd-adjacent)" framing is satisfied by the per-session keying — declarations are still per-session, just centralized.

`relay_for` entries are prefixes with the same boundary semantics as `Rules`: `learning` matches `learning.captured` and `learning:foo` but not `learnings`. When the caller's declared `relay_for` covers the topic, lint returns Pass (suppressed) and no `routing:lint` envelope is written.

## Acceptance Criteria

### Agent
- [x] `crates/termlink-hub/src/topic_lint.rs` adds `RelayDeclarations` struct (Deserialize) with shape `{ sessions: [{ name: String, relay_for: Vec<String> }] }`, plus `RelayDeclarations::defaults()` returning empty (so unconfigured hubs behave identically to T-1300)
- [x] `RelayDeclarations::load_from_path(path)` parses YAML, tolerates unknown keys (forward-compat), returns `Result<Self, anyhow::Error>` with parse-error context
- [x] `relay_suppresses(topic: &str, prefixes: &[String]) -> bool` — pure helper using the same boundary-match semantics as topic-prefix rules
- [x] `init_relay_declarations(runtime_dir)` loads `<runtime_dir>/relay_declarations.yaml` if present; falls back to empty defaults; logs which path it took at info level. Stores `Arc<RwLock<RelayDeclarations>>` in module-level state
- [x] `current_relay_for(display_name: &str) -> Vec<String>` returns the caller's declared prefixes (empty if undeclared)
- [x] Existing `init()` is renamed/extended so the hub bootstraps both rules and relay declarations from a single call (`init_topic_lint(runtime_dir)` or unchanged name) — server.rs calls it once
- [x] SIGHUP reloads BOTH `topic_roles.yaml` AND `relay_declarations.yaml`. Reload errors keep the previous state in place (per-file)
- [x] `run_topic_lint` in `crates/termlink-hub/src/router.rs` consults `current_relay_for(caller.display_name)` after `lint()` produces Warn; if the topic is covered by the caller's `relay_for`, suppress the warning (no dual-write to `routing:lint`); log at debug
- [x] Unit tests in `topic_lint.rs`: (1) `relay_suppresses` matches `learning` prefix to `learning.captured` and `learning:x`, (2) `relay_suppresses` rejects `learnings.foo`, (3) YAML loader parses sample, (4) `current_relay_for` returns empty for undeclared session, (5) end-to-end: a Warn outcome from `lint()` is suppressed when the caller declares a covering relay prefix
- [x] Hot-reload test (file-based): write declarations.yaml → load → modify file → reload → new content reflected
- [x] Operator docs: `docs/operations/topic-lint.md` extended with a `relay_declarations.yaml` schema section
- [x] Workspace builds clean: `cargo build -p termlink-hub`; `cargo clippy -p termlink-hub --tests -- -D warnings`
- [x] All hub crate tests pass: `cargo test -p termlink-hub` 0 failures

### Human

- [ ] [RUBBER-STAMP] **Verify relay_for suppresses warnings — declaration file present + A/B holds on .107.**
  Steps:
  1. `cd /opt/termlink && cat /var/lib/termlink/relay_declarations.yaml`
  2. `cd /opt/termlink && ./target/release/termlink channel state routing:lint --hub 127.0.0.1:9100 --json | python3 -c "import sys,json;raw=sys.stdin.read();rows=json.loads(raw[raw.find('['):]);bk=[r for r in rows if 'tl-bkfp6hqt' in (json.loads(r['payload']).get('from') or '')];bb=[r for r in rows if 'tl-bubfbc3w' in (json.loads(r['payload']).get('from') or '')];print(f'tl-bkfp6hqt(termlink-agent w/ relay_for): {len(bk)} warnings');print(f'tl-bubfbc3w(undeclared): {len(bb)} warnings')"`
  Expected: step 1 shows `name: \"termlink-agent\"` with `relay_for: [\"framework\"]`; step 2 prints `tl-bkfp6hqt(termlink-agent w/ relay_for): 3 warnings` (all from BEFORE the declaration was added — see offset [4] `framework:lint-test-after-relay-declared` is the last termlink-agent row) and `tl-bubfbc3w(undeclared): 1 warnings` (the validation row from 2026-04-29T07:03Z). After the declaration was active, no NEW termlink-agent warnings have fired despite multiple emit attempts.
  If not: if step 1 is missing, the file was deleted — restore from this Updates section. If step 2 shows the bk count growing, suppression broke — file an issue against `topic_lint::relay_suppresses`.

## Verification

cargo build -p termlink-hub 2>&1 | tail -3 | grep -qE "Finished"
cargo test -p termlink-hub topic_lint 2>&1 | tail -10 | grep -qE "test result: ok"
cargo test -p termlink-hub 2>&1 | tail -25 | grep -qE "test result: ok\.\s+[0-9]+ passed" && ! cargo test -p termlink-hub 2>&1 | grep -qE "FAILED"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | tail -3 | grep -qE "Finished"
grep -q "relay_declarations.yaml" docs/operations/topic-lint.md

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

### 2026-04-26T21:19:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1301-build-c-relayfor-per-session-opt-in-hubs.md
- **Context:** Initial task creation

### 2026-04-27T07:01:16Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-04-29T07:05Z — live fleet evidence captured [agent autonomous pass]
- **Active declaration on .107:** `/var/lib/termlink/relay_declarations.yaml` contains:
  ```yaml
  sessions:
    - name: "termlink-agent"
      relay_for: ["framework"]
  ```
- **Suppression proof:** broadcast from `tl-bkfp6hqt` (display_name=termlink-agent) to `framework:t1300-validation-070030`. Caller's roles=[termlink, diagnostics] do NOT match expected=[framework, pickup] → would normally Warn. With relay_for=[framework] declared → suppressed. Result: 13/13 succeeded, **0 new rows in routing:lint** (compared with 7 rows after a non-declared session emitted the same prefix).
- **A/B counter-test:** identical broadcast from `tl-bubfbc3w` (display_name=ntb-dev-test, NOT in relay_declarations) to `framework:t1300-validation-undeclared-070345` → +1 row at offset [6]. Same hub, same topic prefix, same expected_roles. Only the relay declaration differs. Suppression is the dispositive variable.
- **Pre-existing field evidence:** `framework:lint-test-after-relay-declared` row from `tl-bkfp6hqt` (offset [4]) is from a previous validation BEFORE relay_for was declared for that session — confirms the same session emits warnings without declaration, then stops once declared. Time-series evidence of toggle behavior.
- **Soft-lint preserved:** suppression does NOT block broadcasts; emit success counts identical (13/13) in both A and B. Reversible-by-config invariant from T-1297 GO holds.
- **Conclusion:** Build C (per-session relay_for in YAML config suppresses Build B warnings for declared prefixes) is live and observable on .107. Cross-hub semantics: each hub reads its own relay_declarations.yaml independently — by design (T-1301 § Pragmatic chosen over per-client TOML).

### 2026-04-27T07:25Z — build delivered [agent autonomous pass]
- **Schema:** `RelayDeclarations { sessions: Vec<RelayEntry { name, relay_for }> }` added to `crates/termlink-hub/src/topic_lint.rs`. Default = empty so an unconfigured hub behaves identically to T-1300.
- **Loader:** `RelayDeclarations::load_from_path(path)` — same forward-compat YAML tolerance as `Rules`.
- **Pure helper:** `relay_suppresses(topic, &[String]) -> bool` reuses the boundary-aware `topic_has_prefix` from T-1300 for consistency.
- **Init:** Folded into the existing `topic_lint::init(runtime_dir)`; reads `<runtime_dir>/relay_declarations.yaml` next to `topic_roles.yaml`. Two independent `Arc<RwLock<_>>` slots so reload failures on one file don't taint the other.
- **SIGHUP:** Same watcher reloads both files; per-file error handling preserves previous state on parse failure.
- **Lookup:** `current_relay_for(display_name) -> Vec<String>`.
- **Wiring:** `run_topic_lint` in `crates/termlink-hub/src/router.rs` now resolves caller's `display_name` from the `Registration`, calls `current_relay_for`, and on Warn checks `relay_suppresses` before dual-writing. Suppression logs at debug and writes nothing to `routing:lint`.
- **Tests:** 7 new unit tests in `topic_lint::tests` covering the 5 spec cases + unknown-key tolerance + hot-reload.
- **Docs:** `docs/operations/topic-lint.md` gained the `relay_declarations.yaml` schema section, hot-reload note, and lookup-by-display_name caveat.
- **Verification (P-011 gate):** `cargo build -p termlink-hub` ✓; `cargo test -p termlink-hub topic_lint` 19/19 ok; `cargo test -p termlink-hub` 260/260 ok; `cargo clippy -p termlink-hub --tests -- -D warnings` ✓.
- **All Agent ACs ticked.** Owner=human; awaiting operator validation (no Human ACs declared).
