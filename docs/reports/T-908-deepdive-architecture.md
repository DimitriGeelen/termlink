# T-908 Deep Dive: Relay Architecture, Ecosystem, and Failure Modes

**Task:** T-908 (Worker 3 — Architecture Deep Dive)  
**Date:** 2026-04-09  
**Scope:** Crate placement, Rust ecosystem survey, failure mode analysis, graceful degradation, integration points

---

## 1. Crate Placement Options

### Option A: New crate `termlink-relay` (sibling to termlink-hub)

The relay becomes a first-class workspace member at the same dependency tier as the hub.

**Dependency graph:**
```
termlink-protocol
    ^
termlink-session
    ^         ^
termlink-hub  termlink-relay   (siblings — both depend on protocol + session)
    ^              ^
termlink-mcp       |
    ^              |
termlink-cli ------+           (CLI depends on relay for start/stop commands)
```

**Pros:** Clean separation of concerns. The hub handles JSON-RPC routing over Unix/TCP sockets; the relay handles HTTP/SSE proxying to cloud APIs. Each can evolve independently. The relay gets its own Cargo.toml, its own test suite, its own `hyper`/`reqwest` dependencies without polluting the hub's dependency tree. The hub currently has zero HTTP dependencies (it speaks raw newline-delimited JSON over sockets), so adding an HTTP stack to it would be a mismatch.

**Cons:** Adds a new crate to the workspace. The CLI binary grows (links both hub and relay). Two processes to manage if they run as separate daemons.

**Dependency impact:** The relay needs `termlink-protocol` (for governance types like `GovernanceEvent`) and `termlink-session` (for session discovery, so it can look up active tasks via the hub). It does NOT need `termlink-hub` — the relay queries the hub over its socket like any other client. This keeps the dependency graph acyclic with no new edges between existing crates.

**Verdict: Recommended for long-term architecture.** The relay's responsibilities (HTTP proxying, SSE parsing, TLS to external APIs) are fundamentally different from the hub's (local JSON-RPC routing, session registry). A clean crate boundary prevents dependency sprawl. However, the `termlink-session` dependency pulls in auth/crypto crates the relay does not need for its primary job. Consider starting with Option C (standalone) and graduating to Option A when hub integration justifies the dep weight.

### Option B: Module inside termlink-hub

The relay becomes `crates/termlink-hub/src/relay/` with submodules.

**Pros:** No new crate. Shares hub's circuit breaker and bypass registry directly.

**Cons:** The hub has zero HTTP deps today (10 deps in Cargo.toml). Adding HTTP proxying triples that count and pollutes every downstream consumer (`termlink-mcp`, `termlink-cli`). The hub's `server.rs` manages Unix+TCP listeners; adding an HTTP listener creates a chimera.

**Verdict: Not recommended.** The coupling cost exceeds the sharing benefit.

### Option C: Standalone binary (no TermLink crate deps)

A workspace member (`crates/termlink-relay/`) that depends only on external crates (hyper, reqwest, serde_json, tokio). Communicates with TermLink exclusively via JSON-RPC over the hub's Unix socket -- the same way any external client would.

**Pros:** Maximum isolation. Can be built and tested without compiling any other TermLink crate. Deployment flexibility: can run without a hub at all (standalone governance, log to file). Crash isolation: separate process, no shared memory. Dependency weight is additive only (HTTP crates), not multiplicative (no auth/HMAC/crypto pulled in from session).

**Cons:** Must duplicate JSON-RPC message construction (~30 lines) and `runtime_dir()` path resolution (~10 lines). No access to typed `GovernanceEvent` or `TransportAddr`; constructs JSON manually. Must independently discover the hub socket path (trivial: `$XDG_RUNTIME_DIR/termlink/hub.sock` or `$TERMLINK_RUNTIME_DIR/hub.sock`).

**Verdict: Recommended for Phase 0-1 (MVP), with graduation to Option A when hub integration matures.** The relay's primary job (HTTP proxy + SSE rewrite) is orthogonal to TermLink's session/data-plane protocol. A 40-line helper module covering JSON-RPC calls and runtime dir discovery replaces the `termlink-session` dependency without coupling. If governance rules eventually need live-push updates from the hub (rather than config-file reload on SIGHUP), promote to Option A by adding `termlink-protocol` for typed events. Defer `termlink-session` dependency unless the relay needs to register as a session itself.

---

## 2. Rust Ecosystem for SSE/HTTP Proxying

### HTTP Server (inbound from Claude Code)

| Crate | Fit | Notes |
|-------|-----|-------|
| **hyper 1.x** | Best for raw SSE rewriting | Low-level, full streaming body control, ~15 deps |
| **axum 0.8** | Good for structured routing | Built on hyper + tower, adds ~10 deps, ergonomic middleware |
| warp | Avoid | Less maintained, poor SSE ergonomics |
| actix-web | Avoid | Own runtime, conflicts with tokio |

**Recommendation: axum.** The relay needs 2-3 routes (`POST /v1/messages`, `GET /health`). axum's tower middleware (logging, timeouts, header redaction) justifies the modest dep increase over raw hyper, while still exposing the hyper `Body` for streaming control.

### HTTP Client (outbound to api.anthropic.com)

**Recommendation: reqwest 0.12** with `rustls-tls` feature. Handles TLS, connection pooling, and HTTP/2 transparently. `response.bytes_stream()` yields `Stream<Item=Result<Bytes>>` for SSE parsing. The alternative (hyper-util client) requires manual TLS setup and connection management for no benefit.

### SSE Parsing

| Approach | Fit |
|----------|-----|
| **eventsource-stream** | Best -- wraps `Stream<Bytes>` into `Stream<Event>`, handles multi-line data, keepalives |
| **Manual (~50 lines)** | Viable for MVP since Anthropic uses only `event:` + `data:` lines |
| sse-codec | Lower-level, requires manual framing |

**Recommendation: eventsource-stream** for correctness. Hand-roll only if dependency minimalism is paramount. Pipeline: `reqwest stream -> eventsource-stream -> serde_json parse -> governance filter -> reserialize -> axum response stream`.

### TLS and Streaming

- **TLS:** Reuse workspace's `rustls 0.23` via reqwest's `rustls-tls` feature. Add `webpki-roots` for Mozilla CA bundle (the hub's self-signed TOFU model does not apply here -- the relay talks to a public API).
- **Streaming:** `tokio-stream` (transitive) and `futures-core` (transitive) are already available.

### Recommended Stack

```
axum 0.8           (HTTP server)
reqwest 0.12       (HTTP client, rustls backend)
eventsource-stream (SSE parsing)
serde_json         (already in workspace)
tokio-stream       (already transitively present)
```

**New workspace deps:** `axum`, `reqwest`, `eventsource-stream`. This is the first HTTP framework in the workspace, roughly doubling the external dependency count.

---

## 3. Relay Failure Modes

| ID | Failure | Severity | Notes |
|----|---------|----------|-------|
| **FM-R1** | **Relay crash** | Critical | `ANTHROPIC_BASE_URL` points at nothing. Claude Code gets connection refused on every API call. No fallback -- fail-closed by design. |
| **FM-R2** | **SSE parsing bug** | High | Malformed events reach Claude Code (garbage response) or parser hangs waiting for `\n\n` that never arrives. Hard to debug -- users blame Anthropic. |
| **FM-R3** | **Upstream TLS changes** | Medium | api.anthropic.com cert rotation or cipher suite changes. Standard rotations handled by webpki-roots; unusual changes (client certs, CT enforcement) could break silently. |
| **FM-R4** | **SSE format changes** | High | Anthropic adds event types, changes `content_block_start` schema, or alters `input_json_delta` encoding. No versioning contract exists. Could cause corruption (FM-R2) or silent governance bypass. |
| **FM-R5** | **Latency overhead** | Low | Each SSE chunk: TCP receive -> parse -> governance check -> reserialize -> TCP send. Adds <1ms on localhost; network RTT to api.anthropic.com (50-200ms) dominates. |
| **FM-R6** | **Memory pressure** | Low-Medium | Buffering `input_json_delta` for content-based gating. Per-block: typically <10KB. With 5 concurrent subagents: ~50KB peak. Cap at 1MB per block; exceeded = fail-open. |
| **FM-R7** | **Concurrent stream limits** | Low | One downstream + one upstream connection per active stream. tokio handles thousands. Practical limit: Anthropic's rate limits (5-50 concurrent per key), not relay capacity. |

**Mitigations for critical/high modes:**

- **FM-R1:** Health-check bypass (wrapper probes relay before setting env var) + supervisor restart (<3s) + pidfile reuse from `pidfile.rs`.
- **FM-R2:** Fuzz testing with recorded API responses. Byte-for-byte comparison between relay passthrough output and direct API output. Unknown events forwarded verbatim.
- **FM-R4:** Forward unknown event types as-is (pass-through). Log unknown types for early detection. Pin governance parsing to `anthropic-version` header value.

---

## 4. Graceful Degradation Strategy

The relay follows a **fail-open with audit** philosophy: it enhances governance but must never make Claude Code worse than running without a relay.

| Condition | Behavior | Signal |
|-----------|----------|--------|
| Relay healthy, hub reachable | Full governance enforcement | `GET /health` returns `{"governance": "active"}` |
| Relay healthy, hub unreachable | Forward all traffic + log warning | `{"governance": "degraded"}` |
| Governance config corrupt/missing | Passthrough mode (`AtomicBool` flag on hot path) | `{"governance": "passthrough"}` + relay log error |
| Unknown SSE event type | Forward verbatim + log | Yellow: early detection of API changes |
| SSE parse error | Forward raw bytes + log | Yellow: parsing regression signal |
| Relay crash | Claude Code gets connection refused | Red: wrapper script falls back to direct API URL |

**Automatic bypass:** The startup wrapper probes `GET /health` before setting `ANTHROPIC_BASE_URL`. If the relay is down or unhealthy, the wrapper skips it and Claude Code talks directly to Anthropic.

**Kill switch:** `unset ANTHROPIC_BASE_URL && restart claude` (instant), `termlink relay stop` (graceful with 30s drain), or `termlink relay start --passthrough` (relay stays up, forwards everything uninspected).

**Clean shutdown:** On SIGTERM, stop accepting new connections, drain in-flight streams until `message_stop` or 30s deadline, remove pidfile. Mirrors the hub's `watch::channel` shutdown pattern in `server.rs`.

---

## 5. Integration Points with Existing TermLink

**Task awareness:** The relay queries the hub via `session.discover` JSON-RPC to find active tasks. The hub is the source of truth for session state. Task lookup is not on the hot path -- it happens once per `content_block_start` that matches a gated tool name, not per SSE chunk. If the hub is unreachable, the relay fails open (allows the tool call + emits warning).

**Reporting blocked actions:** Three channels, in priority order: (1) TermLink event bus via `event.emit_to` (real-time, other sessions see it), (2) `GovernanceEvent` structs through the data plane if connected (integrates with existing governance subscriber), (3) JSON-lines audit log at `runtime_dir()/relay-audit.jsonl` (persistent, survives restarts).

**CLI integration:** New subcommand group: `termlink relay start|stop|status|rules`. Same pattern as existing command modules in `termlink-cli`. Uses pidfile mechanism from `crates/termlink-hub/src/pidfile.rs`. Startup wrapper: `termlink relay start --port 8080 && export ANTHROPIC_BASE_URL=http://localhost:8080`.

---

## 6. Build Effort Estimate

| Phase | Scope | Effort | Key deliverables |
|-------|-------|--------|------------------|
| **0** | Pass-through proxy | 1 day | axum on `/v1/messages` + `/health`, reqwest forwarding. Zero governance. Verify: `ANTHROPIC_BASE_URL=http://localhost:8080 claude` works identically. |
| **1** | SSE parsing + tool name gating | 3-4 days | eventsource-stream, `content_block_start` detection, YAML blocklist/allowlist, block action (suppress + inject text block), structured logging. |
| **2** | Content-based gating | 3-4 days | `input_json_delta` accumulation, JSON path extraction on `content_block_stop`, glob/regex path rules, 1MB buffer cap (exceeded = fail-open). |
| **3** | Model routing | 2-3 days | Request classification, route rules to model override, `model` field rewrite, multi-provider upstream. |
| **4** | Hub integration + observability | 3-4 days | JSON-RPC hub events (`relay.tool_blocked`, `relay.model_routed`), task context queries, audit log, `/metrics` endpoint. |

**Total: 12-16 working days (~3-4 weeks).** Phase 0+1 delivers a usable governance relay in ~1 week. Phase 0 alone (1 day) provides a logging/debugging proxy.

---

## 7. Risk Matrix

### Technical Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|------------|
| TR-1 | SSE parser bug drops/corrupts events mid-stream | Medium | High -- Claude Code receives malformed response, may crash or hallucinate | Fuzz testing with recorded API responses. Integration test: compare relay output byte-for-byte with direct API output in pass-through mode. |
| TR-2 | Stream rewriting breaks Claude Code's state machine | Medium | High -- Claude Code hangs, crashes, or loops. Replacement text block must have correct index, matching start/stop pair. | Record real Claude Code sessions. Replay through relay with gating enabled. Verify Claude Code processes modified stream without error. |
| TR-3 | Latency exceeds perceptibility threshold (>50ms p99) | Low | Medium -- user perceives sluggish typing speed | Benchmark Phase 0 immediately. hyper + tokio overhead is characterized at <1ms. Network RTT to api.anthropic.com dominates. |
| TR-4 | TLS handshake failures with upstream | Low | High -- all API calls fail | Use reqwest's well-tested rustls backend with webpki-roots (Mozilla CA bundle). Support `SSL_CERT_FILE` / `SSL_CERT_DIR` for corporate proxy environments. |
| TR-5 | tokio runtime panic (poisoned mutex, stack overflow) | Very low | High -- relay crash (FM-R1) | No shared mutable state between connections. Use `Arc<Config>` (read-only after load). Governance state is per-connection, not global. Supervisor auto-restart. |

### Operational Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|------------|
| OR-1 | Relay as single point of failure | High (by design) | High | Health-check bypass + supervisor restart (<3s) |
| OR-2 | Port conflict | Medium | Medium | Configurable port. Consider `ANTHROPIC_UNIX_SOCKET` to avoid port conflicts entirely. |
| OR-3 | Relay not started before Claude Code | Medium | High | Startup wrapper: `termlink relay start && export ANTHROPIC_BASE_URL=... && claude` |
| OR-4 | Relay obsoleted by native Anthropic toolGate | Medium | Low | Modular design: governance is one middleware layer. Logging/routing remain valuable. |

### Coupling Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|------------|
| CR-1 | Anthropic changes SSE event format | Low (breaking change) | High -- governance stops working, may corrupt streams | Forward unknown event types as-is (pass-through). Log unknown types for early detection. Pin to `anthropic-version` header value. |
| CR-2 | Anthropic adds new tool types (beyond tool_use, server_tool_use) | Medium | Medium -- new tool types bypass governance | Allowlist approach (only known-safe types pass) catches new types by default. Blocklist approach misses them. Decision needed on which model. |
| CR-3 | Claude Code stops respecting ANTHROPIC_BASE_URL | Very low | Critical -- relay becomes useless | This env var is part of the Anthropic SDK contract. Breaking it would break every enterprise proxy. Low risk. |
| CR-4 | reqwest/hyper/axum major version bumps | Low | Low -- compile error, fixable | Pin major versions in Cargo.toml. Standard Rust practice. |

### Security Risks

| ID | Risk | Likelihood | Impact | Mitigation |
|----|------|-----------|--------|------------|
| SR-1 | API key exposure in logs | High (if not careful) | Critical -- account compromise, billing abuse | NEVER log request headers containing `x-api-key` or `Authorization`. Redaction middleware in axum tower layer. CI audit of logging code. |
| SR-2 | Relay listens on 0.0.0.0 instead of 127.0.0.1 | Medium (misconfiguration) | High -- API key accessible from network, anyone can proxy through relay | Default bind to `127.0.0.1`. Refuse `0.0.0.0` without explicit `--allow-external` flag and warning. |
| SR-3 | Governance rules tampered with by model | Low | High -- model disables its own governance | Config file permissions 0600. SIGHUP reload logs the diff between old and new rules. Optional: `chattr +i` on governance config. |
| SR-4 | Relay used as open proxy | Low (localhost-only by default) | High -- if exposed externally, forwards any request to Anthropic with user's API key | Bind localhost only. Rate limit per source IP. Require a relay-specific auth token for non-localhost connections. |

---

## Summary

The relay fits cleanly as a new `termlink-relay` workspace crate. The recommended approach is to start standalone (Option C: no TermLink crate dependencies, hub communication via raw JSON-RPC) and graduate to Option A (depend on `termlink-protocol` + `termlink-session`) when hub integration matures. The Rust stack is axum (server) + reqwest (client) + eventsource-stream (SSE parsing), adding three new workspace-level dependencies. This is the first HTTP framework in the workspace, roughly doubling the external dependency count -- a significant but justified expansion given that no existing TermLink crate speaks HTTP.

The relay has seven identified failure modes. The two highest-severity are relay crash (FM-R1, mitigated by health-check bypass and supervisor restart) and SSE format drift (FM-R4, mitigated by pass-through on unknown events). The risk matrix identifies 16 risks across four categories; the most critical are API key logging (SR-1, mitigated by redaction middleware) and stream rewriting desync (TR-2, mitigated by recorded-session replay testing).

Build effort is 12-16 working days (~3-4 weeks) for the full feature set. A usable governance relay (pass-through + tool name gating) is achievable in ~1 week (Phase 0+1). Phase 0 alone (1 day) delivers a valuable logging/debugging proxy.

## Open Questions

1. **Fail-open vs fail-closed default?** When governance evaluation fails, should the relay forward the event (fail-open, preserving usability) or block it (fail-closed, preserving safety)? This is a policy decision, not a technical one.
2. **Allowlist vs blocklist for tool names?** Blocklist is easier to start with (block known-dangerous tools). Allowlist is more secure (only permit known-safe tools). Which model fits the governance philosophy?
3. **Governance rule format.** YAML, TOML, or inline in relay config? Should rules support hot-reload (file watch / SIGHUP) or require process restart?
4. **Non-streaming requests.** Claude Code also makes non-streaming requests (e.g., `count_tokens`). Should the relay proxy all `/v1/*` endpoints or only `/v1/messages` with `stream: true`?
5. **API key isolation.** If multiple sessions use different API keys, the relay must route by key. Does the relay maintain a key-to-session mapping, or is it transparent (forward whatever key the client sends)?
6. **`ANTHROPIC_UNIX_SOCKET` support.** The SDK supports Unix socket transport. Should the relay listen on a Unix socket (lower latency, no port conflicts) in addition to or instead of TCP?
7. **ccproxy hybrid path.** The landscape survey (T-908 Spike 3) identified ccproxy as a Python-based starting point. Should the Rust relay be built from scratch, or prototype in Python (extend ccproxy) and port later?
8. **Metrics cardinality.** Should per-tool-name metrics be unbounded (one counter per tool name seen) or bucketed (known tools + "other")? Unbounded risks label explosion if models invent novel tool names.
