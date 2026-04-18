# T-1071 — Protocol-skew + event.broadcast resilience tier (GO recommendation)

**Date:** 2026-04-18
**Status:** Recommendation GO; awaiting human decision (3 follow-up build tasks proposed).
**Origin task:** `.tasks/active/T-1071-framework-improvements-from-termlink-pro.md`

## Investigation summary

A parallel session (relayed 2026-04-15T21:14Z from ring20-dashboard) reported that termlink client 0.9.844 against .107's newer hub: `command.inject` and `command.exec` failed with parse errors, while `event.broadcast` (via `remote_call`) reached 12/12 sessions cleanly.

This investigation confirms the structural cause and proposes a bounded fix.

## Findings

| Assumption | Status | Evidence |
|---|---|---|
| **A1** — protocol-skew is repeatable | CONFIRMED | `KeyEntry` at `crates/termlink-protocol/src/control.rs:65-74` is `#[serde(tag="type", content="value")]` adjacently-tagged enum; older clients sending bare `String` fail deserialization on hubs built ≥ T-768 (commit 8ea9fa06). |
| **A2** — `event.broadcast` is resilient | CONFIRMED | Payload is opaque JSON relayed by hub without struct-level deserialization. Hub only deserializes the envelope. |
| **A3** — framework could warn | CONFIRMED with caveat | `Capabilities.protocol_version: u8` is declared at registration (`control.rs:79`) — the channel exists. But grep shows zero enforcement code. The only mention of "protocol version mismatch" is a TLS-error string hint at `remote.rs:1676`. |
| **A4** — generic concern | CONFIRMED | Any cross-version typed-RPC system has this failure mode; not termlink-specific. |

## Recommendation

**GO** — split into 3 follow-up build tasks:

1. **[termlink, S]** Wire `protocol_version` enforcement at hub: each registered session declares its version; on RPC with version-incompatible method, return structured error `PROTOCOL_VERSION_TOO_OLD` with minimum required version, instead of opaque serde parse failure. Backwards-compatible: missing field defaults to 1.
2. **[termlink, S]** `fleet doctor`/`fleet status` reports fleet-wide version diversity ("Versions in fleet: 0.9.815 (1 hub), 0.9.99 (1 hub), 0.9.844 (1 hub)"). Cheap — reuses existing `query.capabilities` ping.
3. **[framework, M]** Resilience-tier taxonomy: tag every RPC method as Tier-A (opaque-payload, drift-tolerant: `event.broadcast`, `event.emit`) or Tier-B (typed-struct, drift-fragile: `command.inject`, `command.exec`, `session.update`). Document in `crates/termlink-protocol/src/control.rs` as doc comments. `fleet doctor` flags fleets where Tier-B methods would fail across observed version diversity.

**Load-bearing:** task 1 — converts opaque parse failures into actionable upgrade hints, which is what was missing on 2026-04-15.

## Out of scope (intentional)

- Schema migration tooling (shim that translates old payload to new). Adds complexity for a transient problem; structured error + `fw upgrade` is sufficient.
- Auto-upgrade. Operators must remain in control of fleet versions.

## Decision path

```
fw inception decide T-1071 go --rationale "GO — wire protocol_version enforcement (T-N), fleet version diversity report (T-N), resilience-tier taxonomy (T-N)"
```
