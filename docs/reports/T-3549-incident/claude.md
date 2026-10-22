**Verdict**

The incident is not closed, and the 14:55 cleanup was only partial. I checked the live default server just now. Its PID is 1084466, and it is the same server that the `tl-t2482-r1` spawn from 1409-sprind started at 13:39:08. Its global environment still holds 1409's Claude Code identity:

- `CLAUDE_CODE_MESSAGING_SOCKET=/tmp/cc-socks/214842.sock`
- `CLAUDE_CODE_MESSAGING_TOKEN` (value redacted here)
- `CLAUDE_CODE_SESSION_ID`
- `CLAUDE_PID=214842` (a live `claude --resume` whose working directory is `/opt/1409-sprind`)
- `CLAUDE_CODE_CHILD_SESSION=1`, `AI_AGENT`, `FW_CLAUDE_FW_SUPERVISED`, `CLAUDE_EFFORT`

Panes in 0506-Voxtype, 1023-chromium-vault, 832-Workflow-designer, 999-AEF and /opt/termlink carry that socket and token today. Any agent in those panes can talk to 1409's live agent over the socket, and it holds the token to do so. This is the same class of leak that T-3463 closed for the sidecar sockets, and it has reached a third name set.

The T-3445 deny-list did what it was built to do: the 13:39 command line shows the four `-e` overrides. It is the wrong design. The fix should be a clean server environment plus an allow-list, and the server should be owned by TermLink, not by whichever client happens to start first.

**Root cause (ranked)**

1. **The server's global environment comes from whichever client starts it.** It is shared host state that any client can write to, and nobody owns its lifecycle. Every other problem below comes from this.
2. **A name-based deny-list** (`tmux_spawn.rs:19-25`, `:32`) has failed three times now: PROJECT_ROOT, then TASKS_DIR/CONTEXT_DIR, then CLAUDE_CODE_*. A list of names you know about can never cover variables you don't know about.
3. **The framework trusts inherited paths.** This is the amplifier. Version 1.8.13 or later contained the damage. Older vendored copies did not.
4. **Locating a project through environment variables.** This is a legitimate design. Making the working directory or `.framework.yaml` the source of truth would reduce the harm, but it would not remove the vector.
5. **One server for all projects.** This raises the blast radius but is not the cause. A clean, owned server is safe to share.

**Remediation (prioritised, concrete)**

- **P0, now:**
  - Run `tmux set-environment -g -u` for every `CLAUDE_*`, `AI_AGENT`, `FW_*` and `TERMLINK_*` variable on the server except `TERMLINK_RUNTIME_DIR`, and audit everything else in `show-environment -g`.
  - Treat the token as exposed. Decide whether to restart 1409's agent (PID 214842) so it gets a new socket and token.
  - Restart affected agents, or `unset` these variables in each one, since cleaning the server does not fix panes that are already running.
- **P1, a server TermLink owns:**
  - The hub (`termlink-hub/src/supervisor.rs`) starts it at hub start with `env -i HOME=… PATH=<system path> SHELL=… LANG=… TERM=… tmux -L termlink -f /dev/null start-server`.
  - Then run `set -g update-environment ""`, plus `set -g exit-empty off` so the server cannot die and get re-seeded by a random client.
  - `tmux_new_session` targets `-L termlink` and fails if the server is missing (or asks the hub to start it). It never starts the server implicitly.
- **P1, the tmux client runs with `env_clear()` plus an allow-list.** That means `HOME`, `PATH`, `LANG`, `TERM`, `USER`, `SHELL` and `TMUX_TMPDIR`. Project variables go to the session only, through `-e`, and are re-derived from the target's working directory rather than copied from the caller. Add a test asserting that `show-environment -g` equals the allow-list exactly; the current test at `:142-188` checks only `PROJECT_ROOT`.
- **P2, isolation boundary:**
  - Use one socket per hub, not one per project. Per-project sockets break cockpits and `list-sessions` and multiply servers.
  - Keep the default server for humans. A wrapper `tl-attach` covers the `tmux attach` habit.
  - Moving existing sessions means a drain-and-respawn, not a migration.
- **P2:** Search for every place that can start a server outside the helper:
  - `dispatch.rs:918`, `execution.rs:659` (`tmux -V`, which is harmless, but similar probes like `has-session` start no server)
  - `.agentic-framework/agents/termlink/termlink.sh`
  - `be-reachable.sh`, `continuous-driver.sh`
  - `scripts/tl-dispatch.sh`
  - the `t32xx-livefire-*` tools

  `has-session` and `list-*` do not start a server, but `new-session` from any shell does. Every one of these paths needs `-L termlink` or a clean environment.

**Detection**

- **Server-environment canary.** The hub runs `tmux -L termlink show-environment -g` every 30–60 s and on every spawn, and raises an alert on any variable outside the allow-list. It is cheap, and it would have caught this incident at 13:40 instead of 14:55.
- **Pane coherence check.** Walk `list-panes -a -F '#{pane_pid}'` and read `/proc/<pid>/environ`. Flag a pane when `TASKS_DIR`, `CONTEXT_DIR`, `PROJECT_ROOT` or `CLAUDE_PROJECT_DIR` is not under its working directory, or not under its registered project. Also flag a pane whose `CLAUDE_CODE_MESSAGING_SOCKET`/`CLAUDE_PID` belongs to a process with a different working directory. I ran exactly this check by hand for this review; it takes about 10 lines.
- **Framework hook.** At session start, the framework emits a structured event (not just stderr) whenever it overrides an inherited path, so the hub counts these overrides.

**Damage assessment**

- In `/opt/1409-sprind`:
  - Run `git log`/`git status` over `.tasks` and `.context` for the window 13:39–14:55 (and to now for the `CLAUDE_*` part).
  - Run `find .tasks .context -newermt '2026-10-22 13:39' ! -newermt '2026-10-22 14:56'`.
  - Look for task IDs out of sequence or with a foreign prefix or owner, and episodic or audit records whose `session_id`/working directory is not 1409.
- Map writers to processes: for every pane and agent started in the window, list the framework version vendored in its working directory. Only copies older than 1.8.13 were at risk. Then compare their `.tasks`/`.context` for writes that are missing (written to 1409 instead).
- For the messaging socket, check 1409's Claude transcript for inbound messages it did not expect since 13:39.
- Use the hub's registration log to see which `termlink register` calls ran with foreign variables.

**Framework side**

- **Refuse** when an inherited path is outside the project and disagrees with the path derived from the working directory. Allow it only with an explicit `FW_ALLOW_FOREIGN_ROOT=1`. A warning is not containment.
- Old vendored copies cannot be patched in place. Fleet-scan the vendored versions and force an upgrade, or put a guard in the shared launcher (`tl-claude.sh`) that unsets the variables before running the framework.

**What we missed**

- **Agent identity and credentials leak, not just paths.** The Claude Code messaging socket and token give cross-agent command injection, which is worse than misplaced task files. The same applies to `SSH_AUTH_SOCK`: it is in `update-environment`, so it follows whoever attached last.
- **"Cleared at 14:55" was verified only for the two variables we already knew about.** The checklist after a cleanup should diff against an allow-list, not grep for the known names.
- **`PATH` is a vector.** It is clean today, but a project-local `bin/` entered first would hijack `termlink`/`tmux` for every session.
- **Other shared host state of the same shape:**
  - `TERMLINK_RUNTIME_DIR=/var/lib/termlink`: the first starter decides permissions and ownership.
  - `/tmp/cc-socks`, owned by root, and every agent runs as root. There is no Unix-user boundary at all, so any isolation here is cooperative, never enforced.
  - The user crontab: `.context/cron/agentic-audit.crontab` is a project file, so an install from the wrong `PROJECT_ROOT` targets the wrong repository.
  - systemd user and root slice teardown under load. It will happen again, so the server must be deliberately restarted by its owner, not lazily by whatever client comes first.
- **Per-project systemd units or Unix users** are the real enforced boundary if the fleet keeps growing.
