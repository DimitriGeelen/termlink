# Injectable-Listener Spawn Recipe (T-1800 build #3)

How to run a **persistent, injectable Claude listener** — a long-lived agent
session that another agent can wake on demand to pick up a turn and respond.
This is the receiver end of the doorbell+mail loop; the sender end is
`scripts/agent-send.sh` (T-1804).

The loop, end to end:

```
sender                                     listener (this recipe)
------                                     ----------------------
agent-send.sh                              termlink spawn ... -- claude   (running, idle)
  1. channel post --msg-type turn   ─────▶ (turn sits on dm:<a>:<b>)
  2. inject <listener> "/check-arc" ─────▶ doorbell wakes claude
                                            3. /check-arc respond mode (T-1805)
                                            4. agent-respond.sh posts receipt+reply
  5. polls receipt  ◀──────────────────────  (receipt on same conversation_id)
  -> DELIVERED                              -> back to idle, awaiting next ring
```

The doorbell is just a wake signal (`inject`); the mail is structured content
(`channel.*`). No protocol changes — every step composes existing primitives.

## Why `claude`, not `claude -p`

Run the listener as plain interactive `claude` (default permission mode), **not**
`claude -p "<prompt>"`. A `claude -p` invocation is one-shot: it re-pays the full
context (CLAUDE.md, memory, session state) on every call. A persistent
interactive session pays that cost once and stays warm — the doorbell just
injects `/check-arc` into the already-loaded session. For a listener that may be
rung many times, `claude -p` is the wrong tool; reserve it for cheap one-off
fan-out, not for a standing responder.

## Governance — pick one of two sanctioned modes

A listener runs unattended and posts to hubs, so it MUST stay inside framework
governance. There are exactly two sanctioned ways to launch it:

1. **`FW_SAFE_MODE=1`** — keeps the Tier-0 (consequential-action) gate and the
   context-budget gate active; drops only the per-edit task gate (P-002). Use
   this when the listener is a pure responder that should not be blocked on
   "no active task" for every reply, but you still want destructive-action and
   budget protection.

2. **Its own `started-work` task** — `fw work-on "agent listener for <purpose>"`
   before launching, so every action the listener takes is stamped to a real
   task. Use this when the listener's work is substantive enough to warrant a
   task trail.

**Never** run the listener from an ungoverned scratch dir such as `/tmp`. Per the
T-1800 Evidence, a session launched outside the project root loses **all three**
of: the Tier-0 gate, the budget gate, and the T-559 project-boundary block. An
ungoverned responder that can be woken by any peer is exactly the thing the gates
exist to prevent. Always launch from inside the project root (`/opt/termlink`).

For hands-free replies the listener needs `Bash(termlink:*)` on its allowlist so
it can run `agent-respond.sh` (which shells out to `termlink channel post`)
without a permission prompt on every wake.

> **Root constraint (T-1807).** `--dangerously-skip-permissions` is **refused
> when running as root/sudo** (`cannot be used with root/sudo privileges for
> security reasons`) — the spawned `claude` exits immediately. On a root host
> the ONLY hands-free path is the `Bash(termlink:*)` allowlist above; do not
> reach for the skip-permissions flag. Spawn plain `-- claude`.

> **Respond-mode signal (T-1807 / T-1809).** A plain `/check-arc` doorbell wakes
> the listener in **browse mode (read-only)** — it will read the turn but not
> ack, so the sender never sees DELIVERED. Until T-1809 lands a respond-mode
> signal, the doorbell text must explicitly instruct the listener to enter
> respond mode (e.g. inject an instruction to run `agent-respond.sh` for the
> unread conversation), or use a mechanical responder. See
> `docs/reports/T-1807-doorbell-mail-loop-validation.md`.

## Spawn the listener

From inside the project root:

```bash
cd /opt/termlink && FW_SAFE_MODE=1 termlink spawn \
  --name agent-listener \
  --tags "role=listener,project=termlink" \
  -- claude
```

(Swap the `FW_SAFE_MODE=1` prefix for a prior `fw work-on ...` if you chose the
own-task governance mode.)

## Verify it is injectable

Confirm the session is registered and accepts a doorbell:

```bash
# 1. It shows up in the session list with the name you gave it:
termlink list | grep agent-listener

# 2. A test inject lands (the listener should run /check-arc and find nothing):
termlink pty inject agent-listener "/check-arc" --enter
```

If `termlink list` shows the session `ready` and the inject returns without
error, the listener is wakeable. A real ring from a peer is just
`agent-send.sh --to-session agent-listener ...`.

## Teardown

```bash
# Graceful: tell the claude session to exit; registration clears when it dies.
termlink pty inject agent-listener "/exit" --enter
# Reap the now-stale registration entry (or wait for natural expiry).
termlink clean
```

If the PTY is wedged, `termlink list` to find the PID and stop it, then
`termlink clean` to reap the stale registration entry.

## Related

- `scripts/agent-send.sh` — sender verb (doorbell + mail + receipt wait), T-1804.
  Add `--await-reply <secs>` (T-1811) to also wait for and print the listener's
  reply turn — one full request→confirm→response round-trip in a single command.
- `scripts/agent-respond.sh` — receiver ack (receipt + optional reply), T-1805.
- `.claude/commands/check-arc.md` — the `/check-arc` skill; its "Respond mode"
  section is what a doorbell-woken listener runs.
- `docs/operations/agent-conversations.md` — broader agent-to-agent comms guide.
