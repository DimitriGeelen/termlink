# SSH Tunneling for Cross-Machine TermLink

TermLink's protocol is stream-agnostic — both the JSON-RPC control plane and
binary data plane work transparently over SSH-forwarded Unix sockets. No code
changes are needed.

## Prerequisites

- OpenSSH 7.6+ (for Unix socket forwarding)
- TermLink installed on both machines
- SSH access to the remote machine

## Scenario 1: Forward a Single Session

Connect to a remote TermLink session from your local machine.

```bash
# On remote machine: register a session
termlink register --name worker-1 --roles worker

# Find the socket path
termlink list
# Output: tl-abc123  worker-1  /run/termlink/sessions/tl-abc123.sock

# On local machine: forward the remote socket
ssh -L /tmp/remote-worker.sock:/run/termlink/sessions/tl-abc123.sock user@remote

# Now you can talk to the remote session locally:
termlink ping worker-1 --socket /tmp/remote-worker.sock
termlink send worker-1 "hello from local" --socket /tmp/remote-worker.sock
```

## Scenario 2: Forward the Hub

Forward the entire hub socket to access all remote sessions through one tunnel.

```bash
# On remote machine: start hub
termlink hub

# Find hub socket
termlink info
# Output: Hub socket: /run/termlink/hub.sock

# On local machine: forward the hub
ssh -L /tmp/remote-hub.sock:/run/termlink/hub.sock user@remote

# Access all remote sessions via the forwarded hub:
TERMLINK_HUB_SOCKET=/tmp/remote-hub.sock termlink list
TERMLINK_HUB_SOCKET=/tmp/remote-hub.sock termlink broadcast "hello everyone"
```

## Scenario 3: TCP Bridge via socat

When you need TCP connectivity (e.g., for Docker containers), use socat to
bridge between Unix sockets and TCP.

```bash
# On remote machine: expose a session via TCP
socat TCP-LISTEN:9000,reuseaddr,fork \
      UNIX-CONNECT:/run/termlink/sessions/tl-abc123.sock &

# On local machine: forward the TCP port
ssh -L 9000:localhost:9000 user@remote

# Connect via TCP (requires TcpTransport — Phase 1, T-011)
# For now, bridge back to a local Unix socket:
socat UNIX-LISTEN:/tmp/remote-session.sock,fork TCP-CONNECT:localhost:9000 &
termlink ping worker-1 --socket /tmp/remote-session.sock
```

## Scenario 4: Persistent Tunnel with autossh

For long-running agent coordination, use autossh to maintain the tunnel.

```bash
# Install: brew install autossh (macOS) or apt install autossh (Linux)

# Persistent hub tunnel with auto-reconnect
autossh -M 0 -f -N \
    -L /tmp/remote-hub.sock:/run/termlink/hub.sock \
    -o "ServerAliveInterval 30" \
    -o "ServerAliveCountMax 3" \
    user@remote
```

## Limitations

- **Manual setup:** Each session/hub needs its own tunnel or socat bridge
- **No service discovery:** Remote sessions don't appear in local `termlink list`
  unless the hub is forwarded
- **Latency:** Adds SSH overhead (~1-10ms per RPC call)
- **Socket paths:** Must match between machines or use explicit `--socket` flag

## When to Use TCP Instead

SSH tunneling is best for:
- 1-2 machine setups (dev laptop + cloud VM)
- Ad-hoc cross-machine coordination
- Quick prototyping before implementing native TCP

For container orchestration, CI workers, or 3+ machine topologies, use
native TCP transport (Phase 1 — requires TcpTransport implementation).
