# T-256 Q3: Claude Code Execution Model Constraints for TermLink Integration

## Summary

Claude Code's Bash tool supports long-running blocking commands (up to 10 minutes), background execution, and parallel tool calls. These capabilities align well with TermLink's event-driven primitives (`wait`, `collect`, `request`) but introduce context budget trade-offs that shape the optimal integration pattern.

---

## Q1: Can an Agent Block on `termlink event wait --timeout 300`?

**Yes, with constraints.**

The Bash tool accepts a `timeout` parameter up to 600,000ms (10 minutes). A `termlink event wait --topic task.done --timeout 300 worker-1` command (5 minutes max) fits within this limit. The tool blocks the agent's turn until the command completes or times out.

**Practical constraints:**
- **Foreground blocking is exclusive.** While a foreground Bash call blocks, the agent cannot process other tool calls or respond to the user. The entire turn is suspended.
- **`run_in_background: true` solves this.** Background Bash calls return immediately, and the agent is notified asynchronously when the command completes. The agent can continue working (making other tool calls, responding to the user) while the wait runs.
- **No stdin interaction.** The Bash tool provides no interactive stdin, so blocking commands must be fully parameterized (TermLink's CLI commands already are).

**Recommendation:** Always use `run_in_background: true` for any `termlink wait`, `termlink collect`, or `termlink request` call. Reserve foreground execution for commands expected to return in under 2 seconds.

---

## Q2: Can Multiple Blocking Waits Run in Parallel as Background Tasks?

**Yes.** Multiple `run_in_background: true` Bash calls can be issued in a single turn. Each runs independently and notifies the agent on completion.

**Example — fan-in from 3 workers:**

```
# Turn 1: Issue all three waits in parallel
Bash(run_in_background=true): termlink event wait --topic task.done --timeout 300 worker-1
Bash(run_in_background=true): termlink event wait --topic task.done --timeout 300 worker-2
Bash(run_in_background=true): termlink event wait --topic task.done --timeout 300 worker-3
```

The agent is notified as each completes. It can process results incrementally or wait for all three.

**Alternative — single `collect` call:**

```
Bash(run_in_background=true): termlink event collect --topic task.done --count 3 --timeout 300
```

`collect` does hub-level fan-in: it listens across multiple sessions and exits after receiving `--count` matching events. This is more efficient than N individual waits when the set of workers is dynamic or the agent doesn't need per-worker ordering.

**Trade-off:**
| Pattern | Pros | Cons |
|---------|------|------|
| N × `wait` | Per-worker result ordering, partial failure isolation | N background tasks, N notifications |
| 1 × `collect --count N` | Single task, hub-level aggregation | All-or-nothing timeout, less control over ordering |

---

## Q3: Claude Code Agent Tool vs TermLink Spawn

The Agent tool (Claude Code's sub-agent mechanism) and `termlink spawn` serve different roles:

| Dimension | Agent tool | termlink spawn |
|-----------|-----------|----------------|
| **What it creates** | A new Claude conversation (sub-agent) within the same process | A new terminal/tmux/background process with its own TermLink session |
| **Communication** | Returns a single result blob to the parent when complete | Bidirectional: events, topics, send/receive, request-reply throughout lifetime |
| **Lifetime** | Scoped to a single task; destroyed after returning | Independent process; persists until deregistered or killed |
| **Context** | Shares nothing — parent must provide full context in the prompt | Shares nothing by default — must communicate via TermLink protocol |
| **Parallelism** | Up to ~5 concurrent (framework dispatch protocol limit) | Limited only by system resources |
| **Orchestrator cost** | Result ingested into parent context (tokens consumed) | Result stays on disk or in event bus; parent reads only what it needs |

**Key insight:** The Agent tool is "spawn and forget with a single return." TermLink spawn is "spawn and interact throughout." They are complementary:

- **Agent tool** is ideal for self-contained research/exploration tasks that produce a single artifact.
- **termlink spawn** is ideal for worker processes that report progress, receive mid-flight instructions, or participate in multi-step protocols.

The current dispatch.sh already uses `termlink spawn` for worker Claude instances. The gap is in the *communication back* — workers currently write files and the orchestrator polls. TermLink events (`emit` + `wait`/`collect`) fill this gap without requiring the Agent tool at all.

---

## Q4: The Spawn-Then-Collect Pattern

**Pattern:** Orchestrator spawns N workers via `termlink spawn`, then blocks on `termlink event collect --count N --timeout T`.

```bash
# 1. Spawn workers (each runs a Claude instance with a task)
termlink spawn --name worker-1 --tags "task:T-256" --wait -- claude -c "do X, then: termlink event emit self task.done --payload '{...}'"
termlink spawn --name worker-2 --tags "task:T-256" --wait -- claude -c "do Y, then: termlink event emit self task.done --payload '{...}'"
termlink spawn --name worker-3 --tags "task:T-256" --wait -- claude -c "do Z, then: termlink event emit self task.done --payload '{...}'"

# 2. Collect results (single blocking call)
termlink event collect --topic task.done --count 3 --timeout 300
```

**This works today** with the existing CLI. The `--wait` flag on spawn ensures registration completes before the orchestrator moves on. Workers emit `task.done` events with payloads containing result summaries or file paths. The orchestrator's `collect` call gathers all three.

**Refinement with `request`:** For request-reply semantics (orchestrator sends a task, waits for that specific reply):

```bash
termlink request --topic task.delegate --reply-topic task.completed --timeout 300 worker-1 \
  --payload '{"task": "T-256", "action": "research Q1"}'
```

`request` combines emit + wait into a single atomic operation. It emits to the target, then polls for the reply topic. This is cleaner for 1:1 delegation but doesn't support fan-in (use `collect` for that).

---

## Q5: Context Budget Impact — Waiting vs Polling

**Waiting (background `termlink wait`/`collect`):**
- **Zero context cost while waiting.** A background Bash call consumes no tokens until it returns.
- **One-shot result ingestion.** The notification delivers the command output (event payload) in a single chunk.
- **Budget-predictable.** The orchestrator knows exactly when results arrive and can plan token spend.

**Polling (`termlink event poll` in a loop):**
- **Each poll iteration is a tool call.** Every Bash invocation adds ~200-500 tokens (call + result framing).
- **10 polls × 3 workers = 30 tool calls** = ~6K-15K tokens of overhead before any useful data.
- **Budget-unpredictable.** The orchestrator doesn't know when to stop polling.

**Quantified comparison (3 workers, 60-second average wait):**

| Approach | Tool calls | Estimated token overhead |
|----------|-----------|------------------------|
| 3 × background `wait` | 3 (launch) + 3 (notify) = 6 | ~2K |
| 1 × background `collect --count 3` | 1 (launch) + 1 (notify) = 2 | ~800 |
| Poll loop (5s interval, 60s avg) | 3 × 12 = 36 | ~10K-18K |

**The budget-gate hook (P-009) makes this critical.** At 150K+ tokens, Write/Edit/Bash are blocked. Wasting 10K-18K tokens on polling overhead brings the critical threshold closer by ~7-12%. With a 200K effective budget, that's the difference between completing 4 tasks and completing 3.

**Recommendation:** Never poll. Always use `wait` or `collect` with `run_in_background: true`. The context savings compound across multi-worker dispatches.

---

## Constraints and Caveats

1. **10-minute hard ceiling.** The Bash tool's 600,000ms timeout means no single wait can exceed ~10 minutes. For longer tasks, workers should emit progress events and the orchestrator should chain shorter waits.

2. **No streaming.** Background Bash delivers results only on completion. `termlink event watch` (continuous streaming) would block forever as a foreground call or deliver all output at once when killed. For real-time progress, use periodic `emit` + `collect` with increasing `--count`.

3. **Background task notification timing.** The agent is notified "when it completes" but this notification is processed in the agent's turn flow. If the agent is mid-turn doing other work, the notification queues. This is fine for fan-in patterns but means latency between event arrival and agent processing is non-deterministic.

4. **Agent tool isolation.** Sub-agents spawned via Claude Code's Agent tool cannot directly access TermLink sessions from the parent. If a sub-agent needs TermLink access, it must register its own session. This makes the Agent tool less suitable for TermLink-integrated workflows — prefer `termlink spawn` for workers that need mesh participation.

---

## Recommended Integration Pattern

```
Orchestrator (Claude Code agent with TermLink session)
  │
  ├── termlink spawn --wait worker-1 -- claude -c "..."
  ├── termlink spawn --wait worker-2 -- claude -c "..."
  └── termlink spawn --wait worker-3 -- claude -c "..."
       │
       │  (workers do work, emit events)
       │
  └── Bash(background): termlink event collect --topic task.done --count 3 --timeout 300
       │
       └── (agent notified with all 3 results, processes in single turn)
```

This pattern: zero-cost waiting, single notification, minimal token overhead, full TermLink bidirectional communication available to workers throughout their lifetime.
