use clap::{Parser, Subcommand};

#[derive(Parser)]
#[command(
    name = "termlink",
    about = "Cross-terminal session communication",
    version,
    disable_help_subcommand = true,
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

        /// Per-agent ed25519 identity key file (T-1693 Shape 1, T-1700).
        /// When set, this session signs and registers with the identity at
        /// the given file path instead of the host-shared
        /// `$HOME/.termlink/identity.key`. The file is auto-created (chmod
        /// 600) on first use. Required on shared hosts where co-resident
        /// agents must present distinct envelope identities.
        #[arg(long, value_name = "PATH")]
        identity_key: Option<std::path::PathBuf>,
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

    /// Browse the MCP tool registry from the shell — parity with termlink_help (T-2002, cycle 13 #1)
    ///
    /// Same axis surface as the MCP `termlink_help` tool. Default render is a
    /// human-readable categorized listing; `--json` emits the raw envelope for
    /// piping to `jq`. Cold-start canonical call:
    ///
    ///   termlink help --json --sort-by required_arity --limit 20 \
    ///     --exclude-deprecated --categories channel,agent_chat \
    ///     --fields name,parameter_required_count
    ///
    /// T-2004: positional `<target>` ergonomic shortcut:
    ///   termlink help channel             → equivalent to --name-filter channel
    ///   termlink help termlink_channel_post  → equivalent to --tool-detail (exact match)
    Help {
        /// Optional target: exact tool name → drill in; substring → search names+descriptions (T-2004)
        target: Option<String>,

        /// Output as JSON (raw envelope; matches MCP termlink_help shape)
        #[arg(long)]
        json: bool,

        /// Scope to one category (e.g. `channel`, `agent_chat`)
        #[arg(long)]
        category: Option<String>,

        /// Substring search across tool names and descriptions (case-insensitive, multi-token AND)
        #[arg(long)]
        name_filter: Option<String>,

        /// Return category names + tool counts only (cold-start discovery)
        #[arg(long)]
        list_categories: bool,

        /// Drill in on one tool — returns full description + parameters + related tools
        #[arg(long)]
        tool_detail: Option<String>,

        /// Aggregate registry stats (total tools, deprecated counts, top/bottom categories)
        #[arg(long)]
        summary: bool,

        /// Canonical entry-point tool per category (~27-tool starter set)
        #[arg(long)]
        essentials: bool,

        /// Filter to tools with parameter_count <= N
        #[arg(long)]
        max_parameters: Option<usize>,

        /// Filter to tools with parameter_count >= N
        #[arg(long)]
        min_parameters: Option<usize>,

        /// Hide retirement-WIP tools (deprecated=true) from results
        #[arg(long)]
        exclude_deprecated: bool,

        /// Show ONLY retirement-WIP tools (deprecated=true)
        #[arg(long)]
        deprecated_only: bool,

        /// Cap matches[] at the first N rows (post-sort if --sort-by set)
        #[arg(long)]
        limit: Option<usize>,

        /// Skip the first N rows (pagination cursor; envelope gains next_offset when more remain)
        #[arg(long)]
        offset: Option<usize>,

        /// Sort axis: `name` | `arity` | `required_arity` | `category`
        #[arg(long)]
        sort_by: Option<String>,

        /// Comma-separated row projection (e.g. `name,parameter_required_count`)
        #[arg(long, value_delimiter = ',')]
        fields: Vec<String>,

        /// Comma-separated multi-namespace positive scope (e.g. `channel,agent_chat`)
        #[arg(long, value_delimiter = ',')]
        categories: Vec<String>,

        /// Comma-separated multi-namespace negative scope (exclusion wins on overlap)
        #[arg(long, value_delimiter = ',')]
        exclude_categories: Vec<String>,
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

        /// T-2060 / T-2028 Track C: also probe `hub.governor_status` RPC and
        /// render connection-cap + rate-limit + dedupe counters inline.
        /// No-op if hub is not running.
        #[arg(long)]
        governor: bool,
    },
    /// Export the live hub HMAC secret from <runtime_dir>/hub.secret
    ///
    /// G-011 R3 facet 2: always reads the live file, never the IP-keyed
    /// cache at ~/.termlink/secrets/<host>.hex (which can be stale after
    /// a hub regenerates). Use when handing the secret to a peer:
    ///   termlink hub export-secret | ssh peer 'cat > /path && chmod 600 $_'
    ExportSecret {
        /// Write hex to <path> with chmod 600 (atomic). If omitted, prints to stdout.
        #[arg(long)]
        out: Option<String>,

        /// Output as JSON: {"path":"<live-path>","hex":"<value>","bytes":<len>}
        #[arg(long)]
        json: bool,
    },
    /// Print the sha256 fingerprint of <runtime_dir>/hub.cert.pem (TOFU pin)
    ///
    /// Use to verify what peers should pin after a hub rotation (PL-021)
    /// or first-connection. Output matches the `sha256:<hex>` form stored
    /// by `KnownHubStore` (~/.termlink/known_hubs) so values are directly
    /// comparable across hosts.
    Fingerprint {
        /// Output as JSON: {"path":"<live-cert>","fingerprint":"sha256:..."}
        #[arg(long)]
        json: bool,
    },
    /// Probe a remote hub via TLS handshake and print its leaf cert sha256
    ///
    /// G-011 rotation-protocol companion to `hub fingerprint` (which reads
    /// the local cert). `probe` does NOT require auth, a profile, or shell
    /// access to the remote host — it opens TCP, completes a TOFU-style
    /// handshake accepting any cert, and prints the fingerprint in the
    /// canonical `sha256:<hex>` form. Output is directly comparable to
    /// `KnownHubStore.get(addr)` values and to remote `hub fingerprint`
    /// output. Does NOT mutate `~/.termlink/known_hubs`.
    ///
    /// Use when:
    ///   - verifying a hub is up and presenting a cert (pre-pin diagnostic)
    ///   - comparing-without-trust after a suspected rotation
    ///   - operator first-contact before adding a profile
    Probe {
        /// host:port of the hub to probe (e.g. 192.168.10.122:9100)
        addr: String,
        /// Output as JSON: {"address":"<addr>","fingerprint":"sha256:..."}
        #[arg(long)]
        json: bool,
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
    /// Verify a hub's wire fingerprint against the stored TOFU pin
    ///
    /// T-1659 — pure read-only diagnostic: probes <host> via TLS handshake
    /// (no auth, no profile, no `KnownHubStore` mutation), then compares
    /// the captured fingerprint against the entry in `~/.termlink/known_hubs`.
    /// Exit codes (script-friendly):
    ///   0 — match (pin still valid)
    ///   1 — drift (wire != pin; rotation occurred)
    ///   2 — no pin (host not in KnownHubStore)
    ///   3 — probe failed (unreachable / TLS error)
    Verify {
        /// host:port to verify (e.g. 192.168.10.122:9100)
        host: String,
        /// Output as JSON: {"address":..., "wire":..., "pinned":..., "match":bool, "status":...}
        /// JSON mode always exits 0 so callers can parse; `status` carries the verdict.
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

        /// Auto-create the topic via idempotent `channel.create` before
        /// posting (G-050 mitigation, T-1443). Opt-in: typo'd topic names
        /// still surface as -32013 unknown topic for default callers.
        /// Use this for known-canon topics that may have been lost in a
        /// hub restart (chat-arc, broadcast streams, scratchpads). Failure
        /// of channel.create is non-fatal; the post proceeds and the
        /// underlying -32013 surfaces if the topic genuinely doesn't exist.
        #[arg(long = "ensure-topic")]
        ensure_topic: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Opaque idempotency token (T-2049 Gap A). When present, the hub
        /// uses `(sender_id, client_msg_id)` to dedupe retries within a
        /// short TTL (default 5 min) — a re-post of the same id silently
        /// returns the original offset without re-appending. When omitted,
        /// the CLI mints a fresh random 128-bit id. Set explicitly to
        /// override (e.g. content-hash idempotency for scripts).
        #[arg(long = "client-msg-id", value_name = "ID")]
        client_msg_id: Option<String>,
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
    /// Star (bookmark) an envelope on a topic (T-1354). Per-user analogue of
    /// `channel pin` — Matrix `m.bookmark` flavour. Emits a `msg_type=star`
    /// envelope carrying `metadata.star_target=<offset>` and
    /// `metadata.star=true`. Latest action per (sender_id, target) wins, so
    /// repeating `star` is idempotent. See `channel starred` for aggregation.
    Star {
        /// Topic name
        topic: String,

        /// Offset of the envelope to star
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Remove a star from an envelope on a topic (T-1354). Same shape as
    /// `channel star` but emits `metadata.star=false`. Latest action per
    /// (sender_id, target) wins.
    Unstar {
        /// Topic name
        topic: String,

        /// Offset of the envelope to unstar
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// List currently-starred envelopes on a topic (T-1354). Walks the topic,
    /// applies star/unstar events in offset order (latest action per
    /// (sender_id, target) wins), and renders one row per actively-starred
    /// target. By default scoped to the calling user; pass `--all` to include
    /// every user's stars. Sorted by most-recently starred descending.
    /// Read-only.
    Starred {
        /// Topic name
        topic: String,

        /// Include every user's stars instead of just the caller's.
        #[arg(long)]
        all: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{target, starred_by, starred_ts, payload}]`
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

        /// Retro-membership query (T-1380): only count envelopes with
        /// ts <= this cutoff. Useful for "who was active here as of last
        /// Tuesday?". Parallel to `snapshot --as-of`.
        #[arg(long = "as-of")]
        as_of: Option<i64>,

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

        /// Read only the latest envelope on the topic (broadcast-with-replay,
        /// T-2027 / T-2047). Requires exactly one of `--once` or `--then-live`.
        /// On an empty topic, prints "topic is empty" and exits 0 — never
        /// blocks. Mutually exclusive with `--cursor`, `--limit`, `--since`,
        /// `--until`, `--resume`, `--reset`, `--follow`, `--tail`.
        #[arg(
            long,
            conflicts_with_all = ["resume", "reset", "follow", "tail", "since", "until", "conversation_id", "in_reply_to"]
        )]
        from_latest: bool,

        /// With `--from-latest`: fetch the latest envelope and exit.
        /// Mutually exclusive with `--then-live`. Ignored without `--from-latest`.
        #[arg(long, conflicts_with = "then_live")]
        once: bool,

        /// With `--from-latest`: fetch the latest envelope, then stream forward.
        /// Mutually exclusive with `--once`. Ignored without `--from-latest`.
        #[arg(long, conflicts_with = "once")]
        then_live: bool,

        /// Drop envelopes whose `ts > <ms>` from the printed output (T-1352).
        /// Closing pair to `--since`. Combine for an arbitrary
        /// `[since, until]` window. Same render-side semantics — pagination
        /// unchanged. ts-less envelopes are kept (defensive; same precedent
        /// as `--since`).
        #[arg(long, value_name = "MS")]
        until: Option<i64>,

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
    /// Quotable text snippet for citing a channel message in tasks/docs
    /// (T-1363). Walks the topic, finds the target offset, renders it with
    /// N envelopes of context above and below. Skips meta envelopes so the
    /// snippet stays content-focused.
    Snippet {
        /// Topic name
        topic: String,

        /// Offset of the target envelope
        offset: u64,

        /// Number of context envelopes on each side (default: 2).
        #[arg(long, value_name = "N", default_value_t = 2)]
        lines: u64,

        /// Include a topic + offset citation header above the block.
        #[arg(long)]
        header: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `{topic, target_offset, lines:[{offset,sender,payload}]}`
        #[arg(long)]
        json: bool,
    },
    /// Reverse-view of reactions: list everything a specific sender has
    /// reacted to on a topic (T-1362). Distinct from `subscribe --reactions`
    /// (per-message aggregation) and `emoji-stats` (per-emoji breakdown).
    /// Renders one row per active reaction with its parent payload preview.
    ReactionsOf {
        /// Topic name
        topic: String,

        /// Sender id (fingerprint) to scope to. Default: caller identity.
        #[arg(long)]
        sender: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{reaction_offset, parent_offset, emoji, parent_payload, ts}]`
        #[arg(long)]
        json: bool,
    },
    /// Read-receipt dashboard for a topic (T-1361). Composes
    /// `channel.receipts` with the topic's latest offset and member set into
    /// per-sender lag rows: `<sender_id> ack=<up_to> latest=<L> lag=<N>`.
    /// Surfaces members who have posted content but never sent a receipt.
    /// Distinct from `channel receipts` (raw list, no lag) and `channel
    /// unread <topic>` (single-sender count).
    AckStatus {
        /// Topic name
        topic: String,

        /// Show only members whose lag is > 0.
        #[arg(long)]
        pending_only: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{sender_id, up_to, latest, lag, ts}]`
        #[arg(long)]
        json: bool,
    },
    /// Per-topic emoji-reaction breakdown (T-1359). Walks the topic, tallies
    /// every active (non-redacted) `msg_type=reaction` envelope by payload,
    /// and renders rows sorted by count descending. Distinct from
    /// `channel digest` (shows top 3 only) and from `subscribe --reactions`
    /// (per-message aggregation, no global view).
    EmojiStats {
        /// Topic name
        topic: String,

        /// Add per-reactor breakdown beneath each emoji row
        #[arg(long)]
        by_sender: bool,

        /// Truncate output to the top N emoji
        #[arg(long, value_name = "N")]
        top: Option<usize>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{emoji, count, reactors:[{sender_id,count}]}]`
        #[arg(long)]
        json: bool,
    },
    /// Cross-topic inbox: "what did I miss?" view (T-1358). Walks the local
    /// per-(topic, identity) cursor store written by `subscribe --resume`
    /// (T-1318), queries `channel.list` for each topic's current count,
    /// and renders rows for topics where `count - 1 > cursor`. Read-only;
    /// does not touch cursors. Distinct from `channel unread <topic>` which
    /// is single-topic + receipt-based.
    Inbox {
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{topic, unread, latest, cursor}]`
        #[arg(long)]
        json: bool,
    },
    /// Full per-topic statistics dashboard (T-1368). Walks the topic and
    /// reports total envelopes, distinct senders, msg-type breakdown, top-5
    /// senders, distinct + top-5 emojis, thread roots, active pins, forwards-in,
    /// edits, redactions, and lifetime time span. Like `channel digest` but
    /// unconstrained by time and focused on cumulative totals.
    TopicStats {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Unified per-offset navigation (T-1381). Matrix Client API
    /// `/relations/{eventId}` analogue. For one target offset, returns all
    /// four canonical Matrix relations: replies (m.in_reply_to), reactions
    /// (m.annotation), edits (m.replace), redactions (m.redaction). Each
    /// list sorted ts_ms asc. Forwards excluded — cross-topic relation.
    Relations {
        /// Topic name
        topic: String,

        /// Target offset whose relations to surface
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON (full lists, not truncated to first 5)
        #[arg(long)]
        json: bool,
    },
    /// Per-target reply rollup (T-1379). For each target message that has
    /// been replied to at least once, lists reply count, distinct repliers,
    /// and latest reply timestamp. Per-target companion to `replies-of`
    /// (T-1370, per-sender). Reactions don't count as replies even if they
    /// carry `in_reply_to`. Sort: reply_count desc, target_offset asc.
    QuoteStats {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Point-in-time canonical view of a topic (T-1378). Matrix backfill
    /// semantics: simulate the room as it was at `--as-of <ms>`. Combines
    /// T-1376 collapse (apply edits, hide redactions) with a temporal upper
    /// bound — events with ts > as_of are NOT applied. Useful for forensic
    /// replay ("what did this say last Tuesday?"). Distinct from `state`
    /// (current truth) and `subscribe --until` (raw envelope filter).
    Snapshot {
        /// Topic name
        topic: String,

        /// Cutoff timestamp in ms (envelopes with ts > as_of are ignored)
        #[arg(long = "as-of")]
        as_of: i64,

        /// Show redacted rows with payload "[REDACTED]" instead of dropping.
        #[arg(long)]
        include_redacted: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Chronological receipt audit log (T-1377). Walks the topic and lists
    /// every `msg_type=receipt` envelope as a row in ts asc order. Distinct
    /// from `receipts` (T-1315 LWW snapshot) and `ack-status` (T-1361
    /// dashboard with lag). Optional positional user filter narrows the log
    /// to a single sender. Extends the audit-log family (pin-history,
    /// redactions, edit-stats) to receipt activity.
    AckHistory {
        /// Topic name
        topic: String,

        /// Optional user filter — only show receipts from this sender_id
        user: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Canonical collapsed view of a topic (T-1376). Matrix-style room
    /// render: applies `m.replace` (edits — latest text wins) and hides
    /// `m.redaction`-targeted offsets. One row per visible content message,
    /// in offset-asc order. Distinct from raw `subscribe` (envelope stream),
    /// `info` (topic summary), `edits-of` (single-target history), and
    /// `edit-stats` (count rollup). This is "what does this topic say now?"
    State {
        /// Topic name
        topic: String,

        /// Show redacted rows with payload "[REDACTED]" instead of dropping.
        #[arg(long)]
        include_redacted: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Diff between two point-in-time snapshots (T-1383). Composes two
    /// T-1378 `compute_snapshot` calls and classifies the union of
    /// offsets as added / removed / edited / unchanged. Default text
    /// rendering omits `unchanged` rows; pass `--include-unchanged` to
    /// see all four classes. Useful for forensic replay and audit.
    SnapshotDiff {
        /// Topic name
        topic: String,

        /// Lower-bound timestamp (ms since epoch). State as it was at this ts.
        #[arg(long = "from")]
        from_ms: i64,

        /// Upper-bound timestamp (ms since epoch). State as it was at this ts.
        #[arg(long = "to")]
        to_ms: i64,

        /// Include redacted rows in both snapshots (with payload "[REDACTED]").
        #[arg(long)]
        include_redacted: bool,

        /// Show unchanged rows alongside diff entries (default: hide them).
        #[arg(long)]
        include_unchanged: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Incremental state view (T-1382). Matrix `/sync` analogue: returns
    /// only rows whose canonical state changed at or after `--since`
    /// (new posts, new edits, new redactions). Composes T-1376 state with
    /// a per-row last-change filter. `--since 0` is equivalent to
    /// `channel state` (full state).
    StateSince {
        /// Topic name
        topic: String,

        /// Lower-bound timestamp (ms since epoch). Rows with last change
        /// at or after this ts are returned.
        #[arg(long = "since")]
        since_ms: i64,

        /// Show redacted rows with payload "[REDACTED]" instead of dropping.
        #[arg(long)]
        include_redacted: bool,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Per-target edit count summary for a topic (T-1375). Topic-wide
    /// aggregate companion to `edits-of` (T-1366, single-target full
    /// history). Lists each target offset with edit count, last editor,
    /// latest edit timestamp, and target payload preview. Completes the
    /// audit trio with `pin-history` (T-1372) and `redactions` (T-1373).
    EditStats {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Per-message reaction rollup (T-1374). Matrix annotation API analog —
    /// for a single target offset, group every active reaction by emoji and
    /// list the unique senders. Distinct from `emoji-stats` (topic-wide) and
    /// `reactions-of` (per-sender). Sort: count desc, emoji asc tiebreak.
    /// Honors redaction.
    ReactionsOn {
        /// Topic name
        topic: String,

        /// Target offset whose reactions to roll up
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Chronological redaction audit log for a topic (T-1373). Walks the
    /// topic and lists every `msg_type=redaction` envelope as a row, with
    /// target offset, redactor, optional reason, ts, and a preview of the
    /// original payload when still in the snapshot. Symmetric to
    /// `pin-history` (T-1372). Useful for "what got pulled and why".
    Redactions {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Pin/unpin audit log for a topic (T-1372). Walks the topic and lists
    /// every `msg_type=pin` envelope chronologically — each toggle is a row,
    /// including pins that were later unpinned and re-pins of the same target.
    /// Complements `channel pinned` (T-1345), which shows live last-write-wins
    /// state only. Useful for forensic audits and "when did this happen".
    PinHistory {
        /// Topic name
        topic: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Mentions of a user on a topic (T-1371). Reverse view of `--mention`
    /// posting. Lists every envelope on `<topic>` whose `metadata.mentions`
    /// CSV matches `<user>` (wildcard `*` in either side honored per T-1333).
    /// Renders mention_offset, sender, payload preview, mentions csv, ts.
    /// Sort: mention_offset desc. Honors redaction. Skips meta envelopes
    /// (receipt/typing/edit/redaction/pin/topic_metadata).
    MentionsOf {
        /// Topic name
        topic: String,

        /// User id to scan mentions for (e.g. fingerprint, room nickname).
        /// Pass `*` to find every post that tagged anyone.
        user: String,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Replies posted by a sender on a topic (T-1370). Reverse view of
    /// `channel reply` (T-1313). Sender defaults to the caller fingerprint.
    /// Renders reply_offset, parent (offset + sender + payload preview), reply
    /// payload preview, and ts. Sort: reply_offset desc. Honors redaction.
    /// Excludes reactions (msg_type=reaction also carries in_reply_to but is
    /// not a reply post).
    RepliesOf {
        /// Topic to scan
        topic: String,

        /// Sender fingerprint (defaults to caller identity)
        sender: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Forwards posted by a sender on a topic (T-1367). Reverse view of
    /// `channel forward` (T-1346). Sender defaults to the caller fingerprint.
    /// Renders forward_offset, origin (topic + offset), original sender,
    /// payload preview, and ts. Sort: forward_offset desc. Honors redaction.
    ForwardsOf {
        /// Topic to scan
        topic: String,

        /// Sender fingerprint (defaults to caller identity)
        sender: Option<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Edit history for a target offset (T-1366). Renders the original post
    /// at `<offset>` followed by every `msg_type=edit` envelope whose
    /// `metadata.replaces` equals the target, in chronological order. Matrix
    /// m.replace history analog. Skips redacted edits.
    EditsOf {
        /// Topic name
        topic: String,

        /// Target offset whose edit history to render
        offset: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `{original: {...}, edits: [{...}, ...]}`
        #[arg(long)]
        json: bool,
    },
    /// Index of all threads in a topic (T-1365). Walks the topic and lists
    /// every offset that has at least one reply (a thread root) with reply
    /// count, distinct participants, last activity, and a payload preview.
    /// Sorted by last_ts_ms desc. Honors redaction (redacted root → row dropped;
    /// redacted replies don't count). Matrix m.thread room-overview analog.
    Threads {
        /// Topic name
        topic: String,

        /// Limit to top N rows (after sort by last_ts_ms desc)
        #[arg(long, value_name = "N")]
        top: Option<usize>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `[{root_offset, reply_count, participants, last_ts_ms, root_payload}]`
        #[arg(long)]
        json: bool,
    },
    /// Synthesized recent-activity digest for a topic (T-1356). Walks the
    /// topic, applies a time filter, and renders a compact summary
    /// (posts, top senders, top reactions, pins added/removed, forwards in,
    /// last 3 chat snippets). Use `--since-mins N` for a relative window or
    /// `--since MS` for an absolute lower bound. Read-only.
    Digest {
        /// Topic name
        topic: String,

        /// Window size in minutes — include envelopes with
        /// `ts_unix_ms >= now - N * 60_000`. Mutually exclusive with `--since`.
        #[arg(long, value_name = "N", conflicts_with = "since")]
        since_mins: Option<i64>,

        /// Absolute lower-bound `ts_unix_ms` (epoch milliseconds) — include
        /// envelopes with `ts_unix_ms >= MS`. Mutually exclusive with
        /// `--since-mins`.
        #[arg(long, value_name = "MS")]
        since: Option<i64>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Polls on a topic (T-1355). Matrix `m.poll.start` / `m.poll.response`
    /// / `m.poll.end` analog. Sub-actions: `start`, `vote`, `end`, `results`.
    Poll {
        #[command(subcommand)]
        action: PollAction,
    },
    /// T-2032 (arc-parallel-substrate) — exclusively claim an offset on a
    /// topic so only this worker processes it. Wraps the `channel.claim`
    /// JSON-RPC verb shipped in T-2029. The claim is held for `ttl_ms`
    /// (default 30s, hub-clamped to 1h max) and must be either renewed
    /// (`channel renew`), released (`channel release`), or it lapses for the
    /// next claimant's lazy-evict pass. Returns `claim_id` — keep it; you
    /// need it for renew/release.
    Claim {
        /// Topic name (must already exist — `channel create` first if not).
        topic: String,
        /// Offset within the topic to exclusively claim.
        offset: u64,
        /// Worker identifier — your stable identity so the hub can gate
        /// renew/release on ownership. Free-form string; typically your
        /// fingerprint or an agent_id.
        #[arg(long)]
        claimer: String,
        /// Lease TTL in milliseconds (default 30_000 = 30s; hub clamps to
        /// 1h max). Pick this for "how long can this worker plausibly take
        /// to either ack the work, nack it, or renew the lease?".
        #[arg(long, default_value_t = 30_000)]
        ttl_ms: u32,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// T-2032 (arc-parallel-substrate) — extend a held claim's lease. Wraps
    /// the `channel.renew` JSON-RPC verb shipped in T-2030. Refuses if the
    /// caller is not the original claimer or if the lease has already
    /// lapsed (lazy-evicted by the hub on next access).
    Renew {
        /// Opaque claim_id returned by `channel claim`.
        #[arg(long = "claim-id")]
        claim_id: String,
        /// Same `claimer` value you used in `channel claim` — gates ownership.
        #[arg(long)]
        claimer: String,
        /// Additional lease milliseconds (default 30_000 = 30s; hub clamps
        /// to 1h max). The new `claimed_until` is `now + additional_ttl_ms`.
        #[arg(long, default_value_t = 30_000)]
        additional_ttl_ms: u32,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// T-2032 (arc-parallel-substrate) — consume a held claim. Wraps the
    /// `channel.release` JSON-RPC verb shipped in T-2029. With `--ack`,
    /// advances the claimer's persisted cursor past the offset (work
    /// completed). Without, the slot reopens for the next worker without
    /// cursor advance (work returned for retry).
    Release {
        /// Opaque claim_id returned by `channel claim`.
        #[arg(long = "claim-id")]
        claim_id: String,
        /// Same `claimer` value you used in `channel claim` — gates ownership.
        #[arg(long)]
        claimer: String,
        /// Acknowledge the work as completed — advances cursor past the
        /// offset. Without this flag, slot reopens without cursor advance.
        #[arg(long)]
        ack: bool,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// T-2046 (T-2021 GO, arc-parallel-substrate primitive #3) — atomic
    /// ownership transfer of an existing claim. Cooperative + owner-checked:
    /// `--by` MUST equal the current claimed_by (`CLAIM_NOT_OWNED` otherwise).
    /// Distinct from `claim-force-release` which is the operator-Tier-0
    /// ownership-bypass verb. Lease timestamps (`claimed_at`, `claimed_until`)
    /// survive the transfer — only `claimed_by` mutates.
    ///
    /// Use this when an orchestrator has claimed a slot on a worker's behalf
    /// and now needs to atomically hand the lease to that worker without
    /// the release-then-claim race window.
    ClaimTransfer {
        /// Opaque claim_id returned by the original `channel claim` (look it
        /// up via `channel claims <topic>` if needed).
        #[arg(long = "claim-id")]
        claim_id: String,
        /// New owner the lease transfers TO (e.g. `worker-A`).
        #[arg(long = "to-owner")]
        to_owner: String,
        /// Current owner of the claim. MUST equal the row's `claimed_by` —
        /// cooperative gate (`CLAIM_NOT_OWNED` otherwise).
        #[arg(long)]
        by: String,
        /// Optional audit reason — echoed in the response, surfaced but not
        /// persisted in the claims table.
        #[arg(long)]
        reason: Option<String>,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// T-2044 (arc-parallel-substrate Slice 11) — operator-Tier-0 force
    /// release of a held claim. Wraps the `channel.force_release` JSON-RPC
    /// verb shipped in T-2044. Bypasses the `claimed_by == claimer`
    /// ownership check that `channel release` enforces — for when an
    /// operator must clear a stuck claim faster than the natural TTL
    /// expiry path. Semantics match `release --ack=false` (cursor
    /// unchanged, slot freed for the next worker, work returns for retry).
    /// Pairs with `channel claims-summary --watch` for stuck-worker
    /// detection: detection (Slice 8) → diagnosis (Slice 9) → intervention
    /// (Slice 11).
    ClaimForceRelease {
        /// Opaque claim_id of the stuck claim (look it up via
        /// `channel claims <topic>`).
        #[arg(long = "claim-id")]
        claim_id: String,
        /// Operator-supplied audit reason (echoed in the response and
        /// useful for downstream audit-log forwarding). Optional.
        #[arg(long)]
        reason: Option<String>,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// T-2037 (arc-parallel-substrate Slice 4) — list current claim rows for
    /// a topic. Read-only introspection — answers "what is currently
    /// claimed?" without forcing the operator to attempt a `channel claim`.
    /// Wraps the `channel.claims` JSON-RPC verb. Default surfaces only live
    /// leases; `--include-expired` adds rows past their `claimed_until` for
    /// forensics (e.g. "who held this stuck offset before it expired?").
    Claims {
        /// Topic name to list claims for. The topic must exist
        /// (`channel create` first if not — same contract as `channel claim`).
        topic: String,
        /// Include rows whose `claimed_until` is in the past. Default
        /// (false) surfaces only live leases.
        #[arg(long)]
        include_expired: bool,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// T-2039 (arc-parallel-substrate Slice 6) — show aggregate claim state
    /// for a topic. Observability companion to `channel claims`: answers
    /// "how busy is this topic, and is anything stuck?" in a single line
    /// (active vs expired count, oldest-active age, next free slot).
    ///
    /// Wraps the `channel.claims_summary` JSON-RPC verb. One round-trip,
    /// O(1) at the hub (single SQL aggregate over `idx_claims_topic_until`)
    /// — safe to call on hot paths or from cron.
    ClaimsSummary {
        /// Topic name to summarize claims for. The topic must exist
        /// (`channel create` first if not — same contract as `channel claim`).
        /// Mutually exclusive with `--all` (exactly one is required).
        topic: Option<String>,
        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,
        /// Output as JSON. Incompatible with `--watch` (streaming text vs
        /// one-shot envelope — pick one).
        #[arg(long)]
        json: bool,
        /// T-2041 (arc-parallel-substrate Slice 8): continuous-monitor mode.
        /// Re-runs the summary every N seconds (clamped to 5..=3600) until
        /// SIGINT, clearing the screen between frames. Designed as the
        /// hands-off form of the cron stuck-worker recipe — leave it
        /// running on a side terminal during incident triage. Per-tick
        /// fetch errors are non-fatal (printed, loop continues).
        ///
        /// Composes with `--all`: `--all --watch 30` gives a live
        /// fleet-wide stuck-worker dashboard.
        #[arg(long)]
        watch: Option<u64>,
        /// T-2042 (arc-parallel-substrate Slice 9): fleet-wide sweep mode.
        /// Instead of summarizing one topic, query `channel.list` and
        /// per-topic call `channel.claims_summary`. Renders one line per
        /// topic and annotates `[POTENTIALLY STUCK]` when
        /// `expired_count > 0` OR `oldest_active_age_ms > 60_000`. Footer
        /// shows total topics + stuck count. Per-topic fetch errors are
        /// non-fatal (printed inline). Mutually exclusive with the `topic`
        /// positional (exactly one is required).
        #[arg(long)]
        all: bool,
    },
}

/// Poll sub-actions (T-1355).
#[derive(Subcommand)]
pub(crate) enum PollAction {
    /// Open a poll on a topic. Posts a `msg_type=poll_start` envelope whose
    /// payload is the question and `metadata.poll_options=opt1|opt2|opt3`.
    /// The envelope's offset becomes the poll id used by `vote`/`end`/`results`.
    Start {
        /// Topic name
        topic: String,

        /// The poll question (rendered as the envelope payload)
        #[arg(long)]
        question: String,

        /// Option label. Repeat for each option (>=2 required).
        #[arg(long = "option", required = true, num_args = 1..)]
        options: Vec<String>,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Cast a vote in an open poll. Posts `msg_type=poll_vote` with
    /// `metadata.poll_id=<poll_id>` and `metadata.poll_choice=<index>`.
    /// Re-voting replaces the prior vote (latest action per sender wins).
    Vote {
        /// Topic name
        topic: String,

        /// Poll id (the offset of the poll_start envelope)
        poll_id: u64,

        /// Zero-based option index
        #[arg(long)]
        choice: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Close a poll. Posts `msg_type=poll_end` with
    /// `metadata.poll_id=<poll_id>`. Aggregator drops votes whose ts is
    /// after the end envelope.
    End {
        /// Topic name
        topic: String,

        /// Poll id (the offset of the poll_start envelope)
        poll_id: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },
    /// Render tallies for a poll. Walks the topic, finds the poll_start at
    /// `<poll_id>`, applies poll_vote envelopes (latest per sender wins),
    /// and stops counting after poll_end (if any). Read-only.
    Results {
        /// Topic name
        topic: String,

        /// Poll id (the offset of the poll_start envelope)
        poll_id: u64,

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON `{question, options:[{label,count,voters}], closed}`
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

        /// T-1432: append T-1166 cut-readiness telemetry. For each reachable
        /// hub, query `hub.legacy_usage` and tabulate legacy-primitive
        /// invocations within `--legacy-window-days`.
        ///
        /// T-1459: Verdict has four levels:
        /// - CUT-READY          — every reachable hub reports zero legacy traffic
        /// - CUT-READY-DECAYING — residue exists but no live caller in last 5 min
        /// - WAIT               — at least one hub had a legacy call in last 5 min
        /// - UNCERTAIN          — some hubs unsupported (pre-T-1432) or no audit yet
        #[arg(long)]
        legacy_usage: bool,

        /// T-1432: window in days for `--legacy-usage` (default: 7).
        /// The bake window the operator cares about pre-T-1166 cut.
        #[arg(long, default_value = "7")]
        legacy_window_days: u64,

        /// T-1446: append G-050 audit-sweep telemetry. For each reachable
        /// hub, query `hub.bus_state` and report runtime_dir + bus/meta.db
        /// presence/size/mtime. Fleet verdict is DURABLE iff every hub
        /// reports audit_present=true and runtime_dir_volatile=false (heuristic:
        /// runtime_dir does not start with /tmp/). VOLATILE if any hub on
        /// /tmp/. UNCERTAIN if any hub is unsupported (pre-T-1446) or
        /// audit_present=false. Closes G-050.what_remains audit-sweep ask.
        #[arg(long)]
        topic_durability: bool,

        /// T-1666: additionally probe each hub via TLS handshake and compare
        /// its wire fingerprint against the stored TOFU pin in `~/.termlink/known_hubs`.
        /// Surfaces cert-rotation drift in the same report as auth-mismatch
        /// (which surfaces secret-rotation). The two axes are independent — a
        /// pin-drift does NOT change cut-readiness verdict, and cut-readiness
        /// UNCERTAIN does NOT mask pin-drift. Per-hub JSON gains a `pin_check`
        /// sub-object; fleet rollup JSON gains `pin_check_summary`. Plain mode
        /// adds a `Pin check: <verdict>` footer plus per-hub drift lines.
        /// No-op when false (default) — additive flag, no schema change.
        #[arg(long)]
        include_pin_check: bool,

        /// T-1462: compare cut-readiness against a previously-saved snapshot.
        /// Reads the file at PATH (must be JSON output of an earlier
        /// `fleet doctor --legacy-usage --json` run), computes per-hub +
        /// per-caller + fleet `total_legacy` deltas, and (when both snapshots
        /// have an embedded `_snapshot_ts_ms`) an average rate (calls/min).
        /// Only meaningful when combined with `--legacy-usage`. CLI-only —
        /// no hub upgrade required.
        #[arg(long, value_name = "PATH")]
        diff: Option<std::path::PathBuf>,

        /// T-1463: write the JSON document of this run to PATH (in addition
        /// to whatever stdout/stderr produce). Lets a single invocation both
        /// show the human-readable verdict AND persist a snapshot for later
        /// `--diff` calls. Atomic write (PATH.tmp → rename). Parent directory
        /// must exist.
        #[arg(long, value_name = "PATH")]
        save_snapshot: Option<std::path::PathBuf>,

        /// T-1465: map cut-readiness verdict to a process exit code so
        /// cron / CI can gate on it without parsing JSON. Requires
        /// `--legacy-usage`. Mapping: CUT-READY=0, CUT-READY-DECAYING=0,
        /// WAIT=10, UNCERTAIN=11. Connectivity failures keep their existing
        /// non-zero exit (precedence over verdict mapping).
        #[arg(long)]
        exit_code_on_verdict: bool,

        /// T-1468: read up to N most-recent JSON snapshots from PATH (a
        /// directory of `fleet doctor --legacy-usage --json` outputs, e.g.
        /// the dir written by `scripts/cut-readiness-daily.sh`), and emit
        /// a day-over-day decay table + sparkline. Files are sorted lexically
        /// (matching the `YYYY-MM-DD.json` cron convention = chronological).
        /// Requires `--legacy-usage`.
        #[arg(long, value_name = "PATH")]
        trend: Option<std::path::PathBuf>,

        /// T-1468: cap on snapshots read by `--trend` (default 7, max 30).
        /// Older snapshots are ignored. Default 7 = "show me the last week".
        #[arg(long, default_value = "7")]
        trend_keep: u32,

        /// T-1471: how many top callers to show per hub AND fleet-wide.
        /// Default 3 (preserves existing output shape). Clamped 1..=50 to
        /// prevent runaway output on hubs with many distinct caller IDs.
        #[arg(long, default_value = "3")]
        top_callers: u32,

        /// T-1667: continuous monitoring mode — re-run the diagnostic every N
        /// seconds (5 <= N <= 3600) and emit ONLY per-hub state changes,
        /// prefixed with an RFC3339 timestamp. First cycle emits a full
        /// baseline. State tracked per hub: connectivity status, pin status
        /// (when `--include-pin-check`), and total legacy invocations (when
        /// `--legacy-usage`). SIGINT (Ctrl-C) exits cleanly with code 0.
        /// Cron-replacement for rotation surveillance — leave it running in
        /// a terminal. Internally re-spawns self with `--json` each cycle;
        /// the subprocess boundary keeps transient failures from killing the
        /// watch and avoids a full output-capture refactor.
        ///
        /// Composable with `--include-pin-check` (recommended — catches CERT
        /// rotation), `--legacy-usage` (catches new legacy callers appearing
        /// during the watch), and `--topic-durability`. Incompatible with
        /// `--diff`, `--save-snapshot`, `--exit-code-on-verdict`, and
        /// `--trend` (single-shot semantics — watch already produces a
        /// time-series).
        #[arg(long, value_name = "SECONDS")]
        watch: Option<u64>,

        /// T-1669: invoke this shell command whenever a hub's state changes
        /// during `--watch` (skipped on baseline cycle). Requires `--watch`.
        /// The command is fire-and-forget (not awaited) — a hanging script
        /// will NOT block the watch loop. Non-zero exits are logged to
        /// stderr but the watch continues.
        ///
        /// Per-event environment passed to the command:
        ///   TERMLINK_WATCH_HUB         hub profile name
        ///   TERMLINK_WATCH_CHANGE_KIND "transition" | "new" | "removed"
        ///   TERMLINK_WATCH_OLD_CONN    prior connectivity status (or empty if new)
        ///   TERMLINK_WATCH_NEW_CONN    current connectivity status
        ///   TERMLINK_WATCH_OLD_PIN     prior pin status (or "-")
        ///   TERMLINK_WATCH_NEW_PIN     current pin status (or "-")
        ///   TERMLINK_WATCH_OLD_LEGACY  prior total_legacy count (or "-")
        ///   TERMLINK_WATCH_NEW_LEGACY  current total_legacy count (or "-")
        ///   TERMLINK_WATCH_TS          RFC3339 detection time (T-1676)
        ///
        /// Operator usage: write a shell script that responds to the event
        /// (notify Slack, page on-call, run `fleet reauth $TERMLINK_WATCH_HUB
        /// --bootstrap-from auto` for declared profiles, etc.). Termlink ships
        /// detection + change events; operator ships response policy.
        #[arg(long, value_name = "CMD")]
        notify: Option<String>,

        /// T-1680/T-1683: built-in auto-heal on rotation.
        ///
        /// Two modes:
        ///   * With `--watch`: continuous-monitor mode. On every per-hub change
        ///     event where `new_pin == "drift"` (cert rotation, T-1680) OR
        ///     `new_conn == "auth-mismatch"` (secret-only rotation, T-1681)
        ///     AND the profile declares `bootstrap_from` in hubs.toml,
        ///     spawn the equivalent of `termlink fleet reauth $hub
        ///     --bootstrap-from auto` (fire-and-forget).
        ///   * Without `--watch` (T-1683): single-shot mode. After the
        ///     fleet sweep, classify each hub's current state and fire the
        ///     same heal for any profile that's CURRENTLY in drift or
        ///     auth-mismatch AND has declared `bootstrap_from`. Same R2 gate.
        ///     Useful for page-respond ("doctor says drift, fix it now")
        ///     without starting a watch loop.
        ///
        /// Profiles without declared `bootstrap_from` are skipped with a
        /// one-line stderr hint — R2 (out-of-band anchor) forbids implicit
        /// defaults.
        ///
        /// One-flag continuous-monitor recipe:
        ///   termlink fleet doctor --watch 30 --include-pin-check --auto-heal
        /// One-shot recipe:
        ///   termlink fleet doctor --include-pin-check --auto-heal
        #[arg(long = "auto-heal")]
        auto_heal: bool,

        /// T-1684: preview which heals `--auto-heal` would fire without
        /// actually spawning the heal subprocesses. Useful when wiring
        /// automation, debugging the bootstrap_from gate, or just verifying
        /// the dedup behavior. Output mirrors live mode but each fire site
        /// emits `[DRY-RUN] would fire: termlink fleet reauth ... --bootstrap-from auto`
        /// to stderr instead of spawning a process. Works with both single-shot
        /// and `--watch` modes. Requires `--auto-heal` (alone it has nothing
        /// to dry-run).
        #[arg(long = "dry-run", requires = "auto_heal")]
        dry_run: bool,
    },

    /// T-2062 / T-2028 Track D: fleet-wide aggregation of `hub.governor_status`.
    ///
    /// For every hub in `~/.termlink/hubs.toml`, probes the substrate
    /// connection-cap + per-sender rate-limit + post-dedupe counters that
    /// T-2048 Track B exposed via RPC. Renders a per-hub block plus
    /// fleet-wide rollup (total connections active, total capacity_hits,
    /// total rate_hits, total dedupe_hits, hubs hitting capacity, hubs
    /// hitting rate limits). Read-only (Observe scope), no mutation.
    ///
    /// Pairs with T-2060's `hub status --governor` (single-hub) and
    /// T-2048's `termlink_hub_governor_status` MCP verb.
    GovernorStatus {
        /// Output result as JSON for scripting / dashboards.
        /// Incompatible with `--watch` (streaming text only).
        #[arg(long, conflicts_with = "watch")]
        json: bool,

        /// RPC timeout per hub in seconds (default: 8). Each hub is bounded
        /// independently so a wedged hub cannot hang the fleet view.
        #[arg(long, default_value = "8")]
        timeout: u64,

        /// T-2064 (T-2028 §6 #10 Track E): re-poll the fleet every N seconds
        /// and emit per-hub state changes (capacity_hits / rate_hits /
        /// dedupe_hits deltas, reachable transitions). N clamped to [5, 3600].
        /// Cycle 1 prints a baseline; subsequent cycles print only changed
        /// hubs plus a silent-cycle marker. SIGINT exits cleanly.
        ///
        /// Pattern parity with `fleet doctor --watch` (T-1667). Designed for
        /// "leave running in a terminal" surveillance — answers
        /// "is the substrate being refused right now?" without re-running a
        /// one-shot every minute.
        #[arg(long, value_name = "SECONDS")]
        watch: Option<u64>,

        /// T-2065 (T-2028 §6 #10 Track F): invoke this shell command whenever
        /// a hub's governor state changes during `--watch` (skipped on the
        /// baseline cycle, since there is no prior state to diff). Requires
        /// `--watch`. The command is fire-and-forget (not awaited) — a hanging
        /// script will NOT block the watch loop. Spawn failures log to stderr
        /// but the watch continues.
        ///
        /// Per-event environment passed to the command:
        ///   TERMLINK_GOV_HUB              hub profile name
        ///   TERMLINK_GOV_CHANGE_KIND      "transition" | "new" | "removed"
        ///   TERMLINK_GOV_TS               RFC3339 detection time
        ///   TERMLINK_GOV_OLD_REACH        prior reachability ("ok"|"fail"|"")
        ///   TERMLINK_GOV_NEW_REACH        current reachability
        ///   TERMLINK_GOV_OLD_CONN_ACTIVE  prior connections_active
        ///   TERMLINK_GOV_NEW_CONN_ACTIVE  current connections_active
        ///   TERMLINK_GOV_OLD_CAP_HITS     prior capacity_hits_total
        ///   TERMLINK_GOV_NEW_CAP_HITS     current capacity_hits_total
        ///   TERMLINK_GOV_CAP_HITS_DELTA   max(0, new - old)
        ///   TERMLINK_GOV_OLD_RATE_HITS    prior rate_hits_total
        ///   TERMLINK_GOV_NEW_RATE_HITS    current rate_hits_total
        ///   TERMLINK_GOV_RATE_HITS_DELTA  max(0, new - old)
        ///   TERMLINK_GOV_OLD_DEDUPE_HITS  prior dedupe_hits_total ("" if n/a)
        ///   TERMLINK_GOV_NEW_DEDUPE_HITS  current dedupe_hits_total ("" if n/a)
        ///   TERMLINK_GOV_DEDUPE_HITS_DELTA max(0, new - old) ("" if either n/a)
        ///
        /// Operator usage: write a shell script that responds to the event
        /// (Slack post, PagerDuty incident, scale-out trigger, runaway-poller
        /// containment). Common gate pattern:
        ///   [ "$TERMLINK_GOV_CAP_HITS_DELTA" -gt 0 ] || exit 0
        #[arg(long, value_name = "CMD", requires = "watch")]
        notify: Option<String>,
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
        /// (omit when using --all-drifted).
        profile: Option<String>,

        /// Out-of-band trust anchor that delivers the new secret.
        /// When omitted, this command prints the heal incantation instead of
        /// performing the heal.
        #[arg(long = "bootstrap-from")]
        bootstrap_from: Option<String>,

        /// T-1679: bulk-heal every profile that is drifted AND has declared
        /// `bootstrap_from` in hubs.toml. Mutex with positional <profile>.
        /// Each profile uses its own declared trust anchor; failures on one
        /// profile do not abort the loop. Profiles drifted without declared
        /// bootstrap_from are skipped with a hint pointing at Tier-1.
        #[arg(long = "all-drifted", conflicts_with = "profile")]
        all_drifted: bool,

        /// T-1728: emit machine-readable JSON instead of the human-text plan
        /// or eprintln summary. Same outcome shape that
        /// `termlink_fleet_reauth` MCP returns: `{ok, profile, mode, source,
        /// secret_file, fingerprint_preview, plan_text, error}`. Currently
        /// honored on single-profile heals; bulk `--all-drifted` continues
        /// to render the operator table (out of scope for T-1728).
        #[arg(long)]
        json: bool,
    },

    /// T-1660: probe every hub in ~/.termlink/hubs.toml and compare wire fingerprint vs pin
    ///
    /// Fleet-wide companion to `tofu verify <addr>`. Pure read-only diagnostic:
    /// no auth, no profile mutation, no `KnownHubStore` writes. Cron-friendly.
    ///
    /// Exit codes (fleet rollup, drift dominates):
    ///   0 — every reachable hub is `match`
    ///   1 — any hub is `drift` (rotation happened — heal required)
    ///   2 — any hub is `no-pin` (and no drift/probe-fail)
    ///   3 — any hub is `probe-fail` (and no drift)
    ///
    /// `--exit-on-drift-only` collapses 2/3 to exit 0 so cron only alerts on
    /// actual rotation, ignoring transient connectivity and unpinned hosts.
    Verify {
        /// Output as JSON: {verdict, profiles: [{name, address, status, wire, pinned, error}]}
        #[arg(long)]
        json: bool,

        /// Only exit non-zero on drift; treat no-pin / probe-fail as exit 0.
        /// Useful for cron pages — alert only when a hub has actually rotated.
        #[arg(long)]
        exit_on_drift_only: bool,
    },

    /// T-1671: print the rotation history captured by `fleet doctor --watch`.
    ///
    /// Reads `~/.termlink/rotation.log` (NDJSON, append-only). Each line is
    /// one per-hub state change recorded during an earlier `--watch` session.
    /// Useful for retrospective diagnosis: "did this hub flap before this
    /// week?" / "is /tmp wiping multiple times (PL-021)?" — without needing
    /// to keep a watch terminal open continuously.
    ///
    /// Empty/missing log → prints a hint pointing at `fleet doctor --watch`
    /// to start capturing.
    History {
        /// Window in days (default 7, clamped 1..=365). Older entries skipped.
        #[arg(long, default_value = "7")]
        since: u32,

        /// Restrict output to one hub profile name.
        #[arg(long, value_name = "NAME")]
        hub: Option<String>,

        /// Emit NDJSON (one matching log entry per line) plus a summary footer.
        #[arg(long)]
        json: bool,

        /// T-1686: also include heal events from `~/.termlink/heal.log`
        /// (T-1685), merged in chronological order with rotation events.
        /// Without this flag, only rotation events are shown — preserving
        /// the original T-1671 surface. Heal entries render with the
        /// HEAL/<mode> kind marker and trigger/action fields.
        #[arg(long = "include-heals")]
        include_heals: bool,

        /// T-1690: scan rotation.log for PL-021 flap signatures
        /// (BOTH cert + secret rotating simultaneously, repeatedly).
        /// Replaces the chronological listing with a per-hub classification:
        /// `pl021-candidate` / `cert-only` / `secret-only` / `clean`.
        /// Emits the volatile-/tmp diagnostic verbatim for any candidate so
        /// the operator has a copy-pasteable next step. Exit code 2 when at
        /// least one PL-021 candidate is detected, 0 otherwise.
        ///
        /// Signature: ≥2 entries within the `--since` window where the same
        /// log row carries both `new_pin=drift` (was-not-drift) AND
        /// `new_conn=auth-mismatch` (was-not-auth-mismatch). Single
        /// double-rotation could be a one-off operator nuke; two or more in
        /// the same window points at recurring volatile runtime_dir.
        #[arg(long)]
        analyze: bool,
    },

    /// T-1688: preflight-validate declared `bootstrap_from` anchors WITHOUT
    /// performing any heal. Operator scenario: "I declared
    /// `bootstrap_from = ssh:host` on profile X. Will it actually work
    /// when `--auto-heal` fires?" Runs `fetch_bootstrap_secret(source)`
    /// + `normalize_and_validate_secret_hex(raw)` and reports per-profile
    /// status. Never writes the secret file.
    ///
    /// Status taxonomy:
    ///   ok             — fetched + valid 64-hex
    ///   no-anchor      — no `bootstrap_from` declared on profile
    ///   fetch-fail     — channel error (ssh failed, file unreadable, etc.)
    ///   invalid-format — fetched but not 64 hex chars (trimmed)
    ///
    /// Exit codes:
    ///   0 — no fetch-fail and no invalid-format
    ///   1 — any fetch-fail or invalid-format
    ///   2 — `--all` and no profile declares `bootstrap_from` at all
    ///
    /// Either `<profile>` (positional) OR `--all` must be present.
    ///
    /// Note: `ssh:` channels invoke `ssh <host> -- sudo cat
    /// /var/lib/termlink/hub.secret` interactively (same as the live heal
    /// path). For CI/automation, prefer `file:` anchors.
    BootstrapCheck {
        /// Hub profile name (mutex with --all).
        profile: Option<String>,

        /// Validate every profile that declares `bootstrap_from`.
        /// Profiles without a declared anchor are listed with status=no-anchor.
        #[arg(long, conflicts_with = "profile")]
        all: bool,

        /// Output as JSON: {verdict, profiles: [{name, address, bootstrap_from, status, error?}]}
        #[arg(long)]
        json: bool,
    },

    /// T-1820: Audit `~/.termlink/secrets/*.hex` for security hygiene.
    /// Scans the secrets directory directly (independent of `hubs.toml`),
    /// reporting file perms (must be 0o600 — flags world-/group-readable),
    /// payload size (must be 64-char hex = 32 bytes), and orphan status
    /// (no profile in `hubs.toml` references this file). Complements
    /// `fleet status`, which only inspects perms for hex files referenced
    /// by a profile — orphan caches left behind by IP renumbering or
    /// legacy heal flows are invisible to that path. Read-only; never
    /// authenticates; never contacts a hub; safe for cron/CI.
    /// Closes G-011 item 4 (the 2026-04 incident: `proxmox4.hex` at 0o644).
    SecretsAudit {
        /// Override scan directory (default: `~/.termlink/secrets`)
        #[arg(long)]
        dir: Option<String>,

        /// T-1822: Path to the authoritative `<runtime_dir>/hub.secret` (e.g.
        /// `/var/lib/termlink/hub.secret` or `/tmp/termlink-0/hub.secret`).
        /// When set, the audit reads the named file and adds a drift verdict
        /// to each scanned cache row: `ok-mirror` (content matches), `warn-drift`
        /// (differs), or no drift verdict if the cache is already flagged
        /// warn-format. Closes G-011 item 1 — the 2026-04-20 PL-041 case where
        /// the giving-end's IP-keyed cache had rotted silently after a hub
        /// restart.
        #[arg(long, value_name = "PATH")]
        check_drift: Option<String>,

        /// T-1824: Narrow the drift check to one named cache file. Only valid
        /// when paired with `--check-drift`. When set, only the named cache
        /// gets an ok-mirror/warn-drift verdict; every other cache file
        /// keeps its plain perms/format/orphan verdict. Use this when the
        /// dir contains caches for multiple hubs (peer caches + self caches);
        /// broad-mode `--check-drift` alone would flag every peer cache as
        /// `warn-drift` which is operator-expected, not a real problem.
        #[arg(long, value_name = "PATH")]
        target_cache: Option<String>,

        /// Output as JSON: {ok, dir, files: [{path, mode, size, status, reasons[], referenced_by[]}], summary}
        #[arg(long)]
        json: bool,
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

    /// Contact a peer agent on its canonical `dm:<a>:<b>` topic (T-1429
    /// Phase-1 + Phase-2 mostly shipped).
    ///
    /// Resolves <target> via local `session.discover`, reads the peer's
    /// `identity_fingerprint` from their registration metadata (T-1436),
    /// computes the canonical `dm:<sorted_a>:<sorted_b>` topic, and posts
    /// the message there. Replaces the `remote push` improv pattern for
    /// agent-to-agent contact. See agent-chat-arc topic description for the
    /// full envelope canon (RFC: T-1425). Strict identity-binding lands in
    /// T-1427; this build relies on the authoritative `sender_id` derived
    /// from the local identity key.
    ///
    /// Phase-2 shipped: --thread (task-id routing via `metadata._thread`),
    /// --file (body from path), --require-online + --online-window-secs
    /// (T-1480 presence pre-flight, exit 9), --ack-required +
    /// --ack-timeout-secs (T-1485 synchronous wait, exit 10), --target-fp
    /// (cross-host bypass when local session.discover can't reach the peer),
    /// --dry-run (T-1478 preview without posting).
    ///
    /// Phase-2 still deferred: `name@hub:port` advanced target syntax for
    /// federated name resolution across hubs — track in T-1429 task.
    Contact {
        /// Target session's display_name, optionally qualified with a
        /// project (T-1448 (b)): `<name>` or `<name>:<project>`.
        ///
        /// The bare `<name>` form resolves via local `session.discover` and
        /// reaches whichever co-resident agent is sharing the host's
        /// identity_fingerprint (T-1448: co-resident agents share a single
        /// keypair). When you type `<name>:<project>`, the project suffix
        /// is stamped as `metadata.to_project=<project>` on the dm post —
        /// receivers can filter on `to_project == own from_project` to
        /// disambiguate co-resident sessions.
        ///
        /// Optional iff `--target-fp` is given (the `:project` syntax
        /// applies only to the positional target; with `--target-fp`, pass
        /// `--metadata to_project=<project>` via the channel post path).
        target: Option<String>,

        /// Target identity fingerprint (hex). Use this when the peer is
        /// on a remote hub and the local box can't resolve them via
        /// session.discover. Mutually exclusive with the positional
        /// <TARGET>. Cross-host bypass for the Phase-2 federation gap
        /// (T-1429): caller computes `dm:<sorted_a>:<sorted_b>` from
        /// local identity + this fingerprint without needing to
        /// discover the peer's session metadata locally.
        #[arg(long = "target-fp")]
        target_fp: Option<String>,

        /// Message body. Mutually exclusive with `--file`; exactly one must
        /// be set. For large structured handoffs prefer `--file`.
        #[arg(long)]
        message: Option<String>,

        /// Read message body from a file (T-1646, T-1429 Phase-2 partial).
        /// Mutually exclusive with `--message`; exactly one must be set. The
        /// file is read as UTF-8 via `fs::read_to_string`; empty files are
        /// rejected with a clear error. Useful for large structured
        /// handoffs (proposals, RCAs) where inline `--message "<string>"`
        /// would hit shell-quoting hazards. The peer-command `channel post`
        /// already supports stdin-payload; this brings the same ergonomic
        /// to `agent contact`.
        #[arg(long)]
        file: Option<std::path::PathBuf>,

        /// Thread / task id for routing (sets `metadata._thread=<value>`,
        /// per agent-chat-arc protocol canon). When set, vendored agents
        /// can group dm:* messages by thread server-side without parsing
        /// the body. Protocol invariant: `_thread` is the canonical key
        /// (matches the agent-chat-arc topic description); a typical value
        /// is the task id like `T-1431`.
        #[arg(long)]
        thread: Option<String>,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Preview the resolved dm topic and metadata without posting
        /// (T-1478). Resolves target → FP, computes the canonical
        /// `dm:<sorted_a>:<sorted_b>` topic, builds the metadata block
        /// (from_project auto-injected, to_project from `<name>:<project>`
        /// suffix, `_thread` from `--thread`), prints a JSON preview to
        /// stdout, and exits 0 without contacting the hub for the post.
        /// Use this to verify name resolution and metadata stamping before
        /// committing to a real post — useful in CI or for ad-hoc ops
        /// validation. Same target-resolution errors fire as the live path.
        #[arg(long = "dry-run")]
        dry_run: bool,

        /// Fail-fast (exit 9) when the peer fingerprint hasn't appeared on
        /// `agent-chat-arc` within `--online-window-secs` (T-1480, Q3
        /// deferred from T-1425). Default behavior is queue-on-offline:
        /// chat-arc is offset-durable so dm posts persist until the peer
        /// catches up. Pass this flag for synchronous-contact semantics
        /// where you want to know NOW whether the peer is reachable
        /// (e.g. operator-driven incident chatter, CI gate). Combines with
        /// `--dry-run` to preview the verdict without posting.
        #[arg(long = "require-online")]
        require_online: bool,

        /// Window (seconds) for `--require-online` presence check. Default
        /// 300 (5 min) — covers a few missed heartbeats on a 1/min-cadence
        /// peer. Clamped to [10, 86400]. Ignored without `--require-online`.
        #[arg(long = "online-window-secs", default_value_t = 300)]
        online_window_secs: u64,

        /// Wait for the peer to post back on the dm topic before exiting
        /// (T-1485, Q4 deferred from T-1425). Default behavior is fire-and-
        /// forget. With this flag set: after the post, poll the dm topic
        /// for any non-meta message from the peer's fp posted *after* our
        /// send. On ack: exit 0. On `--ack-timeout-secs` exceeded: exit 10.
        /// Pairs with `--require-online` (pre-flight) for full synchronous-
        /// engagement semantic.
        #[arg(long = "ack-required")]
        ack_required: bool,

        /// Timeout (seconds) for `--ack-required` poll. Default 60.
        /// Clamped to [5, 600]. Ignored without `--ack-required`.
        #[arg(long = "ack-timeout-secs", default_value_t = 60)]
        ack_timeout_secs: u64,
    },

    /// Fleet-wide peer presence — companion to `agent who` (T-1482).
    /// Walks recent `agent-chat-arc` activity, aggregates non-meta posts
    /// by sender_id, and renders one row per active peer with last_seen,
    /// posts in window, and top from_project. Sorted by posts desc. Use
    /// this for fleet situational awareness ("who's on the wire right
    /// now and what are they working on?") — `agent who` is per-peer.
    Presence {
        /// Window (seconds) for the activity slice. Default 3600 (1h).
        /// Clamped to [60, 604800] (1 minute to 1 week). Peers with zero
        /// in-window posts are filtered out (not "present").
        #[arg(long = "window-secs", default_value_t = 3600)]
        window_secs: u64,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Filter by project: only count posts whose `from_project`
        /// metadata equals this value. Peers with zero matching posts in
        /// the window are excluded. Untagged posts also fail the filter
        /// (T-1484). Use this for project-scoped triage — "who's working
        /// on 010-termlink right now?".
        #[arg(long = "filter-project")]
        filter_project: Option<String>,

        /// Filter activity by thread/task id (T-1490). When set, only
        /// posts whose `metadata._thread == <name>` count toward
        /// presence; peers with zero matching posts are excluded.
        /// Untagged posts also fail. AND-composes with `--filter-project`
        /// — when both are set, a post must match BOTH to count. Use
        /// this for fleet-wide thread triage — "who's working on T-1485?".
        #[arg(long = "thread")]
        filter_thread: Option<String>,

        /// Live dashboard mode (T-1486): re-render every
        /// `--watch-interval` seconds until Ctrl-C. Composes with
        /// `--filter-project`, `--window-secs`, `--hub`. Incompatible
        /// with `--json` (one-shot vs streaming mismatch).
        #[arg(long)]
        watch: bool,

        /// Refresh interval (seconds) for `--watch` mode. Default 5.
        /// Clamped to [1, 300]. Ignored without `--watch`.
        #[arg(long = "watch-interval", default_value_t = 5)]
        watch_interval: u64,

        /// Limit output to N busiest peers post-sort (T-1489). When set,
        /// output is truncated to the first N rows after sorting; the
        /// total count is preserved in the footer / JSON envelope so the
        /// operator knows what was clipped. Clamped to [1, 1000].
        #[arg(long = "top")]
        top: Option<usize>,

        /// Aggregate by project instead of by peer (T-1491). Output is
        /// one row per `from_project` (untagged posts excluded), with
        /// posts/peers/top_peer/last_seen. Composes with `--filter-project`
        /// (collapses to single row), `--thread`, `--top`, `--watch`,
        /// `--json`. JSON envelope swaps `peers` → `projects` and adds
        /// `view: "by-project"`.
        #[arg(long = "by-project")]
        by_project: bool,
    },

    /// Peer observability — summarize a peer's recent `agent-chat-arc`
    /// activity (T-1481). Returns: identity fingerprint, last_seen on the
    /// canonical liveness arc, posts in the window, and distinct
    /// `from_project` values observed (with per-project post counts).
    /// Cross-host disambiguation primitive: when you have an unknown FP
    /// and need to know "who is this and what are they working on?" before
    /// engaging with `agent contact`. Pairs with `agent contact --dry-run`
    /// for end-to-end pre-flight verification.
    Who {
        /// Target identity fingerprint (hex, ≥8 chars). Cross-host
        /// disambiguation lookup — no local session.discover required.
        /// Mutually exclusive with --target.
        #[arg(long = "target-fp")]
        target_fp: Option<String>,

        /// Target session display_name — resolves locally via
        /// `session.discover`, mirror of `agent contact <target>`. Use this
        /// for local-hub investigations where you already have the peer's
        /// name. Mutually exclusive with --target-fp; one is required.
        #[arg(long = "target")]
        target: Option<String>,

        /// Window (seconds) for the activity slice. Default 3600 (1h).
        /// Clamped to [60, 604800] (1 minute to 1 week). Counts posts and
        /// groups `from_project` values within the window.
        #[arg(long = "window-secs", default_value_t = 3600)]
        window_secs: u64,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Filter activity by thread/task id (T-1488). When set, only
        /// posts whose `metadata._thread == <name>` count toward
        /// posts_in_window / from_projects. Untagged posts also fail
        /// the filter. `last_seen` is independent of filter (still
        /// reflects ANY peer post). Use this to scope a peer's activity
        /// to a specific task — "what's alice doing on T-1485?".
        #[arg(long = "thread")]
        filter_thread: Option<String>,
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

    /// Operator-facing presence check (T-1487). One-line "is X alive on
    /// agent-chat-arc?" verb. Composes T-1483 name resolution + T-1480
    /// presence probe — no posts, just heartbeat inspection. Exit 0 if
    /// the peer has been seen on chat-arc within `--window-secs`, exit 1
    /// otherwise. Use this for the simplest operator question; reach for
    /// `agent who` when you want activity detail or `agent presence` for
    /// the fleet view.
    Ping {
        /// Target session display_name (resolves locally via
        /// `session.discover` mirror of `agent contact`). Mutually
        /// exclusive with `--target-fp`; one is required.
        target: Option<String>,

        /// Target identity fingerprint (hex, ≥8 chars). Cross-host
        /// path — no local session.discover required. Mutually
        /// exclusive with the positional `<TARGET>`.
        #[arg(long = "target-fp")]
        target_fp: Option<String>,

        /// Window (seconds) for the presence check. Default 300 (5min)
        /// — covers a few missed heartbeats on a 1/min-cadence peer.
        /// Clamped to [10, 86400].
        #[arg(long = "window-secs", default_value_t = 300)]
        window_secs: u64,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON
        #[arg(long)]
        json: bool,
    },

    /// Show recent chat-arc posts from a single peer (T-1492). Content-
    /// access companion to `agent who` / `agent presence`: those tell
    /// you who is active and what they're working on; this shows what
    /// they've actually said. Walks `agent-chat-arc`, filters to a
    /// single sender, optionally narrows by thread/project, prints the
    /// last N posts in chronological order with content snippets.
    Recent {
        /// Target session display_name — resolves locally via
        /// `session.discover`. Mutually exclusive with `--target-fp`;
        /// one is required.
        target: Option<String>,

        /// Target identity fingerprint (hex, ≥8 chars). Cross-host
        /// path — no local session.discover required. Mutually
        /// exclusive with the positional `<TARGET>`.
        #[arg(long = "target-fp")]
        target_fp: Option<String>,

        /// Number of posts to return. Default 10. Clamped to [1, 200].
        #[arg(long = "n", default_value_t = 10)]
        n: usize,

        /// Window (seconds) for the activity slice. Default 86400 (1
        /// day). Clamped to [60, 604800] (1 minute to 1 week). Posts
        /// older than the window are excluded even if `--n` is large.
        #[arg(long = "window-secs", default_value_t = 86400)]
        window_secs: u64,

        /// Filter by thread/task id — only posts whose
        /// `metadata._thread == <name>` are returned.
        #[arg(long = "thread")]
        filter_thread: Option<String>,

        /// Filter by project — only posts whose
        /// `metadata.from_project == <name>` are returned.
        #[arg(long = "project")]
        filter_project: Option<String>,

        /// T-1499: msg_type allowlist (comma-separated). When set,
        /// only posts whose `msg_type` is in the list are returned.
        /// Useful for signal-vs-noise filtering: `--msg-type note`
        /// hides heartbeat-style status/star posts. Composes with
        /// the other filters (AND-composed). Meta types
        /// (reaction/edit/redaction/topic_metadata/receipt) are
        /// always excluded — listing them here does not re-include them.
        #[arg(long = "msg-type", value_delimiter = ',')]
        filter_msg_types: Vec<String>,

        /// T-1501: case-insensitive substring filter against post
        /// content. AND-composes with peer/thread/project/msg-type.
        /// Empty pattern is treated as no filter.
        #[arg(long = "grep")]
        filter_grep: Option<String>,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope
        #[arg(long)]
        json: bool,

        /// Live single-peer streaming mode (T-1498): re-render the
        /// last-N posts from this peer every `--watch-interval` seconds
        /// until Ctrl-C. Composes with `--n`, `--window-secs`,
        /// `--thread`, `--project`, `--hub`. Incompatible with `--json`
        /// (streaming vs one-shot).
        #[arg(long)]
        watch: bool,

        /// Refresh interval (seconds) for `--watch` mode. Default 5.
        /// Clamped to [1, 300]. Ignored without `--watch`.
        #[arg(long = "watch-interval", default_value_t = 5)]
        watch_interval: u64,

        /// T-1817: history depth — number of recent chat-arc envelopes to
        /// fetch before filtering by peer/thread/window. Default 1000 (the
        /// hub's per-page cap, single round-trip — equivalent to pre-T-1817
        /// behavior). Values >1000 trigger bounded multi-page pagination
        /// (T-1796 `fetch_topic_msgs_paginated`) — useful on busy fleets
        /// where the most-recent 1000 envelopes contain few posts from
        /// this peer. Clamped to [1, 100000].
        #[arg(long = "depth", default_value_t = 1000)]
        depth: u64,
    },

    /// Show all recent posts on a thread/task across the fleet (T-1493).
    /// Companion to `agent who --thread` (per-peer aggregate) and
    /// `agent presence --thread` (fleet aggregate): those summarize
    /// activity; this is the chronological reading view. Walks
    /// `agent-chat-arc` filtered by `metadata._thread`, optionally
    /// further by project/peer, and prints last N posts in time order
    /// — natural for "let me read what happened on T-XXX".
    OnThread {
        /// Thread/task identifier (e.g. T-1485). Required positional.
        thread: String,

        /// Number of posts to return. Default 50 (denser than `recent`'s
        /// 10 because thread logs typically have many posts). Clamped
        /// to [1, 500].
        #[arg(long = "n", default_value_t = 50)]
        n: usize,

        /// Window (seconds) for the activity slice. Default 86400 (1
        /// day). Clamped to [60, 604800].
        #[arg(long = "window-secs", default_value_t = 86400)]
        window_secs: u64,

        /// Further filter by project — only posts whose
        /// `metadata.from_project == <name>` are returned.
        #[arg(long = "project")]
        filter_project: Option<String>,

        /// T-1499: msg_type allowlist (comma-separated). When set,
        /// only posts whose `msg_type` is in the list are returned.
        /// Same semantics as `agent recent --msg-type`: AND-composes
        /// with peer/project filters; meta types always excluded.
        #[arg(long = "msg-type", value_delimiter = ',')]
        filter_msg_types: Vec<String>,

        /// T-1501: case-insensitive substring filter against post
        /// content. AND-composes with peer/project/msg-type. Empty
        /// pattern treated as no filter.
        #[arg(long = "grep")]
        filter_grep: Option<String>,

        /// Further narrow to a single peer's posts on this thread —
        /// resolves locally via `session.discover`. Mutually exclusive
        /// with `--peer-fp`.
        #[arg(long = "peer")]
        peer: Option<String>,

        /// Further narrow to a single peer's posts on this thread —
        /// hex fingerprint, no name resolution. Mutually exclusive
        /// with `--peer`.
        #[arg(long = "peer-fp")]
        peer_fp: Option<String>,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope
        #[arg(long)]
        json: bool,

        /// Live thread-following mode (T-1494): re-render every
        /// `--watch-interval` seconds until Ctrl-C. Composes with
        /// `--n`, `--window-secs`, `--project`, `--peer`, `--peer-fp`,
        /// `--hub`. Incompatible with `--json` (one-shot vs streaming).
        #[arg(long)]
        watch: bool,

        /// Refresh interval (seconds) for `--watch` mode. Default 5.
        /// Clamped to [1, 300]. Ignored without `--watch`.
        #[arg(long = "watch-interval", default_value_t = 5)]
        watch_interval: u64,

        /// T-1816: history depth — number of recent chat-arc envelopes to
        /// fetch before filtering by thread/window/peer. Default 1000 (the
        /// hub's per-page cap, single round-trip — equivalent to pre-T-1816
        /// behavior). Values >1000 trigger bounded multi-page pagination
        /// (T-1796 `fetch_topic_msgs_paginated`) — useful on busy fleets
        /// where the most-recent 1000 envelopes contain few matching thread
        /// posts. Clamped to [1, 100000].
        #[arg(long = "depth", default_value_t = 1000)]
        depth: u64,
    },

    /// Fleet-wide chronological log (T-1500): all posts across all peers
    /// in a window, time-ordered, peer-short prefixed. "tail -f for the
    /// fleet" — companion to `recent <peer>` (one peer) and
    /// `on-thread <T-XXX>` (one thread). No peer/thread filter required.
    /// Pure wrapper around `extract_recent_posts(..., peer=None, ...)`;
    /// composes with --thread, --project, --msg-type, --watch, --json.
    Timeline {
        /// Number of posts to return. Default 50. Clamped to [1, 500].
        #[arg(long = "n", default_value_t = 50)]
        n: usize,

        /// Window (seconds) for the activity slice. Default 3600 (1h).
        /// Clamped to [60, 604800].
        #[arg(long = "window-secs", default_value_t = 3600)]
        window_secs: u64,

        /// Restrict to one thread (e.g. T-1485). Optional.
        #[arg(long = "thread")]
        filter_thread: Option<String>,

        /// Restrict to one project — only posts whose
        /// `metadata.from_project == <name>` are returned.
        #[arg(long = "project")]
        filter_project: Option<String>,

        /// msg_type allowlist (comma-separated). Same semantics as
        /// `agent recent --msg-type`: AND-composes with thread/project;
        /// meta types always excluded.
        #[arg(long = "msg-type", value_delimiter = ',')]
        filter_msg_types: Vec<String>,

        /// T-1501: case-insensitive substring filter against post
        /// content. AND-composes with thread/project/msg-type. Empty
        /// pattern treated as no filter.
        #[arg(long = "grep")]
        filter_grep: Option<String>,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope
        #[arg(long)]
        json: bool,

        /// Live tail-f mode: re-render every `--watch-interval` seconds
        /// until Ctrl-C. Incompatible with `--json` (one-shot vs streaming).
        #[arg(long)]
        watch: bool,

        /// Refresh interval (seconds) for `--watch` mode. Default 5.
        /// Clamped to [1, 300]. Ignored without `--watch`.
        #[arg(long = "watch-interval", default_value_t = 5)]
        watch_interval: u64,

        /// T-1818: history depth — number of recent chat-arc envelopes to
        /// fetch before filtering by thread/project/msg-type/grep. Default
        /// 1000 (the hub's per-page cap, single round-trip — equivalent to
        /// pre-T-1818 behavior). Values >1000 trigger bounded multi-page
        /// pagination (T-1796 `fetch_topic_msgs_paginated`) — useful on
        /// busy fleets where the most-recent 1000 envelopes only cover a
        /// short window. Clamped to [1, 100000].
        #[arg(long = "depth", default_value_t = 1000)]
        depth: u64,
    },

    /// Single-shot fleet digest (T-1495): top peers + top projects + last
    /// posts in one render. Designed as the first command of a session —
    /// "what's the fleet doing right now?". Composes existing pure
    /// helpers (`summarize_fleet_presence`, `summarize_fleet_by_project`,
    /// `extract_recent_posts`) on a single chat-arc fetch, so a digest
    /// is one RPC round-trip.
    Overview {
        /// Window (seconds) for the activity slice. Default 3600 (1h).
        /// Clamped to [60, 604800].
        #[arg(long = "window-secs", default_value_t = 3600)]
        window_secs: u64,

        /// Number of rows per section. Default 5. Clamped to [1, 50].
        /// Applies symmetrically to peers / projects / recent posts.
        #[arg(long = "top", default_value_t = 5)]
        top: usize,

        /// Override hub address (default: local hub)
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope
        #[arg(long)]
        json: bool,

        /// T-1819: history depth — number of recent chat-arc envelopes to
        /// fetch before computing the 3-section digest. Default 1000 (the
        /// hub's per-page cap, single round-trip — equivalent to pre-T-1819
        /// behavior). Values >1000 trigger bounded multi-page pagination
        /// (T-1796 `fetch_topic_msgs_paginated`) — useful on busy fleets
        /// where 1000 envelopes only cover a short window. Clamped to
        /// [1, 100000].
        #[arg(long = "depth", default_value_t = 1000)]
        depth: u64,

        /// Live fleet dashboard mode (T-1496): re-render the 3-section
        /// overview every `--watch-interval` seconds until Ctrl-C.
        /// Composes with `--window-secs`, `--top`, `--hub`. Incompatible
        /// with `--json` (streaming vs one-shot).
        #[arg(long)]
        watch: bool,

        /// Refresh interval (seconds) for `--watch` mode. Default 5.
        /// Clamped to [1, 300]. Ignored without `--watch`.
        #[arg(long = "watch-interval", default_value_t = 5)]
        watch_interval: u64,
    },

    /// Fleet-wide aggregate counts (T-1504): single-fetch summary of
    /// chat-arc activity in a window, grouped by msg_type / peer /
    /// project / thread. Operator's "what has the fleet been doing?"
    /// view. Companion to presence (per-peer) / who (single-peer detail)
    /// / timeline (chronological log).
    Stats {
        /// Window (seconds) for the activity slice. Default 86400 (1d).
        /// Clamped to [60, 604800].
        #[arg(long = "window-secs", default_value_t = 86400)]
        window_secs: u64,

        /// Top-N rows per section. Default 10. Clamped to [1, 100].
        #[arg(long = "top", default_value_t = 10)]
        top: usize,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Post a note to agent-chat-arc (T-1503): focus-aware write companion
    /// to the recent/on-thread/timeline reading verbs. Auto-resolves
    /// `--thread` from `.context/working/focus.yaml::current_task` and
    /// `--project` from `.framework.yaml::project_name` if not given.
    /// Posts as `msg_type=note` by default.
    Post {
        /// Text body of the post (positional, required).
        text: String,

        /// Override the thread/task id. If omitted, resolves from
        /// `.context/working/focus.yaml::current_task`.
        #[arg(long = "thread")]
        thread: Option<String>,

        /// Override the project. If omitted, resolves from
        /// `.framework.yaml::project_name`.
        #[arg(long = "project")]
        project: Option<String>,

        /// Message type. Default "note".
        #[arg(long = "msg-type", default_value = "note")]
        msg_type: String,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Walk UP the in_reply_to chain from <OFFSET> to root on agent-chat-arc
    /// (T-1510): thin wrapper over `channel ancestors agent-chat-arc <offset>`.
    /// Companion to `agent thread` (down-walk) — together they let an
    /// operator reconstruct any conversation from any offset.
    Ancestors {
        /// Arc offset to start walking up from (positional, required).
        offset: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Render the full reply subtree rooted at <ROOT> on agent-chat-arc
    /// (T-1509): thin wrapper over `channel thread agent-chat-arc <root>`.
    /// Renders the root post and every descendant linked by
    /// `metadata.in_reply_to`. Compose with `agent reply` (T-1507) to
    /// build threads, `agent quote` (T-1505) to walk parents up, and
    /// `agent thread` to render the whole subtree from any root.
    Thread {
        /// Arc offset of the thread root (positional, required).
        root: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Search the full chat-arc for a substring (T-1508): unbounded by window.
    /// `agent recent --grep` and `agent timeline --grep` are window-capped at
    /// 7 days; `agent search` walks the entire arc and runs the same
    /// case-insensitive substring filter. Returns the LAST N matches in
    /// chronological order. Use it to answer "did anyone ever mention X?"
    Search {
        /// Substring to search for (positional, required, case-insensitive).
        query: String,

        /// Limit results to last N matches. Default 20. Clamped to [1, 500].
        #[arg(long = "n", default_value_t = 20)]
        n: usize,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Reply to a chat-arc post (T-1507): write counterpart to `agent quote`.
    /// Posts `<text>` with `metadata.in_reply_to=<offset>` so the new envelope
    /// shows the parent chain in subsequent `agent quote <new-offset>` and
    /// fits Matrix-style reply traversal. Inherits focus-aware metadata
    /// resolution from `agent post` (thread from focus.yaml, project from
    /// .framework.yaml) — overridable per call.
    Reply {
        /// Arc offset of the post being replied to (positional, required).
        offset: u64,

        /// Text body of the reply (positional, required).
        text: String,

        /// Override the thread/task id. If omitted, resolves from
        /// `.context/working/focus.yaml::current_task`.
        #[arg(long = "thread")]
        thread: Option<String>,

        /// Override the project. If omitted, resolves from
        /// `.framework.yaml::project_name`.
        #[arg(long = "project")]
        project: Option<String>,

        /// Message type. Default "note".
        #[arg(long = "msg-type", default_value = "note")]
        msg_type: String,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Quote a single agent-chat-arc post by offset (T-1505): thin wrapper
    /// over `channel quote agent-chat-arc <offset>`. Renders the post and
    /// its parent (when the post is a reply via `metadata.in_reply_to`).
    /// Operator picks an offset from `agent timeline` / `agent recent` /
    /// `agent on-thread` and pulls the full content here.
    Quote {
        /// Arc offset (positional, required).
        offset: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Read agent-chat-arc topic metadata + counts (T-1524): thin wrapper
    /// over `channel info agent-chat-arc`. Surfaces retention, count,
    /// latest topic description, distinct senders summary, and per-sender
    /// receipts. Operator answer to "what is chat-arc + how big + who's
    /// here + how far behind is each peer".
    Info {
        /// Restrict description/senders/receipts computation to envelopes
        /// since this epoch-ms timestamp. Total count stays unbounded so
        /// "12 of 23 in last hour" stays visible.
        #[arg(long)]
        since: Option<i64>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List reactions emitted BY a sender on agent-chat-arc (T-1521):
    /// thin wrapper over `channel reactions-of agent-chat-arc`. Distinct
    /// from `agent reactions <offset>` (parent-side aggregation): this is
    /// sender-side — "where has agent X reacted on chat-arc?". Default
    /// scope is the local identity FP.
    ReactionsOf {
        /// Sender identity to scope to (default: local FP).
        #[arg(long)]
        sender: Option<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List forwards emitted BY a sender on agent-chat-arc (T-1522):
    /// thin wrapper over `channel forwards-of agent-chat-arc`. Forward
    /// envelopes carry `metadata.forwarded_from=<topic>:<offset>` —
    /// surface the cross-topic re-share trail for a peer. Default
    /// scope is the local identity FP.
    ForwardsOf {
        /// Sender identity to scope to (default: local FP).
        #[arg(long)]
        sender: Option<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List replies authored BY a sender on agent-chat-arc (T-1523):
    /// thin wrapper over `channel replies-of agent-chat-arc`. Distinct
    /// from `agent thread` (subtree) and `agent ancestors` (up-walk):
    /// replies-of is "everywhere this peer has replied to someone, with
    /// the parent for context". Default scope is the local identity FP.
    RepliesOf {
        /// Sender identity to scope to (default: local FP).
        #[arg(long)]
        sender: Option<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Lifetime fleet directory of agent-chat-arc (T-1520): thin wrapper
    /// over `channel members agent-chat-arc`. Lists every distinct
    /// `sender_id` who has posted, with `posts` count, `first_ts`, `last_ts`.
    /// Distinct from `agent presence` (windowed activity) and `agent who`
    /// (single-peer detail): peers is the lifetime "who has ever participated"
    /// view. `--include-meta` counts meta-type envelopes (reactions/edits);
    /// `--as-of` snapshots membership at a given timestamp.
    Peers {
        /// Count meta-type envelopes (reaction/edit/redaction) toward
        /// posts/first/last. Default off — content envelopes only.
        #[arg(long = "include-meta")]
        include_meta: bool,

        /// Snapshot membership as-of this epoch-ms timestamp. Default:
        /// current time (full lifetime view).
        #[arg(long = "as-of")]
        as_of: Option<i64>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Show a windowed context view around a chat-arc offset (T-1519):
    /// thin wrapper over `channel snippet agent-chat-arc <offset>`. Picks
    /// up to N content envelopes on each side (msg_type filtered to
    /// post/chat/note — meta types excluded), renders a fenced markdown
    /// block with `>>` marking the target. Distinct from `agent quote`
    /// (parent + immediate replies) and `agent thread` (subtree): snippet
    /// is chronological context, not threading.
    Snippet {
        /// Arc offset to center the snippet on (positional, required).
        offset: u64,

        /// Lines of context on each side of the target. Default 3.
        #[arg(long, default_value_t = 3)]
        lines: u64,

        /// Print a `From '<topic>' @ offset N:` header above the block.
        #[arg(long)]
        header: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List active pinned posts on agent-chat-arc (T-1517): thin wrapper
    /// over `channel pinned agent-chat-arc`. Computes active pin set
    /// (latest pin/unpin action per target wins) and renders one row per
    /// pinned offset. Companion to `agent starred` (T-1518): pinned is
    /// admin-attention, starred is personal-attention.
    Pinned {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List starred posts on agent-chat-arc (T-1518): thin wrapper over
    /// `channel starred agent-chat-arc`. Default scope is the local
    /// identity ("you"); pass `--all` for fleet-wide stars. Companion
    /// to `agent pinned` (T-1517): stars are personal bookmarks; pins
    /// are admin attention.
    Starred {
        /// Show stars from every identity, not just the local one.
        #[arg(long)]
        all: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Lifetime structural breakdown of agent-chat-arc (T-1516): thin wrapper
    /// over `channel topic-stats agent-chat-arc`. Total envelopes, distinct
    /// senders, by_msg_type histogram, top senders, top emojis, thread roots,
    /// active pins, forwards_in, edits, redactions, time span. Distinct from
    /// `agent stats` (windowed) and `agent digest` (period-summary): this
    /// answers "what's the shape of the arc?".
    TopicStats {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Lifetime emoji-reaction aggregates for agent-chat-arc (T-1515): thin
    /// wrapper over `channel emoji-stats agent-chat-arc`. Walks the full
    /// arc, groups `msg_type=reaction` envelopes by emoji, sorts by count
    /// desc. Companion to `agent reactions <offset>` (per-post): emoji-stats
    /// is per-arc — surfaces dominant ack signals fleet-wide.
    EmojiStats {
        /// Show per-reactor breakdown under each emoji row.
        #[arg(long = "by-sender")]
        by_sender: bool,

        /// Truncate to top N rows. Default: no truncation.
        #[arg(long)]
        top: Option<usize>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Show reactions on a specific chat-arc post (T-1514): thin wrapper over
    /// `channel reactions-on agent-chat-arc <offset>`. Aggregates all
    /// `msg_type=reaction` envelopes whose `metadata.target=<offset>`,
    /// groups by emoji, prints `{emoji × count — senders}` rows. Pairs
    /// with `agent quote <offset>` — together they answer "what did this
    /// post say + how was it received".
    Reactions {
        /// Arc offset of the post to inspect (positional, required).
        offset: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List every chat-arc envelope whose `metadata.mentions` CSV matches
    /// <USER> (T-1513): thin wrapper over `channel mentions-of agent-chat-arc <user>`.
    /// Distinct from `agent search` (free-text substring across payload):
    /// this verb filters on the structured `metadata.mentions` array — so
    /// you find rows that explicitly tagged the user, regardless of body
    /// content. Glob `*` matches any non-empty mentions CSV.
    Mentions {
        /// User identity, peer name, or thread tag to match against
        /// `metadata.mentions`. Use `"*"` to match any non-empty CSV.
        user: String,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Count new posts on agent-chat-arc since the caller's last channel.ack
    /// receipt (T-1512): thin wrapper over `channel unread agent-chat-arc`.
    /// Queries hub-side receipts for the local identity (or override via
    /// `--sender`), walks the arc from up_to+1, and reports unread count
    /// plus first/last new offsets. Operator workflow: `agent unread` →
    /// "N unread" → `agent timeline -n N` to catch up. T-1559: `--watch`
    /// flips on a live monitor refreshing every `--watch-interval` seconds.
    Unread {
        /// Sender identity to check unread for (default: local identity FP).
        /// Use this to query "what would peer X see as unread".
        #[arg(long)]
        sender: Option<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,

        /// Live monitor mode: re-fetch and re-render every
        /// `--watch-interval` seconds. Incompatible with `--json`.
        #[arg(long)]
        watch: bool,

        /// Refresh interval in seconds when `--watch` is set. Clamped to
        /// [1, 300]. Default 3s — chat-arc-only unread is more time-
        /// sensitive than full inbox (T-1558 uses 5s).
        #[arg(long, default_value = "3")]
        watch_interval: u64,
    },

    /// Period summary of agent-chat-arc activity (T-1511): thin wrapper over
    /// `channel digest agent-chat-arc`. Counts posts, distinct senders,
    /// top senders by volume, top reactions, pin/forward activity, and
    /// surfaces a recent_chats sample. Pairs with `agent timeline` (raw
    /// stream) and `agent stats` (lifetime counts) — `agent digest` is
    /// the compressed view of a recent slice. Default window: last 60 min.
    Digest {
        /// Window size in minutes (default 60). Mutually exclusive with --since.
        #[arg(long = "since-mins")]
        since_mins: Option<i64>,

        /// Absolute lower-bound ts_ms (epoch milliseconds). Mutually
        /// exclusive with --since-mins.
        #[arg(long = "since")]
        since: Option<i64>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Emit a reaction on a chat-arc post (T-1525): thin wrapper over
    /// `channel react agent-chat-arc`. Posts a `msg_type=reaction` envelope
    /// with `metadata.target=<offset>` and `metadata.emoji=<reaction>`.
    /// Closes the engagement-emit primitive — read-side already shipped as
    /// `agent reactions <offset>` (T-1514), `agent reactions-of` (T-1521),
    /// `agent emoji-stats` (T-1515).
    React {
        /// Parent post offset to react to.
        offset: u64,

        /// Reaction emoji or short token (e.g. 👍, ✅, "+1").
        emoji: String,

        /// Remove a previously-emitted reaction instead of adding one.
        #[arg(long)]
        remove: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Advance the local read frontier on chat-arc (T-1526): thin wrapper
    /// over `channel ack agent-chat-arc`. Posts a `msg_type=receipt`
    /// envelope with `metadata.up_to=<offset>` so peers see "this sender
    /// has read everything up to N". Companion to T-1512 `agent unread`.
    Ack {
        /// Offset to ack up to (inclusive). Mutually exclusive with --since-ms.
        #[arg(long = "up-to")]
        up_to: Option<u64>,

        /// Ack everything posted at or after this epoch-ms (helper resolves
        /// to a concrete up_to). Mutually exclusive with --up-to.
        #[arg(long = "since-ms")]
        since_ms: Option<i64>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Pin a chat-arc post (T-1527): thin wrapper over
    /// `channel pin agent-chat-arc`. Posts a `msg_type=pin` envelope (or
    /// `unpin` with `--unpin`). Topic-wide curation visible to all peers
    /// — distinct from `agent star` (per-sender bookmark). Read-side
    /// shipped as T-1517 `agent pinned`.
    Pin {
        /// Offset to pin (or unpin with --unpin).
        offset: u64,

        /// Remove an existing pin instead of adding one.
        #[arg(long)]
        unpin: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Star a chat-arc post (T-1528): thin wrapper over
    /// `channel star agent-chat-arc`. Posts a `msg_type=star` envelope
    /// with `metadata.star_target=<offset>` for personal bookmarking.
    /// Distinct from `agent pin` (topic-wide curation). Read-side
    /// shipped as T-1518 `agent starred`.
    Star {
        /// Offset to star (or unstar with --unstar).
        offset: u64,

        /// Remove an existing star instead of adding one.
        #[arg(long)]
        unstar: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Re-publish a chat-arc post to another topic (T-1529): thin
    /// wrapper over `channel forward`. Walks `agent-chat-arc` to fetch
    /// the envelope at `<offset>` and re-posts it to `--to <TOPIC>` with
    /// `metadata.forwarded_from=agent-chat-arc:<offset>` and
    /// `metadata.forwarded_sender=<original-sender>`. Read-side shipped
    /// as T-1522 `agent forwards-of`.
    Forward {
        /// Offset on agent-chat-arc to forward.
        offset: u64,

        /// Destination topic.
        #[arg(long)]
        to: String,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Edit a prior chat-arc post (T-1530): thin wrapper over
    /// `channel edit agent-chat-arc`. Posts a `msg_type=edit` envelope
    /// with `metadata.replaces=<offset>` and the new text payload.
    /// Operator workflow: correct typos / refine wording on a prior
    /// post without authoring a fresh root.
    Edit {
        /// Offset of the prior post being edited.
        offset: u64,

        /// New text payload.
        text: String,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Retract a prior chat-arc post (T-1531): thin wrapper over
    /// `channel redact agent-chat-arc`. Posts a `msg_type=redaction`
    /// envelope with `metadata.redacts=<offset>` and optional
    /// `metadata.reason`. The arc remains immutable — readers see the
    /// redaction marker, not a deletion.
    Redact {
        /// Offset of the post to redact.
        offset: u64,

        /// Optional reason logged on the redaction envelope.
        #[arg(long)]
        reason: Option<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Set chat-arc topic_metadata description (T-1532): thin wrapper
    /// over `channel describe agent-chat-arc`. Posts a
    /// `msg_type=topic_metadata` envelope with
    /// `metadata.description=<text>`. WRITE companion to T-1524
    /// `agent info` which surfaces the latest description.
    Describe {
        /// New topic description text.
        text: String,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List thread roots on chat-arc (T-1533): thin wrapper over
    /// `channel threads agent-chat-arc`. Walks the arc, builds an index
    /// by `metadata.in_reply_to`, returns per-root thread metadata.
    /// Companion to T-1509 `agent thread <root>` (subtree render):
    /// threads is the index, thread is the deep-dive.
    Threads {
        /// Limit output to N busiest threads (post-sort).
        #[arg(long)]
        top: Option<usize>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List retracted posts on chat-arc (T-1534): thin wrapper over
    /// `channel redactions agent-chat-arc`. Walks the arc, surfaces
    /// every `msg_type=redaction` envelope. READ companion to T-1531
    /// `agent redact`.
    Redactions {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Pin/unpin event log on chat-arc (T-1535): thin wrapper over
    /// `channel pin-history agent-chat-arc`. Walks the arc, surfaces
    /// every pin/unpin event in chronological order. Audit companion
    /// to T-1517 `agent pinned` (current pins) and T-1527 `agent pin`.
    PinHistory {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Edit history of a chat-arc post (T-1536): thin wrapper over
    /// `channel edits-of agent-chat-arc`. Walks the arc and returns
    /// the chain of edits for `<offset>` (every `msg_type=edit` whose
    /// `metadata.replaces=<offset>`). READ companion to T-1530
    /// `agent edit`.
    EditsOf {
        /// Offset whose edit history to surface.
        offset: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// All relations of a chat-arc post (T-1537): thin wrapper over
    /// `channel relations agent-chat-arc`. Walks the arc and returns
    /// every envelope pointing AT `<offset>` — replies, edits,
    /// redactions, reactions, forwards, pins, stars. Single-shot
    /// "everything that touched offset N" view; pairs with the
    /// narrower thread/ancestors/reactions/edits-of verbs.
    Relations {
        /// Target offset whose relations to surface.
        offset: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Receipt log on chat-arc (T-1538): thin wrapper over
    /// `channel ack-history agent-chat-arc`. Returns every receipt
    /// envelope chronologically (sender + up_to). Companion to
    /// T-1526 `agent ack`.
    AckHistory {
        /// Filter to a specific user (sender_id fingerprint).
        #[arg(long)]
        user: Option<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Current ack frontiers per sender (T-1539): thin wrapper over
    /// `channel ack-status agent-chat-arc`. Returns each sender's
    /// latest receipt up_to + pending count. Snapshot companion to
    /// T-1538 (history).
    AckStatus {
        /// Show only senders with non-zero pending count.
        #[arg(long = "pending-only")]
        pending_only: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Reduced visible state on chat-arc (T-1540): thin wrapper over
    /// `channel state agent-chat-arc`. Returns every chat post that
    /// hasn't been redacted, with edit overlays applied. Pairs with
    /// T-1524 `agent info` (topic shape vs rendered conversation).
    State {
        /// Include redacted envelopes (rendered as redaction markers).
        #[arg(long = "include-redacted")]
        include_redacted: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Quote citation counts on chat-arc (T-1541): thin wrapper over
    /// `channel quote-stats agent-chat-arc`. Returns per-offset count
    /// of how often it's been quoted. Companion to T-1505
    /// `agent quote`.
    QuoteStats {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Edit-rate analytics on chat-arc (T-1542): thin wrapper over
    /// `channel edit-stats agent-chat-arc`. Per-sender edit ratio
    /// analytics. Companions: T-1530 `agent edit` (write),
    /// T-1536 `agent edits-of` (per-offset chain).
    EditStats {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Start a poll on chat-arc (T-1543): thin wrapper over
    /// `channel poll-start agent-chat-arc`. Posts a `msg_type=poll_start`
    /// envelope with the question and 2+ options. The returned offset
    /// becomes the canonical poll_id for vote/end/results. First of
    /// the 4-verb poll suite.
    PollStart {
        /// Poll question.
        question: String,

        /// Poll options (>=2). Repeat the flag for each option.
        #[arg(long = "option", required = true)]
        option: Vec<String>,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Cast a vote on a chat-arc poll (T-1544): thin wrapper over
    /// `channel poll-vote agent-chat-arc`. Posts a `msg_type=poll_vote`
    /// envelope with `poll_id` + zero-indexed `choice`.
    Vote {
        /// poll_id (offset of the poll_start envelope).
        poll_id: u64,

        /// Zero-indexed option choice.
        choice: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Close a chat-arc poll (T-1545): thin wrapper over
    /// `channel poll-end agent-chat-arc`. Posts a `msg_type=poll_end`
    /// envelope referencing the poll_id; late votes are rejected after.
    PollEnd {
        /// poll_id to close.
        poll_id: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Render a chat-arc poll's tally (T-1546): thin wrapper over
    /// `channel poll-results agent-chat-arc`. Walks the arc, returns
    /// per-option vote counts + voter lists + status (open/closed).
    /// READ side of the 4-verb poll suite.
    PollResults {
        /// poll_id whose tally to render.
        poll_id: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Point-in-time chat-arc state (T-1547): thin wrapper over
    /// `channel snapshot agent-chat-arc`. Returns the reduced visible
    /// state AS OF a specific timestamp (edits + redactions applied
    /// through that point). Companion to T-1540 `agent state` (current).
    Snapshot {
        /// Snapshot timestamp in epoch milliseconds.
        #[arg(long = "as-of")]
        as_of: i64,

        /// Include redacted envelopes (rendered as redaction markers).
        #[arg(long = "include-redacted")]
        include_redacted: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Chat-arc envelopes since a timestamp (T-1548): thin wrapper
    /// over `channel state-since agent-chat-arc`. Returns visible
    /// envelopes posted at-or-after `<MS>`. Operator workflow:
    /// "what's new since I last looked?".
    StateSince {
        /// Lower-bound timestamp in epoch milliseconds.
        #[arg(long = "since")]
        since: i64,

        /// Include redacted envelopes (rendered as redaction markers).
        #[arg(long = "include-redacted")]
        include_redacted: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Chat-arc state delta between two timestamps (T-1549): thin
    /// wrapper over `channel snapshot-diff agent-chat-arc`. Returns
    /// envelopes ADDED, EDITED, REDACTED between `--from` and `--to`.
    /// Pairs with T-1547 `agent snapshot`.
    SnapshotDiff {
        /// Earlier timestamp in epoch milliseconds.
        #[arg(long = "from")]
        from: i64,

        /// Later timestamp in epoch milliseconds.
        #[arg(long = "to")]
        to: i64,

        /// Include redacted envelopes (rendered as redaction markers).
        #[arg(long = "include-redacted")]
        include_redacted: bool,

        /// Include unchanged envelopes (full state instead of just diff).
        #[arg(long = "include-unchanged")]
        include_unchanged: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Emit a typing indicator on chat-arc (T-1550): thin wrapper over
    /// `channel typing-emit agent-chat-arc`. Posts a `msg_type=typing`
    /// envelope with `metadata.expires_at_ms=now+ttl`. Operator workflow:
    /// signal "I'm composing right now" so peers reading `agent typers`
    /// see live composition activity. Read companion: T-1551 `agent typers`.
    Typing {
        /// TTL in milliseconds (how long the typing indicator stays active).
        #[arg(long = "ttl-ms", default_value = "5000")]
        ttl_ms: u64,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// List active typers on chat-arc (T-1551): thin wrapper over
    /// `channel typing-list agent-chat-arc`. Walks the topic, applies
    /// `compute_active_typers` (latest typing envelope per sender, filtered
    /// by `expires_at_ms > now`), and returns one row per active typer.
    /// Write companion: T-1550 `agent typing`. T-1557: `--watch` flips on
    /// a live dashboard refreshing every `--watch-interval` seconds (TTL
    /// is 5s by default, so watching gives a moving picture of who's
    /// composing right now).
    Typers {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,

        /// Live dashboard mode: re-fetch and re-render every
        /// `--watch-interval` seconds. Incompatible with `--json`.
        #[arg(long)]
        watch: bool,

        /// Refresh interval in seconds when `--watch` is set. Clamped to
        /// [1, 60]. Default 1s — typing TTL is 5s by default, so a
        /// 1-second tick gives a smooth-enough view of who's composing.
        #[arg(long, default_value = "1")]
        watch_interval: u64,
    },

    /// List my DM topics with peer + optional unread counts (T-1552):
    /// thin wrapper over `channel dm-list`. Filters all hub topics through
    /// `dm_list_filter` (matching `dm:<a>:<b>` where my fingerprint is
    /// either side) and surfaces peer FP per topic. Pass `--unread` to
    /// add unread deltas. Personal-DM directory companion to `agent
    /// contact` (write side). NOT chat-arc-pinned: this surface is
    /// per-identity, not topic-fixed. T-1559: `--watch` flips on a live
    /// DM-only monitor (companion to `agent inbox --watch` which spans all
    /// topics).
    Dms {
        /// Include unread counts per DM topic.
        #[arg(long)]
        unread: bool,

        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,

        /// Live monitor mode: re-fetch and re-render every
        /// `--watch-interval` seconds. Incompatible with `--json`.
        #[arg(long)]
        watch: bool,

        /// Refresh interval in seconds when `--watch` is set. Clamped to
        /// [1, 300]. Default 5s (matches `agent inbox --watch`).
        #[arg(long, default_value = "5")]
        watch_interval: u64,
    },

    /// Cross-topic unread digest for the local identity (T-1553): thin
    /// wrapper over `channel inbox`. Walks the local cursor store
    /// (`subscribe --resume` recorded), joins with hub-side topic counts,
    /// and reports unread per tracked topic. Operator's first command for
    /// "what needs my attention now". Companion to T-1512 `agent unread`
    /// (chat-arc only) and T-1552 `agent dms` (DM directory). T-1558:
    /// `--watch` flips on a live monitor refreshing every
    /// `--watch-interval` seconds — leave a terminal open and watch new
    /// mail surface as it arrives.
    Inbox {
        /// Override hub address (default: local hub).
        #[arg(long)]
        hub: Option<String>,

        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,

        /// Live monitor mode: re-fetch and re-render every
        /// `--watch-interval` seconds. Incompatible with `--json`.
        #[arg(long)]
        watch: bool,

        /// Refresh interval in seconds when `--watch` is set. Clamped to
        /// [1, 300]. Default 5s — inbox unread counts are not as time-
        /// sensitive as typing presence (T-1557 uses 1s).
        #[arg(long, default_value = "5")]
        watch_interval: u64,
    },

    /// Show the local ed25519 identity (T-1554): thin wrapper over
    /// `identity show`. Surfaces fingerprint + key path. Answer to
    /// "who am I posting as?" without leaving the `agent.*` namespace.
    /// Companion to `agent who` (peer observability) which targets a
    /// remote FP — `agent identity` targets self.
    Identity {
        /// Output result as JSON envelope.
        #[arg(long)]
        json: bool,
    },

    /// Categorized verb index for the agent.* namespace (T-1556).
    /// `agent --help` lists verbs flat-alphabetical (clap default), which
    /// scales poorly past ~30 verbs. `agent verbs` groups the 60+ verbs by
    /// purpose: READING / WRITING / PRESENCE / STATS / POLLS / SNAPSHOTS /
    /// PERSONAL / META — operator's directory of the chat-arc surface.
    /// (`help` is reserved by clap as the built-in help command.)
    Verbs,

    /// T-2045 (T-2020 GO): Hub-derived idle-agent roster.
    /// Walks `agent-presence`, dedups by `agent_id` keeping latest heartbeat,
    /// filters to LIVE (heartbeat newer than 60s), excludes every agent_id
    /// currently holding any active claim, sorts by freshness, applies limit.
    /// Pure read — no state mutation. Orchestrator's "who can I dispatch to?"
    /// primitive; pairs with `channel.claim` for the next-step assign verb.
    FindIdle {
        /// Filter to agents whose `metadata.role` equals this value (e.g. `claude-code`).
        #[arg(long)]
        role: Option<String>,

        /// Require the agent to advertise this capability tag (repeat for AND).
        /// Capabilities are comma-separated in `metadata.capabilities`; missing
        /// = empty set (backward-compat with workers that don't emit the field).
        #[arg(long = "capability", value_name = "TAG")]
        capabilities: Vec<String>,

        /// Cap result count; default unlimited.
        #[arg(long)]
        limit: Option<u32>,

        /// Output as JSON (default: human-readable one-line-per-agent table).
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
        /// Session IDs or display names (omit for all sessions). Mutually
        /// exclusive with `--hub`.
        #[arg(value_name = "TARGET", conflicts_with = "hub")]
        targets: Vec<String>,

        /// Subscribe to the hub-level event aggregator instead of enumerating
        /// per-session buses (T-1645). Surfaces events emitted via
        /// `aggregator().inject()` with `session_id: "hub"` — notably
        /// `inbox.queued` from `channel.post inbox:<id>`. Mutually exclusive
        /// with positional `<TARGET>` args. Aggregator is a real-time
        /// broadcast channel (no `--since` cursor); `--timeout` bounds the
        /// collection window.
        #[arg(long, conflicts_with = "targets")]
        hub: bool,

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

#[cfg(test)]
mod cli_tests {
    use super::*;
    use clap::Parser;

    /// T-1645: `event watch --hub` parses to Watch{ hub: true, targets: empty }.
    #[test]
    fn event_watch_hub_flag_parses() {
        let cli = Cli::try_parse_from([
            "termlink", "event", "watch", "--hub", "--topic", "inbox.queued",
        ])
        .expect("parse");
        match cli.command {
            Command::Event(EventCommand::Watch { hub, targets, topic, .. }) => {
                assert!(hub, "--hub should be true");
                assert!(targets.is_empty(), "no positional targets when --hub");
                assert_eq!(topic.as_deref(), Some("inbox.queued"));
            }
            _ => panic!("expected Event::Watch"),
        }
    }

    /// T-1645: `event watch --hub <target>` is rejected (conflicts_with).
    #[test]
    fn event_watch_hub_conflicts_with_positional_target() {
        let err = Cli::try_parse_from([
            "termlink", "event", "watch", "--hub", "some-session",
        ])
        .err()
        .expect("should reject --hub + positional target");
        let msg = err.to_string();
        assert!(
            msg.contains("cannot be used with") || msg.contains("conflict"),
            "expected conflict error, got: {msg}"
        );
    }

    /// T-1645: `event watch` without --hub still accepts positional targets.
    #[test]
    fn event_watch_without_hub_accepts_targets() {
        let cli = Cli::try_parse_from([
            "termlink", "event", "watch", "alpha", "beta",
        ])
        .expect("parse");
        match cli.command {
            Command::Event(EventCommand::Watch { hub, targets, .. }) => {
                assert!(!hub);
                assert_eq!(targets, vec!["alpha".to_string(), "beta".to_string()]);
            }
            _ => panic!("expected Event::Watch"),
        }
    }
}
