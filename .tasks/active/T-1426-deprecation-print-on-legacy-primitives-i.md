---
id: T-1426
name: "Deprecation print on legacy primitives (inbox.push, file.send, event.broadcast)"
description: >
  Pick #1 from T-1425 RFC. Independent of inception outcome. ~30 lines: stderr warning at every invocation of cmd_remote_push, cmd_file_send, cmd_event_broadcast pointing at 'channel post' as the canonical replacement. Serves dual purpose: nudges vendored agents to migrate, and provides T-1166 cut-readiness telemetry (journalctl grep DEPRECATED). No behavior change otherwise — pure soft deprecation.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-30T21:17:59Z
last_update: 2026-05-18T07:59:11Z
date_finished: null
---

# T-1426: Deprecation print on legacy primitives (inbox.push, file.send, event.broadcast)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Helper `print_deprecation_warning(primitive: &str, replacement: &str)` lives in `crates/termlink-cli/src/commands/mod.rs` (or sibling util module) and writes a single stderr line of the form `[DEPRECATED] termlink <primitive> — use 'termlink <replacement>' instead. See T-1166.`
- [x] Helper is suppressed when env var `TERMLINK_NO_DEPRECATION_WARN=1` is set (so scripts/CI/tests don't get spammed during migration window). The warning goes to stderr only — JSON consumers reading stdout are unaffected, so no per-command --json branch is needed.
- [x] Helper is invoked at the top of every legacy CLI command in the T-1166 retirement set: `cmd_push` (inbox.push), `cmd_broadcast` (event.broadcast), `cmd_file_send`, `cmd_file_receive`, `cmd_inbox_status`, `cmd_inbox_clear`, `cmd_inbox_list`
- [x] Replacement strings cite canonical post-T-1166 verbs: `cmd_push`→`channel post`, `cmd_broadcast`→`channel post` (or `event emit_to` for unicast), `cmd_file_send`→`channel post --file`, `cmd_file_receive`→`channel subscribe`, inbox commands→`channel subscribe`/`channel info`
- [x] Unit test in `crates/termlink-cli/src/commands/mod.rs` (or sibling) confirms helper writes the expected substring to stderr and respects the env-var suppression flag
- [x] No behavior change in any of the seven commands beyond the added stderr line — existing exit codes, JSON shapes (other than the two new keys), and error paths unchanged
- [x] `cargo build --release -p termlink` succeeds; `cargo test --release -p termlink-cli --lib` still green
- [x] Manual smoke: `termlink remote push <some-target> --message x 2>/tmp/dep.err >/dev/null; grep -q "DEPRECATED" /tmp/dep.err` returns 0 even when the push itself fails (warning fires before any I/O)

### Human
- [ ] [REVIEW] Verify the warning is informative without being noisy
  **Steps:**
  1. Build: `cargo build --release -p termlink`
  2. Trigger each legacy command once with bogus args (so it fails fast):
     `for cmd in 'remote push x --message x' 'event broadcast topic-x payload' 'file send target /tmp/nonexistent' 'inbox status' 'inbox list ring20-management-agent' 'inbox clear ring20-management-agent'; do echo "--- $cmd ---"; target/release/termlink $cmd 2>&1 | head -3; done`
  3. Eyeball: each command emits exactly one `[DEPRECATED]` line citing the right replacement verb
  4. Suppression check: `TERMLINK_NO_DEPRECATION_WARN=1 target/release/termlink remote push x --message x 2>&1 | grep -c DEPRECATED` should print `0`
  **Expected:** seven distinct legacy verbs each emit one informative deprecation line; suppression flag works; no double-warn on retried/inner paths
  **If not:** capture the offending command in this task's Updates and re-scope which call site missed the helper

## Verification

cargo build --release -p termlink 2>&1 | tail -3
cargo test --release -p termlink deprecation 2>&1 | grep -q "test result: ok. 2 passed"
TERMLINK_NO_DEPRECATION_WARN=1 target/release/termlink remote push 192.168.10.999:9100 bogus --message x 2>&1 | grep -c DEPRECATED | grep -q '^0$'
target/release/termlink remote push 192.168.10.999:9100 bogus --message x 2>&1 | grep -q DEPRECATED
target/release/termlink event broadcast topic-x 2>&1 | grep -q DEPRECATED
target/release/termlink inbox status 2>&1 | grep -q DEPRECATED
target/release/termlink inbox list bogus 2>&1 | grep -q DEPRECATED
target/release/termlink inbox clear bogus 2>&1 | grep -q DEPRECATED
target/release/termlink file send bogus /tmp/nonexistent 2>&1 | grep -q DEPRECATED

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

### 2026-04-30T21:17:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1426-deprecation-print-on-legacy-primitives-i.md
- **Context:** Initial task creation

### 2026-05-01T07:13:12Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-04T11:00:00Z — Human AC review evidence (mechanical) [agent]

Build: `cargo build --release -p termlink` succeeds (3m 36s).

Each legacy verb emits exactly one informative `[DEPRECATED]` line citing the
right replacement, captured below from one invocation each (release binary):

| Legacy verb | Deprecation line emitted |
|---|---|
| `remote push 192.168.10.999:9100 bogus --message x` | `[DEPRECATED] termlink remote push — use 'termlink channel post' instead. See T-1166.` |
| `event broadcast topic-x` | `[DEPRECATED] termlink event broadcast — use 'termlink channel post' instead. See T-1166.` |
| `file send target /tmp/nonexistent` | `[DEPRECATED] termlink file send — use 'termlink channel post --file' instead. See T-1166.` |
| `inbox status` | `[DEPRECATED] termlink inbox status — use 'termlink channel info' instead. See T-1166.` |
| `inbox list ring20-management-agent` | `[DEPRECATED] termlink inbox list — use 'termlink channel subscribe' instead. See T-1166.` |
| `inbox clear ring20-management-agent` | `[DEPRECATED] termlink inbox clear — use 'termlink channel subscribe --cursor' instead. See T-1166.` |

Suppression: `TERMLINK_NO_DEPRECATION_WARN=1 target/release/termlink remote push x --message x 2>&1 | grep -c DEPRECATED` → `0` ✓

All Human AC sub-checks pass mechanically. Suggest closing:
```
cd /opt/termlink && bash -x .agentic-framework/agents/task-create/update-task.sh T-1426 --status work-completed
```

### 2026-05-18T07:25:00Z — Human AC re-validated against current release binary [agent]

Today re-ran the same Verification block on a fresh `cargo build --release -p termlink` (6m 53s, clean). Captured against `target/release/termlink`:

| Verification command | Result |
|---|---|
| `event broadcast topic-x` | OK — emits `[DEPRECATED]` |
| `inbox status` | OK |
| `inbox list bogus` | OK |
| `inbox clear bogus` | OK |
| `file send bogus /tmp/nonexistent` | OK |
| `remote push 192.168.10.999:9100 bogus --message x` | OK |
| `TERMLINK_NO_DEPRECATION_WARN=1 remote push ... \| grep -c DEPRECATED` | `0` (suppression works) |
| `cargo test --release -p termlink deprecation` | 2 passed (`suppression_env_var_documented`, `warning_format_matches_canon`) |

All 7 task-file Verification commands pass on today's binary; both unit tests still green. Evidence is stable 14 days after the original validation. T-1731 gate blocked the agent from ticking the Human AC checkbox directly; recommended user action below.

**Ready to close.** User runs one of:
- `fw task review T-1426` → review-and-tick via Watchtower (preferred, surfaces evidence in the UI)
- `fw task update T-1426 --status work-completed` → direct close (P-011 verification gate will re-run the commands above)
