# T-1830 — Doorbell+mail adoption gap: driving zero active conversations to non-zero

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/completed/T-1830-doorbellmail-adoption-gap--drive-zero-ac.md`.
> The reasoning below was recorded on 2026-05-28; this file relocates it out of
> the archived task file into `docs/reports/` per C-001.
>
> The task file's `## Problem Statement` and `## Assumptions` sections are
> unfilled templates; everything here comes from the recommendation and the
> recorded decision, which are substantial.
>
> **Decision on record: GO** (2026-05-28T12:37:54Z).

## The finding

T-1829's live fleet validation on 2026-05-28 **proved the runtime is healthy** on
all three reachable hubs — `.107`, `.121`, `.122` all returned selftest PASS, in
51 ms, 391 ms and 453 ms respectively. T-1807 had already validated end-to-end
determinism earlier in May.

And yet: **active conversation count = 0, across 91 topics.**

That pairing is the whole finding. Everything worked and nobody was talking. When
every health signal is green and the thing the system exists for is not happening,
the defect is not in the layer the health signals cover.

## Diagnosis — coordination, not infrastructure

The selftest passes everywhere, so the gap is **socio-technical**. Three specific
causes were named:

1. **No discovery primitive** for who is listening.
2. **No convention** for always-on `/check-arc respond` listeners.
3. **`agent-send.sh` requires `--peer-fp` / `--to-session`** — values an operator
   does not have without prior coordination.

The third is a chicken-and-egg: reaching a peer requires knowing something you can
only learn by already having reached them.

## Recommendation — GO to inception

GO was recommended *for inception specifically*, on the grounds that the runtime
work is done and what remains is protocol design plus an adoption convention. The
distinction matters: this was explicitly not a GO to start building.

Three directions to explore:

- **(a)** a heartbeat / listener-presence topic
- **(b)** a discovery verb listing active listeners
- **(c)** `agent-send.sh` auto-discover

Each could become a small build task — *"but the wiring decisions need a
deliberate design pass first."*

## What became of it

Read from 2026-08, all three directions shipped and are load-bearing today:
`agent-presence` with `/be-reachable` heartbeats (a), `/peers` and
`termlink agent find-idle` (b), and `termlink_agent_send_auto_discover` (c). The
cv_index fast path (T-2103/T-2107) later made (b) O(agents) rather than
O(heartbeats).

*(This paragraph is orientation for a future reader, not part of the 2026-05-28
record.)*
