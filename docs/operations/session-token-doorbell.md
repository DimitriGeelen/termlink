# Session tokens and the sidecar doorbell (T-3533, B′)

A session registered with `--token-secret` refuses unauthenticated calls. Its own
sidecar doorbell used to be among them, because `termlink inject` could not present
a token. This page covers the B′ change that fixes that, and what it does **not** do.

## What this is, and is not

- **It is bearer authorisation without attribution.** Whoever holds a token can use
  the scope it carries. The session cannot tell which agent or person used it, so a
  token is not an identity and the audit log shows the scope, not the caller.
- **It is an interim measure.** It is not O-7 (operator-only control of peer
  authority), and it is not an inject access list. A token that can reach a session
  can do what its scope allows, full stop.
- **The authentication stays in the handshake.** The token is sent once, in
  `auth.token`, on the connection. It is never a field of `command.inject` or
  `command.notify`, so the later signed-envelope design (T-3493 C) can replace it
  without changing either call.

## Scopes

| Scope | Methods it reaches | Who holds it |
|---|---|---|
| `observe` | reads | anyone who can connect |
| `notify` | `command.notify`: the doorbell's fixed line, with message ids only | the sidecar |
| `interact` | `event.emit`, `kv.set`, … | a peer that posts events |
| `control` | `command.inject`, `command.signal` (keystrokes) | the operator's own tooling |
| `execute` | `command.execute` | the operator only |

`notify` ranks below `interact`, so a notify token cannot change state. The doorbell
composes its own line from the message ids (count and eight characters of each id).
A notify token therefore cannot type anything the session did not compose.

## Provisioning a notify token

The token is a file, readable only by its owner. The sidecar finds it by session name:

```bash
# <receiver> is the project's .context/sidecar/receiver directory
mkdir -p <receiver>/notify-tokens
umask 077
termlink token create <session-id> --scope notify --ttl 604800 \
  > <receiver>/notify-tokens/<session-name>.tok
chmod 600 <receiver>/notify-tokens/<session-name>.tok
```

- The file must have no group or other permission bits. A file that is looser is
  refused, with the `chmod 600` fix named in the error. The token is never printed.
- `--ttl` is capped by `TERMLINK_MAX_TOKEN_TTL_SECS` (7 days by default). When it
  expires, rings are refused until you mint a new one (see Rotation).
- The token is never a CLI value and never an environment variable. The CLI has no
  `--token` flag, and the sidecar passes the file's path, not its contents.

For a manual call, use the file or an inherited descriptor:

```bash
termlink pty notify <session> --ids m-1,m-2 --token-file ./notify.tok
termlink pty notify <session> --ids m-1 --token-fd 3 3< ./notify.tok
termlink pty inject <session> "ls" --enter --token-file ./control.tok   # needs control
```

## What a failure looks like

A missing, wrong, or expired token is a visible doorbell failure, never a silent drop:

- `termlink pty notify` exits non-zero, with `Session refused the token: …`.
- The sidecar records the outcome in `receiver/doorbell.json`. A refused ring makes
  `fw sidecar status` print a line starting `WARN doorbell:`, naming the session.
- The watcher's report carries `doorbell refused (termlink pty notify): …`, and the
  message's ledger row records `INJECT_ATTEMPT ok=false`.

A session without `--token-secret` needs no token, and rings as before.

## Rotation

Rotating a session's secret (re-registering it, or a new `--token-secret`) makes every
existing token invalid. Mint new notify tokens against the new session, replace each
file (`chmod 600` again), and check `fw sidecar status` for the `WARN doorbell` line
disappearing after the next ring. An expired token is the same as a wrong one.

## Limits, stated plainly

- Anyone who can read the token file, or who holds a live token, can ring the doorbell
  for as long as the token is valid. Same-user processes can read the file.
- The `tmux` route of the doorbell still types its line directly (the terminal is not
  a `termlink` session). It is not covered by this change.
- `termlink register --token-secret` prints the session secret to its own output. That
  is existing behaviour, not changed here; keep that output out of shared logs.
- Hub-routed calls (`remote inject`, the hub's `command.inject`) keep their existing
  scopes. The notify scope applies to the session's own socket only.
