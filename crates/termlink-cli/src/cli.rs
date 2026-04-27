use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "termlink",
    about = "Cross-terminal session communication",
    version
)]
pub(crate) struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Subcommand)]
pub(crate) enum Command {
    // === Session Management ===

    /// Register a new session and start listening for connections
    Register {
        /// Display name for this session
        #[arg(short, long)]
        name: Option<String>,

        /// Roles this session provides (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        roles: Vec<String>,

        /// Tags for this session (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Capabilities for this session (comma-separated)
        #[arg(long, value_delimiter = ',')]
        cap: Vec<String>,

        /// Start a PTY-backed session (full bidirectional I/O)
        #[arg(long, conflicts_with = "self_mode")]
        shell: bool,

        /// Register current process as event-only endpoint (no PTY)
        #[arg(long = "self", id = "self_mode")]
        self_mode: bool,

        /// Enable token-based authentication (generates a random secret)
        #[arg(long)]
        token_secret: bool,

        /// Restrict command.execute to commands matching these prefixes (comma-separated)
        #[arg(long, value_delimiter = ',')]
        allowed_commands: Vec<String>,

        /// Output session details as JSON on startup
        #[arg(long)]
        json: bool,

        /// Suppress all non-error output at startup
        #[arg(long, short = 'q')]
        quiet: bool,
    },

    /// List all registered sessions
    List {
        /// Include stale/dead sessions
        #[arg(long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Filter by tag (sessions must have this tag)
        #[arg(long)]
        tag: Option<String>,

        /// Filter by name (substring match)
        #[arg(long)]
        name: Option<String>,

        /// Filter by role
        #[arg(long)]
        role: Option<String>,

        /// Filter by capability
        #[arg(long)]
        cap: Option<String>,

        /// Only print the session count (useful for scripting)
        #[arg(long)]
        count: bool,

        /// Print only session display names (one per line, for piping)
        #[arg(long)]
        names: bool,

        /// Print only session IDs (one per line, for piping)
        #[arg(long)]
        ids: bool,

        /// Output only the first matching session (exits 1 if none)
        #[arg(long)]
        first: bool,

        /// Wait (poll) until at least one session matches
        #[arg(long)]
        wait: bool,

        /// Timeout in seconds for --wait (default: 30)
        #[arg(long, default_value = "30")]
        wait_timeout: u64,

        /// Suppress table header and footer (for piping/awk)
        #[arg(long)]
        no_header: bool,

        /// Sort sessions: age, age-desc, name, name-desc, state
        #[arg(long)]
        sort: Option<String>,
    },

    /// Ping a session to verify it's alive
    Ping {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        /// Remote hub address HOST:PORT — forwards the ping through the hub
        /// to the named session on that host (T-921 cross-host parity).
        #[arg(long = "target", value_name = "HOST:PORT")]
        hub: Option<String>,

        /// Path to a hex-encoded HMAC secret file for the remote hub.
        #[arg(long = "secret-file", value_name = "PATH")]
        secret_file: Option<std::path::PathBuf>,

        /// Explicit hex-encoded HMAC secret (64 chars / 32 bytes).
        /// Prefer --secret-file for shell-history hygiene.
        #[arg(long, value_name = "HEX")]
        secret: Option<String>,

        /// Auth scope: observe | interact | control | execute.
        /// Defaults to the per-method minimum (observe for ping).
        #[arg(long, value_name = "NAME")]
        scope: Option<String>,
    },

    /// Query a session's status
    Status {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output one-line summary (name state pid)
        #[arg(long)]
        short: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        /// Remote hub address HOST:PORT — forwards the status query through
        /// the hub to the named session on that host (T-921 cross-host
        /// parity).
        #[arg(long = "target", value_name = "HOST:PORT")]
        hub: Option<String>,

        /// Path to a hex-encoded HMAC secret file for the remote hub.
        #[arg(long = "secret-file", value_name = "PATH")]
        secret_file: Option<std::path::PathBuf>,

        /// Explicit hex-encoded HMAC secret (64 chars / 32 bytes).
        /// Prefer --secret-file for shell-history hygiene.
        #[arg(long, value_name = "HEX")]
        secret: Option<String>,

        /// Auth scope: observe | interact | control | execute.
        /// Defaults to the per-method minimum (observe for status).
        #[arg(long, value_name = "NAME")]
        scope: Option<String>,
    },

    /// Show TermLink runtime information and system status
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// One-line summary output (for scripting)
        #[arg(long)]
        short: bool,

        /// Exit non-zero if hub is stopped or stale sessions exist
        #[arg(long)]
        check: bool,
    },

    // === RPC & Execution ===

    /// Send a JSON-RPC method call to a session
    Send {
        /// Session ID or display name
        target: String,

        /// JSON-RPC method name (e.g., query.capabilities)
        method: String,

        /// JSON params (optional, defaults to {})
        #[arg(short, long, default_value = "{}")]
        params: String,

        /// Output raw JSON-RPC response
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Run a command interactively in a PTY session — injects, waits for completion, returns output
    Interact {
        /// Session ID or display name
        target: String,

        /// Shell command to run
        command: String,

        /// Timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 200)
        #[arg(long, default_value = "200")]
        poll_ms: u64,

        /// Strip ANSI escape sequences from output
        #[arg(long)]
        strip_ansi: bool,

        /// Output as JSON {output, elapsed_ms, marker_found}
        #[arg(long)]
        json: bool,
    },

    /// Execute a shell command on a target session
    Exec {
        /// Session ID or display name
        target: String,

        /// Shell command to execute
        command: String,

        /// Working directory (optional)
        #[arg(long)]
        cwd: Option<String>,

        /// Timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Output as JSON (includes stdout, stderr, exit_code)
        #[arg(long)]
        json: bool,
    },

    /// Send a signal to a session's process (e.g., SIGTERM, SIGINT)
    Signal {
        /// Session ID or display name
        target: String,

        /// Signal name or number (e.g., TERM, INT, KILL, HUP, 15)
        signal: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        /// Remote hub address HOST:PORT — forwards the signal through the
        /// hub to the named session on that host (T-921 cross-host parity).
        #[arg(long = "target", value_name = "HOST:PORT")]
        hub: Option<String>,

        /// Path to a hex-encoded HMAC secret file for the remote hub.
        #[arg(long = "secret-file", value_name = "PATH")]
        secret_file: Option<std::path::PathBuf>,

        /// Explicit hex-encoded HMAC secret (64 chars / 32 bytes).
        /// Prefer --secret-file for shell-history hygiene.
        #[arg(long, value_name = "HEX")]
        secret: Option<String>,

        /// Auth scope: observe | interact | control | execute.
        /// Defaults to the per-method minimum (control for signal).
        #[arg(long, value_name = "NAME")]
        scope: Option<String>,
    },

    // === PTY Operations (grouped) ===

    /// PTY terminal operations (attach, inject, output, resize, stream)
    #[command(subcommand)]
    Pty(PtyCommand),

    // === Event System (grouped) ===

    /// Event system operations (watch, emit, broadcast, wait, topics, collect)
    #[command(subcommand)]
    Event(EventCommand),

    // === Hidden backward-compat aliases for PTY commands ===

    /// Read terminal output from a PTY-backed session
    #[command(hide = true)]
    Output {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,
        #[arg(short, long, default_value = "50")]
        lines: u64,
        #[arg(short, long)]
        bytes: Option<u64>,
        /// Strip ANSI escape sequences from output
        #[arg(long)]
        strip_ansi: bool,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Inject keystrokes into a PTY-backed session
    #[command(hide = true)]
    Inject {
        /// Session ID or display name
        target: String,
        text: String,
        #[arg(long, short = 'e')]
        enter: bool,
        #[arg(long, short)]
        key: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Attach to a PTY session
    #[command(hide = true)]
    Attach {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,
        #[arg(long, default_value = "100")]
        poll_ms: u64,
    },

    /// Resize a PTY session's terminal
    #[command(hide = true)]
    Resize {
        /// Session ID or display name
        target: String,
        cols: u16,
        rows: u16,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Stream a PTY session via data plane
    #[command(hide = true)]
    Stream {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,
    },

    /// Mirror a PTY session — read-only terminal output
    Mirror {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Number of scrollback lines to show on connect (default: 100)
        #[arg(long, default_value = "100")]
        scrollback: u64,

        /// Use legacy byte-passthrough path (no grid emulation)
        #[arg(long)]
        raw: bool,

        /// Mirror all sessions with this tag in a grid layout (mutually exclusive with target)
        #[arg(long, conflicts_with = "target")]
        tag: Option<String>,
    },

    // === Hidden backward-compat aliases for Event commands ===

    /// Poll events from a session's event bus
    #[command(hide = true)]
    Events {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,
        #[arg(long)]
        since: Option<u64>,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
        #[arg(long)]
        payload_only: bool,
    },

    /// Broadcast an event to multiple sessions via the hub
    #[command(hide = true)]
    Broadcast {
        topic: String,
        #[arg(short, long, default_value = "{}")]
        payload: String,
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Emit an event to a session's event bus
    #[command(hide = true)]
    Emit {
        /// Session ID or display name
        target: String,
        topic: String,
        #[arg(short, long, default_value = "{}")]
        payload: String,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Push an event to a target session's event bus via the hub
    #[command(name = "emit-to", hide = true)]
    EmitTo {
        target: String,
        topic: String,
        #[arg(short, long, default_value = "{}")]
        payload: String,
        #[arg(long)]
        from: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Watch events from one or more sessions in real-time
    #[command(hide = true)]
    Watch {
        #[arg(value_name = "TARGET")]
        targets: Vec<String>,
        #[arg(long, default_value = "500")]
        interval: u64,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "0")]
        timeout: u64,
        #[arg(long, default_value = "0")]
        count: u64,
        #[arg(long)]
        payload_only: bool,
        #[arg(long)]
        since: Option<u64>,
    },

    /// List event topics from one or all sessions
    #[command(hide = true)]
    Topics {
        target: Option<String>,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "5")]
        timeout: u64,
        #[arg(long)]
        no_header: bool,
    },

    /// Collect events from multiple sessions via hub (fan-in)
    #[command(hide = true)]
    Collect {
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long, default_value = "500")]
        interval: u64,
        #[arg(long, default_value = "0")]
        count: u64,
        #[arg(long)]
        json: bool,
        #[arg(long, default_value = "0")]
        timeout: u64,
        #[arg(long)]
        payload_only: bool,
        #[arg(long)]
        since: Option<u64>,
    },

    /// Wait for a session to emit an event matching a topic, then exit
    #[command(hide = true)]
    Wait {
        target: Option<String>,
        #[arg(long)]
        topic: String,
        #[arg(long, default_value = "0")]
        timeout: u64,
        #[arg(long, default_value = "250")]
        interval: u64,
        #[arg(long)]
        json: bool,
        #[arg(long)]
        since: Option<u64>,
    },

    // === Metadata & Discovery ===

    /// Update session tags, name, or roles at runtime
    Tag {
        /// Session ID or display name
        target: String,

        /// Set tags (replaces all existing)
        #[arg(long, value_delimiter = ',')]
        set: Vec<String>,

        /// Add tags
        #[arg(long, value_delimiter = ',')]
        add: Vec<String>,

        /// Remove tags
        #[arg(long, value_delimiter = ',')]
        remove: Vec<String>,

        /// Set the display name
        #[arg(long = "name")]
        new_name: Option<String>,

        /// Set roles (replaces all existing)
        #[arg(long, value_delimiter = ',')]
        role: Vec<String>,

        /// Add roles
        #[arg(long, value_delimiter = ',')]
        add_role: Vec<String>,

        /// Remove roles
        #[arg(long, value_delimiter = ',')]
        remove_role: Vec<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        /// Remote hub address HOST:PORT — forwards the tag update through
        /// the hub to the named session on that host (T-921 cross-host parity).
        #[arg(long = "target", value_name = "HOST:PORT")]
        hub: Option<String>,

        /// Path to a hex-encoded HMAC secret file for the remote hub.
        #[arg(long = "secret-file", value_name = "PATH")]
        secret_file: Option<std::path::PathBuf>,

        /// Explicit hex-encoded HMAC secret (64 chars / 32 bytes).
        #[arg(long, value_name = "HEX")]
        secret: Option<String>,

        /// Auth scope: observe | interact | control | execute.
        /// Defaults to observe (read) or interact (write).
        #[arg(long, value_name = "NAME")]
        scope: Option<String>,
    },

    /// Discover sessions by tag, role, capability, or name pattern
    Discover {
        /// Filter by tag (comma-separated, AND logic)
        #[arg(long, value_delimiter = ',')]
        tag: Vec<String>,

        /// Filter by role (comma-separated, AND logic)
        #[arg(long, value_delimiter = ',')]
        role: Vec<String>,

        /// Filter by capability (comma-separated, AND logic)
        #[arg(long, value_delimiter = ',')]
        cap: Vec<String>,

        /// Filter by display name (substring match)
        #[arg(long)]
        name: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Only print the session count (useful for scripting)
        #[arg(long)]
        count: bool,

        /// Print only the first matching session's display name (for piping)
        #[arg(long)]
        first: bool,

        /// Block until at least one session matches the filters
        #[arg(long)]
        wait: bool,

        /// Timeout in seconds when --wait is used (default: 30)
        #[arg(long, default_value = "30")]
        wait_timeout: u64,

        /// Output session ID instead of display name (with --first)
        #[arg(long)]
        id: bool,

        /// Print only session display names (one per line, for piping)
        #[arg(long)]
        names: bool,

        /// Print only session IDs (one per line, for piping)
        #[arg(long)]
        ids: bool,

        /// Suppress table header and footer (for piping/awk)
        #[arg(long)]
        no_header: bool,
    },

    /// Identify which session is running this caller (T-1299 / T-1297).
    ///
    /// Disambiguator chain:
    ///   1. `--session` flag, or `$TERMLINK_SESSION_ID` env var → exact match.
    ///   2. `--name <display_name>` → exact match (rejects on collision).
    ///   3. Neither → prints all live candidates so you can pick.
    Whoami {
        /// Session id hint. Defaults to `$TERMLINK_SESSION_ID` when set.
        #[arg(long)]
        session: Option<String>,

        /// Display-name hint (alternative to --session).
        #[arg(long)]
        name: Option<String>,

        /// Output as JSON.
        #[arg(long)]
        json: bool,
    },

    /// Manage key-value metadata on a session
    Kv {
        /// Session ID or display name
        target: String,

        /// Output result as JSON
        #[arg(global = true, long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(global = true, long, default_value = "5")]
        timeout: u64,

        /// Output raw value (strings without quotes, for piping)
        #[arg(global = true, long)]
        raw: bool,

        /// List only key names (one per line, for scripting)
        #[arg(global = true, long)]
        keys: bool,

        /// Remote hub address HOST:PORT — forwards the kv call through the
        /// hub to the named session on that host (T-921 cross-host parity).
        #[arg(global = true, long = "target", value_name = "HOST:PORT")]
        hub: Option<String>,

        /// Path to a hex-encoded HMAC secret file for the remote hub.
        #[arg(global = true, long = "secret-file", value_name = "PATH")]
        secret_file: Option<std::path::PathBuf>,

        /// Explicit hex-encoded HMAC secret (64 chars / 32 bytes).
        #[arg(global = true, long, value_name = "HEX")]
        secret: Option<String>,

        /// Auth scope: observe | interact | control | execute.
        /// Defaults to observe for get/list and interact for set/del.
        #[arg(global = true, long, value_name = "NAME")]
        scope: Option<String>,

        #[command(subcommand)]
        action: Option<KvAction>,
    },

    // === Execution ===

    /// Run a command in an ephemeral session (register, execute, deregister)
    Run {
        /// Display name for the ephemeral session
        #[arg(short, long)]
        name: Option<String>,

        /// Roles for the session (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        roles: Vec<String>,

        /// Tags for the session (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Capabilities for the session (comma-separated)
        #[arg(long, value_delimiter = ',')]
        cap: Vec<String>,

        /// Timeout in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Output result as JSON (exit_code, stdout, stderr, elapsed_ms)
        #[arg(long)]
        json: bool,

        /// Shell command to execute
        #[arg(trailing_var_arg = true, required = true)]
        command: Vec<String>,
    },

    /// Send a request event and wait for a reply (request-reply pattern)
    Request {
        /// Session ID or display name
        target: String,

        /// Request topic (e.g., "task.delegate")
        #[arg(long)]
        topic: String,

        /// JSON payload for the request
        #[arg(long, default_value = "{}")]
        payload: String,

        /// Topic to wait for as reply (e.g., "task.completed")
        #[arg(long)]
        reply_topic: String,

        /// Timeout in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 250)
        #[arg(long, default_value = "250")]
        interval: u64,

        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Spawn a command in a new terminal with TermLink session registration
    Spawn {
        /// Session display name
        #[arg(short, long)]
        name: Option<String>,

        /// Roles for the spawned session (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        roles: Vec<String>,

        /// Tags for the spawned session (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Capabilities for the spawned session (comma-separated)
        #[arg(long, value_delimiter = ',')]
        cap: Vec<String>,

        /// Environment variables for the session (repeatable: --env KEY=VALUE)
        #[arg(long = "env", value_name = "KEY=VALUE")]
        env_vars: Vec<String>,

        /// Wait for the session to register before returning
        #[arg(long)]
        wait: bool,

        /// Timeout in seconds for --wait (default: 30)
        #[arg(long, default_value = "30")]
        wait_timeout: u64,

        /// Start a PTY-backed shell session (no command needed)
        #[arg(long)]
        shell: bool,

        /// Spawn backend: auto, terminal (macOS Terminal.app), tmux, background
        #[arg(long, default_value = "auto")]
        backend: SpawnBackend,

        /// Output result as JSON (session_name, backend)
        #[arg(long)]
        json: bool,

        /// Command to run in the spawned terminal (after --)
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Dispatch N workers and collect results (atomic spawn+tag+collect)
    Dispatch {
        /// Number of workers to spawn
        #[arg(short = 'n', long)]
        count: u32,

        /// Timeout in seconds for result collection (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Event topic to collect (default: task.completed)
        #[arg(long, default_value = "task.completed")]
        topic: String,

        /// Worker name prefix (workers will be named prefix-1, prefix-2, ...)
        #[arg(long)]
        name: Option<String>,

        /// Roles for workers (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        roles: Vec<String>,

        /// Tags for workers (comma-separated; dispatch metadata tags added automatically)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Capabilities for workers (comma-separated)
        #[arg(long, value_delimiter = ',')]
        cap: Vec<String>,

        /// Environment variables for workers (repeatable: --env KEY=VALUE)
        #[arg(long = "env", value_name = "KEY=VALUE")]
        env_vars: Vec<String>,

        /// Spawn backend: auto, terminal, tmux, background
        #[arg(long, default_value = "auto")]
        backend: SpawnBackend,

        /// Working directory for workers (each worker will cd into this directory)
        #[arg(long)]
        workdir: Option<std::path::PathBuf>,

        /// Create a git worktree per worker for filesystem isolation
        #[arg(long)]
        isolate: bool,

        /// After collection, merge worker branches back to base (requires --isolate)
        #[arg(long)]
        auto_merge: bool,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// LLM model for workers: opus, sonnet, or haiku
        #[arg(long)]
        model: Option<String>,

        /// Command for each worker to run (after --)
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    /// Show status of dispatch manifest (pending, merged, conflict branches)
    #[command(name = "dispatch-status")]
    DispatchStatus {
        /// Exit non-zero if any pending branches exist (for pre-commit gate)
        #[arg(long)]
        check: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    // === Infrastructure ===

    /// Remove stale (dead) session registrations from the runtime directory
    Clean {
        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Suppress table header and footer
        #[arg(long)]
        no_header: bool,

        /// Only print the stale session count (for scripting)
        #[arg(long)]
        count: bool,
    },

    /// Hub server management (routes requests between sessions)
    Hub {
        #[command(subcommand)]
        action: Option<HubAction>,
    },

    /// MCP server — expose TermLink as structured tools for AI agents
    Mcp {
        #[command(subcommand)]
        action: McpAction,
    },

    // === Token Management ===

    /// Create or inspect capability tokens for session authentication
    Token {
        #[command(subcommand)]
        action: TokenAction,
    },

    // === Agent Communication ===

    /// Agent-to-agent communication (typed request/response protocol)
    Agent {
        #[command(subcommand)]
        action: AgentAction,
    },

    // === File Transfer ===

    /// Transfer files between sessions via chunked events
    File {
        #[command(subcommand)]
        action: FileAction,
    },

    // === Remote Operations ===

    /// Interact with sessions on remote hubs (cross-machine)
    Remote {
        #[command(subcommand)]
        action: RemoteAction,
    },

    /// Fleet-wide operations across all configured hubs (defaults to status)
    Fleet {
        #[command(subcommand)]
        action: Option<FleetAction>,
    },

    /// Network connectivity diagnostics (layered per-hub probe)
    Net {
        #[command(subcommand)]
        action: NetAction,
    },

    /// Query the hub's offline file inbox (pending transfers for offline sessions)
    Inbox {
        #[command(subcommand)]
        action: InboxAction,
    },

    /// Manage TOFU (Trust On First Use) certificate trust for remote hubs
    Tofu {
        #[command(subcommand)]
        action: TofuAction,
    },

    /// Manage agent cryptographic identity (ed25519 keypair, T-1159)
    Identity {
        #[command(subcommand)]
        action: IdentityAction,
    },

    /// Interact with the T-1155 agent communication bus — create/post/subscribe/list topics
    Channel {
        #[command(subcommand)]
        action: ChannelAction,
    },

    /// Check TermLink runtime health — validates dirs, sessions, hub, sockets
    Doctor {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Auto-fix issues (clean stale sessions, remove orphaned sockets/pidfiles)
        #[arg(long)]
        fix: bool,

        /// Exit non-zero on warnings (not just failures)
        #[arg(long)]
        strict: bool,

        /// Override the runtime directory to check (default: auto-detected from env/config)
        #[arg(long)]
        runtime_dir: Option<String>,
    },

    /// Vendor TermLink binary into project for path isolation
    Vendor {
        #[command(subcommand)]
        action: Option<VendorAction>,

        /// Source binary path (default: current executable)
        #[arg(long)]
        source: Option<String>,

        /// Target project directory (default: current directory)
        #[arg(long)]
        target: Option<String>,

        /// Show what would happen without copying
        #[arg(long)]
        dry_run: bool,

        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },

    /// Show version, build info, and git commit
    Version {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Output only the version number (for scripting)
        #[arg(long)]
        short: bool,
    },
}

/// Vendor subcommands
#[derive(Subcommand)]
pub(crate) enum VendorAction {
    /// Show vendor status (version, path, drift from global)
    Status {
        /// Target project directory (default: current directory)
        #[arg(long)]
        target: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Exit non-zero if vendor is outdated or misconfigured
        #[arg(long)]
        check: bool,
    },
}

/// Spawn backend for creating new terminal sessions
#[derive(Clone, Debug, clap::ValueEnum)]
pub(crate) enum SpawnBackend {
    /// Auto-detect: Terminal.app on macOS with GUI, tmux if available, background PTY fallback
    Auto,
    /// macOS Terminal.app via osascript
    Terminal,
    /// tmux detached session
    Tmux,
    /// Background PTY process (no terminal emulator)
    Background,
}

impl std::fmt::Display for SpawnBackend {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            SpawnBackend::Auto => write!(f, "auto"),
            SpawnBackend::Terminal => write!(f, "terminal"),
            SpawnBackend::Tmux => write!(f, "tmux"),
            SpawnBackend::Background => write!(f, "background"),
        }
    }
}

/// Hub server actions
#[derive(Subcommand)]
pub(crate) enum HubAction {
    /// Start the hub server (default if no subcommand given)
    Start {
        /// Optional TCP address to listen on (e.g., "0.0.0.0:9100", "127.0.0.1:9100")
        #[arg(long)]
        tcp: Option<String>,

        /// Output startup info as JSON (socket, pidfile, pid)
        #[arg(long)]
        json: bool,
    },
    /// Stop a running hub server
    Stop {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Restart the hub — spawn new process, then stop current (zero-downtime)
    Restart {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show hub server status
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// One-line status output (for scripting)
        #[arg(long)]
        short: bool,

        /// Exit non-zero if hub is not running (for scripting/health checks)
        #[arg(long)]
        check: bool,
    },
}

/// Hub inbox actions (T-997)
#[derive(Subcommand)]
pub(crate) enum InboxAction {
    /// Show inbox status — total pending transfers per target
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List pending transfers for a specific target session
    List {
        /// Target session name
        target: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Clear pending transfers from the inbox (specify a target name, or use --all for everything)
    Clear {
        /// Target session name — clear only this target's transfers (omit and use --all to clear everything)
        target: Option<String>,

        /// Clear all pending transfers for all targets (mutually exclusive with specifying a target)
        #[arg(long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// MCP server actions
#[derive(Subcommand)]
pub(crate) enum McpAction {
    /// Start the MCP server on stdio (for Claude Code, Cursor, etc.)
    Serve,
}

/// Remote hub operations (cross-machine)
#[derive(Subcommand)]
pub(crate) enum RemoteAction {
    /// Ping a remote hub or session (connectivity + latency check)
    Ping {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Optional: target session name to ping through the hub
        session: Option<String>,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "observe")]
        scope: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// List sessions on a remote hub
    List {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "observe")]
        scope: String,

        /// Filter by session name (substring match)
        #[arg(long)]
        name: Option<String>,

        /// Filter by tags (comma-separated, all must match)
        #[arg(long)]
        tags: Option<String>,

        /// Filter by roles (comma-separated, all must match)
        #[arg(long)]
        roles: Option<String>,

        /// Filter by capabilities (comma-separated, all must match)
        #[arg(long)]
        cap: Option<String>,

        /// Only print the session count
        #[arg(long)]
        count: bool,

        /// Output only the first matching session's display name (exits 1 if none)
        #[arg(long)]
        first: bool,

        /// Print only session display names (one per line, for piping)
        #[arg(long)]
        names: bool,

        /// Print only session IDs (one per line, for piping)
        #[arg(long)]
        ids: bool,

        /// Suppress table header and footer
        #[arg(long)]
        no_header: bool,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Get detailed status of a session on a remote hub
    Status {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Target session name or ID (omit to pick interactively)
        session: Option<String>,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "observe")]
        scope: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Output one-line summary (name state pid)
        #[arg(long)]
        short: bool,

        /// RPC timeout in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Inject keystrokes into a session on a remote hub
    Inject {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Target session name or ID
        session: String,

        /// Text to inject
        text: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Append Enter keystroke after message
        #[arg(long, short = 'e')]
        enter: bool,

        /// Send a special key instead of text (Enter, Tab, Escape, etc.)
        #[arg(long, short)]
        key: Option<String>,

        /// Inter-key delay in milliseconds [default: 10]
        #[arg(long, default_value = "10")]
        delay_ms: u64,

        /// Permission scope: observe, interact, control, execute [default: control]
        #[arg(long, default_value = "control")]
        scope: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// Send a file to a session on a remote hub
    SendFile {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Target session name or ID
        session: String,

        /// Path to the local file to send
        path: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Chunk size in bytes (default: 49152 = 48KB, ~64KB base64)
        #[arg(long, default_value = "49152")]
        chunk_size: usize,

        /// Permission scope: observe, interact, control, execute [default: control]
        #[arg(long, default_value = "control")]
        scope: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Transfer timeout in seconds (default: 60)
        #[arg(long, default_value = "60")]
        timeout: u64,
    },

    /// Watch events from sessions on a remote hub (continuous polling)
    Events {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "observe")]
        scope: String,

        /// Filter events by topic
        #[arg(long)]
        topic: Option<String>,

        /// Filter by target session names (comma-separated)
        #[arg(long)]
        targets: Option<String>,

        /// Poll interval in milliseconds
        #[arg(long, default_value = "500")]
        interval: u64,

        /// Stop after collecting N events (0 = unlimited)
        #[arg(long, default_value = "0")]
        count: u64,

        /// Output each event as a JSON line
        #[arg(long)]
        json: bool,

        /// Output only event payloads (one JSON per line, for piping)
        #[arg(long)]
        payload_only: bool,
    },

    /// Push a file or message to a remote session's inbox with PTY notification
    Push {
        /// Remote hub address or profile name
        hub: String,

        /// Target session name or ID
        session: String,

        /// File to push (omit for --message only)
        file: Option<String>,

        /// Inline text message (alternative to file)
        #[arg(long, short)]
        message: Option<String>,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "execute")]
        scope: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// Query inbox on a remote hub (pending transfers for offline sessions)
    Inbox {
        /// Remote hub address or profile name
        hub: String,

        #[command(subcommand)]
        action: Option<RemoteInboxAction>,

        /// Path to file containing 32-byte hex secret
        #[arg(global = true, long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(global = true, long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(global = true, long, default_value = "execute")]
        scope: String,

        /// RPC timeout in seconds (default: 10)
        #[arg(global = true, long, default_value = "10")]
        timeout: u64,
    },

    /// Health check a remote hub — connectivity, sessions, inbox, version
    Doctor {
        /// Remote hub address or profile name
        hub: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "execute")]
        scope: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Manage saved hub profiles (~/.termlink/hubs.toml)
    Profile {
        #[command(subcommand)]
        action: ProfileAction,
    },

    /// Execute a shell command on a remote session via hub routing
    Exec {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Target session name or ID
        session: String,

        /// Shell command to execute
        command: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly (less secure, for scripting)
        #[arg(long)]
        secret: Option<String>,

        /// Permission scope: observe, interact, control, execute
        #[arg(long, default_value = "execute")]
        scope: String,

        /// Command timeout in seconds
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Working directory for the command on the remote session
        #[arg(long)]
        cwd: Option<String>,

        /// Output result as JSON (exit_code, stdout, stderr)
        #[arg(long)]
        json: bool,
    },
}

/// TOFU (Trust On First Use) certificate trust management
#[derive(Subcommand)]
pub(crate) enum TofuAction {
    /// List all trusted hub certificates
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Clear a trusted hub certificate entry (allows re-trust on next connection)
    Clear {
        /// Host:port to clear (e.g., "192.168.10.109:9100"), or omit with --all
        host: Option<String>,

        /// Clear all TOFU entries
        #[arg(long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Agent cryptographic identity management (ed25519 keypair, T-1159)
#[derive(Subcommand)]
pub(crate) enum IdentityAction {
    /// Bootstrap a new ed25519 keypair at ~/.termlink/identity.key (chmod 600)
    Init {
        /// Overwrite an existing key (previous key is renamed to identity.key.bak-<ts>)
        #[arg(long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Print fingerprint + public key hex + file path for the current identity
    Show {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Rotate the current keypair (alias for `init --force` — explicit for operator intent)
    Rotate {
        /// Required — rotation is destructive (the old private key is renamed to a .bak file)
        #[arg(long)]
        force: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Channel bus subcommands (T-1160, T-1155)
#[derive(Subcommand)]
pub(crate) enum ChannelAction {
    /// Create a topic with a retention policy
    Create {
        /// Topic name (e.g. "broadcast:global", "channel:learnings")
        name: String,

        /// Retention: "forever", "days:N", or "messages:N" (default: forever)
        #[arg(long, default_value = "forever")]
        retention: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Post a signed envelope to a topic (reads payload from stdin if not given)
    Post {
        /// Topic name
        topic: String,

        /// Message type (free-form string, e.g. "note", "learning", "artifact")
        #[arg(long, default_value = "note")]
        msg_type: String,

        /// Payload (inline). If omitted, read from stdin.
        #[arg(long)]
        payload: Option<String>,

        /// Artifact reference (e.g. "ref://...") — optional opaque pointer
        #[arg(long)]
        artifact_ref: Option<String>,

        /// Override sender_id (default: the identity file fingerprint)
        #[arg(long)]
        sender_id: Option<String>,

        /// Reply to a parent envelope's offset within the same topic (T-1313,
        /// Matrix `m.in_reply_to` analogue). Sets `metadata.in_reply_to=<offset>`.
        #[arg(long)]
        reply_to: Option<u64>,

        /// Set arbitrary routing-hint metadata. Repeatable. Format: KEY=VALUE.
        /// Well-known keys: conversation_id, event_type, in_reply_to. T-1287/T-1313.
        #[arg(long = "metadata", value_name = "KEY=VALUE")]
        metadata: Vec<String>,

        /// Mention a recipient (Matrix `m.mention` analogue, T-1325).
        /// Repeatable: `--mention alice --mention bob`. Joined into
        /// `metadata.mentions=<csv>` on the envelope.
        #[arg(long = "mention", value_name = "ID")]
        mentions: Vec<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Direct-message a peer agent on the canonical `dm:<a>:<b>` topic (T-1319).
    /// Topic name is auto-resolved from your identity fingerprint plus the
    /// peer's identifier, sorted alphabetically. Auto-creates the topic with
    /// `forever` retention on first use. With no flags, opens the conversation
    /// in read mode (`--resume --reactions`); with `--send`, posts and exits.
    Dm {
        /// Peer identifier (typically their identity fingerprint, but any
        /// stable string works — both ends just need to agree). Used together
        /// with your own identity fingerprint to derive the topic name.
        /// Required unless `--list` is given.
        #[arg(required_unless_present = "list")]
        peer: Option<String>,

        /// Send a message to this DM and exit (instead of opening read mode)
        #[arg(long)]
        send: Option<String>,

        /// Threaded reply — sets metadata.in_reply_to=<offset>. Requires --send.
        #[arg(long, requires = "send")]
        reply_to: Option<u64>,

        /// Mention a recipient (T-1325). Repeatable. Requires --send.
        /// Joined into `metadata.mentions=<csv>` on the outbound post.
        #[arg(long = "mention", value_name = "ID", requires = "send")]
        mentions: Vec<String>,

        /// Print the canonical DM topic name and exit (helper for scripts)
        #[arg(long)]
        topic_only: bool,

        /// List existing DM topics for the caller's identity (T-1320).
        /// Mutually exclusive with `<peer>`; queries `channel.list` and
        /// filters topics matching `dm:<a>:<b>` where one side equals the
        /// caller's identity fingerprint.
        #[arg(long, conflicts_with_all = ["peer", "send", "reply_to", "topic_only"])]
        list: bool,

        /// Inbox view (T-1338): for each listed DM, walk the topic and
        /// compute the caller's unread count + first-unread offset.
        /// Sorts unread-first. Requires `--list`. Slower (one walk per
        /// DM) — opt-in.
        #[arg(long, requires = "list")]
        unread: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON (read mode → JSON-lines; --send → delivery JSON;
        /// --topic-only → JSON object)
        #[arg(long)]
        json: bool,
    },
    /// Post a read-receipt (Matrix `m.receipt` analogue) — shorthand for
    /// `channel post --msg-type receipt` carrying `metadata.up_to=<offset>`.
    /// Without `--up-to`, resolves to the topic's current latest offset
    /// (count-1 from `channel.list`). T-1315.
    Ack {
        /// Topic name
        topic: String,

        /// The offset up to which the sender has been seen. Inclusive.
        /// If omitted (and `--since` is also omitted), auto-resolves to
        /// the topic's current latest offset.
        #[arg(long, conflicts_with = "since")]
        up_to: Option<u64>,

        /// Resolve `up_to` from a timestamp anchor: walks the topic, finds
        /// the highest offset whose envelope has `ts >= <ms>`, and posts
        /// the receipt for it. Errors when no envelope satisfies. Mutually
        /// exclusive with `--up-to`. T-1337.
        #[arg(long, value_name = "MS", conflicts_with = "up_to")]
        since: Option<i64>,

        /// Override sender_id (default: the identity file fingerprint)
        #[arg(long)]
        sender_id: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show the latest read-receipt per sender on a topic (T-1315).
    /// Subscribes from offset 0, filters to `msg_type=receipt`, keeps the
    /// most-recent receipt per sender, prints sorted by sender_id.
    Receipts {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Recursive thread view (Matrix-style): root + all descendant replies,
    /// indented by depth. Walks the topic once, builds a parent→children map
    /// from `metadata.in_reply_to`, DFS-renders the subtree from `<root>`.
    /// Children are visited in ascending offset order. Read-only (T-1328).
    Thread {
        /// Topic name
        topic: String,

        /// Root envelope offset to render the subtree from
        root: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON (flat list with depth field, preserves DFS order)
        #[arg(long)]
        json: bool,
    },
    /// Trace the reply chain UP from a leaf envelope to the root (inverse
    /// of `channel thread`, which walks DOWN). Walks the topic once, indexes
    /// every envelope by offset, then follows `metadata.in_reply_to` from
    /// `<offset>` toward the root, capping recursion at 1024 to defeat
    /// pathological cycles. Renders in root→leaf order so reading top-down
    /// matches the natural conversation flow. Read-only (T-1340).
    Ancestors {
        /// Topic name
        topic: String,

        /// Leaf envelope offset to start the upward walk from
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON (array of envelope records in root→leaf order,
        /// each `{offset, sender_id, msg_type, ts, payload}`)
        #[arg(long)]
        json: bool,
    },
    /// Copy an envelope from one topic to another, preserving original
    /// payload + msg_type (T-1348). Matrix-style forwarding analogue.
    /// The new envelope on `<dst_topic>` has `sender_id` = the forwarder
    /// (current identity) and metadata
    /// `forwarded_from=<src_topic>:<offset>` + `forwarded_sender=<original
    /// sender_id>` so readers can trace provenance.
    Forward {
        /// Source topic to copy the envelope FROM
        src_topic: String,

        /// Offset in src_topic of the envelope to copy
        offset: u64,

        /// Destination topic to copy the envelope TO
        dst_topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Emit or list ephemeral typing indicators (T-1351). Matrix-style
    /// `m.typing` analogue, append-only: emit writes a `msg_type=typing`
    /// envelope with `metadata.expires_at_ms=<now+ttl>`. List walks the
    /// topic and reports senders whose latest typing envelope has not
    /// expired. Default TTL: 30000ms (Matrix's 30s typing window).
    Typing {
        /// Topic name
        topic: String,

        /// Emit a typing indicator (default is list mode)
        #[arg(long)]
        emit: bool,

        /// TTL in milliseconds for the emitted typing indicator (default
        /// 30000). Only meaningful with `--emit`.
        #[arg(long, default_value_t = 30000u64, value_name = "MS")]
        ttl_ms: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON (list mode: array of `{sender_id, expires_at_ms,
        /// ts}`). Emit mode: passes through to `channel.post`.
        #[arg(long)]
        json: bool,
    },
    /// Pin or unpin an envelope on a topic (T-1345). Matrix-style
    /// `m.room.pinned_events` analogue, append-only: emits a `msg_type=pin`
    /// envelope carrying `metadata.pin_target=<offset>` and
    /// `metadata.action=pin|unpin`. The current pin set is computed by
    /// walking the topic (see `channel pinned`).
    Pin {
        /// Topic name
        topic: String,

        /// Offset of the envelope to pin / unpin
        offset: u64,

        /// Reverse the operation — emit `metadata.action=unpin` instead of
        /// `pin`. Latest action per target wins.
        #[arg(long)]
        unpin: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show the current pin set for a topic (T-1345). Walks the topic, applies
    /// pin/unpin events in offset order (latest action per target wins), and
    /// renders one row per actively-pinned target. Sorted by most-recently
    /// pinned descending. Read-only.
    Pinned {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{target, pinned_by, pinned_ts, payload}]`
        #[arg(long)]
        json: bool,
    },
    /// Render an envelope inline with its parent quoted above it (T-1344).
    /// If `<offset>` carries `metadata.in_reply_to=<parent>`, the parent
    /// envelope is fetched and rendered as a `>` quoted block before the
    /// child line. If `<offset>` is not a reply, the child is rendered alone
    /// with a "no parent" note. JSON form returns `{topic, child, parent}`
    /// with `parent: null` for orphans. Walks the topic once. Read-only.
    Quote {
        /// Topic name
        topic: String,

        /// Offset of the envelope to quote-render
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `{topic, child: {...}, parent: {...}|null}`
        #[arg(long)]
        json: bool,
    },
    /// Synthesized topic view: description + retention + post count + top
    /// senders + latest receipts in one shot. Read-only, no state mutation.
    /// Walks the topic once and renders a human-readable summary; pass `--json`
    /// for machine-readable output (T-1324).
    Info {
        /// Topic name
        topic: String,

        /// Restrict description/senders/receipts to records with
        /// `ts_unix_ms >= <ms>`. Total `Posts:` count remains unbounded;
        /// a `(N since <ms>)` parenthetical is added. (T-1331)
        #[arg(long, value_name = "MS")]
        since: Option<i64>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Membership list for one topic (T-1341). Walks the topic once and
    /// reports each distinct sender with post-count, first-seen ts, and
    /// last-seen ts. Sorted by last-seen descending. Counts only content
    /// envelopes by default — pass `--include-meta` to include reactions,
    /// edits, redactions, receipts, and topic_metadata. Read-only.
    Members {
        /// Topic name
        topic: String,

        /// Include meta envelopes (T-1332's set: receipt, reaction,
        /// redaction, edit, topic_metadata) in the per-sender post count
        /// and last-seen timestamp.
        #[arg(long)]
        include_meta: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON array of `{sender_id, posts, first_ts, last_ts}`
        #[arg(long)]
        json: bool,
    },
    /// Reply to the topic's latest content message — auto-resolves the
    /// reply-to offset so interactive use doesn't need a separate
    /// subscribe lookup. Skips meta envelopes (reactions, edits,
    /// redactions, receipts, topic_metadata) when picking the parent.
    /// Errors if the topic has no content yet. (T-1334)
    Reply {
        /// Topic name
        topic: String,

        /// Reply text (sent as the new message's payload)
        payload: String,

        /// One or more mention ids (or `*` for @room) — passed through to
        /// `metadata.mentions` (T-1325 / T-1333)
        #[arg(long, value_name = "ID")]
        mention: Vec<String>,

        /// Override sender_id (default: this identity's fingerprint)
        #[arg(long)]
        sender_id: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Show unread count for a sender on a topic (T-1332). Looks up the
    /// sender's latest `m.receipt.up_to`, walks the topic past that offset,
    /// and counts content envelopes (excludes meta types: receipt, reaction,
    /// redaction, edit, topic_metadata). Slack-style "3 new" UX.
    Unread {
        /// Topic name
        topic: String,

        /// Sender to compute unread for (default: this identity's fingerprint).
        #[arg(long)]
        sender: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Set/update the topic's free-text description (Matrix `m.room.topic`
    /// analogue). Emits a `msg_type=topic_metadata` envelope with
    /// `metadata.description=<text>`; repeat calls add new records and the
    /// reader picks the most recent by ts_ms (T-1323).
    Describe {
        /// Topic name
        topic: String,

        /// New description text (free-form)
        description: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Retract an earlier post (Matrix `m.redaction` analogue) — emits a
    /// `msg_type=redaction` envelope with `metadata.redacts=<offset>` and
    /// optional `metadata.reason=<text>`. Append-only: hub keeps the
    /// original; readers may opt to hide it via `subscribe --hide-redacted`.
    /// Default render shows redactions explicitly so the audit trail is
    /// visible (T-1322).
    Redact {
        /// Topic name
        topic: String,

        /// Offset of the post to retract
        redacts: u64,

        /// Optional reason for the redaction (free-form text, surfaced in
        /// the explicit render: `[N redact] sender → offset M (reason: ...)`)
        #[arg(long)]
        reason: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Replace an earlier post (Matrix `m.replace` analogue) — emits a
    /// `msg_type=edit` envelope with `metadata.replaces=<offset>` carrying
    /// the new payload. Append-only: hub keeps both records. Reader-side
    /// `subscribe --collapse-edits` renders only the latest version (T-1321).
    Edit {
        /// Topic name
        topic: String,

        /// Original envelope offset being replaced
        replaces: u64,

        /// New payload (the corrected message body)
        payload: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Post a reaction (Matrix `m.annotation` analogue) — shorthand for
    /// `channel post --msg-type reaction --reply-to <parent>` (T-1314).
    /// With `--remove`, finds the latest matching reaction this identity
    /// posted on the same parent with the same payload, and emits an
    /// `m.redaction` targeting that offset (T-1330).
    React {
        /// Topic name
        topic: String,

        /// Parent envelope offset within the topic
        parent_offset: u64,

        /// Reaction payload (typically a single emoji or short tag, e.g. "👍", "ack")
        reaction: String,

        /// Override sender_id (default: the identity file fingerprint)
        #[arg(long)]
        sender_id: Option<String>,

        /// Remove a previously-posted reaction by this identity matching
        /// (parent, reaction). Emits `m.redaction` targeting the latest
        /// matching reaction's offset. Errors if no match found. (T-1330)
        #[arg(long)]
        remove: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Read messages from a topic starting at a cursor (prints one envelope per line)
    Subscribe {
        /// Topic name
        topic: String,

        /// Cursor to start from (default: 0). Ignored when --resume is set
        /// and a stored cursor exists for this (topic, identity) pair.
        #[arg(long, default_value_t = 0u64)]
        cursor: u64,

        /// Resume from the locally persisted cursor for this (topic, identity)
        /// pair (T-1318, Matrix `next_batch` analogue). Stored in
        /// `~/.termlink/cursors.json`. After a successful subscribe, the
        /// cursor advances and is written back. No-op if no entry exists.
        /// Mutually exclusive with --reset.
        #[arg(long, conflicts_with = "reset")]
        resume: bool,

        /// Clear the persisted cursor for this (topic, identity) pair before
        /// starting. T-1318. Mutually exclusive with --resume.
        #[arg(long, conflicts_with = "resume")]
        reset: bool,

        /// Maximum messages per poll (default: 100, max 1000)
        #[arg(long, default_value_t = 100u64)]
        limit: u64,

        /// Keep polling every 1s instead of a one-shot drain
        #[arg(long)]
        follow: bool,

        /// Filter to envelopes whose `metadata.conversation_id` matches (T-1287).
        #[arg(long)]
        conversation_id: Option<String>,

        /// Filter to replies to a specific parent offset (T-1313 — only envelopes
        /// whose `metadata.in_reply_to == <offset>` are returned).
        #[arg(long)]
        in_reply_to: Option<u64>,

        /// Aggregate reaction envelopes under their parent (T-1314).
        /// In non-JSON mode, reaction envelopes are NOT printed as standalone
        /// lines; instead the parent line gets a trailing `(👍 ×3, 👀 ×1)`
        /// summary. JSON mode is unaffected — all envelopes still emit.
        #[arg(long)]
        reactions: bool,

        /// Show per-reactor identities in the reactions summary (T-1317).
        /// Requires `--reactions`. Renders `👍 by alice, bob` instead of
        /// the count form `👍 ×2`. Sender order is first-seen.
        #[arg(long, requires = "reactions")]
        by_sender: bool,

        /// Collapse `msg_type=edit` envelopes into the original post (T-1321).
        /// In rendered view the original offset shows the latest edit text
        /// with a `(edited)` marker; the standalone edit envelopes are hidden.
        /// JSON mode is unaffected.
        #[arg(long)]
        collapse_edits: bool,

        /// Hide redacted parents and the redaction envelopes themselves (T-1322).
        /// Default behavior renders redactions explicitly so the operator can
        /// audit what was retracted; this flag opts into a "clean" view.
        /// JSON mode is unaffected.
        #[arg(long)]
        hide_redacted: bool,

        /// Show only envelopes that mention `<id>` in `metadata.mentions`
        /// (Matrix `m.mention` analogue, T-1325). Strict comma-split + trim
        /// on the CSV; substring matches do not count. JSON mode unaffected.
        #[arg(long, value_name = "ID")]
        filter_mentions: Option<String>,

        /// Drop envelopes whose `ts < <ms>` from the printed output. Pure
        /// render-side filter — cursor/pagination behavior is unchanged.
        /// Mirrors `channel info --since` (T-1331) and `channel ack --since`
        /// (T-1337). T-1343.
        #[arg(long, value_name = "MS")]
        since: Option<i64>,

        /// Render each reply with its parent quoted on a preceding `>` line
        /// (T-1344). Seeds an offset-keyed cache from a one-time topic walk
        /// at startup so existing parents are available; new envelopes during
        /// `--follow` are added to the cache as they stream. JSON mode adds
        /// a `parent` field to each emitted envelope (`null` when not a
        /// reply or parent missing).
        #[arg(long)]
        show_parent: bool,

        /// Render only the last N envelopes after all aggregation/filter
        /// passes (T-1346). Pure render-side slice — pagination behavior is
        /// unchanged. Conflicts with `--follow` (tail of an unbounded
        /// stream is ill-defined). Order is preserved (oldest of the
        /// last-N first).
        #[arg(long, value_name = "N", conflicts_with = "follow")]
        tail: Option<usize>,

        /// Filter envelopes to only those whose `sender_id` is in the CSV
        /// (T-1347). Strict equality (comma-split + trim, no substring
        /// match). Empty entries ignored. JSON mode applies the same
        /// filter. Composes with all other render passes — reactions, edits
        /// and redactions still process the full set; only the rendered
        /// subset is filtered.
        #[arg(long, value_name = "CSV")]
        senders: Option<String>,

        /// Render forwarded envelopes (those carrying `metadata.forwarded_from`,
        /// emitted by `channel forward`, T-1348) with a `[fwd from <src>:<off>
        /// by <orig_sender>]` prefix line above the main render line. Pure
        /// render-side hint; no protocol change. T-1349.
        #[arg(long)]
        show_forwards: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON-lines
        #[arg(long)]
        json: bool,
    },
    /// List existing topics (optional prefix filter)
    List {
        /// Filter by topic prefix
        #[arg(long)]
        prefix: Option<String>,

        /// Per-topic content/meta breakdown — for each listed topic, walks it
        /// once and reports `content=N | meta=M | senders=S | first..last`.
        /// "meta" matches T-1332's set: receipt, reaction, redaction, edit,
        /// topic_metadata. Read-only. Slower (one walk per topic). T-1335.
        #[arg(long)]
        stats: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Cross-topic @-mentions inbox (T-1339). Walks every topic (or
    /// just those matching `--prefix`), filters envelopes whose
    /// `metadata.mentions` CSV contains the target id (default: caller's
    /// identity fingerprint) or the wildcard `*` (@room — T-1333). Read-
    /// only. Skips meta envelopes (UNREAD_META_TYPES) — only content
    /// counts toward the inbox.
    Mentions {
        /// Identity to look up mentions for (default: caller's fingerprint).
        /// `*` matches every csv that has any non-empty mention list.
        #[arg(long = "for", value_name = "ID")]
        target: Option<String>,

        /// Restrict the scan to topics whose name starts with this prefix.
        /// Useful for large hubs — without it every topic is walked.
        #[arg(long)]
        prefix: Option<String>,

        /// Cap the number of printed hits (0 = unlimited; default: 0)
        #[arg(long, default_value_t = 0u64)]
        limit: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as flat JSON array of `{topic, offset, sender_id, ts,
        /// payload}` records (no per-topic grouping).
        #[arg(long)]
        json: bool,
    },
    /// Read-only payload grep across one topic. Walks the topic once,
    /// decodes each envelope's payload from base64 (UTF-8 lossy), and
    /// prints matches by ascending offset. Default mode: case-insensitive
    /// substring. `--case-sensitive` makes substring exact; `--regex`
    /// switches to full regex compilation. Meta envelopes (T-1332's set:
    /// receipt, reaction, redaction, edit, topic_metadata) are excluded
    /// unless `--all` is given. Tier-A: pattern stays client-side, the
    /// hub never sees it. T-1336.
    Search {
        /// Topic name
        topic: String,

        /// Pattern (substring by default; full regex with `--regex`)
        pattern: String,

        /// Treat the pattern as a regex (compiled once via the `regex` crate)
        #[arg(long)]
        regex: bool,

        /// Disable case folding (default is case-insensitive substring)
        #[arg(long)]
        case_sensitive: bool,

        /// Search through meta envelopes too — by default reactions, edits,
        /// redactions, receipts, and topic_metadata records are skipped.
        #[arg(long)]
        all: bool,

        /// Cap the number of printed matches (0 = unlimited; default: 0)
        #[arg(long, default_value_t = 0u64)]
        limit: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON array of `{offset, sender_id, ts, msg_type, payload}`
        #[arg(long)]
        json: bool,
    },
    /// Show local offline-queue status — pending-post count + oldest timestamp (T-1161 diagnostic)
    QueueStatus {
        /// Path to the queue sqlite file (default: ~/.termlink/outbound.sqlite, or $TERMLINK_IDENTITY_DIR/outbound.sqlite if set)
        #[arg(long)]
        queue_path: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Fleet-wide operations across all configured hubs
#[derive(Subcommand)]
pub(crate) enum FleetAction {
    /// One-screen operational overview — shows every hub's status, sessions, version, and actions needed
    Status {
        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout per hub in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,

        /// Show session names per hub
        #[arg(short, long)]
        verbose: bool,
    },

    /// Health check all hubs in ~/.termlink/hubs.toml
    Doctor {
        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// RPC timeout per hub in seconds (default: 10)
        #[arg(long, default_value = "10")]
        timeout: u64,
    },

    /// Heal a hub's cached secret. Without `--bootstrap-from` this prints the
    /// copy-pasteable incantation (Tier-1, T-1054). With `--bootstrap-from
    /// <SOURCE>` it actually performs the heal (Tier-2, T-1055, R2 compliance).
    ///
    /// SOURCE forms:
    ///   file:<path>     — read the hex secret from a local file
    ///   ssh:<host>      — run `ssh <host> -- sudo cat /var/lib/termlink/hub.secret`
    ///
    /// The source MUST be out-of-band (its trust anchor must not itself depend
    /// on termlink auth).
    Reauth {
        /// Hub profile name as configured in ~/.termlink/hubs.toml
        profile: String,

        /// Out-of-band trust anchor that delivers the new secret.
        /// When omitted, this command prints the heal incantation instead of
        /// performing the heal.
        #[arg(long = "bootstrap-from")]
        bootstrap_from: Option<String>,
    },
}

/// Network connectivity diagnostics (T-1106)
#[derive(Subcommand)]
pub(crate) enum NetAction {
    /// Run layered connectivity probe (TCP → TLS → AUTH → PING) per hub
    ///
    /// Complements `fleet status` with per-layer diagnostics that pinpoint
    /// exactly where a broken hub connection fails. Useful for VPN/mesh
    /// troubleshooting: if TCP fails it's a network issue; if TLS fails
    /// it's a cert issue; if AUTH fails it's a secret mismatch.
    Test {
        /// Filter to a single hub profile name (default: test all)
        #[arg(long)]
        profile: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout per layer in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },
}

/// Profile management actions
#[derive(Subcommand)]
pub(crate) enum ProfileAction {
    /// Add or update a hub profile
    Add {
        /// Profile name (e.g., "lab", "prod")
        name: String,

        /// Hub address (host:port)
        address: String,

        /// Path to file containing 32-byte hex secret
        #[arg(long)]
        secret_file: Option<String>,

        /// Hex secret directly
        #[arg(long)]
        secret: Option<String>,

        /// Default permission scope
        #[arg(long)]
        scope: Option<String>,

        /// T-1291: declared out-of-band trust anchor for `fleet reauth --bootstrap-from auto`.
        /// Same scheme vocabulary as T-1055: `file:<path>` or `ssh:<host>`.
        #[arg(long)]
        bootstrap_from: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// List saved hub profiles
    List {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Suppress table header and separator
        #[arg(long)]
        no_header: bool,
    },

    /// Remove a hub profile
    Remove {
        /// Profile name to remove
        name: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Remote inbox actions (T-1009)
#[derive(Subcommand)]
pub(crate) enum RemoteInboxAction {
    /// Show inbox status on the remote hub — total pending transfers per target
    Status {
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List pending transfers for a specific target on the remote hub
    List {
        /// Target session name
        target: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Clear pending transfers on the remote hub (specify a target name, or use --all)
    Clear {
        /// Target session name
        target: Option<String>,

        /// Clear all targets
        #[arg(long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Token management actions
#[derive(Subcommand)]
pub(crate) enum TokenAction {
    /// Create a new capability token for a session
    Create {
        /// Session ID or display name (must have token_secret enabled)
        target: String,

        /// Permission scope: observe, interact, control, execute
        #[arg(short, long, default_value = "observe")]
        scope: String,

        /// Time-to-live in seconds (default: 3600 = 1 hour)
        #[arg(long, default_value = "3600")]
        ttl: u64,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Inspect a token without validating (decode the payload)
    Inspect {
        /// The token string to inspect
        token: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
}

/// Agent communication actions
#[derive(Subcommand)]
pub(crate) enum AgentAction {
    /// Send a typed agent request and wait for the response
    Ask {
        /// Target session ID or display name
        target: String,

        /// Action to request (e.g., "query.status", "task.run", "ping")
        #[arg(long)]
        action: String,

        /// JSON parameters for the action (default: {})
        #[arg(long, default_value = "{}")]
        params: String,

        /// Sender identity (default: CLI-<pid>)
        #[arg(long)]
        from: Option<String>,

        /// Timeout in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 250)
        #[arg(long, default_value = "250")]
        interval: u64,

        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Listen for incoming agent requests on a session
    Listen {
        /// Target session ID or display name
        target: String,

        /// Timeout in seconds (0 = listen forever, default: 0)
        #[arg(long, default_value = "0")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 250)
        #[arg(long, default_value = "250")]
        interval: u64,

        /// Output each request as a JSON line (NDJSON)
        #[arg(long)]
        json: bool,
    },

    /// Run a 4-phase format negotiation with a specialist session
    Negotiate {
        /// Specialist session ID or display name
        specialist: String,

        /// JSON Schema for the expected format (or @file for file path)
        #[arg(long)]
        schema: String,

        /// JSON draft to submit as the initial attempt
        #[arg(long)]
        draft: String,

        /// Sender identity (default: CLI-<pid>)
        #[arg(long)]
        from: Option<String>,

        /// Maximum correction rounds (default: 5)
        #[arg(long, default_value = "5")]
        max_rounds: u8,

        /// Timeout per round in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 250)
        #[arg(long, default_value = "250")]
        interval: u64,

        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },
}

/// File transfer actions
#[derive(Subcommand)]
pub(crate) enum FileAction {
    /// Send a file to a target session
    Send {
        /// Target session ID or display name
        target: String,

        /// Path to the file to send
        path: String,

        /// Chunk size in bytes (default: 49152 = 48KB, ~64KB base64)
        #[arg(long, default_value = "49152")]
        chunk_size: usize,

        /// Output transfer result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout per RPC call in seconds (default: 30)
        #[arg(long, default_value = "30")]
        timeout: u64,
    },

    /// Receive a file from a session (waits for file.init event). By default, only processes
    /// fresh events arriving after the receiver starts. Use --replay to process historical events.
    Receive {
        /// Source session ID or display name to watch for file events
        target: String,

        /// Output directory (default: current directory)
        #[arg(long, default_value = ".")]
        output_dir: String,

        /// Timeout in seconds (default: 300)
        #[arg(long, default_value = "300")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 100)
        #[arg(long, default_value = "100")]
        interval: u64,

        /// Process historical events from the event store (inbox pickup). Without this flag,
        /// only events arriving after the receiver starts are processed — preventing stale
        /// transfers from being assembled.
        #[arg(long)]
        replay: bool,

        /// Output transfer result as JSON
        #[arg(long)]
        json: bool,
    },
}

/// PTY terminal operations
#[derive(Subcommand)]
pub(crate) enum PtyCommand {
    /// Read terminal output from a PTY-backed session
    Output {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Number of lines to read (default: 50)
        #[arg(short, long, default_value = "50")]
        lines: u64,

        /// Read by bytes instead of lines
        #[arg(short, long)]
        bytes: Option<u64>,

        /// Strip ANSI escape sequences from output
        #[arg(long)]
        strip_ansi: bool,

        /// Output as JSON (includes output text and byte count)
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Inject keystrokes into a PTY-backed session
    Inject {
        /// Session ID or display name
        target: String,

        /// Text to inject (e.g., "ls -la")
        text: String,

        /// Append Enter key after text
        #[arg(long, short = 'e')]
        enter: bool,

        /// Send a named key instead of text (e.g., Escape, Tab, Up, Down)
        #[arg(long, short)]
        key: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Attach to a PTY session — live output and keyboard forwarding
    Attach {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Output poll interval in milliseconds (default: 100)
        #[arg(long, default_value = "100")]
        poll_ms: u64,
    },

    /// Resize a PTY session's terminal
    Resize {
        /// Session ID or display name
        target: String,

        /// Number of columns
        cols: u16,

        /// Number of rows
        rows: u16,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Stream a PTY session via data plane (real-time binary frames, zero polling)
    Stream {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,
    },

    /// Mirror a PTY session — read-only terminal output (no input forwarded)
    Mirror {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Number of scrollback lines to show on connect (default: 100)
        #[arg(long, default_value = "100")]
        scrollback: u64,

        /// Use legacy byte-passthrough path (no grid emulation)
        #[arg(long)]
        raw: bool,

        /// Mirror all sessions with this tag in a grid layout (mutually exclusive with target)
        #[arg(long, conflicts_with = "target")]
        tag: Option<String>,
    },
}

/// Event system operations
#[derive(Subcommand)]
pub(crate) enum EventCommand {
    /// Poll events from a session's event bus
    Poll {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Only show events after this sequence number (omit for all)
        #[arg(long)]
        since: Option<u64>,

        /// Filter by topic
        #[arg(long)]
        topic: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        /// Output only event payloads (one JSON object per line, for piping)
        #[arg(long)]
        payload_only: bool,
    },

    /// Watch events from one or more sessions in real-time
    Watch {
        /// Session IDs or display names (omit for all sessions)
        #[arg(value_name = "TARGET")]
        targets: Vec<String>,

        /// Poll interval in milliseconds (default: 500)
        #[arg(long, default_value = "500")]
        interval: u64,

        /// Filter by event topic
        #[arg(long)]
        topic: Option<String>,

        /// Output each event as a JSON line (NDJSON)
        #[arg(long)]
        json: bool,

        /// Exit after N seconds (0 = no timeout)
        #[arg(long, default_value = "0")]
        timeout: u64,

        /// Exit after receiving N events (0 = continuous)
        #[arg(long, default_value = "0")]
        count: u64,

        /// Output only event payloads (one JSON per line)
        #[arg(long)]
        payload_only: bool,

        /// Start from this sequence number (replay history from seq onwards)
        #[arg(long)]
        since: Option<u64>,
    },

    /// Emit an event to a session's event bus
    Emit {
        /// Session ID or display name
        target: String,

        /// Event topic (e.g., "build.complete", "test.failed")
        topic: String,

        /// JSON payload (optional, defaults to {})
        #[arg(short, long, default_value = "{}")]
        payload: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Push an event to a target session's event bus via the hub
    #[command(name = "emit-to")]
    EmitTo {
        /// Target session ID or display name
        target: String,

        /// Event topic (e.g., "task.result", "negotiate.offer")
        topic: String,

        /// JSON payload (optional, defaults to {})
        #[arg(short, long, default_value = "{}")]
        payload: String,

        /// Sender session ID (for traceability)
        #[arg(long)]
        from: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Broadcast an event to multiple sessions via the hub
    Broadcast {
        /// Event topic (e.g., "deploy.start", "alert.fire")
        topic: String,

        /// JSON payload (optional, defaults to {})
        #[arg(short, long, default_value = "{}")]
        payload: String,

        /// Target specific sessions (omit for all)
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Wait for a session to emit an event matching a topic, then exit
    Wait {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Event topic to wait for (required)
        #[arg(long)]
        topic: String,

        /// Timeout in seconds (0 = wait forever, default: 0)
        #[arg(long, default_value = "0")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 250)
        #[arg(long, default_value = "250")]
        interval: u64,

        /// Output matched event as JSON
        #[arg(long)]
        json: bool,

        /// Only consider events after this sequence number
        #[arg(long)]
        since: Option<u64>,
    },

    /// List event topics from one or all sessions
    Topics {
        /// Session ID or display name (omit for all sessions)
        target: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        /// Suppress session name headers and summary footer
        #[arg(long)]
        no_header: bool,
    },

    /// Collect events from multiple sessions via hub (fan-in)
    Collect {
        /// Target specific sessions (omit for all)
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,

        /// Filter by event topic
        #[arg(long)]
        topic: Option<String>,

        /// Poll interval in milliseconds (default: 500)
        #[arg(long, default_value = "500")]
        interval: u64,

        /// Exit after receiving N events (0 = continuous)
        #[arg(long, default_value = "0")]
        count: u64,

        /// Output each event as a JSON line (NDJSON)
        #[arg(long)]
        json: bool,

        /// Exit after N seconds (0 = no timeout)
        #[arg(long, default_value = "0")]
        timeout: u64,

        /// Output only event payloads (one JSON per line)
        #[arg(long)]
        payload_only: bool,

        /// Start from this sequence number for all sessions (replay history)
        #[arg(long)]
        since: Option<u64>,
    },
}

#[derive(Subcommand)]
pub(crate) enum KvAction {
    /// Set a key-value pair
    Set {
        /// Key name
        key: String,
        /// Value (JSON string, number, bool, object, or array)
        value: String,
    },
    /// Get a value by key
    Get {
        /// Key name
        key: String,
    },
    /// List all key-value pairs
    List,
    /// Delete a key
    Del {
        /// Key name
        key: String,
    },
}
