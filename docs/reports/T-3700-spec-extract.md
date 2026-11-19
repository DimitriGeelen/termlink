# T-3700 spec extract — P-12 / B-2: consumer list per inbox, flag, bind (AL-30)

Prepared 2026-11-19. Paths relative to `docs/design/` unless stated.

## Spec sources

1. Plan row `08-plan.md:155`: P-12, B-2, R-49, R-60, E-66, E-133, E-169; exit "fixture suite exits 0 with a PASS line per
   listed E-n, mutant list all killed"; rollback "flag-only mode (DR-11 A before W4)".
2. Pseudo-code AL-30, `06a-pseudo-code-ph2-hardening.md:541-571` (2.14): record every subscribe/poll/read/ack/claim of an
   inbox or conversation topic per identity (first, last; 16 kept, oldest evicted with an admin-log row); RPC
   `inbox.consumers(topic, window)`; `admin inbox bind <topic> <identity>` (root-only socket, admin log first, durable);
   sidecar tick: other identity within 24 h → status `inbox consumed by two identities: A, B` + attention coalesced on
   (topic, identity, day); unreadable list → `consumers: unknown` (FCW-125). After a bind every read, ack, claim by
   another identity is refused and logged (FCW-124). A read-only consumer is a consumer (FCW-209, CR-6.33). Table and
   bind written under the per-topic lock in one step. 2.14.h: "the identity is the authenticated caller of the RPC [B]".
3. Architecture `04-architecture.md:495-504` (6.6.1-6.6.2), DR-11 A (`:1413`, flag during W1..W3, refuse after W4 bind),
   V-11 (24 h window, `:949`), V-12 (second live copy on the same hub within one tick, `:950`).
4. Evidence `05-evidence.md:723-729` E-66 (L4; flag within 60 s, refuse after bind, eviction logged, second copy within
   one tick; controls: subscriber-only and ack/claim-only mutants miss `fx-z`), E-133 (`:1310`, continuous check), E-169
   (`:960`, B-2 part only: unreadable list → `consumers: unknown`, never "one"; control: mutant defaulting to empty).

## Finding F-1: the "[B]" premise of 2.14.h does not hold for reads

`[B]` marks "a fact about what is built" (`04-architecture.md:11`). Checked in the code, 2026-11-19:

1. `channel.subscribe` (the only read/poll RPC) takes `topic, cursor, limit, timeout_ms, conversation_id, ...` and no
   caller identity (`crates/termlink-hub/src/channel.rs:1491-1530`). The router passes no caller to channel handlers
   (`router.rs:59`, `route(req, peer_addr)`).
2. Local-socket callers are trusted by OS uid; on .107 every sidecar, CLI and MCP reader is root (TB-1 separates nobody
   until PH-4 accounts, `04-architecture.md:1142`).
3. `channel.claim` carries a self-declared `claimer` string (`channel.rs:2179`).
4. Only posts (acks, stage rows, messages) are authenticated: signed, `sender_id` derived from the key (B-6 key cards).
5. The only per-connection principal is the T-3760 registration claim, used for `session.*` registration only
   (`router.rs:1505-1545`).

So AL-30 can key acks on an authenticated identity, but reads (the BP-15 shape that CR-6.33 added) and claims only on a
caller-declared name. How to identify readers is an operator decision (brief in the T-3700 `## Decisions` once ruled).

## Dialogue Log

1. 2026-11-19 — Brief on F-1 offered A (named reads), B (signed reads), C (postpone), recommending A.
2. Operator: "A still doesn't solve a problem that everybody can be, so I don't see the A plus B here as an option."
   Correction: the brief had left out A+B. Added as E (name every read; sign where a key card exists; `proven` /
   `claimed`; bind holds by key). A rescored down on Reliability (cannot tell a same-name second copy apart, V-12).
   Totals A +28, B +17, C −27, E +39; recommendation revised to E.
3. Operator ruling: **E**. Recorded in T-3700 `## Decisions`. Open: rule for operator diagnostic peeks.
4. Brief on key-less diagnostic reads offered A (no exception), B (self-declared observer), C (local root = observer),
   D (peek through the agent's own sidecar, signed). Recommended D (+32). Operator ruling: **D**.
