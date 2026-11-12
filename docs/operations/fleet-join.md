# Joining a device to a hub: `termlink fleet join` (T-3749)

One command makes a new device (a laptop, a new container) a working client of a hub.
It replaces carrying messages by hand. Origin: ring20 T-2657. The operator's Mac
could not reach the `.122` hub, its send looked successful, and nothing arrived.

```
termlink fleet join <host:port> --name <profile> \
    (--bootstrap-from ssh:<hub-host> | --bootstrap-from file:<path> | --token-file <path>) \
    [--fingerprint sha256:<hex>] [--json]
```

## What it does, step by step

Every step prints one line: `[OK]`, `[NOTE]` or `[FAIL]`. A failure names the step,
prints `NOT JOINED`, and exits 1. Re-running the same command is safe.

| step | action | fails when |
|---|---|---|
| probe | TLS handshake to the hub; reads its certificate fingerprint (no auth, no pin) | hub unreachable, port closed, not TLS |
| pin | TOFU pin in `known_hubs` | `--fingerprint` differs from what the hub presents; or a *different* pin exists and no `--fingerprint` was given (TOFU mismatch) |
| credential | `ssh:`/`file:`: reads the hub secret out of band, writes `~/.termlink/secrets/<name>.hex` mode 600. `--token-file`: checks the token is valid, unexpired, interact scope | ssh/file fails, value is not 64 hex, token expired or wrong scope |
| profile | adds or updates `[hubs.<name>]` in `~/.termlink/hubs.toml` (with `bootstrap_from` so `fleet reauth --bootstrap-from auto` works later) | hubs.toml unparseable |
| round-trip | TLS (pinned) + `hub.auth` + an authenticated `channel.list` | auth refused, secret or token wrong |

Exit 0 means the device is joined and was *proven* able to talk to the hub.

## Trust (R2)

1. The secret only travels over the channel you name (`ssh:`, `file:`, or a token file you
   copied). It never goes over the bus, and it is never printed. Only a sha256 fingerprint
   of it is shown.
2. Pass `--fingerprint` when you can. Get the value on the hub host with
   `termlink hub fingerprint`. Without it the first contact is trust-on-first-use, and the
   output tells you to compare.
3. A stale pin is replaced only when `--fingerprint` matches what the hub presents.

## Example: the operator's Mac to the ring20 `.122` hub

```
termlink fleet join 192.168.10.122:9100 --name ring20-management \
    --bootstrap-from ssh:root@192.168.10.122 \
    --fingerprint sha256:22c19fedafd73da27cb86945d8ae6002202fd4f4e75d22ce8d4097bd19d00d46
```

Without ssh to the hub: on the hub host, run `termlink hub token --out /tmp/mac.tok --scope interact`.
Copy the file to the Mac out of band, `chmod 600` it, then join with `--token-file`.

Send afterwards with `--hub ring20-management`.

## Related loud-failure changes in the same task

1. `channel post` (and everything built on it: `channel dm`, `agent contact`) exits **3**
   with `NOT DELIVERED:` on stderr when the message only reached the local offline queue.
   It used to exit 0. Exit 1 still means it failed outright.
2. A `--hub` value that is not `host:port`, a known profile name, or a path is now an error
   that names the reason (wrong scheme, no profile, bad port). It used to be queued locally
   with exit 0.
3. `termlink register` refuses to start when a hub is running but serves a different
   runtime dir, and names both dirs. The override is `TERMLINK_ALLOW_RUNTIME_DIR_MISMATCH=1`.
