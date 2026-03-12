# T-073: Transport Abstraction — Exploration Report

> Generated: 2026-03-12 | Source: TermLink agent mesh (explore-T073)

## 1. What's Hardcoded to Unix Sockets

**Protocol crate is clean** — no I/O, no tokio, no socket types. Pure data definitions.

**Session crate — 7 coupling points:**
- `manager.rs:85` — `UnixListener::bind()` in `Session::register_in()`
- `server.rs:24` — `handle_connection(stream: UnixStream, ...)`
- `client.rs:17-18` — `UnixStream::connect(socket_path)`
- `data_server.rs:45,53` — `UnixListener::bind()` + accept loop
- `registration.rs:21` — `pub socket: PathBuf` (address = filesystem path)
- `liveness.rs:20` — `reg.socket.exists()` (liveness = file exists)

**Hub crate — 3 coupling points:**
- `server.rs:60` — `UnixListener::bind()`
- `server.rs:189` — `handle_connection(stream: UnixStream)`
- `router.rs:312` — `Client::connect(&reg.socket)` for forwarding

## 2. What Changes for TCP/WebSocket

1. **Address type** — `Registration.socket: PathBuf` → transport-neutral enum
2. **Listener/connect** — 3 bind sites + 3 connect sites need abstraction
3. **Stream handling** — generalize `UnixStream` to `AsyncRead + AsyncWrite`
4. **Liveness** — `socket.exists()` is Unix-only; TCP needs connect probe
5. **Discovery** — `*.sock` glob is filesystem-based; TCP needs registry
6. **Peer credentials** — `SO_PEERCRED` is Unix-only; TCP relies on `auth.token` (already exists)

## 3. Proposed Trait Design

- **`TransportAddr` enum** in protocol crate (no new deps — just serde): `Unix { path }`, `Tcp { host, port }`
- **`Transport` trait** in session crate: `connect(addr) -> Box<dyn Connection>`, `bind(addr) -> Listener`
- **`Connection`** = blanket impl over `AsyncRead + AsyncWrite + Send + Unpin`
- **`TransportListener`**: `accept() -> Connection`, `local_addr() -> TransportAddr`
- **`LivenessProbe`** trait separate (strategy differs per transport)
- Dynamic dispatch via `Box<dyn Connection>` — negligible overhead for I/O-bound work
- Migration: wrap existing `UnixListener`/`UnixStream` in adapter structs
