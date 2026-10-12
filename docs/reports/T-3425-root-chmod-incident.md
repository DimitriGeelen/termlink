# T-3425: `/` lost its execute bits, caused by an agent's `tmux -S` (2026-10-12)

The operator's incident report is `/root/INCIDENT-2026-10-12-boot-failure-root-permissions.md`; it covers the symptom,
the diagnosis and the fix (`chmod 755 /`). This file answers its open question: **what changed `/` to 0644.**

## Cause

At **15:09:51.653Z** this project's agent (session 10c2f139, looking into 055's report about `pen-live-agent`) ran:

```
tmux -S /proc/2908/cwd 2>/dev/null
```

It meant to read the environment of the tmux server with pid 2908. But `-S` names a **socket path**, not a server,
and `/proc/2908/cwd` is a link to that server's working directory, which is `/`. tmux tried to create its socket there
and failed ("Address already in use"). It still ran its socket-permission update on that path, and that update clears
the execute bits when no client is attached. `/` went from 0755 to 0644.

The first permission error in the journal is at **15:09:53Z**, two seconds later
(`systemd-resolved: Failed to stat /etc/hosts: Permission denied`). From then on every non-root process failed, and
the boots that followed could not start D-Bus.

## Reproduction (scratch directory, 2026-10-12, tmux 3.4)

```
(cd victim && exec sleep 30) &  P=$!
stat -c %a victim            # 755
tmux -S /proc/$P/cwd         # "error creating /proc/<P>/cwd (Address already in use)", rc 0
stat -c %a victim            # 644
```

A plain symlink to the directory did **not** reproduce it. The `/proc/<pid>/cwd` form did.

## Ruled out

1. The arc-011 security work (sidecar, OQ-2 E). Its code only sets 0700 or 0600 on its own socket dir and files, and
   none of those paths resolves to `/`.
2. `runme.sh` `install_crontab` (`chmod 644 "$dest"` runs right after a `cp` onto that file).
3. The acceptance run as `testdev` at 14:22Z. It needs `/` to be traversable, and it passed, so `/` was still 0755
   then.

## Recovery checked after the crash

1. `/` is 0755; dbus, lightdm, postgresql and termlink-hub are active.
2. Git: `main` == `origin/main` (097fdabe5). The commit-graph named two commits it could not read; it was rebuilt
   (`git commit-graph write --reachable`), and `git fsck --connectivity-only` is now clean.
3. Every file in the working tree written in the hour before the crash was checked. None is empty or NUL-filled
   except lock and marker files, which are meant to be empty.
4. Units still failed after the reboot. They are not ours, so they are reported only: `fancontrol`, `pen-live-agent`,
   `penelope-cal-sync`, `penelope-cohort-copy-sent`, `penelope-cohort-drift-probe`.

## Lesson

Never pass a path you have not checked to a tool option that **creates or chmods** what it names (`tmux -S`/`-L`,
`--pidfile`, `--log`, `--socket`). A `/proc/<pid>/cwd` or `/proc/<pid>/root` link resolves to a real directory, often
`/`. To read another process's environment use `/proc/<pid>/environ`. For a tmux server, use
`tmux -S <its real socket> show-environment -g`, taking the socket from `ls -l /proc/<pid>/fd`, or don't do it at all.

## Proposed prevention (operator decision)

1. A PreToolUse guard that refuses `tmux -S`/`-L` when the path exists and is not a socket, or is under `/proc`.
2. The host check the operator's report suggests: at boot, alert (or self-heal) if `stat -c %a /` is not `755`.
