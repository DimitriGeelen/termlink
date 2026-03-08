use std::sync::Arc;

use anyhow::{Context, Result};
use clap::{Parser, Subcommand};
use tokio::sync::RwLock;

use termlink_session::client;
use termlink_session::handler::SessionContext;
use termlink_session::manager;
use termlink_session::pty::PtySession;
use termlink_session::registration::SessionConfig;
use termlink_session::server;

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

    /// Discover all sessions (via hub discovery protocol)
    Discover,
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
        Command::Discover => cmd_discover(),
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

    let shared_clone = shared.clone();

    // If PTY, run the read loop in a background task
    let pty_handle = if let Some(ref pty) = pty_session {
        let pty_clone = pty.clone();
        Some(tokio::spawn(async move {
            let _ = pty_clone.read_loop().await;
        }))
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

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}
