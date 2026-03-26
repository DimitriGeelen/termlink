# T-209: fw dispatch — SSH-based Cross-Machine Agent Communication

**Decision:** GO (2026-03-21)
**Rationale:** SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.

## Problem

`fw termlink dispatch` was the only way to communicate with remote framework agents. Without TermLink installed, zero cross-machine agent communication capability existed.

## Key Findings

1. **File-based bus (`fw bus`) is local-only** — cannot reach another machine.
2. **SSH is already available on every machine** where the framework runs. Already authenticated (keys), encrypted (TLS), handles NAT/firewalls.
3. **Hub profiles already store `user@host` connection info** — the registry already exists.
4. **`fw bus` envelopes (YAML with metadata) already exist** — the protocol already exists.
5. **The gap is literally one script (~80 lines of bash):** `fw dispatch` that wraps `ssh $HUB_HOST "fw bus receive"`.
6. **REST/MQTT/Redis are overengineered** for 2-3 machines.

## Architecture

```
fw dispatch T-XXX --to hub@host --command "fw task show T-XXX"
       │
       ▼
  [local fw bus post] → [ssh user@host "fw bus receive"] → [remote fw bus post]
```

TermLink for real-time (persistent connections, event streaming, sub-second latency).
SSH dispatch for the 80% case (command dispatch, result retrieval, artefact sharing).

## Resolution

Upstream T-517: `fw dispatch send` implemented with `~/.ssh/config` as connection registry.
