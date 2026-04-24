# T-1211 — human-gated settings.json patch

The Stop hook handler (`stop-guard.sh`) is built, tested, and wired through
`fw hook stop-guard`. `.claude/settings.json` is policy-B-005 protected and
cannot be edited by the agent.

## Diff

Inside `.claude/settings.json`, add a `Stop` block alongside the other hook
keys (`SubagentStop`, `PostToolUse`, etc.):

```json
    "Stop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".agentic-framework/bin/fw hook stop-guard"
          }
        ]
      }
    ]
```

## Verify after applying

```
# Handler returns cleanly on a stub payload
echo '{"stop_hook_active":true,"session_id":"x","transcript_path":"/nonexistent"}' \
  | .agentic-framework/bin/fw hook stop-guard
echo "exit=$?"   # should print: exit=0

# Stub test covers all 4 conditions (below-threshold, at-threshold + various productive states)
.agentic-framework/agents/context/tests/stop-guard-stub-test.sh
# expected last line: "All stop-guard stub tests PASS"

# Live observation: after applying, counter increments on every assistant response
watch -n 2 'cat .context/working/.stop-counter .context/working/.stop-next-nudge-at 2>/dev/null'
```

## Behavior once active

- On **every** assistant response the handler increments `.stop-counter`.
- When `stop-counter` crosses `stop-next-nudge-at` (default N=15), the handler
  checks: tool-counter == 0 AND focus.yaml has no current_task.
- If both true → emits a stderr nudge the agent sees as additional context on
  the NEXT turn. The agent should then ask the user y/n about capturing the
  conversation as a task.
- If conditions not met (user is being productive) → silent, re-arms for next
  threshold window.

## Tuning

- Override threshold via env: `STOP_NUDGE_EVERY=10` (in whatever environment
  launches claude). Defaults to 15.
- Dismissal: when the user says "n", the agent should write
  `.context/working/.stop-dismissed` with the current timestamp. The handler
  does NOT read this today — re-arm comes from `.stop-next-nudge-at` advancing
  naturally. A future revision could honor explicit dismissals differently
  (e.g., double the threshold on each dismissal).
