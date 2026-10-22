## Verdict

T-3445 did not merely miss two variable names—it **actively broke AEF’s existing provenance defense** in older vendored frameworks. In [`lib/paths.sh:64-66`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/lib/paths.sh#L64-L66), AEF’s T-2289 guard detects leaked `TASKS_DIR`/`CONTEXT_DIR` only when `_FW_PATHS_DERIVED_BY` is present and differs from `PROJECT_ROOT`; when `_FW_PATHS_DERIVED_BY` is empty, `paths.sh` assumes `TASKS_DIR` is a deliberate test-fixture override and preserves it. Because [`tmux_spawn.rs:19-25`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L19-L25) stripped `_FW_PATHS_DERIVED_BY` and `PROJECT_ROOT` from the tmux client while leaving `TASKS_DIR` and `CONTEXT_DIR` in the environment, the first `termlink spawn` after the slice teardown seeded the default tmux server with **provenance-stripped** `TASKS_DIR` and `CONTEXT_DIR`. Every pre-1.8.13 project ( lacking T-4222 at [`lib/paths.sh:74-85`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/lib/paths.sh#L74-L85)) therefore trusted `/opt/1409-sprind/.tasks` unconditionally.

Stop patching deny-lists. A shared mutable daemon seeded by an arbitrary caller’s ambient environment is a **first-caller-wins ambient authority vulnerability**.

---

## Root cause (ranked)

1. **Ambient environment inheritance into a shared, on-demand server (Primary Defect).** [`tmux_new_session`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L63-L67) runs `Command::new("tmux")` without `env_clear()`, letting whichever process happens to run first after a crash seed the host-wide default tmux server’s global environment—and does nothing for manual `tmux`, cron, or framework scripts that start the server outside `tmux_spawn.rs`.
2. **Name-based deny-listing (`PROJECT_SCOPED_ENV`) instead of strict allow-listing.** [`tmux_spawn.rs:19-32`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L19-L32) removes 7 hardcoded names (`PROJECT_ROOT`, `FRAMEWORK_ROOT`, `_FW_PATHS_DERIVED_BY`, `FW_ROUTED_FROM`, `CLAUDE_PROJECT_DIR`, plus 2 sidecar sockets). This is guaranteed to lag new AEF/TermLink variables and stripped the `_FW_PATHS_DERIVED_BY` sentinel while leaving `TASKS_DIR`/`CONTEXT_DIR` behind.
3. **Framework trusting un-opted-in inherited path overrides (`TASKS_DIR`/`CONTEXT_DIR`) and vendored-copy drift.** [`lib/paths.sh:60-66`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/lib/paths.sh#L60-L66) treats an un-sentineled `TASKS_DIR` as an intentional test override by default instead of requiring an explicit opt-in (`FW_TASKS_DIR_OVERRIDE=1`). Meanwhile, stale vendored copies (1.8.10) do not get the R7 authority re-exec ([`bin/fw:379-449`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/bin/fw#L379-L449)), which only redirects git linked worktrees, not standalone consumer repos.
4. **Sharing the default tmux server across projects and human sessions.** Multiplexing all projects onto `/tmp/tmux-0/default` couples failure domains (one `kill-server` or slice teardown drops all projects, as noted in [`tmux_spawn.rs:138-139`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L138-L139)) and mixes human shells with automated dispatches.
5. **Locating project state via mutable environment variables.** Passing `PROJECT_ROOT`, `TASKS_DIR`, and `CONTEXT_DIR` through deep process trees invites cross-project contamination whenever a process crosses a repository boundary.

---

## Remediation (prioritised, concrete)

1. **P0 — Switch `tmux_spawn.rs` to `env_clear()` + minimal allow-list for the tmux client.**
   In [`tmux_new_session_from`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L38-L60), call `base.env_clear()` and pass **only** host-level OS essentials (`HOME`, `USER`, `LOGNAME`, `SHELL`, `TERM`, `LANG`, `LC_ALL`, `TMPDIR`, `XDG_RUNTIME_DIR`, and a sanitized system `PATH` stripped of project-local `.agentic-framework/bin` or `.venv/bin` entries) to the `tmux` client binary. Pass caller session variables exclusively via `new-session -e K=V`—and **only** when the target session does not cross projects (`--workdir` in [`dispatch.rs:240-302`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/dispatch.rs#L240-L302) currently still receives the *caller’s* `PROJECT_ROOT` via `-e` at [`tmux_spawn.rs:51-53`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L51-L53)!).
2. **P0 — Isolate TermLink onto a dedicated, systemd-bootstrapped tmux socket (`-L termlink`).**
   - Do not use the human `default` server, and do not fragment into per-project sockets (which breaks fleet-wide `tmux ls` inspection and leaves N orphan servers). Use **one dedicated socket per TermLink hub** (`tmux -L termlink`, or derived from `TERMLINK_RUNTIME_DIR`). Provide a 2-line shell alias (`tlmux() { tmux -L termlink "$@"; }`) for operator ergonomics.
   - Pre-start the server via a systemd unit (`termlink-tmux.service`) with `env -i HOME=... USER=... PATH=/usr/local/bin:/usr/bin:/bin tmux -L termlink -f /etc/termlink/tmux.conf start-server \; set-option -g update-environment "" \; set-option -g exit-empty off`. Setting `update-environment ""` and `exit-empty off` guarantees the server never dies when sessions hit zero and never absorbs variables from attaching clients.
3. **P1 — Fix the `background` backend and `tl-claude.sh` env leaks.**
   - [`detached_command`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/execution.rs#L747-L757) (`--backend background`, used by [`execution.rs:760`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/execution.rs#L760) and [`dispatch.rs:999`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/dispatch.rs#L999)) currently removes only `LAUNCH_SCOPED_ENV`! A cross-project background dispatch leaks `PROJECT_ROOT`, `TASKS_DIR`, `CONTEXT_DIR`, and `TERMLINK_AGENT_ID` directly into the child. Apply the same env scrubbing there.
   - In [`scripts/tl-claude.sh:171,249`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/scripts/tl-claude.sh#L171), `env -u PROJECT_ROOT` must also unset `FRAMEWORK_ROOT`, `TASKS_DIR`, `CONTEXT_DIR`, `_FW_PATHS_DERIVED_BY`, and `CLAUDE_PROJECT_DIR`.
4. **P1 — Harden AEF [`lib/paths.sh`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/lib/paths.sh#L64-L90) and upgrade vendored copies.**
   - Invert the default: ignore/unset inherited `TASKS_DIR` and `CONTEXT_DIR` **unconditionally** unless `FW_TASKS_DIR_OVERRIDE=1` is explicitly set (update `tests/unit/create_task.bats` to export `FW_TASKS_DIR_OVERRIDE=1`).
   - When `TASKS_DIR`/`CONTEXT_DIR` points outside canonical `PROJECT_ROOT` (`pwd -P`) without `FW_TASKS_DIR_OVERRIDE=1`, **refuse (`exit 2`)** on mutating commands rather than warning, and fix the T-4222 check ([`lib/paths.sh:79`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/lib/paths.sh#L79)) which currently misses projects that have `.tasks/` but no `.framework.yaml`.
   - Extend `_fw_reexec_authority` ([`bin/fw:379`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/bin/fw#L379)) or run a host-wide `fw upgrade` sweep so no vendored copy stays `< 1.8.14`.

---

## Detection

1. **Synchronous pre/post-spawn guard in [`tmux_spawn.rs`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L63) (<50ms, blocks at T=0):** Immediately after `tmux new-session`, run `tmux show-environment -g`. If any key matches `^(PROJECT_ROOT|FRAMEWORK_ROOT|TASKS_DIR|CONTEXT_DIR|_FW_.*|FW_.*|CLAUDE_.*|TERMLINK_.*)$`, immediately execute `tmux set-environment -g -u <KEY>`, log a high-severity alert, and fail the spawn if un-clearable.
2. **Registration-time boundary check in [`Registration::new`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-session/src/registration.rs#L312-L350) (fires on every `termlink register`):** Compare canonical `std::env::current_dir()` (`metadata.cwd`) against `PROJECT_ROOT`, `TASKS_DIR`, and `CONTEXT_DIR`. Record any mismatch in `SessionMetadata` and refuse registration (or emit a `session.env_poisoned` hub event).
3. **`termlink doctor` check in [`crates/termlink-doctor/src/lib.rs`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-doctor/src/lib.rs#L119):** Add a `tmux_global_env` check (and `/proc/<session_pid>/environ` vs `session.metadata.cwd` audit across all live sessions in [`lib.rs:150-165`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-doctor/src/lib.rs#L150-L165)). With `--fix`, run `tmux set-environment -g -u` on offending keys.

---

## Damage assessment

Run an automated forensic sweep across `/opt/1409-sprind` and all sibling project directories for the window **2026-10-22 13:39–14:55** (plus any still-running sessions spawned in that window):

1. **Still-poisoned live processes (check first—bleeding may still be active!):** Clearing `tmux` global env at 14:55 did **not** clean sessions spawned between 13:39 and 14:55. Scan `/proc/[0-9]*/environ` across the host for `1409-sprind` where `/proc/<pid>/cwd` is not `/opt/1409-sprind`.
2. **Filesystem `mtime`/`ctime` in `/opt/1409-sprind/{.tasks,.context}`:** Run `find /opt/1409-sprind/.tasks /opt/1409-sprind/.context -newermt '2026-10-22 13:39:00' ! -newermt '2026-10-22 15:30:00' -printf '%TY-%Tm-%Td %TH:%TM:%TS %p\n'`. Inspect `.context/working/focus*.yaml`, `.context/episodic/`, and `.tasks/active/`.
3. **Git history & working tree:** In `/opt/1409-sprind`, check `git status --ignored`, `git diff`, and `git log --since='2026-10-22 13:39:00' --stat --all` (including `.claude/worktrees/*` and `.termlink/dispatch-manifest.json`) for commits or task files referencing foreign component paths or unfamiliar task ID sequences.
4. **Sequence & cross-repo task ID gaps:** Compare allocated task IDs in `/opt/1409-sprind/.tasks` against other projects’ git commits between 13:39–14:55 (a foreign project on 1.8.10 running `fw task create` or `fw work-on` would allocate or mutate task files inside `1409-sprind/.tasks` while committing code in its own repo).

---

## What we missed

1. **Uncleaned variables still leaking into the tmux server today:**
   - `TERMLINK_AGENT_ID`, `TERMLINK_IDENTITY_FILE`, `TERMLINK_IDENTITY_DIR`: [`registration.rs:54-75`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-session/src/registration.rs#L54-L75) resolves a session’s cryptographic ed25519 identity from `TERMLINK_AGENT_ID`! If an agent with `TERMLINK_AGENT_ID` set starts the tmux server, **every subsequent session on the host inherits that agent’s cryptographic identity**, impersonating it on `channel.post` and DM rails.
   - `FW_SESSION_SCOPED_FOCUS` and `FW_FOCUS_SESSION_KEY` ([`lib/paths.sh:208-220`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/.agentic-framework/lib/paths.sh#L208-L220)): if leaked, all sessions write focus to the same `focus.<key>.yaml`.
   - `PATH` prepended with a project’s `.agentic-framework/bin` or venv, `CARGO_TARGET_DIR` ([`dispatch.rs:256`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/dispatch.rs#L256)), `GIT_DIR`/`GIT_WORK_TREE`, and `TERMLINK_RUNTIME_DIR`.
2. **Cross-project `--workdir` poisoning inside `tmux_spawn.rs`:** Lines [`51-53`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/tmux_spawn.rs#L51-L53) forward the *caller’s* `PROJECT_SCOPED_ENV` via `-e K=V` into the new session even when `termlink dispatch --workdir /opt/other` or `termlink spawn` targets a different project.
3. **Other shared single-UID host state with the same blast radius:**
   - `~/.claude/projects/<encoded-cwd>` transcripts and `~/.claude.json` lockfiles shared across all root agents.
   - `$HOME/.termlink/identity.key` fallback ([`registration.rs:74`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-session/src/registration.rs#L74)) and `~/.termlink/offline-queue.sqlite` ([`substrate.rs:257`](file:///tmp/tl-dispatch/rv-antigravity-3549-8bc341de78a7/tree/crates/termlink-cli/src/commands/substrate.rs#L257))—one SQLite queue shared across all projects on the host.
   - Shared `SSH_AUTH_SOCK` / `git` credential helpers and `/etc/cron.d/termlink-agent-*` (`tl-claude.sh:389`).
