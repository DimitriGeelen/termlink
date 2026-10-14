# T-3442 — arc-011 P9 clean-host test result

Run: 2026-10-14T20:16Z on dimitrimintdev (third consecutive pass), `bash tests/arc011-clean-host.sh`:
an `alpine:3` container with `--network host`, **no `/opt/termlink` checkout and no Rust**, installing with the
real `install.sh` from a local two-version "release" served on 127.0.0.1 (v1 = the musl local-fast build of
HEAD cfe4f7a26 plus the P9 changes; v2 = the same binary with a trailing marker, so a different checksum that
still runs). Exit 0.

## Prerequisites the clean host needed

Installed in the container the way an operator would: `bash`, and a `ps` that understands `-o` (procps); plus
`curl` or `wget` for `install.sh` (busybox wget sufficed).

## Findings while building it

1. **A hub has an id only when started with `--tcp`.** The id comes with the TLS cert the TCP listener creates;
   without it `termlink sidecar hub-id` reports "states no hub_id" and no sidecar can be configured (FX-2). An
   arc-011 host's hub therefore runs with `--tcp`.
2. **Defect fixed: a restart was declared up on the old process's heartbeat.** `sidecar-supervise.sh` accepted
   any heartbeat younger than 120 s, and right after a restart the previous process's heartbeat qualifies, so
   `start` and the backstop reported success before the new sidecar's sockets existed (status calls then failed,
   reading as a lost message). Now the heartbeat's pid must be the pid running now (the unit's MainPID or the
   detached pid). Fixture 11a in `tests/sidecar-supervise-fixtures.sh`.
3. **Status must not hash the binary in the pure view.** Hashing a large binary on every status call slowed
   status enough to skew the deadline tests; the hash is now taken once per process (at sidecar start) and added
   only on the live socket path.

## Output

```
clean host: alpine:3, release served at http://127.0.0.1:51161 (v1 ee4f9394e932, v2 d260b2d750b6)
PASS 0a no /opt/termlink checkout
PASS 0b no Rust toolchain
PASS 1a installed binary is the v1 artifact
PASS 1b sidecar tools installed from the binary
PASS 1c tools point at the install, not a checkout
PASS 1d release checksum record kept
PASS 2a hub up, id dcd576e1792f9d67
PASS 2b both sidecars started from the installed tools (cron supervises: no systemd)
PASS 3a message reached STORED at agent-b
PASS 3b sidecar runs in production mode
PASS 3c status verifies the binary against the release checksum (SI-27)
PASS 4a upgraded to v2, previous binary kept
PASS 4b after upgrade and restart the stored message is still there
PASS 4c v2 verifies against its own release record
PASS 5a rolled back to the v1 binary
PASS 5b after rollback the stored message is still there
PASS 5c v1 verifies again against the restored record
PASS 6a a SIGKILLed sidecar is restarted by the backstop with a fresh heartbeat
RESULT: 0 failed
clean-host test: PASS
```

## Not covered here

- The systemd path on a clean host (the container has no systemd; it is proven on this host by P3 and W1).
- A real GitHub release (the artifact comes from the release workflow; this test serves an equivalent locally).
- macOS (exit 5's macOS CI item and launchd supervision, R-8.g, are separate).
