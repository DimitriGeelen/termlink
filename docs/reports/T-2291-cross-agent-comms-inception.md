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
- **RC3 — Delivery (confirmation).** Default sends record "sent" with no proof of
  receipt; "sent ≠ delivered." *Fix mechanism SHIPPED (T-2286 await-ack); gap =
  opt-in, DM-only, recipient-ack unenforced.*
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
| **V3** delivery-confirm default | RC3 | **5** | **5** | 4 | 4 | **18** | **S** | **Strongest + cheapest.** Mechanism SHIPPED (T-2286 + T-2049 + T-2051); work = flip `--await-ack` to **default** for `/agent-handoff`,`/reply`,`agent-send.sh` + enforce recipient-ack. Turns every silent "sent-but-lost" into a loud timeout. Serves G-019 + PL-011 directly. |
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

1. **V3 first** (cost S) — flip ack-with-retry to default. Near-zero build,
   immediately converts silent loss into loud timeout fleet-wide. Highest
   directive sum (18). Standalone value even before V2/V1.
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

### 2026-06-27 — 5 agents landed, RCA synthesized
- All 5 research agents returned and independently converged. Two corrections to
  the going-in framing: (1) T-1693 per-agent keys are SHIPPED, not unshipped
  (gap = defaults) — A2; (2) the peer registry is ~80% built on agent-presence +
  cv_index, and `kv` is the wrong scope — A4. A1 added a 3rd orthogonal class
  (delivery-confirm). A3 confirmed T-2286 already ships the confirm mechanism
  (gap = opt-in + routing-orthogonal). Recommendation revised DEFER → GO on a
  composite V3+V2+V1 (all "promote shipped primitive," not greenfield). Presented
  to human for go/no-go; variant selection + GO is the human's (sovereignty).
