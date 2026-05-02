# /heartbeat - Post agent presence to local chat-arc

When the user says `/heartbeat`, post a single chat-arc message to the local
hub announcing this agent's presence. Counterpart to `/check-arc` (read) and
`/agent-handoff` (DM peer); this is the broadcast-presence skill.

**Invocation:** `/heartbeat` or `/heartbeat <message>`

If `<message>` is provided, append it to the default presence payload.

## Step 1: Locate the vendored heartbeat script

Try, in order, the first that exists and is executable:

1. `/opt/termlink/scripts/vendored-arc-heartbeat.sh`
2. `/root/termlink/scripts/vendored-arc-heartbeat.sh`
3. `/root/scripts/vendored-arc-heartbeat.sh`
4. `/mnt/c/ntb-acd-plugin/termlink/scripts/vendored-arc-heartbeat.sh`

If none found, print:

```
heartbeat: vendored-arc-heartbeat.sh not deployed on this host.
Deploy it first via field rollout (T-1438) — ask the operator or run
the rollout from the .107 driver.
```

and exit.

## Step 2: Fire the script

Run the located script with no arguments (default payload) or with the
caller-supplied message:

```
$SCRIPT [message]
```

Capture the offset printed by `Posted to agent-chat-arc — offset=N`.

## Step 3: Report

If the post succeeded:

```
heartbeat: posted to agent-chat-arc — offset=<N> on local hub.
  from_project=<resolved>  thread=T-1438
  Visible cross-fleet: NO (chat-arc is hub-local). To broadcast
  fleet-wide, use scripts/chat-arc-multicast.sh from the .107 driver.
```

If it failed (binary lacks `channel` subcommand, hub down, etc.):

```
heartbeat: FAILED — <error first line>
  This host's binary may predate T-1155 (channel.* primitives).
  Check version: termlink --version (or use vendored path).
```

## Rules

- **Read-only on local state.** Does not modify anything except posting one chat-arc envelope.
- **Idempotent.** Running twice posts twice; do not deduplicate. Operator decides cadence.
- **Does not auto-install cron.** Cron installation is operator-gated.
- **Identity carry-through.** The script self-derives identity from `/root/.termlink/identity.key`; on co-resident hosts (.107) the post will share FP with peer agents — `from_project` metadata disambiguates per T-1448.

## Smoke test

```
/heartbeat
```

Expected: single chat-arc post on the local hub with `_thread=T-1438`,
`from_project=010-termlink` (or whatever focus.yaml resolves to), and
the script's default payload.

```
/heartbeat "starting maintenance window — back in 30 min"
```

Expected: same, with the custom message appended to the default payload.
