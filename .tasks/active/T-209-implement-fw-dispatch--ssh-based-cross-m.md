---
id: T-209
name: "Implement fw dispatch — SSH-based cross-machine agent communication"
description: >
  Inception: Implement fw dispatch — SSH-based cross-machine agent communication

status: started-work
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-03-21T15:32:46Z
last_update: 2026-03-21T15:42:58Z
date_finished: null
---

# T-209: Implement fw dispatch — SSH-based cross-machine agent communication

## Problem Statement

`fw termlink dispatch` is the only way to communicate with remote framework agents. Without TermLink installed, there is zero cross-machine agent communication capability. This was experienced directly: reaching the framework agent on 192.168.10.107 required first solving TermLink's own cascading installation failures (T-208).

**Consumer report from macOS ARM64 (2026-03-21):**

### Evidence — The communication gap
```
# Network: 192.168.10.0/24
# Local: 192.168.10.149 (macOS, framework consumer)
# Remote: 192.168.10.107 (framework agent)

# Port scan results on .107:
Port 3000: Email Archive Backend (Express.js)
Port 8080: Email Archive Frontend
Port 8443: (no response)
Port 5000-5041: (no response)

# SSH available but host key unverified:
$ ssh -o BatchMode=yes dimitri@192.168.10.107 echo "SSH OK"
Host key verification failed.

# No fw serve / framework API endpoint discovered on .107
# Only way to communicate: fw termlink dispatch (which requires TermLink installed)
```

After installing TermLink (15 min of troubleshooting), dispatch worked:
```
$ fw termlink dispatch --task T-006 --name agent-107 --prompt "..."
Worker spawned: agent-107
$ fw termlink wait --name agent-107 --timeout 120
Worker agent-107 finished (exit: 0)
$ fw termlink result agent-107
# Full remediation received
```

### Critical Research Finding
- **File-based bus (`fw bus`) is local-only** — cannot reach another machine. Extending it doesn't solve the cross-machine problem.
- **SSH is already available on every machine** where the framework runs. It's already authenticated (keys), encrypted (TLS), and handles NAT/firewalls.
- **Hub profiles already store `user@host` connection info** — the registry for SSH dispatch already exists.
- **`fw bus` envelopes (YAML with metadata) already exist** — the protocol for SSH dispatch already exists.
- **The gap is literally one script (~50 lines of bash):** `fw dispatch` that wraps `ssh $HUB_HOST "fw bus receive"` and pipes the envelope.
- REST/MQTT/Redis are overengineered for 2-3 machines. Each adds infrastructure, service management, and TLS cert provisioning.
- TermLink remains valuable for **real-time** use cases (persistent connections, event streaming, sub-second latency). SSH dispatch handles the **80% case** (command dispatch, result retrieval, artefact sharing).

### Architecture Proposal
```
fw dispatch T-XXX --to hub@192.168.10.107 --command "fw task show T-XXX"
       │
       ▼
  [local fw bus post] ──► [ssh user@host "fw bus receive --envelope stdin"] ──► [remote fw bus post]
```

### User-facing commands
```bash
# Send a command to remote hub
fw dispatch --to myserver --command "fw task status T-042"

# Send a bus envelope to remote hub
fw bus post --task T-042 --agent build --summary "Done" --remote myserver

# Receive (runs on remote, called by SSH)
fw bus receive  # reads envelope from stdin, posts to local bus
```

### Environment
- Local: macOS Darwin 25.3.0, 192.168.10.149
- Remote: 192.168.10.107, SSH available, framework installed
- Network: LAN, no firewall between machines

## Assumptions

- SSH key-based auth can be configured between framework machines
- Remote machines have the framework installed with `fw bus receive` capability
- Hub profiles contain sufficient connection info for SSH dispatch
- ~50 lines of bash is sufficient for MVP implementation

## Exploration Plan

1. Verify hub profile format includes user@host
2. Prototype `fw dispatch` — SSH pipe of bus envelope to remote `fw bus receive`
3. Test: dispatch from .149 to .107 via SSH (after host key setup)
4. Compare latency/reliability with TermLink dispatch
5. Document: SSH dispatch as baseline, TermLink as real-time upgrade

## Technical Constraints

- SSH host key must be accepted before first dispatch (interactive once, then permanent)
- SSH key auth recommended (password auth won't work in non-interactive Claude Code)
- `fw bus receive` must exist on remote (framework installed)
- No streaming — SSH dispatch is request/response, not persistent connection

## Scope Fence

**IN scope:** `fw dispatch` command, `fw bus receive` command, `--remote` flag on `fw bus post`, SSH transport
**OUT of scope:** REST API, MQTT, Redis, WebSocket, TermLink replacement, mDNS discovery

## Acceptance Criteria

- [ ] `fw dispatch --to HOST --command "fw task list"` works over SSH
- [ ] `fw bus post --task T-XXX --remote HOST` delivers envelope to remote bus
- [ ] `fw bus receive` reads envelope from stdin and posts to local bus
- [ ] Hub profiles used as connection registry
- [ ] Failure modes documented (SSH timeout, auth failure, remote fw not installed)
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- SSH dispatch works reliably between two framework machines
- Latency is acceptable for command dispatch use case (<5s round trip on LAN)
- Implementation stays under 100 lines of bash

**NO-GO if:**
- SSH key setup is too complex for typical users
- Bus envelope format is insufficient for cross-machine communication
- Framework's authority model conflicts with remote command execution

## Decisions

**Decision**: GO

**Rationale**: SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.

**Date**: 2026-03-21T15:43:07Z
## Decision

**Decision**: GO

**Rationale**: SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.

**Date**: 2026-03-21T15:43:07Z

## Updates

- 2026-03-21: Consumer unable to reach .107 agent without TermLink — no fallback existed
- 2026-03-21: Critical review: file-based bus is local-only (useless), REST/MQTT overengineered, SSH is the answer
- 2026-03-21: Architecture: fw dispatch → SSH → fw bus receive. ~50 lines of bash, zero new infrastructure

### 2026-03-21T15:42:36Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.

### 2026-03-21T15:42:50Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.

### 2026-03-21T15:42:58Z — status-update [task-update-agent]
- **Change:** owner: human → agent

### 2026-03-21T15:43:02Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.

### 2026-03-21T15:43:07Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** SSH dispatch fills real gap, ~80 lines bash, uses ~/.ssh/config, zero new infrastructure.
