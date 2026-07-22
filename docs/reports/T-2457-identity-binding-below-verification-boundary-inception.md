# T-2457 — Identity-binding gap below the verification boundary (round-13 systemic inception)

**Status:** inception, GO recommended (decision = human's).
**Date:** 2026-07-22.
**Origin:** round-13 ultra-critical review of TermLink's coordination/governance substrate.
**Generalizes:** T-2454 (claim verbs — first-discovered instance).

## Summary

TermLink binds *sender identity* cryptographically for one operation:
`channel.post` rejects `sender_id != fingerprint_of(sender_pubkey_hex)`
(`CHANNEL_IDENTITY_MISMATCH`, T-1427). Round-13 adversarial review found that
**three shipped coordination/governance guards do not sit behind that boundary** —
they compare against *unverified request params* instead. One root cause, three
instances:

| # | Primitive | Guard keyed on | Verified? | Severity | Task |
|---|---|---|---|---|---|
| A | claim verbs (`release`/`renew`/`transfer`) | `claimer`/`by` param | no | HIGH (mesh) | T-2454 |
| B | cv_index current-value (`agent-presence`) | `metadata.cv_key` param | no | **HIGH** | this (new) |
| C | governor rate-limiter buckets | `params.from` / `params.sender_id` | no | **MED** | this (new) |

The unifying question: **where should the identity-verification boundary sit for
governance/coordination state?** Today the T-1427 check lives *inside* the
`channel.post` handler; these three guards run either in sibling handlers (claim),
in the post handler but on the un-signed metadata map (cv_index), or at the
transport layer *before* the post handler even parses (rate-limiter). Answer the
boundary question once and it parameterizes all three fixes.

## Instance B — cv_index current-value spoofing (HIGH)

`metadata` is explicitly excluded from the canonical signed bytes
(`channel.rs:663-669`, comment at `:696-698`: *"optional metadata routing-hint
map. NOT included in canonical signed bytes"*). The cv_index write
(`channel.rs:793-797`) records `(topic, cv_key) -> offset` with **no
`cv_key == sender_id` check**:

```rust
if let Some(cv_key) = env.metadata.get("cv_key") {
    let cv_key = cv_key.trim();
    if !cv_key.is_empty() && cv_key.len() <= 256 {
        let _ = crate::cv_index::record(&topic, cv_key, offset);
    }
}
```

`cv_index::record` is monotonic-max on offset and one-offset-per-key
(`cv_index.rs:106,215`). **Exploit:** validly-signed producer A posts to
`agent-presence` with `metadata.cv_key = "<B's fingerprint>"`. A's fresh (higher)
offset wins the monotonic-max, so `(agent-presence, "B") -> A's offset`. A
`subscribe --include-current-value` / find-idle discovery for key "B" resolves to
A's envelope — **B's real presence entry is evicted/impersonated**, unauthenticated
w.r.t. the impersonated key. The underlying heartbeat *log* is intact (fallback
walk still finds B), so this is a discovery-integrity / presence-spoof hole, not
data loss — but discovery is exactly what find-idle (substrate #2) and the
push-wake doorbell depend on.

**Why the naive fix is wrong.** `cv_key` is a *general* current-value key for
broadcast-with-replay (#9) — a "room-state" key (a document id, a task id), not
always an identity. Forcing `cv_key == sender_id` universally would break every
non-identity cv_key use case. The fix needs a per-topic (or per-key-namespace)
*policy* distinguishing identity-keyed topics (`agent-presence`) from
arbitrary-key topics — a design decision, not a one-line guard.

## Instance C — rate-limiter bucket keying (MED)

`governor::derive_sender_key` (`governor.rs:98-109`) keys per-sender buckets by
`from` → `sender_id` → `peer_addr` → `peer_pid` → `"anonymous"`, and the charge
happens at the transport layer (`server.rs:1149`) **before** `channel.post`'s
signature check. So the bucket key is an *unverified* param.

- **Exploit A (evasion):** rotate `params.from` per request → each mints a fresh
  full bucket (`governor.rs:266`) → the rate limit never trips. Trivially defeats
  the entire limiter.
- **Exploit B (targeted DoS):** set `from = victim_fingerprint`, flood → drain the
  victim's shared bucket → the victim's own traffic gets `RATE_LIMITED` (-32008).

**Why the naive fix is wrong.** The obvious fix — key on the non-spoofable
connection identity (`peer_addr`/`peer_pid`) — **regresses PL-218/PL-209**: the
`from`-first precedence was a *deliberate* T-2432 fix because per-pid keying minted
a fresh bucket per one-shot CLI invocation and bloated to ~380K live buckets
fleet-wide. And the verified fingerprint is *not available* at the charge layer
(verification happens later, inside the post handler). The real fix is a *charge
placement* decision: charge a cheap connection-scoped cap pre-verify AND an
identity-scoped limit post-verify (dual-bucket), or move the charge to after
verification. Both are pipeline changes.

## The shared design question (what the human decides)

Where does the identity-verification boundary belong for governance/coordination
state, given the tension that:

1. The only cryptographically-verified identity today is produced *inside* the
   `channel.post` handler (T-1427), so any guard that must run *before* posting
   (rate-limit) or *outside* posting (claim verbs) has no verified identity to key
   on without new plumbing.
2. Some coordination keys are legitimately *not* identities (arbitrary cv_keys),
   so a blanket "key == fingerprint" rule is wrong.
3. Two of the three naive fixes regress a real prior fix (PL-218) or a real
   feature (broadcast-replay #9).

This is the same trusted-mesh-vs-adversarial threat-model call that gates T-2454
(its IW-2, human-owned). Under a mutually-trusting orchestrator+workers mesh, all
three are accountability/accidental-double-work hardening (MED). Under an
adversarial mesh they are active spoof/evade/DoS (HIGH). The *fix designs* are the
same either way; the *priority* is the human's arc-011 scoping call.

## Proposed direction on GO (design, not yet build)

1. **Decide the boundary model** (the inception's output): (a) a reusable
   `verify_sender_identity(params, conn) -> VerifiedId` primitive callable from any
   handler, hoisting the T-1427 check out of `channel.post`; plus (b) a per-topic
   "identity-keyed" policy flag for cv_index; plus (c) a dual-bucket (connection cap
   + verified-identity limit) charge model for the governor.
2. **Then decompose per instance** (one-bug-one-task builds), each reusing the
   boundary primitive: B (cv_index identity-topic policy), C (governor dual-bucket),
   and A (T-2454, already filed) flip to the shared primitive.

## Verified CLEAN this round (do not re-review)

Q3 — receipt frontier / await-ack: identity-bound (`ack_retry.rs:107`,
`channel.rs:684`), bounded retries, no cross-party false-ack. The one LOW
non-monotonicity quirk (receding self-frontier) was **fixed this round** in
T-2456 (aggregate by max `up_to`, commit a9064ff8, 434 hub tests green).

## Related

- T-2454 — claim-verb instance (A), first-discovered, already filed.
- T-2456 — round-13 build: receipt frontier monotonicity (the Q3 CLEAN-but-LOW fix).
- T-1427 — the `sender_id == fingerprint` binding this generalizes.
- T-2432 / PL-218 / PL-209 — the rate-limiter bucket-bloat fix that blocks C's naive fix.
- arc-011 — the parallel-orchestrator consumer whose threat model sets severity.
