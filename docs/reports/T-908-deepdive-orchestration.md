# T-908 Deep Dive: Orchestration Stack and Relay Routing Convergence

**Task:** T-908 (Worker 2: Orchestration analysis)
**Date:** 2026-04-09
**Sources:** T-233 (specialist orchestration inception), T-903 (task-type routing), T-904 (model routing), T-905 (governance subscriber), T-906 (model dispatch param), T-908 (API relay governance)

---

## 1. How the Relay Completes the Orchestration Stack

TermLink's orchestration stack today has four layers built across T-233, T-903, and T-904, implemented in `crates/termlink-hub/src/router.rs` and supporting modules:

| Layer | Component | File | Purpose |
|-------|-----------|------|---------|
| 0 | Task-gate governance | `tools.rs` (T-902) | Pre-flight: reject MCP tool calls missing `task_id` when `TERMLINK_TASK_GOVERNANCE=1` |
| 1 | Bypass registry | `bypass.rs` (T-233) | Fast path: skip routing for commands with Tier 3 promotion (5+ successes, 0 failures, not denylisted) |
| 2 | Route cache | `route_cache.rs` (T-233) | Warm path: confidence-scored (threshold 0.8), TTL-based (7-day default) specialist routing with decay (0.05/week) |
| 3 | Discovery + selection | `router.rs` (T-233/T-903) | Cold path: enumerate sessions, filter by selector, stable-sort by `task-type:<type>` tag preference |
| 4 | Circuit breaker | `circuit_breaker.rs` (T-233/T-904) | Health: per-session (3 failures, 60s cooldown) + per-model (fallback chain opus->sonnet->haiku) |

After T-903, layers 1-3 operate on a **composite routing key** (`method::task_type` when task_type is present, plain `method` otherwise). This gives the same method dispatched for `build` vs `test` workflows independent routing paths, separate cache entries, and separate bypass promotion tracks.

**The structural gap.** This stack controls which session handles a request and which model a worker should prefer. But model selection happens at dispatch time (when the worker is spawned), not at request time (when the API call is made):

```
orchestrator.route selects specialist
  -> dispatch sets TERMLINK_MODEL=sonnet in worker env
    -> Claude Code worker starts
      -> worker makes N API calls to api.anthropic.com
        -> ALL calls use whatever model Claude Code's config selects
        -> TERMLINK_MODEL env var is IGNORED by Claude Code
```

The relay closes this gap by sitting on the wire:

```
orchestrator.route selects specialist
  -> dispatch sets ANTHROPIC_BASE_URL=http://localhost:PORT
    -> Claude Code worker starts
      -> worker makes N API calls to localhost:PORT (the relay)
        -> relay reads routing rules, rewrites model parameter
          -> relay forwards to api.anthropic.com with chosen model
```

The critical difference: `TERMLINK_MODEL` is advisory (Claude Code ignores it), while `ANTHROPIC_BASE_URL` is structural (the Anthropic SDK reads it natively -- confirmed in T-908 Spike 2 via binary string extraction from Claude Code v2.1.97 and SDK source analysis). The relay transforms an env-var hint into wire-level enforcement, becoming **Layer -1** in the stack.

## 2. Per-Request vs Per-Dispatch Model Routing

### Per-dispatch (T-904 today)

The orchestrator selects a model before spawning the worker. The model persists for the worker's entire lifetime. Implemented via:

- `DispatchParams.model` field in MCP tools (`tools.rs:400-403`) and CLI (`cli.rs:760-762`)
- `TERMLINK_MODEL` env var exported into the worker shell (`dispatch.rs:256-260`)
- `ModelCircuitBreaker.resolve_model()` walking the fallback chain defined as `DEFAULT_MODEL_FALLBACK: &["opus", "sonnet", "haiku"]`
- `RouteCache.model_stats` tracking success rates by `model:task_type` composite key with `best_model_for()` queries

Per-dispatch works for coarse-grained decisions: "use opus for build tasks, sonnet for test tasks." The model circuit breaker handles unavailability gracefully -- 3 consecutive failures open the circuit, after 60s cooldown a half-open probe is allowed.

### Per-request (relay)

The relay intercepts every API call and applies routing rules independently:

| Capability | Per-dispatch (T-904) | Per-request (relay) |
|-----------|---------------------|-------------------|
| Granularity | Once per worker lifetime | Every API call |
| Mid-session switching | No -- env is fixed at spawn | Yes -- relay decides per request |
| Cost optimization | Coarse (all calls same tier) | Fine (thinking on opus, confirmations on haiku) |
| Rate-limit fallback | Worker dies, dispatch retries | Transparent re-route, session continues |
| Context-aware escalation | No | Yes -- escalate to opus when complexity detected |

**Mid-conversation model switching.** A worker starts on sonnet for routine tool execution. The relay detects a complex multi-file refactoring attempt and escalates to opus for that specific request. Subsequent simple Read calls continue on sonnet. The worker is unaware.

**Token-based routing.** The relay sees the full request body including the messages array. Short prompts with few messages go to haiku; long prompts with extensive tool results go to opus. The dispatch layer cannot make this decision because it runs before any API calls happen.

**Provider fallback without agent awareness.** If Anthropic returns 529 (overloaded), the relay transparently retries to Bedrock. The agent made one request and gets one response. Per-dispatch fallback requires re-dispatching the entire task, losing conversation state.

### Trade-offs

Per-request routing introduces coherence risk. If the model changes between turns, the new model sees message history but has no memory of its own prior reasoning patterns. For tool-use patterns (read output, decide next action) this is fine. For extended reasoning chains building on subtle contextual cues, a model switch could degrade quality.

**Mitigation:** Switch only at clean boundaries (after complete tool-use cycles, not mid-thought). Only downgrade (opus to sonnet, not cross-family). The practical architecture uses both: per-dispatch for session-level model selection, per-request for cost optimization and escalation.

## 3. Convergence: Governance + Routing + Observability in One Component

### Three concerns, one observation point

The relay is architecturally unique -- no other component sees every API request AND every SSE response event in real time:

- Layer 1 sees task dispatch events (one per task)
- Layer 2 sees model preference signals (one per dispatch)
- T-905's `GovernanceSubscriber` sees terminal output frames (post-hoc, regex-based pattern detection on PTY output)
- T-902's `check_task_governance()` sees MCP tool calls (only TermLink tools, not native Claude Code tools)
- **The relay** sees every `/v1/messages` request and every SSE response event from every agent

### Governance at the wire

From T-908 Spike 1, the SSE `content_block_start` event contains the tool name and ID immediately:

```
event: content_block_start
data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_01...","name":"get_weather","input":{}}}
```

This enables fast-gate decisions without buffering. For content-aware gating (block Write to specific files), the relay buffers `input_json_delta` fragments until `content_block_stop`, then inspects accumulated JSON.

When blocked, the relay strips the entire block (start through stop) from the SSE stream and injects a text block containing the governance message. Claude Code sees text, not a tool call, and adjusts behavior.

This is the only enforcement surface covering ALL tool calls -- including Claude Code's native tools (Bash, Write, Edit, Read) that bypass TermLink's RPC surface entirely.

### How the three concerns interact

```
Request path:  Worker -> Relay -> [ROUTING: select model, rewrite] -> Anthropic API
Response path: API -> Relay -> [OBSERVABILITY: log tokens, latency, model]
                            -> [GOVERNANCE: inspect tool_use blocks, gate/strip]
                            -> [OBSERVABILITY: log governance actions]
                            -> Worker (filtered SSE stream)
```

Governance and routing never conflict because they operate on different phases. Routing acts on the request (which model). Governance acts on the response (which tool calls to allow). A blocked tool_use does not trigger routing reconsideration.

Observability wraps both: it logs the original request (including routing decisions), the response (what the model tried), and governance actions (what was blocked and why). This creates an audit trail impossible to construct from Claude Code's perspective -- by the time Claude Code sees the rewritten stream, blocked tool calls never existed.

**Governance informing routing.** When a tool call is blocked, the relay can inject a system prompt amendment into the next request: "Your previous Write attempt was blocked because no active task exists." More effective than silently stripping the tool call.

**Routing metadata enriching governance.** The relay knows: which task is active (from `TERMLINK_DISPATCH_ID`), whether this is a subagent (from `TERMLINK_ORCHESTRATOR`), which model tier is in use (haiku calls might get looser governance -- used for simple tasks).

## 4. Integration with Existing TermLink Primitives

### Relay as a TermLink session

The relay registers with the hub as a standard session:

```json
{
  "display_name": "relay",
  "roles": ["infrastructure"],
  "tags": ["api-relay", "governance"],
  "capabilities": ["proxy", "governance", "observability"]
}
```

Discoverable via `session.discover`, visible in `termlink ls`. The orchestrator verifies the relay is running before dispatching workers. Liveness checked via standard heartbeat.

### Event emission

The relay emits events through the hub's `event.broadcast` / `event.emit_to` RPCs:

| Event | Payload | Consumers |
|-------|---------|-----------|
| `api.request` | model, tokens_in, task_id, session_id | Cost tracking, route cache updates |
| `api.response` | tokens_out, latency_ms, stop_reason | Model stats, circuit breaker feedback |
| `governance.blocked` | tool_name, rule, task_id | Audit trail, unified with T-905 governance events |
| `governance.rewritten` | original_tool, injected_text | Debug, policy tuning |

The T-905 `GovernanceSubscriber` could consume `governance.blocked` events alongside its PTY-level pattern detection (`GovernanceEvent` frames), creating a unified audit log spanning both data plane and API plane.

### Route cache feedback loop

**Read:** The relay reads `RouteCache.model_stats` to inform per-request model selection. When `best_model_for("build")` returns "opus" based on accumulated success data, the relay uses it.

**Write:** After each API response, the relay calls `record_model_success` / `record_model_failure`. This creates a bidirectional feedback loop: dispatch decisions and per-request decisions share a single source of truth for model performance.

### Circuit breaker coordination

The existing `ModelCircuitBreaker` tracks model availability at the dispatch level. The relay sees failures at finer granularity -- API 429s, 529s, timeouts. It feeds these into the model circuit breaker, enabling the orchestrator to stop dispatching to a model the relay has observed failing at the wire level. The state flows bidirectionally.

## 5. What This Enables That Does Not Exist Today

### True multi-LLM orchestration

`ModelCircuitBreaker.resolve_model()` walks `["opus", "sonnet", "haiku"]` -- all Anthropic. With a relay, routing maps model identifiers to different providers entirely. "opus" goes to Anthropic, "gpt-4o" to OpenAI, "gemini" to Google. Workers still send Anthropic-format requests to `ANTHROPIC_BASE_URL`; the relay translates. Format translation is substantial work but the architecture makes it possible without changing worker code.

### Cost governance

The relay knows every token spent. Budget caps become enforceable:
- **Per-task:** "T-123 may spend 500K tokens" (from task metadata)
- **Per-session:** "This worker has a 1M token budget"
- **Per-hour:** "No more than 5M tokens/hour across all sessions"

When a cap is hit: reject the request, switch to a cheaper model, or inject a wrap-up message. Today `budget-gate.sh` counts tokens by parsing JSONL transcripts after the fact. The relay provides real-time enforcement.

### A/B testing

Route a percentage of traffic to different models, compare outcomes. The relay assigns routing decisions, logs them; model stats track success rates. After enough data, auto-promote the better model per task type.

### Replay and debugging

Record every SSE stream (request + response) to disk. When a worker produces unexpected results, replay through modified governance rules. Especially valuable for debugging governance false positives.

## Summary

The API relay completes a design converging since T-233. The orchestrator knows which session should handle work (discovery + routing). T-903 added task-type awareness. T-904 added model preference with circuit breakers and success-rate tracking. The relay adds the final piece: wire-level enforcement of model choice, plus governance and observability operating on actual API traffic rather than advisory env vars and after-the-fact transcript parsing. The three concerns (governance, routing, observability) converge naturally in this component because they all require the same capability: sitting on the wire between worker and provider. No existing gateway fills this gap -- LiteLLM, Portkey, Bifrost, and ccproxy all lack streaming response-side hooks, making this a genuine capability differentiator.

## Open Questions

1. **Latency budget.** The relay adds one local hop per API call. Bifrost claims 11us for Go; what can a Rust SSE proxy achieve? Target budget?
2. **Stream buffer strategy.** For fast-gate (tool name only), zero buffering. For content-gate (inspect tool input JSON), buffering delays the stream. Forward `content_block_start` optimistically and hold deltas, or buffer entire blocks?
3. **Stream rewriting stability.** Can Claude Code handle a tool_use block replaced mid-stream with a text block without entering an unrecoverable state? Critical untested assumption.
4. **Multi-provider format translation.** How much Anthropic SSE format is provider-specific? Can a relay translate OpenAI streaming to Anthropic, or does multi-provider need a neutral SDK?
5. **State management.** Relay needs per-conversation state (accumulated tokens, governance decisions). In-memory (simple, lost on crash) or persisted (durable, complex)?
6. **ccproxy hybrid path.** Extend ccproxy (Python, existing plumbing, faster ship) or build `termlink-relay` (Rust, native stack, architecturally cleaner)?
7. **Subagent identification.** Subagents inherit `ANTHROPIC_BASE_URL`. Does the relay need to distinguish parent from subagents for different governance/routing rules? How?
8. **Graceful degradation.** If relay crashes, workers lose API access. Fall back to direct API (bypass governance, fail-open) or hard stop (fail-closed)?
9. **Prompt caching interaction.** Anthropic's prompt caching relies on request prefix stability. Does the relay's header injection or model rewriting invalidate cache hits?
10. **Layer 2 feedback latency.** How do per-request observations feed back into `ModelStats`? Direct write to route-cache.json risks contention with hub reads. Event bus adds latency. Periodic batch sync?
