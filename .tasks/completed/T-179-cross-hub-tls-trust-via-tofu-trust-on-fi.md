---
id: T-179
name: "Cross-hub TLS trust via TOFU (Trust On First Use)"
description: >
  Hub-to-hub TCP forwarding fails because local hub uses local cert to connect to remote hub. Need TOFU model: accept+store remote cert fingerprint on first connect, verify on subsequent. ~300 lines. Uses rustls custom ServerCertVerifier + known_hubs file. See agent research from T-099 session.

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: [security, tls, hub, cross-machine]
components: []
related_tasks: []
created: 2026-03-18T22:19:49Z
last_update: 2026-03-18T22:58:29Z
date_finished: 2026-03-18T22:58:29Z
---

# T-179: Cross-hub TLS trust via TOFU (Trust On First Use)

## Problem Statement

Hub-to-hub forwarding fails because `Client::connect_addr()` (client.rs:34) uses the LOCAL `hub.cert.pem` as the trust root when connecting to a remote hub via TCP. Each hub generates its own self-signed cert on startup — Hub A's cert can't verify Hub B's, so TLS handshake fails.

This blocks cross-machine agent communication (T-163).

## Research Artifact

`docs/reports/T-179-cross-hub-tls-tofu.md`

## Assumptions

1. rustls `ServerCertVerifier` trait allows custom TOFU verification
2. `~/.termlink/known_hubs` is a suitable persistent storage location (survives runtime dir cleanup)
3. SSH's TOFU model is adequate UX for LAN hub trust (2-node case)
4. No cert rotation needed for v1 (self-signed certs regenerate on hub restart anyway)

## Exploration Plan

1. **Code analysis** (done): Map current TLS flow in client.rs and tls.rs
2. **API research**: Confirm rustls `danger::ServerCertVerifier` interface
3. **Design**: TOFU verifier + known_hubs file format
4. **Estimate**: ~200-250 lines across 3 files

## Technical Constraints

- rustls requires `dangerous()` builder for custom cert verification
- `known_hubs` file must be outside runtime dir (ephemeral)
- TCP connections only — Unix sockets don't use TLS
- SHA-256 fingerprint for cert identification (standard practice)

## Scope Fence

**IN:** TOFU verifier implementation, known_hubs file, CLI commands (fingerprint, trust)
**OUT:** CA-based trust, cert rotation, hub-to-hub discovery protocol

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested
- [x] Go/No-Go decision made

## Go/No-Go Criteria

**GO if:**
- rustls provides ServerCertVerifier extension point
- Implementation is bounded (<300 lines)
- Unblocks cross-machine communication

**NO-GO if:**
- rustls API doesn't support custom verification
- TOFU approach creates unacceptable security risk

## Verification

test -f docs/reports/T-179-cross-hub-tls-tofu.md

## Decisions

**Decision**: GO

**Rationale**: rustls ServerCertVerifier supports TOFU, ~250 lines, unblocks T-163 cross-machine communication

**Date**: 2026-03-18T22:58:19Z
## Decision

**Decision**: GO

**Rationale**: rustls ServerCertVerifier supports TOFU, ~250 lines, unblocks T-163 cross-machine communication

**Date**: 2026-03-18T22:58:19Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-18T22:56:45Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-03-18T22:58:19Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** rustls ServerCertVerifier supports TOFU, ~250 lines, unblocks T-163 cross-machine communication

### 2026-03-18T22:58:29Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
