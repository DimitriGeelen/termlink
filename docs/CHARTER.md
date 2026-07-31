# TermLink Charter

> The single owned statement of what TermLink is — and deliberately is not.
> README and `docs/ARCHITECTURE.md` both quote the canonical sentence below; edit
> it here and the docs follow by reference. Origin: T-2468 P1 / T-2470.

## Canonical purpose

**TermLink is a hub-mediated, durable append-log message bus with terminal
endpoints — the coordination substrate that lets a fleet of AI agents (and humans)
discover each other, exchange durable messages, claim work, and control terminal
sessions across one or many machines.**

*(This sentence is human-blessed per the Authority Model — sovereignty over the
project's stated purpose is the human's. Agents propose; the human owns the final
wording.)*

## What TermLink is (the load-bearing nouns)

- **A message bus** — append-log channel topics with durable retention, offsets,
  acks, and replay. Delivery is the product; everything else is built on it.
- **Hub-mediated** — a strict star: spokes never talk peer-to-peer, the hub
  mediates all coordination (see `docs/ARCHITECTURE.md`).
- **With terminal endpoints** — sessions are real PTYs: peers can stream output,
  inject keystrokes, exec, and doorbell-wake them, not just exchange text.
- **A coordination substrate** — presence, claim/lease work-stealing, DM threads,
  and push-wake exist so N agents can divide and hand off work reliably.

## Non-goals (what TermLink deliberately is NOT)

1. **Not an inter-hub federation layer.** Each hub owns independent topic state;
   cross-hub visibility is explicit, client-driven cross-posting — never automatic
   (G-060). A count delta between hubs for a same-named topic is expected, not a bug.
2. **Not a durable database or system of record.** Topics are retention-bounded
   append logs sized for coordination, not archival. Durability means "survives a
   hub blip and replays", not "stored forever".
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
