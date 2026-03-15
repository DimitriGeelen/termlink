---
id: T-144
name: "TCP hub listener + cross-machine session discovery"
description: >
  Inception: TCP hub listener + cross-machine session discovery

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [tcp, distributed, hub]
components: []
related_tasks: [T-133, T-011, T-142, T-079]
created: 2026-03-15T21:58:30Z
last_update: 2026-03-15T21:58:30Z
date_finished: null
---

# T-144: TCP hub listener + cross-machine session discovery

## Problem Statement

T-133 shipped TcpTransport (trait, connect, bind, liveness probe) and refactored
client.rs to accept TransportAddr. But the hub only listens on Unix socket, sessions
only register locally, and discovery is filesystem-only. Cross-machine operation
is impossible until:
1. Hub listens on TCP (remote clients can connect)
2. Sessions can register with a TCP address
3. Discovery returns remote sessions

This unlocks Tier 3 from T-142: cross-machine coordination, remote control tower,
distributed agent dispatch.

## Assumptions

- A1: Hub can dual-listen (Unix + TCP) without architectural changes
- A2: Session registration can store TCP addresses in existing JSON format
- A3: Router already forwards via TransportAddr (validated in T-133)
- A4: TcpLivenessProbe (500ms connect timeout) is sufficient for LAN
- A5: Authentication can be deferred to Phase 2 (LAN-only first)

## Exploration Plan

1. [ ] Validate A1: Can tokio hub bind both Unix + TCP listeners concurrently?
2. [ ] Validate A2: Does registration JSON already support TCP addr serialization?
3. [ ] Design: Hub TCP mode (always-on vs opt-in `--tcp-port`)
4. [ ] Design: Remote session registration flow (who writes what, where)
5. [ ] Design: Cross-machine discovery (local FS + remote hub query)
6. [ ] Spike: Hub dual-listen + remote ping across two machines
7. [ ] Go/No-Go decision

## Technical Constraints

- TCP exposes the hub to network — authentication needed eventually (capability tokens from T-079)
- Firewall/NAT traversal out of scope (LAN-first, SSH tunneling for WAN per T-011)
- Port allocation: need a strategy (fixed port? configurable? ephemeral?)
- Existing Unix socket sessions must keep working (backward compat)

## Scope Fence

**IN:**
- Hub TCP listener (dual Unix + TCP)
- CLI `--addr tcp://host:port` for session registration
- Discovery returning both local and TCP-registered sessions
- LAN operation (same network, no NAT)

**OUT:**
- Authentication/encryption (Phase 2, uses T-079 capability tokens)
- WAN/internet operation (use SSH tunnels per T-011)
- Hub federation (multiple hubs discovering each other)
- mDNS/Bonjour auto-discovery

## Acceptance Criteria

- [ ] Problem statement validated
- [ ] Assumptions tested (A1-A5)
- [ ] Design decisions for hub TCP mode, registration flow, discovery
- [ ] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- Hub can dual-listen without major refactoring (A1)
- Existing registration format supports TCP addresses (A2)
- Effort is bounded (≤3 sessions for Phase 1)

**NO-GO if:**
- Hub architecture requires major rewrite for TCP
- Registration/discovery needs a new protocol (not just extending existing)
- Auth must be solved first (can't defer to Phase 2)

## Verification

test -f docs/reports/T-144-tcp-hub-inception.md

## Decisions

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->
