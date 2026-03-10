# T-079: Capability Token System — Inception Research

## Problem Statement

TermLink's current auth model (Phase 1: UID check, Phase 2: 4-tier scoping) grants **Execute scope to all same-UID connections**. In multi-agent scenarios, 10+ agents running as the same user all get full shell execution access. There's no way to give Agent A read-only event polling while giving Agent B command execution rights.

**Gaps addressed:** G-001 (command injection — tokens can restrict which agents get Execute), G-002 (no auth beyond UID — tokens add a second auth factor).

## Current State Analysis

| Layer | File | What exists | What's missing |
|-------|------|-------------|----------------|
| Protocol | `control.rs:31-42` | `AUTH_REQUIRED` (-32009), `AUTH_DENIED` (-32010) error codes defined | Not used anywhere |
| Protocol | `control.rs:45-54` | `CommonParams` struct | No token/capability field |
| Session auth | `auth.rs:122-201` | `PermissionScope` enum, `method_scope()` mapping, `satisfies()` | Scope is hardcoded to Execute at connect time |
| Session server | `server.rs:140` | `PermissionScope::Execute` for all same-UID | No token-based scope assignment |
| Hub server | `server.rs:200` | Routes to `router::route()` | No scope enforcement at hub level |
| Client | `client.rs:16-22` | `Client::connect(path)` | No token presentation mechanism |

## Assumptions to Validate

1. **A-001:** HMAC-SHA256 is sufficient for token signing (no asymmetric crypto needed)
2. **A-002:** Tokens can be passed as a JSON-RPC parameter (no transport-level changes needed)
3. **A-003:** Token generation happens at session registration time (hub or session creates tokens)
4. **A-004:** Token revocation is not needed for v1 (tokens expire, no revocation list)

## Design Space

### Option A: Request-Level Tokens (per-request)

**How it works:** Each JSON-RPC request includes a `token` field in params. The server validates the token's HMAC signature, extracts the embedded scope, and uses that instead of the hardcoded Execute.

```
Client                          Server
  │                               │
  │ {"method":"command.execute",  │
  │  "params":{                   │
  │    "token":"hmac-signed-blob",│
  │    "command":"ls"}}           │
  │ ─────────────────────────────►│
  │                               │ verify HMAC
  │                               │ extract scope from token
  │                               │ check method_scope() <= token_scope
  │                               │ dispatch if allowed
```

**Token structure:**
```json
{
  "scope": "interact",
  "session_id": "01JQXYZ...",
  "issued_at": 1741654800,
  "expires_at": 1741658400,
  "nonce": "random-16-bytes"
}
```
Signed: `HMAC-SHA256(secret, canonical_json(payload))` → base64

**Pros:**
- Stateless validation (no token store)
- Each request is independently authorized
- Easy to audit (token in every request)
- No transport changes (just a new params field)

**Cons:**
- Token in every request adds ~100 bytes per call
- Secret management: who generates the HMAC secret?
- Token replay within validity window (mitigated by short TTL)

### Option B: Connection-Level Tokens (authenticate once)

**How it works:** Client sends an `auth.token` request immediately after connecting. Server validates and assigns scope for the entire connection lifetime.

```
Client                          Server
  │                               │
  │ {"method":"auth.token",       │
  │  "params":{"token":"..."}}    │
  │ ─────────────────────────────►│
  │                               │ verify HMAC
  │                               │ set connection scope
  │ {"result":"ok","scope":"..."}│
  │ ◄─────────────────────────────│
  │                               │
  │ {"method":"command.execute",  │  ← uses connection scope
  │  "params":{"command":"ls"}}   │
  │ ─────────────────────────────►│
```

**Pros:**
- Auth once, use many times — less overhead per request
- Simpler request format (no token in every call)
- Natural fit with existing `handle_connection(scope)` parameter
- Connection draining already handles scope cleanup

**Cons:**
- Scope locked for entire connection (can't escalate/de-escalate)
- Must handle unauthenticated window between connect and auth
- More stateful (connection carries scope context)

### Option C: Hybrid — Connection-Level with Per-Request Override

Connection auth sets baseline scope; individual requests can include a higher-scope token for specific operations.

**Pros:** Flexible, supports "normally read-only but can execute when needed"
**Cons:** Most complex, unclear use case for v1

## Analysis

### Recommendation: Option B (Connection-Level Tokens)

**Rationale:**

1. **Fits existing architecture.** `handle_connection` already takes a `granted_scope` parameter. Connection-level tokens just change how that scope is determined — from "hardcoded Execute" to "extracted from token." Minimal code change.

2. **Simpler implementation.** One auth exchange per connection vs. validation on every request. The auth method becomes a new RPC method (`auth.token`), fitting naturally into the method inventory.

3. **Matches the threat model.** Same-UID processes can already read the token from disk. The goal isn't preventing a determined attacker (they can ptrace/read memory) — it's **enforcing the principle of least privilege** for cooperating agents. A connection-scope token achieves this.

4. **Natural upgrade path.** If per-request tokens are needed later (Option C), the connection token becomes the baseline and per-request tokens become an extension. No breaking change.

### Secret Management

The session generates a random secret at registration time, stored in the registration JSON sidecar (already exists at `sessions/{id}.json`). Tokens are created by the session owner (the process that registered) and distributed to agents via environment variable or file.

**Flow:**
1. Session registers → generates 32-byte random secret, stored in `{id}.json`
2. Session creates tokens: `termlink token create --scope observe --ttl 1h`
3. Token distributed to agent (env var, file, or command output)
4. Agent connects and presents token via `auth.token` RPC method
5. Server validates HMAC, extracts scope, assigns to connection

### What Changes

| Component | Change |
|-----------|--------|
| `control.rs` | Add `auth.token` method constant, `TokenPayload` struct |
| `auth.rs` | Add `Token` struct, `generate_secret()`, `create_token()`, `validate_token()` |
| `server.rs` | Default unauthenticated connections to `Observe` scope; handle `auth.token` to upgrade |
| `registration.rs` | Add `token_secret` field to `Registration` JSON |
| `client.rs` | Add `authenticate(token)` method |
| `main.rs` (CLI) | Add `termlink token create/list/revoke` subcommands |

### Scope of Change

- ~200-300 lines of new code in `auth.rs` (token generation/validation)
- ~50 lines in `server.rs` (auth method handler, default scope change)
- ~30 lines in `control.rs` (new method + struct)
- ~20 lines in `registration.rs` (secret field)
- ~50 lines in `client.rs` (auth convenience method)
- ~80 lines in CLI (token subcommands)
- Estimated 3-4 build tasks

### Risks

1. **Breaking change:** Changing default scope from Execute to Observe breaks all existing clients. **Mitigation:** Backward compatibility mode — if no token system is configured (no secret in registration), default to Execute (legacy behavior). Only sessions that opt-in to tokens get Observe default.
2. **Secret in registration file:** The `.json` sidecar is readable by same-UID processes. **Mitigation:** This is acceptable — the threat model is cooperating agents, not adversarial same-UID processes. File permissions are already 0o600.
3. **Token replay:** Tokens can be reused within TTL window. **Mitigation:** Short TTL (1h default), session-scoped nonce.

## GO/NO-GO

### GO — with conditions:

1. **Backward compatible:** Legacy (no-token) connections continue to get Execute scope
2. **Opt-in:** Tokens only enforced for sessions that have `token_secret` in registration
3. **Decompose to 3 build tasks:** (a) token generation/validation in auth.rs, (b) server integration, (c) CLI subcommands
4. **Hub integration deferred:** Hub currently doesn't scope — address after session-level tokens work

## Proposed Build Tasks

1. **T-NEW-1: Token generation and validation** — `auth.rs` additions: `Token` struct, HMAC-SHA256 sign/verify, `generate_secret()`, unit tests
2. **T-NEW-2: Server-side token authentication** — `auth.token` RPC method, default scope logic (Execute if no secret, Observe if secret exists), connection scope upgrade
3. **T-NEW-3: CLI token management** — `termlink token create/list` subcommands, `client.authenticate()` method

## Dialogue Log

(No human dialogue yet — inception started autonomously based on security gap register and task backlog priority.)
