# Remote node token: reach a hub without holding its secret (T-3543)

A remote node (another host, a container, a laptop) that only needs the channel rail
should not carry the hub's HMAC secret. Whoever holds `hub.secret` can mint any token,
up to `execute`, which runs shell commands on that hub's sessions. A scoped hub token
gives the node exactly what it needs and nothing more.

## Recipe

1. On the hub host, mint a token. It is written to a 0600 file and never printed:

   ```
   termlink hub token --out /root/node-a.token --scope interact --ttl 604800
   ```

   1a. `--scope` is `observe`, `notify` or `interact` (the default). `control` and
       `execute` are refused: they stay with the holder of the secret.
   1b. `--ttl` is capped by `TERMLINK_MAX_TOKEN_TTL_SECS` (default 7 days). A longer
       request is clamped, with a warning.
   1c. An existing file is not replaced unless you pass `--force`.

2. Copy the file to the node over a channel you already trust (ssh, the provisioning
   tool), keeping mode 600.

3. On the node, add a profile with the token file and no secret:

   ```
   termlink remote profile add hub-a 192.168.10.107:9100 --token-file ~/.termlink/hub-a.token --scope interact
   ```

   `remote profile list` shows the profile's SECRET column as `token`.

4. Use the rail as usual: `termlink channel post <topic> <body> --hub 192.168.10.107:9100`,
   `channel subscribe`, `channel list`, `remote ping hub-a`, `fleet doctor`.

The token is presented as-is in `hub.auth` by every CLI TCP path: `remote *`, the
`channel ... --hub <host:port>` path (including the WebSocket push), `fleet doctor` /
`fleet governor-status` / `substrate`, and `net test`. If a profile carries both a secret
and a `token_file`, the secret wins.

## What interact allows

`channel.post`, `channel.claim/renew/release`, `event.broadcast`, `event.emit_to`,
`session.register_remote` and `session.heartbeat`, plus every observe read
(`channel.list/subscribe/receipts/claims`, `session.discover`, ...). Refused with
"Permission denied ... requires 'control'/'execute'": `command.inject`,
`command.execute`, and `channel.create`. **Create the node's topics on the hub host**
(`termlink channel create <topic>` there) before the node posts to them.

## Rotation and expiry

The node refuses an expired token itself, naming the file, when it expired and the
command to mint a new one, without printing the token. Re-run step 1 with `--force`
before expiry and copy the file again. Nothing on the hub needs restarting.

## Residuals (accepted, not solved here)

1. **No per-topic access control.** An interact token can post to and read every topic
   on that hub, not just the node's own.
2. **Replay within the TTL.** The token is a bearer credential. Anyone who copies the
   file can use it until it expires. Keep the file mode 600 and the TTL short.
3. **Revocation only by expiry or secret rotation.** There is no revocation list.
   Rotating `hub.secret` invalidates every token minted from it, and every client
   holding that secret must be re-keyed (`fleet reauth`). Otherwise a leaked token
   stays valid until its `expires_at`.
4. **Expiry is checked on the node and on the hub.** The node's pre-check is a
   convenience. The hub's validation is what refuses an expired or forged token.

## The fleet hubs (T-3615, operator ruling Q3 = A)

.122, .121 and .132 reach .107 this way. `runme.sh` does all of it, verified on each host:

1. Action 22b mints each host's token on .107 (`scripts/fleet-token-remint.sh`) and installs it at
   `/root/.termlink/secrets/107-hub.token`. It also writes a token-only config home
   (`/root/.termlink/token-only-home`) and adds `token_file` to the host's .107 profile. The proof
   runs with the token-only home, so no secret can be what authenticates: `channel list` succeeds
   and a post to `fleet-token-probe` is accepted.
2. A daily .107 cron (`fleet-token-remint.crontab`) re-mints any token with less than 3 days left.
   It never does a first install. If it fails, the failure goes to
   `.context/working/.fleet-token-remint-canary.log` and onto the needs-attention list.
3. Action 23's telemetry rollup posts through a wrapper that uses the token-only home.
4. Action 24 removes the `secret_file` line and moves the secret to `<file>.t3615-retired`. It runs
   one host at a time, only after that host's decision is recorded in `runme.sh` and its token was
   proven on an earlier run.

## Proof

`TERMLINK_BIN=target/debug/termlink bash tests/hub-token-e2e.sh` starts a throwaway hub
and a throwaway client HOME and checks all of the above (15 assertions).
