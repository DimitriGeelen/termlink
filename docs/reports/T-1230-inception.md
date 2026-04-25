# T-1230 Inception: `inbox.clear` semantic split (Q4 spool deletion)

**Status:** GO
**Decision date:** 2026-04-25
**Workflow:** inception → build (sub-tasks to be created on GO)
**Related:** T-1163 (channel mirror), T-1166 (retirement), T-1220 (parent receiver migration), T-1229 (sibling status migration)

## Problem

After T-1226/27/28/32 the `inbox.list` receiver migration is complete.
The parallel migration for `inbox.clear` raises a fundamental semantic
question: legacy and channel "clear" are NOT the same operation.

- **Legacy `inbox.clear`** (`crates/termlink-hub/src/inbox.rs:449-493`):
  `std::fs::remove_dir_all(&target_dir)` on the hub spool. Affects ALL
  subscribers — the data is gone, not just hidden from one client.
- **Channel-backed clear**: each subscriber owns its own cursor.
  "Clearing" means advancing my cursor past the latest offset; the data
  remains on the hub for other subscribers and for retention to GC.

These are different operations. Conflating them under one verb risks
operator surprise. T-1166 will retire `inbox.clear` so a replacement is
required — keep-legacy is not viable.

Four legacy call sites depend on the destructive semantic (~80 LOC):

- `cmd_inbox_clear` — `crates/termlink-cli/src/commands/infrastructure.rs:802`
- `termlink_inbox_clear` MCP — `crates/termlink-mcp/src/tools.rs:4537`
- `cmd_remote_inbox_inner` Clear arm — `crates/termlink-cli/src/commands/remote.rs:1319`
- `termlink_remote_inbox_clear` MCP — `crates/termlink-mcp/src/tools.rs:~4754`

## Recommendation

**Option A — Two distinct verbs, no semantic conflation.**

1. **`channel.trim(topic, before_offset?)`** — destructive hub-side
   delete. Mirrors legacy `inbox.clear` semantics 1:1. Used by the four
   migration call sites.
2. **`channel.cursor.advance(topic, offset)`** — per-subscriber cursor
   advance. New semantic for new use cases (multi-subscriber broadcast
   topics). No current call site needs this; introduce it lazily when
   the first consumer appears, OR add it now as part of the channel
   surface for completeness.

The legacy migration uses `channel.trim` exclusively. Operator behavior
is preserved: `termlink inbox clear <target>` keeps deleting from hub.

## Go / No-Go

**GO.** Option A resolves all four design questions (Q1-Q4) without
conflict and preserves operator expectations. Sub-tasks (one per call
site, plus hub method) to be created post-decision.

## Q1 — Verb split: A vs B vs C

### Option A: Two verbs, clear distinction — **CHOSEN**

**Mechanism.** Add `channel.trim(topic, before_offset)` to the hub
router as a destructive operation that calls
`crate::inbox::clear_target` / `clear_all` (or a renamed
`channel::delete_topic` once channel storage decouples from inbox spool).
Add `channel.cursor.advance(topic, offset)` for the subscriber-local
semantic — initially no consumers, but lives alongside `channel.subscribe`
in the channel surface for future broadcast use cases.

**Why two verbs are needed.** The two operations have different
authority models:
- `channel.trim` is operator-level: "delete this data from the hub for
  everyone." Requires destructive permission.
- `channel.cursor.advance` is subscriber-level: "I'm caught up, advance
  my read position." No special permission; idempotent per subscriber.

Conflating them forces every call site to make a choice without
operator intent visible in the verb. The verb split makes intent
unambiguous.

**Pros.**
- Legacy migration is mechanical: `inbox.clear` → `channel.trim`. Same
  semantics, new verb. No operator surprise.
- Future-proof: `channel.cursor.advance` exists for new broadcast use
  cases without retrofitting later.
- Clean alignment with T-1166: legacy `inbox.clear` retires, replaced
  by `channel.trim`.
- Multi-subscriber semantics resolved by verb choice (see Q4).

**Cons.**
- Two new server methods vs one. Modest surface addition; offset by
  retiring `inbox.clear`.
- Documentation needs to explain the difference clearly to avoid
  operators reaching for the wrong verb. Mitigation: CLI command
  remains `termlink inbox clear <target>` (or eventually
  `termlink channel trim <topic>`) — operators who type "clear" already
  expect destructive semantics.

### Option B: Single hub-side `channel.trim` only — **REJECTED (incomplete)**

**Mechanism.** Add only `channel.trim`; per-subscriber semantic is
implicit (each subscriber's cursor lives in its own store; "clearing"
client-side just means advancing local cursor without an RPC).

**Why rejected.** Adequate for the legacy migration but punts the
multi-subscriber design question (Q4). When the first new consumer
asks "how do I mark these messages as read without affecting other
subscribers?", we re-open this inception. Better to design both verbs
together once.

### Option C: Single verb meaning cursor-advance only, with CLI warning — **REJECTED**

**Mechanism.** `clear` always means subscriber-local cursor advance.
Spool delete becomes a separate operator command
(`termlink hub gc-spool`). Existing scripts that called `inbox.clear`
expecting deletion would silently behave differently.

**Why rejected.** Backwards-incompatible without a clear failure mode.
A script that ran `termlink inbox clear orphaned-target` to free disk
would now silently no-op (cursor advance is per-subscriber, no spool
mutation). The disk fills up; nobody notices until G-009 fires. Hard
fail.

## Q2 — Backwards compatibility

Under Option A: no deprecation cycle is required for *semantics*. The
verb name changes (`inbox.clear` → `channel.trim`) but the operation is
identical. Migration steps:

1. Add `channel.trim` to hub (additive, no breaking impact).
2. Migrate 4 call sites to call `channel.trim` instead of `inbox.clear`,
   following the T-1226-T-1232 wedge pattern.
3. After 60-day parallel-operation gate (T-1166's standard), retire
   legacy `inbox.clear`.

Existing scripts that shell out to `termlink inbox clear` keep working
through the transition — the CLI command stays; only the underlying RPC
changes.

If we choose to *also* rename the CLI command (`inbox clear` →
`channel trim`), that is a separate UX decision with its own
deprecation cycle (advertise the new verb, warn on the old, remove
after grace period). Recommend keeping the CLI verb stable for now and
revisiting after T-1166 ships.

## Q3 — T-1166 alignment

T-1166 explicitly retires `inbox.clear` (see
`crates/termlink-hub/src/router.rs:75,756` and T-1166 §"Router methods
removed"). A replacement is mandatory.

**Implication for build sub-tasks:**

1. The new `channel.trim` verb must be in production for at least 60
   days before T-1166 can retire `inbox.clear` (parallel-operation gate).
2. The hub method can be added independently of the call site
   migrations.
3. Migration order mirrors T-1229: hub method first, then helper in
   `inbox_channel.rs` (or new `channel_admin.rs`), then per-site
   migrations.

`channel.cursor.advance` is independent of T-1166's retirement scope —
it is a new addition with no legacy counterpart. Can ship on any
schedule.

## Q4 — Multi-subscriber risk

**The scenario.** Two clients subscribe to `inbox:<target>` (e.g., CLI
+ Watchtower learning panel). One calls "clear". Should the other
still see the messages?

**Resolution under Option A:** The verb makes the answer explicit.

- `channel.trim(topic="inbox:<target>")` — both subscribers lose access.
  Hub spool is wiped. Operator chose destructive intent.
- `channel.cursor.advance(topic="inbox:<target>", offset=N)` — only the
  calling subscriber's cursor advances. Other subscribers are
  unaffected. Operator chose subscriber-local intent.

**Resolution under Options B or C:** Either ambiguous (B — operator
must know they're calling the destructive verb without it being named
that way) or surprising (C — operator expects deletion, gets cursor
advance).

**Recommendation for legacy migration:** Use `channel.trim` for all 4
call sites. The legacy `inbox.clear` semantic was always destructive,
operators expect it to be destructive, and `inbox:` topics today are
single-consumer (the offline target session). When/if multiple
subscribers attach to `inbox:` topics, the next consumer can choose
`cursor.advance` if non-destructive intent is required.

**Documentation requirement.** The first time a *new* (non-`inbox:`)
broadcast topic gets a multi-subscriber CLI command, the docs MUST
warn against using `channel.trim` and steer toward `cursor.advance`.
Recommend adding this warning to the channel surface design doc when
sub-task T-1230a (hub method) ships.

## Build Sub-Tasks (post-GO)

Following the T-1226-T-1232 wedge pattern (one task = one deliverable):

| ID | Scope | LOC est |
|----|-------|---------|
| T-1230a | Hub: add `channel.trim(topic, before_offset?)` router method delegating to existing `inbox::clear_*` helpers; add tests covering single-target + all-targets paths | ~80 |
| T-1230b | Hub: add `channel.cursor.advance(topic, offset)` router method (no-op for inbox: topics today; in-memory cursor store for new topics) — OPTIONAL, can defer until first consumer appears | ~60 |
| T-1230c | termlink-session: add `clear_with_fallback{,_with_client}` helper in `inbox_channel.rs` (parity with T-1231 helper) — calls `channel.trim` first, falls back to `inbox.clear` on `-32601` | ~110 |
| T-1230d | CLI local: migrate `cmd_inbox_clear` (infrastructure.rs:802) | ~25 |
| T-1230e | MCP local: migrate `termlink_inbox_clear` (tools.rs:4537) | ~30 |
| T-1230f | CLI remote: migrate `cmd_remote_inbox_inner` Clear arm (remote.rs:1319) | ~25 |
| T-1230g | MCP remote: migrate `termlink_remote_inbox_clear` (tools.rs:~4754) | ~30 |

Total: ~360 LOC across 7 sub-tasks (or 6 if T-1230b is deferred).
T-1230b can ship lazily — recommend deferring until a real consumer
needs the per-subscriber semantic, with a placeholder task captured.

## Dialogue Log

This inception was conducted as a sequential analysis pass alongside
T-1229. No live human dialogue — design questions Q1-Q4 were posed by
the inception task itself and answered by reading T-1166's retirement
scope, the current hub `inbox::clear_target/clear_all` implementation
(inbox.rs:449-493), and the four legacy call sites.

The CHOSEN option (A: two verbs) was confirmed against all four Q's:
- Q1 (verb split): Option A by design.
- Q2 (backwards compat): No semantic break — verb rename only.
- Q3 (T-1166): Aligned — legacy retires, `channel.trim` replaces.
- Q4 (multi-subscriber): Resolved by verb choice; operator picks intent.

If the human disagrees with Option A and prefers a single verb (Option
B), the build scope shrinks to ~280 LOC but the multi-subscriber design
question reopens at the next consumer. Recommend Option A.
