# /self-test — Framework E2E Validation via TermLink

Run framework commands in an isolated PTY session via TermLink, observe output,
and report structured pass/fail results. Uses `termlink interact` for full
interactive execution (sees hooks, prompts, ANSI — everything a real terminal sees).

## Arguments

- `$ARGUMENTS` — optional comma-separated list of commands to test.
  If empty, defaults to: `fw doctor`

## Step 1: Parse test commands

If `$ARGUMENTS` is non-empty, split on commas to get the command list.
Otherwise use the default set:

```
fw doctor
```

Other useful commands to test (pass explicitly if desired):
- `fw audit` (slow — takes 30-60s)
- `fw context init`
- `fw metrics`
- `cargo test --workspace` (very slow — minutes)

## Step 2: Spawn test session

Run in a **new terminal window** via osascript:

```bash
osascript -e 'tell application "Terminal" to do script "termlink register --name self-test --shell"'
```

Then wait for the session to appear:

```bash
for i in $(seq 1 15); do
  termlink list 2>/dev/null | grep -q "self-test" && break
  sleep 1
done
```

If the session doesn't appear after 15 seconds, report failure and stop.

## Step 3: Set environment in test session

The test session needs PROJECT_ROOT for `fw` commands. Inject it:

```bash
termlink interact self-test "export PROJECT_ROOT=/Users/dimidev32/001-projects/010-termlink" --strip-ansi --timeout 5
```

## Step 4: Run each test command

For each command in the test set, run:

```bash
termlink interact self-test "<command>" --strip-ansi --json --timeout <T>
```

**Timeout selection:**
- `fw doctor`, `fw context init`, `fw metrics`, simple shell commands: `--timeout 15`
- `fw audit`: `--timeout 120`
- `cargo test --workspace`: `--timeout 300`
- All others: `--timeout 30`

Parse the JSON result. A command **passes** if:
- `marker_found` is true (command completed)
- The output does not contain obvious error indicators for that command type

**Pass/fail heuristics per command:**

| Command | Pass condition |
|---------|---------------|
| `fw doctor` | No "FAIL" lines in output |
| `fw audit` | No "FAILURE" in output |
| `fw context init` | No error in output |
| `fw metrics` | Output contains numeric data |
| `cargo test --workspace` | Output contains "test result: ok" |
| Other | `marker_found: true` and no "error" / "panic" in output |

Record for each command:
- Command string
- Pass / FAIL / TIMEOUT
- Elapsed ms
- First 10 lines of output (for context on failure)
- Full output on failure (for diagnosis)

## Step 5: Cleanup

Get the PID from `termlink list`, then kill it:

```bash
PID=$(termlink list 2>/dev/null | grep self-test | awk '{print $4}')
if [ -n "$PID" ]; then
  kill "$PID" 2>/dev/null
  sleep 2
  # Force kill if still alive
  kill -9 "$PID" 2>/dev/null
fi
```

Verify cleanup:

```bash
termlink clean 2>/dev/null
termlink list 2>/dev/null | grep -q "self-test" && echo "WARN: session still alive"
```

## Step 6: Report results

Print a structured report:

```
## Self-Test Results

| # | Command | Result | Time |
|---|---------|--------|------|
| 1 | fw doctor | PASS | 1.2s |
| 2 | fw audit | FAIL | 3.4s |

### Failures

**2. fw audit**
```
[first 10 lines of output]
```

### Summary
Passed: 1/2
```

## Error Recovery

- If `termlink interact` exits with code 1 and "Timeout" in the error, report TIMEOUT
- If `termlink interact` fails to connect, the session may have died — report SESSION_LOST
- If the session dies mid-test, report remaining commands as SKIPPED
- Always run cleanup (Step 5) even if tests fail

## Rules

- Always use `--strip-ansi` to get clean text output
- Always use `--json` to get structured results with timing
- If a command hangs (timeout), report it as TIMEOUT, not FAIL
- Do NOT leave the test session running — always clean up in Step 5
- The test session runs in a real terminal — it sources .zshrc/.bashrc
- Between commands, allow 1 second settling time (the shell needs to redraw prompt)
