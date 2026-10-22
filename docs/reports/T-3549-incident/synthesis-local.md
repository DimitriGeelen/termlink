# T-3549 — synthesis of the three local reviews (codex, antigravity, claude; opencode failed)

AEF's three reviews (/opt/999-Agentic-Engineering-Framework/docs/reports/T-4635-review/results/) were still running at
writing time. Fold them in before the operator decision.

## Confirmed finding that changes the severity (antigravity, verified by the orchestrator)
T-3445 made the leak worse for older frameworks. The framework's T-2289 guard (`.agentic-framework/lib/paths.sh:64-66`)
unsets inherited `TASKS_DIR`/`CONTEXT_DIR` only when the sentinel `_FW_PATHS_DERIVED_BY` is present and differs.
T-3445's deny-list strips that sentinel (and `PROJECT_ROOT`) from the tmux client but leaves `TASKS_DIR`/`CONTEXT_DIR`.
So the new server carried the foreign paths without the sentinel, and any framework before T-4222 (paths.sh:68-84) treats
them as a deliberate test override and follows them. Verified by reading paths.sh in our 1.8.14 copy; T-4222's own
comment names this route ("sentinel absent (unset by a polluted session…)").

## Where all three agree
1. The defect class is first-caller-wins ambient authority: a shared, persistent process (the default tmux server)
   seeded by whichever caller starts it. A deny-list cannot uphold "the server never carries a project".
2. Fix at the source by construction: `Command::env_clear()` (env -i) plus a small documented allow-list (HOME, USER,
   LOGNAME, SHELL, TERM, LANG/LC_*, TMPDIR, XDG_RUNTIME_DIR, a fixed system PATH). Project settings go to each
   session explicitly, from the TARGET's validated launch spec, not from the spawning agent.
3. Every spawn path, not just tmux_new_session: the background backend (`execution.rs` detached_command ~747-760),
   `scripts/tl-claude.sh` (its `env -u PROJECT_ROOT` misses the other variables, ~171/249), and AEF's
   `fw termlink dispatch` (1409 traced the seed there).
4. The framework should refuse, not warn, when an inherited path points outside the project, and old vendored copies
   need upgrading (none below 1.8.14 should remain).
5. 14:55 is not the end of exposure: running processes and their descendants keep the values until relaunched.
6. Tests must use canary variables (unknown names), polluted existing servers, and crash/restart races, not just
   PROJECT_ROOT (the current live test, tmux_spawn.rs ~147-189).

## Where they differ: the tmux isolation boundary
- codex: a socket per project, deliberately bootstrapped by a managed launcher; per-hub only if the hub serves one
  project.
- antigravity: one dedicated socket per TermLink hub (`tmux -L termlink`), pre-started by a systemd unit with
  `env -i`, `update-environment ""` and `exit-empty off`; per-project sockets fragment fleet inspection.
- Both: the default (human) server must not be used; cockpits need a wrapper or must enumerate registered sockets.

## Detection (agreed shape)
A live-process scan (/proc environ of registered sessions and their descendants) for project variables that do not
match the process's own project, plus a server-global-env check. It runs as a canary and at spawn time.
`termlink list --json` (created_at, pid) joined with /proc environ already finds them: 30 of 54 on 2026-10-22.

## Proposed remediation order (for the operator)
1. P0 now: env_clear + allow-list in tmux_new_session and the background backend; fix tl-claude.sh; canary tests.
2. P0 now: the detector (live scan + server global env), wired as a cron canary.
3. P1 decision: isolation boundary (per-hub dedicated socket vs per-project); the operator rules after AEF's reviews.
4. AEF side (their tasks): the dispatch allow-list, refuse-not-warn in paths.sh, upgrade old vendored copies.
5. Relaunch contaminated agents per project (owners/operator), oldest framework first.
