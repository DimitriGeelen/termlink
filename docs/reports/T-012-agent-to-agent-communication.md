# T-012: Agent-to-Agent Communication via TermLink

## Research Question

How should TermLink enable the Agentic Engineering Framework to delegate tasks from an orchestrator agent to specialist agents in separate terminals — each with their own context window, governed by the same framework rules?

## The Core Insight: Context Budget is the Bottleneck

The framework's biggest constraint today is **context budget**. Claude Code's sub-agents (Task tool) all share one ~200K context window. At 150K the budget gate blocks. This means:

- Sub-agents compete for context space
- The orchestrator must ingest all sub-agent results back into its own context
- Complex tasks exhaust the window before completing
- The Sub-Agent Dispatch Protocol mitigates this ("write to disk, return path + summary") but doesn't solve it

**TermLink-backed agents solve this structurally:**
- Each specialist runs in its OWN terminal with a FRESH 200K context window
- Results come back as compact event payloads or KV entries, not full content
- The orchestrator's context budget is used for coordination only
- N specialists = N × 200K context capacity, not 200K shared

## Use Cases

### UC-1: Orchestrator dispatches task to running specialist
- Orchestrator in terminal A needs code review
- Specialist "reviewer" agent running in terminal B (registered TermLink session)
- Orchestrator sends task via TermLink event, specialist picks it up, writes results to disk, reports completion

### UC-2: Orchestrator spawns specialist on demand
- Orchestrator needs a test runner but none is registered
- `termlink spawn` starts a new terminal with specialist agent
- Specialist auto-registers, receives task, reports back, stays alive for future tasks

### UC-3: Fan-out / parallel delegation
- Orchestrator delegates to 3 specialists in parallel: reviewer, tester, security scanner
- Each works independently with full context window
- Results fan back via events, orchestrator synthesizes

### UC-4: Persistent specialist pool
- Long-running reviewer agent accumulates codebase knowledge across sessions
- Orchestrator spawns once, delegates repeatedly
- Specialist maintains warm context about the project

## Dialogue Log

### Q1 (User): What's the vision?
- **User wants:** Claude agent orchestrator sends tasks to specialist agents in other terminals, or spawns them
- **Key phrase:** "adhere to framework governance" — specialists must follow CLAUDE.md, task system, enforcement tiers
- **Course correction:** Not just TermLink feature — must integrate with `fw` CLI and framework governance

## Investigation Spikes

### Spike 1: Mapping Existing Primitives

| Agent Need | TermLink Primitive | Status |
|---|---|---|
| Find specialist | `discover --role reviewer` | Works |
| Send task | `emit <target> task.delegate --payload '{...}'` | Works (no ack) |
| Listen for tasks | `wait --topic task.delegate` or `watch --topic task.*` | Works |
| Report completion | `emit <orchestrator> task.completed --payload '{...}'` | Needs orchestrator session |
| Store results | `kv <target> set result '{...}'` | Works (small data) |
| Health check | `ping <target>` | Works |
| Spawn agent | — | **MISSING** |
| Request-reply | — | **MISSING** (need correlation ID) |
| Orchestrator identity | — | **PARTIAL** (orchestrator must register too) |

### Spike 2: Task Delegation Protocol

Standard event schema for agent-to-agent delegation:

```json
// task.delegate — orchestrator → specialist
{
  "request_id": "req-001",
  "task_id": "T-058",
  "action": "review",
  "scope": {
    "files": ["src/main.rs", "src/handler.rs"],
    "criteria": "security, error handling"
  },
  "reply_to": "orchestrator-session-name",
  "timeout_secs": 300
}

// task.accepted — specialist → orchestrator (via reply_to)
{
  "request_id": "req-001",
  "task_id": "T-058",
  "status": "accepted"
}

// task.completed — specialist → orchestrator
{
  "request_id": "req-001",
  "task_id": "T-058",
  "status": "completed",
  "result_path": "docs/reviews/T-058-review.md",
  "summary": "3 issues found: 1 critical (SQL injection), 2 minor"
}

// task.failed — specialist → orchestrator
{
  "request_id": "req-001",
  "task_id": "T-058",
  "status": "failed",
  "error": "Could not access file: permission denied"
}
```

**Key design rules:**
- `reply_to` enables the specialist to report back without hardcoding the orchestrator
- `result_path` follows the existing "write to disk, return path" convention
- `request_id` enables correlation (multiple concurrent delegations)
- `task_id` maps to framework task system (delegation = framework task)

### Spike 3: Framework Governance Integration

**How specialists respect governance:**

1. **Same CLAUDE.md** — Specialist runs in same project directory, inherits CLAUDE.md rules
2. **Task system** — Delegation creates a real framework task (T-XXX) on the specialist side
3. **Enforcement hooks** — PreToolUse hooks apply equally to specialist (same repo, same hooks)
4. **Commit cadence** — Specialist commits its own work with task references
5. **Authority model** — Specialists have INITIATIVE level only; orchestrator delegates initiative, not authority
6. **Audit trail** — Every delegation, acceptance, and completion is a TermLink event (observable, auditable)

**Framework integration points:**

| Framework Feature | Integration |
|---|---|
| Task system | `fw agent delegate T-XXX --to reviewer` creates sub-task on specialist |
| Result bus | Replaces `fw bus` for cross-process delegation (TermLink events > YAML files) |
| Context budget | Each specialist = separate budget, orchestrator stays lean |
| Audit | `fw agent audit` checks delegation history via TermLink event bus |
| Healing | Specialist failure → `task.failed` event → orchestrator healing loop |
| Episodic | Specialist generates episodic for its sub-task on completion |
| Handover | Specialist generates handover if long-running |

**The key governance constraint:**
- A delegated task is still a framework task with all the rules
- The specialist doesn't get to skip enforcement tiers, AC verification, or commit cadence
- The orchestrator can't delegate human-owned tasks or Tier 0 actions

### Spike 4: Agent Lifecycle

**Spawn patterns:**

```bash
# Ephemeral: do one task, exit
termlink spawn --name reviewer-1 --role reviewer \
  --command "claude -p 'Review the files listed in your task.delegate event. Write findings to docs/reviews/. Emit task.completed when done.'"

# Persistent: stay alive for multiple tasks
termlink spawn --name reviewer --role reviewer --persistent \
  --command "claude -c"  # interactive mode, watches for tasks

# Detached: background, no visible terminal
termlink spawn --name ci-runner --role tester --detach \
  --command "claude -p 'Run tests and report results'"
```

**Lifecycle events:**
- `session.ready` — Specialist is registered and accepting tasks
- `session.busy` — Currently processing a task
- `session.idle` — Waiting for tasks
- `session.stopping` — Graceful shutdown

**Health monitoring:**
- Orchestrator pings specialists periodically
- Dead specialists detected by `termlink clean`
- Orchestrator can re-spawn failed specialists

### Spike 5: What's Missing — Gap Analysis

| Gap | Priority | Complexity |
|---|---|---|
| `termlink spawn` command | **P0** — Can't start agents without it | Medium (terminal spawning + registration) |
| `termlink request` command (emit + wait) | **P1** — Request-reply is the core pattern | Low (compose existing emit + wait) |
| Orchestrator self-registration | **P1** — Orchestrator needs a session to receive replies | Low (register in background) |
| Task delegation event schema | **P1** — Convention, no code | Zero (documentation) |
| `fw agent` subcommands | **P2** — Framework convenience layer | Medium |
| Agent prompt templates | **P2** — Specialist role definitions | Low (markdown files) |
| Agent pool / load balancing | **P3** — Advanced, later | High |

## Practical Workflow Example

```bash
# === Terminal 1: Orchestrator (Claude Code) ===

# 1. Register orchestrator as TermLink session (background)
termlink register --name orchestrator --role orchestrator &

# 2. Spawn a reviewer specialist
termlink spawn --name reviewer-1 --role reviewer \
  --command "claude -p 'You are a code review specialist for this project.
    Register as TermLink session. Watch for task.delegate events.
    Review specified files. Write findings to docs/reviews/.
    Emit task.completed to the reply_to session when done.'"

# 3. Wait for specialist ready
termlink wait reviewer-1 --topic session.ready

# 4. Delegate
termlink emit reviewer-1 task.delegate --payload '{
  "request_id": "req-001",
  "task_id": "T-058",
  "action": "review",
  "scope": {"files": ["src/handler.rs"]},
  "reply_to": "orchestrator"
}'

# 5. Wait for result
termlink wait orchestrator --topic task.completed --timeout 300

# 6. Read result summary
termlink kv reviewer-1 get review-summary
```

## Phased Build Plan

### Phase 1: Foundation (this inception → build tasks)
- Define task delegation event schema (convention doc)
- Add `termlink spawn` command (open terminal + auto-register)
- Add `termlink request` command (emit + wait, request-reply sugar)
- Test with manual orchestration

### Phase 2: Framework Integration
- `fw agent spawn/list/delegate/results` subcommands
- Auto-task creation on delegation (specialist gets a real T-XXX)
- Specialist agent prompt templates (reviewer, tester, builder)
- Integration with `fw bus` (TermLink events as transport)

### Phase 3: Advanced Patterns
- Agent pools (multiple reviewers, round-robin via hub)
- Persistent specialists with warm context
- Cross-project agent sharing
- Delegation chains (specialist delegates to sub-specialist)

## Go/No-Go Assessment

**GO if:**
- [x] Context budget multiplier is real (each specialist = fresh 200K) — YES, structurally true
- [x] Existing TermLink primitives cover 70%+ of the pattern — YES (discover, emit, wait, kv, watch)
- [x] Framework governance can be enforced on specialists — YES (same repo, same CLAUDE.md, same hooks)
- [x] Phase 1 is bounded and deliverable — YES (spawn + request + convention doc)

**NO-GO if:**
- [ ] Terminal spawning is platform-dependent nightmare — Needs investigation but macOS/Linux/tmux are tractable
- [ ] Claude Code can't self-register as TermLink session — Needs testing but `register &` should work
- [ ] Event-based task delegation is too lossy — Mitigated by request-reply correlation

**Recommendation: GO** — Phase 1 is 3 bounded build tasks. The context budget multiplier alone justifies the investment. Governance integration is natural because specialists run in the same project.

## Decision

GO — Build Phase 1: task delegation event schema, `termlink spawn`, `termlink request`.
