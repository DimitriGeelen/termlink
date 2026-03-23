# Q2-Interactive: Human-Directed Discovery for Specialist Delegation

## Research Question
How does an orchestrator discover WHAT to delegate when the human explicitly directs it? Design the UX: commands, syntax, intent parsing.

## Three Interaction Patterns

### 1. @-Mention Routing (Recommended Primary)
The human addresses a specialist by role using `@` syntax within normal conversation:

```
@research find how Rust async executors handle cancellation
@infra check disk usage on the remote server
@test write integration tests for the hub reconnect path
```

**Parsing:** The orchestrator intercepts any message starting with `@<role>`. The role maps to a specialist registry (a YAML manifest of known roles → agent configurations). Everything after the role token is the task prompt.

**Advantages:** Familiar (Slack, GitHub, Discord all use @-mentions). Zero learning curve. Works in both chat and CLI contexts.

**Edge cases:**
- Unknown role: `@foo do X` → orchestrator replies "No specialist registered for 'foo'. Known roles: research, infra, test, code, design. Create one?"
- Ambiguous: `@research @infra check server logs` → first @-mention wins as primary, second becomes a tag/collaborator

### 2. Explicit CLI Command
For users who prefer structured commands over chat-style:

```
/delegate research "find how Rust async executors handle cancellation"
/delegate infra --priority high "check disk on .107"
/ask research "what's the state of the art for X?"
```

**Parsing:** Standard command parsing. `/delegate <role> [flags] "<prompt>"`. The `/ask` variant implies a synchronous request-response (maps to `termlink agent ask`), while `/delegate` implies fire-and-forget with result posted to `fw bus`.

**Advantages:** Explicit, scriptable, supports flags (priority, timeout, output format).

### 3. Natural Language with Intent Detection
The human speaks naturally and the orchestrator infers delegation need:

```
"send this to the research agent"
"I need infra help with the deployment"
"can you have someone look into the test failures?"
```

**Parsing:** Pattern matching on trigger phrases:
- `send * to the <role> agent` → extract role + preceding context as prompt
- `I need <role> help` → extract role, ask for task details or use surrounding context
- `have someone <verb>` → orchestrator chooses role based on verb (look into → research, fix → code, deploy → infra)

**Advantages:** Most natural. Lowest friction for conversational users.
**Disadvantages:** Ambiguity. Requires fallback: "I think you want @research — confirm?"

## Recommended UX Design

**Layer the three patterns** with clear precedence:

| Priority | Pattern | Confidence | Confirmation needed? |
|----------|---------|-----------|---------------------|
| 1 | `@role prompt` | High | No — direct dispatch |
| 2 | `/delegate role "prompt"` | High | No — explicit command |
| 3 | Natural language trigger | Medium | Yes — confirm role before dispatch |

**Response contract:** After dispatching, the orchestrator immediately confirms:
```
→ Delegated to @research: "find how Rust async executors handle cancellation"
  Tracking as R-042. Use /status R-042 to check progress.
```

## Implementation Sketch

The orchestrator needs:
1. **Role registry** — `specialists.yaml` mapping role names to agent configs, CLAUDE.md overlays, and capability tags
2. **Intent parser** — regex-based for @-mentions and /commands; LLM-assisted for natural language (with confirmation gate)
3. **Dispatch bridge** — maps parsed intent to `termlink spawn` + `termlink agent ask` or `fw bus post`
4. **Result routing** — specialist posts result via `fw bus post`; orchestrator surfaces summary to human

## Key Design Decision

**@-mention should be the primary pattern.** It's high-confidence (no confirmation needed), zero-learning-curve, and composable with existing chat UX. Natural language is a convenience layer on top, not the foundation.
