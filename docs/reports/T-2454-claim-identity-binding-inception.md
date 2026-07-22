# T-2454 — Claim verbs lack caller-identity binding (round-12 inception)

**Status:** inception, GO recommended (decision = human's).
**Date:** 2026-07-22.
**Origin:** round-12 ultra-critical review of termlink's coordination substrate.

## Summary

TermLink's coordination substrate (claim primitive #1) promises **exclusive
ownership** of a `(topic, offset)` work unit. Adversarial review found the
invariant is enforced only against a **spoofable, world-readable `claimer`
string** — the five claim verbs bind ownership to a plain JSON param, with no
caller-identity check. The comms half of the substrate (`channel.post`) is
cryptographically identity-bound (T-1427); the coordination half is not. A
buggy or misbehaving mesh peer can release or steal another worker's live claim,
producing silent double-work — the exact failure the primitive exists to prevent.

## The invariant and where it breaks

Guarantee: at most one claimer owns `(topic, offset)` at a time; ownership
transitions never silently lose or double-grant.

| Verb | Handler | Ownership check | Identity of caller? |
|---|---|---|---|
| `channel.claim` | channel.rs ~1553 | inserts `claimed_by = claimer` | **none — param string** |
| `channel.renew` | channel.rs ~1821 | `claimed_by == claimer` (meta.rs:623) | **none** |
| `channel.release` | channel.rs:1618 | `claimed_by == claimer` (meta.rs:431) | **none** |
| `channel.claim_transfer` | channel.rs ~1736 | `claimed_by == by` (meta.rs:557) | **none** |
| `channel.claim_force_release` | channel.rs ~1745 | Control-scoped bypass | operator |

Contrast — `channel.post` (channel.rs:684) rejects `sender_id !=
fingerprint_of(sender_pubkey_hex)` (`CHANNEL_IDENTITY_MISMATCH`, T-1427): the
claimed identity must match the key that signed the payload.

## Concrete double-grant scenario (needs only Interact scope)

1. Worker A claims `(topic, 100)` → `claim_id = cX`, `claimed_by = "A"`.
2. Peer B reads `channel.claims` (Observe scope) → sees `{claim_id: cX,
   claimer: "A"}`.
3. B calls `channel.release {claim_id: cX, claimer: "A", ack: false}` — the
   ownership guard `claimed_by == claimer` passes (B supplied "A"); the row is
   deleted, the slot freed. Release is Interact-scoped.
4. A still believes it owns 100. Worker C claims 100 and succeeds. **A and C
   both process offset 100.** The `claim_transfer {by: "A", to_owner: "B"}`
   path silently steals A's lease the same way.

## Threat-model tempering

Under the trusted-mesh model (all token/Unix-scope holders are semi-trusted),
this is primarily an **accountability + accidental-double-work** hardening
rather than an external-attacker breach — which is why the severity sits between
HIGH (invariant break) and MED (mesh-internal). The decision on GO-now vs
DEFER-to-arc-011 hinges on whether the AEF orchestrator+workers are mutually
trusting or adversarial (IW-2, human-owned).

## Proposed fix (design)

Apply the T-1427 signature pattern to claim params:

- Client signs canonical claim bytes (topic, offset/claim_id, claimer, ttl, verb)
  with the session identity key; sends `sender_pubkey_hex` + `signature_hex`.
- Hub verifies the signature and requires `claimer/by == fingerprint_of(pubkey)`
  before the ownership guard runs — reusing `fingerprint_of` + the signature
  verify already in `channel.rs`.
- **Phase-in (IW-3):** hub accepts unsigned claim params during a migration
  window (warn-log), clients start signing, then the hub flips to require-signed.
  Mirrors the signed-post rollout. Avoids breaking live claim callers
  (skills/MCP/orchestrator) on cutover.

This is a **protocol change across hub + session + cli + mcp**, which is why it
is filed as an inception, not a direct build.

## Decomposition on GO (one-bug/one-slice-per-task)

1. Canonical claim-byte format + sign/verify helper in `termlink-session`.
2. Hub-side verify threaded into all five handlers (phase-in accept-unsigned).
3. Client-side signing in CLI/MCP claim commands.
4. Flip to require-signed + regression test (the §"double-grant scenario" now
   returns `CHANNEL_IDENTITY_MISMATCH`).

## Verified CLEAN this round (do not re-review)

Acquire atomicity (single-tx DELETE-expired+INSERT under meta mutex + UNIQUE
index, no TOCTOU); renew no-resurrect; TTL clamped to 1h; release monotonic
cursor advance; transfer atomic single-UPDATE lease-preserving; force-release +
transfer gated at Control scope. The SQL state machine is sound — the ONLY hole
is the missing caller-identity binding.

## Sibling finding (separate task, not in scope here)

MED-2 — lease expiry uses wall-clock (`meta.rs:826-831 now_unix_ms =
SystemTime::now`); an NTP forward step can expire a live lease early → transient
double-grant. Smaller, orthogonal fix; file as its own task if prioritized.
