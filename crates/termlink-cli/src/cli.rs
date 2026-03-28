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

        /// Only print the session count (useful for scripting)
        #[arg(long)]
        count: bool,
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
    },

    /// Query a session's status
    Status {
        /// Session ID or display name (omit to pick interactively)
        target: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
    },

    /// Show TermLink runtime information and system status
    Info {
        /// Output as JSON
        #[arg(long)]
        json: bool,
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
    },

    /// List event topics from one or all sessions
    #[command(hide = true)]
    Topics {
        target: Option<String>,
        #[arg(long)]
        json: bool,
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

        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,
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
    },

    /// Manage key-value metadata on a session
    Kv {
        /// Session ID or display name
        target: String,

        /// Output result as JSON
        #[arg(long)]
        json: bool,

        /// Timeout in seconds (default: 5)
        #[arg(long, default_value = "5")]
        timeout: u64,

        #[command(subcommand)]
        action: KvAction,
    },

    // === Execution ===

    /// Run a command in an ephemeral session (register, execute, deregister)
    Run {
        /// Display name for the ephemeral session
        #[arg(short, long)]
        name: Option<String>,

        /// Tags for the session (comma-separated)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

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

        /// Tags for workers (comma-separated; dispatch metadata tags added automatically)
        #[arg(short, long, value_delimiter = ',')]
        tags: Vec<String>,

        /// Spawn backend: auto, terminal, tmux, background
        #[arg(long, default_value = "auto")]
        backend: SpawnBackend,

        /// Output results as JSON
        #[arg(long)]
        json: bool,

        /// Command for each worker to run (after --)
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
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

    /// Check TermLink runtime health — validates dirs, sessions, hub, sockets
    Doctor {
        /// Output as JSON
        #[arg(long)]
        json: bool,

        /// Auto-fix issues (clean stale sessions, remove orphaned sockets/pidfiles)
        #[arg(long)]
        fix: bool,
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
    },
    /// Stop a running hub server
    Stop,
    /// Show hub server status
    Status {
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

        /// Output result as JSON
        #[arg(long)]
        json: bool,
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
    },

    /// List saved hub profiles
    List,

    /// Remove a hub profile
    Remove {
        /// Profile name to remove
        name: String,
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
    },

    /// Receive a file from a session (waits for file.init event)
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
    },

    /// List event topics from one or all sessions
    Topics {
        /// Session ID or display name (omit for all sessions)
        target: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
