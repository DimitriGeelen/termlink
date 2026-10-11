# T-3395 Root inventory (read-only; OP-1 amendment 1, step 3 10.1.5 "Inventory")

Date: 2026-10-11 (UTC ~19:10). Read-only: `ps`, `ss`, `systemctl show/list-*`, `/etc/cron.d` reads, `stat`, `grep` of `runme.sh`. Nothing was started, stopped, or changed. Names and counts only; no secret or file content.

Verdict key: NEEDS ROOT (evidence of a privileged act), DOES NOT NEED ROOT (evidence that nothing privileged is done), UNKNOWN (not established; stated what would settle it). "Runs as root today" is not evidence of need: on this host nearly everything runs as root because it was started that way.

## Headline numbers

| Fact | Value | Source command |
|---|---|---|
| Processes on host | 1,734 | `ps -e --no-headers \| wc -l` |
| Processes owned by root | 1,454 (84%) | `ps -eo user \| sort \| uniq -c` |
| Processes owned by the desktop user (dimitri-mint-dev) | 258 | same |
| Other system users (postgres 6, ollama 3, avahi 2, kernoops 2, others 1 each) | 22 | same |
| Registered TermLink sessions, all uid 0 | 115 | `termlink discover --json` |
| Cron lines in /etc/cron.d, all user root | 241 (230 are termlink-* or agentic-* files) | `cat /etc/cron.d/* \| grep -E '^[0-9*@]' \| awk` on the user field |
| Listening TCP ports | about 33 named ports plus ephemeral ones; privileged (below 1024): 22, 53, 111, 631 | `ss -ltn`; `sysctl net.ipv4.ip_unprivileged_port_start` = 1024 |

Note on the design's "86 process lines" (10.1.5): that figure is not reproduced; the host shows 1,454 root processes. The 86 was an earlier estimate (source not found). The grouping below gives the real structure.

## 1. The hub

| Item | Runs as | Needs root? | Evidence |
|---|---|---|---|
| `termlink-hub.service` (`termlink hub start --tcp 0.0.0.0:9100 --json`) | root (`User=root` in the unit; `ps` shows the pid from the pidfile as root) | DOES NOT NEED ROOT for its network role; root is needed today only because the runtime dir is root's | Port 9100 is above 1024 (`ip_unprivileged_port_start` = 1024), so no privileged bind. `/var/lib/termlink` is `drwx------ root root`; `hub.secret`, `hub.key.pem` are root 0600; `hub.sock` root 0755. Moving it to its own account means chown of that directory only (the PL-021 persistence risk in 10.1.5 "Hub access"). Not tested: whether anything in the hub calls a privileged syscall (UNKNOWN until a run as a test account). |
| `hub.cert.pem`, `hub.secret`, `hub.key.pem` | root files | n/a (data) | `stat -c '%U %a'` |

## 2. systemd units that run as root (named agentic/termlink/fleet)

Source: `systemctl show -p FragmentPath --value <unit>.service`, then `grep ^User=` / `^ExecStart=` in the unit file.

| Unit | Runs as | Needs root? | Evidence |
|---|---|---|---|
| termlink-termlink-agent, termlink-framework-agent, termlink-email-archive, termlink-cashweb-integration, termlink-bpmn-fetch-832 (each `termlink register ...`) | root (`User=root`) | UNKNOWN, probably not | They register a session and wait. The session socket lives in the root-owned runtime dir, which is why they run as root; whether the project directories they act in are writable by another account was not checked. |
| agentic-fleet-cockpit.service (`python3 .../cockpit-server.py`) | root | UNKNOWN | Reads project files under /opt; no privileged port seen in the unit. Needs a check of what it reads and writes. |
| pulse-agent.service (`pulse-agent --enable-host --enable-docker`) | root | NEEDS ROOT (likely) | `--enable-docker` and `--enable-host` metrics collection; docker socket and host stats. Not verified by a run as another user. |
| fw-bus-watcher.service / .path | no `User=` line (system default root) | UNKNOWN | Runs `agents/context/bus-handler.sh` from another project (999-AEF). |
| Others running (from `systemctl list-units --state=running`): accounts-daemon, avahi, bluetooth, containerd, docker, cron, cups, dbus, getty, lightdm, ModemManager, NetworkManager, nordvpnd, ollama, packagekit, polkit, postgresql, rsyslog, ssh, systemd-*, etc. | root or their own users | Host OS services; outside this project | Not part of the agent estate; listed for completeness. A few run under their own user already (postgres, ollama, avahi, polkitd, syslog). |
| cohort-hub, fleet-shadow, memwatch, nas-file-service, nasring20, fan-dashboard, 501-project-register | root | UNKNOWN | Other projects' services; `User=` not individually read. |

## 3. Cron (all /etc/cron.d jobs run as root)

Source: `ls /etc/cron.d`; field 6 of every job line in `/etc/cron.d/*`.

| Group | Count | Needs root? | Evidence |
|---|---|---|---|
| termlink-* canaries and sweeps (about 40 files: *-canary, preflight, heartbeat, presence-sweep, watchdog, fleet-adoption-snapshot, notify-sidecar/wake supervisors, woken-silent-triage) | most of the 230 termlink+agentic lines | DOES NOT NEED ROOT for most; NEEDS ROOT for the ones that read the root-owned hub runtime dir, restart units, or inspect `/proc` of other users | They are scripts in `/opt/termlink/scripts` reading `termlink ...` output and writing `.context/working/*.log`. `substrate-preflight` and the frozen-husk and waker canaries read `/var/lib/termlink` and `/proc/<pid>/cmdline`. Each was not classified one by one: UNKNOWN per job until run as a test account. |
| agentic-audit-<project> (about 25 projects) | one job set per project | DOES NOT NEED ROOT in principle | Runs the framework audit in each project directory; needs write access to that project's `.context`. Root is the owner of those trees today. |
| agentic-learnings-exchange, agentic-pickup, agentic-onedev-sync | few | UNKNOWN | They push to or pull from the OneDev remote and the `framework:pickup` topic; need the ssh key / token of the account that owns them. |
| Non-project: anacron, e2scrub_all, sysstat, timeshift-hourly, t037-linux-share, normalize-nas-video | few | NEEDS ROOT (OS maintenance, timeshift snapshots, filesystem scrub) | OS tools and btrfs/e2fs maintenance. |
| `crontab -l` (root's own): ntfy backup, ntfy health-check, termlink update at 06:00 | 3 | UNKNOWN | `termlink.sh update` replaces `/root/.cargo/bin/termlink`, which needs write access to that path only. |

## 4. `runme.sh` (operator script, /opt/termlink/runme.sh)

Source: `grep -o -E 'systemctl [a-z-]+|/etc/cron\.d|/etc/systemd[a-z/]*|chown|setfacl|apt' /opt/termlink/runme.sh | sort | uniq -c`.

| Privileged act in the script | Occurrences | Needs root? |
|---|---|---|
| `systemctl restart` | 5 | NEEDS ROOT (or a single sudoers/polkit rule for a named unit) |
| `systemctl reload`, `systemctl cat` | 1, 1 | reload needs root; `cat` does not |
| writes to `/etc/cron.d` | 4 | NEEDS ROOT |
| writes under `/etc/systemd/system/termlink...` | 2 | NEEDS ROOT |
| `apt` | 4 | NEEDS ROOT (package install) |
| `chown`, `setfacl` | 0 | no |

The rest of the script (git, framework and canary checks, reaping zombie sessions) does not itself need root, but it shares one root invocation with the above. The split that stage A4 of 10.1.7 asks for is: keep those four acts behind a named helper, run everything else unprivileged.

## 5. The orchestrator, agents, tools

Source: `ps -eo user:12,args`, grouped by command.

| Group (root-owned processes) | Count | Needs root? | Evidence |
|---|---|---|---|
| `termlink register` (agent and shell sessions) | 123 | DOES NOT NEED ROOT, except where the session must write the root-owned runtime dir | These are shell/agent sessions registered with the hub; they run commands in project directories. |
| `termlink pty` / `termlink channel` / `termlink mcp` / `termlink -c` | 17 / 16 / 13 / 8 | DOES NOT NEED ROOT (they are clients of the hub) | Clients using the hub socket or TCP; nothing privileged observed. |
| `claude` (57) and `claude-fw` (4) | 61 | DOES NOT NEED ROOT | The coding agents. They run as root because the session starts as root; transcripts are in `/root/.claude/projects` (root-owned). Moving them needs per-account credentials (10.1.5 "Per-account state", UNKNOWN: whether the harness login can be provisioned per account). |
| `python3` (182): about 97 under /opt/999-Agentic-Engineering-Framework (framework tooling, watchtower, MCP server `framework_mcp_server.py` ×9), 12 under /opt/055-agentic-fleet-cockpit, 3 gunicorn, 3 under /opt/termlink | 182 | DOES NOT NEED ROOT in principle | Framework and project tooling; Watchtower binds ports above 1024 (3000 to 3014). |
| `bash`, `sh`, `sleep`, `npm`, `node`, `tail`, `run.sh` | 145, 38, 39, 37+, 45, 4, 4 | DOES NOT NEED ROOT | Shell glue and node tools of the agents. |
| `sudo` (16) and `su` (8) | 24 | UNKNOWN | Root processes that call `sudo` or `su` mean a user-switch wrapper is in the tree; the purpose was not read. |
| `tmux` servers | 9 | DOES NOT NEED ROOT | Hold PTYs of sessions; they are root's because the launcher is. |
| `chrome`, `headless_shell` (Playwright) | 27, 5 | DOES NOT NEED ROOT, but Chrome-as-root needs `--no-sandbox` | A sandbox flag was not inspected (UNKNOWN). |
| Kernel threads (`kworker`, `nv_*`, `psimon`) | the rest of the root total | n/a (kernel) | `[...]` entries in `ps`. |

## 6. Files and trees owned by root that agents write

Source: `ls -ld`, `stat`.

- `/opt/termlink`, `.context`, `.tasks`, `.context/sidecar` are `root root` with mode 0775 plus an ACL (`+`). Agents write them as root today.
- `/var/lib/termlink` (hub state, secret, cert) is `root 0700`.
- `/root/.claude/projects` (9.9 GB transcripts) is under root's home.

## 7. What the evidence says

1. Privileged acts proper are few: `systemctl restart/reload`, `/etc/cron.d` and `/etc/systemd/system` writes, `apt` (all in `runme.sh`), OS maintenance cron, docker/host metrics (`pulse-agent`), and ownership of `/var/lib/termlink`. No privileged TCP port is used by the project's hub or agents.
2. The great majority of root processes (agents, tools, sessions) have no evidenced need for root. They are root because the launcher and the checkout are.
3. UNKNOWN items to settle before the A0 pilot: per-job classification of the 230 termlink and agentic cron lines; what the 24 `sudo`/`su` processes do; whether the harness login can be provisioned per account; whether Chrome/Playwright runs without a sandbox as root; the `User=` of the other projects' units.
