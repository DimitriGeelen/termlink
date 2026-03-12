# T-010: Interactive Program Handling — Exploration Report

> Generated: 2026-03-12 | Source: TermLink agent mesh (explore-T010-r2)

## 1. Current PTY/Exec Implementation

**Two separate execution paths exist:**

- **`pty.rs`** — Full PTY session (`PtySession`): `openpty()` + `fork()` + `setsid()` + `TIOCSCTTY`. Provides read loop with scrollback + broadcast, write (inject), resize (`TIOCSWINSZ`), signal forwarding, and `waitpid`. This is the interactive path.

- **`executor.rs`** — Non-interactive exec (`execute()`): `sh -c` via `tokio::process::Command` with piped stdout/stderr. Captures output as strings. No PTY, no terminal state. Has key resolution (`resolve_key`) for inject sequences.

**`handler.rs`** (`SessionContext`): Holds `Option<Arc<PtySession>>` — PTY is optional. RPC methods like `pty.inject` write to the PTY master, `session.exec` uses the non-PTY executor, `pty.resize` calls `TIOCSWINSZ`.

## 2. What Breaks with Interactive Programs

1. **No terminal mode detection.** Zero `tcgetattr`/termios usage on the PTY master side. TermLink has no idea whether the child is in raw mode (vim) or canonical mode (bash). Agents injecting keystrokes are blind to modal state.

2. **No echo-off detection.** Password prompts (sudo, ssh) disable `ECHO` in termios — TermLink doesn't detect this, so scrollback captures password characters if echoed, or agents don't know to suppress output capture.

3. **Scrollback is raw bytes.** Output goes to `ScrollbackBuffer` as undifferentiated bytes. ANSI escape sequences from vim/less (cursor movement, screen clears, alternate screen buffer) pollute the scrollback, making `session.output` useless for screen content.

4. **No alternate screen buffer awareness.** Programs like vim/less switch to the alternate screen buffer (`\e[?1049h`). TermLink doesn't detect this transition — `session.output` returns the alternate buffer's raw escape soup instead of meaningful content.

5. **`session.exec` is non-interactive.** Uses piped stdio (not PTY) — programs that need a TTY (vim, less, python REPL) will either fail or run in degraded mode.

6. **No nested PTY awareness.** SSH/tmux inside a TermLink PTY creates nested PTYs. Inject/output still work mechanically (bytes pass through), but output parsing becomes ambiguous (escape sequences from multiple layers).

## 3. What Needs to Change

| Priority | Change | Effort |
|----------|--------|--------|
| **P1** | **Terminal mode query**: Add `tcgetattr()` on PTY master fd to read slave termios flags. Expose via new RPC `pty.mode` returning `{canonical, echo, raw}` | Small |
| **P2** | **Mode-change events**: Poll termios periodically or on inject, emit `pty.mode-change` event when flags change (raw↔cooked, echo on↔off) | Medium |
| **P3** | **Alternate screen detection**: Scan output stream for `\e[?1049h/l` sequences, track state, expose via `pty.mode` response | Medium |
| **P4** | **Password prompt hint**: When `ECHO` flag drops, emit event so agents know to avoid capturing/logging output | Small (if P1 done) |
| **Optional** | ANSI-aware scrollback (vt100 state machine) for screen content queries | Large — likely out of scope |

**Key insight:** The PTY master fd allows `tcgetattr()` to read the slave's terminal settings. This is low-overhead and reliable for detecting raw/canonical/echo states. The main gap is that TermLink never calls it. Adding a `pty.mode` RPC + mode-change events would let agents adapt to interactive programs without TermLink needing to understand program semantics.
