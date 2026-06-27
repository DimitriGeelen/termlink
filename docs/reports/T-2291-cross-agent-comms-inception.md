# T-2291 — Permanent cross-agent comms fix (identity + delivery): inception

> Inception research artifact (C-001). Created 2026-06-27. Recommendation at
> filing: **DEFER** — this artifact produces the RCA + 5 directive-scored
> remediation variants; the GO + variant selection is the human's (sovereignty).
>
> Predecessor staging: `.context/working/inception-plan-cross-agent-comms.md`.
> Operator request (2026-06-27): "find a permanent solution … incept, ask other
> agents for issues too, present me RCA, with 5 variants for structural
> remediation scored against framework directives … 5 termlink research agents."

## 1. Problem statement (operator-reported, recurring)

Cross-agent communication repeatedly fails. The triggering incident: a
`card-redirect` agent's handoff to the manager was lost. The peer's own
diagnosis named two **compounding structural root causes**:

1. **Shared host fingerprint (identity layer).** All Claude agents co-resident
   on host `.107` (card-redirect, video-app, wd-agent, framework-agent, …)
   authenticate with the host key and share ONE identity `d1993c2c64…`. Every
   manager↔.107-agent DM collapses onto a single `dm:` topic; the only
   discriminator is the human-typed `[manager -> X]` body tag. No per-agent
   addressability. T-1693 (per-agent keys) is the designed-but-unshipped fix.
   See memory `reference_shared_host_identity`; T-1427 (verified identity).

2. **No inter-hub federation — BY DESIGN, not a bug (delivery layer).** A post
   on hub A (`.122`) NEVER reaches a reader on hub B (`.107`). There is no
   mirror that "lags"; it never syncs (G-060 / T-2229 / PL-176). Senders don't
   know which hub a peer reads; "sent" ≠ "delivered." card-redirect's handoff
   sat at offset 48 on `.122` while the manager read `.107` → never arrived.
   See memory `reference_chatarc_dm_federation`; CLAUDE.md §"Channel Topic
   Semantics — Per-Hub State (G-060)".

**Recurrence evidence (systemic, not a one-off):** ring20 T-1259 / T-1264 /
T-1296 (re-filed 3×); ring20 G-155 (false-green comms probe) + G-156 (no peer
registry); this session's card-redirect handoff loss; the ring20 no-federation
correction couriered to AEF (processed `TL-COURIER-ring20-T1296`, R1–R5).

## 2. Open questions (inception working questions)

- **IW-1:** Is controlled cross-hub relay in scope, or a non-goal? (G-060 is a
  deliberate design choice — any "federation" variant must be opt-in + explicit
  + loud; passive replication is a known invalid premise per T-2229.)
- **IW-2:** Identity-fix vs delivery-fix sequencing — which unblocks more, and
  can one ship without the other?
- **IW-3:** Is single-hub convergence acceptable (eliminates cross-hub delivery
  at a portability/SPOF cost)?

## 3. Research plan (5 agents) — findings appended below as they land

| Agent | Stream | Output |
|---|---|---|
| A1 | Cross-agent issue harvester (reach live peers via termlink) | failure catalogue |
| A2 | Identity-layer RCA (shared fingerprint, T-1693, T-1427) | identity findings |
| A3 | Delivery-layer RCA (no-federation, peer registry, delivery-confirm) | delivery findings |
| A4 | Substrate-capability scout (kv, cv_index, claim, ack-retry, find-idle) | primitive→cause map |
| A5 | Prior-art + directive-scoring lead (ring20, AEF, T-1693, G-060/155/156) | scoring rubric + pre-scores |

### A1 — Cross-agent issue harvester (`.context/working/T-2291-A1.md`)
**16 distinct failures catalogued.** Cluster into 3 classes: (1) no inter-hub
federation (G-060/PL-176/T-1791/T-1986 — DM showed 30 posts on `.122` vs 22 on
`.107`); (2) shared-host identity collapse (G-056/PL-166 — reproduced live, every
`.107` listener signs `d1993c2c3ec44c94`); (3) **no delivery/ack confirmation**
(PL-011 + G-063: `framework:pickup` write-only sink, 36 envelopes / 0 receipts) —
orthogonal to both. **Live fleet DARK** (2026-06-27): only `.107` responded;
`.141` no-route, `.122`/`.121` timeout; **0 of 8 listeners LIVE**; `whoami`
returned 100+ ambiguous co-resident sessions. The darkness itself underscores the
false-green failure mode.

### A2 — Identity-layer RCA (`.context/working/T-2291-A2.md`)
Identity = ed25519 keypair; fingerprint = `sha256(pubkey)[..16]`
(`crates/termlink-session/src/agent_identity.rs:174`). Which key a session loads
is pure file-path precedence (`registration.rs:48`): `TERMLINK_IDENTITY_FILE` →
`TERMLINK_IDENTITY_DIR` → **`$HOME/.termlink/identity.key`** (host default).
Co-resident agents share `$HOME`, set no override → same key → same fingerprint.
DM topics are `dm:<sorted_a>:<sorted_b>` (`channel.rs:927`) so a shared fp
collapses every DM onto one topic. **Correction to the problem statement:
T-1693 is SHIPPED, not unshipped** (G-056 RESOLVED 2026-05-19) — per-agent
`--identity-key`, `TERMLINK_IDENTITY_FILE`, `whoami`/`doctor` hints all landed.
The crypto = **S, done**; the remaining gap = **adoption/defaults (M)**: nothing
in `register` / `/be-reachable` / `listener-heartbeat.sh` actually *sets* a
per-agent key. Blast radius (all reversible, transport trust untouched):
DM-topic discontinuity on key change, presence/find-idle fp churn, TOFU re-pin.

### A3 — Delivery-layer RCA (`.context/working/T-2291-A3.md`)
No-federation is **ratified working-as-designed** (T-2229) with an explicit
rejection cost-list (T-1793: state-sync, consistency, conflict-resolution,
bandwidth, retention divergence) under "explicit never implicit" + Reliability.
The only open thread is **discoverability** of that fact. **T-2286
`channel post --await-ack` already ships delivery-confirm** (2026-06-25): polls
`channel.receipts` frontier, re-posts on deadline (exactly-once via T-2049
dedupe), exits loud on exhaustion — directly answers PL-011. **Gap:** it's
opt-in (default sends still silently record "sent"), DM-only, recipient-ack half
is unenforced convention, and **orthogonal to routing** (await-ack on the wrong
hub times out forever). Peer-registry (G-156): agent-presence carries
`agent_id/role/listen_topics/capabilities/host` but **no `hub_addr`**; `/peers`
reconstructs `agent_id→hub_addr` by walking `hubs.toml` but is stale-prone +
script-tier; `find-idle` is local-hub-only. False-green probe (G-155): tests
configured peers, not the intended correspondent on the hub it reads.

### A4 — Substrate-capability scout (`.context/working/T-2291-A4.md`)
**`kv` is the WRONG scope** — `SessionContext.kv` is per-session in-memory, not
fleet state; don't build a registry on it. **The agent-presence heartbeat +
cv_index pair is the highest-leverage substrate:** heartbeats publish a hub-wide,
cv-indexed envelope keyed by a **stable logical `agent_id`** (the RC1 antidote —
identity decoupled from the shared host fp) already carrying
`listen_topics/host/capabilities`, readable as a roster in O(N) via
`channel cv-keys agent-presence`. A peer registry is ~80% shipped; needs only two
**additive** deltas: (1) put the agent's reachable **`addr:port`** in heartbeat
metadata (today `host` = who, not where-to-post), (2) a **fleet-rollup reader**
fanning per-hub cv_index across `hubs.toml`. ack-with-retry (T-2286) adds **no
new hub state** (composes T-2049 dedupe + T-2051 queue + receipts frontier).

### A5 — Prior-art + directive-scoring lead (`.context/working/T-2291-A5.md`)
Pre-scored all 5 variants (table below, reconciled in §5). Three load-bearing
prior-art conclusions: (1) **V3's mechanism already exists** (T-2286
work-completed 2026-06-25) — V3 = "generalize + make default," cost S, highest
directive sum; (2) **no-federation is BY DESIGN** (0 federation primitives exist
in code) — V4/V5 must respect that passive replication is a known invalid
premise; (3) **the failure classes are orthogonal** → V1+V2+V3 is the likely
composite; V5 conflicts with Portability. (AEF courier P-051 unreadable this
session — T-559; R-mapping taken from the in-repo staging plan.)

## 4. RCA (synthesized)

Cross-agent comms fail along **three orthogonal structural axes**, plus a
**meta-cause** that explains the *recurrence*:

- **RC1 — Identity (addressability).** Co-resident agents share one host
  fingerprint, so DM topics collapse and attribution is structurally impossible.
  *Fix mechanism SHIPPED (T-1693/G-056); gap = it is not the default.*
- **RC2 — Routing (addressing).** No canonical `agent_id → hub_addr →
  topics-read` registry; a sender cannot resolve *which hub a peer reads*, and
  hubs never federate (by design). *Fix substrate ~80% SHIPPED (agent-presence +
  cv_index); gap = no `addr:port` field + no fleet-rollup reader.*
- **RC3 — Delivery, which is TWO coupled sub-problems (notify + confirm).**
  - **RC3a — Notification/wake (the load-bearing one).** A harness-driven agent
    is turn-based and idle between turns; it does not watch the bus. If nothing
    *wakes* it, the message sits unread, no receipt is ever written, and the
    confirm chain is plumbing on a tap that never opens. Poll-to-discover is
    structurally impossible for harness agents (§5 rejection). The shipped
    epoch-1 mechanism (T-1800 PTY doorbell) is *preemptive* and can be **missed
    mid-turn** (the open T-2285-class gap). **Mechanism of choice = the epoch-2
    §5 deterministic-sidecar listener:** a no-LLM sidecar does a **remote-write →
    local flag/KV + heartbeat timestamp** on the recipient host; the agent
    **cooperatively polls the local flag at its own yield points**; a **stale
    timestamped delta ⇒ "deaf" ⇒ stop before acting** (self-check-ears) so a
    broken listener is *self-detected*, never silently missed; sender
    missing-ack ⇒ retry. "The flag is a file/KV, not a keystroke." Determinism
    comes from the timestamp (absence of a fresh delta is itself the signal),
    not from the transport. Source: AEF ADR §5 / `docs/architecture/parallel-
    execution-substrate.md`; substrate homes shipped (`kv` flag, `agent-presence`
    timestamp, T-2051 queue).
  - **RC3b — Confirmation.** Once read, "sent ≠ delivered" is closed by the
    recipient-ack advancing the `channel.receipts` frontier; the sender's
    await-ack retry reads it. *Mechanism SHIPPED (T-2286 `--await-ack`,
    `e67ded8f`); gap = opt-in, DM-only, recipient auto-ack is an unenforced
    sidecar convention.* The §5 sidecar is also the natural home for the
    recipient auto-ack (it emits `channel.ack` after materializing the mail).
- **META — Discoverability.** No-federation is a deliberate design choice that is
  not surfaced *at the moment of failure*, so peers (ring20 T-1259/T-1264/T-1296
  ×3, and the manager in the triggering incident) keep re-diagnosing it as "the
  mirror lags." This is why the same misconception recurs.

**Why the framework stayed blind (G-019 lens):** the comms probe is false-green
(G-155 — tests configured peers, not intended correspondents); `framework:pickup`
is a write-only sink (G-063 — 0 receipts); there is no delivery canary. The
3×-refiled ring20 tickets are the signature of a framework with no structural
counter to the misconception. RC1/RC2/RC3 each have a *shipped mechanism* but no
*default + observability*, so failures stay silent until a human notices a lost
handoff.

## 5. The 5 remediation variants — developed + scored

Scores 1–5 per directive (priority order Antifragility > Reliability > Usability
> Portability); Cost S/M/L/XL. Reconciled across A2/A3/A4 (which corrected A5's
cost basis where a mechanism turned out already-shipped).

| Variant | Fixes | A | R | U | P | Sum | Cost | Note (post-reconciliation) |
|---|---|---|---|---|---|---|---|---|
| **V1** per-agent identity keys | RC1 | 3 | 4 | 4 | 4 | 15 | **S→M** | Crypto SHIPPED (T-1693/G-056); work = make per-agent key the **default** in register/`be-reachable`/heartbeat. Blast: DM-topic discontinuity, fp churn, TOFU re-pin (reversible). |
| **V2** fleet peer registry | RC2 | 3 | 4 | **5** | 3 | 15 | **S→M** | ~80% SHIPPED on agent-presence + cv_index (A4); add `addr:port` to heartbeat + fleet-rollup reader. **NOT `kv`** (per-session, wrong scope). Biggest "stop knowing topology by hand" win. New fleet state = staleness risk. |
| **V3** delivery (notify + confirm) | RC3a+b | **5** | **5** | 4 | 4 | **18** | **S→M** | **Strongest.** Confirm half SHIPPED (T-2286 `--await-ack` + T-2049 + T-2051) — flip to default + enforce recipient auto-ack. **Notify half = the §5 deterministic-sidecar listener** (remote-write→local flag/KV + heartbeat timestamp; cooperative yield-point poll; stale-delta self-check; missing-ack retry) — designed (AEF epoch 2), not yet built for comms → the real cost. Replaces the preemptive PTY doorbell (T-2285 miss-gap). Turns silent "sent-but-lost" into deterministic, self-detecting delivery. Serves G-019 + PL-011 directly. |
| **V4** opt-in cross-hub relay | RC2(alt) | 3 | 4 | 3 | 3 | 13 | L | A deliberate, loud cross-post bridge (NOT passive replication, which G-060/T-2229 reject). Net-new hub surface = new failure mode. Only if a hard cross-hub need emerges that V2 routing can't serve. |
| **V5** single-hub convergence | RC2(alt) | 2 | 4 | 4 | **1** | 11 | XL | Simplest delivery story but **violates Portability** (SPOF — one hub down = whole fleet dark, contra A1's observed darkness) + low Antifragility. XL migration. Lowest score. |

**Key interaction (the reason a composite is needed):** V3 alone makes failure
*loud* but does not *route* — `--await-ack` on the wrong hub times out forever.
V2 supplies the routing (which hub the peer reads). V1 makes the registry's
per-agent keys collision-free at the DM-topic layer. The three compose; none
fully solves comms alone.

## 6. Recommendation (advisory — GO/variant selection is the human's)

**Going-in recommendation was DEFER; after the RCA it is GO on a composite of
V3 + V2 + V1, sequenced V3 → V2 → V1.** Rationale:

1. **V3 first** (cost S→M) — TWO halves. (a) *Confirm*: flip T-2286 await-ack to
   default (near-zero, shipped). (b) *Notify*: stand up the §5 deterministic-
   sidecar listener (remote-write → local flag/KV + heartbeat timestamp;
   cooperative yield-point poll; **stale-delta self-check ⇒ deaf ⇒ stop**;
   missing-ack retry) as the cross-agent wake substrate — *replacing* the
   preemptive PTY doorbell (T-2285 miss-gap). The notify half is the real build;
   homes exist (`kv`, `agent-presence`, T-2051). Determinism is the timestamp,
   not the transport. Highest directive sum (18); standalone value before V2/V1.
2. **V2 second** (cost S→M) — add `addr:port` to the agent-presence heartbeat +
   a fleet-rollup registry reader. Gives senders the *right hub*, so V3's
   await-ack lands where the peer actually reads. Builds on shipped substrate.
3. **V1 third** (cost S→M) — make per-agent identity keys the default in the
   register/`be-reachable`/heartbeat path. Resolves the DM-topic collapse and
   restores attribution. Coordinate the rollout (DM-topic discontinuity).

Plus a **near-free META fix** folded into V3/V2: a loud "this hub does not
federate — did you mean `--hub <peer>`?" hint + a delivery canary, so the
discoverability gap stops re-spawning the "mirror lags" misconception.

**Reject V5** (violates Portability directive — SPOF). **Hold V4** unless a
concrete cross-hub need survives V2 (explicit relay is the only defensible
federation, but it adds a failure surface V2 routing avoids).

Each leg is "promote/extend an already-shipped primitive," not greenfield —
total ≈ S + (S→M) + (S→M). On GO, file **three separate build tasks** (one per
leg, per task-sizing rules) — do not build under this inception ID.

**Sovereignty:** the GO decision and the variant/composite selection are the
human's. Record via `fw task review T-2291` (Watchtower).

## Dialogue Log

### 2026-06-27 — inception filed
- Operator authorized the inception (5 variants, directive-scored, 5 research
  agents, ~8 turns). Filed T-2291 with going-in recommendation DEFER. This
  artifact created before research (C-001).

### 2026-06-27 — refinement dialogue with operator (step-by-step)
- **Step 1 — RCA decomposition: ACCEPTED.** Operator confirmed the three
  orthogonal axes (RC1 identity / RC2 routing / RC3 delivery) + META
  discoverability; no RC4 raised, not over-split.
- **Step 2 — RC1 identity / V1: keying model = (a) stable per-agent-id key**
  (`~/.termlink/identities/<agent_id>.key`, reused across sessions; identity =
  logical role, keeps DM history continuous while agent_id stable). **Clean
  cutover accepted** — no DM-history migration/alias needed (fleet is dark; old
  shared-key threads may orphan). This is also what keys RC2's registry correctly.
- **Step 3 — RC2 routing / V2:** registry built on agent-presence + cv_index
  (NOT kv). Two additive deltas: (1) `addr:port` in heartbeat, (2) client-side
  fleet-rollup reader over `hubs.toml`. **Decisions:** `addr:port` source of
  truth = **hub-stamped observed address**, self-report as fallback (renumber-
  proof — ring20 churn lesson). **Fleet rollup = client-side fan-out** (respects
  G-060 no-federation), **hubs.toml bootstrap dependency accepted for v1**
  (self-discovery deferred as a separate, larger problem).
- **Step 4 — RC3 delivery / V3: SPLIT into RC3a notify + RC3b confirm.** Operator
  flagged the missing tier: confirmation is plumbing on a tap that never opens
  unless the recipient is *woken*. Confirm half (RC3b) = T-2286 await-ack
  (shipped). **Async-tracked chosen (1b)** but with a DETERMINISTIC notify
  mechanism, NOT a poll/preemptive doorbell. **Mechanism = the epoch-2 AEF §5
  deterministic-sidecar listener** ("remote file write through a listener +
  timestamped delta"): no-LLM sidecar does remote-write → local flag/KV +
  heartbeat timestamp; agent cooperatively polls the local flag at its yield
  points; stale-delta ⇒ deaf ⇒ stop (self-check-ears); missing-ack ⇒ retry.
  Determinism = the timestamp (absent fresh delta is itself the signal), not the
  transport. This **answers two challenges I (agent) raised**: a local file CAN
  cross hosts because the listener runs on the recipient host and owns the hop;
  and it is neither bus-polling (cheap local flag read) nor preemptive (no
  mid-turn miss; replaces the T-2285-gapped PTY doorbell). V3 cost revised S →
  **S→M** (the notify listener is the real, not-yet-built-for-comms delta;
  homes exist: `kv`, `agent-presence`, T-2051). Source: AEF ADR §5 /
  `docs/architecture/parallel-execution-substrate.md`; [[reference_wakeup_two_epochs]].
- **Step 5 — META discoverability: NO standalone variant — folds into V2/V3.**
  V2 removes the *need* to know hub topology by hand; V3 removes the *silence*.
  Residual META = 3 cheap riders: (1) loud in-the-moment "this hub does not
  federate — peer reads `<hub>`" hint (rides V2); (2) **delivery canary = the
  unconfirmed-set made observable — MUST-HAVE**, falls out of V3 async-tracked
  for free, kills the write-only-sink class (`framework:pickup` 36-sent/0-recv);
  (3) **false-green probe fix (G-155) IN SCOPE as a small V2 follow-on** (probe
  the intended correspondent on the hub they read, not configured peers — needs
  the registry to know who/where).
- **Step 6 — V4/V5 + the V6 pivot.** V5 single-hub **REJECTED** (Portability=1,
  SPOF — A1 saw the fleet dark live; XL). V4 continuous-relay **HOLD/lean-reject**
  (V2 + explicit `--hub` cover per-message cross-hub; V4's standing bridge ≈ the
  passive replication G-060 rejects). **Operator pivot:** critically re-evaluate
  the "comms over hub" principle → **V6: direct host-to-host transport-first +
  discovery service + hub-as-fallback.** Critical reframe (agent): V6 does NOT
  escape V1/V2/V3 — identity (auth the socket + gate discovery), discovery-
  registry (V2 re-pointed to `host:port`), and §5-notify (wake needed on any
  transport) are the shared foundation. V6 adds a transport plane on top. Cost
  **L→XL**; gated on a reachability spike (dispatched).
  - **Bidirectionality (operator catch):** V2 registry is SYMMETRIC (any agent
    resolves any agent both directions); V1 is the prerequisite that makes the
    *sender* nameable for the reply. Both ends must be registered; envelope
    carries self-reported return-address as fallback (hub-stamped = source of
    truth, mirrors Step-3). Conversation locus: **(a) split/registry-resolved**
    chosen for v1 ((b) home-hub later if thread-locality hurts; (b) gives V4's
    shared-locus benefit without the blind mirror).
  - **Audit (operator inversion — CORRECT):** the single hub-firehose IS the
    obfuscation problem, not the audit asset — 31,527 heartbeats = 70.5% of the
    store, ~30k stale, O(N≈30k) join walk (comms-analysis 2026-06-22). **Decision:
    durable 1-to-1 messages move OFF the firehose** → per-conversation/per-agent
    append-only journal (T-2250 Tier-0 pattern + offline-queue/ack-retry SQLite);
    fleet forensic = aggregate-on-demand across journals (the `/recent-dm`-walks-
    hubs pattern, re-pointed). Hub keeps presence/discovery, broadcast, store-and-
    forward fallback. Makes messages MORE findable.
  - **Discovery (operator decision): TWO-TIER.** Tier 1 = LAN broadcast (CSMA/CD-
    style, hub-optional, renumber-proof — antidote to ring20 churn, same-broadcast-
    domain only). Tier 2 = hub registry (primary/initial/bootstrap + cross-VLAN +
    fallback). Guardrail: broadcast is a HINT not trust — V1 keypair (T-2024
    symmetric auth) is the gate; discovery resolves *where*, identity proves *who*.
  - **Confirm ladder (3 levels, operator: elaborate→accepted):** (1) TCP ACK =
    weak, ignore as delivery; (2) sidecar "journaled" receipt on the open socket =
    strong synchronous "delivered to mailbox," no hub/poll, AND makes the direct
    path store-and-forward (survives recipient restart); (3) read-receipt =
    async "consumed" at the agent's yield point. Direct path confirms via (2)+(3);
    **hub receipts-frontier (T-2286) = FALLBACK-path confirm ONLY.** One sender-API
    confirm contract across both transports; §5 sidecar is the common receipt
    producer. Hub fallback now triggers only when the recipient HOST is unreachable
    (not merely agent-busy — the sidecar journal covers that).

### 2026-06-27 — 5 agents landed, RCA synthesized
- All 5 research agents returned and independently converged. Two corrections to
  the going-in framing: (1) T-1693 per-agent keys are SHIPPED, not unshipped
  (gap = defaults) — A2; (2) the peer registry is ~80% built on agent-presence +
  cv_index, and `kv` is the wrong scope — A4. A1 added a 3rd orthogonal class
  (delivery-confirm). A3 confirmed T-2286 already ships the confirm mechanism
  (gap = opt-in + routing-orthogonal). Recommendation revised DEFER → GO on a
  composite V3+V2+V1 (all "promote shipped primitive," not greenfield). Presented
  to human for go/no-go; variant selection + GO is the human's (sovereignty).
