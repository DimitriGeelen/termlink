# T-233 Q2: PreToolUse Hook Evaluation for Specialist Delegation

## Mechanism

Claude Code's PreToolUse hooks fire before every tool call, receiving structured JSON on stdin with `tool_name`, `tool_input`, `session_id`, and `cwd`. The hook can **allow**, **deny**, or **ask** via exit codes and JSON output. This creates a natural interception point for delegation.

## How It Works

**Classification phase** (hook script):
1. Hook receives stdin JSON: `{ tool_name: "Bash", tool_input: { command: "ssh server ..." } }`
2. Script pattern-matches the command/file path against a **domain registry** (YAML mapping patterns → specialist types)
3. If a specialist match is found, hook returns exit 0 with structured JSON

**Signaling phase** (two viable patterns):

**Pattern A — Deny + Reroute:** Hook returns `permissionDecision: "deny"` with `permissionDecisionReason: "DELEGATE:infra: ssh commands should be handled by the infrastructure specialist. Use: termlink agent ask infra-agent '...'"`. The orchestrator agent sees the denial reason, recognizes the `DELEGATE:` prefix, and dispatches via TermLink instead of retrying the blocked tool.

**Pattern B — Allow + Annotate:** Hook returns `permissionDecision: "allow"` with `additionalContext: "This is infrastructure work. Consider delegating to infra-agent via termlink agent ask."`. The tool executes normally, but the agent receives a nudge. Softer — the agent decides whether to delegate.

## Exit Code Semantics

| Exit | Meaning | Delegation Use |
|------|---------|---------------|
| 0 | Success, parse stdout JSON | Classification + allow/deny decision |
| 2 | Hard block, stderr → agent | Emergency block (e.g., Tier 0 + delegation) |
| 1 | Non-blocking error, logged | Classification failed, fall through |

## Domain Registry Design

```yaml
# .claude/hooks/delegation-rules.yaml
domains:
  infra:
    patterns:
      - command: "ssh|scp|rsync|systemctl|journalctl"
      - command: "docker|kubectl|helm"
    specialist: infra-agent
    mode: deny  # hard delegation

  test:
    patterns:
      - command: "cargo test|pytest|jest|npm test"
    specialist: test-agent
    mode: suggest  # soft nudge via additionalContext

  research:
    patterns:
      - tool: "WebSearch|WebFetch"
    specialist: research-agent
    mode: suggest
```

The hook script reads this registry, matches against the incoming tool call, and produces the appropriate JSON response.

## Hook Script Skeleton

```bash
#!/bin/bash
INPUT=$(cat)
TOOL=$(echo "$INPUT" | jq -r '.tool_name')
CMD=$(echo "$INPUT" | jq -r '.tool_input.command // empty')

# Match against domain registry
MATCH=$(match-domain "$TOOL" "$CMD")  # reads delegation-rules.yaml
if [ -z "$MATCH" ]; then exit 0; fi

SPECIALIST=$(echo "$MATCH" | jq -r '.specialist')
MODE=$(echo "$MATCH" | jq -r '.mode')

if [ "$MODE" = "deny" ]; then
  jq -n '{hookSpecificOutput: {hookEventName: "PreToolUse",
    permissionDecision: "deny",
    permissionDecisionReason: "DELEGATE:\($spec): Route via termlink agent ask"}}' \
    --arg spec "$SPECIALIST"
else
  jq -n '{hookSpecificOutput: {hookEventName: "PreToolUse",
    permissionDecision: "allow",
    additionalContext: "Consider delegating to \($spec)"}}' \
    --arg spec "$SPECIALIST"
fi
```

## Strengths

- **Zero agent cooperation required** — works even if the agent doesn't know about specialists; the hook enforces delegation structurally
- **Composable with existing hooks** — chains with budget-gate, check-tier0, check-active-task
- **Pattern A (deny) is deterministic** — agent *must* use TermLink dispatch; no discretion involved
- **Configurable per-domain** — hard delegation for infra, soft nudges for testing

## Limitations

- **Hooks cannot spawn agents** — they can only signal; the agent must act on the signal
- **Hooks cannot change tool_name** — a Bash call stays a Bash call; can only modify input or block
- **Pattern matching is shallow** — regex on commands catches `ssh` but not "deploy the service" (semantic intent requires LLM classification, not hook scripts)
- **Latency** — every tool call pays the hook evaluation cost (should be <50ms for YAML lookup)
- **No context awareness** — hook doesn't know the task type or conversation history; it classifies purely on tool input syntax

## Verdict

PreToolUse hooks are a strong **structural enforcement layer** for delegation — they work like Tier 0/Tier 1 gates but for specialist routing. Best suited for **deterministic, pattern-matchable** work (infra commands, test runs) where the tool input alone reveals the domain. Not suited for semantic classification ("is this research or coding?"). Recommended as one layer in a multi-mechanism approach: hooks catch the obvious cases, while interactive/reactive mechanisms handle the nuanced ones.
