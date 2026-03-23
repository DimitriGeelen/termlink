# T-233 Q2: Reactive Instruction — Agent Self-Assessment & Human Injection

## Summary

Reactive instruction means the delegation trigger comes from **within** the conversation — either the agent recognizes it needs help, or the human explicitly tells it to delegate. This is the most natural discovery mechanism because it mirrors how human teams work: "I'm stuck, can someone else handle this?" or "Hand this off to the infra team."

## Agent Self-Assessment Heuristics

An agent can recognize it needs a specialist through several signals:

### 1. Competence Boundary Detection
The agent monitors its own confidence and flags when it's operating outside its domain:
- **Repeated failures** — 2+ failed attempts at the same operation (e.g., cargo build errors in unfamiliar crate) trigger: "I'm not making progress; a specialist might do better."
- **Tool mismatch** — The agent finds itself reaching for tools it doesn't have (e.g., needing SSH access, database queries, or hardware APIs it can't call).
- **Context overload** — The agent's working set exceeds what it can reason about effectively. Heuristic: if it needs to re-read the same files 3+ times, the task may be too broad for one agent.

### 2. Structured Self-Declaration
The agent emits a typed signal when it decides to request help:

```yaml
# termlink event: specialist.request
type: specialist.request
from: agent-coder-01
reason: "Repeated SSH failures — need infra specialist"
domain: infrastructure
context_refs:
  - .tasks/active/T-233-specialist-orchestration.md
  - crates/termlink-hub/src/server.rs:142
```

This uses TermLink's existing `events emit` primitive. The orchestrator subscribes to `specialist.request` events and routes them.

### 3. Protocol: Agent-to-Agent Request
Using `termlink agent ask`, an agent can directly request help:

```bash
termlink agent ask --to orchestrator --type delegate \
  --payload '{"domain":"infra","task":"deploy to .107","context":"T-233"}'
```

The orchestrator receives this as a typed request, evaluates available specialists, and either dispatches or queues. The requesting agent gets back a handle to poll for results via `fw bus`.

## Human Injection of Delegation Commands

The human needs to redirect work mid-conversation without breaking flow:

### 1. Inline Directive
The simplest pattern — the human types a natural-language instruction:
> "Delegate the deployment part to the infra agent"

The orchestrator parses this as a delegation intent. No special syntax needed if the orchestrator has NL understanding. However, explicit syntax is more reliable:
> `/delegate infra "deploy the hub binary to .107"`

### 2. PreToolUse Hook Interception
A PreToolUse hook on Bash/Write could detect when the agent is about to perform work outside its declared domain. For example, if a "coder" agent runs `ssh` or `systemctl`, the hook intercepts:

```
[HOOK] Agent "coder" attempting infrastructure operation (ssh).
       Suggest: /delegate infra "deploy hub to remote"
       Allow / Delegate / Block?
```

This is **reactive but framework-assisted** — the framework watches for domain crossover and prompts the human.

### 3. Event-Driven Human Override
The human monitors agent activity (via `termlink mirror` or Watchtower) and injects delegation at any point:

```bash
termlink interact orchestrator --command '/delegate research "compare WebSocket vs gRPC for agent comms"'
```

## Design Recommendation

The strongest reactive pattern combines all three:
1. **Agent self-assesses** and emits `specialist.request` events (autonomous, no human needed)
2. **Framework hooks detect** domain crossover and suggest delegation (semi-autonomous)
3. **Human can always override** with `/delegate` syntax (sovereign control)

The orchestrator consumes all three signal types identically — they all produce the same `specialist.request` event shape. The only difference is the `trigger` field: `self`, `hook`, or `human`.

**Key insight:** Reactive discovery is the fallback for when proactive mechanisms (hooks, keyword triggers) miss. It's the "escape valve" — if nothing else catches it, the agent or human always can.
