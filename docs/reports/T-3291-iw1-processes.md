# T-3291 IW-1 — Census of live `termlink register` processes

Measured 2026-09-30 (host .107), strictly read-only: `/proc`, `ps`, `tmux list-panes`,
`tmux capture-pane -p` (one pane), session JSON reads, and source reads. No process was
signalled, no `termlink` subcommand was run.

## Method (exact commands)

```bash
# 1. enumerate: every PID whose argv[0] ends in "termlink" and argv[1] == "register"
ps -eo pid,ppid,etimes,tty,stat,args --no-headers | awk '$6 ~ /termlink$/ && $7=="register"'
# 2. per-pid collection (python over /proc): cmdline, stat (ppid/sid/tty/starttime vs btime),
#    exe readlink, parent chain via /proc/<p>/stat, environ (TMUX*, TERMLINK_*), children via
#    /proc/<p>/task/*/children, fd -> socket inode -> /proc/net/unix path, cgroup
# 3. registry join: json.load of /var/lib/termlink/sessions/*.json and
#    /tmp/termlink-0/sessions/*.json, keyed by .pid; file mtime + heartbeat_at
# 4. tmux join
tmux list-panes -a -F '#{pane_pid} #{session_name} #{session_attached} #{pane_dead} #{session_activity} #{pane_current_command}'
# 5. descendant tree of each register pid (looking for claude / fw below the pane shell)
# 6. dispatch-dir join: /tmp/tl-dispatch/<name>/exit_code
```

Scratch data (rows*.json) lives in the session scratchpad, not in the repo.

## 1. Count and age

**508** live `termlink register` processes (515 `grep` hits incl. non-exact matches).
**502 / 508** have `/proc/<pid>/exe` → `/root/.cargo/bin/termlink (deleted)`. All in state `S`.

| age (etimes) | count |
|---|---|
| < 1h | 35 |
| 1h – 1d | 116 |
| 1 – 3d | 56 |
| 3 – 7d | 93 |
| > 7d | 208 |

Combined RSS: ~2.2 GB for the register processes + ~2.0 GB for their pane `bash` children.

## 2. Classification

| class | count | definition |
|---|---|---|
| **tmux-idle** | **483** | parent = tmux server (pid 2600); pane shell `bash` has **no children** |
| tmux-busy | 9 | tmux pane whose shell has `claude` / `fw` below it |
| systemd-supervised | 4 | cgroup `termlink-{termlink-agent,email-archive,framework-agent,cashweb-integration}.service` (1 of them has a claude below it) |
| other non-tmux | 12 | 2 in `agentic-fleet-cockpit.service` cgroup (`bpmn-fetch-832`, `scratch-t159-pty`, 8.5–8.6d); 6× `t392w-*` (ppid 1, user-1000 session scope, `--name` only, ~0.1d); 3 `sh`-parented (`email-archive`, `scratch-t152b`, `dimitri-home-fix`, 6.5–9.6d); `kristin-recovery` (bash in tmux) |

tmux-idle by age: <1h 30 · 1h–1d 108 · 1–3d 55 · 3–7d 90 · >7d 200.

The 9 busy tmux panes: `t3602-fix`, `claude-master-335512`, `t3593-round4`,
`t3580-r6-review-…`, `verifier-t1654` (all <1h), `claude-master-1238189` (7.7h),
`t1516-onedev` (4.9d), `pen-live-agent` (5.4d), `vr-gatherer-r1` (9.3d).

## 3. Parent chain / tmux

- **492** have ppid **2600** = the single tmux server
  (`tmux new-session -d -s fleet-001-CashWeb-… env -i … claude-fw -c --termlink`, up 9.6d).
  Every one of them is the **pane process** of a detached (`session_attached=0`) tmux
  session named `tl-<name>` — the exact name produced by `spawn_via_tmux`
  (`crates/termlink-cli/src/commands/execution.rs:673-685`). `pane_dead=0` for all; the panes exist.
- tmux `session_activity` age matches process age (>7d: 201, 3–7d: 92, …) — no pane output
  since creation, i.e. nobody has attached or produced output.
- 12 ppid=1: 4 systemd units (legit), 2 cockpit-service leftovers, 6 `t392w-*`.
- Controlling tty: 495 have one (the tmux pane pty), 13 do not (systemd / `--self` / detached).

## 4. Cmdline flags

| flags | count |
|---|---|
| `--name --tags --shell` | 493 |
| `--name` only | 6 (`t392w-*`) |
| `--name --shell` | 3 |
| `--name --tags` | 2 |
| `--identity-key --roles --cap --allowed-commands --shell` (systemd agents) | 3 |
| `--self --identity-key --roles --cap` | 1 (cashweb-integration) |

Tag families: `task:T-N,task-type:<build|inception|specification|review|default|test|refactor>` = **435**;
`master,claude,framework` = **48**; empty = 9.

## 5. Registration files and activity

- **All 508 have a registration JSON + socket**, split across two runtime dirs:
  251 in `/var/lib/termlink/sessions/` (env `TERMLINK_RUNTIME_DIR=/var/lib/termlink`) and
  257 in the legacy **`/tmp/termlink-0/sessions/`** (no `TERMLINK_RUNTIME_DIR` in environ —
  the tmux server was started with `env -i`, and `build_spawn_shell_cmd` only exports the
  var when the *spawner* has it, `execution.rs:580-583`). 7 further `/tmp` JSONs belong to live
  non-register pids.
- **Every JSON mtime < 1h and `heartbeat_at` within the last ~30s**: the T-2230 heartbeat task
  (`session.rs:437-465`) keeps all 508 looking LIVE. Heartbeat freshness is therefore *not*
  evidence of use — it is emitted by the register process itself regardless of whether
  anything is attached or running in the shell.
- Socket mtimes = creation time (no reconnection signal available from mtime).

## 6. Name families (top)

`claude-master-N` 48 · `verifier-tN` 46 · `tl-tN-rN` 15 · `tN-reviewN` 10 · `vrN-gN` 10 ·
`tN-roundN` 9 · `pruef-N` 8 · `wNstN` 8 · `pNstN` 8 · `tN-fix` 7 · `pfN-rN` 7 · `tN-revN` 7 ·
`remN` 6 · `verifierN-tN` 6 · `tNw-N-N` 6 · `worker-tN` 5 · `seq-tN-rN-{review,procasfit}` 5+5 · …
(long tail of per-round orchestration names).

By project (cwd of the pane shell): `/opt/1409-sprind` 186 (verifier-t*, pruef-*, w*st*),
`/opt/999-Agentic-Engineering-Framework` 117, `/opt/055-agentic-fleet-cockpit` 75,
`/opt/050-email-archive` 35, `/opt/termlink` 22, `/opt/1023-…vault` 19, 832/100/025 ≤ 11 each.
The leak is fleet-wide (every consumer project with the vendored framework), not termlink-only.

Only 27 names still have a `/tmp/tl-dispatch/<name>/` dir (24 with `exit_code` = worker
finished, 3 without); the rest had their dispatch dir removed (`fw termlink cleanup` does
`rm -rf $DISPATCH_DIR`) while the session stayed alive.

## 7. Producers

| producer | tags it emits | idle leaked | file:line |
|---|---|---|---|
| `fw termlink spawn` / `fw termlink dispatch` (vendored AEF) | `task:T-N,task-type:X` | **434** | `.agentic-framework/agents/termlink/termlink.sh:279-323` (spawn), `:932-940` (dispatch → `cmd_spawn` then `termlink pty inject "bash $wdir/run.sh …" --enter`) |
| `claude-fw --termlink` | `master,claude,framework` | **46** | `.agentic-framework/bin/claude-fw:60-70` (`claude-master-$$`, `--backend auto --shell`) |
| systemd units | various | 0 (legit) | `/etc/systemd/system/termlink-*.service` |

Neither `/etc/cron.d` nor any script under `scripts/`, `.claude/`, `.context/cron/` calls
`fw termlink cleanup` or kills `tl-*` tmux sessions.

### Why the dispatch path never ends
1. `termlink spawn --shell` runs `tmux new-session -d -s tl-<name> "<termlink> register --name … --shell"`
   (`execution.rs:591-599`, `:673-685`). The register process is the pane; bash is its PTY child.
2. `fw termlink dispatch` **injects** `bash run.sh …` into that interactive shell
   (`termlink.sh:936-940`). `run.sh` ends with `echo "Result: …"` — no `exit`
   (`termlink.sh:~924-928`). When the worker finishes, the interactive bash returns to its prompt
   and waits forever.
3. `cmd_wait` / `cmd_result` only poll `exit_code`; they never tear the session down.
4. `cmd_cleanup` (`termlink.sh:370-495`) skips any worker with `exit_code` (`:382`), calls
   `termlink clean` (removes only *dead* registrations), and closes windows **only on macOS**
   (`:451`, `is_macos`). On Linux there is no tmux `kill-session`. It then `rm -rf`s the dispatch
   dir, erasing the only record linking the name to a finished worker.

### Why claude-fw leaks
`termlink_cleanup` (`claude-fw:84-90`, trap at `:94`) injects `exit` into the shell then runs
`termlink clean`. 46 of 48 `claude-master-<pid>` launchers are dead yet their pane bash is alive,
so the trap either did not run (launcher killed by SIGKILL/tmux kill) or the inject failed.
Even when `exit` lands, see the register gap below.

## 8. Exit mechanism in `termlink register` (the absence)

`cmd_register` (`crates/termlink-cli/src/commands/session.rs:225`), final wait at
**`session.rs:467-493`**:

```rust
tokio::select! {
    _ = server::run_accept_loop(listener, shared_clone) => {}
    _ = tokio::signal::ctrl_c() => { … SIGTERM pty child, remove json/sock … }
}
```

- The PTY read loop runs in a detached `tokio::spawn` (`pty_handle`, `session.rs:420-431`) that is
  **not** a select arm. `read_loop_with_broadcast` returns `Ok(())` on EOF/EIO when the shell exits
  (`crates/termlink-session/src/pty.rs:279`, `:295-298`) — and nothing observes it. **A register
  process outlives its own shell.**
- No `prctl(PR_SET_PDEATHSIG)`, no parent-pid polling, no stdin-EOF watch, no TTL, no idle
  timeout anywhere in `crates/` (grep for `PDEATHSIG|pdeathsig|getppid` = 0 hits).
- Only SIGINT triggers the graceful path. SIGTERM/SIGHUP (e.g. `tmux kill-session`) hit the
  default disposition — the process dies but leaves its JSON/socket behind for `termlink clean`.
- The heartbeat task (`session.rs:444-465`) refreshes `heartbeat_at` every 30s unconditionally,
  so every leaked session is indistinguishable from a live one to any heartbeat-based detector
  (including the T-2239 frozen-husk canary).
- `--self` (`endpoint.rs:112-133`) has the same shape (accept / ctrl_c / heartbeat only).
- By contrast the Rust `termlink dispatch` path does clean up: it SIGTERMs its workers
  (`crates/termlink-cli/src/commands/dispatch.rs:529-537`) — that path is not the leak.

## Root-cause hypothesis (confidence: high, ~85%)

The leak is **structural lifecycle absence**, not a crash: `termlink spawn --shell` creates a
persistent tmux-hosted interactive shell whose lifetime is bound to nothing. The dominant
producer, vendored `fw termlink dispatch` (434/483 idle), runs one-shot workers *inside* that
persistent shell and has no Linux teardown (macOS-only window close, no `tmux kill-session`,
no `exit` at the end of `run.sh`); `claude-fw` accounts for 46 more. `termlink register` itself
has no exit trigger other than SIGINT — it does not exit on shell exit, parent death, or
idleness — and its self-heartbeat makes every husk look live, so no canary fires. ~483 of 508
(95%) are idle and safe-to-reap candidates; 9 tmux + 4 systemd + a handful of others are in use.
Remaining uncertainty: whether a human ever re-attaches to old `tl-*` panes (tmux shows
`session_attached=0` and no activity since creation for all 483), and why claude-fw's EXIT trap
did not land for 46 launchers.
