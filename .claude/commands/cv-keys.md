# /cv-keys — inspect hub-side cv_index for a topic (T-2121 skill-layer wrap)

Wraps `termlink channel cv-keys` (substrate primitive #9 BROADCAST-WITH-REPLAY
inspection, shipped under T-2106). Answers **"which cv_keys are currently
advertising on this topic, and at what offsets?"** — the diagnostic verb
operators reach for when `/governor` flags `cv_overflow > 0`.

Read-only, no state mutation, no auth side-effects. Probes the local hub
by default via `channel.cv_keys` JSON-RPC.

## Why this skill exists

When `/governor` (or `--only-pressured` / `--watch --notify` / `page-on-cv-overflow.sh`)
fires on `cv_index_overflow_total > 0`, the operator's next question is:

> "WHICH topic is saturating the cv_index cap, and WHICH producer is
> mis-emitting `cv_key`?"

`/cv-keys` answers it. Before this skill the operator had to either grep
`CLAUDE.md` for the verb or `termlink channel --help | grep cv` to find it.
This skill closes the discoverability gap and forms the natural follow-up
in the `/governor → /cv-keys → fix producer` investigation chain.

`/cv-keys` is the **substrate #9 BROADCAST-READ inspection** companion to
the four substrate-read daily verbs:

- **/find-idle** (T-2092) — DISPATCH (substrate #2)
- **/claims** (T-2093) — CLAIM (substrate #1)
- **/queue-status** (T-2094) — RESILIENCE (substrate #5)
- **/governor** (T-2095) — BACKPRESSURE (substrate #10)
- **/cv-keys** (this skill) — BROADCAST-WITH-REPLAY (substrate #9 inspection)

**Invocation:**

| Form | Action |
|------|--------|
| `/cv-keys <topic>` | List cv_keys + offsets for `<topic>` on the local hub |
| `/cv-keys <topic> --hub <addr>` | Probe a non-default hub |
| `/cv-keys <topic> --json` | Machine-readable envelope (passthrough to verb) |

**What the verb reports** (per T-2106 schema):

- `count` — number of distinct `(topic, cv_key)` entries currently in the hub's cv_index for `<topic>`
- `entries[]` — per-cv_key rows: `{cv_key: "<string>", offset: <u64>}`

Each entry is the LAST-WRITE-WINS latest offset that producer posted with
that cv_key. Read-side consumers receive this snapshot inline via
`channel subscribe --include-current-value` (T-2105) for O(K) replay
instead of O(N_envelopes) walk.

## Step 1: Pre-flight

Run:

```
termlink channel cv-keys --help >/dev/null 2>&1
```

If exit non-zero: **stop**. Print:

```
cv-keys: `termlink` CLI not on PATH or substrate primitive #9
(BROADCAST-WITH-REPLAY inspection, T-2106) not available in this build.
Run `termlink --version` and upgrade if needed.
```

## Step 2: Parse arguments

`$ARGUMENTS` is the operator's tail. The first positional is the topic
name (required). Remaining flags pass through verbatim — the underlying
verb validates and errors with usage on malformed input.

If the operator passes no topic: **stop**. Print:

```
cv-keys: topic required.
Usage: /cv-keys <topic> [--hub <addr>] [--json]

Common topics to inspect:
  agent-presence     # listener-heartbeat.sh cv_index (one entry per LIVE agent)
  <your-topic>       # any topic whose producers wire metadata.cv_key=...
```

Examples:

| Operator typed | Command emitted |
|------|------|
| `/cv-keys agent-presence` | `termlink channel cv-keys agent-presence` |
| `/cv-keys work-queue --json` | `termlink channel cv-keys work-queue --json` |
| `/cv-keys foo --hub 192.168.10.107:9100` | passthrough |

## Step 3: Run the verb

Execute the constructed command via Bash. Capture stdout + stderr + exit code.

## Step 4: Render the result

For **default human-format** output:

- Pass the verb's stdout through verbatim. The verb already prints:
  - `topic=<X> count=<N>` header
  - One row per cv_key: `  <cv_key> -> @<offset>`

For `--json` mode: pass the JSON envelope through verbatim. Do not
re-render — callers piping/parsing rely on the substrate verb's schema:

```json
{
  "count": <N>,
  "entries": [
    {"cv_key": "<string>", "offset": <u64>},
    ...
  ]
}
```

## Step 5: Empty-result hint (loud-not-silent)

For the empty-cv_index case, the verb prints:

```
no cv_keys recorded on topic "<X>"
```

After passing that through, append diagnostic context:

```
This means no producer on this hub has posted to `<X>` with
`metadata.cv_key=...` annotation. Two common causes:

1. The topic genuinely has no cv-indexed producers (broadcast-only,
   conversation-style topic). This is healthy — read-side consumers
   replay the full envelope log instead of the O(K) cv-key snapshot.

2. The producer SHOULD be cv-indexing but isn't wired correctly. See
   `docs/operations/substrate-broadcast-with-replay.md` for the
   producer-side recipe (`channel post --metadata cv_key=<stable-id>`;
   `listener-heartbeat.sh` does this automatically as `--metadata cv_key=$agent_id`, T-2107).

If you reached this skill because `/governor` flagged
`cv_overflow > 0`: the saturating topic is likely a DIFFERENT topic
than this one. Run `/governor --json | jq '.hubs[] | {hub, cv_overflow_hits_total}'`
to identify the hub, then walk that hub's topic list:
`termlink channel list --json | jq '.[].name'` and re-run `/cv-keys`
on each candidate.
```

Never silent on empty.

## Step 6: Reference the broadcast-with-replay arc

For human-format mode (NOT json), after the verb's output, append:

```
Related forms / next step:
- termlink channel subscribe <topic> --include-current-value
                                     # late-joiner O(K) replay (T-2105)
- termlink fleet governor-status --only-pressured
                                     # is any hub's cv_index overflowing? (T-2118)
- termlink fleet governor-status --watch 30 --notify <cmd>
                                     # page when a producer mis-emits cv_key (T-2119)
- docs/operations/substrate-broadcast-with-replay.md
                                     # producer-side cv_key wiring + design rationale
```

Skip this section in `--json` mode (machine output stays pure).

## Rules

- **Read-only by contract.** Never post, never modify cv_index state,
  never tune the per-topic cap.
- **Pure Observe-scope reads** — no auth side-effects on the hub.
- **Local-hub-default.** Unlike fleet-wide verbs (`/governor`,
  `/peers --all`), `/cv-keys` probes the local hub unless `--hub` is
  passed. cv_index is per-hub state (no federation primitive — see
  G-060) so a fleet-wide form would mislead operators into thinking
  the keys aggregate.
- **No `AskUserQuestion`** — just run and report.
- **Pair with `/governor`** for the full investigation flow. `/governor`
  detects overflow, `/cv-keys` identifies which topic, then the operator
  fixes the producer.

## Common patterns

**Inspect the canonical agent-presence cv_index:**

```
/cv-keys agent-presence              # one entry per LIVE agent (T-2107 wiring)
```

This is the highest-value default — every `/be-reachable` heartbeat
populates this index, so `count` = LIVE agent count and the entries
are the agent IDs.

**Investigate a `cv_overflow > 0` alert from /governor:**

```
/governor --only-pressured           # which hub is overflowing?
/cv-keys <suspect-topic> --hub <addr>  # which keys are on the saturating topic?
```

If `count` is at or near the per-topic cap (default 1000, settable via
`TERMLINK_CV_INDEX_CAP_PER_TOPIC`): the producer is mis-emitting
cv_key (probably a timestamp or session id instead of a stable id).
Find the producer in the topic's recent envelopes and fix the
annotation.

**Script-friendly inspection:**

```
/cv-keys agent-presence --json | jq '.count'
/cv-keys agent-presence --json | jq -r '.entries[].cv_key'
```

## Related

- T-2018 — arc-parallel-substrate ADR; this skill operationalizes #9
  inspection at the daily-verb tier.
- T-2027 — substrate primitive #9 BROADCAST-WITH-REPLAY (cv_index +
  late-joiner replay).
- T-2089 — cv_index design + hub-side implementation.
- T-2103 — hub records `(topic, cv_key) → latest_offset` on every post.
- T-2104 — `channel.subscribe` accepts `include_current_value: bool`.
- T-2105 — CLI + MCP wire-up of `--include-current-value`.
- T-2106 — `channel.cv_keys` / `termlink channel cv-keys` /
  `termlink_channel_cv_keys` MCP read-only inspection verb (the
  underlying CLI command this skill wraps).
- T-2107 — `listener-heartbeat.sh` wires `metadata.cv_key=$agent_id`
  (the highest-value producer of cv_index entries).
- T-2110 — cv_index telemetry surfaced via `hub.governor_status`
  (`cv_index_entries_active` / `_topics_active` / `_overflow_total` /
  `_cap_per_topic`).
- T-2118 — `--only-pressured` predicate fires on `cv_index_overflow_total > 0`.
  Pre-T-2118, cv_overflow wouldn't surface in fleet rollups.
- T-2119 — watch/notify/log/history carry cv_overflow deltas. The
  `page-on-cv-overflow.sh` recipe in
  `docs/operations/substrate-governor.md` is the automated counterpart
  to `/cv-keys` (event hook → operator runs `/cv-keys`).
- T-2120 — operator-facing documentation for the cv_overflow signal
  across substrate-governor.md + `/governor` + `/substrate`.
- T-2092 / `/find-idle` — sibling daily-verb skill (substrate #2 read).
- T-2093 / `/claims` — sibling daily-verb skill (substrate #1 read).
- T-2094 / `/queue-status` — sibling daily-verb skill (substrate #5 read).
- T-2095 / `/governor` — sibling daily-verb skill (substrate #10 read). The
  detect-side companion to this verb's diagnose-side.
- `docs/operations/substrate-broadcast-with-replay.md` — substrate #9
  producer/consumer wiring master recipe.
- `docs/operations/substrate-governor.md` — substrate #10 master recipe
  including `page-on-cv-overflow.sh` template.
- G-060 — channel topics are PER-HUB state. cv_index is no exception —
  hence local-hub-default behavior.
- PL-187 — verb-stack pattern rung 6 (ephemeral session skills).
