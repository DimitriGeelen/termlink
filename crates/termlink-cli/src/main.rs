use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{CommandFactory, Parser, Subcommand};
use tokio::sync::RwLock;

use termlink_session::client;
use termlink_session::codec::{FrameReader, FrameWriter};
use termlink_session::data_server;
use termlink_session::handler::SessionContext;
use termlink_session::manager;
use termlink_session::pty::PtySession;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

use termlink_protocol::data::{FrameFlags, FrameType};
use termlink_protocol::events::{
    agent_topic, file_topic, AgentRequest, AgentResponse, AgentStatus,
    FileInit, FileChunk, FileComplete, SCHEMA_VERSION,
};

#[derive(Parser)]
#[command(
    name = "termlink",
    about = "Cross-terminal session communication",
    version
)]
struct Cli {
    #[command(subcommand)]
    command: Command,
}

#[derive(Subcommand)]
enum Command {
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
        #[arg(long)]
        shell: bool,

        /// Enable token-based authentication (generates a random secret)
        #[arg(long)]
        token_secret: bool,

        /// Restrict command.execute to commands matching these prefixes (comma-separated)
        #[arg(long, value_delimiter = ',')]
        allowed_commands: Vec<String>,
    },

    /// List all registered sessions
    List {
        /// Include stale/dead sessions
        #[arg(long)]
        all: bool,

        /// Output as JSON
        #[arg(long)]
        json: bool,
    },

    /// Ping a session to verify it's alive
    Ping {
        /// Session ID or display name
        target: String,
    },

    /// Query a session's status
    Status {
        /// Session ID or display name
        target: String,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
    },

    /// Send a signal to a session's process (e.g., SIGTERM, SIGINT)
    Signal {
        /// Session ID or display name
        target: String,

        /// Signal name or number (e.g., TERM, INT, KILL, HUP, 15)
        signal: String,
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
        /// Session ID or display name
        target: String,
        #[arg(short, long, default_value = "50")]
        lines: u64,
        #[arg(short, long)]
        bytes: Option<u64>,
        /// Strip ANSI escape sequences from output
        #[arg(long)]
        strip_ansi: bool,
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
    },

    /// Attach to a PTY session
    #[command(hide = true)]
    Attach {
        /// Session ID or display name
        target: String,
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
    },

    /// Stream a PTY session via data plane
    #[command(hide = true)]
    Stream {
        /// Session ID or display name
        target: String,
    },

    // === Hidden backward-compat aliases for Event commands ===

    /// Poll events from a session's event bus
    #[command(hide = true)]
    Events {
        /// Session ID or display name
        target: String,
        #[arg(long)]
        since: Option<u64>,
        #[arg(long)]
        topic: Option<String>,
        #[arg(long)]
        json: bool,
    },

    /// Broadcast an event to multiple sessions via the hub
    #[command(hide = true)]
    Broadcast {
        topic: String,
        #[arg(short, long, default_value = "{}")]
        payload: String,
        #[arg(long, value_delimiter = ',')]
        targets: Vec<String>,
    },

    /// Emit an event to a session's event bus
    #[command(hide = true)]
    Emit {
        /// Session ID or display name
        target: String,
        topic: String,
        #[arg(short, long, default_value = "{}")]
        payload: String,
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
    },

    /// Wait for a session to emit an event matching a topic, then exit
    #[command(hide = true)]
    Wait {
        target: String,
        #[arg(long)]
        topic: String,
        #[arg(long, default_value = "0")]
        timeout: u64,
        #[arg(long, default_value = "250")]
        interval: u64,
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

        /// Command to run in the spawned terminal (after --)
        #[arg(trailing_var_arg = true)]
        command: Vec<String>,
    },

    // === Infrastructure ===

    /// Remove stale (dead) session registrations from the runtime directory
    Clean {
        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,
    },

    /// Hub server management (routes requests between sessions)
    Hub {
        #[command(subcommand)]
        action: Option<HubAction>,
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

    /// Generate shell completions
    Completions {
        /// Shell to generate completions for
        #[arg(value_enum)]
        shell: clap_complete::Shell,
    },
}

/// Spawn backend for creating new terminal sessions
#[derive(Clone, Debug, clap::ValueEnum)]
enum SpawnBackend {
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
enum HubAction {
    /// Start the hub server (default if no subcommand given)
    Start {
        /// Optional TCP address to listen on (e.g., "0.0.0.0:9100", "127.0.0.1:9100")
        #[arg(long)]
        tcp: Option<String>,
    },
    /// Stop a running hub server
    Stop,
    /// Show hub server status
    Status,
}

/// Remote hub operations (cross-machine)
#[derive(Subcommand)]
enum RemoteAction {
    /// Inject keystrokes into a session on a remote hub
    Inject {
        /// Remote hub address (e.g., 192.168.10.107:9100)
        hub: String,

        /// Target session name or ID on the remote hub
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

        /// Target session name or ID on the remote hub
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
}

/// Token management actions
#[derive(Subcommand)]
enum TokenAction {
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
    },
    /// Inspect a token without validating (decode the payload)
    Inspect {
        /// The token string to inspect
        token: String,
    },
}

/// Agent communication actions
#[derive(Subcommand)]
enum AgentAction {
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
    },
}

/// File transfer actions
#[derive(Subcommand)]
enum FileAction {
    /// Send a file to a target session
    Send {
        /// Target session ID or display name
        target: String,

        /// Path to the file to send
        path: String,

        /// Chunk size in bytes (default: 49152 = 48KB, ~64KB base64)
        #[arg(long, default_value = "49152")]
        chunk_size: usize,
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
    },
}

/// PTY terminal operations
#[derive(Subcommand)]
enum PtyCommand {
    /// Read terminal output from a PTY-backed session
    Output {
        /// Session ID or display name
        target: String,

        /// Number of lines to read (default: 50)
        #[arg(short, long, default_value = "50")]
        lines: u64,

        /// Read by bytes instead of lines
        #[arg(short, long)]
        bytes: Option<u64>,

        /// Strip ANSI escape sequences from output
        #[arg(long)]
        strip_ansi: bool,
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
    },

    /// Attach to a PTY session — live output and keyboard forwarding
    Attach {
        /// Session ID or display name
        target: String,

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
    },

    /// Stream a PTY session via data plane (real-time binary frames, zero polling)
    Stream {
        /// Session ID or display name
        target: String,
    },
}

/// Event system operations
#[derive(Subcommand)]
enum EventCommand {
    /// Poll events from a session's event bus
    Poll {
        /// Session ID or display name
        target: String,

        /// Only show events after this sequence number (omit for all)
        #[arg(long)]
        since: Option<u64>,

        /// Filter by topic
        #[arg(long)]
        topic: Option<String>,

        /// Output as JSON
        #[arg(long)]
        json: bool,
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
    },

    /// Wait for a session to emit an event matching a topic, then exit
    Wait {
        /// Session ID or display name
        target: String,

        /// Event topic to wait for (required)
        #[arg(long)]
        topic: String,

        /// Timeout in seconds (0 = wait forever, default: 0)
        #[arg(long, default_value = "0")]
        timeout: u64,

        /// Poll interval in milliseconds (default: 250)
        #[arg(long, default_value = "250")]
        interval: u64,
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
    },
}

#[derive(Subcommand)]
enum KvAction {
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

#[tokio::main]
async fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "termlink=info".into()),
        )
        .init();

    let cli = Cli::parse();

    match cli.command {
        // Session management
        Command::Register { name, roles, tags, shell, token_secret, allowed_commands } => {
            cmd_register(name, roles, tags, shell, token_secret, allowed_commands).await
        }
        Command::List { all, json } => cmd_list(all, json),
        Command::Ping { target } => cmd_ping(&target).await,
        Command::Status { target, json } => cmd_status(&target, json).await,
        Command::Info { json } => cmd_info(json),
        Command::Send { target, method, params } => cmd_send(&target, &method, &params).await,
        Command::Interact { target, command, timeout, poll_ms, strip_ansi, json } => {
            cmd_interact(&target, &command, timeout, poll_ms, strip_ansi, json).await
        }
        Command::Exec { target, command, cwd, timeout } => {
            cmd_exec(&target, &command, cwd.as_deref(), timeout).await
        }
        Command::Signal { target, signal } => cmd_signal(&target, &signal).await,

        // PTY subcommand group
        Command::Pty(pty) => match pty {
            PtyCommand::Output { target, lines, bytes, strip_ansi } => cmd_output(&target, lines, bytes, strip_ansi).await,
            PtyCommand::Inject { target, text, enter, key } => {
                cmd_inject(&target, &text, enter, key.as_deref()).await
            }
            PtyCommand::Attach { target, poll_ms } => cmd_attach(&target, poll_ms).await,
            PtyCommand::Resize { target, cols, rows } => cmd_resize(&target, cols, rows).await,
            PtyCommand::Stream { target } => cmd_stream(&target).await,
        },

        // Event subcommand group
        Command::Event(ev) => match ev {
            EventCommand::Poll { target, since, topic, json: _ } => {
                cmd_events(&target, since, topic.as_deref()).await
            }
            EventCommand::Watch { targets, interval, topic } => {
                cmd_watch(targets, interval, topic.as_deref()).await
            }
            EventCommand::Emit { target, topic, payload } => {
                cmd_emit(&target, &topic, &payload).await
            }
            EventCommand::Broadcast { topic, payload, targets } => {
                cmd_broadcast(&topic, &payload, targets).await
            }
            EventCommand::Wait { target, topic, timeout, interval } => {
                cmd_wait(&target, &topic, timeout, interval).await
            }
            EventCommand::Topics { target, json: _ } => cmd_topics(target.as_deref()).await,
            EventCommand::Collect { targets, topic, interval, count } => {
                cmd_collect(targets, topic.as_deref(), interval, count).await
            }
        },

        // Hidden backward-compat aliases (PTY)
        Command::Output { target, lines, bytes, strip_ansi } => cmd_output(&target, lines, bytes, strip_ansi).await,
        Command::Inject { target, text, enter, key } => {
            cmd_inject(&target, &text, enter, key.as_deref()).await
        }
        Command::Attach { target, poll_ms } => cmd_attach(&target, poll_ms).await,
        Command::Resize { target, cols, rows } => cmd_resize(&target, cols, rows).await,
        Command::Stream { target } => cmd_stream(&target).await,

        // Hidden backward-compat aliases (Event)
        Command::Events { target, since, topic, json: _ } => {
            cmd_events(&target, since, topic.as_deref()).await
        }
        Command::Broadcast { topic, payload, targets } => {
            cmd_broadcast(&topic, &payload, targets).await
        }
        Command::Emit { target, topic, payload } => {
            cmd_emit(&target, &topic, &payload).await
        }
        Command::Watch { targets, interval, topic } => {
            cmd_watch(targets, interval, topic.as_deref()).await
        }
        Command::Topics { target, json: _ } => cmd_topics(target.as_deref()).await,
        Command::Collect { targets, topic, interval, count } => {
            cmd_collect(targets, topic.as_deref(), interval, count).await
        }
        Command::Wait { target, topic, timeout, interval } => {
            cmd_wait(&target, &topic, timeout, interval).await
        }

        // Metadata & Discovery
        Command::Tag { target, set, add, remove } => {
            cmd_tag(&target, set, add, remove).await
        }
        Command::Discover { tag, role, cap, name, json } => {
            cmd_discover(tag, role, cap, name, json)
        }
        Command::Kv { target, action } => cmd_kv(&target, action).await,

        // Execution
        Command::Run { name, tags, timeout, command } => {
            cmd_run(name, tags, timeout, command).await
        }
        Command::Request { target, topic, payload, reply_topic, timeout, interval } => {
            cmd_request(&target, &topic, &payload, &reply_topic, timeout, interval).await
        }
        Command::Spawn { name, roles, tags, wait, wait_timeout, shell, backend, command } => {
            cmd_spawn(name, roles, tags, wait, wait_timeout, shell, backend, command).await
        }

        // Infrastructure
        Command::Clean { dry_run } => cmd_clean(dry_run),
        Command::Hub { action } => match action {
            None | Some(HubAction::Start { tcp: None }) => cmd_hub_start(None).await,
            Some(HubAction::Start { tcp: Some(ref addr) }) => cmd_hub_start(Some(addr)).await,
            Some(HubAction::Stop) => cmd_hub_stop(),
            Some(HubAction::Status) => cmd_hub_status(),
        },
        Command::Token { action } => match action {
            TokenAction::Create { target, scope, ttl } => {
                cmd_token_create(&target, &scope, ttl).await
            }
            TokenAction::Inspect { token } => cmd_token_inspect(&token),
        },
        Command::Agent { action } => match action {
            AgentAction::Ask { target, action, params, from, timeout, interval } => {
                cmd_agent_ask(&target, &action, &params, from.as_deref(), timeout, interval).await
            }
            AgentAction::Listen { target, timeout, interval } => {
                cmd_agent_listen(&target, timeout, interval).await
            }
        },
        Command::File { action } => match action {
            FileAction::Send { target, path, chunk_size } => {
                cmd_file_send(&target, &path, chunk_size).await
            }
            FileAction::Receive { target, output_dir, timeout, interval } => {
                cmd_file_receive(&target, &output_dir, timeout, interval).await
            }
        },
        Command::Remote { action } => match action {
            RemoteAction::Inject { hub, session, text, secret_file, secret, enter, key, delay_ms, scope, json } => {
                cmd_remote_inject(&hub, &session, &text, secret_file.as_deref(), secret.as_deref(), enter, key.as_deref(), delay_ms, &scope, json).await
            }
            RemoteAction::SendFile { hub, session, path, secret_file, secret, chunk_size, scope, json } => {
                cmd_remote_send_file(&hub, &session, &path, secret_file.as_deref(), secret.as_deref(), chunk_size, &scope, json).await
            }
        },
        Command::Completions { shell } => {
            clap_complete::generate(
                shell,
                &mut Cli::command(),
                "termlink",
                &mut std::io::stdout(),
            );
            Ok(())
        }
    }
}

async fn cmd_register(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    shell: bool,
    enable_token_secret: bool,
    allowed_commands: Vec<String>,
) -> Result<()> {
    let mut config = SessionConfig {
        display_name: name,
        roles,
        tags,
        ..Default::default()
    };

    // Add data_plane capability when shell mode is enabled
    if shell {
        config.capabilities.push("data_plane".into());
        config.capabilities.push("stream".into());
    }

    let mut session = termlink_session::Session::register(config)
        .await
        .context("Failed to register session")?;

    // Enable token-based auth if requested
    if enable_token_secret {
        let secret = termlink_session::auth::generate_secret();
        let secret_hex: String = secret.iter().map(|b| format!("{b:02x}")).collect();
        session.registration.token_secret = Some(secret_hex.clone());
        println!("Token auth enabled. Secret: {secret_hex}");
        println!("  Create tokens with: termlink token create {} --scope observe", session.id());
    }

    // Set command allowlist if specified
    if !allowed_commands.is_empty() {
        session.registration.allowed_commands = Some(allowed_commands.clone());
        println!("Command allowlist: {:?}", allowed_commands);
    }

    println!("Session registered:");
    println!("  ID:      {}", session.id());
    println!("  Name:    {}", session.display_name());
    println!("  Socket:  {}", session.registration.socket_path().display());

    // Set up session context (with or without PTY)
    let pty_session = if shell {
        // Set data_socket metadata for discoverability
        let data_path = data_server::data_socket_path(&session.registration.socket_path());
        session.registration.metadata.data_socket =
            Some(data_path.to_string_lossy().into_owned());

        let pty = PtySession::spawn(None, 1024 * 1024)
            .context("Failed to spawn PTY session")?;
        println!("  PTY:     yes (shell: {})",
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()));
        Some(Arc::new(pty))
    } else {
        println!("  PTY:     no (use --shell for bidirectional I/O)");
        None
    };

    // Persist updated registration (capabilities + metadata + auth + allowlist)
    if shell || enable_token_secret || session.registration.allowed_commands.is_some() {
        session.persist_registration()
            .context("Failed to persist updated registration")?;
    }

    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    let session_id = session.id().clone();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );

    let (registration, listener, _) = session.into_parts();
    let ctx = if let Some(ref pty) = pty_session {
        SessionContext::with_pty(registration.clone(), pty.clone())
            .with_registration_path(json_path)
    } else {
        SessionContext::new(registration.clone())
            .with_registration_path(json_path)
    };
    let shared = Arc::new(RwLock::new(ctx));

    let reg_for_cleanup = registration;

    // Compute data socket path before moving reg
    let data_socket_path = if shell {
        Some(data_server::data_socket_path(&reg_for_cleanup.socket_path()))
    } else {
        None
    };

    let shared_clone = shared.clone();

    // If PTY, create broadcast channel and run read loop with broadcasting
    let pty_handle = if let Some(ref pty) = pty_session {
        let pty_clone = pty.clone();
        if let Some(ref data_path) = data_socket_path {
            // Shell mode: broadcast PTY output to data plane clients
            let (tx, rx) = tokio::sync::broadcast::channel::<Vec<u8>>(256);
            let data_pty = pty.clone();
            let data_path = data_path.clone();
            println!("  Data:    {}", data_path.display());

            // Start data plane server
            tokio::spawn(async move {
                if let Err(e) = data_server::run(&data_path, data_pty, rx).await {
                    tracing::error!(error = %e, "Data plane server error");
                }
            });

            // PTY read loop with broadcast
            Some(tokio::spawn(async move {
                let _ = pty_clone.read_loop_with_broadcast(Some(tx)).await;
            }))
        } else {
            // No data plane — plain read loop
            Some(tokio::spawn(async move {
                let _ = pty_clone.read_loop().await;
            }))
        }
    } else {
        None
    };

    tokio::select! {
        _ = server::run_accept_loop(listener, shared_clone) => {}
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("Shutting down...");

            // Kill PTY child if running
            if let Some(ref pty) = pty_session {
                let _ = pty.signal(libc::SIGTERM);
            }
            if let Some(h) = pty_handle {
                h.abort();
            }

            // Clean up registration files
            let json_path = termlink_session::Registration::json_path(&sessions_dir, &session_id);
            let _ = std::fs::remove_file(&reg_for_cleanup.socket_path());
            let _ = std::fs::remove_file(&json_path);

            // Clean up data socket if present
            if let Some(ref data_path) = data_socket_path {
                let _ = std::fs::remove_file(data_path);
            }

            println!("Session {} deregistered.", session_id);
        }
    }

    Ok(())
}

fn cmd_list(include_stale: bool, json: bool) -> Result<()> {
    let sessions = manager::list_sessions(include_stale)
        .context("Failed to list sessions")?;

    if json {
        let items: Vec<serde_json::Value> = sessions.iter().map(|s| {
            serde_json::json!({
                "id": s.id.as_str(),
                "display_name": s.display_name,
                "state": s.state.to_string(),
                "pid": s.pid,
                "tags": s.tags,
                "roles": s.roles,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if sessions.is_empty() {
        println!("No active sessions.");
        return Ok(());
    }

    println!(
        "{:<14} {:<16} {:<14} {:<8} TAGS",
        "ID", "NAME", "STATE", "PID"
    );
    println!("{}", "-".repeat(64));

    for session in &sessions {
        let tags = if session.tags.is_empty() {
            String::new()
        } else {
            session.tags.join(",")
        };
        println!(
            "{:<14} {:<16} {:<14} {:<8} {}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            session.pid,
            tags,
        );
    }

    println!();
    println!("{} session(s)", sessions.len());
    Ok(())
}

fn cmd_clean(dry_run: bool) -> Result<()> {
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let stale = manager::clean_stale_sessions(&sessions_dir, !dry_run)
        .context("Failed to scan for stale sessions")?;

    if stale.is_empty() {
        println!("No stale sessions found.");
        return Ok(());
    }

    let action = if dry_run { "Would remove" } else { "Removed" };

    println!(
        "{:<14} {:<16} {:<8} CREATED",
        "ID", "NAME", "PID"
    );
    println!("{}", "-".repeat(54));

    for s in &stale {
        println!(
            "{:<14} {:<16} {:<8} {}",
            &s.id[..s.id.len().min(13)],
            truncate(&s.display_name, 15),
            s.pid,
            &s.created_at[..s.created_at.len().min(19)],
        );
    }

    println!();
    println!("{} {} stale session(s).", action, stale.len());
    Ok(())
}

async fn cmd_ping(target: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(reg.socket_path(), "termlink.ping", serde_json::json!({}))
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            println!(
                "PONG from {} ({}) — state: {}",
                result["id"].as_str().unwrap_or("?"),
                result["display_name"].as_str().unwrap_or("?"),
                result["state"].as_str().unwrap_or("?"),
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Ping failed: {}", e);
        }
    }
}

async fn cmd_status(target: &str, json: bool) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(reg.socket_path(), "query.status", serde_json::json!({}))
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&result)?);
                return Ok(());
            }
            println!("Session: {}", result["id"].as_str().unwrap_or("?"));
            println!("  Name:        {}", result["display_name"].as_str().unwrap_or("?"));
            println!("  State:       {}", result["state"].as_str().unwrap_or("?"));
            println!("  PID:         {}", result["pid"]);
            println!("  Created:     {}", result["created_at"].as_str().unwrap_or("?"));
            println!("  Heartbeat:   {}", result["heartbeat_at"].as_str().unwrap_or("?"));
            if let Some(caps) = result.get("capabilities").and_then(|c| c.as_array()) {
                let cap_strs: Vec<&str> = caps.iter().filter_map(|c| c.as_str()).collect();
                println!("  Capabilities: {}", cap_strs.join(", "));
            }
            if let Some(tags) = result.get("tags").and_then(|t| t.as_array()) {
                if !tags.is_empty() {
                    let tag_strs: Vec<&str> = tags.iter().filter_map(|t| t.as_str()).collect();
                    println!("  Tags:        {}", tag_strs.join(", "));
                }
            }
            if let Some(roles) = result.get("roles").and_then(|r| r.as_array()) {
                if !roles.is_empty() {
                    let role_strs: Vec<&str> = roles.iter().filter_map(|r| r.as_str()).collect();
                    println!("  Roles:       {}", role_strs.join(", "));
                }
            }
            if let Some(mode) = result.get("terminal_mode") {
                let canonical = mode["canonical"].as_bool().unwrap_or(false);
                let echo = mode["echo"].as_bool().unwrap_or(false);
                let raw = mode["raw"].as_bool().unwrap_or(false);
                let alt_screen = mode["alternate_screen"].as_bool().unwrap_or(false);
                let mode_label = if raw {
                    "raw"
                } else if canonical && echo {
                    "canonical+echo"
                } else if canonical {
                    "canonical"
                } else {
                    "cooked"
                };
                print!("  Term Mode:   {}", mode_label);
                if alt_screen {
                    print!(" (alternate screen)");
                }
                println!();
            }
            if let Some(meta) = result.get("metadata") {
                if let Some(shell) = meta.get("shell").and_then(|s| s.as_str()) {
                    println!("  Shell:       {}", shell);
                }
                if let Some(term) = meta.get("term").and_then(|s| s.as_str()) {
                    println!("  Terminal:    {}", term);
                }
                if let Some(cwd) = meta.get("cwd").and_then(|s| s.as_str()) {
                    println!("  CWD:         {}", cwd);
                }
                if let Some(ds) = meta.get("data_socket").and_then(|s| s.as_str()) {
                    println!("  Data plane:  {}", ds);
                }
            }
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Status query failed: {}", e);
        }
    }
}

async fn cmd_exec(target: &str, command: &str, cwd: Option<&str>, timeout: u64) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = serde_json::json!({
        "command": command,
        "timeout": timeout,
    });
    if let Some(dir) = cwd {
        params["cwd"] = serde_json::json!(dir);
    }

    let resp = client::rpc_call(reg.socket_path(), "command.execute", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let exit_code = result["exit_code"].as_i64().unwrap_or(-1);
            let stdout = result["stdout"].as_str().unwrap_or("");
            let stderr = result["stderr"].as_str().unwrap_or("");

            if !stdout.is_empty() {
                print!("{stdout}");
            }
            if !stderr.is_empty() {
                eprint!("{stderr}");
            }

            if exit_code != 0 {
                std::process::exit(exit_code as i32);
            }
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Execution failed: {}", e);
        }
    }
}

/// Strip ANSI escape sequences from a string.
fn strip_ansi_codes(s: &str) -> String {
    // Match: ESC[ ... final byte (letters), ESC] ... ST, and other OSC/CSI sequences
    let mut result = String::with_capacity(s.len());
    let mut chars = s.chars().peekable();
    while let Some(c) = chars.next() {
        if c == '\x1b' {
            // ESC sequence
            match chars.peek() {
                Some('[') => {
                    // CSI sequence: ESC [ params final_byte
                    chars.next(); // consume '['
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch.is_ascii_alphabetic() || ch == 'h' || ch == 'l' || ch == 'K' || ch == 'J' || ch == 'H' {
                            break;
                        }
                    }
                }
                Some(']') => {
                    // OSC sequence: ESC ] ... BEL or ESC \ (ST)
                    chars.next(); // consume ']'
                    while let Some(&ch) = chars.peek() {
                        chars.next();
                        if ch == '\x07' {
                            break; // BEL terminates OSC
                        }
                        if ch == '\x1b' {
                            // ESC \ (ST) terminates OSC
                            if chars.peek() == Some(&'\\') {
                                chars.next();
                            }
                            break;
                        }
                    }
                }
                _ => {
                    // Unknown ESC sequence, skip next char
                    chars.next();
                }
            }
        } else if c == '\r' {
            // Skip carriage returns (terminal artifact)
            continue;
        } else {
            result.push(c);
        }
    }
    result
}

async fn cmd_interact(
    target: &str,
    command: &str,
    timeout: u64,
    poll_ms: u64,
    strip_ansi: bool,
    json_output: bool,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    // Generate unique marker per invocation
    let marker = format!(
        "___TERMLINK_DONE_{:x}_{:x}___",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .subsec_nanos()
    );

    // Capture scrollback snapshot before injection — we'll diff against this
    let pre_resp = client::rpc_call(
        reg.socket_path(),
        "query.output",
        serde_json::json!({ "bytes": 131072 }),
    )
    .await
    .context("Failed to query output (is this a PTY session?)")?;

    let pre_output = match client::unwrap_result(pre_resp) {
        Ok(r) => r["output"].as_str().unwrap_or("").to_string(),
        Err(e) => anyhow::bail!("Session has no PTY: {}", e),
    };
    let pre_len = pre_output.len();

    // Inject strategy: send command + marker echo on a SINGLE line using `;`.
    // The shell processes them sequentially: run command, then echo marker.
    // The marker appears twice in scrollback: once in the command echo (terminal
    // echoes the full input line), and once in the actual echo output.
    // We count occurrences: 1 = still running, 2 = command finished.
    let inject_line = format!("{command}; echo \"{marker} exit=$?\"");
    let keys = serde_json::json!([
        { "type": "text", "value": inject_line },
        { "type": "key", "value": "Enter" }
    ]);
    client::rpc_call(
        reg.socket_path(),
        "command.inject",
        serde_json::json!({ "keys": keys }),
    )
    .await
    .context("Failed to inject command")?;

    let start = std::time::Instant::now();
    let deadline = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(poll_ms);

    // Poll until marker appears in scrollback
    loop {
        if start.elapsed() > deadline {
            anyhow::bail!("Timeout after {}s waiting for command to complete", timeout);
        }

        tokio::time::sleep(poll_interval).await;

        // Request enough bytes to cover new output since injection
        let resp = client::rpc_call(
            reg.socket_path(),
            "query.output",
            serde_json::json!({ "bytes": 131072 }),
        )
        .await
        .context("Failed to poll output")?;

        let result = match client::unwrap_result(resp) {
            Ok(r) => r,
            Err(e) => anyhow::bail!("Output poll failed: {}", e),
        };

        let full_output = result["output"].as_str().unwrap_or("");

        // Only look at content that appeared AFTER our injection
        // Compare against pre-injection snapshot length
        let output = if full_output.len() > pre_len {
            &full_output[pre_len..]
        } else {
            // Scrollback may have wrapped — search the whole thing
            full_output
        };

        // With single-line injection, the marker appears in the command echo BUT
        // also in the actual echo output after the command finishes. The command
        // echo line contains the literal text `echo "MARKER exit=$?"` where $?
        // is not yet expanded. The actual output contains `MARKER exit=0` (with
        // a digit). We distinguish them by requiring exit= followed by a digit.
        let marker_with_exit = format!("{marker} exit=");
        let has_marker = output.contains(&marker_with_exit) && {
            // Verify at least one occurrence has exit= followed by a digit (not $)
            let mut found_digit = false;
            for line in output.lines() {
                if let Some(pos) = line.find(&marker_with_exit) {
                    let after = &line[pos + marker_with_exit.len()..];
                    if after.starts_with(|c: char| c.is_ascii_digit()) {
                        found_digit = true;
                        break;
                    }
                }
            }
            found_digit
        };
        if has_marker {
            let elapsed_ms = start.elapsed().as_millis();

            // Find the marker line and extract exit code
            let mut exit_code: Option<i32> = None;
            for line in output.lines() {
                if line.contains(&marker) {
                    if let Some(exit_str) = line.split("exit=").nth(1) {
                        exit_code = exit_str.trim().parse().ok();
                    }
                }
            }

            // Extract command output from scrollback.
            // With single-line injection, new content layout is:
            //   [cmd echo line (contains command + marker text, may be wrapped)]\n
            //   [command output lines]\n
            //   [marker output: "___TERMLINK_DONE_xxx___ exit=N"]\n
            //   [prompt]
            // We want everything between the command echo and the marker output.
            let clean_output = {
                // Skip the first line (command echo)
                let after_cmd_echo = output.find('\n')
                    .map(|pos| &output[pos + 1..])
                    .unwrap_or(output);

                // Find the marker output line and take everything before it
                if let Some(pos) = after_cmd_echo.find(&marker_with_exit) {
                    let before = &after_cmd_echo[..pos];
                    // Go back to the last newline — that's the end of real output
                    before.rfind('\n')
                        .map(|nl| &after_cmd_echo[..nl])
                        .unwrap_or("")
                        .to_string()
                } else {
                    after_cmd_echo.to_string()
                }
            };

            let final_output = if strip_ansi {
                strip_ansi_codes(&clean_output)
            } else {
                clean_output
            };

            // Trim leading/trailing whitespace
            let final_output = final_output.trim();

            if json_output {
                let json = serde_json::json!({
                    "output": final_output,
                    "exit_code": exit_code,
                    "elapsed_ms": elapsed_ms,
                    "marker_found": true,
                    "bytes_captured": output.len(),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
            } else {
                if !final_output.is_empty() {
                    println!("{final_output}");
                }
                if let Some(code) = exit_code {
                    if code != 0 {
                        std::process::exit(code);
                    }
                }
            }

            return Ok(());
        }
    }
}

async fn cmd_send(target: &str, method: &str, params_str: &str) -> Result<()> {
    let params: serde_json::Value =
        serde_json::from_str(params_str).context("Invalid JSON params")?;

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(reg.socket_path(), method, params)
        .await
        .context("Failed to connect to session")?;

    match resp {
        termlink_protocol::jsonrpc::RpcResponse::Success(r) => {
            println!("{}", serde_json::to_string_pretty(&r.result)?);
        }
        termlink_protocol::jsonrpc::RpcResponse::Error(e) => {
            eprintln!("Error {}: {}", e.error.code, e.error.message);
            if let Some(data) = &e.error.data {
                eprintln!("{}", serde_json::to_string_pretty(data)?);
            }
            std::process::exit(1);
        }
    }

    Ok(())
}

fn cmd_discover(
    tags: Vec<String>,
    roles: Vec<String>,
    caps: Vec<String>,
    name: Option<String>,
    json: bool,
) -> Result<()> {
    let sessions = manager::list_sessions(false)
        .context("Failed to discover sessions")?;

    let has_filters = !tags.is_empty() || !roles.is_empty() || !caps.is_empty() || name.is_some();

    let filtered: Vec<_> = sessions
        .into_iter()
        .filter(|s| {
            // All specified tags must be present
            tags.iter().all(|t| s.tags.contains(t))
                // All specified roles must be present
                && roles.iter().all(|r| s.roles.contains(r))
                // All specified capabilities must be present
                && caps.iter().all(|c| s.capabilities.contains(c))
                // Name substring match (case-insensitive)
                && name.as_ref().map_or(true, |n| {
                    s.display_name.to_lowercase().contains(&n.to_lowercase())
                })
        })
        .collect();

    if json {
        let items: Vec<serde_json::Value> = filtered.iter().map(|s| {
            serde_json::json!({
                "id": s.id.as_str(),
                "display_name": s.display_name,
                "state": s.state.to_string(),
                "pid": s.pid,
                "tags": s.tags,
                "roles": s.roles,
                "capabilities": s.capabilities,
            })
        }).collect();
        println!("{}", serde_json::to_string_pretty(&items)?);
        return Ok(());
    }

    if filtered.is_empty() {
        if has_filters {
            println!("No sessions match the specified filters.");
        } else {
            println!("No sessions discovered.");
        }
        return Ok(());
    }

    println!(
        "{:<14} {:<16} {:<14} {:<20} {:<16} TAGS",
        "ID", "NAME", "STATE", "CAPABILITIES", "ROLES"
    );
    println!("{}", "-".repeat(90));

    for session in &filtered {
        println!(
            "{:<14} {:<16} {:<14} {:<20} {:<16} {}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            truncate(&session.capabilities.join(","), 19),
            truncate(&session.roles.join(","), 15),
            session.tags.join(","),
        );
    }

    println!();
    println!("{} session(s) discovered", filtered.len());
    Ok(())
}

async fn cmd_output(target: &str, lines: u64, bytes: Option<u64>, strip_ansi: bool) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = if let Some(b) = bytes {
        serde_json::json!({ "bytes": b })
    } else {
        serde_json::json!({ "lines": lines })
    };

    if strip_ansi {
        params["strip_ansi"] = serde_json::json!(true);
    }

    let resp = client::rpc_call(reg.socket_path(), "query.output", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let output = result["output"].as_str().unwrap_or("");
            print!("{output}");
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Output query failed: {}", e);
        }
    }
}

async fn cmd_inject(target: &str, text: &str, enter: bool, key: Option<&str>) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut keys = Vec::new();

    if let Some(key_name) = key {
        keys.push(serde_json::json!({ "type": "key", "value": key_name }));
    } else {
        keys.push(serde_json::json!({ "type": "text", "value": text }));
    }

    if enter {
        keys.push(serde_json::json!({ "type": "key", "value": "Enter" }));
    }

    let params = serde_json::json!({ "keys": keys });

    let resp = client::rpc_call(reg.socket_path(), "command.inject", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let bytes = result["bytes_len"].as_u64().unwrap_or(0);
            println!("Injected {bytes} bytes");
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Inject failed: {}", e);
        }
    }
}

async fn cmd_remote_inject(
    hub: &str,
    session: &str,
    text: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    enter: bool,
    key: Option<&str>,
    delay_ms: u64,
    scope: &str,
    json: bool,
) -> Result<()> {
    use termlink_session::auth::{self, PermissionScope};

    // --- Parse hub address ---
    let parts: Vec<&str> = hub.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid hub address '{}'. Expected format: host:port", hub);
    }
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse()
        .context(format!("Invalid port in '{}'", hub))?;

    // --- Read secret ---
    let hex_secret = if let Some(path) = secret_file {
        std::fs::read_to_string(path)
            .context(format!("Secret file not found: {}", path))?
            .trim()
            .to_string()
    } else if let Some(hex) = secret_hex {
        hex.to_string()
    } else {
        anyhow::bail!("Either --secret-file or --secret is required");
    };

    // --- Parse hex to bytes ---
    if hex_secret.len() != 64 {
        anyhow::bail!("Secret must be 64 hex characters (32 bytes), got {} characters", hex_secret.len());
    }
    let secret_bytes: Vec<u8> = (0..hex_secret.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_secret[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Secret contains invalid hex characters")?;
    let secret: auth::TokenSecret = secret_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Secret must be exactly 32 bytes"))?;

    // --- Parse scope ---
    let perm_scope = match scope {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        _ => anyhow::bail!("Invalid scope '{}'. Use: observe, interact, control, execute", scope),
    };

    // --- Generate auth token ---
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    // --- Connect to remote hub via TOFU TLS ---
    let addr = termlink_protocol::TransportAddr::Tcp { host, port };
    let mut client = client::Client::connect_addr(&addr)
        .await
        .context(format!("Cannot connect to {} — is the hub running?", hub))?;

    // --- Authenticate ---
    match client.call("hub.auth", serde_json::json!("auth"), serde_json::json!({"token": token.raw})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Authentication failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Authentication error: {}", e);
        }
    }

    // --- Build keys array ---
    let mut keys = Vec::new();
    if let Some(key_name) = key {
        keys.push(serde_json::json!({ "type": "key", "value": key_name }));
    } else {
        keys.push(serde_json::json!({ "type": "text", "value": text }));
    }
    if enter {
        keys.push(serde_json::json!({ "type": "key", "value": "Enter" }));
    }

    // --- Inject via hub routing ---
    let inject_params = serde_json::json!({
        "target": session,
        "keys": keys,
        "inject_delay_ms": delay_ms,
    });

    match client.call("command.inject", serde_json::json!("inject"), inject_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(r)) => {
            if json {
                println!("{}", serde_json::to_string_pretty(&r.result)?);
            } else {
                let bytes = r.result["bytes_len"].as_u64().unwrap_or(0);
                println!("Injected {} bytes into '{}' on {}", bytes, session, hub);
            }
            Ok(())
        }
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            if e.error.message.contains("not found") || e.error.message.contains("No route") {
                anyhow::bail!("Session '{}' not found on {}", session, hub);
            }
            anyhow::bail!("Inject failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Inject error: {}", e);
        }
    }
}

async fn cmd_remote_send_file(
    hub: &str,
    session: &str,
    path: &str,
    secret_file: Option<&str>,
    secret_hex: Option<&str>,
    chunk_size: usize,
    scope: &str,
    json: bool,
) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};
    use termlink_session::auth::{self, PermissionScope};

    // --- Read file ---
    let file_path = std::path::Path::new(path);
    let file_data = std::fs::read(file_path)
        .context(format!("Failed to read file: {}", path))?;

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let size = file_data.len() as u64;
    let chunk_sz = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total_chunks = ((file_data.len() + chunk_sz - 1) / chunk_sz) as u32;
    let transfer_id = generate_request_id().replace("req-", "xfer-");

    // Compute SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let sha256 = format!("{:x}", hasher.finalize());

    // --- Parse hub address ---
    let parts: Vec<&str> = hub.split(':').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid hub address '{}'. Expected format: host:port", hub);
    }
    let host = parts[0].to_string();
    let port: u16 = parts[1].parse()
        .context(format!("Invalid port in '{}'", hub))?;

    // --- Read secret ---
    let hex_secret = if let Some(path) = secret_file {
        std::fs::read_to_string(path)
            .context(format!("Secret file not found: {}", path))?
            .trim()
            .to_string()
    } else if let Some(hex) = secret_hex {
        hex.to_string()
    } else {
        anyhow::bail!("Either --secret-file or --secret is required");
    };

    // --- Parse hex to bytes ---
    if hex_secret.len() != 64 {
        anyhow::bail!("Secret must be 64 hex characters (32 bytes), got {} characters", hex_secret.len());
    }
    let secret_bytes: Vec<u8> = (0..hex_secret.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&hex_secret[i..i + 2], 16))
        .collect::<Result<Vec<u8>, _>>()
        .context("Secret contains invalid hex characters")?;
    let secret: auth::TokenSecret = secret_bytes.try_into()
        .map_err(|_| anyhow::anyhow!("Secret must be exactly 32 bytes"))?;

    // --- Parse scope ---
    let perm_scope = match scope {
        "observe" => PermissionScope::Observe,
        "interact" => PermissionScope::Interact,
        "control" => PermissionScope::Control,
        "execute" => PermissionScope::Execute,
        _ => anyhow::bail!("Invalid scope '{}'. Use: observe, interact, control, execute", scope),
    };

    // --- Generate auth token ---
    let token = auth::create_token(&secret, perm_scope, "", 3600);

    // --- Connect to remote hub via TOFU TLS ---
    let addr = termlink_protocol::TransportAddr::Tcp { host, port };
    let mut client = client::Client::connect_addr(&addr)
        .await
        .context(format!("Cannot connect to {} — is the hub running?", hub))?;

    // --- Authenticate ---
    match client.call("hub.auth", serde_json::json!("auth"), serde_json::json!({"token": token.raw})).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Authentication failed: {} {}", e.error.code, e.error.message);
        }
        Err(e) => {
            anyhow::bail!("Authentication error: {}", e);
        }
    }

    eprintln!(
        "Sending '{}' ({} bytes, {} chunks) to '{}' on {}",
        filename, size, total_chunks, session, hub
    );

    // --- Emit file.init via hub routing ---
    let init = FileInit {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        filename: filename.clone(),
        size,
        total_chunks,
        from: format!("remote-cli-{}", std::process::id()),
    };
    let init_payload = serde_json::to_value(&init)?;
    let emit_params = serde_json::json!({
        "target": session,
        "topic": file_topic::INIT,
        "payload": init_payload,
    });
    match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            if e.error.message.contains("not found") || e.error.message.contains("No route") {
                anyhow::bail!("Session '{}' not found on {}", session, hub);
            }
            anyhow::bail!("Failed to emit file.init: {} {}", e.error.code, e.error.message);
        }
        Err(e) => anyhow::bail!("Failed to emit file.init: {}", e),
    }

    // --- Emit chunks ---
    let encoder = base64::engine::general_purpose::STANDARD;
    for (i, chunk_data) in file_data.chunks(chunk_sz).enumerate() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: encoder.encode(chunk_data),
        };
        let chunk_payload = serde_json::to_value(&chunk)?;
        let emit_params = serde_json::json!({
            "target": session,
            "topic": file_topic::CHUNK,
            "payload": chunk_payload,
        });
        match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
            Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
            Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
                anyhow::bail!("Failed to emit chunk {}/{}: {} {}", i + 1, total_chunks, e.error.code, e.error.message);
            }
            Err(e) => anyhow::bail!("Failed to emit chunk {}/{}: {}", i + 1, total_chunks, e),
        }
        if total_chunks > 1 {
            eprint!("\r  Chunk {}/{}", i + 1, total_chunks);
        }
    }
    if total_chunks > 1 {
        eprintln!();
    }

    // --- Emit file.complete ---
    let complete = FileComplete {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        sha256: sha256.clone(),
    };
    let complete_payload = serde_json::to_value(&complete)?;
    let emit_params = serde_json::json!({
        "target": session,
        "topic": file_topic::COMPLETE,
        "payload": complete_payload,
    });
    match client.call("event.emit", serde_json::json!("emit"), emit_params).await {
        Ok(termlink_protocol::jsonrpc::RpcResponse::Success(_)) => {}
        Ok(termlink_protocol::jsonrpc::RpcResponse::Error(e)) => {
            anyhow::bail!("Failed to emit file.complete: {} {}", e.error.code, e.error.message);
        }
        Err(e) => anyhow::bail!("Failed to emit file.complete: {}", e),
    }

    if json {
        println!("{}", serde_json::json!({
            "transfer_id": transfer_id,
            "filename": filename,
            "size": size,
            "chunks": total_chunks,
            "sha256": sha256,
            "hub": hub,
            "session": session,
        }));
    } else {
        eprintln!("Transfer complete. SHA-256: {}", sha256);
        println!("Sent '{}' ({} bytes) to '{}' on {}", filename, size, session, hub);
    }

    Ok(())
}

async fn cmd_signal(target: &str, signal: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let sig_num = parse_signal(signal)
        .context(format!("Unknown signal: '{}'. Use TERM, INT, KILL, HUP, USR1, USR2, or a number.", signal))?;

    let resp = client::rpc_call(
        reg.socket_path(),
        "command.signal",
        serde_json::json!({ "signal": sig_num }),
    )
    .await
    .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            println!(
                "Signal {} sent to PID {}",
                result["signal"].as_i64().unwrap_or(sig_num as i64),
                result["pid"].as_u64().unwrap_or(0),
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Signal failed: {}", e);
        }
    }
}

fn parse_signal(s: &str) -> Option<i32> {
    // Try as number first
    if let Ok(n) = s.parse::<i32>() {
        return Some(n);
    }

    // Named signals (case-insensitive, with or without SIG prefix)
    let name = s.to_uppercase();
    let name = name.strip_prefix("SIG").unwrap_or(&name);

    match name {
        "TERM" => Some(libc::SIGTERM),
        "INT" => Some(libc::SIGINT),
        "KILL" => Some(libc::SIGKILL),
        "HUP" => Some(libc::SIGHUP),
        "USR1" => Some(libc::SIGUSR1),
        "USR2" => Some(libc::SIGUSR2),
        "STOP" => Some(libc::SIGSTOP),
        "CONT" => Some(libc::SIGCONT),
        "QUIT" => Some(libc::SIGQUIT),
        _ => None,
    }
}

async fn cmd_attach(target: &str, poll_ms: u64) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    // Verify the session has PTY output
    let resp = client::rpc_call(reg.socket_path(), "query.output", serde_json::json!({ "lines": 0 }))
        .await
        .context("Failed to connect to session")?;
    if let Err(e) = client::unwrap_result(resp) {
        anyhow::bail!("{}", e);
    }

    eprintln!("Attached to {} ({}). Press Ctrl+] to detach.",
        reg.display_name, reg.id);
    eprintln!();

    // Put terminal in raw mode
    let stdin_fd = libc::STDIN_FILENO;
    let orig_termios = unsafe {
        let mut t = std::mem::zeroed::<libc::termios>();
        if libc::tcgetattr(stdin_fd, &mut t) != 0 {
            anyhow::bail!("Failed to get terminal attributes");
        }
        t
    };

    let mut raw = orig_termios;
    unsafe { libc::cfmakeraw(&mut raw) };
    unsafe {
        if libc::tcsetattr(stdin_fd, libc::TCSANOW, &raw) != 0 {
            anyhow::bail!("Failed to set raw mode");
        }
    }

    // Restore terminal on exit
    let result = attach_loop(reg.socket_path(), poll_ms).await;

    unsafe {
        libc::tcsetattr(stdin_fd, libc::TCSANOW, &orig_termios);
    }

    eprintln!();
    eprintln!("Detached.");

    result
}

/// The main attach loop — polls output and forwards stdin.
async fn attach_loop(
    socket: &std::path::Path,
    poll_ms: u64,
) -> Result<()> {
    use tokio::io::AsyncReadExt;

    let mut last_buffered: u64 = 0;

    // Get initial output snapshot
    let resp = client::rpc_call(socket, "query.output", serde_json::json!({ "lines": 100 }))
        .await?;
    if let Ok(result) = client::unwrap_result(resp) {
        let output = result["output"].as_str().unwrap_or("");
        if !output.is_empty() {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            std::io::Write::write_all(&mut out, output.as_bytes())?;
            std::io::Write::flush(&mut out)?;
        }
        last_buffered = result["total_buffered"].as_u64().unwrap_or(0);
    }

    let mut stdin = tokio::io::stdin();
    let mut stdin_buf = [0u8; 256];
    let poll_interval = tokio::time::Duration::from_millis(poll_ms);

    loop {
        tokio::select! {
            // Read stdin and inject into session
            n = stdin.read(&mut stdin_buf) => {
                let n = n.context("stdin read error")?;
                if n == 0 {
                    break; // EOF
                }

                // Check for detach key: Ctrl+] (0x1d)
                if stdin_buf[..n].contains(&0x1d) {
                    break;
                }

                // Send as text injection
                let text = String::from_utf8_lossy(&stdin_buf[..n]);
                let keys = vec![serde_json::json!({ "type": "text", "value": text })];
                let params = serde_json::json!({ "keys": keys });

                // Fire-and-forget — don't block on response
                let _ = client::rpc_call(socket, "command.inject", params).await;
            }

            // Poll for new output
            _ = tokio::time::sleep(poll_interval) => {
                // Request more bytes than could have arrived since last poll
                let resp = client::rpc_call(
                    socket,
                    "query.output",
                    serde_json::json!({ "bytes": 8192 }),
                ).await;

                match resp {
                    Ok(resp) => {
                        if let Ok(result) = client::unwrap_result(resp) {
                            let new_buffered = result["total_buffered"].as_u64().unwrap_or(0);

                            if new_buffered > last_buffered {
                                // New data arrived — show the delta
                                let delta = (new_buffered - last_buffered) as usize;
                                let output = result["output"].as_str().unwrap_or("");
                                let output_bytes = output.as_bytes();

                                // Take the last `delta` bytes of the output
                                let start = output_bytes.len().saturating_sub(delta);
                                let new_data = &output_bytes[start..];

                                let stdout = std::io::stdout();
                                let mut out = stdout.lock();
                                std::io::Write::write_all(&mut out, new_data)?;
                                std::io::Write::flush(&mut out)?;
                            }

                            last_buffered = new_buffered;
                        }
                    }
                    Err(_) => {
                        // Connection lost
                        eprintln!("\r\nConnection lost.");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn cmd_events(target: &str, since: Option<u64>, topic: Option<&str>) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = serde_json::json!({});
    if let Some(s) = since {
        params["since"] = serde_json::json!(s);
    }
    if let Some(t) = topic {
        params["topic"] = serde_json::json!(t);
    }

    let resp = client::rpc_call(reg.socket_path(), "event.poll", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let events = result["events"].as_array().unwrap();
            if events.is_empty() {
                println!("No events (next_seq: {})", result["next_seq"]);
                return Ok(());
            }

            for event in events {
                let seq = event["seq"].as_u64().unwrap_or(0);
                let topic = event["topic"].as_str().unwrap_or("?");
                let payload = &event["payload"];
                let ts = event["timestamp"].as_u64().unwrap_or(0);

                if payload.is_null() || (payload.is_object() && payload.as_object().unwrap().is_empty()) {
                    println!("[{seq}] {topic} (t={ts})");
                } else {
                    println!("[{seq}] {topic}: {} (t={ts})", serde_json::to_string(payload)?);
                }
            }
            println!();
            println!("{} event(s), next_seq: {}", result["count"], result["next_seq"]);
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Event poll failed: {}", e);
        }
    }
}

async fn cmd_emit(target: &str, topic: &str, payload_str: &str) -> Result<()> {
    let payload: serde_json::Value =
        serde_json::from_str(payload_str).context("Invalid JSON payload")?;

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(
        reg.socket_path(),
        "event.emit",
        serde_json::json!({ "topic": topic, "payload": payload }),
    )
    .await
    .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            println!(
                "Event emitted: {} (seq: {})",
                result["topic"].as_str().unwrap_or("?"),
                result["seq"].as_u64().unwrap_or(0),
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Event emit failed: {}", e);
        }
    }
}

async fn cmd_broadcast(topic: &str, payload_str: &str, targets: Vec<String>) -> Result<()> {
    let payload: serde_json::Value =
        serde_json::from_str(payload_str).context("Invalid JSON payload")?;

    let hub_socket = termlink_hub::server::hub_socket_path();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    let mut params = serde_json::json!({
        "topic": topic,
        "payload": payload,
    });
    if !targets.is_empty() {
        params["targets"] = serde_json::json!(targets);
    }

    let resp = client::rpc_call(&hub_socket, "event.broadcast", params)
        .await
        .context("Failed to connect to hub")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let targeted = result["targeted"].as_u64().unwrap_or(0);
            let succeeded = result["succeeded"].as_u64().unwrap_or(0);
            let failed = result["failed"].as_u64().unwrap_or(0);
            println!(
                "Broadcast '{}': {}/{} succeeded{}",
                result["topic"].as_str().unwrap_or(topic),
                succeeded,
                targeted,
                if failed > 0 {
                    format!(" ({} failed)", failed)
                } else {
                    String::new()
                },
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Broadcast failed: {}", e);
        }
    }
}

async fn cmd_resize(target: &str, cols: u16, rows: u16) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(
        reg.socket_path(),
        "command.resize",
        serde_json::json!({ "cols": cols, "rows": rows }),
    )
    .await
    .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            println!(
                "Resized to {}x{}",
                result["cols"].as_u64().unwrap_or(cols as u64),
                result["rows"].as_u64().unwrap_or(rows as u64),
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Resize failed: {}", e);
        }
    }
}

async fn cmd_stream(target: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    // Connect to the data socket
    let data_socket = data_server::data_socket_path(reg.socket_path());
    if !data_socket.exists() {
        anyhow::bail!(
            "No data plane for '{}'. Start with --shell to enable data plane.",
            target
        );
    }

    // Fetch initial scrollback via control plane before entering raw mode
    let resp = client::rpc_call(reg.socket_path(), "query.output", serde_json::json!({ "lines": 100 }))
        .await
        .context("Failed to fetch initial scrollback")?;
    if let Ok(result) = client::unwrap_result(resp) {
        let output = result["output"].as_str().unwrap_or("");
        if !output.is_empty() {
            let stdout = std::io::stdout();
            let mut out = stdout.lock();
            std::io::Write::write_all(&mut out, output.as_bytes())?;
            std::io::Write::flush(&mut out)?;
        }
    }

    let stream = tokio::net::UnixStream::connect(&data_socket)
        .await
        .context("Failed to connect to data plane")?;

    eprintln!(
        "Streaming {} ({}) via data plane. Press Ctrl+] to detach.",
        reg.display_name, reg.id
    );
    eprintln!();

    // Put terminal in raw mode
    let stdin_fd = libc::STDIN_FILENO;
    let orig_termios = unsafe {
        let mut t = std::mem::zeroed::<libc::termios>();
        if libc::tcgetattr(stdin_fd, &mut t) != 0 {
            anyhow::bail!("Failed to get terminal attributes");
        }
        t
    };

    let mut raw = orig_termios;
    unsafe { libc::cfmakeraw(&mut raw) };
    unsafe {
        if libc::tcsetattr(stdin_fd, libc::TCSANOW, &raw) != 0 {
            anyhow::bail!("Failed to set raw mode");
        }
    }

    let result = stream_loop(stream).await;

    // Restore terminal
    unsafe {
        libc::tcsetattr(stdin_fd, libc::TCSANOW, &orig_termios);
    }

    eprintln!();
    eprintln!("Detached.");

    result
}

/// Get the current terminal size (cols, rows).
fn terminal_size() -> (u16, u16) {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };
    if ret == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        (ws.ws_col, ws.ws_row)
    } else {
        (80, 24) // sensible default
    }
}

/// Encode terminal dimensions as a 4-byte Resize payload (big-endian cols + rows).
fn resize_payload(cols: u16, rows: u16) -> [u8; 4] {
    let mut buf = [0u8; 4];
    buf[0..2].copy_from_slice(&cols.to_be_bytes());
    buf[2..4].copy_from_slice(&rows.to_be_bytes());
    buf
}

/// Real-time data plane streaming loop with SIGWINCH handling.
async fn stream_loop(stream: tokio::net::UnixStream) -> Result<()> {
    use tokio::io::AsyncReadExt;

    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    // Send initial terminal size as Resize frame
    let (cols, rows) = terminal_size();
    let _ = writer.write_frame(
        FrameType::Resize,
        FrameFlags::empty(),
        0,
        &resize_payload(cols, rows),
    ).await;

    // Set up SIGWINCH handler for terminal resize
    let mut sigwinch = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::window_change(),
    ).context("Failed to register SIGWINCH handler")?;

    let mut stdin = tokio::io::stdin();
    let mut stdin_buf = [0u8; 256];

    loop {
        tokio::select! {
            // Read Output frames from data plane
            frame = reader.read_frame() => {
                match frame {
                    Ok(Some(frame)) => {
                        match frame.header.frame_type {
                            FrameType::Output => {
                                let stdout = std::io::stdout();
                                let mut out = stdout.lock();
                                std::io::Write::write_all(&mut out, &frame.payload)?;
                                std::io::Write::flush(&mut out)?;
                            }
                            FrameType::Pong => {
                                // Keepalive response — ignore
                            }
                            FrameType::Close => {
                                eprintln!("\r\nSession closed connection.");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {
                        eprintln!("\r\nData plane disconnected.");
                        break;
                    }
                    Err(e) => {
                        eprintln!("\r\nData plane error: {e}");
                        break;
                    }
                }
            }

            // Read stdin and send as Input frames
            n = stdin.read(&mut stdin_buf) => {
                let n = n.context("stdin read error")?;
                if n == 0 {
                    break;
                }

                // Check for detach key: Ctrl+] (0x1d)
                if stdin_buf[..n].contains(&0x1d) {
                    // Send Close frame before detaching
                    let _ = writer.write_frame(
                        FrameType::Close,
                        FrameFlags::empty(),
                        0,
                        &[],
                    ).await;
                    break;
                }

                // Send as Input frame
                if let Err(e) = writer.write_frame(
                    FrameType::Input,
                    FrameFlags::empty(),
                    0,
                    &stdin_buf[..n],
                ).await {
                    eprintln!("\r\nData plane write error: {e}");
                    break;
                }
            }

            // Handle terminal resize (SIGWINCH)
            _ = sigwinch.recv() => {
                let (cols, rows) = terminal_size();
                let _ = writer.write_frame(
                    FrameType::Resize,
                    FrameFlags::empty(),
                    0,
                    &resize_payload(cols, rows),
                ).await;
            }
        }
    }

    Ok(())
}

async fn cmd_tag(
    target: &str,
    set: Vec<String>,
    add: Vec<String>,
    remove: Vec<String>,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = serde_json::json!({});
    if !set.is_empty() {
        params["tags"] = serde_json::json!(set);
    }
    if !add.is_empty() {
        params["add_tags"] = serde_json::json!(add);
    }
    if !remove.is_empty() {
        params["remove_tags"] = serde_json::json!(remove);
    }

    let resp = client::rpc_call(reg.socket_path(), "session.update", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let tags = result["tags"]
                .as_array()
                .map(|a| {
                    a.iter()
                        .filter_map(|t| t.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                })
                .unwrap_or_default();
            println!(
                "Updated {}: tags=[{}]",
                result["display_name"].as_str().unwrap_or(target),
                tags,
            );
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Tag update failed: {}", e);
        }
    }
}

async fn cmd_watch(
    targets: Vec<String>,
    interval_ms: u64,
    topic_filter: Option<&str>,
) -> Result<()> {
    use std::collections::HashMap;

    // Resolve targets: if empty, watch all live sessions
    let registrations = if targets.is_empty() {
        let sessions = manager::list_sessions(false)
            .context("Failed to list sessions")?;
        if sessions.is_empty() {
            anyhow::bail!("No active sessions to watch.");
        }
        sessions
            .iter()
            .filter_map(|s| manager::find_session(s.id.as_str()).ok())
            .collect::<Vec<_>>()
    } else {
        targets
            .iter()
            .map(|t| manager::find_session(t).context(format!("Session '{}' not found", t)))
            .collect::<Result<Vec<_>>>()?
    };

    if registrations.is_empty() {
        anyhow::bail!("No reachable sessions to watch.");
    }

    let session_names: HashMap<String, String> = registrations
        .iter()
        .map(|r| (r.id.as_str().to_string(), r.display_name.clone()))
        .collect();

    eprintln!(
        "Watching {} session(s): {}. Press Ctrl+C to stop.",
        registrations.len(),
        registrations
            .iter()
            .map(|r| r.display_name.as_str())
            .collect::<Vec<_>>()
            .join(", "),
    );
    eprintln!();

    // Track last seen sequence per session (start with u64::MAX sentinel = get all)
    let mut cursors: HashMap<String, Option<u64>> = registrations
        .iter()
        .map(|r| (r.id.as_str().to_string(), None))
        .collect();

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!();
                eprintln!("Stopped watching.");
                break;
            }
            _ = tokio::time::sleep(poll_interval) => {
                for reg in &registrations {
                    let sid = reg.id.as_str();
                    let name = session_names.get(sid).map(|s| s.as_str()).unwrap_or(sid);

                    let mut params = serde_json::json!({});
                    if let Some(cursor) = cursors.get(sid).and_then(|c| *c) {
                        params["since"] = serde_json::json!(cursor);
                    }
                    if let Some(t) = topic_filter {
                        params["topic"] = serde_json::json!(t);
                    }

                    let resp = match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                        Ok(r) => r,
                        Err(_) => {
                            // Session may have gone away — skip silently
                            continue;
                        }
                    };

                    if let Ok(result) = client::unwrap_result(resp) {
                        if let Some(events) = result["events"].as_array() {
                            for event in events {
                                let seq = event["seq"].as_u64().unwrap_or(0);
                                let topic = event["topic"].as_str().unwrap_or("?");
                                let payload = &event["payload"];
                                let ts = event["timestamp"].as_u64().unwrap_or(0);

                                if payload.is_null()
                                    || (payload.is_object()
                                        && payload.as_object().unwrap().is_empty())
                                {
                                    println!("[{name}#{seq}] {topic} (t={ts})");
                                } else {
                                    println!(
                                        "[{name}#{seq}] {topic}: {} (t={ts})",
                                        serde_json::to_string(payload).unwrap_or_default()
                                    );
                                }

                                // Update cursor to latest seen
                                cursors.insert(sid.to_string(), Some(seq));
                            }
                        }
                        // Also update cursor from next_seq if no events
                        if let Some(next) = result["next_seq"].as_u64() {
                            if cursors.get(sid).and_then(|c| *c).is_none() && next > 0 {
                                // First poll returned events, cursor set above.
                                // If no events, set cursor to next_seq - 1 to avoid re-fetching
                                cursors.insert(sid.to_string(), Some(next.saturating_sub(1)));
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(())
}

async fn cmd_kv(target: &str, action: KvAction) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    match action {
        KvAction::Set { key, value } => {
            // Try to parse value as JSON; if that fails, treat as string
            let json_value: serde_json::Value = serde_json::from_str(&value)
                .unwrap_or_else(|_| serde_json::Value::String(value));

            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.set",
                serde_json::json!({"key": key, "value": json_value}),
            )
            .await
            .context("Failed to connect to session")?;

            match client::unwrap_result(resp) {
                Ok(result) => {
                    let replaced = result["replaced"].as_bool().unwrap_or(false);
                    println!(
                        "{} {}={}",
                        if replaced { "Updated" } else { "Set" },
                        result["key"].as_str().unwrap_or("?"),
                        serde_json::to_string(&json_value)?,
                    );
                }
                Err(e) => anyhow::bail!("kv.set failed: {}", e),
            }
        }
        KvAction::Get { key } => {
            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.get",
                serde_json::json!({"key": key}),
            )
            .await
            .context("Failed to connect to session")?;

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if result["found"].as_bool().unwrap_or(false) {
                        println!("{}", serde_json::to_string_pretty(&result["value"])?);
                    } else {
                        eprintln!("Key '{}' not found", key);
                        std::process::exit(1);
                    }
                }
                Err(e) => anyhow::bail!("kv.get failed: {}", e),
            }
        }
        KvAction::List => {
            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.list",
                serde_json::json!({}),
            )
            .await
            .context("Failed to connect to session")?;

            match client::unwrap_result(resp) {
                Ok(result) => {
                    let entries = result["entries"].as_array();
                    if let Some(entries) = entries {
                        if entries.is_empty() {
                            println!("No key-value pairs.");
                        } else {
                            for entry in entries {
                                let key = entry["key"].as_str().unwrap_or("?");
                                let value = &entry["value"];
                                println!("{}={}", key, serde_json::to_string(value)?);
                            }
                            println!();
                            println!("{} pair(s)", result["count"]);
                        }
                    }
                }
                Err(e) => anyhow::bail!("kv.list failed: {}", e),
            }
        }
        KvAction::Del { key } => {
            let resp = client::rpc_call(
                reg.socket_path(),
                "kv.delete",
                serde_json::json!({"key": key}),
            )
            .await
            .context("Failed to connect to session")?;

            match client::unwrap_result(resp) {
                Ok(result) => {
                    if result["deleted"].as_bool().unwrap_or(false) {
                        println!("Deleted '{}'", key);
                    } else {
                        eprintln!("Key '{}' not found", key);
                        std::process::exit(1);
                    }
                }
                Err(e) => anyhow::bail!("kv.delete failed: {}", e),
            }
        }
    }

    Ok(())
}

fn cmd_info(json: bool) -> Result<()> {
    let runtime_dir = termlink_session::discovery::runtime_dir();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let hub_socket = termlink_hub::server::hub_socket_path();
    let hub_running = hub_socket.exists();
    let live = manager::list_sessions(false)
        .map(|s| s.len())
        .unwrap_or(0);
    let all = manager::list_sessions(true)
        .map(|s| s.len())
        .unwrap_or(0);
    let stale = all - live;

    if json {
        println!("{}", serde_json::to_string_pretty(&serde_json::json!({
            "runtime_dir": runtime_dir.to_string_lossy(),
            "sessions_dir": sessions_dir.to_string_lossy(),
            "hub_socket": hub_socket.to_string_lossy(),
            "hub_running": hub_running,
            "sessions": {
                "live": live,
                "stale": stale,
                "total": all,
            },
        }))?);
        return Ok(());
    }

    println!("TermLink Runtime");
    println!("{}", "-".repeat(40));
    println!("  Runtime dir:  {}", runtime_dir.display());
    println!("  Sessions dir: {}", sessions_dir.display());
    println!("  Hub socket:   {}", hub_socket.display());
    println!(
        "  Hub:          {}",
        if hub_running { "running" } else { "stopped" }
    );

    println!();
    println!("Sessions");
    println!("{}", "-".repeat(40));
    println!("  Live:   {}", live);
    println!("  Stale:  {}", stale);
    println!("  Total:  {}", all);

    if stale > 0 {
        println!();
        println!("  Tip: run 'termlink clean' to remove stale sessions");
    }

    Ok(())
}

async fn cmd_topics(target: Option<&str>) -> Result<()> {
    use std::collections::BTreeMap;

    let registrations = if let Some(t) = target {
        vec![manager::find_session(t).context(format!("Session '{}' not found", t))?]
    } else {
        manager::list_sessions(false).context("Failed to list sessions")?
    };

    if registrations.is_empty() {
        println!("No active sessions.");
        return Ok(());
    }

    let mut session_topics: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for reg in &registrations {
        match client::rpc_call(reg.socket_path(), "event.topics", serde_json::json!({})).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(topics) = result["topics"].as_array() {
                        let topic_list: Vec<String> = topics
                            .iter()
                            .filter_map(|t| t.as_str().map(String::from))
                            .collect();
                        if !topic_list.is_empty() {
                            session_topics
                                .insert(reg.display_name.clone(), topic_list);
                        }
                    }
                }
            }
            Err(_) => continue,
        }
    }

    if session_topics.is_empty() {
        println!("No event topics found.");
        return Ok(());
    }

    for (name, topics) in &session_topics {
        println!("{}:", name);
        for topic in topics {
            println!("  {}", topic);
        }
    }

    let total: usize = session_topics.values().map(|v| v.len()).sum();
    println!();
    println!(
        "{} topic(s) across {} session(s)",
        total,
        session_topics.len()
    );
    Ok(())
}

async fn cmd_collect(
    targets: Vec<String>,
    topic_filter: Option<&str>,
    interval_ms: u64,
    max_count: u64,
) -> Result<()> {
    let hub_socket = termlink_hub::server::hub_socket_path();
    if !hub_socket.exists() {
        anyhow::bail!("Hub is not running. Start it with: termlink hub");
    }

    eprintln!("Collecting events via hub. Press Ctrl+C to stop.");
    if let Some(t) = topic_filter {
        eprintln!("  Topic filter: {}", t);
    }
    if !targets.is_empty() {
        eprintln!("  Targets: {}", targets.join(", "));
    }
    eprintln!();

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);
    let mut cursors = serde_json::json!({});
    let mut total_received: u64 = 0;

    loop {
        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                eprintln!();
                eprintln!("Stopped. {} event(s) collected.", total_received);
                break;
            }
            _ = tokio::time::sleep(poll_interval) => {
                let mut params = serde_json::json!({});
                if !targets.is_empty() {
                    params["targets"] = serde_json::json!(targets);
                }
                if let Some(t) = topic_filter {
                    params["topic"] = serde_json::json!(t);
                }
                if !cursors.as_object().unwrap_or(&serde_json::Map::new()).is_empty() {
                    params["since"] = cursors.clone();
                }

                let resp = match client::rpc_call(&hub_socket, "event.collect", params).await {
                    Ok(r) => r,
                    Err(e) => {
                        eprintln!("Hub connection error: {}. Retrying...", e);
                        continue;
                    }
                };

                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let session_name = event["session_name"].as_str().unwrap_or("?");
                            let seq = event["seq"].as_u64().unwrap_or(0);
                            let topic = event["topic"].as_str().unwrap_or("?");
                            let payload = &event["payload"];
                            let ts = event["timestamp"].as_u64().unwrap_or(0);

                            if payload.is_null()
                                || (payload.is_object()
                                    && payload.as_object().unwrap().is_empty())
                            {
                                println!("[{session_name}#{seq}] {topic} (t={ts})");
                            } else {
                                println!(
                                    "[{session_name}#{seq}] {topic}: {} (t={ts})",
                                    serde_json::to_string(payload).unwrap_or_default()
                                );
                            }

                            total_received += 1;
                        }
                    }

                    // Update cursors from response
                    if let Some(new_cursors) = result.get("cursors") {
                        if let Some(obj) = new_cursors.as_object() {
                            for (k, v) in obj {
                                cursors[k] = v.clone();
                            }
                        }
                    }

                    // Check count limit
                    if max_count > 0 && total_received >= max_count {
                        eprintln!();
                        eprintln!("{} event(s) collected (limit reached).", total_received);
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

async fn cmd_run(
    name: Option<String>,
    tags: Vec<String>,
    timeout: u64,
    command_parts: Vec<String>,
) -> Result<()> {
    use termlink_session::executor;

    let command_str = command_parts
        .iter()
        .map(|part| {
            if part.contains(' ') || part.contains('"') || part.contains('\'') || part.contains('\\') || part.contains('$') || part.contains('`') {
                format!("'{}'", part.replace('\'', "'\\''"))
            } else {
                part.clone()
            }
        })
        .collect::<Vec<_>>()
        .join(" ");

    let config = SessionConfig {
        display_name: name,
        tags,
        ..Default::default()
    };

    let session = termlink_session::Session::register(config)
        .await
        .context("Failed to register ephemeral session")?;

    let session_id = session.id().clone();
    let sessions_dir = termlink_session::discovery::sessions_dir();

    eprintln!("Session {} ({}) registered", session.id(), session.display_name());
    eprintln!("Running: {}", command_str);

    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );
    let (registration, listener, _) = session.into_parts();
    let ctx = SessionContext::new(registration.clone())
        .with_registration_path(json_path);
    let shared = Arc::new(RwLock::new(ctx));
    let shared_clone = shared.clone();

    let reg_for_cleanup = registration;

    // Run RPC listener in background so the session is queryable during execution
    let rpc_handle = tokio::spawn(async move {
        server::run_accept_loop(listener, shared_clone).await;
    });

    // Execute the command (CLI-initiated, no allowlist restriction)
    let result = executor::execute(
        &command_str,
        None,
        None,
        Some(std::time::Duration::from_secs(timeout)),
        None,
    )
    .await;

    // Abort RPC listener
    rpc_handle.abort();

    // Cleanup: deregister session
    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );
    let _ = std::fs::remove_file(&reg_for_cleanup.socket_path());
    let _ = std::fs::remove_file(&json_path);
    eprintln!("Session {} deregistered", session_id);

    match result {
        Ok(exec_result) => {
            if !exec_result.stdout.is_empty() {
                print!("{}", exec_result.stdout);
            }
            if !exec_result.stderr.is_empty() {
                eprint!("{}", exec_result.stderr);
            }
            if exec_result.exit_code != 0 {
                std::process::exit(exec_result.exit_code);
            }
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Command failed: {}", e);
        }
    }
}

async fn cmd_wait(target: &str, topic: &str, timeout_secs: u64, interval_ms: u64) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    eprintln!("Waiting for event topic '{}' from {}...", topic, reg.display_name);

    let poll_interval = tokio::time::Duration::from_millis(interval_ms);
    let deadline = if timeout_secs > 0 {
        Some(tokio::time::Instant::now() + tokio::time::Duration::from_secs(timeout_secs))
    } else {
        None
    };

    // Start polling from the beginning. The first poll with topic filter
    // will catch both pre-existing events and newly emitted ones.
    // Using None means "no since filter" which returns all matching events.
    let mut cursor: Option<u64> = None;

    loop {
        if let Some(dl) = deadline {
            if tokio::time::Instant::now() >= dl {
                anyhow::bail!("Timeout waiting for event topic '{}'", topic);
            }
        }

        tokio::select! {
            _ = tokio::signal::ctrl_c() => {
                anyhow::bail!("Interrupted");
            }
            _ = tokio::time::sleep(poll_interval) => {
                let mut params = serde_json::json!({ "topic": topic });
                if let Some(c) = cursor {
                    params["since"] = serde_json::json!(c);
                }
                let resp = match client::rpc_call(reg.socket_path(), "event.poll", params).await {
                    Ok(r) => r,
                    Err(_) => {
                        anyhow::bail!("Session '{}' disconnected while waiting", target);
                    }
                };

                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        if let Some(event) = events.first() {
                            // Found matching event — print payload and exit
                            let payload = &event["payload"];
                            if payload.is_null()
                                || (payload.is_object()
                                    && payload.as_object().unwrap().is_empty())
                            {
                                println!("{}", topic);
                            } else {
                                println!("{}", serde_json::to_string(payload)?);
                            }
                            return Ok(());
                        }
                    }
                    if let Some(next) = result["next_seq"].as_u64() {
                        cursor = if next > 0 { Some(next - 1) } else { None };
                    }
                }
            }
        }
    }
}

async fn cmd_request(
    target: &str,
    topic: &str,
    payload: &str,
    reply_topic: &str,
    timeout: u64,
    interval: u64,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    // Generate a request ID for correlation
    let request_id = format!("req-{}-{}", std::process::id(), std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis());

    // Parse user payload and inject request_id
    let mut payload_json: serde_json::Value = serde_json::from_str(payload)
        .context("Invalid JSON payload")?;
    if let Some(obj) = payload_json.as_object_mut() {
        obj.insert("request_id".to_string(), serde_json::json!(request_id));
    }

    // Snapshot the current next_seq BEFORE emitting — we'll poll for replies after this point
    let cursor: Option<u64> = {
        let params = serde_json::json!({});
        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    result["next_seq"].as_u64()
                } else { None }
            }
            Err(_) => None,
        }
    };

    // Emit the request event
    let emit_params = serde_json::json!({
        "topic": topic,
        "payload": payload_json,
    });

    let emit_resp = client::rpc_call(reg.socket_path(), "event.emit", emit_params)
        .await
        .context("Failed to emit request event")?;

    match client::unwrap_result(emit_resp) {
        Ok(result) => {
            println!("Request sent: {} (seq: {}, request_id: {})",
                topic,
                result["seq"].as_u64().unwrap_or(0),
                request_id);
        }
        Err(e) => {
            anyhow::bail!("Failed to emit request: {}", e);
        }
    }

    // Now poll for the reply topic
    println!("Waiting for reply on topic '{}' (timeout: {}s)...", reply_topic, timeout);

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(interval);
    let mut poll_cursor = cursor;

    loop {
        let mut params = serde_json::json!({ "topic": reply_topic });
        if let Some(c) = poll_cursor {
            params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            // Check if this reply matches our request_id
                            let event_payload = &event["payload"];
                            let matches = event_payload
                                .get("request_id")
                                .and_then(|r| r.as_str())
                                .map(|r| r == request_id)
                                .unwrap_or(true); // If no request_id in reply, accept it

                            if matches {
                                println!("Reply received:");
                                println!("{}", serde_json::to_string_pretty(event_payload)?);
                                return Ok(());
                            }
                        }
                    }

                    // Update cursor only when events were received
                    if let Some(events) = result["events"].as_array() {
                        if !events.is_empty() {
                            if let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if start.elapsed() > timeout_dur {
            anyhow::bail!("Timeout waiting for reply on topic '{}' ({}s)", reply_topic, timeout);
        }

        tokio::time::sleep(poll_interval).await;
    }
}

async fn cmd_spawn(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    wait: bool,
    wait_timeout: u64,
    shell: bool,
    backend: SpawnBackend,
    command: Vec<String>,
) -> Result<()> {
    let session_name = name.clone().unwrap_or_else(|| {
        format!("spawn-{}", std::process::id())
    });

    // Build the shell command string for the session
    let shell_cmd = build_spawn_shell_cmd(&session_name, &roles, &tags, shell, &command)?;

    // Resolve backend (auto-detect if needed)
    let resolved = resolve_spawn_backend(&backend);
    match resolved {
        SpawnBackend::Terminal => spawn_via_terminal(&session_name, &shell_cmd)?,
        SpawnBackend::Tmux => spawn_via_tmux(&session_name, &shell_cmd)?,
        SpawnBackend::Background => spawn_via_background(&session_name, &shell_cmd)?,
        SpawnBackend::Auto => unreachable!("resolve_spawn_backend always resolves Auto"),
    }

    println!("Spawned session '{}' via {} backend", session_name, resolved);

    // If --wait, poll for the session to appear
    if wait {
        println!("Waiting for session to register (timeout: {}s)...", wait_timeout);
        let start = std::time::Instant::now();
        let timeout = std::time::Duration::from_secs(wait_timeout);

        loop {
            if manager::find_session(&session_name).is_ok() {
                println!("Session '{}' is ready", session_name);
                return Ok(());
            }
            if start.elapsed() > timeout {
                anyhow::bail!(
                    "Timeout waiting for session '{}' to register ({}s)",
                    session_name,
                    wait_timeout
                );
            }
            tokio::time::sleep(std::time::Duration::from_millis(250)).await;
        }
    }

    Ok(())
}

/// Build the shell command string that registers a TermLink session.
fn build_spawn_shell_cmd(
    session_name: &str,
    roles: &[String],
    tags: &[String],
    shell: bool,
    command: &[String],
) -> Result<String> {
    let termlink_bin = std::env::current_exe()
        .context("Failed to determine termlink binary path")?;
    let termlink_path = termlink_bin.to_string_lossy();

    let mut register_args = vec![
        "register".to_string(),
        "--name".to_string(),
        session_name.to_string(),
    ];
    if !roles.is_empty() {
        register_args.push("--roles".to_string());
        register_args.push(roles.join(","));
    }
    if !tags.is_empty() {
        register_args.push("--tags".to_string());
        register_args.push(tags.join(","));
    }
    if shell || command.is_empty() {
        register_args.push("--shell".to_string());
    }

    let shell_cmd = if command.is_empty() {
        let mut parts = vec![termlink_path.to_string()];
        parts.extend(register_args.iter().cloned());

        if let Ok(rd) = std::env::var("TERMLINK_RUNTIME_DIR") {
            format!("TERMLINK_RUNTIME_DIR={} {}", shell_escape(&rd), parts.join(" "))
        } else {
            parts.join(" ")
        }
    } else {
        let mut reg_parts = vec![termlink_path.to_string()];
        reg_parts.extend(register_args.iter().cloned());

        let user_cmd = command.iter()
            .map(|arg| shell_escape(arg))
            .collect::<Vec<_>>()
            .join(" ");
        let env_prefix = if let Ok(rd) = std::env::var("TERMLINK_RUNTIME_DIR") {
            format!("export TERMLINK_RUNTIME_DIR={}; ", shell_escape(&rd))
        } else {
            String::new()
        };

        format!(
            "{env_prefix}{} &\nTL_PID=$!\nsleep 1\n{user_cmd}\nkill $TL_PID 2>/dev/null\nwait $TL_PID 2>/dev/null",
            reg_parts.join(" ")
        )
    };

    Ok(shell_cmd)
}

/// Resolve Auto backend to a concrete backend based on platform and environment.
fn resolve_spawn_backend(backend: &SpawnBackend) -> SpawnBackend {
    match backend {
        SpawnBackend::Auto => {
            // macOS with GUI → Terminal.app
            #[cfg(target_os = "macos")]
            {
                // Check if we have a GUI (WindowServer running)
                if std::process::Command::new("pgrep")
                    .args(["-x", "WindowServer"])
                    .stdout(std::process::Stdio::null())
                    .stderr(std::process::Stdio::null())
                    .status()
                    .map(|s| s.success())
                    .unwrap_or(false)
                {
                    return SpawnBackend::Terminal;
                }
            }

            // tmux available → use tmux
            if std::process::Command::new("tmux")
                .arg("-V")
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .status()
                .map(|s| s.success())
                .unwrap_or(false)
            {
                return SpawnBackend::Tmux;
            }

            // Fallback → background PTY
            SpawnBackend::Background
        }
        other => other.clone(),
    }
}

/// Spawn via macOS Terminal.app using osascript.
fn spawn_via_terminal(session_name: &str, shell_cmd: &str) -> Result<()> {
    let escaped_cmd = shell_cmd.replace('\\', "\\\\").replace('"', "\\\"");
    let applescript = format!(
        r#"tell application "Terminal"
    activate
    do script "{escaped_cmd}"
end tell"#
    );

    let status = std::process::Command::new("osascript")
        .arg("-e")
        .arg(&applescript)
        .status()
        .context("Failed to run osascript — is Terminal.app available?")?;

    if !status.success() {
        anyhow::bail!("Failed to open new Terminal.app window for session '{}'", session_name);
    }
    Ok(())
}

/// Spawn via tmux detached session.
fn spawn_via_tmux(session_name: &str, shell_cmd: &str) -> Result<()> {
    let tmux_session = format!("tl-{}", session_name);
    let status = std::process::Command::new("tmux")
        .args(["new-session", "-d", "-s", &tmux_session, shell_cmd])
        .status()
        .context("Failed to run tmux — is tmux installed?")?;

    if !status.success() {
        anyhow::bail!("Failed to create tmux session '{}' for TermLink session '{}'", tmux_session, session_name);
    }
    Ok(())
}

/// Spawn via background process (detached shell).
/// Works on both macOS and Linux — no terminal emulator needed.
fn spawn_via_background(session_name: &str, shell_cmd: &str) -> Result<()> {
    // Try setsid (Linux), fall back to plain sh (macOS)
    let child = std::process::Command::new("setsid")
        .args(["sh", "-c", shell_cmd])
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .stdin(std::process::Stdio::null())
        .spawn()
        .or_else(|_| {
            // setsid not available (macOS) — spawn sh directly
            std::process::Command::new("sh")
                .args(["-c", shell_cmd])
                .stdout(std::process::Stdio::null())
                .stderr(std::process::Stdio::null())
                .stdin(std::process::Stdio::null())
                .spawn()
        })
        .context("Failed to spawn background session")?;

    let _ = child; // fire-and-forget — child runs independently
    let _ = session_name;
    Ok(())
}

/// Escape a string for use in a shell command.
fn shell_escape(s: &str) -> String {
    if s.contains(|c: char| c.is_whitespace() || c == '\'' || c == '"' || c == '\\' || c == '$') {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}

async fn cmd_hub_start(tcp_addr: Option<&str>) -> Result<()> {
    let socket_path = termlink_hub::server::hub_socket_path();
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

    println!("Starting hub server...");
    println!("  Socket:  {}", socket_path.display());
    if let Some(addr) = tcp_addr {
        println!("  TCP:     {}", addr);
    }
    println!("  Pidfile: {}", pidfile_path.display());

    let handle = termlink_hub::server::run_with_tcp(&socket_path, tcp_addr)
        .await
        .context("Hub server error")?;

    if tcp_addr.is_some() {
        let secret_path = termlink_hub::server::hub_secret_path();
        let cert_path = termlink_hub::tls::hub_cert_path();
        println!("  Secret:  {}", secret_path.display());
        println!("  TLS cert: {}", cert_path.display());
        println!();
        println!("TCP connections use TLS with auto-generated self-signed certificate.");
        println!("Auth required. Clients must call 'hub.auth' with a token.");
        println!("Read the secret: cat {}", secret_path.display());
    }
    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    // Wait for Ctrl+C, then trigger graceful shutdown
    tokio::signal::ctrl_c().await.ok();
    println!();
    println!("Shutting down hub...");
    handle.shutdown();

    // Give the background task time to drain
    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
    println!("Hub stopped.");

    Ok(())
}

fn cmd_hub_stop() -> Result<()> {
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();

    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            println!("Hub is not running.");
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            println!("Hub pidfile found (PID {pid}) but process is dead. Cleaning up.");
            termlink_hub::pidfile::remove(&pidfile_path);
            let socket_path = termlink_hub::server::hub_socket_path();
            let _ = std::fs::remove_file(&socket_path);
        }
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            println!("Stopping hub (PID {pid})...");
            unsafe { libc::kill(pid as i32, libc::SIGTERM) };
            // Wait briefly for process to exit
            for _ in 0..20 {
                std::thread::sleep(std::time::Duration::from_millis(100));
                if !termlink_session::liveness::process_exists(pid) {
                    println!("Hub stopped.");
                    return Ok(());
                }
            }
            println!("Hub did not stop within 2 seconds. You may need to kill -9 {pid}.");
        }
    }
    Ok(())
}

fn cmd_hub_status() -> Result<()> {
    let pidfile_path = termlink_hub::pidfile::hub_pidfile_path();
    let socket_path = termlink_hub::server::hub_socket_path();

    match termlink_hub::pidfile::check(&pidfile_path) {
        termlink_hub::pidfile::PidfileStatus::NotRunning => {
            println!("Hub: not running");
        }
        termlink_hub::pidfile::PidfileStatus::Stale(pid) => {
            println!("Hub: stale (PID {pid} is dead, pidfile needs cleanup)");
            println!("  Run 'termlink hub stop' to clean up.");
        }
        termlink_hub::pidfile::PidfileStatus::Running(pid) => {
            println!("Hub: running (PID {pid})");
            println!("  Socket: {}", socket_path.display());
            println!("  Pidfile: {}", pidfile_path.display());
        }
    }
    Ok(())
}

async fn cmd_token_create(target: &str, scope: &str, ttl: u64) -> Result<()> {
    use termlink_session::auth;

    // Resolve target to a registration
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    // Check for token_secret
    let secret_hex = reg
        .token_secret
        .as_ref()
        .ok_or_else(|| anyhow::anyhow!(
            "Session '{}' does not have token auth enabled. Register with --token-secret.",
            target
        ))?;

    // Decode hex secret
    let secret_bytes: auth::TokenSecret = {
        if secret_hex.len() != 64 {
            anyhow::bail!("Invalid token_secret in registration (expected 64 hex chars)");
        }
        let mut bytes = [0u8; 32];
        for i in 0..32 {
            bytes[i] = u8::from_str_radix(&secret_hex[i * 2..i * 2 + 2], 16)
                .context("Invalid hex in token_secret")?;
        }
        bytes
    };

    // Parse scope
    let permission_scope = auth::parse_scope(scope)
        .map_err(|e| anyhow::anyhow!("{e}"))?;

    let token = auth::create_token(&secret_bytes, permission_scope, reg.id.as_str(), ttl);

    println!("{}", token.raw);
    eprintln!("Scope: {scope}, TTL: {ttl}s, Session: {}", reg.id);

    let _ = sessions_dir; // suppress unused
    Ok(())
}

fn cmd_token_inspect(token_str: &str) -> Result<()> {
    use base64::Engine;

    let parts: Vec<&str> = token_str.splitn(2, '.').collect();
    if parts.len() != 2 {
        anyhow::bail!("Invalid token format (expected payload.signature)");
    }

    let payload_json = base64::engine::general_purpose::URL_SAFE_NO_PAD
        .decode(parts[0])
        .context("Invalid base64 in token payload")?;

    let payload: serde_json::Value =
        serde_json::from_slice(&payload_json).context("Invalid JSON in token payload")?;

    println!("{}", serde_json::to_string_pretty(&payload)?);

    // Check expiry
    if let Some(expires) = payload["expires_at"].as_u64() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if now > expires {
            eprintln!("WARNING: Token has expired ({} seconds ago)", now - expires);
        } else {
            eprintln!("Expires in {} seconds", expires - now);
        }
    }

    Ok(())
}

/// Generate a request ID for agent protocol correlation.
fn generate_request_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("req-{}-{}", std::process::id(), ts)
}

async fn cmd_agent_ask(
    target: &str,
    action: &str,
    params_str: &str,
    from: Option<&str>,
    timeout: u64,
    interval: u64,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let request_id = generate_request_id();
    let sender = from.map(|s| s.to_string()).unwrap_or_else(|| format!("cli-{}", std::process::id()));

    let params: serde_json::Value = serde_json::from_str(params_str)
        .context("Invalid JSON in --params")?;

    let request = AgentRequest {
        schema_version: SCHEMA_VERSION.to_string(),
        request_id: request_id.clone(),
        from: sender.clone(),
        to: target.to_string(),
        action: action.to_string(),
        params,
        timeout_secs: if timeout > 0 { Some(timeout) } else { None },
    };

    // Snapshot cursor before emitting
    let cursor: Option<u64> = {
        let poll_params = serde_json::json!({});
        match client::rpc_call(reg.socket_path(), "event.poll", poll_params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    result["next_seq"].as_u64()
                } else { None }
            }
            Err(_) => None,
        }
    };

    // Emit the agent.request event
    let payload = serde_json::to_value(&request)
        .context("Failed to serialize AgentRequest")?;
    let emit_params = serde_json::json!({
        "topic": agent_topic::REQUEST,
        "payload": payload,
    });

    let emit_resp = client::rpc_call(reg.socket_path(), "event.emit", emit_params)
        .await
        .context("Failed to emit agent request")?;

    match client::unwrap_result(emit_resp) {
        Ok(result) => {
            let seq = result["seq"].as_u64().unwrap_or(0);
            eprintln!("Request sent: action={}, request_id={}, seq={}", action, request_id, seq);
        }
        Err(e) => {
            anyhow::bail!("Failed to emit agent request: {}", e);
        }
    }

    // Poll for agent.response and agent.status
    eprintln!("Waiting for response (timeout: {}s)...", timeout);

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(interval);
    let mut poll_cursor = cursor;

    loop {
        // Poll for both response and status topics
        let mut poll_params = serde_json::json!({});
        if let Some(c) = poll_cursor {
            poll_params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", poll_params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let topic = event["topic"].as_str().unwrap_or("");
                            let event_payload = &event["payload"];

                            // Check request_id correlation
                            let matches = event_payload
                                .get("request_id")
                                .and_then(|r| r.as_str())
                                .map(|r| r == request_id)
                                .unwrap_or(false);

                            if !matches { continue; }

                            if topic == agent_topic::RESPONSE {
                                // Final response — parse and display
                                if let Ok(response) = serde_json::from_value::<AgentResponse>(event_payload.clone()) {
                                    if response.status == termlink_protocol::events::ResponseStatus::Ok {
                                        println!("{}", serde_json::to_string_pretty(&response.result)?);
                                    } else {
                                        let msg = response.error_message.as_deref().unwrap_or("unknown error");
                                        eprintln!("Error: {}", msg);
                                        std::process::exit(1);
                                    }
                                } else {
                                    // Fallback: print raw payload
                                    println!("{}", serde_json::to_string_pretty(event_payload)?);
                                }
                                return Ok(());
                            }

                            if topic == agent_topic::STATUS {
                                // Intermediate status — print and continue waiting
                                if let Ok(status) = serde_json::from_value::<AgentStatus>(event_payload.clone()) {
                                    let pct = status.percent.map(|p| format!(" ({}%)", p)).unwrap_or_default();
                                    let msg = status.message.as_deref().unwrap_or("");
                                    eprintln!("[status] {}{}: {}", status.phase, pct, msg);
                                }
                            }
                        }
                    }

                    if let Some(events) = result["events"].as_array() {
                        if !events.is_empty() {
                            if let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if start.elapsed() > timeout_dur {
            anyhow::bail!("Timeout waiting for agent response ({}s). request_id={}", timeout, request_id);
        }

        tokio::time::sleep(poll_interval).await;
    }
}

async fn cmd_agent_listen(
    target: &str,
    timeout: u64,
    interval: u64,
) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    eprintln!("Listening for agent requests on '{}' (topic: {})...", target, agent_topic::REQUEST);
    if timeout > 0 {
        eprintln!("Timeout: {}s", timeout);
    } else {
        eprintln!("Press Ctrl+C to stop");
    }

    let start = std::time::Instant::now();
    let timeout_dur = if timeout > 0 {
        Some(std::time::Duration::from_secs(timeout))
    } else {
        None
    };
    let poll_interval = std::time::Duration::from_millis(interval);

    // Start with no cursor — first poll gets all events, then track via next_seq.
    let mut poll_cursor: Option<u64> = None;

    loop {
        let mut params = serde_json::json!({ "topic": agent_topic::REQUEST });
        if let Some(c) = poll_cursor {
            params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let event_payload = &event["payload"];
                            if let Ok(req) = serde_json::from_value::<AgentRequest>(event_payload.clone()) {
                                println!("[{}] from={} action={} request_id={}",
                                    event["seq"].as_u64().unwrap_or(0),
                                    req.from, req.action, req.request_id);
                                if req.params != serde_json::json!(null) && req.params != serde_json::json!({}) {
                                    println!("  params: {}", serde_json::to_string(&req.params)?);
                                }
                            } else {
                                // Raw event
                                println!("{}", serde_json::to_string_pretty(event_payload)?);
                            }
                        }
                    }

                    if let Some(events) = result["events"].as_array() {
                        if !events.is_empty() {
                            if let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if let Some(td) = timeout_dur {
            if start.elapsed() > td {
                eprintln!("Listen timeout reached ({}s)", timeout);
                return Ok(());
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

/// Default chunk size for file transfers (48KB raw → ~64KB base64).
const DEFAULT_CHUNK_SIZE: usize = 49152;

async fn cmd_file_send(target: &str, path: &str, chunk_size: usize) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let file_path = std::path::Path::new(path);
    let file_data = std::fs::read(file_path)
        .context(format!("Failed to read file: {}", path))?;

    let filename = file_path
        .file_name()
        .map(|n| n.to_string_lossy().to_string())
        .unwrap_or_else(|| "unnamed".to_string());

    let size = file_data.len() as u64;
    let chunk_sz = if chunk_size == 0 { DEFAULT_CHUNK_SIZE } else { chunk_size };
    let total_chunks = ((file_data.len() + chunk_sz - 1) / chunk_sz) as u32;

    let transfer_id = generate_request_id().replace("req-", "xfer-");

    // Compute SHA-256
    let mut hasher = Sha256::new();
    hasher.update(&file_data);
    let sha256 = format!("{:x}", hasher.finalize());

    // Emit file.init
    let init = FileInit {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        filename: filename.clone(),
        size,
        total_chunks,
        from: format!("cli-{}", std::process::id()),
    };
    let init_payload = serde_json::to_value(&init)?;
    let emit_params = serde_json::json!({
        "topic": file_topic::INIT,
        "payload": init_payload,
    });
    client::rpc_call(reg.socket_path(), "event.emit", emit_params)
        .await
        .context("Failed to emit file.init")?;

    eprintln!(
        "Sending '{}' ({} bytes, {} chunks) transfer_id={}",
        filename, size, total_chunks, transfer_id
    );

    // Emit chunks
    let encoder = base64::engine::general_purpose::STANDARD;
    for (i, chunk_data) in file_data.chunks(chunk_sz).enumerate() {
        let chunk = FileChunk {
            schema_version: SCHEMA_VERSION.to_string(),
            transfer_id: transfer_id.clone(),
            index: i as u32,
            data: encoder.encode(chunk_data),
        };
        let chunk_payload = serde_json::to_value(&chunk)?;
        let emit_params = serde_json::json!({
            "topic": file_topic::CHUNK,
            "payload": chunk_payload,
        });
        client::rpc_call(reg.socket_path(), "event.emit", emit_params)
            .await
            .context(format!("Failed to emit chunk {}/{}", i + 1, total_chunks))?;

        if total_chunks > 1 {
            eprint!("\r  Chunk {}/{}", i + 1, total_chunks);
        }
    }
    if total_chunks > 1 {
        eprintln!();
    }

    // Emit file.complete
    let complete = FileComplete {
        schema_version: SCHEMA_VERSION.to_string(),
        transfer_id: transfer_id.clone(),
        sha256: sha256.clone(),
    };
    let complete_payload = serde_json::to_value(&complete)?;
    let emit_params = serde_json::json!({
        "topic": file_topic::COMPLETE,
        "payload": complete_payload,
    });
    client::rpc_call(reg.socket_path(), "event.emit", emit_params)
        .await
        .context("Failed to emit file.complete")?;

    eprintln!("Transfer complete. SHA-256: {}", sha256);
    Ok(())
}

async fn cmd_file_receive(
    target: &str,
    output_dir: &str,
    timeout: u64,
    interval: u64,
) -> Result<()> {
    use base64::Engine;
    use sha2::{Digest, Sha256};

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let out_path = std::path::Path::new(output_dir);
    if !out_path.exists() {
        std::fs::create_dir_all(out_path)
            .context(format!("Failed to create output directory: {}", output_dir))?;
    }

    eprintln!("Waiting for file transfer on '{}' (timeout: {}s)...", target, timeout);

    let start = std::time::Instant::now();
    let timeout_dur = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(interval);

    // Snapshot the current event cursor so we only see NEW events.
    // Without this, the receiver replays all historical events and picks up
    // stale transfers from previous runs (T-198).
    let mut poll_cursor: Option<u64> = match client::rpc_call(
        reg.socket_path(), "event.poll", serde_json::json!({}),
    ).await {
        Ok(resp) => {
            if let Ok(result) = client::unwrap_result(resp) {
                result["next_seq"].as_u64()
            } else {
                None
            }
        }
        Err(_) => None,
    };

    // State machine: waiting for init → collecting chunks → complete
    let mut transfer_id: Option<String> = None;
    let mut filename: Option<String> = None;
    let mut expected_chunks: u32 = 0;
    let mut chunks: std::collections::BTreeMap<u32, Vec<u8>> = std::collections::BTreeMap::new();

    let decoder = base64::engine::general_purpose::STANDARD;

    loop {
        let mut params = serde_json::json!({});
        if let Some(c) = poll_cursor {
            params["since"] = serde_json::json!(c);
        }

        match client::rpc_call(reg.socket_path(), "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    if let Some(events) = result["events"].as_array() {
                        for event in events {
                            let topic = event["topic"].as_str().unwrap_or("");
                            let payload = &event["payload"];

                            match topic {
                                t if t == file_topic::INIT => {
                                    if let Ok(init) = serde_json::from_value::<FileInit>(payload.clone()) {
                                        eprintln!(
                                            "Receiving '{}' ({} bytes, {} chunks) from {}",
                                            init.filename, init.size, init.total_chunks, init.from
                                        );
                                        transfer_id = Some(init.transfer_id);
                                        filename = Some(init.filename);
                                        expected_chunks = init.total_chunks;
                                        chunks.clear();
                                    }
                                }
                                t if t == file_topic::CHUNK => {
                                    if let Ok(chunk) = serde_json::from_value::<FileChunk>(payload.clone()) {
                                        if transfer_id.as_deref() == Some(&chunk.transfer_id) {
                                            let decoded = decoder.decode(&chunk.data)
                                                .context(format!("Invalid base64 in chunk {}", chunk.index))?;
                                            chunks.insert(chunk.index, decoded);

                                            if expected_chunks > 1 {
                                                eprint!("\r  Chunk {}/{}", chunks.len(), expected_chunks);
                                            }
                                        }
                                    }
                                }
                                t if t == file_topic::COMPLETE => {
                                    if let Ok(complete) = serde_json::from_value::<FileComplete>(payload.clone()) {
                                        if transfer_id.as_deref() == Some(&complete.transfer_id) {
                                            if expected_chunks > 1 {
                                                eprintln!();
                                            }

                                            // Reassemble file
                                            let mut file_data = Vec::new();
                                            for i in 0..expected_chunks {
                                                match chunks.get(&i) {
                                                    Some(data) => file_data.extend_from_slice(data),
                                                    None => anyhow::bail!("Missing chunk {} of {}", i, expected_chunks),
                                                }
                                            }

                                            // Verify SHA-256
                                            let mut hasher = Sha256::new();
                                            hasher.update(&file_data);
                                            let actual_sha256 = format!("{:x}", hasher.finalize());

                                            if actual_sha256 != complete.sha256 {
                                                anyhow::bail!(
                                                    "SHA-256 mismatch! Expected: {}, Got: {}",
                                                    complete.sha256, actual_sha256
                                                );
                                            }

                                            // Write file
                                            let fname = filename.as_deref().unwrap_or("received-file");
                                            let dest = out_path.join(fname);
                                            std::fs::write(&dest, &file_data)
                                                .context(format!("Failed to write file: {}", dest.display()))?;

                                            eprintln!("File saved: {} ({} bytes)", dest.display(), file_data.len());
                                            eprintln!("SHA-256 verified: {}", actual_sha256);
                                            return Ok(());
                                        }
                                    }
                                }
                                t if t == file_topic::ERROR => {
                                    if let Some(msg) = payload.get("message").and_then(|m| m.as_str()) {
                                        let xfer = payload.get("transfer_id").and_then(|t| t.as_str()).unwrap_or("?");
                                        if transfer_id.as_deref() == Some(xfer) || transfer_id.is_none() {
                                            anyhow::bail!("Transfer error: {}", msg);
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }

                    // Only advance cursor when events were returned — avoids
                    // skipping seq 0 when the bus was empty on first poll.
                    if let Some(events) = result["events"].as_array() {
                        if !events.is_empty() {
                            if let Some(next) = result["next_seq"].as_u64() {
                                poll_cursor = Some(next);
                            }
                        }
                    }
                }
            }
            Err(e) => {
                tracing::warn!("Poll error: {}", e);
            }
        }

        if start.elapsed() > timeout_dur {
            if transfer_id.is_some() {
                anyhow::bail!(
                    "Timeout: received {}/{} chunks before timeout ({}s)",
                    chunks.len(), expected_chunks, timeout
                );
            } else {
                anyhow::bail!("Timeout waiting for file transfer ({}s)", timeout);
            }
        }

        tokio::time::sleep(poll_interval).await;
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn strip_ansi_removes_csi_sequences() {
        let input = "\x1b[0;32mOK\x1b[0m  Framework installation";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "OK  Framework installation");
    }

    #[test]
    fn strip_ansi_removes_osc_sequences() {
        let input = "\x1b]7;file://host/path\x07prompt % ";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "prompt % ");
    }

    #[test]
    fn strip_ansi_preserves_plain_text() {
        let input = "hello world\nline 2\nline 3";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "hello world\nline 2\nline 3");
    }

    #[test]
    fn strip_ansi_removes_carriage_returns() {
        let input = "line1\r\nline2\r\n";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "line1\nline2\n");
    }

    #[test]
    fn strip_ansi_complex_terminal_output() {
        // Simulate real fw doctor output with ANSI
        let input = "\x1b[1mfw doctor\x1b[0m - Health Check\r\n  \x1b[0;32mOK\x1b[0m  Git hooks\r\n  \x1b[1;33mWARN\x1b[0m  Version mismatch\r\n";
        let result = strip_ansi_codes(input);
        assert_eq!(result, "fw doctor - Health Check\n  OK  Git hooks\n  WARN  Version mismatch\n");
    }
}
