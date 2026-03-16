# T-149: Platform-Aware Spawn — Research Artifact

> Status: research complete, ready for GO/NO-GO
> Created: 2026-03-16

## Problem Statement

TermLink's `spawn` and dispatch are hardcoded to macOS Terminal.app via osascript.
Framework runs on headless Linux. Workers can't be spawned. Cross-platform is a
requirement, not a nice-to-have (Linux first-class, macOS, Windows/WSL later).

## Key Finding: Blast Radius is Tiny

**Only 3 touch points contain osascript:**
1. `cmd_spawn()` in `main.rs:2930-3044` — ~50 lines of osascript
2. `tl-dispatch.sh:127` — one osascript spawn line
3. `tl-dispatch.sh:329-385` — 3-phase osascript cleanup

**Everything else is already cross-platform:**
- PTY allocation: raw `libc::openpty()` + `fork()` + `execvp()` — pure POSIX
- PTY I/O: `AsyncFd` read + `Arc<Mutex<OwnedFd>>` write — generic
- `register --shell`: allocates PTY in-process, no terminal emulator needed
- `pty inject/output`: RPC over Unix socket, no terminal dependency
- Events, hub, discovery, protocol: all platform-agnostic

## Critical Insight: TermLink IS the Multiplexer

TermLink's `register --shell` already creates a self-sufficient PTY:
- Allocates master/slave pair via `openpty()`
- Forks shell process connected to slave
- Manages scrollback buffer, mode detection, resize
- Accepts input via `pty.inject` RPC, returns output via `pty.output` RPC

**The terminal emulator (Terminal.app, tmux) is optional — it's for human
observation, not a technical requirement for agent dispatch.** Agents interact
via RPC, not keyboard/screen.

## Three Spawn Backends

| Backend | Platform | How | Cleanup | Human-observable? |
|---------|----------|-----|---------|-------------------|
| **Terminal.app** | macOS + GUI | `osascript` | 3-phase (fragile) | Yes (window) |
| **tmux** | Linux/macOS | `tmux new-session -d` | `tmux kill-session` (one command) | Yes (`tmux attach`) |
| **Background PTY** | Universal | `setsid termlink register --shell` | `kill <pid>` | No (headless only) |

### tmux Backend (recommended for headless)

All 10 research questions answered YES:
- Detached spawn: `tmux new-session -d -s name "termlink register --name X --shell"`
- Input injection: works via `termlink pty inject` (RPC, bypasses tmux entirely)
- Output capture: works via `termlink pty output` (scrollback, not tmux pane)
- Cleanup: `tmux kill-session -t name` — single command, clean
- Persistence: sessions survive parent exit (tmux's core feature)
- Nesting detection: `$TMUX` environment variable
- Human observation: `tmux attach -t name` for live view

**Design decision:** Skip nested PTY. Use tmux purely as a process host.
TermLink's own PTY inside the tmux session handles all I/O. tmux is just
the container that keeps the process alive and provides human attach capability.

### Background PTY Fallback (no multiplexer)

For environments without tmux or Terminal.app:
- `setsid termlink register --name X --shell &` — daemonize with PTY
- Track PID in registration file (already exists)
- Cleanup: `kill <pid>` + `termlink clean`
- No human observation possible (headless-only)

## Backend Selection Strategy

```
Runtime detection (in cmd_spawn):

1. --backend flag explicitly set? → use that
2. macOS + GUI (WindowServer running)? → Terminal.app
3. tmux available? → tmux
4. Fallback → background PTY (setsid)
```

Or simpler: `termlink spawn --backend tmux|terminal|background|auto`

## Assumptions Validated

- A1: tmux available on headless — **VALID** (standard on Linux)
- A2: tmux hosts TermLink PTY — **VALID** (TermLink PTY is independent)
- A3: inject/output through tmux — **VALID** (RPC-based, tmux-agnostic)
- A4: platform-aware spawn — **VALID** (Rust `#[cfg]` + runtime detection)
- A5: background PTY viable — **VALID** (`register --shell` already does it)

## Build Task Decomposition (if GO)

| Task | Scope | Effort |
|------|-------|--------|
| **B1** | Refactor `cmd_spawn()` — extract backend trait/enum, keep Terminal.app as default | Small |
| **B2** | tmux backend — `spawn_via_tmux()` + cleanup | Small |
| **B3** | Background PTY backend — `spawn_via_background()` + cleanup | Small |
| **B4** | `--backend` CLI flag + auto-detection logic | Small |
| **B5** | Update `tl-dispatch.sh` — delegate to `termlink spawn`, simplify cleanup | Small |
| **B6** | Cross-platform CI test (Linux + macOS) | Medium |

Total: ~5-6 small tasks, each fits in one session.

## GO/NO-GO Assessment

**GO criteria — all met:**
- tmux backend proven (all capabilities confirmed)
- Background PTY fallback viable (already implemented in pty.rs)
- Existing macOS behavior unaffected (backward compatible via detection)
- Blast radius bounded (spawn command + dispatch script only)
- No protocol/session layer changes needed

**NO-GO criteria — none triggered:**
- No nested PTY issues (TermLink's PTY is independent)
- Platform detection is straightforward (cfg + runtime)
- No protocol internals affected

## Dialogue Log

### Q1: Initial scoping (2026-03-16)
**Human:** Framework on headless server can't spawn workers.
**Agent:** Proposed inception.
**Human:** Approved multi-agent inception. Corrected faulty assumption — Linux/macOS
are both first-class. Windows/WSL is future. Cross-platform was always the intent.
