# Incident review brief (SEV-1): cross-project environment leak through a shared tmux server

You are one of seven independent reviewers. Investigate deeply, think adversarially, and propose remediation.
Disagreeing with our current thinking is useful. Answer in English, at most ~900 words, markdown, with the sections
listed at the end. If you can read the referenced source in your workspace, do so and cite file:line. Otherwise answer
from this text.

## System

- **AEF (agent framework).** Each software project on a Linux host has its own directory with a task store
  (`.tasks/`), a context store (`.context/`) and a vendored copy of the framework. Framework tools locate a project
  through environment variables, notably `PROJECT_ROOT`, `FRAMEWORK_ROOT`, `TASKS_DIR` and `CONTEXT_DIR`. AI coding
  agents (Claude Code) run one per project, usually inside a tmux session.
- **TermLink** is a Rust substrate on the same host. It runs a hub, registers terminal sessions, and has `termlink
  spawn` / dispatch commands that start new terminal sessions, usually with `tmux new-session` on the host's
  **default** tmux server. That one server is shared by every project and every user agent on the host, because all
  run as the same Unix user.
- **tmux behaviour that matters:** the first client to start a server passes its own environment, and the server
  keeps it as its **global environment**. Every session started on that server later inherits the global environment,
  unless the variable is overridden per session (`new-session -e K=V`) or listed in `update-environment`.

## What happened (2026-10-22, from AEF's RCA, their T-4635)

1. **13:39.** Under host load, systemd force-tore down the root user's slice. The default tmux server died, and
   sessions inside it were killed.
2. **The next `termlink spawn`** came from project **1409-sprind**. It started a new default server, and that
   server's global environment received 1409's `TASKS_DIR` and `CONTEXT_DIR`.
3. **13:39–14:55.** Every session started or restarted on the host inherited those two variables. That includes
   relaunched agents of other projects and TermLink's own agent session. We verified that this session still carries
   `TASKS_DIR=/opt/1409-sprind/.tasks` and `CONTEXT_DIR=/opt/1409-sprind/.context`.
4. **14:55.** AEF cleared the server's global environment. Sessions started after that are clean.
5. **Containment that held:** framework versions ≥ 1.8.13 detect "inherited `TASKS_DIR` outside project" and
   re-derive the path, with a warning only. Older vendored framework copies (e.g. 1.8.10) follow the inherited paths
   and could write tasks or context into 1409's directory. **Damage across projects is not yet measured.**
   TermLink's own session (framework 1.8.14) wrote nothing foreign.

## This is a recurrence

- **2026-10-16:** a dispatch from one project started the default server, and five fleet agents plus the TermLink main
  session then ran with that project's `PROJECT_ROOT` / `FRAMEWORK_ROOT`, acting on the wrong repository.
  Fixed by TermLink T-3445.
- **The fix**, in `crates/termlink-cli/src/commands/tmux_spawn.rs`:
  - The tmux client runs with a **deny-list** removed: `PROJECT_SCOPED_ENV = ["PROJECT_ROOT", "FRAMEWORK_ROOT",
    "_FW_PATHS_DERIVED_BY", "FW_ROUTED_FROM", "CLAUDE_PROJECT_DIR"]`.
  - Each listed variable the caller has is passed to the new session only (`-e K=V`).
  - `LAUNCH_SCOPED_ENV` (sidecar sockets) are removed and not forwarded.
  - Production always uses the default server; a `-L` socket is used in tests only.
- **Callers:** `commands/execution.rs` (spawn, ~line 700) and `commands/dispatch.rs` (worker dispatch, ~line 962).
  Both use `tmux_new_session(...)`.
- **The gap:** `TASKS_DIR` and `CONTEXT_DIR` are not in the deny-list, and those are exactly the variables that leaked
  today. A deny-list of names missed the next name. Unknowns: other variables (any `FW_*`, `TERMLINK_*`, agent ids,
  tokens, `PATH` entries pointing into a project), and other ways a tmux server can be started on this host (scripts,
  wrappers, the framework's own launcher, manual `tmux`) that do not go through this helper.
- **Not prevented:** nothing detects a polluted server environment. It was silent for 76 minutes and was found by a
  crash RCA, not by a check.

## Questions for you

1. **Root cause.** What is the real defect class? Name-based deny-list, sharing one tmux server across projects,
   environment-variable project location, or the framework trusting inherited paths? Rank them.
2. **Source-side guard.** Deny-list vs allow-list vs `env -i` (clean environment plus an explicit minimal set) for the
   tmux client. What should the server's global environment contain, and how do you start a server with a clean
   environment when any client may be the first? Consider `tmux -L <per-project or per-hub socket>`, `-f /dev/null`,
   `set-environment -g -u`, `update-environment`, and starting a dedicated server deliberately before use.
3. **Isolation boundary.** One tmux socket per project? Per hub? Per Unix user? The cost: `tmux attach` habits,
   cockpits that list sessions, existing sessions, restarts of the shared server under load.
4. **Detection.** A detector for a polluted server environment, or for a session whose project variables disagree with
   its working directory or registration. Where should it run, and how fast must it fire?
5. **Damage assessment.** How to find out, after the fact, whether any process wrote into another project's
   `.tasks`/`.context` in the window: git status, mtimes, task ids out of sequence, episodic files, audit logs.
6. **Framework side.** Should the framework refuse rather than warn when inherited paths point outside the project?
   What about old vendored copies?
7. **Anything we have not thought of,** including other shared host state with the same shape (a hub runtime dir,
   `/tmp` sockets, the user's crontab, ssh agents).

Sections: **Verdict** · **Root cause (ranked)** · **Remediation (prioritised, concrete)** · **Detection** ·
**Damage assessment** · **What we missed**.
