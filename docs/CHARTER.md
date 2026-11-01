# TermLink Charter

> The single owned statement of what TermLink is — and deliberately is not.
> README and `docs/ARCHITECTURE.md` both carry a copy of the canonical sentence
> below (markdown has no transclusion, so they are copies, not references). The
> three copies are kept in sync by a canary — `scripts/check-charter-sentence-drift.sh`
> (T-2484) fires if any diverges. Edit the sentence here (it is authoritative), then
> re-sync the two copies. Origin: T-2468 P1 / T-2470.

## Canonical purpose

**TermLink makes sure every message reaches the recipient it is meant for: it finds
that recipient, wakes it in its terminal when it must, hands the message over, and
shows where it is until an agent that can act has it, or it is loudly stuck.**

It is the base layer: it records work handed off between agents, and leaves what
work to do, and who should do it, to the framework above.

This states the product's intended contract; what is covered today and what is not
yet built is listed with each objective (`.context/project/objectives.yaml`, P-1..P-7),
so the sentence is not a claim that every gap is closed.

*(This sentence is human-blessed per the Authority Model — sovereignty over the
project's stated purpose is the human's. Agents propose; the human owns the final
wording.)*

## What TermLink is (the load-bearing nouns)

- **Durable topic records** — append-log channel topics with retention, offsets,
  acks, and replay. The hub is authoritative for coordination within its retention
  window; it is not an archival database. Delivery to an agent that can act is the
  product; everything else is built on it.
- **Hub-mediated** — hubs own their topic records and never relay topic messages
  between hubs; signed directory cards and hub-brokered direct circuits with hub
  fallback are permitted (direct circuits not yet implemented). See
  `docs/ARCHITECTURE.md`.
- **With terminal endpoints** — sessions are real PTYs: peers can stream output,
  inject keystrokes, exec, and doorbell-wake them, not just exchange text.
- **A coordination substrate** — presence, claim/lease work-stealing, DM threads,
  and push-wake exist so N agents can divide and hand off work reliably.

## Non-goals (what TermLink deliberately is NOT)

1. **Not an inter-hub federation layer.** Each hub owns independent topic state;
   cross-hub visibility is explicit, client-driven cross-posting — never automatic
   (G-060). A count delta between hubs for a same-named topic is expected, not a bug.
   *Guarded by* `crates/termlink-hub/tests/no_federation_tripwire.rs` (T-2569): the
   hub crate may not read peer-hub config, may not build a hub-speaking client, and
   its set of outbound connections is enumerated. Adding an auto-relay fails the build.
2. **Not an archival database.** The hub is authoritative for coordination within
   its retention window; it is not an archival database. Topics are
   retention-bounded append logs sized for coordination. Durability means "survives
   a hub blip and replays", not "stored forever".
3. **Not a social / engagement platform.** Presence, threads, reactions, and pins
   serve coordination. Social-analytics surfaces that trace to no coordination
   purpose are removal candidates, not features (T-2471 / P4).
4. **Not a workflow or orchestration engine.** TermLink provides the primitives
   (claim/lease, DM, doorbell, presence); the Agentic Engineering Framework (AEF)
   builds orchestration *policy* on top. The substrate stays mechanism, not policy.
5. **Not a security boundary between mutually-distrusting tenants.** The trust
   model is a cooperating fleet with TOFU-pinned hubs and a shared HMAC secret;
   formal multi-tenant isolation is an open product decision (T-2468 P6), not a
   current guarantee.

## Origin and evolution

TermLink began as a cross-terminal session-control tool (attach to and mirror a
running terminal from elsewhere) and grew into the coordination layer for
multi-agent parallel execution. The authoritative statement of the substrate
design and its invariants is
[`docs/architecture/parallel-execution-substrate.md`](architecture/parallel-execution-substrate.md).
This charter is the *purpose* layer above that *design* document: the design says
how the substrate works; the charter says what it is for and what it refuses to be.
