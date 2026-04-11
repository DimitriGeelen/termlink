# TermLink Hub — runtime dir migration (`/tmp/termlink-0` → `/var/lib/termlink`)

**Task:** T-935 (from T-930 decomposition)
**When to read this:** Only if this host previously ran `termlink hub start`
from an interactive shell or from a wrapper script (as opposed to the
systemd unit shipped by T-931).

## Why it moved

Before T-931, the hub ran from whatever shell launched it and wrote its
state to `${TERMLINK_RUNTIME_DIR:-/tmp/termlink-0}/`. On Linux, `/tmp/`
is tmpfs and wipes on every reboot, so:

- `hub.secret` (HMAC auth) was regenerated every boot → cross-host
  clients needed a fresh secret after every restart.
- `hub.cert.pem` + `hub.key.pem` (TLS) were regenerated every boot →
  TOFU-pinned clients broke.
- `hub.sock` was recreated every boot — fine for local clients, but a
  signal of the whole "nothing persists" pattern.

T-931 installed a systemd unit (`.context/systemd/termlink-hub.service`)
with `Environment=TERMLINK_RUNTIME_DIR=/var/lib/termlink` and
`StateDirectory=termlink`, so the hub now writes to `/var/lib/termlink/`.
T-933 made `hub.secret` persist-if-present, so a restart no longer
rotates it.

## How to tell if you need to migrate

Both conditions:

1. `systemctl is-active termlink-hub.service` → `active`
2. `ls /tmp/termlink-0/` → directory exists and is non-empty

If the systemd unit is running but `/tmp/termlink-0/` still has cert / key
/ secret / sessions files from before the migration, those files are
stale. The hub no longer reads from them. They occupy disk and can
confuse anyone looking for hub state.

## Migration command

One line, safe to re-run, no service interruption:

```
sudo rm -rf /tmp/termlink-0
```

That's the whole migration. There is nothing to back up:

- `hub.secret` at `/tmp/termlink-0/hub.secret` was rotated out of
  validity when the systemd unit first started with the new runtime
  dir. No client trusts it anymore.
- `hub.cert.pem` + `hub.key.pem` were likewise rotated. First-time
  clients will re-accept the new cert via TOFU.
- `sessions/` contained registration JSON for sessions that registered
  against the old socket path. Those sessions lost hub-side
  registration the moment systemd took over — the processes are still
  running (see `ps auxf | grep termlink register`) but they're
  orphaned at the hub level. They need to be killed and re-spawned to
  re-register against the new hub socket.

## Verifying the migration

```
sudo systemctl is-active termlink-hub.service   # → active
sudo ls /tmp/termlink-0/ 2>&1                   # → No such file or directory
TERMLINK_RUNTIME_DIR=/var/lib/termlink termlink doctor
```

Doctor should show:

```
✓ runtime_dir: /var/lib/termlink
✓ hub: running (PID ...), responding
✓ ufw_listener: ufw allows 9100/tcp — listener present
```

If `/tmp/termlink-0` reappears after deletion, something is still writing
there — check for rogue `termlink hub start` processes
(`ps auxf | grep "termlink hub start"`) and stop them, then verify the
systemd unit is the only hub running.

## If you have remote clients cached against the old secret

You do not. When the systemd unit started for the first time, the old
`/tmp/termlink-0/hub.secret` was already invalidated. Anyone still
trying to auth with it is getting `-32010 Token validation failed` and
needs the current `/var/lib/termlink/hub.secret` value regardless of
whether you run the migration command above.

## References

- T-930 — Inception: hub supervisor + TCP-default policy decision
- T-931 — systemd unit + installer
- T-933 — secret persist-if-present
- `.context/systemd/termlink-hub.service` — the live unit
- `.context/systemd/install.sh` — install / uninstall helper
