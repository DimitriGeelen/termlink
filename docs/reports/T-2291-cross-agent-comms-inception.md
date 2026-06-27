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

### A1 — Cross-agent issue harvester
_pending_

### A2 — Identity-layer RCA
_pending_

### A3 — Delivery-layer RCA
_pending_

### A4 — Substrate-capability scout
_pending_

### A5 — Prior-art + directive-scoring lead
_pending_

## 4. RCA (synthesized — filled at T5–T6)

_TBD after agent findings land._

## 5. The 5 remediation variants (developed + scored — filled at T7)

- **V1 — Per-agent identity keys (T-1693).** Each agent gets its own keypair.
- **V2 — Fleet peer registry (R2/G-156).** Canonical `agent_id → hub_addr →
  topics-read`, fleet-discoverable.
- **V3 — Mandatory delivery confirmation (R3).** Generalize T-2286
  ack-with-retry to all cross-agent sends; never record "delivered" without a
  receipt.
- **V4 — Opt-in cross-hub relay primitive.** A deliberate, loud cross-post
  bridge (distinct from rejected passive replication).
- **V5 — Single-hub convergence.** Collapse the fleet onto one hub.

### Directive-scoring rubric (Four Constitutional Directives + cost)

| Variant | Antifragility | Reliability | Usability | Portability | Cost | Notes |
|---|---|---|---|---|---|---|
| V1 per-agent keys | | | | | | |
| V2 peer registry | | | | | | |
| V3 delivery-confirm | | | | | | |
| V4 opt-in relay | | | | | | |
| V5 single-hub | | | | | | |

## 6. Recommendation (filled at T8, presented to human)

_TBD — variant selection + GO is the human's (sovereignty)._

## Dialogue Log

### 2026-06-27 — inception filed
- Operator authorized the inception (5 variants, directive-scored, 5 research
  agents, ~8 turns). Filed T-2291 with going-in recommendation DEFER. This
  artifact created before research (C-001).
