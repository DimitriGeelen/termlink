# T-1025: Hub restart TCP detection — research artifact

## Problem

`termlink hub restart` (T-1024) needs to know the TCP address the current hub is listening on so it can restart with the same config. Currently it reads `hub.tcp` from the runtime dir, but this file is only written by `cmd_hub_start` in the CLI layer (`infrastructure.rs:43-50`). Hubs started before T-1024 never wrote this file, so the first restart after deploying new code loses TCP — the hub comes back with Unix socket only, unreachable over the network.

**Real-world impact:** Deployed v0.9.809 to .121 via termlink, ran `hub restart`, hub came back without TCP. Agent on .121 had to manually restart with `--tcp 0.0.0.0:9100`.

## Investigation

### Spike 1: Where is TCP address known?

The TCP address flows through:
1. **CLI arg** `cmd_hub_start(tcp_addr)` — `infrastructure.rs:26`
2. **Server bind** `run_with_tcp(socket_path, tcp_addr)` — `server.rs:114`
3. **TcpListener::bind(addr)** — `server.rs:145` — actual binding, `local_addr()` available at line 148

The server knows the real bound address but doesn't persist it. The CLI writes `hub.tcp` after calling `run_with_tcp` — but only if this specific CLI codepath was used.

### Spike 2: Options for hub.tcp persistence

| Option | Where | Covers all paths? | Actual addr? | Lines changed |
|--------|-------|-------------------|-------------|---------------|
| A: CLI only (current) | `infrastructure.rs:43` | No — only CLI start | CLI arg, not bound addr | Already done |
| B: Server after bind | `server.rs:149` | Yes — all start paths | Yes, `local_addr()` | ~5 lines |
| C: Server + CLI override | Both | Yes + explicit control | Yes | ~8 lines |

**Option B is sufficient.** Write `hub.tcp` at `server.rs:149` right after `local_addr()` is known. Remove the CLI-layer write to avoid duplication.

### Spike 3: Cleanup on shutdown

Line 182-188 in `server.rs` already handles cleanup (removes socket, pidfile, TLS). Adding `hub.tcp` removal here is 1 line. This prevents a stale `hub.tcp` from making a subsequent `hub start` (without `--tcp`) accidentally use an old TCP config.

### Spike 4: hub restart --tcp override

Adding `--tcp` to `hub restart` is orthogonal — useful for changing TCP config during restart. Falls back to `hub.tcp` file. This is ~5 lines in `cli.rs` + `infrastructure.rs`.

### Spike 5: Discovery function

`hub_runtime_dir()` doesn't exist as a public function. The runtime dir is `termlink_session::discovery::runtime_dir()`. The `hub.tcp` path is `{runtime_dir}/hub.tcp`.

## Findings

1. **Root cause confirmed:** `hub.tcp` is written at wrong layer (CLI, not server)
2. **Fix is 5 lines:** Write after bind in `server.rs`, remove after shutdown, delete CLI-level write
3. **No backward compatibility issue:** `hub.tcp` is brand new from T-1024, no existing consumers
4. **Bonus:** Server writes `local_addr().to_string()` which is the REAL bound address (handles `0.0.0.0` → actual interface binding)
5. **Optional:** `hub restart --tcp` override for explicit control — small ergonomic win

## Dialogue Log

- **User asked:** "please elaborate" on the known gap
- **Agent explained:** bootstrapping problem, three fix options
- **User said:** "please incept"
- **Agent:** created inception, researching code paths
