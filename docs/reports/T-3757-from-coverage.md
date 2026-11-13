# T-3757 — declared caller identity (`from`) on founding verbs: coverage

Step 1 of the operator's ruling A (2026-11-12): every caller of `command.inject` / `command.execute`
declares `params.from`, so that owner-only can later be on by default for launcher sessions (step 2)
without refusing a working route. Identity is declared, not proven (B-6 / PH-4).

## What a caller declares

`termlink_session::founding_log::declared_from()`, in this order (first non-blank wins):

| # | source | example |
|---|--------|---------|
| 1 | `TERMLINK_FROM` (explicit override) | `TERMLINK_FROM=ops-bot termlink exec …` |
| 2 | `TERMLINK_AGENT_ID` (set by `tl-claude.sh --reachable`, be-reachable) | `agent-a` |
| 3 | `user:<USER or LOGNAME>` | `user:root` |
| 4 | `uid:<effective uid>` | `uid:0` |

Never empty, so no use-log row is anonymous. A `from` already in the params is kept (an explicit
value wins; a relay never overwrites the originator). `~/.termlink/be-reachable.state` is not read:
it names one agent for the whole host.

## Call sites

| caller | site | test |
|--------|------|------|
| CLI `exec` | `termlink-cli/src/commands/session.rs` (`command.execute`) | `t3757_exec_declares_termlink_from`, `t3757_no_identity_still_names_the_user`, owner-only control |
| CLI `inject`, `pty inject` | `pty.rs` (`command.inject`, token path) | `t3757_pty_inject_and_inject_declare_agent_id` |
| CLI `pty attach` keystrokes, marker run | `pty.rs` (two more `command.inject` sites) | static sweep |
| CLI `remote inject`, `remote exec` | `remote.rs` | static sweep (needs a hub; no hermetic e2e) |
| CLI push helpers | `push.rs` (exec + inject) | static sweep |
| MCP `termlink_exec`, `termlink_inject` | `termlink-mcp/src/tools.rs` | `t3757_mcp_exec_and_inject_declare_from` |
| MCP `termlink_interact`, `termlink_remote_inject`, `termlink_remote_exec`, `termlink_batch_exec` | `tools.rs` | static sweep |
| MCP `termlink_remote_call` (any method) | `tools.rs`, stamped only when the method is a founding verb | code review (method is a variable) |
| Shell wakers: `notify-injector.sh`, `be-reachable-pushwaker.sh`, `tl-dispatch.sh` | inject through the CLI | covered by the CLI stamping; no script change |
| Sidecar doorbell | `termlink-sidecar/src/doorbell.rs` `notify_params` (`command.notify`, declares the agent it serves) | `t3757_notify_declares_the_agent_as_from` |
| Hub relay | `termlink-hub/src/router.rs` forwards params unchanged | deliberately NOT stamped: it would put the hub's name on an anonymous relayed call |

Static sweep: `tests/t3757-from-fixtures.sh` checks every non-test line in the CLI commands and the
MCP tools that names `command.inject|execute|exec` next to an RPC call passes `with_declared_from`
(14 sites today). Run `bash tests/t3757-from-fixtures.sh` (fixtures) and `… mutants`.

## Notes for step 2

1. A shell waker declares whatever its environment says. Under `tl-claude.sh --reachable` that is the
   agent id (the session owner), so it passes owner-only. A waker started without `TERMLINK_AGENT_ID`
   declares `user:<login>` and would be refused once owner-only is on. The 7-day review must look for
   `user:` / `uid:` rows on launcher sessions.
2. MCP `termlink_batch_exec` and `termlink_remote_exec` call `command.exec`, which no handler in this
   repository serves (the session method is `command.execute`). Out of scope here; stamped anyway.

## 7-day use-log evidence (AC 2)

Not started. The clock starts when a build containing this change is installed on .107.
