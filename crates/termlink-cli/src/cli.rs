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

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
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
        /// If omitted, auto-resolves to the topic's current latest offset.
        #[arg(long)]
        up_to: Option<u64>,

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
    /// Post a reaction (Matrix `m.annotation` analogue) — shorthand for
    /// `channel post --msg-type reaction --reply-to <parent>` (T-1314)
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

        /// Target hub address (unix path or host:port). Default: local hub.
        #[arg(long)]
        hub: Option<String>,

        /// Output as JSON
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
