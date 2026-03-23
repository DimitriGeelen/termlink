# T-240 Q2: Multi-Round Exchange Feasibility on Existing Primitives

## Summary

TermLink's existing `agent.request`/`agent.response`/`agent.status` primitives **can support multi-round correlated negotiation without new RPCs**, but with ergonomic friction. The building blocks are all present; what's missing is a higher-level convention, not infrastructure.

## Primitive Analysis

### Event Schemas (`events.rs`)

The agent message protocol defines three event types:

| Event | Key Fields | Reference |
|-------|-----------|-----------|
| `agent.request` | `request_id`, `from`, `to`, `action`, `params` | `events.rs:137-155` |
| `agent.response` | `request_id`, `from`, `status`, `result`, `error_message` | `events.rs:161-176` |
| `agent.status` | `request_id`, `from`, `phase`, `message`, `percent` | `events.rs:181-197` |

**Critical observation:** All three types share a `request_id` field for correlation (`events.rs:142,165,185`). The `params` and `result` fields are both `serde_json::Value` — completely untyped. The `phase` field on `AgentStatus` is a free-form `String` (`events.rs:189`).

### CLI: `agent ask` (`agent.rs:12-146`)

1. Generates a unique `request_id` via `generate_request_id()` (`agent.rs:23`)
2. Emits an `agent.request` event to the target's event bus via `event.emit` RPC (`agent.rs:60`)
3. Polls the **same session's** event bus for `agent.response` or `agent.status` matching `request_id` (`agent.rs:82-145`)
4. On receiving `agent.response`, prints result and **exits** (`agent.rs:104-117`)
5. On receiving `agent.status`, prints the phase/message and **continues polling** (`agent.rs:119-124`)

**Limitation:** `agent ask` is a single-shot command — it exits on the first `agent.response`. It cannot send a follow-up request correlated to the same conversation.

### CLI: `agent listen` (`agent.rs:148-218`)

1. Polls the target session's event bus filtered to `agent.request` topic only (`agent.rs:174`)
2. Prints each incoming request's `from`, `action`, `request_id` (`agent.rs:186-188`)
3. **Does not respond** — it's read-only, a monitoring tool (`agent.rs:185-194`)

**Limitation:** `agent listen` is observe-only. There's no built-in "listen and respond" loop.

### RPC Methods (`control.rs`)

Relevant RPCs for multi-round exchange:

- `event.emit` (`control.rs:17`) — emit event to a specific session's bus
- `event.poll` (`control.rs:18`) — poll events with cursor-based pagination
- `event.broadcast` (`control.rs:22`) — emit to multiple sessions via hub
- `event.collect` (`control.rs:23`) — fan-in collect from multiple sessions

No negotiation-specific RPCs exist. No RPC restricts what topics or payloads can be emitted — `event.emit` accepts arbitrary topic+payload.

## Answers to Specific Questions

### Can `agent ask/listen` handle multiple rounds with the same `request_id`?

**No, not as currently implemented.** `agent ask` exits on the first `agent.response` (`agent.rs:116: return Ok(())`). `agent listen` is read-only. However, the underlying **event bus** supports it — nothing prevents emitting multiple `agent.request` events with the same `request_id`. The constraint is in the CLI commands, not the protocol.

### Does `agent listen` support responding to a specific `request_id`?

**No.** `agent listen` only prints requests (`agent.rs:186-191`). It doesn't emit responses. However, `termlink emit` can send an `agent.response` to any session with any `request_id`:

```bash
termlink emit <requester-session> agent.response \
  '{"request_id":"<id>","from":"responder","status":"ok","result":{...}}'
```

This works because `cmd_emit` (`events.rs:52-80`) accepts arbitrary topic+payload and calls `event.emit` RPC directly.

### Can an agent emit `agent.status` events with custom phases?

**Yes.** The `phase` field is `String` (`events.rs:189`), not an enum. Any value works: `"negotiating"`, `"counter-offer"`, `"awaiting-approval"`. The `agent ask` command already displays status events with arbitrary phases (`agent.rs:122: eprintln!("[status] {}{}: {}", status.phase, ...)`).

### Can correlated multi-round exchange work without new RPC methods?

**Yes.** The protocol layer supports it fully:

1. **Correlation:** `request_id` is a free-form string on all three event types
2. **Routing:** `event.emit` can target any session, `event.poll` with `since` cursor supports continuous consumption
3. **Payload:** `params` and `result` are untyped JSON — can carry round numbers, counter-proposals, etc.
4. **Status signaling:** `agent.status` with custom `phase` values can signal negotiation state transitions

What's missing is CLI ergonomics for multi-round, not protocol capability.

## Minimal Spike: 2-Round Negotiation (Pseudocode)

**Scenario:** Session A proposes a format to Session B; B counter-proposes; A accepts.

### Convention

Use `action` field to distinguish rounds: `negotiate.propose`, `negotiate.counter`, `negotiate.accept`. All share the same `request_id` for correlation.

### Initiator (Session A)

```bash
# Round 1: Propose
REQ_ID=$(termlink agent ask sessionB negotiate.propose \
  --params '{"format":"json","version":"1.0"}' \
  --timeout 30)
# agent ask exits with B's counter-proposal in result

# Round 2: Accept (using emit, since agent ask generates new request_id)
termlink emit sessionB agent.request \
  '{"request_id":"'$REQ_ID'","from":"sessionA","to":"sessionB",
    "action":"negotiate.accept","params":{"accepted_format":"yaml"}}'

# Poll for final confirmation
termlink events sessionA --topic agent.response --timeout 10
```

### Responder (Session B) — scripted

```bash
# Listen for proposals (in a script/hook, not interactive)
while true; do
  REQ=$(termlink events sessionB --topic agent.request --timeout 60 --json)
  ACTION=$(echo "$REQ" | jq -r '.action')
  REQ_ID=$(echo "$REQ" | jq -r '.request_id')
  FROM=$(echo "$REQ" | jq -r '.from')

  case "$ACTION" in
    negotiate.propose)
      # Counter-propose
      termlink emit "$FROM" agent.response \
        '{"request_id":"'$REQ_ID'","from":"sessionB",
          "status":"ok","result":{"counter":"yaml","version":"2.0"}}'
      ;;
    negotiate.accept)
      # Confirm
      termlink emit "$FROM" agent.response \
        '{"request_id":"'$REQ_ID'","from":"sessionB",
          "status":"ok","result":{"negotiated":"yaml","version":"2.0"}}'
      ;;
  esac
done
```

### Friction Points

1. **`agent ask` is single-shot** — Round 2+ requires raw `termlink emit` + manual polling
2. **No built-in respond command** — `agent listen` is read-only; responding requires `termlink emit` with manual payload construction
3. **No conversation state** — each round is stateless; the script must track `request_id` correlation
4. **Polling overhead** — both sides must poll; no push notification

## Feasibility Verdict

| Aspect | Status |
|--------|--------|
| Protocol support | **Full** — `request_id` correlation, untyped payloads, free-form phases |
| CLI support | **Partial** — works via `emit` + `events`, but no dedicated multi-round commands |
| New RPCs needed | **None** — `event.emit` + `event.poll` are sufficient |
| Ergonomic cost | **Medium** — multi-round requires shell scripting around raw emit/poll |

**Recommendation:** Multi-round negotiation is feasible on existing primitives. A 4-phase format negotiation protocol can be built as a **convention layer** (action naming + payload schemas) without any Rust changes. CLI convenience commands (`agent respond`, `agent negotiate`) would improve ergonomics but are not blockers for a spike.
