# agent-presence retention reset (T-2059)

Operator runbook to shrink `agent-presence` (or any high-rate topic) and
optionally change its retention policy from `Retention::Forever` to
`Retention::Messages(N)`. Filed as the operator-facing follow-through of
the T-2057 retention audit, which surfaced 13,443 envelopes on a single
topic as the largest in-the-wild T-1991 reproduction surface.

## §1 When to use

Run this when EITHER:

- `termlink channel list --json | jq '.topics[] | select(.name=="agent-presence")'`
  shows `count` in the thousands AND `retention.kind == "forever"`.
- `fw doctor` or `fleet doctor` surfaces topic-growth pressure.
- You're following up on a T-2058 warn-log: a `tracing::warn!` fired at
  create time naming a high-rate pattern with `Forever`.

Do NOT run this on operator-named durable record topics — these are
intentionally `Forever` and shrinking them loses real history. The
audit's §5 explicitly excluded `channel:learnings`, `policy-decisions`,
`framework:pickup`, and `broadcast:global` from the high-rate-pattern
list for this reason.

## §2 Understanding the gap

TermLink has **no `channel.set-retention` RPC** (T-2057 audit §3). Once
a topic is created with a retention policy, you have two ways to change
its effective footprint:

| Goal | Method |
|---|---|
| Shrink history NOW, future growth unchanged | `channel.trim` (destructive but online) |
| Change the retention policy itself | `sqlite3 UPDATE topics ...` (direct metadata edit) |
| Shrink AND change policy (full T-1991 fix) | Both, in sequence |

Both are destructive and have no undo. Subscriber cursors that pointed
into the trimmed range will skip-forward to the first surviving offset
on their next `channel.subscribe` (per `Bus::sweep` semantics in
`crates/termlink-bus/src/lib.rs:225`). For agent-presence this is
typically fine — presence is a low-water-mark stream where stale
heartbeats are valueless.

## §3 Option A — trim only (history shrunk, policy stays Forever)

The least invasive path: keep the topic on `Forever` retention but
delete most of the accumulated history.

```sh
# Pick the topic and a keep-recent count (e.g. 200 envelopes ≈ 100 min
# of heartbeats at 30s × 5 agents — match this to your real fleet).
TOPIC=agent-presence
KEEP=200

# Read the current offset (next_offset is the high-water mark).
CURRENT=$(termlink channel list --json \
  | jq -r ".topics[] | select(.name==\"$TOPIC\") | .count")
echo "current count: $CURRENT"

# Compute the trim cutoff. before_offset deletes records with offset < N.
BEFORE=$((CURRENT - KEEP))
if [ "$BEFORE" -lt 1 ]; then
  echo "already at or below KEEP — nothing to do"
  exit 0
fi
echo "trimming records with offset < $BEFORE (keeping ~$KEEP most recent)"

# Run it via the hub's RPC. Affects ALL subscribers.
# (No operator CLI verb for channel.trim — use the local socket directly.)
SOCK=$(termlink hub status --json | jq -r .socket)
printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"channel.trim","params":{"topic":"'"$TOPIC"'","before_offset":'"$BEFORE"'}}' \
  | socat - "UNIX-CONNECT:$SOCK"
```

Topic remains on `Forever`, so it will re-grow. Re-run periodically OR
use Option B to also change the policy.

## §4 Option B — change retention + trim (T-1991 fix proper)

Closes the operator-default vector for one topic. Combines a sqlite3
metadata edit (changes the policy) with a `channel.trim` (shrinks the
existing history).

**Caveat:** sqlite3 metadata edits bypass the running hub's invariants.
Stop the hub OR accept the risk that an in-flight write races the edit.
For agent-presence (heartbeat traffic, no critical writes), the race
window is benign — at worst one heartbeat is appended under the old
policy before the new policy applies. For any topic carrying ledger /
governance traffic, stop the hub first.

```sh
TOPIC=agent-presence
KEEP=200
# Locate the hub's runtime_dir. Defaults — adjust if TERMLINK_RUNTIME_DIR is set.
DB="${TERMLINK_RUNTIME_DIR:-/var/lib/termlink}/meta.sqlite"
test -f "$DB" || { echo "meta.sqlite not at $DB — check TERMLINK_RUNTIME_DIR"; exit 1; }

# 1. Inspect the current row.
sqlite3 "$DB" \
  "SELECT name, retention_kind, retention_value FROM topics WHERE name='$TOPIC';"

# 2. Switch the policy. Messages(N) keeps the most recent N records on
#    every sweep; the hub's existing compaction path will respect it.
sqlite3 "$DB" \
  "UPDATE topics SET retention_kind='messages', retention_value=$KEEP WHERE name='$TOPIC';"

# 3. Verify.
sqlite3 "$DB" \
  "SELECT name, retention_kind, retention_value FROM topics WHERE name='$TOPIC';"

# 4. Now shrink the existing history. Same logic as Option A.
CURRENT=$(termlink channel list --json \
  | jq -r ".topics[] | select(.name==\"$TOPIC\") | .count")
BEFORE=$((CURRENT - KEEP))
if [ "$BEFORE" -ge 1 ]; then
  SOCK=$(termlink hub status --json | jq -r .socket)
  printf '%s\n' '{"jsonrpc":"2.0","id":1,"method":"channel.trim","params":{"topic":"'"$TOPIC"'","before_offset":'"$BEFORE"'}}' \
    | socat - "UNIX-CONNECT:$SOCK"
fi
```

From this point, the topic auto-respects `Messages(N)`. Future heartbeats
trigger no re-growth past the cap.

## §5 Verification

After EITHER option, confirm the new state:

```sh
termlink channel list --json \
  | jq '.topics[] | select(.name=="agent-presence")'
```

Expected after Option A: `count` near `KEEP`, `retention.kind == "forever"`.
Expected after Option B: `count` near `KEEP`, `retention.kind == "messages"`, `retention.value == KEEP`.

If `count` did not drop: the `channel.trim` RPC returned a non-zero
`deleted` value but the visible count is cached. Re-run the same query
after the next post to the topic; `channel.list` reads live from the
metadata table.

If the new `retention.kind` did not stick (Option B): the sqlite3 UPDATE
hit a different database. Double-check `$TERMLINK_RUNTIME_DIR` and
re-locate `meta.sqlite`.

## §6 Related

- T-2057 — Track A retention audit (this runbook's parent).
- T-2058 — high-rate-pattern warn-log at create time (structural prevention).
- T-1991 — original silent-growth incident on `agent-presence`.
- G-058 — the gap this runbook closes for one specific topic.
- `docs/architecture/parallel-execution-substrate.md` §6 #10 — the
  budget/retention primitive framing (T-2028 inception).
- `docs/operations/substrate-governor.md` — operator surface for related
  Track B telemetry.
- Future task: `channel.set-retention` RPC would obsolete §4's sqlite3
  edit. Not filed — file when there's evidence of multi-topic operator
  pain (single-topic is small enough to live with the workaround).
