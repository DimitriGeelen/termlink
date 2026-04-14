# T-1051: TermLink Auth/Connect Reliability — Inception

**Status:** In progress
**Type:** Inception
**Owner:** agent (with human dialogue)
**Created:** 2026-04-14

## Problem Statement

Cross-host TermLink hub connections keep breaking. The recurring failure shape:

1. Hub restarts → regenerates TLS cert → client's pinned TOFU fingerprint no longer matches → `TOFU cache stale` error.
2. Hub rotates/regenerates its HMAC secret → client's cached secret file becomes invalid → `Authentication failed: -32010 Token validation failed: invalid signature`.
3. Client has no path to self-heal:
   - SSH is not available (`Permission denied (publickey,password)`).
   - `/root/.termlink/` is outside project boundary on the agent's side (blocked by T-559 boundary enforcement).
   - There is no in-band mechanism for clients to rotate/refresh secrets.

Symptoms observed today (2026-04-14):
- Agent on .100 (this session) trying to send to hub on .109 — auth invalid, secret stale since 2026-04-13.
- Separate agent on .121 (ring20-dashboard) hitting exactly the same failure, same hub.
- Workaround in memory: PL-006 "TOFU cache stale → grep -v … > known_hubs". Workaround for secret: "user has to update them manually."

**The framework was blind to this for weeks** (per G-019: framework should register a gap whenever a class of error persists undetected >7 days).

## Goal

Devise a design that makes TermLink hub authentication:
- **Antifragile** — recurring failure classes surface as self-registered gaps and drive codified healing.
- **Reliable** — a restart or key rotation doesn't silently break clients.
- **Self-healing** — where safe, the system can renegotiate trust without human intervention; where not safe, it fails loudly with a single explicit human action to recover.

## Scope boundary

In scope:
- Secret rotation / refresh model (how clients obtain a new HMAC secret after rotation).
- TOFU cert model (how clients re-pin after hub cert regeneration).
- Operator UX for the unavoidable human-intervention paths.
- Detection / alerting so stale credentials don't go unnoticed for days.
- Fleet-doctor / doctor enhancements that surface this state machine clearly.

Out of scope:
- Completely rewriting the TLS story (we keep TOFU + HMAC as the primitives).
- Key management services (Vault, etc.) — over-engineering for the scale here.

## Assumptions (to validate)

A-001: Hub TLS cert regeneration on every restart is the dominant TOFU failure cause. (Partially addressed by T-945/T-1028 which switched to persist-if-present. Need to verify whether it's fully landed on the actual hosts causing today's failure.)

A-002: Hub HMAC secret regeneration on every restart is the dominant auth failure cause. (T-933 added persist-if-present for secrets. Same question — is it deployed to the problematic hosts?)

A-003: Clients cannot self-heal because there is no bootstrap protocol — a client needs the secret to authenticate, but needs to authenticate to obtain a new secret. Chicken-and-egg.

A-004: The stale-secret failure lasting multiple days is a *detection* failure more than a *recovery* failure. Fleet-doctor already flags it, but nothing wakes the operator up.

A-005: Most hub restarts are operator-initiated (upgrade, config change), not crashes. Which means the operator has a window to publish a new secret/cert ahead of the restart.

## Research Plan

Spike 1 — Map the current state (CODE AS-IS)
- Read hub startup path: secret loading, cert loading, persist semantics (T-933, T-945, T-1028).
- Read client auth path: secret file resolution, TOFU pinning, known_hubs format.
- Identify *every* place where "can't auth" becomes a user error and grade recoverability.

Spike 2 — Verify which hosts are running the persist-if-present code
- .107, .109, .121 binary versions
- Did T-945/T-1028 actually deploy? The secret file on .109 is dated 2026-04-13 11:48 and no longer matches the hub — so *the hub rotated after that*. Means either the hub wasn't persisting (old code) or the hub's secret file was deleted.

Spike 3 — Existing healing mechanisms
- `termlink tofu clear` exists. What about `termlink fleet reauth`?
- Is there any hub→client announce mechanism already? (Events? Hub-level subscriptions?)

Spike 4 — Design space for the renegotiation protocol
- Option A: **Pre-shared bootstrap token** — hub exposes a "rotate secret" RPC that requires a different (long-lived) operator token.
- Option B: **Operator "push" from hub host** — hub writes new secret + cert fingerprint to a well-known URL/path; clients fetch via out-of-band mechanism.
- Option C: **Persist-by-default with loud failure** — lean into persist-if-present, and when it fails, fail very loudly (page, scream, block) so secrets only rotate when intended.
- Option D: **Hybrid** — C as baseline + A as break-glass.

Spike 5 — Detection & self-registration of gaps
- When fleet-doctor reports failing for N days, should it self-register a concern?
- Should `termlink` agent processes emit a structural learning when PL-006 fires?

## Dialogue Log

### 2026-04-14 — initial framing

**User question (paraphrased from cross-session transcript):** We keep having connection issues. Investigate, ask around with different agents, look back at our own logs & tasks, and devise a design/structure/approach to make our mechanism more antifragile, more reliable, more self-healing.

**Observations from the transcript and local context:**

1. Same failure hit two independent agents today on the same hub (.109) — not a one-off.
2. Existing workarounds (PL-006, secret-refresh instructions) are *recipes*, not *mechanisms*. Every recurrence costs operator time.
3. Prior work (T-945 cert persist, T-1028 TLS cert persist, T-933 hub secret persist) aimed to fix this, but the failure keeps happening → either the fix didn't deploy, or it didn't cover the full failure surface.

**Agent position for this dialogue:** Start with Spike 2 — find out whether today's failure is "persist code didn't deploy" (operational) or "persist code has a gap" (design). That answer steers the whole inception.

**Awaiting:** Operator go-ahead on the investigation plan or course correction.
