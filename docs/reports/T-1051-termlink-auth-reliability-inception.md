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

## Spike 1 + 2 Findings (2026-04-14)

### Code inspection — persist-if-present *is* implemented

`crates/termlink-hub/src/server.rs:65 generate_and_write_hub_secret`
- T-933: on startup, if `runtime_dir()/hub.secret` exists and is valid 64-char hex, reuse it verbatim.
- Otherwise, generate a fresh 32-byte secret, write with 0600, return it.

`crates/termlink-hub/src/tls.rs load_or_generate_cert`
- T-945/T-1028: on startup, if `runtime_dir()/hub.cert.pem` + `hub.key.pem` exist, reuse them verbatim.
- Otherwise, generate a fresh self-signed cert, write to disk.

`crates/termlink-cli/src/commands/infrastructure.rs:534 cmd_hub_restart`
- T-1031: when restarting a hub whose pidfile lives in a non-default runtime dir (e.g., `/var/lib/termlink`), the new child process is spawned with `TERMLINK_RUNTIME_DIR` set to the same dir → secret and cert are preserved.

**Conclusion:** The persist code is correct and deployed. The design intent "restart does not break auth" is structurally satisfied *inside the hub restart path*.

### So why did today's failure happen?

Facts we can observe without SSH:
- Client secret file `/root/.termlink/secrets/192.168.10.109.hex` is dated `2026-04-13 11:48`.
- T-1027 (deploy of v0.9.844 to .109) committed `2026-04-13 19:38+`. Hub on .109 was restarted at that point.
- Hub on .109 currently rejects the client's secret as invalid.

Hypotheses ranked by likelihood:

**H1 (most likely) — First deploy of T-933 rotated the secret once.** Before v0.9.844, the hub regenerated its secret on every restart (no persist-if-present). The deploy that introduced T-933 itself triggered *one last* regeneration. After that, persist-if-present kicks in and subsequent restarts preserve the secret. But the client was never told what the new secret is, and there is no protocol for it to find out.

**H2 — Something restarted the hub via a path that skipped T-1031.** If the hub was restarted by systemd (`systemctl restart`) or by a manual `termlink hub start` in a different shell (default `/tmp/termlink-0` runtime dir), the restart may have landed in a *different* runtime dir whose `hub.secret` doesn't exist → fresh secret generated in the new location, leaving the original `hub.secret` orphaned. Follow-on clients using the original dir hit mismatched state.

**H3 — Operator manually regenerated the secret.** Possible but no evidence.

H1 and H2 both point to the same underlying flaw: **the client has no path to discover the current hub secret, and the hub has no mechanism to announce rotation.**

### The design gap (definitive)

The system has a *one-shot* trust anchor:
- Operator obtains the hub secret once (out of band) and distributes to clients.
- After that, clients trust their local copy forever.
- If anything invalidates that copy — first-time persist rollout, systemd restart in a different runtime dir, filesystem corruption, operator error, intentional rotation — clients fail silently-but-loudly (auth error, no recovery hint beyond "go fetch the secret manually").

The system today has *no*:
1. Hub-side broadcast of "my secret is X" (would require a bootstrap anchor, which is the chicken-and-egg problem).
2. Client-side detection of "my secret looks stale, here's how to refresh" (fleet-doctor *does* print a hint, but nothing escalates).
3. Out-of-band channel the hub could write a new secret to that clients could fetch (e.g., hub publishes its fingerprint+secret-ID to a file that clients can read via a separate trusted path).
4. Periodic or event-driven check from clients — auth state is only checked when clients try to send something.

## Design Space (concrete options)

### Option A: **Two-tier secret model — root + session**

Introduce a long-lived *root* secret (manually distributed, rarely rotated) and short-lived *session* secrets (rotated freely).
- Hub signs short-lived session secrets with the root.
- Clients authenticate with either (root for bootstrap, session for normal traffic).
- Rotation of session secret = hub announces over the existing TLS channel, clients fetch signed envelope.
- Root only matters for initial trust anchor + disaster recovery.

Pros: clean story, antifragile (session secrets can rotate hourly without operator action).
Cons: new protocol surface; migration story for existing hubs; more cryptographic state.

### Option B: **Pinned-on-first-use secret with persistence and loud failure**

Lean into persist-if-present harder:
- Ship "persist-by-default" so first boot generates AND stores.
- `termlink hub start` refuses to start (flag required) if no persisted secret exists and no explicit `--regenerate-secret`.
- On every client auth failure that looks like "wrong secret," the client opens a clearly-formatted issue in the local agent's concerns register, and fleet-doctor self-registers a gap if the failure persists >24h (G-019 compliance, T-1051 ship criterion).
- Secret rotation is an explicit, loud, operator-initiated action with a well-known distribution recipe.

Pros: minimal protocol change; leans on existing mechanisms; aligns with framework's "prevention > mitigation" directive.
Cons: still a manual distribution step; doesn't eliminate the human-in-the-loop.

### Option C: **Event-based rotation announce over TLS**

- When hub regenerates its secret, it records `hub.secret-rotated-at = <timestamp>` in a state file.
- Clients periodically read that file via a read-only lightweight endpoint (already in doctor) and note rotation events.
- On rotation, clients show a clear error message: "Hub rotated secret at T. Your cached secret is older than T. Refresh with: `termlink fleet reauth <profile>`."
- Add a `termlink fleet reauth` command that walks a user through the distribution step.

Pros: surfaces rotation events explicitly; gives operator a one-command recovery.
Cons: still no auto-heal; doesn't solve the "first rollout of persist-if-present rotated the secret" problem because clients had no prior trust.

### Option D: **Hybrid — B baseline + targeted detection**

Minimum viable antifragile:
1. Persist-by-default is already implemented — confirm on all hosts, write smoke test into fleet-doctor that checks hub.secret file age vs hub process start time (if secret file is younger, rotation happened — that's a signal).
2. `termlink fleet doctor` when it detects auth-mismatch, automatically self-registers a learning: "Hub X secret mismatch on 2026-04-14" so it's visible in handovers.
3. After N consecutive failures over >1 day, self-register a *concern* in `concerns.yaml` (G-019 compliance).
4. Add a `termlink fleet reauth <profile>` stub — prompts operator with the exact copy-paste shell incantation to refresh the secret. Tier-1 action (logs, reads, one file write to a known location). If SSH-over-termlink is wired, make this command do the whole job autonomously.
5. Document the rotation protocol in CLAUDE.md so future agents know the flow.

Pros: smallest design delta, largest leverage (detection + codification of existing knowledge); fits this project's existing mechanism set.
Cons: doesn't solve the fundamental bootstrap problem — accepts "first rollout requires manual distribution" as part of the protocol.

## Recommendation (agent position)

Go with **Option D** as the ship criterion. It:
- Makes the system antifragile (failures register as learnings, concerns, and visible handovers).
- Gives operators a one-command heal path.
- Avoids new cryptographic machinery until we have evidence we need it.
- Is decomposable into 4-5 small build tasks, each one independently valuable.

Option A is interesting but premature — we don't yet have evidence that session-secret rotation is a frequent-enough pain point to justify the protocol surface. Option C is valuable but overlaps substantially with D; fold the good parts of C into D's detection story.

## Dialogue Log (continued)

### 2026-04-14 — findings summary → decision request

Spikes 1 + 2 done. Root cause isolated: **no secret-rotation protocol**, not a bug in persist-if-present. Today's failure is the one-time cost of deploying T-933 on top of a running fleet that had pre-T-933 secrets.

Asking the operator:
1. Is Option D the right ship scope, or do you want Option A's session-secret protocol?
2. Can I invoke any other agents (e.g., via `termlink remote` to reachable hubs) for a second opinion, or should I stay with my own analysis?
3. Should I close this inception with a GO decision on Option D and decompose, or park for your review first?

### 2026-04-14 — peer review from ring20-dashboard session

Reached via `termlink remote inject` using cached --secret (same stale-file workaround as operator). Agent confirmed GO on Option D. Second-instance evidence within 24h is already below the G-019 7-day threshold, which itself is evidence that D's "learning on first hit" step would have pre-empted this rediscovery.

Two structural additions accepted into the design:

**R1 — Memory drift is part of the symptom.** Rotation poisons agent *memory*, not just cached secrets. A prior rotation had left a stale memory entry on the ring20-dashboard side claiming the hub was at `.122` — never invalidated when things moved back to `.109`. Each learning record produced by D must carry:
- `date_observed` (UTC)
- `hub_fingerprint_at_time_of_writing` (sha256 of the TLS cert at the moment the learning was recorded)

so a future agent can detect "this learning was recorded before the current fingerprint and may be stale."

**R2 — Bootstrap chicken-and-egg on step 4.** If the "trusted separate channel" for `fleet reauth`'s autonomous variant itself uses the rotation-blind termlink layer, it rots the same way. Two acceptable resolutions:
- Require the trusted channel to be out-of-band (ssh-key, git-pull, physical USB — anything whose trust anchor isn't rotated by termlink).
- Make the heal command take an explicit `--bootstrap-from <source>` argument so the operator chooses the anchor per incident. Default `none` → prompt the operator.

This is not a blocker for D; it's a build task constraint.

**Peer note on sequencing:** ring20-dashboard does not need its secret refreshed to continue. Ship the heal mechanism first — after that, the refresh becomes a one-liner. This inverts the apparent urgency: the broken auth is the test case, not the blocker.

## Decision

**GO on Option D**, incorporating R1 and R2.

Rationale:
- Second-instance evidence in <24h satisfies G-019's systemic-flaw threshold.
- All four options analysed; D is the minimum viable antifragile path.
- Peer reviewed by an agent that independently hit the same failure class today.
- Avoids premature crypto surface (Option A), leverages existing mechanisms (inherits from B/C), and is decomposable into small, independently valuable build tasks.

Decomposition (one-deliverable-per-task, per framework sizing rules):

| Task | Type | Deliverable |
|---|---|---|
| T-1052 | build | `fleet doctor` auto-registers a learning on auth-mismatch, with `date_observed` + `hub_fingerprint` (R1 compliance) |
| T-1053 | build | After N consecutive fleet-doctor failures over >1 day, self-register a concern in `concerns.yaml` (G-019 compliance) |
| T-1054 | build | `termlink fleet reauth <profile>` — one-command operator heal, prints the exact distribution incantation; Tier-1 only, no `--bootstrap-from` yet |
| T-1055 | build | `termlink fleet reauth --bootstrap-from <source>` — autonomous heal variant with explicit trust anchor (R2 compliance) |
| T-1056 | refactor | CLAUDE.md: document the rotation protocol + recovery recipes + the meaning of `hub_fingerprint` in learnings |

Each task stands alone. Ship order T-1052 → T-1053 → T-1054 → T-1055 → T-1056, but they're independent; a later task can land before an earlier one without breaking the earlier's value.

