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

        let output = if full_output.len() > pre_len {
            &full_output[pre_len..]
        } else {
            full_output
        };

        let marker_with_exit = format!("{marker} exit=");
        let has_marker = output.contains(&marker_with_exit) && {
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

            let mut exit_code: Option<i32> = None;
            for line in output.lines() {
                if line.contains(&marker)
                    && let Some(exit_str) = line.split("exit=").nth(1) {
                        exit_code = exit_str.trim().parse().ok();
                    }
            }

            let clean_output = {
                let after_cmd_echo = output.find('\n')
                    .map(|pos| &output[pos + 1..])
                    .unwrap_or(output);

                if let Some(pos) = after_cmd_echo.find(&marker_with_exit) {
                    let before = &after_cmd_echo[..pos];
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

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "query.output", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => result.context("Failed to connect to session")?,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Output query timed out after {}s", timeout_secs),
                }));
                std::process::exit(1);
            }
            anyhow::bail!("Output query timed out after {}s", timeout_secs);
        }
    };

    match client::unwrap_result(resp) {
        Ok(result) => {
            let output = result["output"].as_str().unwrap_or("");
            if json {
                println!("{}", serde_json::json!({
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
                println!("{}", serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
                std::process::exit(1);
            }
            anyhow::bail!("Output query failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_inject(target: &str, text: &str, enter: bool, key: Option<&str>, json: bool, timeout_secs: u64) -> Result<()> {
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

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(reg.socket_path(), "command.inject", params);
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => result.context("Failed to connect to session")?,
        Err(_) => {
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
                println!("{}", serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
                std::process::exit(1);
            }
            anyhow::bail!("Inject failed: {}", e);
        }
    }
}

pub(crate) async fn cmd_resize(target: &str, cols: u16, rows: u16, json: bool, timeout_secs: u64) -> Result<()> {
    let reg = manager::find_session(target)
        .context(format!("Session '{}' not found", target))?;

    let timeout_dur = std::time::Duration::from_secs(timeout_secs);
    let rpc_future = client::rpc_call(
        reg.socket_path(),
        "command.resize",
        serde_json::json!({ "cols": cols, "rows": rows }),
    );
    let resp = match tokio::time::timeout(timeout_dur, rpc_future).await {
        Ok(result) => result.context("Failed to connect to session")?,
        Err(_) => {
            if json {
                println!("{}", serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("Resize timed out after {}s", timeout_secs),
                }));
                std::process::exit(1);
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
                println!("{}", serde_json::json!({
                    "ok": false,
                    "target": target,
                    "error": format!("{e}"),
                }));
                std::process::exit(1);
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
                                let delta = (new_buffered - last_buffered) as usize;
                                let output = result["output"].as_str().unwrap_or("");
                                let output_bytes = output.as_bytes();

                                // delta is computed from total scrollback, but output
                                // only contains the last N bytes (e.g. 8192). When
                                // delta exceeds the returned buffer, all returned
                                // bytes are new — print the whole buffer.
                                let new_data = if delta >= output_bytes.len() {
                                    output_bytes
                                } else {
                                    &output_bytes[output_bytes.len() - delta..]
                                };

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

pub(crate) async fn cmd_mirror(target: &str, scrollback_lines: u64) -> Result<()> {
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
        "Mirroring {} ({}) — read-only. Press Ctrl+C to stop.",
        reg.display_name, reg.id
    );

    mirror_loop(stream).await
}

/// Read-only data plane mirror loop — receives Output frames, ignores everything else.
async fn mirror_loop(stream: tokio::net::UnixStream) -> Result<()> {
    let (read_half, _write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);

    // Handle Ctrl+C gracefully
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
                        // Silently ignore all other frame types (Pong, Close, etc.)
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
