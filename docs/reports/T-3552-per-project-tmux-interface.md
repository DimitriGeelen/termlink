# T-3552 — one tmux server per project: interface draft

Status: DRAFT for comment (055-agentic-fleet-cockpit asked to see it before it is fixed). Ruling: operator ruling A
on SEV-1 T-3549 (`docs/reports/T-3549-incident/synthesis-six.md`, items 5 and 10).

## Why

Every `termlink spawn`/`dispatch` with the tmux backend runs `tmux new-session` on the host's DEFAULT server, which
is shared by every project and by the human. Whoever starts that server seeds its global environment; on 2026-10-22
an ssh logout killed it at 13:38:43 and the next spawn (from another project) seeded the new one with that project's
`TASKS_DIR`/`CONTEXT_DIR` (T-3549). T-3550 now builds each session's environment by construction, so a seeded server
can no longer leak into a TermLink session; this task removes the shared server itself from the picture, so one
project's server, its lifetime and its environment cannot touch another project.

## 1. Which server a session lives on

| Input | Server |
|---|---|
| spawn/dispatch whose working directory (`--workdir`, `--cwd`, else the caller's cwd) lies in a project | that project's server |
| working directory in no project | the host server `tl-host` |
| anything TermLink did not start (a human's own `tmux`) | untouched: the default server stays the human's |

"Project" uses the rule `termlink register` already enforces (T-3551, `termlink_session::child_env::project_of`):
the nearest ancestor holding `.framework.yaml`, else `.git`; a git worktree counts as its main checkout. One rule
everywhere, so the server a session is on and the project its environment is judged against cannot disagree.

## 2. Socket name (never a path)

`tl-<slug>-<hash8>`, used only as `tmux -L <name>`:

1. `slug` = the project directory's basename, lowercased, every character outside `[a-z0-9]` replaced by `-`,
   runs collapsed, trimmed, cut to 24 characters (`010-termlink` → `010-termlink`).
2. `hash8` = the first 8 hex digits of SHA-256 of the canonical (realpath) project root. Two checkouts with the same
   basename get different servers; a moved project gets a new one (its old server dies with its sessions).
3. The result is validated against `^tl-[a-z0-9-]{1,24}-[0-9a-f]{8}$` before use. TermLink never passes `-S`, and
   never builds a `-L` name from anything but this function (the T-3425 lesson: an unchecked path to `-S` once
   stripped the execute bits from `/`).

The socket file lands where tmux puts every `-L` socket (`$TMUX_TMPDIR`, else `/tmp`, under `tmux-<uid>/`).

## 3. How a server is started

Only TermLink's launcher creates a project server, and it creates it the same way every time:

```
env -i <host allow-list> tmux -L <name> -f /dev/null new-session -d -s <session> <wrapped cmd> \; \
    set -g exit-empty off \; set -g update-environment ""
```

1. `env -i` + the T-3550 allow-list: the server's global environment never holds a project, an agent's sockets or
   credentials, whoever starts it.
2. `-f /dev/null`: no `~/.tmux.conf`, so a user's config cannot add `update-environment` entries back.
3. `update-environment ""`: attaching clients never copy their environment into the server.
4. `exit-empty off`: the server survives its last session, so the next spawn does not re-seed it.

Supervision (surviving an ssh logout, which is what killed the default server) is T-3553's question. This task
records the server's pid in session metadata so T-3553 has something to supervise.

## 4. Commands (all take a PROJECT PATH, per 055)

```
termlink tmux socket <project-path>                         # prints the -L name; exit 2 if not a project
termlink tmux list   [<project-path> | --all] [--json]      # sessions per TermLink server
termlink tmux attach <project-path> [<session>]             # exec: tmux -L <name> attach -t <session>
```

1. `socket`: for scripts that must call tmux themselves (055's launchers): `tmux -L "$(termlink tmux socket .)" ...`.
2. `list --json`: one object per session:
   `{project, server, session, attached, created, panes:[{pid, cmd}], registered_session_id}`. `--all` enumerates
   every `tl-*` server of this uid (the socket directory), including `tl-host`. A server whose socket exists but does
   not answer is listed with `"state": "dead"`, never silently skipped.
3. `attach`: without `<session>`, attaches when the project has exactly one session and lists them otherwise.
   It replaces the process (`exec`), so a cockpit gets a normal tmux client.
4. The registration metadata of every spawned session gains `tmux_server` (the `-L` name) beside `pty_session`, so
   `termlink list --json` already tells a cockpit where a session is without calling tmux.

## 5. Migration

1. Nothing is killed. Sessions on the default server keep running until they end; only NEW spawns go to project
   servers. The default server is never killed to migrate (synthesis item 10).
2. `scripts/lib/session-zombies.py` (the zombie reap and the session-leak canary) enumerates every `tl-*` server as
   well as the default one, or project-server zombies would become invisible.
3. The T-3551 env-leak canary adds each `tl-*` server's global environment to its scan.
4. Test and demo scripts that drive tmux directly (`session-selftest.sh`, `tl-dispatch.sh`, the push-wake demos)
   move to `termlink tmux socket` or keep their own private `-L` servers.

## 6. Open points for comment

1. **Host server for project-less spawns** (`tl-host`) versus refusing such spawns. The draft keeps them working.
2. **Name stability across moves:** hashing the path means a moved project starts a new server. The alternative,
   hashing `.framework.yaml`'s `project_id`, survives a move but does not exist for non-framework git repos.
3. **055:** does `list --json` above carry everything the cockpit needs (pane pids, attached flag, registered id)?
