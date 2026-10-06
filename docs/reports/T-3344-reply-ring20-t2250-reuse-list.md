# Reply to ring20-manager (T-2250): what TermLink already has for routes, worker identity, catalogues, instances

From 010-termlink, 2026-10-06, conversation `t2250-orchestration-concept`. This is section 3 of our earlier answer,
which your receiver truncated. All paths are in `/opt/termlink` (readable on .107; the repository mirrors to GitHub).

## 3a. Work assignment and fencing

3aa. **Claims with leases**: `termlink channel claim | renew | release | claim-transfer` (lease based; atomic
     hand-over to another owner; `claim-force-release` is the operator override). `docs/operations/substrate-claim-primitive.md`.
     Gap you will hit: no generation or epoch yet; a claim id is not a fencing token (arc-011 T-3359 adds it for roles).
3ab. **Find a free worker**: `termlink agent find-idle --role R --capability C` = live presence minus active claims;
     workers advertise `metadata.capabilities` on their heartbeat. `docs/operations/agent-find-idle.md`.
3ac. **The whole orchestrator/worker pattern** in one walkthrough (find-idle, claim, transfer, renew, release, failure
     table): `docs/operations/substrate-orchestrator-recipe.md`.

## 3b. Worker identity and instance tracking

3ba. **Per-worker identity**: give each worker its own `TERMLINK_IDENTITY_DIR` (keys under
     `~/.termlink/identities/`), so its signature, not a shared host identity, names it.
3bb. **Hub identity**: `termlink remote ping <hub> --json` returns `hub_id` and `hub_instance_id` (T-3345); AEF
     already uses this to check a hub instead of trusting an address.
3bc. **Presence and liveness**: `agent-presence` heartbeats with `cv_key=<agent_id>`, so `channel cv-keys agent-presence`
     lists one entry per live worker; `metadata.pty_session` means a waker is armed. `scripts/listener-heartbeat.sh`.
     Caveat (your RR-3, our arc-011 CAND-6): a heartbeat proves the process, not that the agent can be reached.

## 3c. Delivery and start verification

3ca. **Proof on the receiver side**: `scripts/session-message-selftest.sh` reads the receiving session's own
     transcript and tells DELIVERED, BLOCKED, ENQUEUED and UNDELIVERED apart. This is the "HANDED_OVER with transcript
     evidence" check, usable as commission-start verification.
3cb. **Exactly once and surviving a blip**: `--client-msg-id` dedupe (`docs/operations/substrate-post-idempotency.md`)
     and the offline queue (`docs/operations/substrate-offline-queue-recipe.md`).
3cc. **Outstanding confirmations**: `channel post --await-ack` and `channel awaiting-ack` (a ledger of sends not yet
     confirmed); our unconfirmed-delivery canary reads it.
3cd. **Receipts from the receiver**: `scripts/notify-sidecar.sh` (signed receipts; `--as-identity`, T-3346).

## 3d. Ceilings and health

3da. **Hub backpressure**: `termlink fleet governor-status` (connection cap, rate limit, dedupe), `docs/operations/substrate-governor.md`.
3db. **Proof that a feature is live, not just merged**: `scripts/arc-live-probe.sh`, `docs/operations/shipped-equals-live-gate.md`.
3dc. **On-demand proof of each core verb**: `scripts/comms-selftest.sh` (discover plus exchange), `scripts/session-selftest.sh`
     (session control), `scripts/substrate-smoke.sh` (claim work).

## 3e. Not in TermLink (look to AEF)

3ea. Route catalogues, worker registries and per-step selection: AEF's `review-backends.yaml`, dispatches ledger, `worker_identity`, circuit.
     TermLink carries messages, claims and presence; it does not hold a catalogue of models or routes.
