# Framework Pickup Prompt — claude-fw --termlink Flag

> Paste everything below the line into a Claude Code session in the framework project.

---

I want to add a `--termlink` flag to `claude-fw` so the Claude Code session auto-registers as a TermLink session, enabling remote observation and input injection.

**What this enables:**
- Monitor the master Claude Code session from another terminal: `termlink attach claude-master`
- Inject input from another machine via TCP hub: `termlink pty inject claude-master "text" --enter`
- Workers discover the master: `termlink discover --tag master`
- Cross-machine access when hub has TCP listener

**TermLink binary:** `/Users/dimidev32/.cargo/bin/termlink` (already installed)
**Repo:** `https://github.com/DimitriGeelen/termlink`

**Install / upgrade:**
```bash
cargo install --git https://github.com/DimitriGeelen/termlink.git termlink --force
```

## What to change

### 1. `bin/claude-fw` — Add `--termlink` flag (~15 lines)

Current launch (around line 66):
```bash
command claude "${CLAUDE_ARGS[@]}"
```

Change to:
```bash
if [ "$TERMLINK_ENABLED" = "1" ] && command -v termlink >/dev/null 2>&1; then
    # Persistent mode: shell session survives claude restart
    SESSION_NAME="claude-master-$$"
    if ! termlink list 2>/dev/null | grep -q "$SESSION_NAME"; then
        termlink spawn --name "$SESSION_NAME" --tags "master,claude,framework" \
            --backend auto --shell --wait --wait-timeout 15
    fi
    sleep 1
    termlink pty inject "$SESSION_NAME" "claude ${CLAUDE_ARGS[*]}" --enter >/dev/null 2>&1

    # Wait for claude to finish (poll for shell prompt return)
    while termlink list 2>/dev/null | grep -q "$SESSION_NAME"; do
        sleep 5
    done
else
    command claude "${CLAUDE_ARGS[@]}"
fi
```

**Flag parsing** — add before the main loop:
```bash
TERMLINK_ENABLED=0
# Check for --termlink flag
for arg in "$@"; do
    if [ "$arg" = "--termlink" ]; then
        TERMLINK_ENABLED=1
    fi
done
# Also check env var
[ "${TL_CLAUDE_ENABLED:-0}" = "1" ] && TERMLINK_ENABLED=1
```

**Restart handling** — when auto-restart fires, re-inject into existing session:
```bash
# In the restart loop, replace `command claude "${CLAUDE_ARGS[@]}"` with:
if [ "$TERMLINK_ENABLED" = "1" ]; then
    termlink pty inject "$SESSION_NAME" "claude -c" --enter >/dev/null 2>&1
else
    command claude "${CLAUDE_ARGS[@]}"
fi
```

**Cleanup** — add before exit:
```bash
if [ "$TERMLINK_ENABLED" = "1" ] && [ -n "$SESSION_NAME" ]; then
    termlink pty inject "$SESSION_NAME" "exit" --enter >/dev/null 2>&1 || true
    sleep 2
    termlink clean 2>/dev/null || true
fi
```

### 2. CLAUDE.md — Add usage section

```markdown
## Remote Session Access (TermLink)

Launch with TermLink wrapping for remote access:

    claude-fw --termlink        # Register session for remote access
    TL_CLAUDE_ENABLED=1 claude-fw  # Same via env var

From another terminal:

    termlink list                              # See the session
    termlink attach claude-master-<PID>        # Full TUI mirror (bidirectional)
    termlink pty output claude-master-<PID> --strip-ansi  # Read recent output

TermLink is optional. Without it installed, `--termlink` is silently ignored.
```

### Key design decisions

- **Opt-in only** — `--termlink` flag or `TL_CLAUDE_ENABLED=1` env var. No behavior change by default.
- **Graceful fallback** — if `termlink` binary not on PATH, falls back to normal `command claude`
- **Session name includes PID** — avoids collisions with multiple instances
- **Persistent mode** — uses `--shell` so session survives restart cycle
- **No TermLink code changes needed** — uses existing `spawn`, `inject`, `clean` commands

### Testing

1. `claude-fw --termlink` — verify Claude starts and session appears in `termlink list`
2. From another terminal: `termlink attach <session>` — verify TUI mirrors
3. In Claude, run `/exit` — verify session persists
4. Check auto-restart injects new claude into same session
5. `claude-fw` without `--termlink` — verify no change in behavior
