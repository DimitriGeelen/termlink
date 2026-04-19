# T-1074 — Periodic Cross-Agent Learning Exchange (Inception)

**Status:** started-work · inception · owner: human
**Task file:** `.tasks/active/T-1074-periodic-cross-agent-learning-exchange--.md`
**Research artifact:** this file

## Problem Statement (mirrored from task)

Agents in the fleet (this dev box on .107, ring20 LXC sessions, parallel Claude instances, framework-agent) accumulate learnings — bugs encountered, workarounds discovered, protocol gotchas, structural insights — but those learnings stay **local** until something forces an exchange. Current exchange channels are all ad-hoc:

- Pickup envelopes (manual, "this is worth sharing")
- termlink inject (interactive, targeted)
- cross-project git mirrors (slow, human-driven)

**Hypothesis:** a periodic (e.g. 15-min) cron that asks every reachable peer "what did you learn since last exchange?" would surface insights that would otherwise decay with the session.

## Exploration Plan

*(Deferred — this artifact exists to satisfy C-001 documentation gate; the actual spikes happen when human prioritizes the task. See `fw inception status` for live state.)*

Spikes to run when this task moves to horizon=now:

1. **S-1 — Channel inventory.** What already exists? Pickup envelopes, termlink kv, cross-host event.broadcast, learnings.yaml diff-poll. Map each channel's reliability tier and failure modes.
2. **S-2 — Cadence model.** 15-min? On commit? On session-end? On learning-entry write? Pick the event that minimizes wasted pulls against peers that have nothing new.
3. **S-3 — Schema.** What's a "learning" on the wire? Stable key (e.g. PL-NNN), summary, source task, optional patch/fix, fingerprint for de-duplication.
4. **S-4 — Dedup + merge policy.** Receiver-side rule: how to apply inbound learnings without duplicating or overwriting local work.
5. **S-5 — Security model.** Signed envelopes? Pre-shared auth? Or ride termlink's existing hub auth?

## Recommendation

*(Pending — inception decide awaits the spikes above.)*

## Dialogue Log

*(Pending.)*

---

_Research artifact skeleton created 2026-04-19 as part of T-1139 audit remediation (C-001 gate). Fill in as exploration proceeds._
