# T-2409 (framework-side): `inbox.queued` CLI deposit-path gap

## Problem Statement

The hub function `mirror_inbox_deposit_with()` (`crates/termlink-hub/src/channel.rs:166`)
fires the `inbox.queued` hub-aggregator event and is proven correct by unit +
integration tests. But when exercised live, two user-facing CLI flows did NOT
produce an observable `inbox.queued`:

1. `termlink file send` / `termlink remote send-file` to an offline target.
2. A `channel post` where a member session had been `kill -9`'d.

A framework subscriber long-polling `inbox.queued` (T-1636/T-1820, tracked
upstream as T-2409 in `/opt/999-Agentic-Engineering-Framework`) saw nothing
for these flows. This report traces every CLI deposit path, pins down which
ones reach the emit and which don't, reproduces a working path live, and
files the fix as a task in this repo.

## (a) Every CLI deposit path, traced

TermLink actually has **two independent `inbox.queued` emit sites**, not one:

- **Site 1 — legacy mirror.** `mirror_inbox_deposit_with()`
  (`channel.rs:166-218`), called only from `mirror_inbox_deposit()`
  (`channel.rs:150-161`), called only from `handle_event_emit_to()`
  (`crates/termlink-hub/src/router.rs:366-378`) when `event.emit_to`'s
  target session is not found and `crate::inbox::deposit()`
  (`crates/termlink-hub/src/inbox.rs:80`) succeeds. Added by commit
  `b901b6fe` (T-1636).
- **Site 2 — generic channel.post.** Inline in `handle_channel_post()`
  (`channel.rs:748-768`): any successful post to a topic literally prefixed
  `inbox:` fires the same aggregator inject, independent of Site 1 and
  independent of `mirror_inbox_deposit_with()` entirely. Added by commit
  `cb1fadb6` (T-1637), explicitly to make `channel.post` "the only inbox
  delivery RPC" post-T-1166. A sibling emit for `dm:<a>:<b>` topics
  (`dm.queued`, `channel.rs:769-805`, commit `d905c367`/T-2323) follows the
  same pattern.

Both sites only fire when the target topic name is literally `inbox:<id>`
(Site 1 always constructs this name; Site 2 checks `topic.strip_prefix("inbox:")`)
or `dm:<a>:<b>`. **Any other topic name gets no addressee-based emit at all** —
confirmed by grepping `crates/termlink-hub/src/` for a channel-membership
registry: there isn't one. `channel members <topic>` (`crates/termlink-cli/src/commands/channel.rs:3380`)
is a derived per-sender activity summary over posted envelopes, not a
membership/offline-delivery registry.

### Flow 1: `termlink file send` (local CLI, `crates/termlink-cli/src/commands/file.rs`)

`cmd_file_send()` (`file.rs:160`) tries two paths in order:

1. **T-1249 artifact path (primary, tried whenever a hub socket exists,
   file.rs:216-272):** `try_send_via_artifact()` (`file.rs:104`) →
   `send_artifact_via_client()` (`crates/termlink-session/src/artifact.rs:137`)
   → posts via the generic `channel.post` RPC to topic `inbox:<target>`
   (`artifact.rs:230,252-259`) regardless of whether the target is actually
   online. This reaches **Site 2**, not `mirror_inbox_deposit_with()`. On a
   hub with T-1637 (any current build), this correctly fires `inbox.queued`
   — reproduced live, see (b).
2. **Legacy 3-phase fallback (`file.rs:274-370`), only on `LegacyOnly`/error
   from step 1:** routes through `DeliveryRoute::emit()` (`file.rs:31-77`).
   For an offline target, `DeliveryRoute::Hub` is selected (`file.rs:166-186`)
   and correctly calls RPC `event.emit_to` (`file.rs:67`) — this reaches
   `handle_event_emit_to` → **Site 1** correctly.

**Conclusion for local `file send`: on current HEAD, both paths reach an
emit site.** Neither calls `mirror_inbox_deposit_with()` directly for the
common (artifact) case, but Site 2 covers it. A live-tested miss here points
at a **stale/pre-T-1636/T-1637 hub binary** in the test environment (a
recurring, separately-documented failure class in this repo — see
`CLAUDE.md` §"Fleet binary-freshness canary" / G-069 / G-070), not a current
code defect.

### Flow 2: `termlink remote send-file` (`crates/termlink-cli/src/commands/remote.rs`)

`cmd_remote_send_file_inner()` (`remote.rs:1602`) tries the same two paths
against a **remote** hub connection:

1. **T-1249 artifact path (`remote.rs:1647-1729`):** identical to Flow 1 —
   reaches Site 2 correctly when it succeeds.
2. **Legacy 3-phase fallback (`remote.rs:1731+`), on `LegacyOnly`/error:**
   `remote.rs:1750` calls `client.call("event.emit", ..., {target, topic,
   payload})` — **this is the wrong RPC method.** The hub's dispatch table
   (`crates/termlink-hub/src/router.rs:66-151`) has no case for `event.emit`
   (only `EVENT_EMIT_TO` at `router.rs:77`); unmatched methods fall through
   to the generic `forward_to_target()` (`router.rs:151,1599-1658`), which
   resolves `params.target` via `manager::find_session()` then the remote
   store and, if not found — **exactly the offline-target case this
   fallback exists for** — returns `SESSION_NOT_FOUND` (`router.rs:1628-1635`)
   without ever touching `inbox::deposit()`, `mirror_inbox_deposit()`, or
   any aggregator inject.

**Conclusion for `remote send-file`: this is a genuine, current, code-level
bug.** Whenever the T-1249 fast path is unavailable (older/capability-limited
remote hub — precisely what the legacy fallback is *for*) and the target is
genuinely offline, `remote send-file` errors out with "session not found"
and never deposits into the inbox at all, let alone emits `inbox.queued`.
This is very likely what the live test in T-2409 actually hit — the task
description groups "file send / remote send-file" together, and this is the
one of the two CLI commands whose legacy fallback is structurally incapable
of reaching either emit site.

### Flow 3: `channel post` to a killed member's channel

Any `termlink channel post <topic>` (`crates/termlink-cli/src/commands/channel.rs`)
against a topic that is **not** literally prefixed `inbox:` or `dm:` reaches
`handle_channel_post()` and posts successfully, but **no code path checks
whether any subscriber/"member" is offline, and no addressee event is ever
constructed** — because no channel-membership concept exists at the hub
layer (see above). This is not a bug in the emit logic; it's that this flow
never attempts to reach an emit site. Root cause, stated precisely: **the
hub's `channel.post` RPC has an addressee-aware wakeup emit only for two
specific topic-naming conventions (`inbox:<id>`, `dm:<a>:<b>`); a "channel"
with several members subscribed to an arbitrary topic name has no per-member
offline-delivery/wakeup mechanism at all.** If a producer wants a killed
member to be woken via `inbox.queued`, it must route that message through
`inbox:<member-id>` (or `dm:<a>:<b>` for `dm.queued`) explicitly — posting to
a shared topic name will never trigger it, regardless of hub version.

### MCP surface (adjacent, not asked, noted for completeness)

`termlink_file_send` (`crates/termlink-mcp/src/tools.rs:13418`) requires
`manager::find_session(&p.target)` to succeed **before** attempting anything
(`tools.rs:13423-13426`) — it has no offline/hub-spool fallback at all, so
MCP file-send cannot reach an offline target's inbox by any path. Filed as a
note on the follow-up task rather than a separate one; out of scope for this
CLI-focused trace.

## (b) A flow that DOES fire `inbox.queued`, reproduced live

`termlink channel post inbox:<target> --ensure-topic` reliably fires
`inbox.queued` on the hub aggregator stream (Site 2). Reproduced against the
host's live hub (PID confirmed via `termlink hub status`, governor JSON
carries T-2110-era `cv_index_*` fields, so a reasonably current binary):

```
$ termlink event watch --hub --topic inbox.queued --json --timeout 6 &
$ termlink channel post "inbox:t2409-scratch-target" --msg-type file.init \
    --payload '{"probe":"T-2409"}' --ensure-topic --json
{
  "delivered": { "offset": 0, "ts": 1783245316116 }
}
```

Captured on the watcher:

```
{"ok":true,"payload":{"addressee_session_id":"t2409-scratch-target","channel":"inbox:t2409-scratch-target","enqueued_at":1783245316116,"message_offset":0,"schema_version":"1.0"},"seq":0,"session":"hub","session_name":"hub","source":"hub-aggregator","timestamp":1783245316116,"topic":"inbox.queued"}
```

No scratch termlink sessions needed to be spawned for this repro (a plain
`channel post` to a nonexistent target name is sufficient and leaves no
process to clean up); the target topic `inbox:t2409-scratch-target` is
scratch data on the shared hub and safe to leave (bounded `Messages(1000)`
retention, per `mirror_inbox_deposit_with`'s `create_topic` call).

This confirms Site 2 (and by extension the whole `inbox:`-prefixed emit
contract) works correctly end-to-end on this host's current hub build. It
isolates the live-observed gap to the two structural issues in (a): the
`remote send-file` wrong-RPC bug (code-level, fixable), and the
channel-membership architectural gap (design constraint, not a bug).

## (c) Follow-up filed

**T-2363** — "Fix remote send-file legacy fallback uses wrong RPC (event.emit
not event.emit_to), skips inbox.queued" (`.tasks/active/T-2363-fix-remote-send-file-legacy-fallback-use.md`).
References framework T-2409. Scope: fix `crates/termlink-cli/src/commands/remote.rs:1750`
to call `event.emit_to` (matching `crates/termlink-cli/src/commands/file.rs:67`),
plus an integration test exercising `cmd_remote_send_file_inner`'s legacy
fallback against a genuinely offline target on a two-node hub harness,
asserting `inbox.queued` is observed via the hub aggregator. Notes the MCP
`termlink_file_send` gap and the channel-membership architectural gap as
related-but-separate follow-ups for whoever picks it up to triage.

## Summary Table

| Flow | Primary path reaches emit? | Legacy fallback reaches emit? | Verdict |
|---|---|---|---|
| `file send` (local) | Yes — Site 2 (`channel.post`→`inbox:<target>`) | Yes — Site 1 (`event.emit_to`, correct RPC) | OK on current HEAD; live miss ⇒ suspect stale hub binary |
| `remote send-file` | Yes — Site 2, same as above | **No** — calls wrong RPC (`event.emit` instead of `event.emit_to`), hits `forward_to_target`'s generic `SESSION_NOT_FOUND`, never reaches deposit/emit | **Bug — T-2363 filed** |
| `channel post` to a killed member's shared topic | N/A — no addressee concept for non-`inbox:`/`dm:` topics | N/A | Architectural: not a bug, a design boundary (route through `inbox:<id>`/`dm:<a>:<b>` explicitly) |
