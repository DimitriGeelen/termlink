# Session control — getting started

**Audience:** operators and agents who know TermLink as a message bus but have
never used its **founding** capability — controlling real terminal sessions. The
goal of this doc is to get you from "I didn't know it did that" to spawning,
exec'ing into, and watching a live PTY in five minutes.

## 1. What this is, in one paragraph

The charter's very first identity clause is "control terminal sessions," and its
Origin section states plainly: **TermLink *began* as a cross-terminal
session-control tool** (`docs/CHARTER.md`). A registered session is a **real
PTY** — not a text channel. Peers can stream its output, inject keystrokes, exec
commands and capture the result, doorbell-wake it, and signal its process. The
message-bus, claim, and discover verbs grew *on top* of this; session control is
the load-bearing noun "with terminal endpoints." Everything below drives an
actual shell.

This is a peer capability of the substrate coordination verbs, not a subset of
them: `docs/operations/substrate-getting-started.md` covers discover / exchange /
claim; **this** doc covers the fourth verb.

## 2. Is there anything to control here? One daily verb

One read-only skill answers "what terminal sessions are registered on this host?"

| Skill | Reads | Answers |
|---|---|---|
| `/sessions` | `termlink list --json` | "What sessions exist here, and what can I do with each?" |

`/sessions` is the session-control sibling of `/peers` (discover), `/claims`
(claim-work), and `/queue-status` (resilience) — a pure local read, <2s, no hub
call. Empty output is normal on a coordination-only host; it prints the spawn
path rather than a blank table.

## 3. Your first session — five minutes

Copy-paste this. It spawns a named PTY, execs a command into it (inject → run →
capture), reads its output, and cleans up — the full round-trip.

```bash
# Step 1 — spawn a persistent session running a shell.
termlink spawn --name my-first -- bash
# → registered session tl-... (display_name=my-first)

# Step 2 — exec a command INTO that session and capture stdout+exit.
#          This is the inject→run→capture round-trip, returned as JSON.
termlink exec my-first 'echo hello from the PTY' --json
# → {"ok":true,"exit_code":0,"stdout":"hello from the PTY\n","stderr":"","truncated":false}

# Step 3 — read the session's recent terminal output (what a viewer would see).
termlink pty output my-first
# (or watch it live: termlink mirror my-first   —   read-only stream)

# Step 4 — clean up: signal the process, then reap dead registrations.
termlink signal my-first TERM
termlink clean
```

That is the founding verb end-to-end: **spawn → exec → view → teardown.** You now
know what "control terminal sessions" means because you did it.

**One-shot variant.** If you don't need a persistent session, `termlink run --
<cmd>` registers, execs, and deregisters in one call:

```bash
termlink run -- bash -lc 'uname -a'
```

**Prove the verb works right now (T-2485).** For an automated PASS/FAIL of the
round-trip without typing each step:

```bash
bash scripts/session-selftest.sh          # human report, exit 0 proven / 1 broken / 2 tooling
bash scripts/session-selftest.sh --json   # {ok, proven, broken_stage, stages, session, sentinel}
```

It runs SPAWN → EXEC → CLEANUP and names the broken stage if any. Exit 0 means
the verb is live on this host. See `docs/operations/session-selftest.md`.

## 4. The full surface

`termlink --help` is headed "Cross-terminal session communication." The
session-control verbs:

| Verb | Does |
|---|---|
| `termlink spawn --name <s> -- <cmd>` | Register a new PTY session running `<cmd>` |
| `termlink register` | Register the current terminal as a session + listen |
| `termlink list [--json]` | List registered sessions (the `/sessions` skill wraps this) |
| `termlink status <s>` / `ping <s>` | Query a session's health / liveness |
| `termlink exec <s> '<cmd>' [--json]` | Inject a command, wait, capture stdout+stderr+exit |
| `termlink interact <s> '<cmd>'` | Interactive PTY run — inject, wait for completion, return output |
| `termlink run -- <cmd>` | Ephemeral session: register + exec + deregister |
| `termlink pty attach <s>` | Attach to a session's PTY (interactive) |
| `termlink pty inject <s> '<keys>'` | Inject keystrokes into the PTY |
| `termlink pty output <s>` | Print the session's recent terminal output |
| `termlink pty stream <s>` | Stream output continuously |
| `termlink pty resize <s> <cols> <rows>` | Resize the PTY |
| `termlink mirror <s>` | Read-only live mirror of the PTY output |
| `termlink signal <s> <SIG>` | Send a signal (TERM/INT/…) to the session's process |
| `termlink clean` | Remove stale (dead) session registrations |
| `termlink discover --tag/--role/--cap` | Find sessions by tag / role / capability / name |
| `termlink dispatch` | Spawn N workers + tag + collect (atomic) |

All are reachable via MCP too (`termlink_{exec,inject,output,spawn,interact,run,
signal,pty_mode,resize,list_sessions,…}`), so an agent controls sessions without
shelling out.

## 5. The doorbell-wake path

A session registered as *reachable* can be **doorbell-woken** — a peer's durable
DM rings its PTY so it picks up work even when nothing else is watching. This is
the wake half of session control and the backbone of push-based agent comms
(arc-004). To make a session wakeable, spawn it as an injectable listener:

- `docs/operations/injectable-listener-spawn-recipe.md` — spawn a doorbell-wakeable session
- `/be-reachable` — opt the current agent session into presence so peers can reach it

## 6. Where to go next

- **"How do I prove session control is live on a host?"** —
  `docs/operations/session-selftest.md` (the affirmative prover).
- **"How do I make a session wakeable by peers?"** —
  `docs/operations/injectable-listener-spawn-recipe.md` + `/be-reachable`.
- **"What about the coordination verbs (discover / exchange / claim)?"** —
  `docs/operations/substrate-getting-started.md` (the sibling on-ramp).
- **"What is TermLink, exactly?"** — `docs/CHARTER.md` (the four verbs; this doc
  serves the fourth and founding one).

## References

- **docs/CHARTER.md** — the canonical purpose; "control terminal sessions" is verb #4 and the founding one
- **T-2485** — `scripts/session-selftest.sh` (the affirmative prover this on-ramp points at)
- **T-2539** — this doc + the `/sessions` skill (session-control consumer surface parity)
- **`.claude/commands/sessions.md`** — the `/sessions` read-tier skill
