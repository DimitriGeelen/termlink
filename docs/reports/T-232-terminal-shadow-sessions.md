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

### 2026-03-23 — Scope & UX decisions

**Q1: Single session or multi?**
- **A:** Start with single session. Multi-session (split/tabbed) as later tasks.

**Q2: Raw output or rendered?**
- **A:** Both, selectable. Start with raw (`--raw` default), but expect it to look messy with escape codes. Spawn a separate research task for clean/rendered output mode.

**Q3: Local only or also remote?**
- **A:** Both local (Unix socket) and remote (TCP hub). Definitely.

**Q4: Name?**
- **A:** "mirror" — `termlink mirror <session>`. Resonates better than shadow/monitor/watch/tail.

### Derived task decomposition

From these decisions, the build work breaks down as:

1. **T-build-1**: `termlink mirror <session>` — read-only data plane streaming (single session, raw output, local + remote)
2. **T-research**: Research clean terminal output rendering for mirror mode (escape sequence filtering, or embedded terminal emulator)
3. **T-build-2** (later): `termlink mirror --rendered` — clean output mode based on research findings
4. **T-build-3** (later): Multi-session mirror — `termlink mirror --tag agent` with split/tabbed TUI

## Architecture Assessment

### Broadcast channel multi-subscriber (A-2 validation)

`tokio::broadcast::channel` natively supports multiple receivers. Each call to `tx.subscribe()` creates an independent receiver. Existing interactive client uses one receiver; mirror clients each get their own. **No refactor needed — this is a subscribe call.**

### Data plane read-only mode

Current flow in `data_server.rs`:
1. Client connects to `.sock.data`
2. Handler spawns read loop (PTY→client Output frames) and write loop (client Input→PTY)
3. Both directions are always active

For mirror mode:
- Skip the write loop (don't read Input frames from client)
- Or: read frames but reject Input/Signal/Resize types with an error frame
- Handshake parameter: `mode: "mirror"` vs `mode: "interactive"` (or derive from auth scope)

### Permission model

`Observe` scope (tier 0) is the natural fit. Currently Observe only covers control plane. Extending it to allow data plane read-only access is clean — no new scope needed, just widen Observe to include data plane subscription.

### Effort estimate

| Component | Lines | Complexity |
|---|---|---|
| Data plane mirror mode (handshake + skip write loop) | ~80 | Low |
| CLI `cmd_mirror()` (similar to `cmd_stream()` minus input) | ~100 | Low |
| Permission: allow Observe on data plane | ~20 | Low |
| Hub forwarding for remote mirror | ~50 | Medium (reuse TCP relay) |
| Tests | ~150 | Medium |
| **Total** | **~400** | **Low-Medium** |

## Go/No-Go Assessment

**GO criteria check:**
- Broadcast channel natively supports multi-subscriber: **YES**
- Clear UX distinct from attach: **YES** — `mirror` is read-only, `attach` is interactive
- Estimated build < 400 lines: **~400 lines** — at the boundary but manageable

**NO-GO criteria check:**
- Data plane needs fundamental redesign: **NO** — subscribe() is native
- Permission model needs breaking changes: **NO** — widen Observe scope
- Polling is "good enough": **NO** — polling adds latency and load, streaming is superior

**Recommendation: GO**

