# /agent-handoff - Cross-Host Agent Contact (T-1429 wrapper)

Wraps `termlink agent contact` so vendored Claude Code agents can hand off
context to a peer agent in one command. Replaces the legacy `remote push`
+ inbox.push improv pattern (T-1166 retired).

**Invocation:** `/agent-handoff <target> <task-id> "<message>"`

- `<target>` — the peer agent's display_name, resolvable via local
  `termlink session.discover` (e.g., `framework-agent`, `oss-dash`).
- `<task-id>` — the active task this handoff belongs to (e.g., `T-1429`).
  Must exist under `.tasks/active/`. Embedded as a `[T-XXX]` prefix in the
  message body so the receiver can route by thread.
- `<message>` — the handoff summary in plain text. Quote it.

Protocol canon is documented on the topic itself — run
`termlink channel info agent-chat-arc` to see the 5 invariants in-place
(T-1430).

## Step 1: Verify the task exists

Run:

```
ls .tasks/active/<task-id>-*.md 2>/dev/null
```

If no match, **stop**. Print:

```
agent-handoff: task <task-id> not found in .tasks/active/.
Create it first: fw work-on "topic" --type build
```

Exit non-zero. Do NOT post to a non-existent task.

## Step 2: Capture self identity (for log visibility)

The wire-level envelope `sender_id` (what DM topics are keyed on and
what handoff logs should reference) is NOT exposed by `whoami --json` —
its `candidates[].sender_id` is structurally `null` and on shared
hosts the response is `{ambiguous: true, candidates: [N]}`. Read the
sender_id from the local hub's view of any topic this host has signed.

**Primary path** (O(1), works for log visibility):

```
termlink channel info agent-presence --json | jq -r '.senders[0].sender_id // empty'
```

**Fallback** if `agent-presence` has no posts on the local hub:

```
termlink channel info agent-chat-arc --json | jq -r '.senders[] | select(.posts > 0) | .sender_id' | head -1
```

If both return empty: continue with `sender_id=unknown` and note in
the Update entry. Identity binding is enforced server-side once T-1427
ships — this skill's self-fp capture is for log audit only, not
routing, so an `unknown` self-fp does NOT block the handoff.

**Shared-host semantics (PL-195, T-1874 predecessor).** On a shared
host (multiple claude sessions co-resident, same termlink install)
every session signs with the same host-level identity key, so the
resolved `sender_id` is the HOST's fingerprint, shared across every
agent on this host until T-1693 (per-agent identity keys) ships.
Sufficient for handoff audit; ambiguous if you need to attribute a
handoff to a specific agent vs another agent on the same host.

## Step 3: Post to the peer

Run:

```
termlink agent contact "<target>" --message "[<task-id>] <message>" --json
```

Capture stdout. The actual JSON shape (T-1429 Phase-1, verified live
2026-05-01):

```json
{
  "delivered": { "offset": <integer>, "ts": <ms-since-epoch> }
}
```

If exit code is non-zero or `delivered` is missing: **stop**. Print the
verb's stderr and exit non-zero. The verb already prints actionable
errors (peer offline, missing identity_fingerprint, etc.) — do not
swallow them.

### Step 3.5: Fallback when peer lacks identity_fingerprint (T-1644)

If `agent contact` exits with **exit code 8** and stderr contains "no
identity_fingerprint in metadata — likely registered before T-1436",
the peer's session was created before T-1436 shipped the metadata field
and DM resolution is impossible. The verb's error message names three
recovery paths; pick option (3) for this skill since it works without
restarting the peer and without knowing the peer's fingerprint:

```
termlink channel post agent-chat-arc \
  --msg-type proposal \
  --metadata _thread=<task-id> \
  --mention <target> \
  --payload "[<task-id>] <message>"
```

The receiver picks the post up via mention-routing on the agent-chat-arc
broadcast topic (T-1430 protocol canon). The `_thread=<task-id>`
metadata threads it the same way `--thread` would on the dm path, so
downstream tooling (`agent on-thread`, `agent recent --thread`) groups
it correctly.

For large structured payloads (T-1646), use `--payload "$(cat
/tmp/handoff.txt)"` or pipe via stdin (`channel post` accepts both).
The body should still begin with `[<task-id>]` for portability — agents
on older binaries route by body prefix.

When option (3) is used, the dm topic does not exist; the next session
should still record the handoff in the task's Updates section (Step 4)
but with topic `agent-chat-arc` instead of `dm:<a>:<b>`.

The dm topic name is not in the JSON; if the user wants it for follow-up
subscribe, derive it from the self-fp resolved in Step 2 (via
`channel info agent-presence`) and the target's discovered fingerprint
(sorted lex), or run `termlink channel list --prefix "dm:<self-fp>:"`
to locate it. Do NOT use `whoami` — its fingerprint field is unrelated
to the envelope `sender_id` that DM topics are named with (PL-195).

## Step 4: Append Update entry to the task file

Append to the `## Updates` section of `.tasks/active/<task-id>-*.md`:

```
### {ISO-8601 UTC now} — handoff-posted [agent-handoff-skill]
- **Action:** Cross-host handoff via `termlink agent contact`
- **Target:** <target>
- **Self:** <self sender_id> (or `unknown` if whoami was ambiguous)
- **Offset:** <offset>
- **Message:** [<task-id>] <first 80 chars of message>...
- **Status hint:** awaiting-reply
```

Use `>>` append, not full-file rewrite.

## Step 5: Report to user

Print a 4-line summary:

```
✓ Handed off to <target> on dm:<a>:<b> @ offset=<offset>
  Self: <self sender_id>
  Task <task-id> updated with handoff entry
  Reply via: termlink channel subscribe dm:<a>:<b> --cursor <offset+1>
```

## Rules

- **NEVER** use `termlink remote push` for agent-to-agent contact (T-1166
  retired the corresponding inbox.push primitive).
- **NEVER** use the retired `inbox.push` primitive, `event.broadcast --target`, or post
  to invented topics like `agent.reply`. The canonical contact channel is
  the `dm:<a>:<b>` topic the verb computes from identity fingerprints.
- **NEVER** improvise the sender label by passing `--metadata-from <x>` or
  similar. The identity comes from the local `~/.termlink/identity.key`
  via the registered session — do not override it.
- **NEVER** post to multiple peers in one invocation. One target per call;
  use parallel invocations if you need to fan out (the verb is idempotent
  per-message, not idempotent across runs).
- **NEVER** retry on failure without surfacing the error to the user
  first. The verb's error messages are actionable; let them through.
- **Fail fast** if any step exits non-zero. No silent fallbacks, no
  alternative "nearby" topics, no degraded paths.

## Smoke test (run once after editing this file)

Skill is a thin wrapper. End-to-end smoke:

```
/agent-handoff framework-agent T-1429 "smoke test from agent-handoff skill"
```

Expected: offset returned, T-1429 task file gets an Update entry with
`handoff-posted [agent-handoff-skill]`, and the message lands on
`dm:<self>:<framework-agent>` visible via
`termlink channel subscribe dm:... --limit 1`.

## Related

- `/check-arc` (`T-1810` + `T-1874`) — RECEIVE side; T-1874 closed the
  same `whoami` self-fp gap there. Step 2 in this skill applies the
  same fix to the SEND side for log audit consistency.
- `/be-reachable` (`T-1841`) — establishes presence on `agent-presence`,
  which is what Step 2 reads to resolve self-fp.
- **PL-195** — `whoami` self-fp resolution is structurally broken
  (candidates[].sender_id always null + ambiguous on shared hosts).
  This skill's Step 2 originally inherited that bad path.
- **T-1693** — per-agent identity keys (structural fix; until it ships,
  the resolved self-fp is the host signing key shared across sessions).
