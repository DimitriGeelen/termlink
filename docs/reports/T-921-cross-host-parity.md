# T-921: Full cross-host parity — research artifact

**Status:** inception, exploring
**Started:** 2026-04-11
**Related:** T-920 (MCP surface for remote family), T-163 (cross-host hub), T-182 (TOFU TLS), T-922 (auto-exposure meta-rule)

## Why this exists (C-001)

Per CLAUDE.md inception discipline: the thinking trail IS the artifact. This
file is the live record of how we scoped T-921. It is written and committed
before spikes execute, and updated incrementally as dialogue produces findings.
Conversations are ephemeral; this file is permanent.

The goal of the inception is to answer **one** question:

> How should termlink CLI commands target a remote hub, so that driving a
> session on another machine feels the same as driving a session locally?

## Spike 1 — CLI command census

Counted by reading `crates/termlink-cli/src/cli.rs` (1864 lines). Every variant
of every `Command` enum, grouped by cross-host suitability. The categories
below are the ones defined in the task's Exploration Plan.

### (a) Local-only — structural

These commands operate on local daemon state or installation state. A
`--target` flag would be dishonest — there is nothing remote to target.

| Command | Why local-only |
|---|---|
| `hub start` / `hub stop` / `hub status` / `hub serve` | Manages the local hub process, pid file, unix socket. |
| `mcp serve` | Runs the MCP stdio server locally. |
| `vendor` | Copies binaries into the current project. |
| `doctor` | Reports on *this host's* termlink installation. |
| `completions` | Shell completion generation. |
| `version` | Binary version. |
| `clean` | Cleans local runtime state (sockets, pids). |

**Count:** ~7 operations. **Verdict:** do not cross-host.

### (b) Session-scoped — natural cross-host candidates

These commands take `--session SESSION_ID` and operate via a hub RPC call. They
are the bread-and-butter of termlink and the real target of T-921. Today they
all resolve to `default_socket_path()`; to go cross-host they need to route
their single RPC through a remote hub instead.

| Command | Hub RPC |
|---|---|
| `ping` | `termlink.ping` |
| `status` | `session.status` |
| `info` | `session.info` |
| `list` | `session.list` |
| `register` | `session.register` |
| `send` | `command.inject` (or `command.run`) |
| `inject` | `command.inject` |
| `exec` | `command.exec` |
| `run` | `command.run` |
| `request` | `command.request` |
| `signal` | `command.signal` |
| `output` (non-follow) | `session.output` |
| `tag` | `session.tag` |
| `kv set/get/list/del` | `kv.*` |
| `emit` / `emit-to` / `broadcast` | `event.*` |
| `events poll` / `watch` | `event.poll` / `event.subscribe` |
| `topics` | `event.topics` |
| `collect` | `event.collect` |
| `wait` | `event.wait` |
| `discover` | `session.discover` |
| `token create` / `token inspect` | `token.*` |
| `agent ask` / `agent listen` / `agent negotiate` | `agent.*` |

**Count:** ~30 operations. **Verdict:** natural cross-host. This is the fat
part of the distribution.

### (c) Session-scoped with streaming

These commands stream bytes from a session's PTY buffer. The current hub RPC
vocabulary is request/response; long-lived streams over TCP hub RPC have never
been exercised in production and may need a different transport path.

| Command | Notes |
|---|---|
| `output --follow` | Tail-f style byte stream. |
| `stream` | Full PTY mirror, byte stream. |
| `mirror` | Read-only PTY mirror (T-234). |
| `attach` | Bidirectional PTY attach. |
| `resize` | Window resize signal (one-shot, easy). |
| `interact` | Stdin pipe + PTY tail. |

**Count:** ~6 operations. **Verdict:** cross-host in spirit, but the transport
answer may be "defer to a follow-up" for the streaming subset. `resize` is
one-shot and trivially cross-host today.

### (d) Batch / orchestration

These commands fan out across multiple sessions (possibly across multiple
hubs). They are the most interesting case: if "cross-host" means "each target
resolves independently", these become naturally distributed.

| Command | Notes |
|---|---|
| `dispatch` | Spawns N workers; each is a local process today. Cross-host = "some workers on host B". |
| `dispatch-status` | Aggregates manifest state; already local-only over file reads. |
| `spawn` | Local process spawn. Cross-host = `session.spawn` RPC on remote hub. |
| `file send` / `file receive` | Today uses hub RPC with chunked payload. Already designed for remote transport in the `remote` subcommand. |
| `pty` (mode selection) | Local preference, no RPC. |

**Count:** ~5 operations. **Verdict:** these are the "needs care" cases. They
will drive the hardest decisions in Spike 2.

### (e) The `remote` subcommand itself

The existing `remote` subtree (see `cli.rs` 841 + `commands/remote.rs`) mirrors
the session-scoped commands but with `--hub` / `--secret` / `--scope` flags:

```
remote ping / list / status / inject / send-file / events / push / profile /
  exec / add / list / remove
```

This is **the structural evidence of the duplication T-921 is trying to
collapse**. Every cross-host capability today has to be re-expressed as a
`remote X` wrapper. The inception is fundamentally asking: is this subcommand
namespace the right answer going forward, or should it be replaced by a
`--target` flag that every command understands?

### Census totals

| Category | Count | Fraction |
|---|---:|---:|
| Local-only (structural) | ~7 | 15% |
| Session-scoped (natural cross-host) | ~30 | 63% |
| Session-scoped streaming | ~6 | 13% |
| Batch / orchestration | ~5 | 11% |
| **Total considered** | **~48** | 100% |

The `remote` subtree adds ~12 duplicate wrappers, but those do not count in
the "total command surface" — they are a projection of the session-scoped set.

**A1 validated:** 63% of session-scoped commands are naturally cross-host with
no streaming concerns. If you include the one-shot `resize` case, it is 65%.

**A2 validated:** the local-only set is real and small (~7 operations). These
should not grow a `--target` flag.

## Spike 2 — Option analysis

> TODO — dialogue-driven. Four options (A/B/C/D) and the five comparison axes
> (ergonomics / duplication / MCP parity / auth / backward compat) will be
> filled in the next pass, after human input on preferences.

### Option A — Unified `--target` flag

Every command in category (b) and (c) gains an optional
`--target HOST:PORT` flag. When present, the command routes its hub RPC to
the remote hub via `connect_remote_hub` (reused from T-920). When absent,
behaviour is unchanged (local unix socket).

- **Flag shape:** `termlink ping --target 10.0.0.107:9100 --session S-x`
- **Secret lookup:** implicit from `~/.termlink/secrets/<host>.hex` (falls back
  to `--secret-file`)
- **Scope:** defaulted per-command — `ping` is `read`, `inject` is `command`

### Option B — Expand `remote` subcommand with per-command wrappers

Keep the current `remote` namespace, add one wrapper per command in categories
(b), (c), (d). `termlink remote ping --hub ... --session ...`,
`termlink remote kv-set --hub ... --key ... --value ...`, etc.

### Option C — `remote call` only

Do not add any new wrappers. Rely on `termlink remote call --method
command.inject --params '{...}'` for every cross-host operation. Ergonomics
are purely on the MCP layer (where `termlink_remote_call` exists already).

### Option D — Hybrid

`--target` on the "simple" session-scoped commands (category b), keep the
`remote` subcommand for the exotic cases (streaming, dispatch, file send).
Retire the duplicated `remote` wrappers that are now covered by `--target`.

### Comparison axes

| Axis | A | B | C | D |
|---|---|---|---|---|
| Ergonomics (human) | ? | ? | ? | ? |
| Ergonomics (agent) | ? | ? | ? | ? |
| Code duplication | ? | ? | ? | ? |
| MCP parity | ? | ? | ? | ? |
| Auth / scope clarity | ? | ? | ? | ? |
| Backward compat | ? | ? | ? | ? |

## Spike 3 — Routing architecture

> TODO — after Spike 2 has a preferred option.

## Spike 4 — Decomposition into build tasks

> TODO — only after human has approved option choice.

## Dialogue Log

### 2026-04-11 — Framing committed

- **Agent:** Filled the T-921 template, ran Spike 1 (CLI census), wrote this
  research artifact. Confirmed A1 (63% natural cross-host) and A2 (structural
  local-only set is real and small). Committed without executing Spike 2 so
  the human can steer option selection.
- **Question for human:** Which option do you want to pursue — A (unified
  flag), B (remote wrappers), C (rpc-call only), or D (hybrid)? My current
  lean is A, because it matches the mental model "cross-host is a transport
  decision, not a separate command family", and because it makes the T-922
  auto-exposure rule trivial ("every new session-scoped command must accept
  `--target`"). But Option D is defensible if streaming or dispatch have
  genuinely different semantics — which Spike 2's detailed comparison will
  decide.
