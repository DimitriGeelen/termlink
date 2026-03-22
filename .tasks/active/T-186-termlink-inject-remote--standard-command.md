---
id: T-186
name: "termlink inject-remote — standard command for cross-machine prompt injection"
description: >
  Design a standard CLI command for repeatable cross-machine prompt injection. Currently requires: hex secret parsing, HMAC token generation, TOFU TLS, hub routing — all manual via tofu_test example. Need: 'termlink inject-remote host:port session-name "message" --secret-file /path'. Should handle auth, TOFU, and split-writes automatically. Also consider 'termlink connect-hub' for persistent hub connections.
status: started-work
workflow_type: inception
owner: human
horizon: now
tags: [cli, cross-machine, ux]
components: []
related_tasks: [T-182, T-183, T-184, T-185]
created: 2026-03-18T23:28:42Z
last_update: 2026-03-18T23:47:58Z
date_finished: null
---

# T-186: termlink inject-remote — standard command for cross-machine prompt injection

## Problem Statement

Cross-machine prompt injection via TermLink currently requires 6 manual steps: hex secret parsing, HMAC token generation, TOFU TLS connection, hub authentication, hub-routed inject with target param, and split-write delay handling. This was proven in T-183/T-184 (5 prompts, 7.4KB injected into remote framework agent) but relies on the `tofu_test.rs` example — not a reusable CLI command.

**For whom:** Any TermLink user who needs to inject text/commands into sessions on remote machines (primary use case: sending improvement prompts to framework agents on other machines).

**Why now:** The primitives are all proven and complete (TOFU: T-182, auth: T-164, split-write: T-178, hub routing: T-163). Only the CLI surface is missing.

## Assumptions

- A-001: All required primitives exist in termlink-session crate (VALIDATED — proven in T-183/T-184)
- A-002: A `remote` subcommand namespace won't conflict with existing CLI (VALIDATED — no `remote` command exists)
- A-003: Secret-file approach is sufficient for auth (no key exchange protocol needed for MVP)
- A-004: ~150 lines of CLI code wrapping existing library functions

## Exploration Plan

1. ~~Analyze current CLI structure and inject command~~ — DONE (see research artifact)
2. ~~Evaluate 4 design variants (A-D)~~ — DONE (Variant D: `remote` subcommand family recommended)
3. Present to human for GO/NO-GO decision
4. If GO: create build task(s) for implementation

See full research: `docs/reports/T-186-inject-remote-cli-design.md`

## Technical Constraints

- macOS + Linux (both TOFU TLS paths work)
- No new crate dependencies needed
- TOFU known_hubs at `~/.termlink/known_hubs`
- Hub must be running with `--tcp` on remote machine
- Secret shared out-of-band (plain hex file)
- `command.inject` requires `PermissionScope::Control` (not Execute)

## Scope Fence

**IN scope:**
- `termlink remote inject` CLI command design
- Secret file reading + hex parsing
- TOFU+auth+inject chain in one command
- Error messages for each failure mode

**OUT of scope:**
- `termlink remote list/status/trust/run` (future extensions)
- Secret distribution protocol
- Hub auto-discovery (mDNS)
- Persistent connections / connection pooling
- `termlink connect-hub` (deferred to separate task)

## Acceptance Criteria

- [x] Problem statement validated
- [x] Assumptions tested (A-001 through A-004 all validated)
- [x] Go/No-Go decision made (GO — all primitives proven, Variant D chosen)

## Go/No-Go Criteria

**GO if:**
- All building blocks proven (TOFU, auth, hub routing, split-write) — YES
- Design variant chosen with clear UX — YES (Variant D: `remote` subcommand)
- Implementation estimate is bounded (~150 lines, no new deps) — YES

**NO-GO if:**
- Missing primitives require new protocol work — NOT THE CASE
- CLI namespace conflicts prevent clean design — NOT THE CASE

## Verification

# Research artifact exists
test -f docs/reports/T-186-inject-remote-cli-design.md

## Decisions

**Decision**: GO

**Rationale**: All primitives proven (TOFU, auth, hub routing, split-write). Variant D (remote subcommand family) chosen. ~150 lines, no new deps.

**Date**: 2026-03-19T05:50:30Z
## Decision

**Decision**: GO

**Rationale**: All primitives proven (TOFU, auth, hub routing, split-write). Variant D (remote subcommand family) chosen. ~150 lines, no new deps.

**Date**: 2026-03-19T05:50:30Z

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-03-19T05:50:30Z — inception-decision [inception-workflow]
- **Action:** Recorded inception decision
- **Decision:** GO
- **Rationale:** All primitives proven (TOFU, auth, hub routing, split-write). Variant D (remote subcommand family) chosen. ~150 lines, no new deps.
