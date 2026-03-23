# T-232: Terminal Shadow Sessions — Research Artifact

## Problem Statement

When multiple agents are running through a TermLink hub, the human operator has no way to see what's happening inside each agent's terminal in real-time. You can query snapshots (`termlink output`) or interactively attach (`termlink attach`), but there's no **read-only, non-intrusive monitoring** — a "shadow" that lets you read along without affecting the agent.

## What Exists Today

| Capability | Command | Mode | Limitation |
|---|---|---|---|
| Scrollback snapshot | `termlink output <session>` | One-shot read | No streaming, must poll |
| Interactive attach | `termlink attach <session>` | Bidirectional | Exclusive, sends keystrokes |
| Data plane stream | `termlink stream <session>` | Bidirectional | Exclusive, interactive |
| Event watch | `termlink watch` | Read-only | Events only, not terminal output |

### Key architectural facts:
- **Data plane** (`data_server.rs`): PTY output broadcasts to a `tokio::broadcast::channel(256)`. Currently one interactive client at a time.
- **Scrollback** (`scrollback.rs`): Ring buffer (1 MiB default), byte-oriented, `Arc<Mutex<ScrollbackBuffer>>`.
- **Permission model** (`auth.rs`): 4 tiers — Observe(0), Interact(1), Control(2), Execute(3). `Observe` covers control plane reads but NOT data plane.
- **Hub discovery** (`session.discover`): Already supports filtering by tags/roles.

## Dialogue Log

*(To be filled during inception dialogue with human)*

