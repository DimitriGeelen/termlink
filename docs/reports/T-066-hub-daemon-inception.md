# T-066: Hub as Persistent Daemon — Inception Research

## Current State Analysis

The hub (`termlink hub` CLI command) is a **stateless routing service**:
- Binds to `$RUNTIME_DIR/hub.sock`, accepts JSON-RPC connections
- Routes: `session.discover`, `event.broadcast`, `event.collect`, forward-to-target
- Session registry is **file-based** (reads `sessions/*.json` on every discover call)
- Event stores live **in sessions**, not in the hub
- Shutdown: `Ctrl+C` handler removes socket file, no graceful drain
- No pidfile, no signal handling, no supervision

### Key Insight: Hub is Lightweight

Unlike a message broker (RabbitMQ, NATS), the hub holds **no persistent state**. It:
1. Reads session registry from disk (always fresh)
2. Forwards requests to session sockets (pass-through)
3. Fan-out/fan-in for broadcast/collect (ephemeral)

This makes daemon extraction **low risk** — crash recovery is just "restart and re-read the session directory."

## Spike 1: Pidfile Management

### Design

```
$RUNTIME_DIR/hub.pid    # Contains PID as ASCII text
```

**Lifecycle:**
1. On start: check for existing pidfile → validate PID liveness → remove if stale → write new PID
2. On shutdown (SIGTERM/SIGINT): remove pidfile → remove socket → exit
3. On crash: pidfile remains → next start detects stale → cleans up

**Existing patterns to reuse:**
- `liveness.rs`: `is_process_alive(pid)` via `kill(pid, 0)` — reusable for pidfile validation
- `discovery.rs`: Runtime dir resolution — reusable for pidfile path
- `registration.rs`: Already stores PID in session JSON — same pattern for hub

### Complexity Assessment: LOW
- ~50 lines for pidfile read/write/validate
- No new dependencies needed

## Spike 2: Graceful Shutdown

### Design

```
SIGTERM/SIGINT → set shutdown flag → stop accepting new connections
  → drain active connections (with timeout) → remove pidfile → remove socket → exit 0
```

**Implementation:**
- `tokio::signal::ctrl_c()` already used in CLI (main.rs:2696)
- Add `tokio::signal::unix::signal(SignalKind::terminate())` for SIGTERM
- Use `tokio::select!` with shutdown channel to drain accept loop
- Timeout: 5 seconds for drain, then force-close

### Complexity Assessment: LOW
- Tokio signal handling is well-documented
- Accept loop already structured for select! (server.rs)
- ~30 lines to add SIGTERM handler + drain logic

## Spike 3: Session Supervision

### Current Liveness Detection

Sessions are detected as stale by:
1. `kill(pid, 0)` — process alive check
2. Socket file existence check
3. Cleanup runs on **next `session.discover` call** (lazy)

### Proposed: Active Supervision Loop

```
loop {
    sleep(30s)
    for session in list_sessions() {
        if !is_alive(session.pid) {
            log::warn!("Session {} died, cleaning up", session.id)
            cleanup_stale_session(session)
            emit_event("session.died", session.id)  // optional
        }
    }
}
```

### Value Assessment

| Approach | Detection Latency | Complexity |
|----------|------------------|------------|
| Current (lazy) | Until next discover call | None |
| Active supervision (30s) | 30 seconds max | ~20 lines |
| Heartbeat-based | Configurable | Higher (requires session changes) |

**Verdict:** Active supervision at 30s interval is the sweet spot. Low complexity, meaningful improvement over lazy detection.

## Research: Service Manager Integration

### What Similar Tools Do

| Tool | Daemon Strategy |
|------|----------------|
| tmux server | Auto-starts on first client, persists, exits when no sessions |
| Docker daemon | systemd/launchd service, pidfile, socket activation |
| SSH agent | Background process, socket path in env var |
| Mosh server | Per-connection process, no central daemon |

### TermLink Best Fit: tmux-like

The hub should **auto-start when needed, persist, exit when idle**:
- `termlink hub start` — explicit start (foreground or `--daemonize`)
- `termlink hub stop` — graceful shutdown via SIGTERM to PID from pidfile
- `termlink hub status` — check if running (pidfile + liveness)
- Auto-start: Any command that needs the hub checks if running, starts if not

### Service Files (Phase 2, out of scope for now)
- launchd plist: straightforward, KeepAlive=true
- systemd unit: Type=simple, ExecStart=/path/to/termlink hub start
- Both require installed binary path — defer until packaging story

## Go/No-Go Analysis

### GO Criteria Assessment

| Criterion | Result |
|-----------|--------|
| Clean extraction (no protocol changes) | **YES** — hub is already a separate crate, routing is stateless |
| Pidfile + SIGTERM works reliably | **YES** — standard Unix patterns, tokio signal support exists |
| Session supervision adds value | **YES** — 30s detection vs. "whenever someone calls discover" |

### NO-GO Criteria Assessment

| Criterion | Result |
|-----------|--------|
| Hub state too complex for in-memory | **NO** — hub is stateless, reads from disk |
| Daemon complexity exceeds benefit | **NO** — ~100 lines total for pidfile + signal + supervision |

### Recommendation: **GO**

**Rationale:**
1. Hub is stateless → crash recovery is just restart
2. Implementation is ~100 lines across 3 features (pidfile, SIGTERM, supervision)
3. Addresses gap G-004 (hub as CLI subcommand = single point of failure)
4. tmux-like auto-start pattern fits TermLink's usage model
5. No protocol changes, no session API changes needed

### Proposed Build Tasks

1. **T-080: Hub pidfile + start/stop/status commands** — pidfile lifecycle, `hub start --daemonize`, `hub stop`, `hub status`
2. **T-081: Hub graceful shutdown** — SIGTERM/SIGINT handler, connection drain with timeout, socket cleanup
3. **T-082: Hub session supervision loop** — Active liveness polling at 30s, stale cleanup, optional event emission

## Assumptions Validation

- **A1 (persistent daemon more reliable):** VALIDATED — stateless hub means restart is cheap, pidfile prevents double-start
- **A2 (supervision requires persistence):** VALIDATED — active polling loop needs a long-running process
- **A3 (clean extraction):** VALIDATED — `termlink-hub` crate is already separate, only CLI entry point changes
- **A4 (launchd/systemd feasible):** VALIDATED but DEFERRED — straightforward but depends on packaging/installation story
