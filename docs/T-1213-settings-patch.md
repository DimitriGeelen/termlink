# T-1213 — human-gated settings.json patch

The `SubagentStop` hook handler is built, tested, and wired through
`fw hook subagent-stop`. To activate it in this project, `.claude/settings.json`
needs one block appended to the `hooks` object. This file is protected by
policy B-005 (Enforcement Config Protection) and cannot be edited by the agent.

## Diff

Inside `.claude/settings.json`, after the closing `]` of the existing `PostToolUse`
array (around line 167), add:

```json
    "SubagentStop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".agentic-framework/bin/fw hook subagent-stop"
          }
        ]
      }
    ]
```

The final object should end like this (note the `,` after the `PostToolUse` array):

```json
    "PostToolUse": [
      ...existing entries...
    ],
    "SubagentStop": [
      {
        "matcher": "",
        "hooks": [
          {
            "type": "command",
            "command": ".agentic-framework/bin/fw hook subagent-stop"
          }
        ]
      }
    ]
  }
}
```

## Verify after applying

```
# Handler stays exit-0 on any stub payload
echo '{"transcript_path":"/nonexistent","agent_type":"stub","agent_id":"x","session_id":"y"}' \
  | .agentic-framework/bin/fw hook subagent-stop
echo "exit=$?"   # should print: exit=0

# Stub test covers both under- and over-threshold paths
.agentic-framework/agents/context/tests/subagent-stop-stub-test.sh
# expected last line: "All stub tests PASS"

# After applying settings and dispatching a real sub-agent, a telemetry line appears:
tail -1 .context/working/subagent-returns.jsonl
```

## Retirement of check-dispatch.sh (deferred)

T-1213 also plans to retire `check-dispatch.sh` (PostToolUse advisory for
Task/TaskOutput) once the SubagentStop handler has run live for at least one
session and the telemetry file shows sensible data. That retirement is a
separate follow-up: remove the `matcher: "Task|TaskOutput"` block from
`PostToolUse` in `.claude/settings.json`. Hold until live confidence is
established — check-dispatch and subagent-stop coexist without conflict.
