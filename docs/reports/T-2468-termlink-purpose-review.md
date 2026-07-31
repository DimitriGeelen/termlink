# T-2468 — Critical Review of TermLink's Purpose & Goals

**Inception task:** T-2468
**Started:** 2026-07-31
**Question:** Where does TermLink's implementation diverge from its stated purpose,
and which divergences warrant action?

## Method

Four parallel review dimensions (IW-1..IW-4), each producing ranked gaps with
file:line evidence, synthesized into a prioritized gap register (IW-5). Yardstick:
the Four Constitutional Directives (Antifragility, Reliability, Usability, Portability).

## Canonical purpose statements

- `README.md:3-4` — "A coordination substrate for parallel AI agents — a hub-mediated,
  durable append-log message bus with terminal endpoints."
- `README.md:11-13` — "It began as a cross-terminal session-control tool and grew into
  the coordination layer for multi-agent parallel execution" (mission-evolution admitted
  in the canonical doc).
- `docs/architecture/parallel-execution-substrate.md:24-27` — "most of the substrate's
  job is not 'isolate trees' but 'provide the primitives that let the orchestrator assign
  disjoint work and detect when disjointness is violated.'"
- `docs/ARCHITECTURE.md:3` — "Cross-terminal session communication system in Rust …
  enables multiple terminal sessions to communicate." (older purpose, still canonical, unretired)

**Key correction (IW-1):** The Four Constitutional Directives (Antifragility/Reliability/
Usability/Portability) belong to the **AEF meta-framework**, NOT to TermLink-the-product.
TermLink has **no product-level charter of its own** — its purpose lives only in README +
the substrate ADR, and those two disagree. This itself is finding #1.

## Findings by dimension

### IW-1 — Purpose–reality alignment

- **[HIGH] Two live, non-identical purpose statements coexist unreconciled** — "coordination
  substrate for parallel AI agents" (README.md:3) vs "cross-terminal session communication
  system" (docs/ARCHITECTURE.md:3). A new user gets a different product depending which doc
  they open. Drift documented but not resolved.
- **[HIGH] OVER-SERVICE: ~57 of 276 MCP tools (~21%) are Slack-clone social analytics** —
  emoji_stats, reaction_rate, star/pin/poll, thread_health, top_reacted, top_pinners,
  co_posters, post_streak, typers. No substrate purpose requires reaction leaderboards. Pure
  mission creep. (evidence: 276 tools total, ~57 social-analytics.)
- **[HIGH] UNDER-SERVICE: the substrate's own stated core job — collision detection on
  disjoint write-sets — is unbuilt.** ADR §2: "No filesystem-write observation, anywhere…
  the hub cannot see what files an agent touches." The system that exists to "detect when
  disjointness is violated" cannot observe writes.
- **[MED] Scope formally undecided (multi-tenant)** — trust model is single-operator
  ("treat every authenticated peer as trusted", README.md:53) yet concerns.yaml:480 still
  asks whether multi-tenant is in scope. Unowned purpose boundary.
- **[MED] Task/verb sprawl is now a cost** — 2097 completed tasks, ~40 CLI/skill verbs,
  ~39k tokens/agent of MCP descriptions (arc-005 exists because the surface is too large).
- **[LOW] README claims "30 commands"** while real surface is 276 MCP tools + ~40 skills —
  headline understates sprawl (README.md:82).

Recommended inceptions: (1) reconcile into one canonical TermLink charter [Usability];
(2) social-analytics decommission-or-justify review [recovers context budget];
(3) close-or-descope collision-detection under-service [Reliability].

### IW-2 — Comms rail reliability (the doorbell replacement)

Round-trip map: **DELIVER** = durable/guaranteed (offline-queue T-2051, dedupe T-2049);
**WAKE** = best-effort (WS push over poll floor, drops on Lagged / pre-subscribe;
depends on out-of-band pushwaker that may be dead); **CONSUME** = **NOT guaranteed, silent
break** (a busy/manual-accept session gets the inject unsubmitted, discarded on next
`--continue`; LIVE+pty_session ≠ listening); **ACK** = observable + retry-enforced but ONLY
via `--await-ack` path (raw `inject`/bare post bypass it); **REPLY** = app-level.

- **[HIGH] Wake-consumption is unconfirmed (rung-but-not-read fails silently)** — the
  load-bearing hole (G-083). Heartbeat proves process-alive, not session-listening; no
  post-ring read-cursor check. Blind since the doorbell shipped (T-1800).
- **[MED] Exactly-once does not survive hub restart / >5-min blip** — hub dedupe is
  in-memory (5-min TTL) while client replays client_msg_id durably → double-append on
  restart. ADR §5 overstates the guarantee (G-088, incepted T-2459).
- **[MED] WS push drops lagged/pre-subscribe wakes with no gap signal** — degrade-to-poll
  contract undocumented/unenforced, no cursor on ws_subscribe (T-2371, open).
- **[MED] Shipped≠live: rail depends on out-of-band wakers/binaries that may never run**
  (G-069) — mitigated only by crons that must themselves fire.
- **[MED] Ack-with-retry only fires on the agent-send path** — enforced leg is opt-in, not
  structural (T-2286).
- **[LOW] Reliability substantially achieved by a canary pile** — observers, not closers;
  the consumption-confirmation primitive is still unbuilt.

Recommended inceptions: (1) consumption-confirmation as next loud-contract link (closes
G-083, highest value) [Reliability]; (2) persist await-ack dedupe to runtime_dir OR
downgrade ADR §5 to at-least-once + idempotent consumer [Reliability]; (3) enforce WS
degrade-to-poll invariant [Antifragility].

### IW-3 — Coordination substrate: coherence vs bloat

_(pending agent)_

### IW-4 — Adoption / shipped≠live / portability

_(pending agent)_

## Prioritized gap register (IW-5 synthesis)

_(filled after all four dimensions return)_

## Recommendation

_(per-gap GO/NO-GO)_
