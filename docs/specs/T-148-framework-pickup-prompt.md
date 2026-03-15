# Framework Pickup Prompt — TermLink Integration

> Paste everything below the line into a Claude Code session in the framework project.

---

I want to integrate TermLink into the framework. TermLink is a cross-terminal session communication tool we built — it's our project, hosted at `https://onedev.docker.ring20.geelenandcompany.com/termlink`. It's already installed at `/Users/dimidev32/.cargo/bin/termlink` (v0.1.0, 26 commands), built from source via `cargo install --path .`. Battle-tested with 264 passing tests across 4 crates.

**Repo:** `https://onedev.docker.ring20.geelenandcompany.com/termlink`
**Install:** `git clone https://onedev.docker.ring20.geelenandcompany.com/termlink && cd termlink && cargo install --path crates/termlink-cli`
**Binary:** `/Users/dimidev32/.cargo/bin/termlink`

## What TermLink already provides (DO NOT rebuild these)

TermLink is a Rust binary with 26 commands. Run `termlink --help` to see them all. Key ones:

```
termlink register     # Register a session (--shell for interactive, --name, --tag)
termlink list         # List sessions (--json)
termlink interact     # Run command in PTY session, wait for completion, return output (--json)
termlink discover     # Find sessions by tag/role/name (--json)
termlink event emit   # Emit event to session
termlink event wait   # Wait for event on session (--topic, --timeout)
termlink event broadcast  # Fan-out to all listeners
termlink pty inject   # Send input to PTY (fire-and-forget, --enter)
termlink pty output   # Read terminal output (--strip-ansi)
termlink status       # Session details (--json)
termlink spawn        # Spawn command in new terminal with session registration
termlink hub start    # Start hub server (--tcp for cross-machine)
termlink run          # Ephemeral session: register, execute, deregister
termlink clean        # Remove stale session registrations
termlink kv           # Per-session key-value store
termlink token        # Capability-based auth tokens
```

Every query command supports `--json`. Exit codes are semantic (0=success, 1=timeout/not-found).

TermLink also has a working **dispatch script** that spawns `claude -p` workers in real Terminal.app windows. This script is tested and working — adapt it, don't rewrite it. The full source is included below.

## What the framework needs to build (Phase 0)

### 1. `fw doctor` check

Add to the optional tools section of `doctor.sh`:

```bash
if command -v termlink >/dev/null 2>&1; then
    version=$(termlink --version 2>/dev/null | head -1)
    echo -e "  ${GREEN}OK${NC}  TermLink ($version)"
else
    echo -e "  ${YELLOW}WARN${NC}  TermLink not installed (cargo install termlink)"
    warnings=$((warnings + 1))
fi
```

WARN not FAIL — TermLink is optional. Include install hint.

### 2. Create `agents/termlink/AGENT.md`

This should document when and how to use TermLink from framework agents. Include the primitives table:

| Command | Purpose | Framework Use |
|---------|---------|---------------|
| `termlink interact <session> <cmd> --json` | Run command, get structured output | **Star primitive.** `fw termlink exec` wraps this. |
| `termlink discover --json` | Find sessions by tag/role/name | Worker discovery |
| `termlink event emit/wait/poll` | Inter-session signaling | Coordination backbone |
| `termlink event broadcast <topic> <data>` | Fan-out to all listeners | Multi-worker notification |
| `termlink list --json` | List all sessions | Status overview |
| `termlink status <session> --json` | Session details | Health check |
| `termlink pty output <session> --strip-ansi` | Read terminal output | Log observation |
| `termlink pty inject <session> --enter` | Send input (fire-and-forget) | Long-running command start |
| `termlink register --shell --name X --tag Y` | Create named session | Tagged session lifecycle |
| `termlink hub start [--tcp ADDR]` | Start hub (optional TCP) | Cross-machine coordination |

### 3. Create `agents/termlink/termlink.sh`

This is a **thin wrapper** around the `termlink` binary. It adds framework-specific concerns (task-tagging, budget checks, cleanup tracking) but delegates all real work to the binary. Subcommands:

```
fw termlink check                        # Is termlink on PATH? Print version. Exit 0/1.
fw termlink spawn --task T-XXX [--name N] # Spawn tagged Terminal.app session via osascript + termlink register
fw termlink exec <session> <command>      # Wraps `termlink interact --json`
fw termlink status                        # Wraps `termlink list --json` + annotates with task tags
fw termlink cleanup                       # 3-phase Terminal.app cleanup (SEE BELOW — CRITICAL)
fw termlink dispatch --task T-XXX --name <worker> --prompt "..."
                                          # Spawn claude -p worker in real terminal
fw termlink wait --name <worker> [--all]  # Wait for worker.done event
fw termlink result --name <worker>        # Read worker result file
```

### 4. `fw termlink` route in fw CLI

```bash
termlink)
    exec "$AGENTS_DIR/termlink/termlink.sh" "$@"
    ;;
```

### 5. CLAUDE.md section

Add a TermLink section:
- When to use: self-test, parallel dispatch, observation, remote control
- Available via: `fw termlink <subcommand>` or raw `termlink` CLI
- Budget rule: don't spawn new sessions when context > 60%
- Cleanup rule: always `fw termlink cleanup` before session end
- The `termlink` binary does the heavy lifting — the framework wrapper adds task context

## CRITICAL: Terminal Cleanup — 3-Phase Protocol

**Never close Terminal.app windows directly. We learned this the hard way (twice).**

Direct close kills interactive sessions, loses unsaved work, and leaves orphaned processes.

The correct cleanup:

1. **Phase 1 — Kill child processes via TTY** (spare login/shell):
```bash
tty=$(osascript -e "tell application \"Terminal\" to try
    return tty of tab 1 of window id $wid
end try" 2>/dev/null)
if [ -n "$tty" ]; then
    ps -t "${tty#/dev/}" -o pid=,comm= 2>/dev/null \
        | grep -v -E '(login|-zsh|-bash)' \
        | awk '{print $1}' | xargs kill -9 2>/dev/null || true
fi
```

2. **Phase 2 — Exit shells gracefully:**
```bash
osascript -e "tell application \"Terminal\" to try
    do script \"exit\" in window id $wid
end try" 2>/dev/null || true
```

3. **Phase 3 — Close remaining by tracked window ID (fallback):**
```bash
osascript -e "tell application \"Terminal\"
    set targetIds to {$id_list}
    repeat with w in (reverse of (windows as list))
        try
            if (id of w) is in targetIds then close w
        end try
    end repeat
end tell" 2>/dev/null || true
```

**You MUST track window IDs at spawn time** (extract from osascript output: `sed -n 's/.*window id \([0-9]*\).*/\1/p'`).

## Reference Implementation: tl-dispatch.sh (ADAPT THIS, DON'T REWRITE)

This is the working, tested dispatch script from the TermLink project. The `dispatch`, `wait`, `result`, and `cleanup` subcommands in `termlink.sh` should adapt this code directly. Key patterns to preserve:

- **osascript spawn + window ID tracking** for cleanup
- **`termlink register --name` + wait loop** for session registration
- **`termlink pty inject --enter`** for fire-and-forget command injection (NOT `interact` — claude takes minutes)
- **Background process + kill watchdog** for timeout (macOS has no `timeout` command)
- **`termlink event emit worker.done`** for completion signaling
- **File-based result collection** (`/tmp/tl-dispatch/<worker>/result.md`)

```bash
#!/bin/bash
# tl-dispatch.sh — Spawn claude workers in real terminals via TermLink
# Tested with 3 parallel workers, all producing correct results.
set -e

DISPATCH_DIR="/tmp/tl-dispatch"

die() { echo "ERROR: $1" >&2; exit 1; }

ensure_termlink() {
    command -v termlink >/dev/null 2>&1 || die "termlink not found on PATH"
}

cmd_spawn() {
    local name="" prompt="" prompt_file="" project_dir="" timeout=600

    while [[ $# -gt 0 ]]; do
        case $1 in
            --name) name="$2"; shift 2 ;;
            --prompt) prompt="$2"; shift 2 ;;
            --prompt-file) prompt_file="$2"; shift 2 ;;
            --project) project_dir="$2"; shift 2 ;;
            --timeout) timeout="$2"; shift 2 ;;
            *) die "Unknown option: $1" ;;
        esac
    done

    [ -z "$name" ] && die "Missing --name"
    [ -z "$prompt" ] && [ -z "$prompt_file" ] && die "Missing --prompt or --prompt-file"

    if [ -n "$prompt_file" ]; then
        [ -f "$prompt_file" ] || die "Prompt file not found: $prompt_file"
        prompt=$(cat "$prompt_file")
    fi

    project_dir="${project_dir:-$(pwd)}"
    local wdir="$DISPATCH_DIR/$name"
    mkdir -p "$wdir"
    echo "$prompt" > "$wdir/prompt.md"

    cat > "$wdir/meta.json" <<METAEOF
{
  "name": "$name",
  "project": "$project_dir",
  "timeout": $timeout,
  "started": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
  "status": "running"
}
METAEOF

    # Worker script runs inside the spawned terminal
    cat > "$wdir/run.sh" <<'RUNEOF'
#!/bin/bash
WORKER_NAME="$1"; PROJECT_DIR="$2"; WDIR="$3"; TIMEOUT="$4"
cd "$PROJECT_DIR"

# Background process + kill watchdog (macOS has no `timeout`)
claude -p "$(cat "$WDIR/prompt.md")" --output-format text > "$WDIR/result.md" 2>"$WDIR/stderr.log" &
CLAUDE_PID=$!
(sleep "$TIMEOUT" && kill "$CLAUDE_PID" 2>/dev/null && echo "TIMEOUT" > "$WDIR/stderr.log") &
WATCHDOG_PID=$!
wait "$CLAUDE_PID" 2>/dev/null
EXIT_CODE=$?
kill "$WATCHDOG_PID" 2>/dev/null || true

echo "$EXIT_CODE" > "$WDIR/exit_code"
date -u +%Y-%m-%dT%H:%M:%SZ > "$WDIR/finished_at"
termlink event emit "$WORKER_NAME" worker.done \
    -p "{\"exit_code\":$EXIT_CODE,\"result\":\"$WDIR/result.md\"}" 2>/dev/null || true

echo ""
echo "=== Worker $WORKER_NAME finished (exit: $EXIT_CODE) ==="
echo "Result: $WDIR/result.md"
RUNEOF
    chmod +x "$wdir/run.sh"

    # Spawn terminal and track window ID
    local spawn_output
    spawn_output=$(osascript -e "tell application \"Terminal\" to do script \"termlink register --name $name --shell\"" 2>&1)
    local wid=$(echo "$spawn_output" | sed -n 's/.*window id \([0-9]*\).*/\1/p' | head -1)
    [ -n "$wid" ] && echo "$wid" > "$wdir/window_id"

    # Wait for session registration (up to 15s)
    local found=false
    for i in $(seq 1 15); do
        termlink list 2>/dev/null | grep -q "$name" && { found=true; break; }
        sleep 1
    done
    [ "$found" = true ] || die "Session $name did not register within 15s"

    # Inject worker script (fire-and-forget via pty inject, NOT interact)
    sleep 1
    termlink pty inject "$name" "bash $wdir/run.sh '$name' '$project_dir' '$wdir' '$timeout'" --enter >/dev/null 2>&1

    echo "Worker spawned: $name (wdir: $wdir)"
}

cmd_status() {
    echo "=== Active Workers ==="
    [ -d "$DISPATCH_DIR" ] || { echo "No workers dispatched."; return; }

    for wdir in "$DISPATCH_DIR"/*/; do
        [ -d "$wdir" ] || continue
        local name=$(basename "$wdir")
        local status="running"
        if [ -f "$wdir/exit_code" ]; then
            local ec=$(cat "$wdir/exit_code")
            [ "$ec" = "0" ] && status="complete" || status="failed (exit: $ec)"
        fi
        local session_alive="no"
        termlink list 2>/dev/null | grep -q "$name" && session_alive="yes" || true
        printf "  %-20s  status: %-20s  session: %s\n" "$name" "$status" "$session_alive"
    done
}

cmd_wait() {
    local name="" wait_all=false timeout=600
    while [[ $# -gt 0 ]]; do
        case $1 in
            --name) name="$2"; shift 2 ;;
            --all) wait_all=true; shift ;;
            --timeout) timeout="$2"; shift 2 ;;
            *) die "Unknown option: $1" ;;
        esac
    done

    if [ "$wait_all" = true ]; then
        [ -d "$DISPATCH_DIR" ] || die "No workers dispatched"
        local deadline=$(($(date +%s) + timeout))
        while [ "$(date +%s)" -lt "$deadline" ]; do
            local all_done=true
            for wdir in "$DISPATCH_DIR"/*/; do
                [ -d "$wdir" ] || continue
                [ -f "$wdir/exit_code" ] || { all_done=false; break; }
            done
            [ "$all_done" = true ] && { echo "All workers complete."; return 0; }
            sleep 2
        done
        echo "Timeout waiting for workers."; return 1
    else
        [ -z "$name" ] && die "Missing --name (or use --all)"
        local wdir="$DISPATCH_DIR/$name"
        [ -d "$wdir" ] || die "No worker named '$name'"

        # Event-based wait, then file confirmation
        termlink list 2>/dev/null | grep -q "$name" && \
            termlink event wait "$name" --topic worker.done --timeout "$timeout" >/dev/null 2>&1 || true

        local deadline=$(($(date +%s) + timeout))
        while [ ! -f "$wdir/exit_code" ] && [ "$(date +%s)" -lt "$deadline" ]; do sleep 2; done

        if [ -f "$wdir/exit_code" ]; then
            local ec=$(cat "$wdir/exit_code")
            echo "Worker $name finished (exit: $ec)"
            return "$ec"
        else
            echo "Timeout waiting for worker $name"; return 1
        fi
    fi
}

cmd_result() {
    local name=""
    while [[ $# -gt 0 ]]; do
        case $1 in --name) name="$2"; shift 2 ;; *) die "Unknown: $1" ;; esac
    done
    [ -z "$name" ] && die "Missing --name"
    local wdir="$DISPATCH_DIR/$name"
    [ -f "$wdir/result.md" ] && cat "$wdir/result.md" || { echo "No result yet."; return 1; }
}

cmd_cleanup() {
    [ -d "$DISPATCH_DIR" ] || { echo "No workers."; return; }

    # Collect tracked window IDs
    local window_ids=""
    for wdir in "$DISPATCH_DIR"/*/; do
        [ -f "$wdir/window_id" ] && window_ids="${window_ids:+$window_ids }$(cat "$wdir/window_id")"
    done

    termlink clean 2>/dev/null || true

    # Phase 1: kill child processes via TTY (spare login/shell)
    for wid in $window_ids; do
        local tty=$(osascript -e "tell application \"Terminal\" to try
            return tty of tab 1 of window id $wid
        end try" 2>/dev/null || true)
        if [ -n "$tty" ]; then
            ps -t "${tty#/dev/}" -o pid=,comm= 2>/dev/null \
                | grep -v -E '(login|-zsh|-bash)' \
                | awk '{print $1}' | xargs kill -9 2>/dev/null || true
        fi
    done
    sleep 2

    # Phase 2: exit shells gracefully
    for wid in $window_ids; do
        osascript -e "tell application \"Terminal\" to try
            do script \"exit\" in window id $wid
        end try" 2>/dev/null || true
    done
    sleep 2

    # Phase 3: close remaining by tracked window ID
    if [ -n "$window_ids" ]; then
        local id_list=""
        for wid in $window_ids; do id_list="${id_list:+$id_list, }$wid"; done
        osascript -e "tell application \"Terminal\"
            set targetIds to {$id_list}
            repeat with w in (reverse of (windows as list))
                try
                    if (id of w) is in targetIds then close w
                end try
            end repeat
        end tell" 2>/dev/null || true
    fi

    rm -rf "$DISPATCH_DIR"
    echo "All workers cleaned up."
}

# --- Main ---
ensure_termlink
case "${1:-}" in
    status)  cmd_status ;;
    wait)    shift; cmd_wait "$@" ;;
    result)  shift; cmd_result "$@" ;;
    cleanup) cmd_cleanup ;;
    --name)  cmd_spawn "$@" ;;
    *)       echo "Usage: tl-dispatch.sh --name <w> --prompt '...' | status | wait | result | cleanup" ;;
esac
```

## What's already built in TermLink (don't rebuild — just use)

**Repo:** `https://onedev.docker.ring20.geelenandcompany.com/termlink`

- **26 CLI commands** — all with `--json`, semantic exit codes
- **`interact --json`** — run command, wait, return `{output, exit_code, elapsed_ms, marker_found}`
- **Event system** — emit/wait/poll/broadcast/topics/collect
- **TCP hub** — `termlink hub start --tcp 0.0.0.0:9100` for cross-machine
- **Remote session store** — register_remote/heartbeat/deregister_remote RPCs with TTL
- **Hybrid discovery** — `termlink discover` returns both local + remote TCP sessions
- **Hub forwarding** — transparently routes requests to remote sessions
- **Auth tokens** — `termlink token` for capability-based session auth
- **Working dispatch prototype** — `scripts/tl-dispatch.sh` in the repo, tested with 3 parallel workers
- **264 tests passing** across 4 crates

To update TermLink: `cd /Users/dimidev32/001-projects/010-termlink && git pull && cargo install --path crates/termlink-cli`

## Phased Rollout (framework owns all phases)

| Phase | Scope | TermLink provides |
|-------|-------|-------------------|
| **0** | fw doctor + agents/termlink/ + fw termlink route | Binary on PATH |
| **1** | Self-test (fw self-test via termlink interact) | interact, output, register |
| **2** | Parallel dispatch (fw termlink dispatch) | pty inject, events, discover |
| **3** | Remote control + observation | attach, inject, output, stream |
| **4** | Cross-machine coordination | TCP hub, remote store, hybrid discover |

Build Phase 0 now. The `termlink` binary is at `/Users/dimidev32/.cargo/bin/termlink`. Verify it works: `termlink --version` should print `termlink 0.1.0`.
