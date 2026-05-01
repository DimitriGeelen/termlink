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

Run:

```
termlink whoami --json 2>/dev/null
```

If single candidate: capture `sender_id`.
If multi-candidate (multiple sessions on this hub): prefer the one whose
`cwd` matches `$(pwd)`. If still ambiguous, fall back to the first
candidate and note the ambiguity in the Update entry.
If `whoami` fails: continue with `sender_id=unknown` — identity binding
is enforced server-side once T-1427 ships, not here.

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

The dm topic name is not in the JSON; if the user wants it for follow-up
subscribe, derive it from `termlink whoami` (self fingerprint) and the
target's discovered fingerprint (sorted lex), or run
`termlink channel list --prefix "dm:<self-fp>:"` to locate it.

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
- **NEVER** use `termlink inbox push`, `event.broadcast --target`, or post
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
