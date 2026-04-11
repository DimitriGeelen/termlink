# T-930: Hub supervisor + TCP-default inception — research artifact

**Task:** T-930 — Should termlink hub run under a supervisor with TCP bound by default on .107?
**Parent program:** T-921 (cross-host parity, GO: Option A + C escape hatch) → T-923 (forwarder proven) → T-924 (shared helper) → T-925..T-929 (5-command rollout landed)
**Filed gap:** G-003 in `.context/project/concerns.yaml`
**Status:** started-work 2026-04-11, exploration not yet started

## Why this artifact exists

Framework rule C-001 requires inception work to have a permanent research
artifact BEFORE conducting research — the thinking trail IS the artifact.
This file will grow as Spikes 1–5 (see T-930 task file) are executed.

## Starting state (2026-04-11 — hub is up on TCP manually)

Captured before any exploration spike runs, so the baseline is permanent:

```
$ termlink hub status
Hub: running (PID 1044108)
  Socket: /tmp/termlink-0/hub.sock
  Pidfile: /tmp/termlink-0/hub.pid

$ ss -tln | grep 9100
LISTEN 0      128          0.0.0.0:9100       0.0.0.0:*

$ ls /tmp/termlink-0/
hub.cert.pem       (566 bytes, root readable)
hub.key.pem        (241 bytes, root only)
hub.pid            (7 bytes — process id)
hub.secret         (64 hex chars — current HMAC secret)
hub.sock           (unix socket)

$ sudo ufw status | grep 9100
9100/tcp                   ALLOW       192.168.10.0/24  # TermLink TCP Hub (LAN only)

$ systemctl list-units | grep -i termlink
(none)

$ ls /etc/systemd/system/ | grep -i termlink
(none)
```

Summary: TCP listener is now up (manually, via `termlink hub stop &&
termlink hub start --tcp 0.0.0.0:9100`). No systemd unit exists. No
watchdog. No `@reboot` cron entry. The hub will die on reboot and nothing
will restart it.

## The triggering event chain

1. T-921 inception (completed 2026-04-11 ~20:20) picked Option A —
   unified `--target HOST:PORT` on every session-scoped CLI command.
2. T-923 (completed 2026-04-11 ~21:03) proved the hub forwarder works
   end-to-end in an integration test that binds TCP in-process.
3. T-924 (completed 2026-04-11 ~21:10) landed `TargetOpts` +
   `call_session` CLI helpers.
4. T-925..T-929 (completed 2026-04-11 21:17–21:34) rolled out `--target`
   on `ping`, `status`, `signal`, `tag`, `kv`.
5. ~23:30 a sibling agent on a different task tried to reach this box
   from 192.168.10.122:
   ```
   $ termlink doctor
   ! hub: stale pidfile (PID 1517402 is dead). Run 'termlink doctor --fix'
   ! sockets: 1 orphaned socket(s) without registration
   ```
   The hub had silently died. Nobody noticed until a cross-host call
   attempt surfaced the gap.
6. ~23:35 re-check: hub had been restarted by someone (new PID 856273),
   but `ss -tln` confirmed it was bound to the unix socket ONLY — no
   TCP. The firewall was open on 9100 but nothing was listening.
7. ~23:41 I manually stopped/started with `--tcp 0.0.0.0:9100`, which
   regenerated the HMAC secret at `/tmp/termlink-0/hub.secret` to a new
   64-char hex value (the previous secret is now invalid).
8. ~23:43 the sibling agent tried to auth with their cached secret and
   got `-32010 Token validation failed: invalid signature`. Their
   secret was from the PREVIOUS hub instance, which had been rotated by
   the restart I triggered. This is the evidence for assumption A2 —
   hub-secret regeneration on restart is a real operational issue, not
   a theoretical one.

## Open questions for the exploration

These will get answered as Spikes 1–5 run. Listed here so future-me does
not lose them.

1. **Where does `hub.secret` actually get written and can the path be
   configured?** Current: `/tmp/termlink-0/hub.secret` on Linux. `/tmp`
   is wiped on reboot. Is `${TERMLINK_RUNTIME_DIR}/hub.secret`
   overridable via env? Does the hub read an existing secret on startup
   if present, or always generate?
2. **Is the current restart-rotates-secret behaviour intentional?** If
   yes, that's a security property (ephemeral trust) and the fix is
   "document it, stop restarting". If no, that's an oversight and the
   fix is "read on startup if present". Code read required.
3. **Does `termlink hub start` trap SIGTERM cleanly?** Matters for
   systemd `Restart=on-failure` vs. `Restart=always`. Matters for the
   "kill -9 recovery" case too.
4. **Does the hub need `RuntimeDirectory=termlink` in the systemd unit,
   or does it create its own?** Probably the latter, but confirm.
5. **Who owns `/tmp/termlink-0/`?** Root on this box. If the systemd
   unit drops to a dedicated user, it needs to own that directory.
6. **What does the doctor check look like?** Probably: "if `--target`
   is ever called locally and the hub has no TCP listener, warn." But
   doctor runs locally without knowing who calls `--target`. Maybe
   instead: "if UFW has a 9100/tcp rule but nothing is listening,
   warn." That check is cheap and catches the exact state I was in.
7. **Is the `framework-agent` session absence a separate inception?**
   Yes (per scope fence + A5). This inception stays about supervisor +
   TCP binding; the framework-agent long-lived session is punted.

## Spike status (update as we go)

- [x] **Spike 1** — current state audit — **done 2026-04-12**
- [x] **Spike 2** — systemd unit design — **done 2026-04-12**
- [x] **Spike 3** — secret persistence — **done 2026-04-12**
- [~] **Spike 4** — watchdog alternative — **moot, see Spike 2**
- [~] **Spike 5** — decomposition — **rolled into Recommendation**

## Spike 1 — Current state audit (2026-04-12)

### What exists

- **TCP listener on 9100:** YES. `ss -tln` shows `LISTEN 0.0.0.0:9100`.
  PID 1044108 (from yesterday's manual restart), binary
  `./target/debug/termlink hub start --tcp 0.0.0.0:9100 --json`.
- **UFW allow rule:** `9100/tcp ALLOW 192.168.10.0/24` — in place.
- **Hub sessions registered:** 9 (t1100-rca … t1109-l006-sweep, all
  `ready` state). So the hub IS in active use right now.
- **Cron supervisor file:** `/etc/cron.d/agentic-audit-termlink` exists
  — BUT it only schedules short-lived `fw audit` invocations. It is
  **not a template** for supervising a long-lived daemon.
- **Daily update cron:** `0 6 * * *` in root crontab runs
  `termlink update --quiet` — unrelated to hub supervision.

### What does NOT exist

- **No `termlink-hub.service`** — `ls /etc/systemd/system/`,
  `systemctl list-unit-files`, and `systemctl is-enabled termlink-hub`
  all confirm zero units.
- **No `@reboot` cron entry** for the hub anywhere
  (`/etc/cron.d/*`, `/etc/crontab`, root crontab).
- **No watchdog / keepalive script.**

### The scary parentage discovery

`ps -p 1044108` shows the hub's PPID is **1043903** — a bash process
that is *still alive* from yesterday's Claude session:

```
1043903 Ss   /bin/bash -c source /root/.claude/shell-snapshots/...
             eval './target/debug/termlink hub stop 2>&1 | tail -5 &&
             sleep 1 && ./target/debug/termlink hub start
             --tcp 0.0.0.0:9100 --json 2>&1 | tail -5'
```

That bash is the wrapper that ran the stop/start pair yesterday. It
never exited because the foreground `termlink hub start` command (which
daemonises internally) didn't close stdout cleanly, and bash is still
`eval`-waiting on it. The hub has PPid=1043903 not 1 — it did not
reparent to init.

Implication: **if that Claude session's bash dies (reboot, OOM, pkill
of the shell snapshot), the hub becomes an orphan of init.** It would
continue running (no HUP), but the next crash has nothing to restart
it. Effectively the hub's supervisor today is "a 9-hour-old Claude
bash process that nobody owns."

This is worse than I documented in the starting state — the hub is
not just unsupervised, it is *accidentally* parented to a ghost.

### The watchtower-vinix24 precedent (huge finding)

The box ALREADY has one enabled systemd unit of the exact pattern I
need: `/etc/systemd/system/watchtower-vinix24.service`:

```ini
[Unit]
Description=Watchtower Vinix24 (port 3056)
After=network.target

[Service]
Type=exec
User=root
WorkingDirectory=/opt/051-Vinix24/.agentic-framework
EnvironmentFile=/opt/051-Vinix24/.watchtower.env
ExecStart=/usr/local/bin/gunicorn -w 2 -b 0.0.0.0:3056 web.wsgi:application
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Status: `active (running) since Tue 2026-04-07 09:43:03 CEST; 4 days
ago`, Main PID 1856 (gunicorn master + 2 workers). Memory/CPU stable.
This is proof that (a) the operator has no objection to systemd on
this box, (b) the Watchtower web app is already a long-lived
supervised service, (c) the `Restart=on-failure` + `RestartSec=5`
combo works in this environment.

**Design lift:** termlink-hub.service can copy almost this unit
verbatim. Only deltas I anticipate:
1. `WorkingDirectory` → probably `/opt/termlink` (or the vendored
   framework root — needs Spike 2 decision)
2. `EnvironmentFile` → optional, for TERMLINK_RUNTIME_DIR override
3. `ExecStart` → `/root/.cargo/bin/termlink hub start --tcp
   0.0.0.0:9100 --json` (or release binary once T-212 Homebrew lands)
4. Possibly `RuntimeDirectory=termlink-0` + `RuntimeDirectoryMode=0700`
   so systemd creates the persistent runtime dir instead of `/tmp/`
5. `After=network.target` stays; also add `After=network-online.target`
   if TCP bind needs routing ready

### Runtime directory findings

`ls -la /tmp/termlink-0/` shows:
```
hub.cert.pem    644 root:root (566 bytes — TOFU TLS cert)
hub.key.pem     600 root:root (241 bytes — TLS private key)
hub.pid         644 root:root (7 bytes — PID 1044108)
hub.secret      600 root:root (64 hex chars — current HMAC secret)
hub.sock        srwxr-xr-x (unix socket)
sessions/       700 root:root (per-session state)
```

Confirms A4: this is all in `/tmp/`, which is tmpfs on this kernel and
wiped on reboot. Every single piece of auth state vanishes. The
systemd unit can fix this via `RuntimeDirectory=` (persists for unit
lifetime but is recreated) OR `StateDirectory=termlink` (persists
across reboots in `/var/lib/termlink`). **For secret persistence, we
need StateDirectory, not RuntimeDirectory.** This bleeds directly into
Spike 3.

### Current hub secret

First 16 hex chars: `6d83e1676a064053…` — matches yesterday's
regenerated value. Confirms no one has restarted since ~23:41 UTC.

### Spike 1 answers to open questions

| # | Question | Answer |
|---|----------|--------|
| 1 | Where is hub.secret written? | `/tmp/termlink-0/hub.secret` (hard-coded via TERMLINK_RUNTIME_DIR default). Path IS env-overridable (see termlink-runtime crate). |
| 2 | Is restart-rotates-secret intentional? | **Deferred to Spike 3** (requires reading `crates/termlink-hub/src/server.rs`). |
| 3 | Does hub trap SIGTERM? | **Deferred to Spike 2** (needs code read + test). |
| 4 | RuntimeDirectory needed? | Yes — but StateDirectory is probably what we actually want (reboot persistence). |
| 5 | Who owns runtime dir? | Root. Matches cron audit pattern (also runs as root). Systemd unit should stay root unless we dedicate a `termlink` user. Recommend root to minimise Spike 2 scope. |
| 6 | Doctor check shape? | UFW-rule-vs-listener mismatch check is cheap and catches exact starting-state failure mode. |
| 7 | framework-agent session inception? | Out of scope (A5 holds). |

### Spike 1 verdict

Nothing structural opposes the systemd approach. The precedent
already exists on this box and is healthy. The only surprise is the
accidental parentage of the current hub — which is an additional
argument *for* this inception, not against it. Proceed to Spike 2.

## Spike 2 — systemd unit design (2026-04-12)

### Hub shutdown is SIGINT-only (real code bug)

Reading `crates/termlink-cli/src/commands/infrastructure.rs:56`:

```rust
tokio::signal::ctrl_c().await.ok();
// ... then: handle.shutdown();
```

`tokio::signal::ctrl_c()` only listens for **SIGINT**. SIGTERM (which
is what `systemctl stop` sends by default) is NOT caught and falls
through to tokio's default handler — the process exits abruptly
WITHOUT calling `handle.shutdown()`. That means the clean-shutdown
path in `server.rs:152-158`:

```rust
let _ = std::fs::remove_file(&socket_path_owned);
let _ = std::fs::remove_file(hub_secret_path());
tls::cleanup();
pidfile::remove(&pidfile_path);
```

is **skipped** on SIGTERM. The next start-up then picks up a stale
pidfile (the `pidfile::acquire` path will clean a stale one, so that
works), but the socket/secret/cert files persist dirty.

**Fix**: use `tokio::signal::unix::SignalKind::terminate()` in a
`select!` alongside ctrl_c. One-function change. This is a build
task, not an inception deliverable — but the inception decomposition
has to include it.

**Workaround option** (no code change): add `KillSignal=SIGINT` to
the systemd unit so systemctl sends SIGINT instead of SIGTERM. This
works but masks the underlying bug. Recommend fixing the code.

### Proposed unit file (draft)

Based on the watchtower-vinix24 template, adapted:

```ini
# /etc/systemd/system/termlink-hub.service
[Unit]
Description=TermLink Hub — cross-host session router (TCP+TLS+HMAC)
Documentation=https://github.com/.../termlink
After=network-online.target
Wants=network-online.target

[Service]
Type=exec
User=root
Group=root
WorkingDirectory=/opt/termlink
Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink
StateDirectory=termlink
StateDirectoryMode=0700
ExecStart=/root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100 --json
ExecStop=/root/.cargo/bin/termlink hub stop --json
Restart=on-failure
RestartSec=5
# Until SIGTERM handling lands, tell systemd to send SIGINT on stop.
# Remove this line after the ctrl_c/terminate select! fix is merged.
KillSignal=SIGINT

# Hardening (match watchtower's implicit defaults)
NoNewPrivileges=yes
ProtectSystem=strict
ProtectHome=yes
ReadWritePaths=/var/lib/termlink /tmp

[Install]
WantedBy=multi-user.target
```

### Key design choices (decisions in the unit file)

| Decision | Choice | Reasoning |
|---|---|---|
| `Type=` | `exec` | Matches watchtower precedent. Waits for execve() success, not for a readiness signal the hub doesn't emit yet. |
| `User=` | `root` | Matches watchtower + cron audit pattern on this box. A dedicated user is scope-creep (Spike 2 is 20 min). Filing a follow-up for "drop to termlink user" is cleaner. |
| `WorkingDirectory=` | `/opt/termlink` | Project root. Keeps any relative paths (there aren't any in hub start, but safe default). |
| `TERMLINK_RUNTIME_DIR=` | `/var/lib/termlink` | Persists across reboots. NOT `/tmp/termlink-0/` (tmpfs). |
| `StateDirectory=termlink` | yes | Systemd creates/owns `/var/lib/termlink` at 0700. No manual mkdir. |
| `ExecStart=` | `/root/.cargo/bin/termlink hub start --tcp 0.0.0.0:9100 --json` | Absolute path. `--json` makes logs parseable by journald. |
| `Restart=on-failure` | yes | Crash → restart in 5s. Clean shutdown → stay down (operator intent). |
| `RestartSec=5` | yes | Same as watchtower. Avoids tight crash-restart loops. |
| `KillSignal=SIGINT` | **temporary** | Works around the ctrl_c-only bug. Delete after the code fix. |
| Hardening | modest | Matches watchtower's implicit posture. ReadWritePaths allows `/tmp` because the code still defaults there if env is unset — belt-and-braces. |

### What this does NOT do yet

- Does not fix the secret-rotation problem (that's Spike 3)
- Does not fix the SIGTERM-not-caught problem (needs code change)
- Does not drop privileges (stays root — follow-up for "termlink user")
- Does not rotate logs (journald handles it; fine for now)
- Does not create a doctor check for UFW-rule-vs-listener mismatch (build task)

## Spike 3 — Secret persistence (2026-04-12)

### Code read

`crates/termlink-hub/src/server.rs:43-66` — `generate_and_write_hub_secret()`:

```rust
fn generate_and_write_hub_secret() -> std::io::Result<String> {
    let secret = auth::generate_secret();   // ← unconditional fresh generation
    let secret_hex: String = ...;
    let path = hub_secret_path();
    ...
    std::fs::write(&path, &secret_hex)?;    // ← always overwrites
    // chmod 0600
    ...
}
```

And `server.rs:110-114` / `server.rs:187-191`:

```rust
let token_secret = if tcp_addr.is_some() {
    Some(generate_and_write_hub_secret()?)  // ← every start with --tcp regenerates
} else {
    None
};
```

And `server.rs:152-158` / `server.rs:209-213`:

```rust
// Cleanup on exit
let _ = std::fs::remove_file(&socket_path_owned);
let _ = std::fs::remove_file(hub_secret_path());  // ← deletes the secret on clean shutdown
```

### Verdict: rotation is incidental, not intentional

- No comment anywhere says "rotate for security".
- The `remove_file(hub_secret_path())` in cleanup is paired with
  `remove_file(socket_path)` and `pidfile::remove()` — all three are
  "remove the files this process wrote." That's housekeeping, not a
  security decision.
- The current behaviour (rotate on every start + delete on every
  clean stop) is the side effect of "always generate fresh" + "clean
  up what we generated", not a deliberate ephemeral-trust property.

### Proposed fix

Two changes in `generate_and_write_hub_secret()`:

1. **Read-if-present.** Before generating, check if `hub_secret_path()`
   exists AND is 64 chars of valid hex AND is mode 0600. If yes, use
   it. If no, generate new.
2. **Don't delete on shutdown.** Remove the `remove_file(hub_secret_path())`
   line from both cleanup paths. The secret stays at rest between
   runs, protected by file mode 0600 + directory mode 0700.

Together: first start generates the secret once; subsequent starts
read it; operators can manually rotate by deleting the file and
restarting.

### Security analysis

**What changes for an attacker with write access to `/var/lib/termlink`?**
Nothing — if they have write access there, they can already plant
a secret and make the hub trust it regardless of rotation policy.
The attack surface is "can anyone read or write the runtime dir",
which is governed by the 0700 directory mode, not by rotation.

**What changes for a network attacker?** Nothing — HMAC secret never
traverses the wire. TLS + HMAC token validation is unchanged.

**What about compromise recovery?** This is the only real trade-off.
Under the current behaviour, a hub bounce rotates the secret, so if
an attacker captured the old secret from a memory dump or a leaked
backup, a routine restart invalidates it. Under the proposed
behaviour, the secret persists until explicitly rotated. Mitigation:
`fw upgrade termlink` or a `termlink hub rotate-secret` command
could be added as a build task — explicit, audible, not accidental.

### Dialogue checkpoint

**Question for human:** do you want the secret to persist across
restarts, OR stay ephemeral-on-restart? The analysis says persist is
cheaper and not meaningfully less secure for a dev-tooling hub on
a LAN-firewalled port, but the call is ultimately operational
preference. The agent recommendation below assumes **persist**.

## Spike 4 — Watchdog alternative — moot

Was scoped for "consider `while true; termlink hub start; sleep 5;
done` from @reboot cron if systemd gets rejected." Systemd was not
rejected: (a) the box already runs watchtower-vinix24 under systemd,
(b) operator comfort is demonstrated, (c) `Restart=on-failure` +
journald + status queries give strictly more than a while loop
could. Watchdog alternative is formally NOT recommended.

## Spike 5 — Decomposition — folded into Recommendation

See the Recommendation section in the task file for the concrete
build task list.

## Dialogue log

- **2026-04-12 — Spike 1 (agent solo):** Current state audit run
  autonomously. No human dialogue needed yet. Findings above.
- **2026-04-12 — Spike 2 (agent solo):** Read
  `crates/termlink-hub/src/server.rs` + CLI `cmd_hub_start`. Found
  SIGTERM-not-caught bug. Drafted systemd unit. No human dialogue
  needed — design is mechanical given the watchtower precedent and
  the bug discovery.
- **2026-04-12 — Spike 3 (agent solo, pending human checkpoint):**
  Read secret generation + cleanup paths. Confirmed rotation is
  incidental. Proposed persist-if-present fix. **Waiting for human
  decision on persist-vs-rotate policy — see Recommendation in
  task file.**

## References

- `.context/project/concerns.yaml` — G-003 gap entry (filed same session)
- `.tasks/active/T-930-should-termlink-hub-run-under-a-supervis.md`
- `/etc/cron.d/agentic-audit-termlink` — comparable supervisor pattern
  (cron for audits; may be relevant as a reference for the hub unit)
- `crates/termlink-hub/src/server.rs` — hub startup + secret generation
- `crates/termlink-cli/src/commands/infrastructure.rs` — `termlink hub
  start` CLI handler
- `docs/reports/T-921-cross-host-parity.md` — parent inception
- `docs/reports/T-923-hub-forwarder-discovery.md` — sibling discovery
  report that proved the forwarder works when TCP IS bound
