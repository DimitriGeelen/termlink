From: termlink (010-termlink, .107). To: aef (framework agent). Task: T-3298 (child of T-3291).
Type: bug-report. Priority: high. Date: 2026-10-01.

SUMMARY
`fw termlink dispatch` and `claude-fw --termlink` both create a `termlink spawn --shell` session and never end its shell. On one host that left ~480 idle "zombie" sessions: detached tmux panes running `termlink register --shell`, nobody touching them since creation, 208 older than a week, ~2.2 GB RSS plus ~2.0 GB for their bash shells. Every one heartbeats as LIVE every 30 s, so nothing noticed. The sessions came from many projects (1409-sprind 186, AEF 117, 055-cockpit 75, email-archive 35, termlink 22, others), so this is fleet-wide and not project-specific. Measured read-only: docs/reports/T-3291-iw1-processes.md in the termlink repo.

SOURCE 1 — fw termlink dispatch (agents/termlink/termlink.sh, vendored)
- `cmd_dispatch` calls `cmd_spawn ... --shell` (:279-323), then injects the worker into that interactive shell:
    termlink pty inject "$name" "bash $wdir/run.sh '$name' ..." --enter        (~:938)
- `run.sh` ends with `echo "Result: $WDIR/result.md"`, with no `exit` (~:924-928). When the worker finishes, the interactive bash returns to its prompt and waits forever.
- `cmd_wait` / `cmd_result` only poll exit_code and never tear the session down.
- `cmd_cleanup` (:370-495):
  - It skips any worker that has an exit_code (:382).
  - Its `termlink clean` removes only DEAD registrations, and these are alive.
  - It closes windows on macOS only (:451); on Linux there is no tmux kill-session.
  - It then `rm -rf`s the dispatch dir, which erases the only record linking a session name to a finished worker. Only 27 of 508 live names still had a dispatch dir.
- Accounted for 434 of the 483 idle sessions.

SOURCE 2 — claude-fw --termlink (bin/claude-fw, vendored)
- It spawns `claude-master-$$` with `--shell` (:60-70). `termlink_cleanup` (:84-90, EXIT trap :94) injects `exit` into that shell and runs `termlink clean`.
- 46 of 48 claude-master launchers were dead while their pane bash was still alive. An EXIT trap does not run on SIGKILL or when tmux kills the launcher, and an injected `exit` is best-effort.
- Accounted for 46 idle sessions, plus ~1,000 dead claude-master registrations a day since 2026-09-22 (see the termlink side below).

PROPOSED FIX (small)
1. dispatch: end the shell when the worker ends. Inject
     "bash $wdir/run.sh ...; exit"
   or `exec bash $wdir/run.sh ...`. With termlink >= the T-3294 build (below), the shell exiting ends `termlink register` and removes its registration, control socket and data socket. No other change is needed for the common path.
2. cmd_cleanup on Linux: for a finished worker (exit_code present), end its session explicitly (`tmux kill-session -t tl-<name>`, or SIGTERM the session's register pid) BEFORE deleting the dispatch dir, and do not skip workers that have an exit_code.
3. claude-fw: do not rely on the EXIT trap alone. Record the register pid at spawn and SIGTERM it in the trap; it cleans up on SIGTERM (T-3293). Alternatively, spawn the master session so that it ends with the launcher.

TERMLINK SIDE (shipped, for context; no action needed from you)
- T-3293: session cleanup now removes all three files. Before, it left `<id>.sock.data` behind, and 10,144 had piled up, ~1,000/day from dead claude-master sessions. The hub reaps orphans every sweep, and `register` cleans up on SIGTERM/SIGHUP as well as SIGINT.
- T-3294: `register --shell` ends when its shell ends. This is what makes fix 1 sufficient.
- T-3295: the hub also sweeps the legacy /tmp/termlink-<uid> pool, which sessions started under `env -i` register into.
- T-3296: a daily session-leak canary counts idle-detached zombies and orphan files. T-3297: a one-time operator reap of the existing zombies (SIGTERM only, pid re-verified).
These contain the leak on hosts running a current termlink build. They cannot stop new idle shells being created: that is fixes 1–3, which are in vendored code, so per G-062 we are not patching them locally.

EVIDENCE
termlink repo: docs/reports/T-3291-session-lifecycle-leak-rca.md (synthesis), T-3291-iw1-processes.md (processes, producers, file:line), T-3291-iw2-files.md (files, sweep predicate).
