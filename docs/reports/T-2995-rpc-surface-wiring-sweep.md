# T-2995 — RPC-surface wiring-check sweep

**Status:** exploration in progress. Created BEFORE research (C-001).
**Concerns:** C-27 (`orchestrator.route`), C-28 (`dialog.presence`), C-31 (`session.*`),
C-32 (`event.*` family), C-33 (`event.broadcast`).
**Source:** `docs/reports/VALUE-REVIEW-repo-2026-09-19-consolidated.md` rows C-27..C-33 (S-21).

## Why these five are one question, not five

Each row asks the same thing of a different RPC method: **is this surface wired, or is it
resident code nobody can reach?** One predicate, five instances — so this is one inception
under the sizing rule, not an umbrella. What differs per row is only the *disposition* a
clean answer implies (WIRE, DELETE, or leave alone), and one of those dispositions —
C-27's, against non-goal #4 — is explicitly reserved to the human.

## The prior that must be tested first

C-30 found that `kv.*` usage is **structurally invisible**: session-daemon calls never pass
the hub audit sink, so a zero-call reading there measures the instrument, not the traffic.
C-31 records that the same blind spot "plausibly applies (not individually verified)".

That makes IW-2 load-bearing for the whole sweep. If the blind spot extends to `session.*`
and `event.*`, then "zero calls" in those rows is not weak evidence of disuse — it is **no
evidence at all**, and any DELETE reasoning resting on it is unsound regardless of how the
wiring matrix comes out. IW-2 is therefore answered before any disposition is proposed.

## Open questions

- **IW-1** — per-surface wiring matrix: hub-implemented? CLI verb? MCP tool?
- **IW-2** — does the C-30 audit blind spot apply to `session.*` / `event.*`?
- **IW-3** — does the `event.broadcast` reference sweep (DELETE checks 4-5) come back clean?
- **IW-4** — does anything depend on `orchestrator.route` remaining present (the federation tripwire)?

## Findings

### F1 — The blind spot is real, and now individually verified (IW-2)

`crates/termlink-hub/src/server.rs:1610` is the **only** general dispatch audit call site
(`rpc_audit::record(&req.method, ...)`). It fires on requests reaching the hub's JSON-RPC
server. `crates/termlink-session/src/handler.rs` — which serves `session.register`,
`session.deregister`, `session.heartbeat`, `session.update` — contains **zero** references
to `rpc_audit` or any audit sink. Measured: `grep -rn 'rpc_audit|audit' handler.rs` returns
nothing.

So for the C-31 lifecycle methods, "zero hub-observed calls" is not weak evidence of
disuse. It is **structurally guaranteed regardless of traffic**. C-31 asked for the
verification C-30 had not done; this is it, and it comes back positive.

There is a second, independent blind spot the review did not name. `rpc_audit.rs:47`
declares `SKIP_METHODS = ["event.poll", "event.collect"]` — deliberately excluded because
"a single `event collect` CLI invocation generates ~13K `event.collect` dispatches"
(T-1307). Those two methods are invisible **by design**.

**Consequence for this sweep:** every zero-call reading in C-31 and C-32 is inadmissible as
evidence of disuse. Nothing here may be deleted on a usage argument until C-45/C-30
telemetry (T-2996) exists. What CAN be decided is wiring — a structural property the source
answers directly — which is what the rest of these findings rest on.

### F2 — The wiring matrix (IW-1)

| method | hub arm | CLI | MCP | reading |
|---|---|---|---|---|
| `orchestrator.route` | yes (`router.rs:78`) | 0 | 0 | reachable only by raw JSON-RPC — see F4 |
| `dialog.presence` | yes (`router.rs:140`) | 0 | 0 | handler + producer + tests, no client surface — F5 |
| `session.register` / `.deregister` / `.heartbeat` / `.update` | **no** | via daemon | — | served by `termlink-session/src/handler.rs`, not the hub |
| `event.emit` | **no** (hub is a *client*) | 7 files | 1 | `rpc_call_addr(EVENT_EMIT)` at `supervisor.rs:144`, `router.rs:435` |
| `event.emit_to` | yes (`router.rs:77`) | — | — | live, advertised |
| `event.subscribe` | yes, hub-level only (`router.rs:74`) | 4 files | 1 | live |
| `event.topics` | no | 2 files | 1 | daemon-served |
| `event.state_change` | no | 0 | 0 | **orphan constant** — F6 |
| `event.error` | no | 0 | 0 | **orphan constant** — F6 |
| `event.broadcast` | **retired** | residue | residue | cut 2026-05-31 — F3 |

The hub router carries 64 method arms; only five are `session.*`/`event.*`
(`SESSION_DISCOVER`, `SESSION_WHOAMI`, `EVENT_COLLECT`, `EVENT_SUBSCRIBE`,
`EVENT_EMIT_TO`). Everything else in those families is daemon-served or forwarded through
the `_ => forward_to_target` catchall.

### F3 — C-33 is reading a stale residue: `event.broadcast` was retired four months ago (IW-3)

`router.rs:1017` records it plainly: *"T-1166 / T-1415: `event.broadcast`, `inbox.list`,
`inbox.status`, `inbox.clear` were retired (cut landed **2026-05-31**). Their advertisement
here was removed 2026-06-05."* `router.rs:70` confirms the arms were deleted. The hub
returns `-32601` for it today.

The reference sweep (DELETE checks 4-5) comes back **clean of callers** — but the finding is
larger than "clean". What survives is residue for a method the hub no longer serves:
- `control.rs:69` — the `EVENT_BROADCAST` constant
- `rpc_audit.rs:55` — a `LEGACY_METHODS` entry that warns on dispatch of a retired method
- `tests/no_legacy_callers.rs` — a guard test that **enforces** nobody calls it

So C-33's premise ("recorded intent to retire, zero calls, DELETE candidate") is inverted:
the intent was already executed. Zero calls is not evidence of disuse, it is the guaranteed
consequence of the cut. And `no_legacy_callers.rs` is a live tripwire that references the
constant by name — removing the constant carelessly breaks the guard that keeps the
retirement enforced. The residue is small and its removal is a tidy-up with a test
dependency, not a DELETE decision.

### F4 — C-27's "shelved scaffolding" reading does not survive contact (IW-4)

`orchestrator.route` has zero CLI verbs and zero MCP tools — the review is right about the
client surface. Everything else it infers from that is wrong:

- The handler spans `router.rs:1160-1507+` — route caching, circuit breaking, a bypass
  registry, candidate promotion.
- `route_cache.rs` is a **dedicated module** describing it as "Layer 3" of a three-layer
  routing design sitting behind the bypass registry (Layer 1).
- `tests/e2e/level8-orchestration-harness.sh` **drives it end-to-end** via raw JSON-RPC
  payloads (`level8-orchestration-harness.sh:121`).

And on the tripwire C-27 warns about: `tests/no_federation_tripwire.rs:43` names
`orchestrator.route` explicitly as the residual path that *"is NOT covered and cannot be by
a static check"*. So the tripwire does not depend on the code being present — it documents
its own inability to cover it. The warning is real but points the other way: the guard
cannot see this path either way.

This is a **WIRE-or-rule** candidate, not a DELETE candidate. Its zero-call reading is
explained by the missing client surface, not by disuse. Whether it may exist at all is the
non-goal-#4 question, which is the human's.

### F5 — C-28 is the cheapest finding in the sweep: everything but the client surface exists

`dialog.presence` has the hub arm (`router.rs:140`), the handler
(`channel.rs:1532`, `handle_dialog_presence`), capability advertisement (`router.rs:1051`),
and **unit tests** (`channel.rs:3784`, `:3873`, `:3884`). The producer side is live too:
`channel.rs:941` maintains a timestamp *"so dialog.presence can answer 'who's active
here?'"*.

So the hub is **continuously maintaining state for a query nobody can issue**. There is no
CLI verb, no MCP tool, and no script or doc that drives it. The expensive half is built and
tested; only the client surface is absent. If WIRE is the disposition, it is a small one.

### F6 — the `event.*` family does not share one fate, which C-32 suspected and is correct about

Three distinct classes, not one:
- **Live:** `event.subscribe`, `event.emit_to` (hub arms, client surfaces).
- **Daemon-served, hub-invisible:** `event.emit` (the hub is a *client* of it), `event.topics`.
  Their zero-call readings are F1 artefacts.
- **Orphan constants:** `event.state_change` and `event.error` have **zero references
  anywhere outside their own definition** in `control.rs` — no hub arm, no CLI, no MCP, no
  test, no script. Nothing can emit them and nothing can receive them.

The two orphans are the only surfaces in this entire sweep whose removal rests on no usage
argument at all: they are unreferenced by construction, so F1's inadmissibility does not
apply to them.

## Recommendation

**Recommendation:** GO — for a sweep whose main result is that **three of the five rows
were asking the wrong question**, and one row can be closed today.

**Rationale.** The sweep was scoped as "wired vs orphaned". The measurement says the rows
divide on a different axis:

1. **Two rows rest on inadmissible evidence.** C-31 and most of C-32 read "zero hub-observed
   calls" as weak evidence of disuse. F1 verifies it is evidence of nothing: the session
   daemon has no audit sink at all, and `event.poll`/`event.collect` are excluded from the
   audit by design. C-31 asked for exactly this verification; it comes back positive, so
   C-31's own disposition is *"do not decide on usage until T-2996 lands"*.
2. **One row's premise is already spent.** C-33 treats `event.broadcast` as a DELETE
   candidate awaiting a reference sweep. It was cut on **2026-05-31**. The sweep is clean,
   and what remains is a constant plus a `LEGACY_METHODS` warn entry for a method that
   returns `-32601`, held in place by a guard test that names it.
3. **One row's reading is refuted.** C-27 calls `orchestrator.route` shelved scaffolding.
   It has a 350-line handler, a dedicated `route_cache` module describing it as Layer 3 of
   a three-layer design, and a live E2E harness driving it. What it lacks is a client
   surface. The federation tripwire does not depend on it — `no_federation_tripwire.rs:43`
   names it as the path it *cannot* cover.
4. **One row is cheap and clean.** C-28's `dialog.presence` has handler, tests, capability
   advertisement and a live producer maintaining state for it. Only the client surface is
   missing.
5. **Two surfaces are genuinely orphaned.** `event.state_change` and `event.error` have zero
   references outside their own definition. These are the only DELETE candidates in the
   sweep that need no usage evidence, because nothing references them either way.

**Dependency-ordered scope, if GO:**
1. `event.state_change` + `event.error` — unreferenced constants, removable on structure
   alone. No dependency on T-2996.
2. `dialog.presence` — add the client surface (CLI verb and/or MCP tool). Small; producer,
   handler and tests already exist.
3. `event.broadcast` residue — retire the constant and the `LEGACY_METHODS` entry
   **together with** the `no_legacy_callers.rs` expectations that reference them, or leave
   all three alone. They are one unit.
4. `orchestrator.route` — **blocked on the human**. The measurement says WIRE is viable and
   DELETE is not supported by evidence; whether it may exist at all is the non-goal-#4
   ruling, which is not mine.
5. `session.*` and the daemon-served `event.*` — **blocked on T-2996**. No usage disposition
   may be taken before the telemetry exists.

**Do not start (5) before T-2996** — and note that (1) is genuinely independent of it, which
is what makes this sweep worth having run now rather than after.

**What this task did not do:** no RPC method was added, wired, removed or renamed. The
sweep was read-only throughout.

## Decision

Not mine to make. `owner: agent` covers the measurement; the C-27 ruling against non-goal #4
is reserved to the human, and this task records no decision of any kind.
