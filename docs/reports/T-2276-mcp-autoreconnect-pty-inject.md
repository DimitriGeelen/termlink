# T-2276 — Auto-reconnect MCP via PTY-inject after binary upgrade (inception)

**Date:** 2026-06-24
**Status:** exploration (DEFER pending evidence)
**Question (one):** After a `termlink` binary upgrade, can we make a session
self-activate its new MCP server by PTY-injecting a reconnect command into the
Claude Code TUI — eliminating the manual `/mcp reconnect` keystroke?

## Why this matters

The T-2267..T-2270 deploy proved the last-mile gap: the CLI picks up a new
binary immediately, but each session's `termlink mcp serve` child keeps running
the OLD binary until that session reconnects MCP. Today that reconnect is a
human keystroke per session. If a deploy could drive it, the upgrade would
*heal its own last mile* (antifragility / reliability — no silent stale-MCP).

## Assumptions to validate (go/no-go gates) — map to IW-1/2/3 in the task

- **A1/IW-1 — injectable session.** The Claude Code session is a registered
  termlink session with a PTY we can `termlink inject` into.
- **A2/IW-2 — drivable reconnect command.** Claude Code accepts a
  *non-interactive* MCP reconnect a single injected line can trigger. If `/mcp`
  is only an arrow-key menu, a single injected string cannot drive it reliably.
- **A3/IW-3 — safe + effective.** Injection submits cleanly (newline/bracketed-
  paste) AND reconnecting MCP mid-turn does not corrupt in-flight state.

## Hard constraints

- `.107` is **multi-tenant**: target only the intended session, never blast
  other sessions' PTYs.
- Injecting into one's OWN session is recursive — reason about whether the
  reconnect interrupts the very turn doing the injecting.
- No build artifacts before a GO (inception discipline).

## Findings

(filled during exploration)

### A1/IW-1 — injectable session?

### A2/IW-2 — drivable reconnect command?

### A3/IW-3 — safe + effective?

## Decision

DEFER -> GO/NO-GO after evidence.

## Dialogue Log

- 2026-06-24 — User: "can we incept and experiment, to lookup session and pty
  inject '/mcp reconnect'". Framed as inception; created T-2276 + this artifact.
