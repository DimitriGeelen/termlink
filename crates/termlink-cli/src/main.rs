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

    /// Stream a PTY session via data plane (real-time binary frames, zero polling)
    Stream {
        /// Session ID or display name
        target: String,
    },

    /// Discover all sessions (via hub discovery protocol)
    Discover,

    /// Start the hub server (routes requests between sessions)
    Hub,
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
        Command::Register { name, roles, shell } => cmd_register(name, roles, shell).await,
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
        Command::Stream { target } => cmd_stream(&target).await,
        Command::Discover => cmd_discover(),
        Command::Hub => cmd_hub().await,
    }
}

async fn cmd_register(name: Option<String>, roles: Vec<String>, shell: bool) -> Result<()> {
    let config = SessionConfig {
        display_name: name,
        roles,
        ..Default::default()
    };

    let session = termlink_session::Session::register(config)
        .await
        .context("Failed to register session")?;

    println!("Session registered:");
    println!("  ID:      {}", session.id());
    println!("  Name:    {}", session.display_name());
    println!("  Socket:  {}", session.registration.socket.display());

    // Set up session context (with or without PTY)
    let pty_session = if shell {
        let pty = PtySession::spawn(None, 1024 * 1024)
            .context("Failed to spawn PTY session")?;
        println!("  PTY:     yes (shell: {})",
            std::env::var("SHELL").unwrap_or_else(|_| "/bin/sh".into()));
        Some(Arc::new(pty))
    } else {
        println!("  PTY:     no (use --shell for bidirectional I/O)");
        None
    };

    println!();
    println!("Listening for connections... (Ctrl+C to stop)");

    let ctx = if let Some(ref pty) = pty_session {
        SessionContext::with_pty(session.registration.clone(), pty.clone())
    } else {
        SessionContext::new(session.registration.clone())
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
        "{:<14} {:<16} {:<14} {:<8}",
        "ID", "NAME", "STATE", "PID"
    );
    println!("{}", "-".repeat(54));

    for session in &sessions {
        println!(
            "{:<14} {:<16} {:<14} {:<8}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            session.pid,
        );
    }

    println!();
    println!("{} session(s)", sessions.len());
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

fn cmd_discover() -> Result<()> {
    let sessions = manager::list_sessions(false)
        .context("Failed to discover sessions")?;

    if sessions.is_empty() {
        println!("No sessions discovered.");
        return Ok(());
    }

    println!(
        "{:<14} {:<16} {:<14} {:<20} ROLES",
        "ID", "NAME", "STATE", "CAPABILITIES"
    );
    println!("{}", "-".repeat(70));

    for session in &sessions {
        println!(
            "{:<14} {:<16} {:<14} {:<20} {}",
            session.id.as_str(),
            truncate(&session.display_name, 15),
            session.state,
            session.capabilities.join(","),
            session.roles.join(","),
        );
    }

    println!();
    println!("{} session(s) discovered", sessions.len());
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

/// Real-time data plane streaming loop.
async fn stream_loop(stream: tokio::net::UnixStream) -> Result<()> {
    use tokio::io::AsyncReadExt;

    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

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
        }
    }

    Ok(())
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
