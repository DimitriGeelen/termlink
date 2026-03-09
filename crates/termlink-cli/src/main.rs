use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
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
    },

    /// List all registered sessions
    List {
        /// Include stale/dead sessions
        #[arg(long)]
        all: bool,
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
    },

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

    /// Send a signal to a session's process (e.g., SIGTERM, SIGINT)
    Signal {
        /// Session ID or display name
        target: String,

        /// Signal name or number (e.g., TERM, INT, KILL, HUP, 15)
        signal: String,
    },

    /// Poll events from a session's event bus
    Events {
        /// Session ID or display name
        target: String,

        /// Only show events after this sequence number
        #[arg(long, default_value = "0")]
        since: u64,

        /// Filter by topic
        #[arg(long)]
        topic: Option<String>,
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
    },

    /// Manage key-value metadata on a session
    Kv {
        /// Session ID or display name
        target: String,

        #[command(subcommand)]
        action: KvAction,
    },

    /// Show TermLink runtime information and system status
    Info,

    /// List event topics from one or all sessions
    Topics {
        /// Session ID or display name (omit for all sessions)
        target: Option<String>,
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

    /// Remove stale (dead) session registrations from the runtime directory
    Clean {
        /// Show what would be removed without actually removing
        #[arg(long)]
        dry_run: bool,
    },

    /// Start the hub server (routes requests between sessions)
    Hub,
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
        Command::Register { name, roles, tags, shell } => {
            cmd_register(name, roles, tags, shell).await
        }
        Command::List { all } => cmd_list(all),
        Command::Ping { target } => cmd_ping(&target).await,
        Command::Status { target } => cmd_status(&target).await,
        Command::Send { target, method, params } => cmd_send(&target, &method, &params).await,
        Command::Exec { target, command, cwd, timeout } => {
            cmd_exec(&target, &command, cwd.as_deref(), timeout).await
        }
        Command::Output { target, lines, bytes } => cmd_output(&target, lines, bytes).await,
        Command::Inject { target, text, enter, key } => {
            cmd_inject(&target, &text, enter, key.as_deref()).await
        }
        Command::Attach { target, poll_ms } => cmd_attach(&target, poll_ms).await,
        Command::Signal { target, signal } => cmd_signal(&target, &signal).await,
        Command::Events { target, since, topic } => {
            cmd_events(&target, since, topic.as_deref()).await
        }
        Command::Broadcast { topic, payload, targets } => {
            cmd_broadcast(&topic, &payload, targets).await
        }
        Command::Emit { target, topic, payload } => {
            cmd_emit(&target, &topic, &payload).await
        }
        Command::Resize { target, cols, rows } => cmd_resize(&target, cols, rows).await,
        Command::Stream { target } => cmd_stream(&target).await,
        Command::Tag { target, set, add, remove } => {
            cmd_tag(&target, set, add, remove).await
        }
        Command::Watch { targets, interval, topic } => {
            cmd_watch(targets, interval, topic.as_deref()).await
        }
        Command::Discover { tag, role, cap, name } => {
            cmd_discover(tag, role, cap, name)
        }
        Command::Kv { target, action } => cmd_kv(&target, action).await,
        Command::Info => cmd_info(),
        Command::Topics { target } => cmd_topics(target.as_deref()).await,
        Command::Collect { targets, topic, interval, count } => {
            cmd_collect(targets, topic.as_deref(), interval, count).await
        }
        Command::Run { name, tags, timeout, command } => {
            cmd_run(name, tags, timeout, command).await
        }
        Command::Wait { target, topic, timeout, interval } => {
            cmd_wait(&target, &topic, timeout, interval).await
        }
        Command::Clean { dry_run } => cmd_clean(dry_run),
        Command::Hub => cmd_hub().await,
    }
}

async fn cmd_register(
    name: Option<String>,
    roles: Vec<String>,
    tags: Vec<String>,
    shell: bool,
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

    println!("Session registered:");
    println!("  ID:      {}", session.id());
    println!("  Name:    {}", session.display_name());
    println!("  Socket:  {}", session.registration.socket.display());

    // Set up session context (with or without PTY)
    let pty_session = if shell {
        // Set data_socket metadata for discoverability
        let data_path = data_server::data_socket_path(&session.registration.socket);
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

    // Persist updated registration (capabilities + metadata)
    if shell {
        session.persist_registration()
            .context("Failed to persist updated registration")?;
    }

    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    let json_path = termlink_session::registration::Registration::json_path(
        &termlink_session::discovery::sessions_dir(),
        session.id(),
    );
    let ctx = if let Some(ref pty) = pty_session {
        SessionContext::with_pty(session.registration.clone(), pty.clone())
            .with_registration_path(json_path)
    } else {
        SessionContext::new(session.registration.clone())
            .with_registration_path(json_path)
    };
    let shared = Arc::new(RwLock::new(ctx));

    // Handle Ctrl+C for graceful shutdown
    let session_id = session.id().clone();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let listener = session.listener;
    let reg_for_cleanup = session.registration;

    // Compute data socket path before moving reg
    let data_socket_path = if shell {
        Some(data_server::data_socket_path(&reg_for_cleanup.socket))
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
            let _ = std::fs::remove_file(&reg_for_cleanup.socket);
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

fn cmd_list(include_stale: bool) -> Result<()> {
    let sessions = manager::list_sessions(include_stale)
        .context("Failed to list sessions")?;

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

    let resp = client::rpc_call(&reg.socket, "termlink.ping", serde_json::json!({}))
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

async fn cmd_status(target: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(&reg.socket, "query.status", serde_json::json!({}))
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
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

    let resp = client::rpc_call(&reg.socket, "command.execute", params)
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

async fn cmd_send(target: &str, method: &str, params_str: &str) -> Result<()> {
    let params: serde_json::Value =
        serde_json::from_str(params_str).context("Invalid JSON params")?;

    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let resp = client::rpc_call(&reg.socket, method, params)
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

async fn cmd_output(target: &str, lines: u64, bytes: Option<u64>) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let params = if let Some(b) = bytes {
        serde_json::json!({ "bytes": b })
    } else {
        serde_json::json!({ "lines": lines })
    };

    let resp = client::rpc_call(&reg.socket, "query.output", params)
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

    let resp = client::rpc_call(&reg.socket, "command.inject", params)
        .await
        .context("Failed to connect to session")?;

    match client::unwrap_result(resp) {
        Ok(result) => {
            let bytes = result["bytes_written"].as_u64().unwrap_or(0);
            println!("Injected {bytes} bytes");
            Ok(())
        }
        Err(e) => {
            anyhow::bail!("Inject failed: {}", e);
        }
    }
}

async fn cmd_signal(target: &str, signal: &str) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let sig_num = parse_signal(signal)
        .context(format!("Unknown signal: '{}'. Use TERM, INT, KILL, HUP, USR1, USR2, or a number.", signal))?;

    let resp = client::rpc_call(
        &reg.socket,
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
    let resp = client::rpc_call(&reg.socket, "query.output", serde_json::json!({ "lines": 0 }))
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
    let result = attach_loop(&reg.socket, poll_ms).await;

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

async fn cmd_events(target: &str, since: u64, topic: Option<&str>) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let mut params = serde_json::json!({ "since": since });
    if let Some(t) = topic {
        params["topic"] = serde_json::json!(t);
    }

    let resp = client::rpc_call(&reg.socket, "event.poll", params)
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
        &reg.socket,
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
        &reg.socket,
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
    let data_socket = data_server::data_socket_path(&reg.socket);
    if !data_socket.exists() {
        anyhow::bail!(
            "No data plane for '{}'. Start with --shell to enable data plane.",
            target
        );
    }

    // Fetch initial scrollback via control plane before entering raw mode
    let resp = client::rpc_call(&reg.socket, "query.output", serde_json::json!({ "lines": 100 }))
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

    let resp = client::rpc_call(&reg.socket, "session.update", params)
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

                    let resp = match client::rpc_call(&reg.socket, "event.poll", params).await {
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
                &reg.socket,
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
                &reg.socket,
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
                &reg.socket,
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
                &reg.socket,
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

fn cmd_info() -> Result<()> {
    let runtime_dir = termlink_session::discovery::runtime_dir();
    let sessions_dir = termlink_session::discovery::sessions_dir();
    let hub_socket = termlink_hub::server::hub_socket_path();

    println!("TermLink Runtime");
    println!("{}", "-".repeat(40));
    println!("  Runtime dir:  {}", runtime_dir.display());
    println!("  Sessions dir: {}", sessions_dir.display());
    println!("  Hub socket:   {}", hub_socket.display());

    // Check hub status
    let hub_running = hub_socket.exists();
    println!(
        "  Hub:          {}",
        if hub_running { "running" } else { "stopped" }
    );

    // Count sessions
    let live = manager::list_sessions(false)
        .map(|s| s.len())
        .unwrap_or(0);
    let all = manager::list_sessions(true)
        .map(|s| s.len())
        .unwrap_or(0);
    let stale = all - live;

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
        match client::rpc_call(&reg.socket, "event.topics", serde_json::json!({})).await {
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

    let command_str = command_parts.join(" ");

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
        session.id(),
    );
    let ctx = SessionContext::new(session.registration.clone())
        .with_registration_path(json_path);
    let shared = Arc::new(RwLock::new(ctx));
    let shared_clone = shared.clone();

    let listener = session.listener;
    let reg_for_cleanup = session.registration;

    // Run RPC listener in background so the session is queryable during execution
    let rpc_handle = tokio::spawn(async move {
        server::run_accept_loop(listener, shared_clone).await;
    });

    // Execute the command
    let result = executor::execute(
        &command_str,
        None,
        None,
        Some(std::time::Duration::from_secs(timeout)),
    )
    .await;

    // Abort RPC listener
    rpc_handle.abort();

    // Cleanup: deregister session
    let json_path = termlink_session::registration::Registration::json_path(
        &sessions_dir,
        &session_id,
    );
    let _ = std::fs::remove_file(&reg_for_cleanup.socket);
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

    // Get current event bus state. We want to only see events emitted AFTER
    // the wait starts. The poll RPC uses `seq > since`, so to catch the next
    // event at seq=N we need cursor = N-1. For an empty bus (next_seq=0),
    // we use None to signal "no since filter" on the first poll.
    let initial_next_seq: Option<u64> = {
        let params = serde_json::json!({});
        match client::rpc_call(&reg.socket, "event.poll", params).await {
            Ok(resp) => {
                if let Ok(result) = client::unwrap_result(resp) {
                    result["next_seq"].as_u64()
                } else {
                    None
                }
            }
            Err(_) => None,
        }
    };
    // If next_seq > 0, we can safely use next_seq - 1 as cursor (catches seq >= next_seq).
    // If next_seq == 0, the bus is empty; use None to skip the "since" filter initially.
    let mut cursor: Option<u64> = initial_next_seq.and_then(|n| {
        if n > 0 { Some(n - 1) } else { None }
    });

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
                let resp = match client::rpc_call(&reg.socket, "event.poll", params).await {
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

async fn cmd_hub() -> Result<()> {
    let socket_path = termlink_hub::server::hub_socket_path();

    println!("Starting hub server...");
    println!("  Socket: {}", socket_path.display());
    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    let socket_clone = socket_path.clone();
    tokio::select! {
        result = termlink_hub::server::run(&socket_path) => {
            result.context("Hub server error")?;
        }
        _ = tokio::signal::ctrl_c() => {
            println!();
            println!("Shutting down hub...");
            let _ = std::fs::remove_file(&socket_clone);
            println!("Hub stopped.");
        }
    }

    Ok(())
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
