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

## Exploration: How the Self-Test Loop Works

### Scenario: Agent Tests `fw doctor`

```
┌──────────────────┐        ┌──────────────────┐
│  Agent Session   │        │  Test Session     │
│  (Claude Code)   │        │  (TermLink PTY)   │
│                  │        │                   │
│  1. Register ────────────►│  Spawned          │
│                  │        │                   │
│  2. Exec ────────────────►│  fw doctor        │
│     "fw doctor"  │        │  ... runs ...     │
│                  │◄───────│  {exit:1, stdout,  │
│  3. Analyze      │        │   stderr}         │
│     output       │        │                   │
│                  │        │                   │
│  4. Fix script   │        │                   │
│     (Edit tool)  │        │                   │
│                  │        │                   │
│  5. Retry ───────────────►│  fw doctor        │
│     "fw doctor"  │        │  ... runs ...     │
│                  │◄───────│  {exit:0, stdout}  │
│  6. Assert pass  │        │                   │
│                  │        │                   │
│  7. Cleanup ─────────────►│  Terminated       │
└──────────────────┘        └──────────────────┘
```

### Step-by-Step Protocol

```bash
# 1. Register a test session
termlink register --name fw-test --shell

# 2. Wait for ready
termlink wait fw-test --state ready --timeout 5

# 3. Execute the command under test
RESULT=$(termlink exec fw-test "fw doctor" 2>&1)
EXIT_CODE=$?

# 4. Agent reads RESULT, diagnoses issues

# 5. Agent fixes the script (Edit tool on the source file)

# 6. Retry
RESULT=$(termlink exec fw-test "fw doctor" 2>&1)

# 7. Cleanup
termlink send fw-test session.signal '{"signal": "SIGTERM"}'
```

### What the Agent Sees

`command.execute` returns structured JSON:
```json
{
  "exit_code": 1,
  "stdout": "=== Framework Health Check ===\n✓ fw binary found\n✗ context not initialized\n...",
  "stderr": "Error: .context/working/focus.yaml not found"
}
```

The agent gets full stdout + stderr + exit code. No streaming needed for
short-running commands. For long-running commands (>30s), use:
- `termlink stream fw-test` in background (pipes real-time output)
- `termlink send fw-test query.output '{"lines": 200}'` (poll scrollback)

### What Can Be Tested This Way

| Framework Component | Test Command | What Agent Observes |
|----|---|---|
| Health checks | `fw doctor` | Exit code + diagnostic output |
| Session init | `fw context init` | Success/failure + created files |
| Task creation | `fw task create --name test --type build` | Task file created, ID returned |
| Git hooks | `fw git commit -m "T-000: test"` | Hook output, pass/fail |
| Audit | `fw audit` | Compliance report, exit code |
| Budget gate | Write a file, check if hook fires | Hook stderr output |
| Handover | `fw handover` | Handover file created |
| Episodic generator | `fw context generate-episodic T-001` | macOS date bug reproduction |

### Limitations

1. **Hook reload**: If the agent fixes a PreToolUse hook script, the fix takes
   effect immediately (hooks are re-read per invocation). But changes to
   CLAUDE.md or Claude Code settings require a new Claude Code session.

2. **Interactive commands**: Commands requiring stdin interaction (e.g.,
   `git rebase -i`) cannot be tested via `command.execute`. They need
   `command.inject` (keystroke injection into PTY).

3. **Environment isolation**: The test session inherits the user's environment.
   To test in a clean environment, use `command.execute` with explicit `env`
   parameter to override variables.

4. **Timeout**: Default 30s timeout on `command.execute`. Long-running tests
   need explicit timeout parameter or streaming observation.

## Options Considered

### Option A: Direct Bash Execution (Status Quo)
The agent runs `fw doctor` directly in its own Bash tool.

- **Pro**: Simplest, no TermLink dependency
- **Con**: Pollutes the agent's own environment; can't test init sequences
  that modify state; hooks fire in the agent's session (circular); can't
  test multi-session scenarios

### Option B: TermLink `command.execute` (Recommended)
Agent spawns a TermLink session, runs commands via `command.execute` RPC.

- **Pro**: Clean isolation, structured output (exit_code + stdout + stderr),
  timeout control, allowlist support, no environment pollution
- **Con**: Requires TermLink running; adds dependency; slight overhead

### Option C: TermLink Streaming + PTY Observation
Agent spawns session, sends commands via `command.inject`, reads output via
`query.output` or data plane streaming.

- **Pro**: Tests interactive scenarios (vim, REPLs); sees exactly what a
  human would see including ANSI sequences
- **Con**: More complex; needs ANSI parsing; harder to assert on output

### Option D: Hybrid (B for scripts, C for interactive)
Use `command.execute` for simple script testing. Use inject + streaming
for interactive program testing.

- **Pro**: Best of both worlds; covers all scenarios
- **Con**: More implementation effort; two code paths

## Decision

### 2026-03-14 — Self-test observation mechanism
- **Chose:** Option D (Hybrid) — `command.execute` for script testing,
  inject + streaming for interactive testing
- **Why:** Script testing is the 90% case and `command.execute` makes it
  trivial (structured output, exit code, timeout). Interactive testing is
  needed for edge cases (hooks firing in PTY, vim detection) and streaming
  handles that. Both mechanisms already exist and are tested.
- **Rejected:**
  - Option A (direct Bash): No isolation, circular hook execution
  - Option B alone: Can't test interactive scenarios
  - Option C alone: Unnecessary complexity for simple script tests

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
Agent uses Bash tool to run TermLink CLI commands directly:
```bash
termlink register --name self-test --shell
termlink exec self-test "fw doctor"
# ... analyze, fix, retry ...
```
No new code. Just agent discipline.

### Phase 1: Framework Skill (`/self-test`)
Create a skill that automates the spawn → exec → observe → report loop.
Returns structured test results. ~1 session effort.

### Phase 2: Continuous Validation
Agent runs `/self-test` after modifying framework scripts. Integrated into
the commit cadence: edit → self-test → commit. ~1 session effort.

### Phase 3: Interactive Testing
Add inject + streaming observation for PTY-based tests (vim detection,
hook output in interactive terminals). ~2 sessions effort.

## Related Tasks
- T-121: PTY mode detection (enables Phase 3 interactive testing)
- T-011: Distributed topology (cross-machine self-testing)
- T-100: Output capture as conversation logger (related observation pattern)
