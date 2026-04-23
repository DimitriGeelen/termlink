use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::codec::{FrameReader, FrameWriter};
use termlink_session::data_server;
use termlink_session::manager;

use termlink_protocol::data::{FrameFlags, FrameType};

use crate::util::{resize_payload, strip_ansi_codes, terminal_size};

pub(crate) async fn cmd_interact(
    target: &str,
    command: &str,
    timeout: u64,
    poll_ms: u64,
    strip_ansi: bool,
    json_output: bool,
) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json_output {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

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
    .await;
    let pre_resp = match pre_resp {
        Ok(r) => r,
        Err(e) => {
            if json_output {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to query output (is this a PTY session?): {}", e)}));
            }
            return Err(e).context("Failed to query output (is this a PTY session?)");
        }
    };

    let pre_output = match client::unwrap_result(pre_resp) {
        Ok(r) => r["output"].as_str().unwrap_or("").to_string(),
        Err(e) => {
            if json_output {
                super::json_error_exit(serde_json::json!({"ok": false, "output": "", "exit_code": null, "error": format!("Session has no PTY: {e}"), "marker_found": false}));
            }
            anyhow::bail!("Session has no PTY: {}", e);
        }
    };
    let pre_len = pre_output.len();

    // Inject strategy: send command + marker echo on a SINGLE line using `;`.
    let inject_line = format!("{command}; echo \"{marker} exit=$?\"");
    let keys = serde_json::json!([
        { "type": "text", "value": inject_line },
        { "type": "key", "value": "Enter" }
    ]);
    if let Err(e) = client::rpc_call(
        reg.socket_path(),
        "command.inject",
        serde_json::json!({ "keys": keys }),
    )
    .await
    {
        if json_output {
            super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to inject command: {}", e)}));
        }
        return Err(e).context("Failed to inject command");
    }

    let start = std::time::Instant::now();
    let deadline = std::time::Duration::from_secs(timeout);
    let poll_interval = std::time::Duration::from_millis(poll_ms);

    // Poll until marker appears in scrollback
    loop {
        if start.elapsed() > deadline {
            if json_output {
                super::json_error_exit(serde_json::json!({"ok": false, "output": "", "exit_code": null, "error": format!("Timeout after {}s waiting for command to complete", timeout), "elapsed_ms": start.elapsed().as_millis() as u64, "marker_found": false}));
            }
            anyhow::bail!("Timeout after {}s waiting for command to complete", timeout);
        }

        tokio::time::sleep(poll_interval).await;

        let resp = client::rpc_call(
            reg.socket_path(),
            "query.output",
            serde_json::json!({ "bytes": 131072 }),
        )
        .await
        .context("Failed to poll output")?;

        let result = match client::unwrap_result(resp) {
            Ok(r) => r,
            Err(e) => {
                if json_output {
                    super::json_error_exit(serde_json::json!({"ok": false, "output": "", "exit_code": null, "error": format!("Output poll failed: {e}"), "marker_found": false}));
                }
                anyhow::bail!("Output poll failed: {}", e);
            }
        };

        let full_output = result["output"].as_str().unwrap_or("");

        let output = if full_output.len() > pre_len {
            &full_output[pre_len..]
        } else {
            full_output
        };

        if has_marker(output, &marker) {
            let elapsed_ms = start.elapsed().as_millis();

            let exit_code = parse_exit_code(output, &marker);

            let clean_output = extract_clean_output(output, &marker);

            let final_output = if strip_ansi {
                strip_ansi_codes(&clean_output)
            } else {
                clean_output
            };

            let final_output = final_output.trim();

            if json_output {
                let is_ok = exit_code.is_none_or(|c| c == 0);
                let json = serde_json::json!({
                    "ok": is_ok,
                    "output": final_output,
                    "exit_code": exit_code,
                    "elapsed_ms": elapsed_ms,
                    "marker_found": true,
                    "bytes_captured": output.len(),
                });
                println!("{}", serde_json::to_string_pretty(&json)?);
                if let Some(code) = exit_code
                    && code != 0 {
                        std::process::exit(code);
                    }
            } else {
                if !final_output.is_empty() {
                    println!("{final_output}");
                }
                if let Some(code) = exit_code
                    && code != 0 {
                        std::process::exit(code);
                    }
            }

            return Ok(());
        }
    }
}

pub(crate) async fn cmd_output(target: &str, lines: u64, bytes: Option<u64>, strip_ansi: bool, json: bool, timeout_secs: u64) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let mut params = if let Some(b) = bytes {
        serde_json::json!({ "bytes": b })
    } else {
        serde_json::json!({ "lines": lines })
    };

    if strip_ansi {
        params["strip_ansi"] = serde_json::json!(true);
    }

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "query.output", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Output query timed out after {}s", timeout_secs),
                }));
            }
            anyhow::bail!("Output query timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            let output = result["output"].as_str().unwrap_or("");
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "output": output,
                    "bytes": output.len(),
                    "target": target,
                    "total_buffered": result["total_buffered"],
                }));
            } else {
                print!("{output}");
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
            }
            anyhow::bail!("Output query failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_inject(target: &str, text: &str, enter: bool, key: Option<&str>, json: bool, timeout_secs: u64) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

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

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "command.inject", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Inject timed out after {}s", timeout_secs)}));
            }
            anyhow::bail!("Inject timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            let bytes = result["bytes_len"].as_u64().unwrap_or(0);
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "target": target,
                    "bytes_injected": bytes,
                }));
            } else {
                println!("Injected {bytes} bytes");
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
            }
            anyhow::bail!("Inject failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_resize(target: &str, cols: u16, rows: u16, json: bool, timeout_secs: u64) -> Result<()> {
    let reg = match manager::find_session(target) {
        Ok(r) => r,
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Session '{}' not found: {}", target, e)}));
            }
            return Err(e).context(format!("Session '{}' not found", target));
        }
    };

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(
        reg.socket_path(),
        "command.resize",
        serde_json::json!({ "cols": cols, "rows": rows }),
    );
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => match result {
            Ok(r) => r,
            Err(e) => {
                if json {
                    super::json_error_exit(serde_json::json!({"ok": false, "target": target, "error": format!("Failed to connect to session: {}", e)}));
                }
                return Err(e).context("Failed to connect to session");
            }
        },
        Err(_) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Resize timed out after {}s", timeout_secs),
                }));
            }
            anyhow::bail!("Resize timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": true,
                    "target": target,
                    "cols": result["cols"].as_u64().unwrap_or(cols as u64),
                    "rows": result["rows"].as_u64().unwrap_or(rows as u64),
                }));
            } else {
                println!(
                    "Resized to {}x{}",
                    result["cols"].as_u64().unwrap_or(cols as u64),
                    result["rows"].as_u64().unwrap_or(rows as u64),
                );
            }
            Ok(())
        }
        Err(e) => {
            if json {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
            }
            anyhow::bail!("Resize failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_attach(target: &str, poll_ms: u64) -> Result<()> {
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
                                let output = result["output"].as_str().unwrap_or("");
                                let output_bytes = output.as_bytes();
                                let new_data = compute_output_delta(output_bytes, last_buffered, new_buffered);

                                if !new_data.is_empty() {
                                    let stdout = std::io::stdout();
                                    let mut out = stdout.lock();
                                    std::io::Write::write_all(&mut out, new_data)?;
                                    std::io::Write::flush(&mut out)?;
                                }
                            }

                            last_buffered = new_buffered;
                        }
                    }
                    Err(_) => {
                        eprintln!("\r\nConnection lost.");
                        break;
                    }
                }
            }
        }
    }

    Ok(())
}

pub(crate) async fn cmd_stream(target: &str) -> Result<()> {
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

pub(crate) async fn cmd_mirror(target: &str, scrollback_lines: u64, raw: bool) -> Result<()> {
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

    // Fetch initial scrollback via control plane
    let resp = client::rpc_call(
        reg.socket_path(),
        "query.output",
        serde_json::json!({ "lines": scrollback_lines }),
    )
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
        "Mirroring {} ({}){} — read-only. Press Ctrl+C to stop.",
        reg.display_name,
        reg.id,
        if raw { " [raw]" } else { "" }
    );

    if raw {
        mirror_loop_raw(stream).await
    } else {
        mirror_loop_grid(stream).await
    }
}

/// Legacy byte-passthrough mirror loop (pre-T-1199).
async fn mirror_loop_raw(stream: tokio::net::UnixStream) -> Result<()> {
    let (read_half, _write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);

    let mut sigint = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::interrupt(),
    ).context("Failed to register SIGINT handler")?;

    loop {
        tokio::select! {
            frame = reader.read_frame() => {
                match frame {
                    Ok(Some(frame)) => {
                        if frame.header.frame_type == FrameType::Output {
                            let stdout = std::io::stdout();
                            let mut out = stdout.lock();
                            std::io::Write::write_all(&mut out, &frame.payload)?;
                            std::io::Write::flush(&mut out)?;
                        }
                        if frame.header.frame_type == FrameType::Close {
                            eprintln!("\nSession closed connection.");
                            break;
                        }
                    }
                    Ok(None) => {
                        eprintln!("\nData plane disconnected.");
                        break;
                    }
                    Err(e) => {
                        eprintln!("\nData plane error: {e}");
                        break;
                    }
                }
            }

            _ = sigint.recv() => {
                eprintln!("\nMirror stopped.");
                break;
            }
        }
    }

    Ok(())
}

/// Grid-aware mirror loop — feeds Output frames through a vte parser and emits
/// a full repaint per frame. Dirty-cell diffing is a follow-up (see T-1191).
async fn mirror_loop_grid(stream: tokio::net::UnixStream) -> Result<()> {
    use super::mirror_grid::Grid;

    let (read_half, _write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);

    let mut sigint = tokio::signal::unix::signal(
        tokio::signal::unix::SignalKind::interrupt(),
    ).context("Failed to register SIGINT handler")?;

    let (cols, rows) = terminal_size();
    let mut grid = Grid::new(cols.max(1), rows.max(1));
    let mut parser = vte::Parser::new();

    loop {
        tokio::select! {
            frame = reader.read_frame() => {
                match frame {
                    Ok(Some(frame)) => {
                        match frame.header.frame_type {
                            FrameType::Output => {
                                for b in &frame.payload {
                                    parser.advance(&mut grid, *b);
                                }
                                let stdout = std::io::stdout();
                                let mut out = stdout.lock();
                                let _ = grid.render_full(&mut out);
                            }
                            FrameType::Resize => {
                                if frame.payload.len() >= 4 {
                                    let c = u16::from_be_bytes([frame.payload[0], frame.payload[1]]);
                                    let r = u16::from_be_bytes([frame.payload[2], frame.payload[3]]);
                                    grid.resize(c.max(1), r.max(1));
                                }
                            }
                            FrameType::Close => {
                                eprintln!("\nSession closed connection.");
                                break;
                            }
                            _ => {}
                        }
                    }
                    Ok(None) => {
                        eprintln!("\nData plane disconnected.");
                        break;
                    }
                    Err(e) => {
                        eprintln!("\nData plane error: {e}");
                        break;
                    }
                }
            }

            _ = sigint.recv() => {
                eprintln!("\nMirror stopped.");
                break;
            }
        }
    }

    Ok(())
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

// === Extracted pure functions ===

/// Check if the output contains the completion marker with a valid exit code.
pub(crate) fn has_marker(output: &str, marker: &str) -> bool {
    let marker_with_exit = format!("{marker} exit=");
    output.contains(&marker_with_exit) && {
        output.lines().any(|line| {
            if let Some(pos) = line.find(&marker_with_exit) {
                let after = &line[pos + marker_with_exit.len()..];
                after.starts_with(|c: char| c.is_ascii_digit())
            } else {
                false
            }
        })
    }
}

/// Extract the exit code from output containing a marker line.
pub(crate) fn parse_exit_code(output: &str, marker: &str) -> Option<i32> {
    for line in output.lines() {
        if line.contains(marker) && let Some(exit_str) = line.split("exit=").nth(1) {
            return exit_str.trim().parse().ok();
        }
    }
    None
}

/// Strip the command echo (first line) and marker line from output,
/// returning only the command's actual output.
pub(crate) fn extract_clean_output(output: &str, marker: &str) -> String {
    let marker_with_exit = format!("{marker} exit=");

    let after_cmd_echo = output
        .find('\n')
        .map(|pos| &output[pos + 1..])
        .unwrap_or(output);

    if let Some(pos) = after_cmd_echo.find(&marker_with_exit) {
        let before = &after_cmd_echo[..pos];
        before
            .rfind('\n')
            .map(|nl| &after_cmd_echo[..nl])
            .unwrap_or("")
            .to_string()
    } else {
        after_cmd_echo.to_string()
    }
}

/// Compute which bytes are new in a polled output buffer.
///
/// `buffer` is the latest output slice (e.g. last 8192 bytes).
/// `last_buffered` / `new_buffered` are cumulative byte counters from the session.
/// Returns the slice of `buffer` that represents new data since `last_buffered`.
pub(crate) fn compute_output_delta(
    buffer: &[u8],
    last_buffered: u64,
    new_buffered: u64,
) -> &[u8] {
    if new_buffered <= last_buffered {
        return &[];
    }
    let delta = (new_buffered - last_buffered) as usize;
    if delta >= buffer.len() {
        buffer
    } else {
        &buffer[buffer.len() - delta..]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MARKER: &str = "___TERMLINK_DONE_abc_123___";

    // --- has_marker tests ---

    #[test]
    fn has_marker_with_exit_code() {
        let output = format!("some output\n{MARKER} exit=0\n$");
        assert!(has_marker(&output, MARKER));
    }

    #[test]
    fn has_marker_nonzero_exit() {
        let output = format!("error output\n{MARKER} exit=127\n$");
        assert!(has_marker(&output, MARKER));
    }

    #[test]
    fn has_marker_without_exit_code() {
        let output = format!("some output\n{MARKER}\n$");
        assert!(!has_marker(&output, MARKER));
    }

    #[test]
    fn has_marker_partial_marker() {
        let output = format!("some output\n{MARKER} exit=\n$");
        assert!(!has_marker(&output, MARKER), "exit= without digit should not match");
    }

    #[test]
    fn has_marker_not_present() {
        let output = "just regular output\nno marker here\n$";
        assert!(!has_marker(output, MARKER));
    }

    #[test]
    fn has_marker_empty_output() {
        assert!(!has_marker("", MARKER));
    }

    #[test]
    fn has_marker_embedded_in_longer_line() {
        let output = format!("prefix {MARKER} exit=42 suffix");
        assert!(has_marker(&output, MARKER));
    }

    // --- parse_exit_code tests ---

    #[test]
    fn parse_exit_code_zero() {
        let output = format!("output\n{MARKER} exit=0\n$");
        assert_eq!(parse_exit_code(&output, MARKER), Some(0));
    }

    #[test]
    fn parse_exit_code_nonzero() {
        let output = format!("output\n{MARKER} exit=1\n$");
        assert_eq!(parse_exit_code(&output, MARKER), Some(1));
    }

    #[test]
    fn parse_exit_code_127() {
        let output = format!("output\n{MARKER} exit=127\n$");
        assert_eq!(parse_exit_code(&output, MARKER), Some(127));
    }

    #[test]
    fn parse_exit_code_no_marker() {
        assert_eq!(parse_exit_code("no marker here", MARKER), None);
    }

    #[test]
    fn parse_exit_code_marker_without_exit() {
        let output = format!("{MARKER} something_else");
        assert_eq!(parse_exit_code(&output, MARKER), None);
    }

    #[test]
    fn parse_exit_code_negative() {
        let output = format!("{MARKER} exit=-1");
        assert_eq!(parse_exit_code(&output, MARKER), Some(-1));
    }

    // --- extract_clean_output tests ---

    #[test]
    fn extract_clean_output_normal() {
        let output = format!("ls; echo \"{MARKER} exit=$?\"\nfile1.txt\nfile2.txt\n{MARKER} exit=0\n$");
        let clean = extract_clean_output(&output, MARKER);
        assert_eq!(clean, "file1.txt\nfile2.txt");
    }

    #[test]
    fn extract_clean_output_single_line() {
        let output = format!("echo hi; echo \"{MARKER} exit=$?\"\nhi\n{MARKER} exit=0\n$");
        let clean = extract_clean_output(&output, MARKER);
        assert_eq!(clean, "hi");
    }

    #[test]
    fn extract_clean_output_empty_result() {
        let output = format!("true; echo \"{MARKER} exit=$?\"\n{MARKER} exit=0\n$");
        let clean = extract_clean_output(&output, MARKER);
        assert_eq!(clean, "");
    }

    #[test]
    fn extract_clean_output_no_marker() {
        let output = "echo hi\nhi\n$";
        let clean = extract_clean_output(output, MARKER);
        assert_eq!(clean, "hi\n$");
    }

    #[test]
    fn extract_clean_output_no_newline() {
        let output = format!("{MARKER} exit=0");
        let clean = extract_clean_output(&output, MARKER);
        assert_eq!(clean, "");
    }

    // --- compute_output_delta tests ---

    #[test]
    fn delta_new_data_within_buffer() {
        let buffer = b"hello world";
        let new_data = compute_output_delta(buffer, 100, 105);
        assert_eq!(new_data, b"world");
    }

    #[test]
    fn delta_exceeds_buffer() {
        let buffer = b"short";
        let new_data = compute_output_delta(buffer, 0, 1000);
        assert_eq!(new_data, buffer.as_slice());
    }

    #[test]
    fn delta_exact_buffer_size() {
        let buffer = b"exact";
        let new_data = compute_output_delta(buffer, 100, 105);
        assert_eq!(new_data, buffer.as_slice());
    }

    #[test]
    fn delta_no_change() {
        let buffer = b"anything";
        let new_data = compute_output_delta(buffer, 100, 100);
        assert!(new_data.is_empty());
    }

    #[test]
    fn delta_backwards() {
        let buffer = b"anything";
        let new_data = compute_output_delta(buffer, 200, 100);
        assert!(new_data.is_empty());
    }

    #[test]
    fn delta_one_byte() {
        let buffer = b"abcde";
        let new_data = compute_output_delta(buffer, 50, 51);
        assert_eq!(new_data, b"e");
    }

    #[test]
    fn delta_empty_buffer() {
        let buffer: &[u8] = &[];
        let new_data = compute_output_delta(buffer, 0, 10);
        assert!(new_data.is_empty());
    }
}
