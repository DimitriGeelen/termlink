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

- [ ] **Spike 1** — current state audit (20 min) — **not started**
- [ ] **Spike 2** — systemd unit design (20 min) — **not started**
- [ ] **Spike 3** — secret persistence (20 min) — **not started**
- [ ] **Spike 4** — watchdog alternative (15 min) — **not started**
- [ ] **Spike 5** — decomposition (15 min) — **not started**

## Dialogue log

Empty — conducted zero research dialogue yet. This artifact was created
pre-exploration per framework rule C-001. Next session will fill in the
spike results and the decision.

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
