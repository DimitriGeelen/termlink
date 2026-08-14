# T-1692 — MCP `channel_post` metadata exposure

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/completed/T-1692-mcp-channelpost-metadata-exposure--close.md`.
> The exploration and decision below happened on 2026-05-18; this file relocates
> that trail out of the archived task file into `docs/reports/` per C-001. No
> finding here is new.
>
> **Decision on record: GO** (2026-05-18T21:02:13Z).

## The gap

The TermLink wire protocol carries `envelope.metadata` — proven by `chat-arc:350`,
which has a populated `{thread, in_reply_to, task, mentions}`. The MCP tool
`termlink_channel_post` did **not** accept a `metadata` parameter, so any agent
posting via MCP emitted `envelope.metadata = {}`.

The consumer impact was named and concrete. Cohort-agent's n8n event-watch matcher
(`cohort_hub/n8n_event_match.py:42`, a different repo) keys on
`envelope.metadata.thread` plus `conversation_id`. Pen's acks landed with both
empty, so **n8n exec 7 was parked waiting for an ack that could never match**.

**The class behind the instance:** any MCP tool that wraps a protocol verb can
silently drop protocol-level fields. T-1692 fixed this instance; a sibling audit
task was scoped to catch the class.

## Assumptions

- **A1** — `envelope.metadata` is server-side a `Map<String, JsonValue>`
  *(verify via `crates/termlink-protocol/`)*.
- **A2** — adding `metadata: Option<serde_json::Value>` to `ChannelPostParams` is
  a pure additive MCP schema change; no protocol version bump needed.
- **A3** — T-1560 already defined `_thread` / `_project` as underscore-prefixed
  metadata keys for `termlink_agent_post`. The new parameter either subsumes that
  convention or coexists with it.

## Design shapes considered

Steel-manned from cohort-agent's letter, scored against the four Constitutional
Directives:

### Shape 1 — free-form metadata object · **recommended**

Caller passes `metadata: {thread, in_reply_to, conversation_id, …}` as arbitrary
JSON. The tool layer is pass-through; contracts define their own conventions.

Antifragility ✓ · Reliability ✓ · Usability ✓ · Portability ✓

*Risk:* contracts disagree on key names. *Mitigation:* per-contract convention
docs.

### Shape 2 — structured with reserved keys

The tool layer defines a fixed schema (thread, in_reply_to, mentions) and rejects
unknown keys.

Antifragility ◌ · Reliability ✓ · Usability ✗ · Portability ✗

### Shape 3 — per-contract typed MCP tools

`termlink_channel_post_reply(parent_offset, thread, …)` and siblings.

Reliability ✓ · Usability ◌ · Portability ✗ *(proliferation)*

## Decision — GO

Confirmed gap with named consumer impact (cohort-agent n8n exec 7 parked). A pure
additive MCP-schema change: no protocol version bump, no breaking change to
existing callers. Cohort-agent recommended **Shape 1** (free-form object) — it
matches existing protocol design, lets contracts evolve independently, and imposes
no schema lock-in at the tool layer. Implementation estimated under an hour.

The sibling audit task ("MCP-vs-protocol surface gap") is the class fix; this one
closes the named instance.

### Evidence

- Cohort-agent letter, operator-relayed via chat-arc — an explicit ask from a
  named consumer
- `chat-arc:350` — proves `envelope.metadata` is on-wire
- `chat-arc:351`, `chat-arc:383` — Pen acks with empty `envelope.metadata`: the
  gap itself
- `cohort_hub/n8n_event_match.py:42` (consumer-side, different repo) — the matcher
  keying on metadata fields
- T-1560 — `termlink_agent_post` already accepted underscore-prefixed metadata
  keys; this task generalises that
