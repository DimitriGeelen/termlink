# DRAFT: TermLink round-2 (final) review of ring20 T-2250 orchestration concept v0.3

Status: draft, not sent. Task: T-3344. Source read 2026-10-06 from
`http://192.168.10.122:3000/project/docs--reports--T-2250-orchestration-concept` (sections 4R, 5R, 5E, 5F, 5G; 5B/5C for context).

## Context

ring20 asks stakeholders to answer round 2 only where they disagree or see a gap (5R), with TermLink's area being
5R.5 "circuit-id attribution vs per-agent authentication, ack and liveness". TermLink agrees with the shape and with
"supervisor-owned results channel" (5E.1.2). Below are only gaps and corrections. Every TermLink claim was checked
against code or docs in `/opt/termlink`; anything not verified here is marked UNVERIFIED. Grounding:
`docs/reports/T-3344-reply-ring20-t2250-reuse-list.md` (our reuse list), `docs/design/interactive-agent-communication-01-requirements.md`
and `docs/design/interactive-agent-communication/interview-step-01.md` (arc-011 operator rulings), `docs/CHARTER.md`.

## Findings

### 1. BLOCKER: S-1 isolation and "workers talk TermLink" contradict each other (4R.10 S-1, 5E.1.1, 5E.1.2)

1a. Problem. 5E.1.1 runs S-1 workers as an unprivileged Unix user with no read access to /root or credential files.
    4R.10 S-1 says "workers addressed by circuit id" and "ack carries the commission id". On .107 the hub socket and
    secret are not reachable by such a user: `ls -la /var/lib/termlink` shows the directory `drwx------ root` holding
    `hub.sock`, `hub.secret` (0600) and the TLS key; per-agent keys under `~/.termlink/identities/*.key` are root 0600.
    An isolated worker therefore cannot post, ack, claim or heartbeat at all. Giving it the hub secret would hand it a
    fleet-wide credential (charter non-goal 5: TermLink is not a security boundary between distrusting tenants,
    `docs/CHARTER.md`).
1b. Change wanted. State in the S-1 contract (5E.1.11) that in S-1 the worker has NO TermLink access. The supervisor
    (root) is the only TermLink client: it holds the claim, posts the ack and hand-back on the worker's behalf, and
    derives liveness from the launched PID. "Addressed by circuit id" means attribution in the ledger, not a TermLink
    endpoint. Any later slice that gives a worker its own TermLink identity needs its own uid, its own key file, and
    a decision on hub-secret custody (arc-011 gap GP-1).
1c. Evidence: `/var/lib/termlink` permissions (command above); `docs/CHARTER.md` non-goal 5;
    `docs/design/interactive-agent-communication-01-requirements.md` QS-15 / GP-1.

### 2. SHOULD: the identity row in 4R.1 names the wrong owner and is partly stale (4R.1, 5C.4)

2a. Problem. 4R.1 lists "5-level circuit ids (host/hub/project/session/agent)" under owner TermLink. TermLink has no
    circuit-id concept: `grep -rn circuit crates --include=*.rs` finds only the hub's circuit breaker. 5C.4 itself
    cites `lib/sidecar/circuit.py`, which is AEF's. Separately, "per-agent authentication still open" is half true:
    per-agent signing keys exist and the hub verifies them on every post (`crates/termlink-hub/src/channel.rs:876-885`,
    T-1427: `sender_id` must match the signing key; resolver order `TERMLINK_IDENTITY_FILE > TERMLINK_AGENT_ID >
    TERMLINK_IDENTITY_DIR > host default`, `crates/termlink-cli/src/commands/identity.rs:98-105`). What is genuinely
    open: (i) all keys sit under one uid, so any process of that uid can sign as any agent; (ii) see finding 3.
    The "T-3405" reference in 5C.4 is not a TermLink task id (UNVERIFIED which project owns it).
2b. Change wanted. Split the row: circuit ids = AEF (`lib/sidecar/circuit.py`); TermLink = signed sender identity,
    presence, claims, delivery. Reword the caveat to "per-agent keys exist; they authenticate only across a uid
    boundary; claims are not bound to them (finding 3)".
2c. Evidence: paths above; `docs/reports/T-3344-reply-ring20-t2250-reuse-list.md` 3ba.

### 3. SHOULD: TermLink claims are not a fencing token and are not authenticated (4R.3.4, 5E.1.2, 4R.6 "fencing after restart")

3a. Problem. A claim row is `{claim_id, topic, offset, claimer, claimed_at, claimed_until}`
    (`crates/termlink-bus/src/claim.rs:18-25`): no generation or epoch. `claimer` and `to_owner` are free strings
    taken from params (`crates/termlink-hub/src/channel.rs:2013`, `:2251`), not checked against the signing key,
    unlike posts. So "claim transferred atomically to the worker's circuit id" is true for atomicity
    (`claim.rs` T-2046) but gives attribution only, and a claim id cannot reject a stale or forged hand-back.
    The generation arc-011 adds (OD-18, `interview-step-01.md:494-515`) is for the role "main", is task T-3359,
    status `captured`, i.e. not built.
3b. Change wanted. Keep fencing entirely in the supervisor ledger (commission id + attempt, checked at the effect
    boundary), as 5E.1.2 says, and say explicitly that it does not depend on TermLink claims. Do not implement the
    "exclusive ledger lease" with `channel claim`; use a local lock (AEF `lib/keylock.py`, already on the R-list).
    If S-4 wants hub-side fencing, depend on T-3359 and say so.
3c. Evidence: paths above; reuse list 3aa ("no generation or epoch yet").

### 4. SHOULD: "lease expires → lost" has no event behind it (4R.5 lost worker)

4a. Problem. Claims expire lazily: an expired row is reaped only when the same offset is claimed again
    (`claim.rs:7-9`); nothing is emitted at expiry. Maximum TTL is 1 h (`channel.rs:2029`), default 30 s, so any
    commission longer than the TTL must renew. A supervisor that waits for an expiry event waits forever.
4b. Change wanted. The supervisor detects loss itself: PID exit / wall-clock deadline from the ledger, polled on the
    framework polling ladder (15 s, 1 min, 5 min ... each rung twice; arc-011 R-30, requirements 4.2.16). If a claim
    is used at all, the supervisor renews it while the PID lives and reads `claims-summary --only-stuck` as a
    secondary signal.
4c. Evidence: `crates/termlink-bus/src/claim.rs:7-9`, `crates/termlink-hub/src/channel.rs:2028-2044`.

### 5. SHOULD: "ack" must mean hand-over proven on the receiver side, not a send or a receipt (4R.3.1, 4R.10 S-1 "ack carries the commission id")

5a. Problem. arc-011 OD-13 2b (operator ruling): nothing reports "delivered" without a recorded HANDED_OVER; until
    then it says "accepted" or "stored" (`interview-step-01.md` OD-13). A TermLink receipt (`channel ack`, up-to
    offset) or `--await-ack` row proves the message was read by a client, not that the worker started on it. For a
    one-shot `claude -p` worker there is no interactive transcript to check.
5b. Change wanted. Define the S-1 ack as a supervisor-observed start event (process launched under the commission
    id + first valid output), recorded in the ledger. Put the commission id in message metadata as
    `conversation_id` and `commission-id:attempt` as `--client-msg-id` (hub dedupe,
    `docs/operations/substrate-post-idempotency.md`). Reserve the transcript check
    (`scripts/session-message-selftest.sh`) for standing interactive workers (S-4).
5c. Evidence: paths above; reuse list 3ca, 3cb.

### 6. SHOULD (S-4, not S-1): standing-worker liveness is defined as a heartbeat (4R.6)

6a. Problem. "Liveness = a descending agent process plus a progress heartbeat." arc-011 CAND-6 ruled that every
    heartbeat proves a process, not reachability: the claude-termlink-alt sidecar had a seconds-old heartbeat while
    refusing 32,205 confirmations (`interview-step-01.md`, CAND-6, 6a-6c).
6b. Change wanted. For S-4, liveness = last surface time (latest real hand-over or confirmation, never a heartbeat);
    a silent channel shows UNKNOWN, never DEAD; busy is not dead (OD-18). Reuse the OD-18 readiness-gated lease with
    generation (T-3359) for standing roles instead of a new one.
6c. Evidence: `docs/design/interactive-agent-communication/interview-step-01.md` CAND-6 and OD-18.

### 7. SHOULD: a "TermLink transport" route cannot be attested by TermLink (5G.1.2, 5F.3.1)

7a. Problem. "codex via subscription, through a TermLink worker on .122" makes TermLink the first egress hop. TermLink
    can prove which signed identity answered and which hub (`hub_id` / `hub_instance_id`,
    `crates/termlink-hub/src/router.rs:818-832`), not which model or method the remote peer then used. The material
    also lands on the peer's host and hub first.
7b. Change wanted. A route with transport = termlink records the peer's own route id as a declared, unverified
    attribute, and its approved data classes are the intersection of the peer's route and the transport (both hosts).
    Default: estate-confidential and above not approved over termlink transport until the operator rules.
7c. Evidence: `router.rs:818-832`; `docs/CHARTER.md` non-goal 5.

### 8. NICE: ledger and analytics stay off the bus (4R.3.3, S-2, S-7)

8a. If commission transitions are mirrored to a hub topic for visibility, set bounded retention: TermLink topics are
    retention-bounded coordination logs, not a system of record (`docs/CHARTER.md` non-goal 2). The jsonl ledger
    stays the truth.

### 9. NICE: claims and find-idle are per hub (4R.1, 5F.1 fleet orchestrator T-2251)

9a. Claims, presence and `agent find-idle` are local to one hub (G-060, charter non-goal 1). A worker on .122 must
    use the orchestrator's hub, or the fleet orchestrator (T-2251) needs one orchestrator per hub.
9b. For 5E.1.12 forensics, record `hub_id` + `hub_instance_id` (T-3345), not the hub address.

## Ready-to-send message (under 1800 bytes)

```
TermLink round 2 on T-2250 v0.3 (gaps only):
1 BLOCKER S-1 isolation vs TermLink: the unprivileged worker (5E.1.1) cannot reach the hub (/var/lib/termlink is root 0700). Say in the S-1 contract: worker has no TermLink access; supervisor holds claim, posts ack and hand-back, liveness = PID.
2 SHOULD 4R.1: circuit ids are AEF (lib/sidecar/circuit.py), not TermLink. Per-agent signing keys exist and posts are verified (T-1427); open part: keys share one uid, and claims are not bound to them.
3 SHOULD fencing: TermLink claims have no generation/epoch and claimer is an unchecked string. Keep fencing in your ledger (commission id + attempt); use keylock, not channel claim, for the ledger lease. Hub-side generation is T-3359, not built.
4 SHOULD 4R.5: claims expire lazily, no expiry event, TTL max 1 h. Detect loss from PID/deadline, polled on the framework ladder.
5 SHOULD ack: "delivered" only on proven hand-over (arc-011). S-1 ack = supervisor-observed start; commission id as conversation_id, id:attempt as client-msg-id.
6 SHOULD S-4: liveness = last surface time, not heartbeat; reuse OD-18 lease.
7 SHOULD 5G.1.2: TermLink cannot attest a remote peer's model/route; classes = intersection of peer route and transport.
8 NICE keep the ledger off the bus (bounded retention).
9 NICE claims/find-idle are per hub; log hub_id + hub_instance_id.
Details: /opt/termlink/docs/reports/T-3344-t2250-round2-draft.md
```
