# T-006: Session Identity, Discovery, and Lifecycle

## Question

How do TermLink sessions find each other, identify themselves, and handle lifecycle transitions?

## Parent

T-003 (GO: message bus + injection adapter, control/data plane split)

## Research Areas

1. Naming scheme — human-readable, UUID, role-based, composite
2. Registration protocol — auto vs explicit, what gets registered
3. Discovery mechanisms — filesystem, broker, mDNS
4. Lifecycle state machine — states, transitions, notifications
5. Liveness detection — socket probe, PID check, heartbeat
6. Cross-environment considerations
7. Concrete design proposal

---

## Findings

### 1. Naming Scheme Analysis

#### Approach Comparison

| Approach | Example | Pros | Cons | Prior Art |
|----------|---------|------|------|-----------|
| Human-readable | `builder`, `tester` | Memorable, type-able, tab-completable | Collision risk across users/projects | tmux sessions |
| UUID | `a1b2c3d4-e5f6-...` | Globally unique, zero collision | Unreadable, untypeable, no semantic value | D-Bus unique names (`:1.42`) |
| Role-based | `:coder`, `:reviewer` | Semantic addressing, intent-clear | One per role (or needs disambiguation) | D-Bus well-known names |
| Composite | `user.project.role` | Hierarchical, scoped, rich addressing | Verbose, parsing complexity | DNS, Java packages, D-Bus paths |
| PID-based | `session-12345` | Auto-unique per host | Meaningless, recycled after reboot, not stable | Common in /tmp patterns |
| Hybrid (D-Bus model) | Unique `:tl-a1b2c3` + alias `builder` | Best of both: stable identity + human UX | Two names to manage | D-Bus (unique + well-known) |

#### Prior Art Deep Dive

**tmux:** Sessions have human-readable names (`tmux new -s work`). No uniqueness enforcement — creating a session with a duplicate name fails. Discovery is via `tmux list-sessions`. Simple but fragile for programmatic use.

**D-Bus:** The gold standard for session identity. Every connection gets a **unique name** (`:1.42`) assigned by the bus. Applications can additionally claim **well-known names** (`org.freedesktop.Notifications`). The unique name is immutable for the connection lifetime; well-known names can be transferred. This dual-name system separates identity (unique) from addressing (well-known).

**kitty:** Uses a single Unix socket per instance at a well-known path. No multi-session naming — each kitty instance is a single endpoint. Discovery is implicit (the socket exists or it doesn't).

**Zellij:** Sessions have human-readable names with auto-generated defaults (`adjective-noun` pattern like `welcome-sailfish`). Listed via `zellij list-sessions`. Socket path encodes the session name.

**Docker:** Container IDs (SHA256 prefix) + human-assigned names (`--name myapp`). Names are optional aliases for the canonical ID. Exactly the hybrid model.

#### Recommendation: Dual-Name Hybrid (D-Bus Model)

Every TermLink session gets two identifiers:

1. **Unique ID:** `tl-{random8}` — 8 characters from base32 alphabet (a-z, 2-7), assigned at creation, immutable. Example: `tl-k7mx2b4n`. This is the canonical identity. Always filesystem-safe. Probability of collision with 8 base32 chars: 1 in ~1 trillion for a pair, negligible for any realistic number of concurrent sessions.

2. **Display name (optional alias):** Human-assigned or auto-generated, used for addressing. Examples: `builder`, `test-runner`, `my-project-coder`. Must be filesystem-safe (alphanumeric, hyphens, underscores, dots). Can be changed without affecting identity.

**Why base32 and not UUID?** UUIDs are 36 characters — too long to type, too long for socket filenames, and the full uniqueness guarantee is overkill for sessions on a single machine (or even a LAN). 8 base32 characters (40 bits of entropy) give ~1 trillion possibilities — more than enough for concurrent local sessions while remaining typeable and readable.

**Why not pure human-readable?** Collisions. Two users both naming their session `builder` on the same machine, or the same user in two projects. The unique ID prevents this structurally.

**Addressing rules:**
- Messages can be addressed to unique ID (`tl-k7mx2b4n`) or display name (`builder`)
- If a display name resolves to multiple sessions (collision), the sender gets an error listing matches — no silent delivery to the wrong target
- Role-based addressing is a special case of display name: `@coder` means "find session with display name or role `coder`"
- Wildcard patterns are supported for broadcast: `test-*` sends to all matching sessions

**Filesystem representation:** The unique ID is the socket filename. The display name is stored in the registration metadata.

```
# Socket file named by unique ID
/run/user/1000/termlink/sessions/tl-k7mx2b4n.sock

# Registration file with display name and metadata
/run/user/1000/termlink/sessions/tl-k7mx2b4n.json
```

---

### 2. Registration Protocol

#### Registration Directory

**Linux:** `$XDG_RUNTIME_DIR/termlink/sessions/` (typically `/run/user/$UID/termlink/sessions/`)
- XDG_RUNTIME_DIR is per-user, tmpfs-mounted, cleaned on logout
- Guaranteed by systemd on modern Linux distros

**macOS:** `$TMPDIR/termlink-$UID/sessions/` (typically `/var/folders/.../termlink-$UID/sessions/`)
- macOS has no XDG_RUNTIME_DIR equivalent
- $TMPDIR is per-user and per-session on macOS
- Fallback: `/tmp/termlink-$UID/sessions/` if TMPDIR is unavailable

**Resolution order:**
1. `$TERMLINK_RUNTIME_DIR` (explicit override, for testing/containers)
2. `$XDG_RUNTIME_DIR/termlink/` (Linux standard)
3. `$TMPDIR/termlink-$UID/` (macOS)
4. `/tmp/termlink-$UID/` (universal fallback)

**Directory permissions:** `0700` (owner only). Created on first session registration.

#### Auto-Registration Flow

```
1. Session starts
   ├── Generate unique ID: tl-{random8}
   ├── Resolve display name (CLI arg, env var, or auto-generate)
   └── Resolve runtime directory

2. Create registration
   ├── Create socket: $RUNTIME_DIR/sessions/tl-xxx.sock
   ├── Bind and listen on socket
   ├── Write registration: $RUNTIME_DIR/sessions/tl-xxx.json
   │   (atomic write via temp file + rename)
   └── Write PID lock: include PID in registration JSON

3. Session is READY
   ├── Accepting connections on control plane socket
   └── Registration is visible to discovery
```

#### Registration Entry Format

```json
{
  "version": 1,
  "id": "tl-k7mx2b4n",
  "display_name": "builder",
  "pid": 12345,
  "uid": 1000,
  "socket": "/run/user/1000/termlink/sessions/tl-k7mx2b4n.sock",
  "created_at": "2026-03-08T15:30:00Z",
  "heartbeat_at": "2026-03-08T15:30:00Z",
  "state": "ready",
  "capabilities": ["inject", "command", "query"],
  "roles": ["coder"],
  "metadata": {
    "shell": "/bin/zsh",
    "term": "xterm-256color",
    "cwd": "/home/user/project",
    "termlink_version": "0.1.0"
  }
}
```

**Field definitions:**

| Field | Type | Required | Purpose |
|-------|------|----------|---------|
| `version` | int | Yes | Registration format version (for forward compat) |
| `id` | string | Yes | Unique session ID (`tl-{random8}`) |
| `display_name` | string | Yes | Human-readable name (assigned or auto-generated) |
| `pid` | int | Yes | Process ID of owning process (for liveness checks) |
| `uid` | int | Yes | User ID (for multi-user disambiguation) |
| `socket` | string | Yes | Absolute path to control plane Unix socket |
| `created_at` | string | Yes | ISO 8601 timestamp |
| `heartbeat_at` | string | Yes | Last activity timestamp (updated periodically) |
| `state` | string | Yes | Lifecycle state (see section 4) |
| `capabilities` | array | Yes | What this session can do |
| `roles` | array | No | Semantic roles for role-based addressing |
| `metadata` | object | No | Environment info for display and debugging |

**Capabilities vocabulary (v1):**

| Capability | Meaning |
|------------|---------|
| `inject` | Accepts keystroke injection into terminal |
| `command` | Accepts structured command execution |
| `query` | Responds to state queries (CWD, env, output) |
| `stream` | Supports data plane streaming |
| `notify` | Accepts notification messages |

#### Name Collision Handling

When a session attempts to register with a display name that already exists:

1. **Check if existing session is alive** (PID check + socket probe)
2. **If stale:** Clean up the dead registration, proceed with the name
3. **If alive:** Reject the registration with error: `"display name 'builder' is already in use by tl-abc12345 (PID 6789)"`
4. **Auto-suffix option:** If `--auto-suffix` is set, append `-2`, `-3`, etc. (`builder-2`)
5. **Force option:** If `--replace` is set, send a `deregister` notification to the existing session and take the name (requires the existing session to be in `ready` state)

Unique IDs never collide (probabilistically negligible). If they somehow do (astronomically unlikely), regenerate.

---

### 3. Discovery Mechanisms

#### Mechanism Comparison

| Mechanism | Latency | Reliability | Complexity | Broker needed? | Cross-machine? |
|-----------|---------|-------------|------------|:-:|:-:|
| Filesystem scan | 1-5ms | High (atomic files) | Low | No | No |
| Broker query | <1ms (cached) | Depends on broker | Medium | Yes | Yes |
| mDNS/Bonjour | 100-2000ms | Medium (UDP, cache) | High | No | LAN only |
| Inotify/FSEvents | <1ms (event-driven) | High | Medium | No | No |
| Socket broadcast | 5-50ms | Medium | Medium | No | No |

#### Recommended: Filesystem-Primary with Watch Overlay

**Primary discovery: filesystem scan.** The registration directory is the source of truth. This satisfies the constraint that discovery must work without a broker.

**Secondary: filesystem watch.** inotify (Linux) / FSEvents (macOS) for real-time join/leave notifications without polling.

**Tertiary (future, T-011): broker-mediated.** For cross-machine discovery, a lightweight broker can aggregate registrations from multiple hosts.

#### Discovery API

```
list_sessions() -> Session[]
    Scan $RUNTIME_DIR/sessions/*.json
    Parse each registration file
    Optionally validate liveness (PID check)
    Return sorted by created_at

find_session(query: string) -> Session | Error
    If query matches tl-{8chars} pattern: lookup by unique ID
    Else: search by display_name
    If multiple matches: return AmbiguousMatch error with candidates
    If no match: return NotFound error

find_by_capability(capability: string) -> Session[]
    Scan all registrations
    Filter by capabilities array contains capability
    Return matching sessions

find_by_role(role: string) -> Session[]
    Scan all registrations
    Filter by roles array contains role
    Return matching sessions

watch_sessions(callback: fn(Event)) -> Watcher
    Set up inotify/FSEvents watch on $RUNTIME_DIR/sessions/
    On file create: parse registration, emit SessionJoined event
    On file delete: emit SessionLeft event
    On file modify: parse registration, emit SessionUpdated event
    Return handle for unsubscribing
```

**Event types:**

```
SessionJoined  { session: Session }
SessionLeft    { id: string, reason: "graceful" | "crashed" | "cleaned" }
SessionUpdated { session: Session, changed_fields: string[] }
```

#### Performance Characteristics

For a typical scenario of 2-20 concurrent sessions:

- **`list_sessions()`:** One `readdir()` + N `read()` calls. With 20 sessions, each registration JSON ~500 bytes, total I/O < 10KB. Expected latency: 1-3ms on SSD/tmpfs.
- **`find_session(name)`:** Same as list + filter. Could be optimized with a name-to-ID symlink, but unnecessary at this scale.
- **`watch_sessions()`:** Zero-cost when idle (kernel event-driven). Event delivery in <1ms from file system operation.

**Scaling note:** This design is appropriate for tens to low hundreds of sessions. If thousands of sessions are needed (unlikely for terminal sessions), a broker-backed index would be warranted. That's a future concern.

---

### 4. Lifecycle State Machine

#### State Definitions

```
                    ┌──────────────────────────────────────────────┐
                    │                                              │
                    ▼                                              │
            ┌──────────────┐                                      │
            │ initializing │                                      │
            └──────┬───────┘                                      │
                   │ socket bound,                                │
                   │ registration written                          │
                   ▼                                              │
            ┌──────────────┐     command      ┌──────────────┐   │
            │    ready     │ ──────────────── │     busy     │   │
            └──────┬───────┘                  └──────┬───────┘   │
                   │ ▲          command done          │           │
                   │ └───────────────────────────────┘           │
                   │                                              │
                   │ shutdown signal                              │
                   ▼                                              │
            ┌──────────────┐                                      │
            │   draining   │──── timeout/complete ────┐           │
            └──────────────┘                          │           │
                                                      ▼           │
            ┌──────────────┐                   ┌──────────────┐  │
            │   crashed    │                   │     gone     │  │
            └──────────────┘                   └──────────────┘  │
                   │                                  ▲           │
                   └──── cleanup detected ────────────┘           │
                                                                  │
            Any state ──── process dies unexpectedly ─── crashed ─┘
```

#### State Details

| State | Meaning | Accepts Messages? | Detection from Outside | Registration File |
|-------|---------|:-:|---|---|
| `initializing` | Socket being created, not yet ready | No (connection refused) | Socket file exists but connect fails | JSON exists with `state: initializing` |
| `ready` | Idle, waiting for work | Yes (all types) | Socket accepts connections | `state: ready` |
| `busy` | Processing a command/injection | Yes (queue/reject policy) | Socket accepts, response includes busy flag | `state: busy` |
| `draining` | Shutting down gracefully, finishing in-flight work | No new commands; in-flight complete | Socket accepts but returns "draining" status | `state: draining` |
| `gone` | Cleanly deregistered | No (socket removed) | No socket file, no registration | Registration file removed |
| `crashed` | Process died unexpectedly | No (connection refused, but socket file remains) | Socket file exists, PID is dead | Registration exists but PID check fails |

#### Transitions

| From | To | Trigger | Side Effects |
|------|-----|---------|-------------|
| `initializing` | `ready` | Socket bound, listening | Update registration `state: ready`, emit `SessionJoined` |
| `ready` | `busy` | Command received | Update registration `state: busy` |
| `busy` | `ready` | Command completed | Update registration `state: ready`, update `heartbeat_at` |
| `ready` | `draining` | SIGTERM, explicit shutdown | Update registration `state: draining`, stop accepting new commands |
| `busy` | `draining` | SIGTERM during command | Finish current command, then drain |
| `draining` | `gone` | All in-flight complete or timeout (5s) | Remove socket file, remove registration, emit `SessionLeft` |
| Any | `crashed` | Process dies (SIGKILL, OOM, panic) | No action (socket/registration remain as orphans) |
| `crashed` | `gone` | Cleanup detected by another session or periodic scan | Remove socket file, remove registration, emit `SessionLeft(reason: crashed)` |

#### Scenario Walkthroughs

**Normal startup:**
```
1. Process starts, enters "initializing"
2. Generates unique ID tl-abc12345
3. Creates socket file, binds, listens
4. Writes registration JSON (atomic: write temp, rename)
5. Transitions to "ready"
6. Other sessions see SessionJoined via filesystem watch
```

**Normal shutdown (SIGTERM):**
```
1. Receives SIGTERM, enters "draining"
2. Updates registration state to "draining"
3. Stops accepting new commands, finishes in-flight
4. Closes socket, removes socket file
5. Removes registration JSON
6. Process exits
7. Other sessions see SessionLeft via filesystem watch
```

**Crash (SIGKILL, OOM):**
```
1. Process dies immediately — no cleanup runs
2. Socket file and registration JSON remain on disk
3. Next discovery scan finds the entry
4. PID check (kill -0) returns "no such process"
5. Cleanup: remove socket file, remove registration JSON
6. Emit SessionLeft(reason: crashed)
```

**Busy session policy:**
When a session is `busy`, incoming messages are handled by policy:
- **Queue (default):** Message is queued, processed when session returns to `ready`
- **Reject:** Message is rejected with `SessionBusy` error (caller can retry)
- **Interrupt:** For high-priority messages (SIGINT forwarding), bypass the queue

The policy is configurable per-session and per-message-type.

---

### 5. Liveness Detection

#### Approach Comparison

| Approach | Speed | Reliability | False Positives | False Negatives | Resource Cost |
|----------|-------|-------------|:---:|:---:|---|
| PID check (`kill -0`) | <0.1ms | High on same host | PID recycled (rare) | None | Negligible |
| Socket connect probe | 1-10ms | High | None | Slow startup (initializing) | One TCP connect per check |
| Heartbeat (periodic) | Depends on interval | Highest | None | Delayed detection | Continuous overhead |
| Hybrid (PID + socket) | 1-10ms | Highest | None | None | One check per validation |

#### Recommended: Hybrid (PID-first, socket-confirm)

```
is_alive(session: Registration) -> bool:
    # Fast path: PID check (microseconds)
    if not process_exists(session.pid):
        return false  # Definitely dead

    # PID exists — but could be recycled (different process reused the PID)
    # Check socket connectivity to confirm (milliseconds)
    sock = try_connect(session.socket)
    if sock.error == ECONNREFUSED:
        return false  # Socket exists but nothing listening — zombie entry
    if sock.error == ENOENT:
        return false  # Socket file was removed

    # Confirm it's actually a TermLink session (not a PID-recycled process
    # that coincidentally created a file at the same path — astronomically unlikely
    # but defense-in-depth)
    response = sock.send({"method": "termlink.ping"})
    if response.result.id == session.id:
        return true   # Confirmed alive and correct identity
    else:
        return false  # Something else at this socket
```

**Why PID-first?** The `kill(pid, 0)` syscall is essentially free (~0.05ms). It eliminates dead sessions without any network I/O. Only if the PID is alive do we spend 1-10ms on a socket probe. For a discovery scan of 20 sessions, this means ~1ms for dead sessions vs ~200ms if we socket-probed everything.

**PID recycling concern:** On Linux, PIDs wrap around at `pid_max` (default 32768, can be 4194304). A recycled PID would need to occur at the exact same PID number AND the new process would need to NOT be a TermLink session (or be a different TermLink session). The socket probe eliminates this: even with a recycled PID, the socket path won't match or the ping response will have a different session ID.

#### Stale Entry Cleanup

**When to check:**

1. **On discovery (lazy cleanup):** Every `list_sessions()` or `find_session()` call validates liveness. Dead entries are cleaned synchronously before returning results. This is the primary cleanup mechanism.

2. **On failed send (reactive cleanup):** If sending a message to a session fails with `ECONNREFUSED` or timeout, mark the session as crashed and clean up.

3. **Periodic sweep (background, optional):** A low-frequency timer (every 60s) scans all registrations. Catches entries missed by lazy cleanup (e.g., sessions no one has tried to contact).

**Cleanup algorithm:**

```
cleanup_stale(session: Registration):
    # Step 1: Confirm death (double-check to avoid race)
    if is_alive(session):
        return  # Still alive, abort cleanup

    # Step 2: Acquire cleanup lock (prevent concurrent cleanup races)
    lock_path = session.socket + ".cleanup"
    lock = try_acquire_lock(lock_path, timeout=1s)
    if not lock:
        return  # Another process is cleaning this entry

    # Step 3: Re-confirm death (check again after acquiring lock)
    if is_alive(session):
        release_lock(lock)
        return  # Came alive between our check and lock acquisition

    # Step 4: Remove artifacts
    remove_file(session.socket)       # Remove socket file
    remove_file(session.registration) # Remove JSON registration
    release_lock(lock)
    remove_file(lock_path)            # Remove lock file

    # Step 5: Emit event
    emit(SessionLeft { id: session.id, reason: "crashed" })
```

**Race condition: cleaning a starting session:**

The critical race is: Session A is starting up (writing registration), and Session B concurrently decides the entry is stale and deletes it.

**Mitigation:** The `initializing` state with a recent `created_at` timestamp. The cleanup algorithm includes a grace period:

```
# Don't clean entries less than 5 seconds old
if now() - session.created_at < 5s:
    return  # Too young to be stale, might be initializing
```

This gives new sessions 5 seconds to complete initialization. Since initialization (create socket, bind, listen, write JSON) takes <100ms in practice, this provides ample margin.

---

### 6. Cross-Environment Considerations

> Note: Full cross-machine design is T-011. This section identifies identity-specific concerns only.

#### Docker Containers

**Challenge:** PID namespaces are isolated. A container's PID 1 is not the host's PID 1. PID-based liveness checks from host to container (or vice versa) will fail.

**Identity impact:**
- Session IDs are still globally unique (random, not PID-based)
- Registration files can be shared via volume mount: `-v /run/user/1000/termlink:/run/user/1000/termlink`
- PID in registration must note the namespace: `"pid_ns": "container:abc123"` or liveness checks switch to socket-only mode for cross-namespace entries
- Alternative: each container runs its own termlink runtime directory, and a bridge process handles cross-boundary discovery

#### SSH Sessions

**Challenge:** No shared filesystem between local and remote. Unix sockets aren't network-accessible.

**Identity impact:**
- Remote sessions have their own registration directory on the remote host
- SSH socket forwarding (`-L` / `-R`) can tunnel a specific session's socket, but not the whole discovery directory
- For full remote discovery, the control plane's Streamable HTTP transport is the answer — the remote host exposes a termlink HTTP endpoint
- Session IDs remain globally unique across hosts (random generation)

#### tmux/screen (Nested Sessions)

**Challenge:** A TermLink session inside tmux is a real terminal with a real PTY, but the outer tmux session also exists.

**Identity impact:**
- Both the tmux session and the inner TermLink session can register independently
- Metadata should include `"parent": "tmux:work"` to indicate nesting
- Injection into a tmux-nested session should go through TermLink's socket (direct), not through tmux's send-keys (indirect) — TermLink is the inner session's owner
- Discovery should surface nesting information so tools can decide which level to target

#### VS Code Integrated Terminal

**Challenge:** VS Code terminals run as child processes of the VS Code extension host. Multiple terminal tabs share a parent process.

**Identity impact:**
- Each VS Code terminal tab can be its own TermLink session with its own registration
- The extension host PID is not the terminal's PID — use the shell PID instead
- VS Code extensions could provide enhanced discovery (showing TermLink sessions in the terminal tab UI)
- Metadata: `"environment": "vscode"`, `"terminal_id": "terminal-1"`

---

### 7. Concrete Design Proposal

#### Registration Directory Structure

```
$RUNTIME_DIR/
├── termlink/
│   ├── sessions/
│   │   ├── tl-k7mx2b4n.sock          # Control plane Unix socket
│   │   ├── tl-k7mx2b4n.json          # Registration metadata
│   │   ├── tl-k7mx2b4n.data.sock     # Data plane Unix socket (if streaming)
│   │   ├── tl-p3qr9w2f.sock
│   │   ├── tl-p3qr9w2f.json
│   │   └── tl-p3qr9w2f.data.sock
│   └── bus.sock                        # Optional: broker socket (if running)
```

**Path resolution (pseudocode):**
```
fn runtime_dir() -> Path:
    if env("TERMLINK_RUNTIME_DIR"):
        return env("TERMLINK_RUNTIME_DIR")
    if env("XDG_RUNTIME_DIR"):
        return env("XDG_RUNTIME_DIR") / "termlink"
    if env("TMPDIR"):
        return env("TMPDIR") / "termlink-{uid}"
    return "/tmp/termlink-{uid}"
```

#### Registration File Format (JSON)

Example: `/run/user/1000/termlink/sessions/tl-k7mx2b4n.json`

```json
{
  "version": 1,
  "id": "tl-k7mx2b4n",
  "display_name": "builder",
  "pid": 12345,
  "uid": 1000,
  "socket": "/run/user/1000/termlink/sessions/tl-k7mx2b4n.sock",
  "data_socket": "/run/user/1000/termlink/sessions/tl-k7mx2b4n.data.sock",
  "created_at": "2026-03-08T15:30:00Z",
  "heartbeat_at": "2026-03-08T15:45:12Z",
  "state": "ready",
  "capabilities": ["inject", "command", "query", "stream"],
  "roles": ["coder"],
  "metadata": {
    "shell": "/bin/zsh",
    "term": "xterm-256color",
    "cwd": "/home/user/my-project",
    "termlink_version": "0.1.0",
    "parent": null,
    "environment": "native"
  }
}
```

#### Socket Naming Convention

| Socket | Pattern | Purpose |
|--------|---------|---------|
| Control plane | `tl-{id}.sock` | MCP-compatible JSON-RPC 2.0 |
| Data plane | `tl-{id}.data.sock` | Length-prefixed binary frames |

Both sockets are created by the session and cleaned up on shutdown. The control plane socket is always present; the data plane socket is created only when streaming capabilities are enabled.

#### Discovery API (Function Signatures)

```rust
// Core discovery
fn list_sessions(opts: ListOptions) -> Result<Vec<Session>, Error>;
fn find_session(query: &str) -> Result<Session, Error>;  // by ID or display_name
fn find_by_role(role: &str) -> Result<Vec<Session>, Error>;
fn find_by_capability(cap: &str) -> Result<Vec<Session>, Error>;

// Lifecycle watch
fn watch_sessions() -> Result<Receiver<SessionEvent>, Error>;

// Registration (called by session itself)
fn register(config: SessionConfig) -> Result<Registration, Error>;
fn deregister(id: &str) -> Result<(), Error>;
fn update_state(id: &str, state: State) -> Result<(), Error>;

// Liveness
fn is_alive(session: &Session) -> bool;
fn cleanup_stale() -> Vec<String>;  // returns cleaned session IDs

// Types
struct ListOptions {
    include_stale: bool,   // default false — validate liveness
    filter_state: Option<State>,
    filter_capability: Option<String>,
}

enum SessionEvent {
    Joined(Session),
    Left { id: String, reason: LeaveReason },
    Updated { session: Session, changed: Vec<String> },
}

enum LeaveReason {
    Graceful,
    Crashed,
    Cleaned,
}

enum State {
    Initializing,
    Ready,
    Busy,
    Draining,
    Gone,
}
```

#### Lifecycle State Machine (ASCII Art)

```
                         ┌─────────────────────────────────────────┐
                         │           LIFECYCLE STATES               │
                         └─────────────────────────────────────────┘

    Process starts
         │
         ▼
  ┌──────────────┐   socket bound     ┌──────────────┐
  │ INITIALIZING │ ─────────────────▶  │    READY     │◀─────────┐
  └──────────────┘   + reg written     └──────┬───────┘          │
         │                                     │                  │
         │ startup fails                       │ command          │ command
         │                                     │ received         │ done
         ▼                                     ▼                  │
  ┌──────────────┐                      ┌──────────────┐         │
  │     GONE     │◀────── timeout ────  │     BUSY     │─────────┘
  └──────────────┘                      └──────┬───────┘
         ▲                                     │
         │                                     │ SIGTERM
         │ all drained                         │ (mid-command)
         │ or timeout(5s)                      ▼
         │                              ┌──────────────┐
         ├───────────────────────────── │   DRAINING   │
         │                              └──────────────┘
         │
         │ cleanup
  ┌──────┴───────┐
  │   CRASHED    │◀──── process dies unexpectedly (from any state)
  └──────────────┘
```

#### Stale Entry Cleanup Algorithm

```
fn cleanup_stale_entries():
    entries = read_dir(runtime_dir() / "sessions" / "*.json")
    for entry in entries:
        session = parse_json(entry)

        # Grace period for initializing sessions
        if now() - session.created_at < 5s:
            continue

        # Fast path: PID check
        if process_exists(session.pid):
            # PID alive — try socket probe to confirm identity
            match try_connect(session.socket):
                Ok(conn):
                    resp = conn.send({"method": "termlink.ping"})
                    if resp.result.id == session.id:
                        continue  # Alive and confirmed
                    else:
                        # Wrong identity at this socket — stale
                        pass
                Err(_):
                    # Can't connect — stale despite PID existing
                    # (PID recycled or socket broken)
                    pass
        # else: PID dead — definitely stale

        # Acquire cleanup lock
        lock = try_lock(session.socket + ".cleanup", timeout=1s)
        if not lock:
            continue  # Someone else is cleaning

        # Double-check after lock acquisition
        if process_exists(session.pid) and try_ping(session.socket, session.id):
            release(lock)
            continue  # Came alive between check and lock

        # Remove stale artifacts
        remove(session.socket)
        remove(session.data_socket)  # if exists
        remove(entry)                # registration JSON
        release(lock)
        remove(lock.path)

        emit(SessionLeft { id: session.id, reason: Cleaned })
```

**Trigger points:**
1. `list_sessions()` with `include_stale: false` (default) runs cleanup implicitly
2. Failed message delivery triggers cleanup for the specific target
3. Optional background timer every 60 seconds

---

## Synthesis

The design converges on a **filesystem-first, broker-optional** architecture that mirrors the D-Bus dual-name model adapted for Unix socket IPC:

| Aspect | Decision | Rationale |
|--------|----------|-----------|
| **Naming** | Dual: unique ID (`tl-{random8}`) + optional display name | D-Bus model. Unique ID prevents collisions structurally. Display name provides usability (D3). |
| **Registration** | Auto-registration on session start, JSON sidecar files | No opt-in required. JSON is human-readable and debuggable. Atomic write via rename. |
| **Discovery** | Filesystem scan + inotify/FSEvents watch | No broker dependency (D4). Sub-5ms latency. Watch provides real-time events. |
| **Lifecycle** | 5-state machine: initializing → ready ↔ busy → draining → gone (+ crashed) | Covers normal operation, graceful shutdown, and crash recovery. |
| **Liveness** | Hybrid PID check + socket probe + identity ping | PID check is microseconds (fast path). Socket probe confirms. Identity ping eliminates PID recycling. |
| **Cleanup** | Lazy (on discovery) + reactive (on failed send) + optional periodic | Primary: lazy cleanup gives zero overhead when all sessions are healthy. |

**Alignment with constitutional directives:**

| Directive | How satisfied |
|-----------|---------------|
| D1 Antifragile | Crash recovery is structural (stale cleanup). No single point of failure. Sessions can disappear and the system self-heals. |
| D2 Reliable | Liveness detection with triple-check (PID → socket → identity). Atomic registration writes. Lock-based cleanup prevents races. |
| D3 Usable | Human-readable display names. Role-based addressing. Tab-completable. Auto-registration — zero configuration for basic use. |
| D4 Portable | Filesystem-based (works everywhere). XDG path resolution with macOS fallback. No mandatory infrastructure. JSON for interop. |

## Decision

**GO.** The design satisfies all go/no-go criteria:

- Discovery works without mandatory broker (filesystem-based) -- **GO**
- Lifecycle state machine covers crash, graceful shutdown, busy states -- **GO**
- Naming scheme is usable (display names) without sacrificing reliability (unique IDs) -- **GO**
- No mandatory infrastructure required (no portability violation) -- **GO**
- Reliable stale-entry cleanup mechanism exists (hybrid liveness + lazy/reactive/periodic cleanup) -- **GO**

**Key design decisions:**
1. **Dual naming (D-Bus model):** Unique ID for identity, display name for addressing
2. **Filesystem-primary discovery:** No broker needed; inotify/FSEvents for real-time
3. **Hybrid liveness:** PID check (fast) → socket probe (confirm) → identity ping (paranoid)
4. **5-state lifecycle:** initializing, ready, busy, draining, gone (+ crashed as detected state)
5. **Lazy-first cleanup:** Clean stale entries on discovery, not proactively

## Dialogue Log

### 2026-03-08 — Investigation started
- **Approach:** Sub-agent research dispatch

### 2026-03-08 — Research completed
- **Agent:** Comprehensive analysis of naming schemes, registration protocol, discovery mechanisms, lifecycle states, liveness detection, and cross-environment considerations
- **Key insight:** D-Bus dual-name model (unique ID + well-known name) maps perfectly to TermLink's needs — separates identity from addressing
- **Key insight:** Filesystem-based discovery with lazy liveness cleanup provides the best balance of simplicity, reliability, and portability
- **Key insight:** Hybrid PID+socket+identity liveness check eliminates all known false-positive and false-negative scenarios including PID recycling
- **Decision:** GO — filesystem-first, broker-optional, dual-name identity system with 5-state lifecycle
