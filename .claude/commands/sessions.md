# /sessions — Registered Terminal Sessions (SESSION-CONTROL read verb)

The read-tier entry point for the **founding charter verb — "control terminal
sessions"** (`docs/CHARTER.md`). Answers "what terminal sessions are registered
on this host right now, and what can I do with them?" — the session-control
sibling of `/peers` (discover), `/claims` (claim-work), and `/queue-status`
(resilience).

TermLink began as a cross-terminal session-control tool: a registered session is
a **real PTY** you can stream output from, inject keystrokes into, exec commands
in, and doorbell-wake. This skill surfaces those sessions and points at the
action verbs. Read-only — it never spawns, execs, signals, or cleans.

**Invocation:**

| Form | Action |
|------|--------|
| `/sessions` | Render the human-format session table + per-session next-step hints |
| `/sessions --json` | Emit `termlink list --json` verbatim (`{ok, sessions:[…]}`) |

## Step 1: Pre-flight

Run:

```
termlink --version >/dev/null 2>&1
```

If `termlink` is not on PATH: **stop**. Print:

```
sessions: termlink not found on PATH. This skill needs the TermLink CLI.
Build + install: cargo build --release && install -m 755 target/release/termlink ~/.cargo/bin/
```

## Step 2: Read the session registry

Run:

```
termlink list --json
```

This is a pure local read of the runtime directory's session registrations — no
hub call, no auth, no mutation. The envelope is
`{"ok":true,"sessions":[{id, display_name, state, pid, age, capabilities, roles,
tags, metadata:{cwd, shell, termlink_version, identity_fingerprint}}, …]}`.

If invoked as `/sessions --json`, print that envelope verbatim and stop.

## Step 3: Render the table

Sort by `state` (ready first) then `display_name`. Print:

```
sessions: <N> registered on this host

  <display_name>  (<id>)   state=<state>  pid=<pid>  age=<age>
    cwd: <metadata.cwd>   shell: <metadata.shell>   v<metadata.termlink_version>
    caps: <capabilities csv>   tags: <tags csv>
    → exec:   termlink exec <id> '<cmd>' --json
    → view:   termlink pty output <id>   |   termlink mirror <id>   |   termlink pty stream <id>
    → inject: termlink pty inject <id> '<keystrokes>'
    → signal: termlink signal <id> TERM      (clean: termlink clean)

  <display_name-2> (<id-2>)  …
```

Keep the per-session hints to the action verbs that are safe to suggest; the
operator chooses which to run.

## Step 4: Empty-state framing (loud, not silent zero)

If `sessions` is empty, do NOT print a blank table. Print the diagnostic ladder:

```
sessions: 0 registered on this host.

This host runs no controllable terminal sessions right now. To create one:

  # One-shot: register, run a command, deregister
  termlink run -- bash -lc 'echo hello'

  # Persistent: spawn a named session you can exec/attach/inject into
  termlink spawn --name my-session -- bash
  termlink exec my-session 'echo ready' --json

  # Prove the whole verb works right now (spawn → exec → cleanup)
  bash scripts/session-selftest.sh

New here? Read docs/operations/session-control-getting-started.md — the on-ramp
for the founding "control terminal sessions" verb.
```

## Step 5: Non-empty next-step nudge

After the table (non-empty case), append once:

```
Next: exec a command (termlink exec <id> '…' --json), watch live output
(termlink mirror <id>), or read the on-ramp:
docs/operations/session-control-getting-started.md
```

## Rules

- **Read-only by contract.** This skill runs only `termlink list --json`. It
  never spawns, execs, injects, signals, or cleans — those are explicit operator
  actions the hints suggest, not things the skill performs.
- **Local by design.** `termlink list` reads the local runtime dir. For sessions
  on another hub use `termlink remote list --hub <addr>` (cross-machine session
  control is client-driven, per charter non-goal #1 — no federation).
- **Loud empty-state.** Zero sessions is a normal state on a coordination-only
  host; surface the spawn path, don't render a blank table.
- **No `AskUserQuestion`** — just run and report.

## Common patterns

**Cold-start "what can I control here?":**

```
/sessions
```

**Pipe session ids into a loop:**

```
/sessions --json | jq -r '.sessions[].id'
```

**Prove the verb before relying on it:**

```
bash scripts/session-selftest.sh   # spawn → exec → cleanup, exit 0 = proven
```

## Related

- `docs/operations/session-control-getting-started.md` — the on-ramp for this verb
- `docs/operations/session-selftest.md` — the affirmative prover (T-2485)
- `docs/operations/injectable-listener-spawn-recipe.md` — spawn a doorbell-wakeable session
- `/peers` (discover) · `/claims` (claim-work) · `/queue-status` (resilience) — the sibling read verbs
- `docs/CHARTER.md` — the four verbs; this skill serves the fourth (founding) one
