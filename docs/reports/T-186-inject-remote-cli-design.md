# T-186: termlink inject-remote — CLI Design Research

## Problem Statement

Cross-machine prompt injection currently requires manual steps proven in T-183/T-184:
1. Parse hex secret from hub config
2. Generate HMAC capability token with correct scope
3. Build TOFU TLS connection to remote hub
4. Authenticate via `hub.auth`
5. Send `command.inject` with `target` param for hub routing
6. Handle split-write delays for special keys

This works but is not repeatable without deep protocol knowledge. We need a single CLI command:
```
termlink inject-remote <host:port> <session> "message" --secret-file /path
```

## Current State Analysis

### What exists today

| Component | Location | Status |
|-----------|----------|--------|
| TOFU TLS verifier | `crates/termlink-session/src/tofu.rs` | Complete (T-182) |
| Client TCP+TOFU path | `crates/termlink-session/src/client.rs:29-68` | Complete |
| Hub auth (HMAC tokens) | `crates/termlink-session/src/auth.rs` | Complete |
| Hub routing (target param) | `crates/termlink-hub/src/router.rs:479` | Complete |
| Split-write inject | `crates/termlink-session/src/handler.rs:460-530` | Complete (T-178) |
| Local inject command | `crates/termlink-cli/src/main.rs:1741` | Local-only (Unix socket) |
| Integration example | `crates/termlink-session/examples/tofu_test.rs` | Manual proof-of-concept |

### Gap: Local inject vs remote inject

Current `cmd_inject()` (main.rs:1741):
- Finds session via `manager::find_session(target)` — local filesystem only
- Connects via Unix socket: `client::rpc_call(reg.socket_path(), ...)`
- No auth, no TLS, no hub routing

Remote inject needs:
- TCP connection with TOFU TLS
- HMAC token generation from secret
- Hub authentication (`hub.auth`)
- Hub-routed `command.inject` with `target` param

## Design Variants

### Variant A: New top-level `inject-remote` command
```
termlink inject-remote <host:port> <session> "message" \
  --secret-file /path/to/secret \
  --enter \
  --delay-ms 10
```

**Pros:** Clear separation, explicit about what it does, no ambiguity
**Cons:** Duplicates inject logic, two commands to learn

### Variant B: Extend existing `inject` with `--remote` flag
```
termlink inject <session> "message" --remote <host:port> --secret-file /path --enter
```

**Pros:** Single inject command, familiar UX
**Cons:** Overloaded command, confusing when session name exists locally AND remotely

### Variant C: `hub` subcommand family
```
termlink hub inject <host:port> <session> "message" --secret-file /path --enter
termlink hub connect <host:port> --secret-file /path  # persistent connection
termlink hub list <host:port> --secret-file /path      # list remote sessions
```

**Pros:** Groups all hub operations, extensible (list, connect, inject)
**Cons:** `hub` already exists for start/stop/status of local hub server

### Variant D: `remote` subcommand family (RECOMMENDED)
```
termlink remote inject <host:port> <session> "message" --secret-file /path --enter
termlink remote list <host:port> --secret-file /path
termlink remote status <host:port> --secret-file /path
```

**Pros:** Clean namespace, extensible, no conflict with existing commands
**Cons:** New top-level command to discover

## Recommended Design: Variant D

### Command: `termlink remote inject`

```
termlink remote inject <host:port> <session> <message> [OPTIONS]

Arguments:
  <host:port>    Remote hub address (e.g., 192.168.10.107:9100)
  <session>      Target session name or ID on the remote hub
  <message>      Text to inject

Options:
  --secret-file <path>   Path to file containing 32-byte hex secret
  --secret <hex>         Hex secret directly (less secure, for scripting)
  --enter                Append Enter keystroke after message
  --key <name>           Send a special key instead of text (Enter, Tab, etc.)
  --delay-ms <ms>        Inter-key delay in milliseconds [default: 10]
  --scope <scope>        Permission scope: observe|interact|control|execute [default: control]
  --no-tofu              Skip TOFU verification (use with caution)
  --json                 Output result as JSON
```

### Secret file format
Plain text file containing 64 hex characters (32 bytes):
```
071d6bb74304c0545b6d5d2ca3fc551ea9856c08f335e33902800d55389cc2f4
```

### Implementation flow
```
1. Read secret from --secret-file or --secret
2. Parse hex → 32-byte array
3. create_token(secret, scope, "", 3600) → CapabilityToken
4. Client::connect_addr(TransportAddr::Tcp{host, port}) → TOFU TLS
5. client.call("hub.auth", _, {"token": token.raw}) → authenticated
6. client.call("command.inject", _, {"target": session, "keys": [...], "inject_delay_ms": delay})
7. Report: "Injected N bytes into <session> on <host:port>"
```

### Error scenarios
| Error | User message |
|-------|-------------|
| Secret file not found | `Error: Secret file not found: /path` |
| Secret not 32 bytes | `Error: Secret must be 64 hex characters (32 bytes)` |
| TOFU violation | `Error: Certificate fingerprint changed for <host:port>. Possible MITM. Run 'termlink remote trust <host:port>' to re-accept.` |
| Auth failed | `Error: Authentication failed — check secret` |
| Session not found | `Error: Session '<name>' not found on <host:port>` |
| Connection refused | `Error: Cannot connect to <host:port> — is the hub running?` |

### Future extensions (Variant D enables)
- `termlink remote list <host:port>` — list sessions on remote hub
- `termlink remote status <host:port>` — remote hub health
- `termlink remote trust <host:port>` — re-accept TOFU fingerprint
- `termlink remote run <host:port> <session> "command"` — inject + wait for output (like `termlink run` but remote)

## Technical Constraints

- macOS + Linux support (both TOFU paths work)
- No new crate dependencies needed (all primitives exist)
- TOFU known_hubs at `~/.termlink/known_hubs` (shared with library)
- Hub must be running with `--tcp` on the remote machine
- Secret must be shared out-of-band (no key exchange protocol)

## Scope Fence

**IN scope:**
- `termlink remote inject` command implementation
- Secret file reading + hex parsing
- TOFU+auth+inject chain in one command
- Clear error messages for each failure mode

**OUT of scope:**
- `termlink remote list/status/trust/run` (future tasks)
- Secret distribution/exchange protocol
- Hub auto-discovery (mDNS, etc.)
- Persistent connections / connection pooling

## Implementation Estimate

All building blocks exist. The command is essentially a CLI wrapper around what `tofu_test.rs` does manually. Estimated: ~150 lines of new CLI code + tests.

## Dialogue Log

### Session S-2026-0319-0040 (prior session)
- **User:** "termlink also needs to be enhanced with the standard capability to repeatably execute this connection to claude master on another machine"
- **Agent:** Created T-186 inception task, noted that tofu_test.rs proves all primitives work
- **User direction:** Should handle auth, TOFU, and split-writes automatically
