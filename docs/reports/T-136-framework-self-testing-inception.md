# T-136: Framework Agent Self-Testing via TermLink — Inception Report

> Task: T-136 | Type: inception | Created: 2026-03-14
> Question: Can the framework agent use TermLink to spawn terminals, test its own scripts, observe results, and self-heal?

## Problem Statement

Currently, when the framework agent (Claude Code + agentic-fw) needs to test
its own tooling — init scripts, hooks, `fw doctor`, `fw context init`, etc. —
there is no automated path. The human must:

1. Open another terminal
2. Run the command manually (e.g., `fw context init`)
3. Copy-paste the output back to the agent
4. Agent analyzes and suggests fixes
5. Repeat

This is slow, error-prone, and breaks the agent's autonomous work loop. With
TermLink, the agent should be able to spawn a fresh terminal, run framework
commands in it, observe the output, diagnose failures, fix the scripts, and
retry — all within a single session.

## Assumptions

### A1: TermLink can execute commands and capture output
**Status:** VALIDATED
**Evidence:** `command.execute` RPC returns `{exit_code, stdout, stderr}` in
a single call. Tested across 253 tests. The executor supports timeout, env
vars, working directory, and command allowlists.

### A2: The agent can observe real-time terminal output
**Status:** VALIDATED
**Evidence:** Three mechanisms available:
1. `command.execute` — returns stdout/stderr after completion (simplest)
2. `query.output` — reads scrollback buffer (1 MiB ring, lines or bytes)
3. Data plane streaming — real-time binary frames over separate socket

For automated test loops, `command.execute` is sufficient. For interactive
debugging or long-running commands, streaming + scrollback polling works.

### A3: Claude Code can invoke TermLink CLI commands
**Status:** VALIDATED
**Evidence:** Claude Code's Bash tool can run `termlink exec <session> "command"`.
The 26 CLI commands are all available from the shell. The agent mesh (dispatch.sh)
already demonstrates Claude Code orchestrating TermLink sessions.

### A4: A fresh terminal session provides clean environment for testing
**Status:** VALIDATED
**Evidence:** `termlink register --shell` spawns a new PTY-backed session with
its own shell, environment, and working directory. Each session is isolated.
The session inherits the user's shell profile but starts clean.

### A5: The agent can fix scripts and retry without human intervention
**Status:** PARTIALLY VALIDATED
**Evidence:** Claude Code can edit files (Edit tool), run tests (Bash tool),
and re-execute commands. The loop is: exec → observe → diagnose → fix → retry.
Limitation: the agent cannot restart hooks or reload CLAUDE.md mid-session
(requires session restart for some changes).

## Exploration: Full Interactive E2E Testing Loop

### Requirement

The framework agent must be able to do **everything a human can do** in a
terminal: type commands, see all output (including prompts, colors, hook stderr),
respond to prompts, run sequences of commands where each depends on the previous,
and iterate.

This rules out `command.execute` as the primary mechanism — it runs commands in
a subprocess, not in the PTY. The agent needs the **inject + observe** path.

### Two Modes of Observation

| Mode | Mechanism | What It Sees | Use Case |
|------|-----------|-------------|----------|
| **Subprocess** | `command.execute` | stdout + stderr + exit_code | One-shot tests, no state |
| **Interactive** | `inject` + `query.output` | Everything: prompt, ANSI, hooks, errors | Full E2E, stateful sequences |

**The interactive mode is the primary requirement.** `command.execute` is a
convenience shortcut for simple cases, but the real value is the interactive loop.

### How Interactive Mode Works

```
┌───────────────────┐        ┌───────────────────────────┐
│  Agent Session    │        │  Test Session (PTY)        │
│  (Claude Code)    │        │  termlink register --shell │
│                   │        │                            │
│  1. Register ─────────────►│  Shell spawned (zsh/bash)  │
│                   │        │  $ ▌                       │
│  2. Inject ───────────────►│  $ fw context init▌        │
│     "fw context   │        │  === Context Init ===      │
│      init\n"      │        │  ✓ Created focus.yaml      │
│                   │        │  ✗ Error: missing dir      │
│  3. Poll ─────────────────►│                            │
│     query.output  │◄───────│  (scrollback: everything)  │
│     {"lines":100} │        │  $ ▌                       │
│                   │        │                            │
│  4. Analyze:      │        │                            │
│     "prompt is    │        │                            │
│      back → done" │        │                            │
│                   │        │                            │
│  5. Fix the bug   │        │                            │
│     (Edit tool)   │        │                            │
│                   │        │                            │
│  6. Inject ───────────────►│  $ fw context init▌        │
│     "fw context   │        │  === Context Init ===      │
│      init\n"      │        │  ✓ Created focus.yaml      │
│                   │        │  ✓ Created session.yaml    │
│  7. Poll ─────────────────►│                            │
│     query.output  │◄───────│  $ ▌  (prompt back = done) │
│                   │        │                            │
│  8. Inject ───────────────►│  $ fw doctor▌              │
│     "fw doctor\n" │        │  === Health Check ===      │
│                   │        │  ✓ All checks pass         │
│  9. Poll ─────────────────►│                            │
│     query.output  │◄───────│  $ ▌                       │
│                   │        │                            │
│  10. Done! ───────────────►│  SIGTERM → Terminated      │
└───────────────────┘        └───────────────────────────┘
```

### Step-by-Step Protocol (Interactive)

```bash
# 1. Spawn a PTY-backed test session
termlink register --name fw-test --shell
termlink wait fw-test --state ready --timeout 5

# 2. Inject a command (types into the PTY, like a human)
termlink send fw-test command.inject '{"keys":[
  {"type":"text","value":"fw context init"},
  {"type":"key","value":"Enter"}
]}'

# 3. Wait for command to finish (poll until prompt returns)
sleep 2  # or smart poll loop (see below)

# 4. Read EVERYTHING the terminal shows
termlink send fw-test query.output '{"lines":100}'
# Returns: raw scrollback including prompt, colors, output, errors

# 5. Agent analyzes output, fixes scripts if needed

# 6. Inject next command
termlink send fw-test command.inject '{"keys":[
  {"type":"text","value":"fw doctor"},
  {"type":"key","value":"Enter"}
]}'

# 7. Poll again
sleep 2
termlink send fw-test query.output '{"lines":100}'

# 8. Cleanup
termlink send fw-test session.signal '{"signal":"SIGTERM"}'
```

### What the Agent Sees (Scrollback)

`query.output` returns the raw terminal content including ANSI sequences:

```json
{
  "output": "$ fw context init\r\n\u001b[0;36m=== Context Init ===\u001b[0m\r\n\u001b[0;32m✓ Created focus.yaml\u001b[0m\r\n\u001b[1;31m✗ Error: .context/working/ does not exist\u001b[0m\r\n$ ",
  "bytes_len": 247,
  "total_buffered": 247
}
```

The agent sees **exactly what a human sees** — prompts, colors, errors,
everything. It can:
- Detect when the shell prompt returns (`$ ` at end → command finished)
- Parse success/failure markers (✓ / ✗)
- Read error messages and diagnose root causes
- Decide what to type next based on what it saw

### Synchronization: Knowing When a Command Finishes

The key challenge: after inject, how does the agent know the command is done?

**Option 1: Marker Injection (Recommended)**
```bash
# Inject: fw doctor; echo "___MARKER_DONE___"
# Poll until scrollback contains ___MARKER_DONE___
```
Reliable, deterministic, works for any command.

**Option 2: Prompt Detection**
Poll scrollback until the last line matches the shell prompt pattern (`$ `, `% `).
Works for most cases but fragile with multi-line prompts.

**Option 3: Stabilization Polling**
Poll `query.output` repeatedly. When `total_buffered` stops changing for 1s,
the command is done. Simple but adds latency.

**Option 4: Smart Hybrid (Recommended)**
Use marker injection for automated tests. Use stabilization polling as fallback.
Marker injection is deterministic; stabilization handles edge cases where the
marker itself might produce output.

### Input Capabilities (`command.inject`)

The inject handler accepts three key types:
- **Text:** `{"type": "text", "value": "fw doctor"}` — UTF-8 string, passed as-is
- **Key:** `{"type": "key", "value": "Enter"}` — named keys: Enter (0x0D), Tab,
  Escape, Ctrl+A through Ctrl+Z, arrows, Home/End, Delete
- **Raw:** `{"type": "raw", "value": "Aw=="}` — base64-encoded arbitrary bytes
  (e.g., 0x03 = Ctrl+C)

This means the agent can type anything a human can type — including control
sequences, escape codes, and special keys.

### Scrollback Content

`query.output` returns raw terminal bytes as UTF-8, including:
- Shell prompts (`$ `, `% `)
- ANSI escape sequences (colors: `\x1b[0;32m`, cursor movement, bold)
- Command output (stdout interleaved with stderr in the PTY)
- Error messages from hooks, scripts, etc.

No ANSI stripping is built in. The agent (Claude) can parse ANSI or regex-strip
it (`\x1b\[[0-9;]*m`). A future enhancement could add an ANSI-strip option.

### Existing Pattern: `attach` Command

The CLI `termlink attach` already implements the interactive loop:
1. Put stdin in raw mode
2. `tokio::select!` on stdin + poll timer
3. On stdin: inject keystrokes (fire-and-forget, no RPC response wait)
4. On timer: poll `query.output(bytes=8192)`, detect delta via `total_buffered`
5. Write new output to stdout

Claude Code replicates this synchronously: inject → sleep → poll → analyze.

### What Can Be Tested (Full E2E)

| Scenario | How | What Agent Sees |
|----------|-----|-----------------|
| `fw doctor` | inject + poll | All diagnostic output, exit indication |
| `fw context init` then `fw doctor` | inject sequence | Init creates state, doctor validates it |
| Hook behavior during commit | inject `git commit` | Hook stderr, pass/fail, prompts |
| Interactive git operations | inject + respond to prompts | Full git workflow output |
| `fw audit` compliance | inject + poll | Full audit report with colors |
| Init sequence from scratch | inject sequence of commands | Build up from empty .context/ |
| Test hook scripts after editing | Edit hook → inject trigger | See if fixed hook works |
| macOS date bug reproduction | inject `generate-episodic` | See exact error output |
| Budget gate trigger | inject Write tool equivalent | See gate block message |

### Limitations

1. **ANSI sequences**: Scrollback includes raw ANSI. The agent (Claude) can
   read these, but pattern matching is harder. Could add ANSI-strip option
   to `query.output` as enhancement.

2. **Timing**: No built-in "command done" signal for interactive mode.
   Marker injection solves this reliably.

3. **Claude Code Bash tool is synchronous**: Each inject + poll is a separate
   Bash call. The agent cannot stream and inject simultaneously in one call.
   This is fine — the poll loop is: inject → sleep → poll → analyze → repeat.

4. **Shell state**: The test session is a real shell. Environment variables,
   working directory, and shell state persist between commands — exactly like
   a human's terminal. This is a feature, not a bug.

5. **CLAUDE.md changes**: If the agent fixes CLAUDE.md, that won't affect the
   test session (CLAUDE.md is Claude Code-specific). Hook script fixes take
   effect immediately since hooks are re-read per invocation.

## Options Considered

### Option A: Direct Bash Execution (Status Quo)
The agent runs `fw doctor` directly in its own Bash tool.

- **Pro**: Simplest, no TermLink dependency
- **Con**: Pollutes the agent's own environment; can't test init sequences
  that modify state; hooks fire in the agent's session (circular); can't
  test multi-session scenarios; agent can't observe its own hooks firing

### Option B: TermLink `command.execute` Only
Agent spawns a session, runs commands via `command.execute` RPC (subprocess).

- **Pro**: Structured output (exit_code + stdout + stderr), timeout control
- **Con**: Subprocess, not interactive PTY. No shell state between commands.
  Can't test sequences. Can't see hook output in the PTY. Can't respond to
  prompts. Not what a human would experience.

### Option C: TermLink Interactive (inject + query.output)
Agent spawns PTY session, injects keystrokes via `command.inject`, reads
ALL output via `query.output` scrollback polling. Full E2E.

- **Pro**: Sees everything a human sees. Shell state persists between commands.
  Can test sequences (init → doctor → commit). Can respond to prompts.
  Can test interactive programs. Tests the real user experience.
- **Con**: Needs synchronization (marker injection or prompt detection).
  Raw ANSI in output. Slightly more complex than subprocess.

### Option D: Hybrid (C primary, B for quick checks)
Interactive mode (inject + poll) as the primary mechanism. `command.execute`
available as a shortcut for isolated one-shot checks where you don't need
shell state or interaction.

- **Pro**: Full E2E capability with a fast path for simple cases
- **Con**: Two code paths, but both already exist

## Decision

### 2026-03-14 — Self-test observation mechanism
- **Chose:** Option C (Full Interactive) as primary, with D (Hybrid) for
  convenience. The agent must see everything and be able to interact.
- **Why:** The whole point is end-to-end testing. `command.execute` runs a
  subprocess — it doesn't test the real user experience. The framework agent
  needs to see hooks firing, prompts appearing, shell state carrying over
  between commands. Only the interactive path (inject + scrollback) gives
  this. `command.execute` remains available as a convenience shortcut.
- **Rejected:**
  - Option A (direct Bash): No isolation, circular hook execution
  - Option B alone: Subprocess, not interactive. Misses the point.

## Go/No-Go Assessment

### Go Criteria
1. TermLink can execute commands and return structured output — **YES**
2. Agent can spawn isolated sessions — **YES**
3. Output is readable without streaming for short commands — **YES**
4. Streaming available for long/interactive commands — **YES**
5. No new code needed for basic loop — **YES** (all CLI commands exist)

### Risk Assessment
- **Low risk**: All required capabilities exist and are tested (253 tests)
- **Dependency**: Requires TermLink binary on PATH (already true in dev)
- **Framework integration**: Needs a skill or protocol for the test loop
  (new work, but small — wrapper around existing CLI commands)

### Verdict: **GO**

The framework agent can use TermLink today to spawn terminals, test its own
scripts, observe output, and fix issues in a loop. No new TermLink code is
needed. The integration work is:

1. **Framework skill** (`/self-test` or `/validate`): Wrapper that registers
   a test session, runs a command, captures output, returns structured result
2. **Test loop protocol**: Define how the agent should iterate (max retries,
   what to fix, when to escalate to human)
3. **Example workflows**: Document common self-test scenarios (fw doctor,
   hook testing, init sequence)

## Phased Implementation

### Phase 0: Manual Proof-of-Concept (Today)
Agent uses Bash tool to run TermLink CLI commands directly. Full interactive:
```bash
# Spawn
termlink register --name fw-test --shell
termlink wait fw-test --state ready --timeout 5

# Inject command
termlink send fw-test command.inject '{"keys":[
  {"type":"text","value":"fw doctor"},
  {"type":"key","value":"Enter"}
]}'

# Wait + observe
sleep 2
termlink send fw-test query.output '{"lines":100}'

# Agent reads output, fixes issues, injects next command...

# Cleanup
termlink send fw-test session.signal '{"signal":"SIGTERM"}'
```
No new code. All primitives exist. Just agent discipline + Bash calls.

### Phase 1: Synchronization Helper
Add a wrapper script or CLI command that handles the inject → wait → poll
loop with marker-based synchronization:
```bash
# Proposed: termlink interact <session> "fw doctor"
# Internally: injects command + marker, polls until marker appears, returns output
```
~1 session effort. Makes the agent's job easier (one call instead of
inject + sleep + poll).

### Phase 2: Framework Skill (`/self-test`)
Create a skill that automates the full E2E test loop:
- Spawns test session
- Runs a sequence of framework commands interactively
- Observes all output (scrollback)
- Reports structured results (pass/fail per command, full output log)
- Cleans up session
~1-2 sessions effort.

### Phase 3: Self-Healing Loop
Agent runs `/self-test`, diagnoses failures from output, fixes scripts,
re-runs `/self-test`. Integrated into commit cadence:
edit → self-test → fix → self-test → commit. ~1 session effort.

### Phase 4: ANSI-Clean Output Option
Add optional ANSI-stripping to `query.output` so the agent gets clean text
for pattern matching. Useful but not blocking — Claude can parse ANSI.
~0.5 session effort.

## Proof-of-Concept Spike Results (2026-03-14)

Actual interactive test loop executed against a live TermLink PTY session:

| Test | Result |
|------|--------|
| Register PTY session | PASS — `spike-test` spawned with zsh |
| Inject keystrokes | PASS — `echo HELLO_TERMLINK` typed, output captured |
| Read scrollback | PASS — full output with prompt, ANSI, command echo |
| Run `fw doctor` E2E | PASS — all OK/WARN/FAIL diagnostics visible |
| Marker synchronization | PASS — `___DONE___` detected in scrollback via grep |
| Shell state persistence | PASS — env var set in inject 1, read in inject 2 |
| Session cleanup | PASS — SIGTERM terminates, removed from list |

### Gotchas Discovered
1. `termlink send` uses `--params` flag, not positional: `termlink send <session> <method> --params '{...}'`
2. ANSI control chars in scrollback break Python `json.load` — use grep or regex, not JSON parsing
3. Signal is via `termlink signal <session> SIGTERM` CLI command, not RPC via `send`
4. Zsh `PROMPT_SP` (`%` marker) appears at start of output — cosmetic, not a problem
5. 1-2 second sleep between inject and poll is sufficient for framework commands

## Related Tasks
- T-121: PTY mode detection (enables Phase 3 interactive testing)
- T-011: Distributed topology (cross-machine self-testing)
- T-100: Output capture as conversation logger (related observation pattern)
