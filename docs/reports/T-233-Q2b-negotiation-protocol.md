# T-233 Q2b: Negotiation Protocol Design

## The Problem

When an agent needs specialist help (e.g., formatting output, structuring a config), the interaction isn't a single request-response. It's a **dialogue**: the orchestrator introduces the specialist and its expected format, the agent attempts compliance, the specialist corrects, and they iterate 2-3 rounds until alignment. This is schema negotiation.

## Protocol: Format Negotiation over Agent Events

### Message Types

Built on existing `agent.request`/`agent.response`/`agent.status` primitives, using the `action` field to distinguish negotiation phases:

| Phase | Action | Direction | Payload |
|-------|--------|-----------|---------|
| 1. Introduce | `negotiate.offer` | Orchestrator -> Agent | `{specialist_id, format_schema, example, constraints}` |
| 2. Attempt | `negotiate.attempt` | Agent -> Specialist | `{draft: Value, questions: [String]}` |
| 3. Correct | `negotiate.correction` | Specialist -> Agent | `{accepted: bool, fixes: [{field, expected, got, hint}], revised_schema?: Schema}` |
| 4. Accept | `negotiate.accept` | Specialist -> Agent | `{final_schema, template}` |

All messages share the same `request_id` — the negotiation is one logical request with multiple status rounds.

### Format Specification Shape

```json
{
  "format_id": "specialist/report-v2",
  "schema": {
    "type": "object",
    "required": ["title", "findings", "severity"],
    "properties": {
      "title": {"type": "string", "maxLength": 80},
      "findings": {"type": "array", "items": {"$ref": "#/finding"}},
      "severity": {"enum": ["info", "warn", "error"]}
    }
  },
  "example": {"title": "...", "findings": [...], "severity": "warn"},
  "constraints": ["findings must reference file:line", "severity derives from worst finding"]
}
```

JSON Schema is the wire format for structural validation. The `constraints` array carries semantic rules that can't be expressed in schema alone.

### Correction Signaling

The specialist returns a `negotiate.correction` with:
- **`accepted: false`** + `fixes[]` — agent must revise and resubmit
- **`accepted: true`** — negotiation complete, followed by `negotiate.accept`

Each fix entry is surgical: `{field: "findings[0].ref", expected: "file:line format", got: "line 42", hint: "prefix with filename"}`. This lets the agent patch incrementally rather than guess.

### Completion Detection

The agent knows negotiation is done when it receives `negotiate.accept`. It then caches `final_schema` locally for future interactions with that specialist — no renegotiation needed until the specialist's format version changes.

**Timeout:** If no correction arrives within the request's `timeout_secs`, the agent falls back to its best-effort draft and emits `agent.status` with `phase: "negotiation-timeout"`.

**Max rounds:** Hard cap of 5 correction cycles. If not converged, the specialist sends `negotiate.correction` with `accepted: false` and an `error_message` explaining the impasse.

### Mapping to TermLink Primitives

| Concept | TermLink Primitive |
|---------|-------------------|
| Negotiation session | Single `request_id` across all messages |
| Message exchange | `event.emit` to target session's bus |
| Polling for replies | `event.poll` with `topic: "agent.response"`, filtered by `request_id` |
| Progress tracking | `agent.status` events with `phase: "negotiating"` |
| Orchestrator routing | Orchestrator emits `negotiate.offer` to agent, gives specialist's session ID for direct dialogue |

The orchestrator **brokers the introduction** (phase 1), then the agent and specialist talk directly (phases 2-4). This avoids the orchestrator becoming a bottleneck during iteration.

### Key Design Decision

**Direct agent-specialist dialogue** (not relayed through orchestrator). The orchestrator introduces parties, then steps back. This halves message hops during correction rounds and lets the orchestrator manage other agents concurrently. The orchestrator monitors via `agent.status` events — it sees `phase: "negotiating"` updates but doesn't relay content.
