# T-3291 — Session lifecycle leak: RCA

Status: RCA complete; recommendation GO (slices below), awaiting the operator's decision.
Detail reports: `T-3291-iw1-processes.md` (processes), `T-3291-iw2-files.md` (files).
All measurement was read-only. Nothing was killed or deleted, and no cleaning
`termlink` subcommand was run.

## Summary

There are **two independent leaks** with separate root causes. Both are invisible to
every existing guard.

| Leak | Size (2026-09-30) | Root cause | Why nothing noticed |
|---|---|---|---|
| **Zombie sessions** | 508 live `termlink register` processes; ~483 (95%) idle detached `tl-*` tmux shells; 208 older than 7 days; ~4.2 GB RSS incl. shells | Nothing ends a spawned session: the dispatched script finishes and the shell stays at its prompt, and `register` never exits on shell exit, parent death or idleness | `register` heartbeats every 30 s whether or not anyone uses it, so every zombie looks LIVE to every canary |
| **Orphan data-plane files** | 10,144 orphan `*.sock.data` in `/var/lib/termlink/sessions` (+ at least 386 in `/tmp/termlink-0/sessions`); ~1,000/day since 2026-09-22 | The hub sweep removes a dead session's `.json` + `.sock` but **never its `.sock.data`**; once the `.json` is gone no code path can find it | The sweep runs and logs "Cleaned stale" (845 sweeps since 09-23), so it looks healthy; no check counts orphan files |

## IW-1 — the processes

- **508 live** `register` processes; 502 of them run a since-replaced binary.
- **By age:** <1h 35, 1h–1d 116, 1–3d 56, 3–7d 93, **>7d 208**.
- **Classes:**
  - **483 idle.** Each is the pane of a detached `tl-<name>` tmux session under one tmux server (pid 2600). The pane has `session_attached=0`, no activity since it was created, and its bash has no children.
  - **9 busy** (a claude or fw process under the shell).
  - **4 systemd-supervised** agents, which are legitimate.
  - **12 other:** leftovers from a stopped service, `t392w-*` processes orphaned to init, and one recovery session.
- **Producers:**
  - The vendored `fw termlink spawn/dispatch` accounts for 434 of the 483 idle (`.agentic-framework/agents/termlink/termlink.sh:279-323`, `:932-940`).
  - `claude-fw --termlink` accounts for 46 (`.agentic-framework/bin/claude-fw:60-70`); 46 of its 48 launchers are dead.
  - The leak is **fleet-wide across projects**: 1409-sprind 186, AEF 117, 055-cockpit 75, email-archive 35, termlink 22, and others.
  - **Nothing** in cron, systemd, scripts or skills ever runs a cleanup of `tl-*` sessions.
- **How `register` exits** (`crates/termlink-cli/src/commands/session.rs:467-493`):
  - It waits only on its accept loop and on `ctrl_c`.
  - The PTY read loop (`session.rs:420-431`) is a detached task nobody watches, so **shell exit does not end `register`**.
  - There is no parent-death signal, no parent polling, no TTL and no idle timeout anywhere in `crates/`.
  - SIGTERM and SIGHUP take the default action: the process dies but leaves its files behind.
- **Dispatch never tears a session down:**
  - `fw termlink dispatch` injects `bash run.sh`, and `run.sh` has no `exit`.
  - `cmd_cleanup` skips any worker that has an `exit_code`, then deletes `/tmp/tl-dispatch`, which is the only record linking a name to a finished worker.

## IW-2 — the files

- **10,888 entries:** 10,386 `.sock.data`, 251 `.json`, 251 `.sock`.
- **10,144 session ids have only a `.sock.data`.** All 251 JSONs parse, and every pid is a live termlink process: **no dead registrations remain**. So the hub's sweep is keeping up with deaths.
- **The sweep runs every 30 s** (`supervisor.rs:18`, started at `server.rs:368-371`), and the journal shows 7,686 distinct ids cleaned.
  - `TERMLINK_SWEEP_INTERVAL_SECS=3600` in the unit only affects the bus retention sweeper, not this one.
- **Gap 1:** `cleanup_stale` (`crates/termlink-session/src/liveness.rs:44-55`) deletes `.sock` and `.json` only. Every other cleaner either reuses it or looks for orphans by `.sock` (list/clean `manager.rs:412`, doctor `--fix` `infrastructure.rs:403`, MCP clean `tools.rs:13709`), so all of them miss `.sock.data`.
- **Gap 2:** `register` removes its own `.sock.data` only on SIGINT (`session.rs:475-490`).
- **Gap 3:** when `TERMLINK_RUNTIME_DIR` is set, the hub sweeps only that directory (`discovery.rs:44-46`). The ~262 sessions launched from an `env -i` tmux server register in `/tmp/termlink-0/sessions` (1,172 entries), which **nothing sweeps**.
- **Growth driver** (medium confidence): 7,654 of the dead ids are `claude-master-<pid>` from `claude-fw --termlink`, which tears its session down by injecting `exit` and running `termlink clean`, never SIGINT. Volume jumps from 10–30/day to 400–2,200/day from 2026-09-22.

## Recommendation: GO, in four slices (ordered by risk and value)

1. **S1 — file leak, local Rust (low risk).**
   - `cleanup_stale` also deletes `<id>.sock.data`.
   - The sweep reaps a `.sock.data` with no matching `.json` once it is older than a grace period, never while a live registration references it.
   - `register` cleans up on SIGTERM and SIGHUP as well as SIGINT.
   - The existing 10k orphans are then reaped by the hub itself after the restart onto the fixed binary. There is no manual `rm`.
2. **S2 — `register --shell` exits when its shell exits (local Rust).** Watch the PTY task in the `select!`: EOF/EIO on the PTY ends `register` and runs cleanup. This makes a session's life equal its shell's life.
3. **S3 — end the idle zombies that exist today, then stop new ones. Needs your authority:**
   - **(a) Upstream, per G-062:** the vendored dispatch and `claude-fw` must end their shell when the work finishes. File this at `framework:pickup`.
   - **(b) Locally:** a one-time runme action that terminates **only** the idle zombies. Criteria: detached `tl-*` tmux session, bash with no children, no pane activity for over 24 h. It kills the tmux session, never an attached or busy one, and prints every name first.
4. **S4 — detection (G-019).** A daily **session-leak canary**. It fires when:
   - orphan `.sock.data` files exceed a threshold;
   - more than N sessions are idle-detached for over 24 h; or
   - any session registry is outside the hub's swept directories (the `/tmp/termlink-0` pool).

   This is the check whose absence let both leaks run for weeks.

**Not recommended:** an idle-timeout TTL inside `register`. A session idle for a day can be legitimate, for example a parked worker. The shell-exit rule (S2) plus the upstream exit (S3a) removes the cause without guessing at intent.

## Dialogue Log

- Operator (2026-09-30): "we need to RCA that and remediate that structurally
  because this is debt building up with a risk of exploding over some time and
  dragging us down. We cannot absolutely have 460 active sessions running … I'm
  sure they're not all active. … think how we're gonna manage better exit of these
  sessions or a way that we don't get a build-up of zombie sessions all the time.
  And zombie files."
- Measured outcome: the operator's intuition was correct, and it turned out to be two leaks, not one.
