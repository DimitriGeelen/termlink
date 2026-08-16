use anyhow::{Context, Result};

use termlink_session::client;
use termlink_session::codec::{FrameReader, FrameWriter};
use termlink_session::data_server;
use termlink_session::manager;

use termlink_protocol::data::{FrameFlags, FrameType};

use crate::util::{resize_payload, strip_ansi_codes, terminal_size};

/// T-2736 — why an `interact` call reached its deadline.
///
/// The root shape here is charter-correct and worth stating plainly, because it
/// determines the remedy: **TermLink models a PTY nobody is watching.** There is
/// no terminal emulator behind the session to answer a child's DSR/OSC query, so
/// a child that asks "where is the cursor?" or "what colour is the background?"
/// and then blocks for the reply will block until the deadline. That is not a
/// bug in the child and not a bug the operator can fix by retrying — but before
/// this task the message said only "Timeout after Ns waiting for command to
/// complete", which reads as "your command is slow" and sends the operator
/// looking in the wrong place.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum InteractTimeout {
    /// The child emitted a terminal query and nothing followed it. Nothing was
    /// ever going to.
    UnansweredQuery { query: &'static str },
    /// Not a single byte appeared. The command probably never ran — the shell
    /// may not be at a prompt, or the session may be in a full-screen program.
    NoOutput,
    /// Output appeared but the completion marker never did. Ordinary slowness,
    /// an interactive prompt waiting on input, or a command that never exits.
    NoMarker,
}

/// Terminal queries a child may block on, paired with a human name. Each is a
/// request for information only a real emulator can supply.
///
/// Byte patterns, not regexes: these are exact control sequences, and a literal
/// match cannot drift the way a hand-written pattern can.
const TERMINAL_QUERIES: &[(&str, &str)] = &[
    ("\x1b[6n", "CSI 6n — cursor position report (DSR)"),
    ("\x1b[14t", "CSI 14t — text area size in pixels"),
    ("\x1b[16t", "CSI 16t — character cell size in pixels"),
    ("\x1b[18t", "CSI 18t — text area size in characters"),
    ("\x1b[>c", "CSI >c — secondary device attributes (DA2)"),
    ("\x1b[?u", "CSI ?u — kitty keyboard protocol query"),
    ("\x1b[c", "CSI c — primary device attributes (DA1)"),
    ("\x1b]10;?", "OSC 10 — foreground colour query"),
    ("\x1b]11;?", "OSC 11 — background colour query"),
    ("\x1b]4;", "OSC 4 — palette colour query"),
];

/// How much trailing output to show the operator on a timeout.
const DIAGNOSIS_TAIL_BYTES: usize = 2048;

/// Classify a timed-out `interact` from the output the child produced.
///
/// Pure over the diff so every branch is testable without a PTY, a session, or a
/// sleep — the loop that calls it cannot be exercised in a unit test, which is
/// precisely why the decision lives out here.
pub(crate) fn classify_interact_timeout(diff: &str) -> InteractTimeout {
    if diff.trim().is_empty() {
        return InteractTimeout::NoOutput;
    }

    // Find the LAST query in the stream, then ask whether anything answered it.
    // Position matters: a child that queried, got a reply, and carried on is not
    // stuck, and flagging it would make this warning worthless (PL-219).
    let mut best: Option<(usize, &'static str)> = None;
    for (seq, name) in TERMINAL_QUERIES {
        if let Some(idx) = diff.rfind(seq)
            && best.is_none_or(|(prev, _)| idx > prev)
        {
            best = Some((idx + seq.len(), *name));
        }
    }

    if let Some((end, name)) = best
        && !has_meaningful_output_after(diff, end)
    {
        return InteractTimeout::UnansweredQuery { query: name };
    }

    InteractTimeout::NoMarker
}

/// Did anything the child would only print *after* getting its answer appear?
///
/// Whitespace does not count — a trailing newline is not evidence of progress.
/// Escape sequences do not count either: a child often emits a query as part of
/// a burst of setup sequences, and treating a neighbouring `ESC[?25l` as
/// "it continued" would mask exactly the case being detected.
fn has_meaningful_output_after(diff: &str, from: usize) -> bool {
    let Some(rest) = diff.get(from..) else {
        return false;
    };
    let mut in_escape = false;
    for ch in rest.chars() {
        if ch == '\x1b' {
            in_escape = true;
            continue;
        }
        if in_escape {
            // CSI/OSC sequences end at a final byte in this range; close enough
            // for "did real text follow", and deliberately conservative.
            if ch.is_ascii_alphabetic() || ch == '\x07' || ch == '~' {
                in_escape = false;
            }
            continue;
        }
        if !ch.is_whitespace() && ch != '\0' {
            return true;
        }
    }
    false
}

/// Trailing slice of the diff to show on timeout, cut on a char boundary.
pub(crate) fn tail_for_diagnosis(diff: &str) -> String {
    if diff.len() <= DIAGNOSIS_TAIL_BYTES {
        return diff.to_string();
    }
    let start = char_boundary_floor(diff, diff.len() - DIAGNOSIS_TAIL_BYTES);
    diff[start..].to_string()
}

impl InteractTimeout {
    /// Stable machine-readable discriminant for `--json` consumers.
    pub(crate) fn code(&self) -> &'static str {
        match self {
            InteractTimeout::UnansweredQuery { .. } => "unanswered-terminal-query",
            InteractTimeout::NoOutput => "no-output",
            InteractTimeout::NoMarker => "no-marker",
        }
    }

    pub(crate) fn summary(&self) -> String {
        match self {
            InteractTimeout::UnansweredQuery { query } => format!(
                "the child sent a terminal query and is waiting for a reply that will never come ({query})"
            ),
            InteractTimeout::NoOutput => {
                "the session produced no output at all — the command may never have run".to_string()
            }
            InteractTimeout::NoMarker => {
                "the command produced output but never signalled completion".to_string()
            }
        }
    }

    pub(crate) fn hint(&self) -> &'static str {
        match self {
            InteractTimeout::UnansweredQuery { .. } => {
                "TermLink drives a PTY with no terminal emulator behind it, so nothing answers \
                 DSR/OSC queries. This is by design, not a fault you can retry past. Run the \
                 program with the query disabled (many honour TERM=dumb or a --no-color / \
                 --plain flag), or use `termlink attach` where your own terminal replies."
            }
            InteractTimeout::NoOutput => {
                "Check the session is at a shell prompt with `termlink output <session>`. A \
                 full-screen program (editor, pager, TUI) will swallow the injected line \
                 without running it."
            }
            InteractTimeout::NoMarker => {
                "The command may still be running, or may be waiting on input. Raise --timeout, \
                 or inspect live with `termlink output <session>`."
            }
        }
    }
}

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

    // T-2736: retain the most recent diff so a timeout can say what the child
    // actually produced. Before this, the timeout branch reported `output: ""`
    // and a message naming only the deadline — discarding evidence the poll loop
    // had already collected and paid for.
    let mut last_diff = String::new();

    // Poll until marker appears in scrollback
    loop {
        if start.elapsed() > deadline {
            let cause = classify_interact_timeout(&last_diff);
            let elapsed_ms = start.elapsed().as_millis() as u64;
            let tail = tail_for_diagnosis(&last_diff);
            if json_output {
                super::json_error_exit(serde_json::json!({
                    "ok": false,
                    "output": tail,
                    "exit_code": null,
                    "error": format!("Timeout after {}s: {}", timeout, cause.summary()),
                    "cause": cause.code(),
                    "hint": cause.hint(),
                    "elapsed_ms": elapsed_ms,
                    "marker_found": false,
                    "bytes_captured": last_diff.len(),
                }));
            }
            anyhow::bail!(
                "Timeout after {timeout}s: {}\n\
                 Hint: {}\n\
                 Last output seen from the session ({} byte(s)):\n{}",
                cause.summary(),
                cause.hint(),
                tail.len(),
                if tail.is_empty() { "(nothing)" } else { tail.as_str() }
            );
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

        // `pre_len` is a byte offset from a *different* (earlier) snapshot.
        // Slicing this later snapshot at that raw offset can land in the middle
        // of a multi-byte UTF-8 char (emoji, box-drawing chars — routine in TUI
        // output) and panic with "byte index is not a char boundary". Walk back
        // to the nearest char boundary so the slice is always valid. The marker
        // is unique per invocation (pid:nanos), so any minor over-inclusion of
        // prior bytes is harmless to marker/exit-code extraction.
        let output = if full_output.len() > pre_len {
            &full_output[char_boundary_floor(full_output, pre_len)..]
        } else {
            full_output
        };

        // T-2736: keep the freshest diff for timeout diagnosis (see the deadline
        // branch above). Cheap — bounded by the 128KiB query window.
        last_diff.clear();
        last_diff.push_str(output);

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

/// T-2697: did `command.inject` actually write to a terminal?
///
/// The handler (`termlink-session handler.rs::handle_command_inject`) answers two
/// different things with the same RPC success:
///   * `status: "injected"`  — a PTY existed and took the bytes
///   * `status: "resolved"`  — no PTY; keys were resolved and written NOWHERE,
///                             plus a `note` naming the remedy
///
/// Only the first is an inject. Treating the second as success is a silent
/// no-op-reported-as-success, which is what `termlink inject` did until T-2694
/// caught it while building a prover for the charter's "inject keystrokes" claim.
///
/// Fail CLOSED on a missing/unknown `status`: an envelope we cannot classify must
/// not be optimistically read as delivered. Mirrors the MCP side's
/// `mcp_inject_outcome` (T-2580), whose comment already stated the rule — that fix
/// simply never reached this surface.
///
/// Pure so the status-awareness is unit-testable rather than only observable: the
/// tests below fail if the check is removed.
pub(crate) fn inject_status_is_injected(result: &serde_json::Value) -> bool {
    result["status"].as_str() == Some("injected")
}

/// T-2644: throttle policy for the interactive attach loop's inject-failure hint.
///
/// The attach loop forwards every keystroke as its own `command.inject`. Warning
/// unconditionally would print once per keypress — and the operator holding a key
/// down against a dead session is precisely when it would fire hardest, scribbling
/// over the PTY render at exactly the wrong moment. Warning only ONCE per process
/// has the opposite failure: a persistent condition (no PTY) says nothing after the
/// first line, and the operator who looked away never learns.
///
/// So: announce the START of a failure streak, then re-announce periodically while
/// it persists. `consecutive` is the count INCLUDING the current failure, and is
/// reset to 0 by any delivered inject — so a flaky link that recovers re-announces
/// on its next streak rather than staying silent for the life of the attach.
///
/// Pure so the throttle is unit-testable rather than only observable by holding a
/// key down against a dead session in a terminal (which is how it would otherwise
/// have to be verified, and therefore would not be).
pub(crate) fn should_warn_inject_failure(consecutive: u32) -> bool {
    consecutive == 1 || (consecutive > 0 && consecutive % 25 == 0)
}

/// T-2644: classify one attach-loop inject attempt — `None` when the keystrokes
/// actually landed, `Some(reason)` with an operator-facing cause when they did not.
///
/// Two distinct failure modes reach here, and only one of them is otherwise visible:
///
///   * **transport error** — the socket is gone / the session exited. The loop's
///     sibling output-poll branch also notices this within one poll interval and
///     prints "Connection lost.", so this case was already *eventually* surfaced;
///     what was lost is the keystrokes typed inside that window.
///   * **`status: "resolved"`** — the RPC SUCCEEDS. There is no PTY, so the keys
///     were resolved and written nowhere (T-2697). `query.output` keeps succeeding,
///     so the poll branch stays perfectly happy and NOTHING ever surfaces it. The
///     operator types into a live-looking session forever. This is the case that
///     was genuinely undetectable, and it is why reusing `inject_status_is_injected`
///     here matters more than the transport check.
///
/// Fails CLOSED, inheriting `inject_status_is_injected`: an envelope whose status we
/// cannot classify counts as not-delivered rather than optimistically as delivered.
pub(crate) fn attach_inject_failure_reason(
    outcome: &Result<serde_json::Value, String>,
) -> Option<String> {
    match outcome {
        Err(e) => Some(e.clone()),
        Ok(result) => {
            if inject_status_is_injected(result) {
                None
            } else {
                Some(
                    result["note"]
                        .as_str()
                        .unwrap_or("no PTY — keys were resolved but written nowhere")
                        .to_string(),
                )
            }
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
            // T-2697: the RPC succeeding is NOT the same as the keystrokes landing.
            // `command.inject` returns status:"injected" when a PTY took the bytes and
            // status:"resolved" when there is no PTY — keys resolved, nothing written,
            // still an RPC *success*. Reporting that as "Injected N bytes" is a silent
            // no-op-reported-as-success (Directive #2). T-2580 already established this
            // for the MCP surface (`mcp_inject_outcome`, with a load-bearing test) and
            // its comment states the rule outright: it "MUST NOT read 'Injected
            // successfully'". That fix was never migrated here — found by T-2694 while
            // building an inject prover, when a no-PTY session answered
            // `{"bytes_injected":18,"ok":true}` and nothing happened.
            if !inject_status_is_injected(&result) {
                let note = result["note"].as_str().unwrap_or(
                    "No PTY session — keys were resolved but never written to a terminal.",
                );
                if json {
                    super::json_error_exit(serde_json::json!({
                        "ok": false,
                        "target": target,
                        "status": result["status"].as_str().unwrap_or("unknown"),
                        "bytes_resolved": bytes,
                        "error": format!("Keys were resolved but NOT injected: {note}"),
                    }));
                }
                // Loud in text mode too — a bare non-zero exit here would be
                // indistinguishable from a crash (the T-2666 class).
                eprintln!("Not injected: {note}");
                eprintln!("  {bytes} byte(s) were resolved but never reached a terminal.");
                eprintln!("  Re-create the session with `termlink spawn --shell` (or `register --shell`).");
                anyhow::bail!("inject did not reach a PTY (status={})",
                    result["status"].as_str().unwrap_or("unknown"));
            }
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

/// Bytes that return the operator's terminal to a sane mode set on detach.
///
/// T-2732 (herdr backlog item 4). `tcsetattr` restores `termios` — the kernel
/// line discipline. It cannot touch terminal **private modes**, which live in
/// the emulator and get switched on by bytes the child wrote straight through
/// to the operator's screen. Detach from a child sitting in `vim`, `less` or
/// `htop` and, before this existed, the operator was handed back a shell still
/// on the alternate screen, still emitting escape garbage on every mouse move,
/// still wrapping every paste in `\e[200~`.
///
/// The tree could already *detect* alt screen (`PtySession::scan_alternate_screen`)
/// and had no way to leave it: a grep for these sequences found no emission
/// site in product code at all, only tests feeding the detector.
///
/// Every sequence here is a **disable**, and the set is emitted
/// unconditionally. Disabling a mode that was never enabled is a no-op in
/// terminals that implement it and ignored by those that do not; the
/// alternative — tracking which modes the child turned on — would have to be
/// perfect to be safe, and the scan only ever watched one of them. tmux,
/// screen and vim all emit their restore set unconditionally for the same
/// reason.
///
/// Deliberately NOT included: the kitty keyboard protocol pop (`CSI <u`). It
/// pops a stack this code never pushed to, so on a terminal where the
/// operator's outer application pushed an entry, emitting it would discard
/// *their* state — a restore that breaks something is worse than the leak it
/// closes. If TermLink ever pushes a kitty entry, the matching pop belongs
/// next to that push, not here.
const TERMINAL_PRIVATE_MODE_RESTORE: &[u8] = b"\
\x1b[?1006l\
\x1b[?1005l\
\x1b[?1015l\
\x1b[?1003l\
\x1b[?1002l\
\x1b[?1001l\
\x1b[?1000l\
\x1b[?2004l\
\x1b[?1004l\
\x1b[?25h\
\x1b[0m\
\x1b[?1049l\
\x1b[?1047l\
\x1b[?47l";

/// Emit [`TERMINAL_PRIVATE_MODE_RESTORE`] to the operator's terminal.
///
/// Best-effort by design: the only party who could act on a write error here
/// is the terminal we just failed to write to. Propagating it would replace
/// the attach loop's own result — the thing the operator actually asked about
/// — with a report about the cleanup of a screen that is already gone.
fn restore_terminal_private_modes() {
    use std::io::Write;
    let stdout = std::io::stdout();
    let mut out = stdout.lock();
    let _ = out.write_all(TERMINAL_PRIVATE_MODE_RESTORE);
    let _ = out.flush();
}

/// Full terminal restore for a detaching attach loop: private modes first,
/// then `termios`.
///
/// Both orderings work, but they are not equally safe to leave to a caller.
/// This is one function rather than two calls at each site specifically so the
/// order cannot drift apart between the two detach paths — T-2728 is the
/// freshest evidence in this repo that a duplicated terminal primitive
/// diverges, and it cost two identical defects living in two copies of
/// `strip_ansi_codes` for as long as both existed.
///
/// # Safety
///
/// `orig` must be a `termios` previously read from `stdin_fd`.
unsafe fn restore_terminal(stdin_fd: libc::c_int, orig: &libc::termios) {
    restore_terminal_private_modes();
    unsafe {
        libc::tcsetattr(stdin_fd, libc::TCSANOW, orig);
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

    // Restore terminal on exit — on the error return as much as the clean one.
    // `attach_loop` yields its Result rather than `?`-ing out, so a detach
    // caused by a failed loop still leaves the terminal usable (T-2732).
    let result = attach_loop(reg.socket_path(), poll_ms).await;

    unsafe {
        restore_terminal(stdin_fd, &orig_termios);
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

    // T-2644: consecutive failed injects, reset by any delivered one. Drives the
    // throttled hint so a held-down key against a dead session cannot spam the render.
    let mut inject_failures: u32 = 0;

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

                // T-2644: this was `let _ = client::rpc_call(...)` — fire-and-forget.
                // A failed inject discarded the operator's keystrokes with zero
                // feedback: they kept typing into what looked like a live session.
                // Directive #2 (no silent failures), and the sibling branches in this
                // very loop already got it right — the output-poll branch prints
                // "Connection lost." and the data-plane loop prints "Data plane write
                // error". Only this branch stayed silent.
                let outcome = match client::rpc_call(socket, "command.inject", params).await {
                    Ok(resp) => client::unwrap_result(resp),
                    Err(e) => Err(format!("{e}")),
                };
                match attach_inject_failure_reason(&outcome) {
                    None => inject_failures = 0,
                    Some(reason) => {
                        inject_failures = inject_failures.saturating_add(1);
                        if should_warn_inject_failure(inject_failures) {
                            // Raw mode gives no implicit carriage return, so a bare
                            // "\n" would stair-step the message across the PTY render.
                            // Lead AND trail with "\r\n" to start and finish at column
                            // 0 — the sibling handlers lead with "\r\n" and can skip
                            // the trailer only because they `break` immediately after.
                            // This one continues the loop, so it must land the cursor
                            // back itself.
                            eprint!(
                                "\r\n[termlink] input not delivered ({reason}) \
                                 — press Ctrl+] to detach.\r\n"
                            );
                        }
                    }
                }
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

    // Restore terminal — private modes then termios, same as the control-plane
    // attach above, through the one shared helper so they cannot drift (T-2732).
    unsafe {
        restore_terminal(stdin_fd, &orig_termios);
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

    // T-2732: `--raw` is byte passthrough — the child's output reaches the
    // operator's terminal unparsed, so it can switch on alt screen, mouse
    // reporting or bracketed paste exactly as an attach can. Mirror never
    // enters raw mode, so there is no `termios` to hand back; only the
    // emulator-side modes need clearing. The grid loop does not pass bytes
    // through, but it does paint SGR attributes, which the same set resets.
    //
    // Binding the result rather than returning it directly is what puts the
    // error exit through the restore too: a mirror that ends on a data-plane
    // error leaves the terminal no worse than one stopped with Ctrl+C.
    let result = if raw {
        mirror_loop_raw(stream).await
    } else {
        mirror_loop_grid(stream).await
    };

    restore_terminal_private_modes();

    result
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
                                let _ = grid.render_diff(&mut out);
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

/// Return the largest byte index `<= idx` that is a valid char boundary in `s`.
///
/// Used to make a byte-offset slice of a `&str` panic-safe when the offset was
/// derived from a *different* string (e.g. an earlier scrollback snapshot in
/// `cmd_interact`). Slicing `&s[idx..]` at a non-boundary offset panics; walking
/// back to the nearest boundary keeps the slice valid. `idx` is clamped to
/// `s.len()`; index 0 is always a boundary so the loop always terminates.
pub(crate) fn char_boundary_floor(s: &str, idx: usize) -> usize {
    let mut i = idx.min(s.len());
    while i > 0 && !s.is_char_boundary(i) {
        i -= 1;
    }
    i
}

#[cfg(test)]
mod tests {
    use super::*;

    // === T-2644: a failed inject in the attach loop must not vanish silently ===
    //
    // The defect: the interactive attach loop forwarded keystrokes with
    // `let _ = client::rpc_call(socket, "command.inject", params).await;` — a
    // fire-and-forget whose error was discarded. The operator kept typing into a
    // session that was taking nothing, with zero indication.
    //
    // These tests are load-bearing for BOTH halves of the fix. Delete the status
    // check and `no_pty_resolved_is_a_failure_even_though_the_rpc_succeeded` fails;
    // make the hint unconditional and `does_not_warn_on_every_keystroke` fails.

    #[test]
    fn warns_on_the_first_failure_of_a_streak() {
        assert!(should_warn_inject_failure(1));
    }

    #[test]
    fn does_not_warn_on_every_keystroke() {
        // The whole point of the throttle: an operator holding a key down against a
        // dead session must not get one line per keypress scribbled over the render.
        for n in 2..25 {
            assert!(
                !should_warn_inject_failure(n),
                "consecutive={n} should be suppressed"
            );
        }
    }

    #[test]
    fn re_announces_while_the_failure_persists() {
        // ...but it must not go silent forever either: a persistent no-PTY condition
        // that spoke only once leaves an operator who looked away none the wiser.
        assert!(should_warn_inject_failure(25));
        assert!(should_warn_inject_failure(50));
        assert!(should_warn_inject_failure(100));
    }

    #[test]
    fn zero_never_warns() {
        // 0 is the reset/"nothing has failed" state, and `% 25 == 0` would otherwise
        // make it warn — a hint fired on the success path.
        assert!(!should_warn_inject_failure(0));
    }

    #[test]
    fn delivered_inject_is_not_a_failure() {
        let ok = Ok(serde_json::json!({ "status": "injected", "bytes_len": 3 }));
        assert_eq!(attach_inject_failure_reason(&ok), None);
    }

    #[test]
    fn no_pty_resolved_is_a_failure_even_though_the_rpc_succeeded() {
        // The case nothing else in the loop can see: the RPC SUCCEEDS, `query.output`
        // keeps succeeding, so the sibling poll branch never fires "Connection lost."
        // — yet the keystrokes were written nowhere (T-2697).
        let resolved = Ok(serde_json::json!({
            "status": "resolved",
            "note": "No PTY session — keys were resolved but never written to a terminal."
        }));
        let reason = attach_inject_failure_reason(&resolved).expect("must be a failure");
        assert!(reason.contains("No PTY"), "reason should name the cause: {reason}");
    }

    #[test]
    fn transport_error_is_a_failure_and_carries_its_message() {
        let err: Result<serde_json::Value, String> =
            Err("connection refused".to_string());
        assert_eq!(
            attach_inject_failure_reason(&err),
            Some("connection refused".to_string())
        );
    }

    #[test]
    fn unclassifiable_envelope_fails_closed() {
        // Inherited from `inject_status_is_injected`: an envelope we cannot classify
        // must count as not-delivered, never optimistically as delivered.
        let missing_status = Ok(serde_json::json!({ "bytes_len": 3 }));
        assert!(attach_inject_failure_reason(&missing_status).is_some());

        let unknown_status = Ok(serde_json::json!({ "status": "sometthing-new" }));
        assert!(attach_inject_failure_reason(&unknown_status).is_some());
    }

    #[test]
    fn failure_without_a_note_still_yields_an_actionable_reason() {
        let bare = Ok(serde_json::json!({ "status": "resolved" }));
        let reason = attach_inject_failure_reason(&bare).expect("must be a failure");
        assert!(!reason.is_empty(), "an empty reason renders as an empty hint");
    }

    // === T-2736: an interact timeout must name a cause, not just a deadline ===
    //
    // The defect: on timeout the command reported `output: ""` and "Timeout after
    // Ns waiting for command to complete" — it discarded the diff it had already
    // collected, and its message read as "your command is slow" even when the
    // real cause was a child blocked on a query nothing was ever going to answer.

    #[test]
    fn unanswered_cursor_position_query_is_named() {
        // The canonical case: a program asks where the cursor is and waits.
        let diff = "some setup\n\x1b[6n";
        assert_eq!(
            classify_interact_timeout(diff),
            InteractTimeout::UnansweredQuery { query: "CSI 6n — cursor position report (DSR)" },
            "a trailing DSR must be reported as the cause, not hidden behind 'slow command'"
        );
    }

    #[test]
    fn unanswered_background_colour_query_is_named() {
        let diff = "\x1b]11;?";
        assert!(matches!(
            classify_interact_timeout(diff),
            InteractTimeout::UnansweredQuery { .. }
        ));
    }

    #[test]
    fn a_query_that_was_answered_and_moved_on_is_not_flagged() {
        // PL-219: this is the common case. A child that queried, got its reply
        // from a real terminal earlier in the pipeline, and carried on producing
        // output is NOT stuck on the query — flagging it would train the operator
        // to ignore the warning by the time it is true.
        let diff = "\x1b[6n\x1b[12;40Rbuilding project...\nstill going";
        assert_eq!(
            classify_interact_timeout(diff),
            InteractTimeout::NoMarker,
            "real output after the query proves the child continued past it"
        );
    }

    #[test]
    fn trailing_whitespace_after_a_query_is_not_progress() {
        // A newline is not evidence the child got its answer.
        let diff = "\x1b[6n\n  \n";
        assert!(matches!(
            classify_interact_timeout(diff),
            InteractTimeout::UnansweredQuery { .. }
        ));
    }

    #[test]
    fn neighbouring_escape_sequences_do_not_count_as_progress() {
        // Queries usually ship inside a burst of setup sequences. If an adjacent
        // `ESC[?25l` counted as "it continued", the detector would miss the exact
        // case it exists for.
        let diff = "\x1b[6n\x1b[?25l\x1b[2J";
        assert!(matches!(
            classify_interact_timeout(diff),
            InteractTimeout::UnansweredQuery { .. }
        ));
    }

    #[test]
    fn empty_diff_is_no_output_not_a_query() {
        assert_eq!(classify_interact_timeout(""), InteractTimeout::NoOutput);
        assert_eq!(classify_interact_timeout("   \n\t "), InteractTimeout::NoOutput);
    }

    #[test]
    fn ordinary_slow_command_is_no_marker() {
        assert_eq!(
            classify_interact_timeout("compiling...\nlinking...\n"),
            InteractTimeout::NoMarker
        );
    }

    #[test]
    fn the_last_query_wins_when_several_appear() {
        // Two queries, only the second unanswered — the operator needs the one
        // actually blocking, not the first one in the buffer.
        let diff = "\x1b[6n\x1b[12;40Rok\n\x1b]11;?";
        assert_eq!(
            classify_interact_timeout(diff),
            InteractTimeout::UnansweredQuery { query: "OSC 11 — background colour query" }
        );
    }

    #[test]
    fn every_cause_carries_a_distinct_code_and_a_nonempty_hint() {
        // Directive #2/#3: a named cause with no remedy is only half an answer.
        let causes = [
            InteractTimeout::UnansweredQuery { query: "x" },
            InteractTimeout::NoOutput,
            InteractTimeout::NoMarker,
        ];
        let mut codes = Vec::new();
        for c in &causes {
            assert!(!c.hint().is_empty(), "{:?} must carry an actionable hint", c);
            assert!(!c.summary().is_empty());
            codes.push(c.code());
        }
        codes.sort_unstable();
        codes.dedup();
        assert_eq!(codes.len(), 3, "each cause needs its own machine-readable code");
    }

    #[test]
    fn query_hint_says_the_silence_is_by_design() {
        // The operator must not be sent hunting for a fault that is not there:
        // nothing is behind this PTY to answer, and that is charter-correct.
        let hint = InteractTimeout::UnansweredQuery { query: "x" }.hint();
        assert!(
            hint.contains("no terminal emulator") && hint.contains("by design"),
            "the hint must say the silence is intentional, not a fault to retry past: {hint}"
        );
    }

    #[test]
    fn diagnosis_tail_is_bounded_and_utf8_safe() {
        // The tail is cut from a byte offset, so it must land on a char boundary
        // — the same class T-2733 fixed in scrollback.
        let long = "é".repeat(4000);
        let tail = tail_for_diagnosis(&long);
        assert!(tail.len() <= DIAGNOSIS_TAIL_BYTES + 4);
        assert!(tail.chars().all(|c| c == 'é'), "tail must not split a character");
        let short = "short output";
        assert_eq!(tail_for_diagnosis(short), short);
    }

    // === T-2732 LOAD-BEARING: detach must not leave the terminal in a child's mode ===
    //
    // `termios` is not the terminal's whole state. A child that enabled alt screen,
    // mouse reporting or bracketed paste leaves those switched on in the *emulator*,
    // where `tcsetattr` cannot reach — so before T-2732 a detach from `vim` handed
    // the operator back a shell on the alternate screen emitting escape garbage on
    // every mouse move. The tree could already detect alt screen and had no way to
    // leave it.
    //
    // These pin the byte content, so an edit that silently drops a mode fails here
    // rather than in someone's terminal. Deleting any sequence from
    // TERMINAL_PRIVATE_MODE_RESTORE makes the first test fail.

    #[test]
    fn private_mode_restore_disables_every_mode_a_child_can_leak() {
        let s = std::str::from_utf8(TERMINAL_PRIVATE_MODE_RESTORE)
            .expect("restore sequence must be valid UTF-8");

        // Alternate screen — all three variants. `?1049` is the modern one, but
        // `?1047` and `?47` are what older curses apps actually emit, and a
        // terminal left on the alt screen by `?47h` is not returned by `?1049l`.
        // T-2731's sibling finding: handling only the variant you expected is
        // indistinguishable from handling none of them, from the operator's seat.
        for seq in ["\x1b[?1049l", "\x1b[?1047l", "\x1b[?47l"] {
            assert!(s.contains(seq), "alt-screen exit {seq:?} missing from restore set");
        }

        // Mouse reporting: the click/drag/motion modes and the three encodings.
        // Leaving any one on means every pointer move types escape bytes into
        // the operator's next command line.
        for seq in [
            "\x1b[?1000l",
            "\x1b[?1001l",
            "\x1b[?1002l",
            "\x1b[?1003l",
            "\x1b[?1005l",
            "\x1b[?1006l",
            "\x1b[?1015l",
        ] {
            assert!(s.contains(seq), "mouse mode {seq:?} missing from restore set");
        }

        // Bracketed paste, focus reporting, cursor visibility. A child that hid
        // the cursor (`?25l`) and died leaves the operator typing blind.
        for seq in ["\x1b[?2004l", "\x1b[?1004l", "\x1b[?25h"] {
            assert!(s.contains(seq), "{seq:?} missing from restore set");
        }

        // SGR reset: not a private mode, same leak. A child killed mid-colour
        // leaves the prompt painted in whatever it was using.
        assert!(s.contains("\x1b[0m"), "SGR reset missing from restore set");
    }

    #[test]
    fn private_mode_restore_is_all_disables_never_an_enable() {
        // The whole set is unconditional, which is only safe because every
        // sequence turns something OFF. One stray `h`-terminated private-mode
        // sequence here would switch a mode ON in the terminal of every
        // operator who detaches — a restore that causes the fault it prevents.
        // `?25h` (show cursor) is the sole intentional enable.
        let s = std::str::from_utf8(TERMINAL_PRIVATE_MODE_RESTORE).expect("valid UTF-8");
        for part in s.split('\x1b').filter(|p| p.starts_with("[?")) {
            assert!(
                part.ends_with('l') || part == "[?25h",
                "private-mode sequence ESC{part:?} is not a disable"
            );
        }
    }

    #[test]
    fn private_mode_restore_leaves_alt_screen_last() {
        // Ordering matters for what the operator sees: disabling mouse/paste
        // while still on the alt screen keeps that churn off the restored
        // scrollback, and the alt-screen exit is what redraws their shell. If a
        // future edit appends a sequence after the alt-screen exits, it lands
        // on the *restored* screen instead.
        let s = std::str::from_utf8(TERMINAL_PRIVATE_MODE_RESTORE).expect("valid UTF-8");
        let last_alt = s.rfind("\x1b[?47l").expect("?47l present");
        let first_mouse = s.find("\x1b[?1006l").expect("?1006l present");
        assert!(first_mouse < last_alt, "mouse disables must precede the alt-screen exit");
        assert!(s.ends_with("\x1b[?47l"), "alt-screen exit must be the final sequence");
    }

    // === T-2697 LOAD-BEARING: `inject` must not report success for a no-op ===
    //
    // Sibling of the MCP side's `inject_no_pty_is_not_reported_as_success` (T-2580).
    // The handler answers status:"resolved" — an RPC SUCCESS — when there is no PTY
    // and the keys went nowhere. Removing the status check in cmd_inject makes these
    // fail. Found by T-2694 while building an inject prover: a no-PTY session
    // answered `{"bytes_injected":18,"ok":true}` and nothing had happened.

    #[test]
    fn inject_status_injected_is_a_real_inject() {
        let injected = serde_json::json!({"status": "injected", "bytes_len": 5});
        assert!(
            inject_status_is_injected(&injected),
            "status:injected is the only shape that means the bytes reached a terminal"
        );
    }

    #[test]
    fn inject_status_resolved_is_not_success() {
        // The exact envelope a no-PTY session returns.
        let resolved = serde_json::json!({
            "status": "resolved",
            "bytes_len": 18,
            "note": "No PTY session. Use `register --shell` for PTY-backed injection.",
        });
        assert!(
            !inject_status_is_injected(&resolved),
            "status:resolved means keys were resolved and written NOWHERE — never success"
        );
    }

    #[test]
    fn inject_missing_status_fails_closed() {
        // An envelope we cannot classify must not be optimistically read as
        // delivered — the same fail-closed rule the exec truncated-check uses.
        let bare = serde_json::json!({"bytes_len": 3});
        assert!(!inject_status_is_injected(&bare), "absent status must not read as injected");
        let unknown = serde_json::json!({"status": "queued", "bytes_len": 3});
        assert!(!inject_status_is_injected(&unknown), "unknown status must not read as injected");
        let wrong_type = serde_json::json!({"status": 1, "bytes_len": 3});
        assert!(!inject_status_is_injected(&wrong_type), "non-string status must not read as injected");
    }

    // --- char_boundary_floor / interact-slice safety tests ---

    #[test]
    fn char_boundary_floor_ascii_is_identity() {
        let s = "hello world";
        assert_eq!(char_boundary_floor(s, 5), 5);
    }

    #[test]
    fn char_boundary_floor_walks_back_into_multibyte() {
        // "a" + '€' (3 bytes: E2 82 AC) + "b". Byte offset 2 is mid-'€'.
        let s = "a€b";
        assert_eq!(s.len(), 5);
        // offset 2 and 3 are inside the euro sign → floor to 1 (the '€' start).
        assert_eq!(char_boundary_floor(s, 2), 1);
        assert_eq!(char_boundary_floor(s, 3), 1);
        assert_eq!(char_boundary_floor(s, 4), 4); // 'b' start is a boundary
    }

    #[test]
    fn char_boundary_floor_clamps_past_end() {
        let s = "hi";
        assert_eq!(char_boundary_floor(s, 99), 2);
    }

    #[test]
    fn interact_slice_does_not_panic_mid_multibyte() {
        // Reproduces the cmd_interact scrollback-diff scenario: pre_len is a byte
        // length from an earlier snapshot that lands mid-UTF-8-char in the later
        // one. The raw `&full_output[pre_len..]` slice would panic here; the
        // char-boundary floor makes it safe. LOAD-BEARING: revert the fix (slice
        // at raw `pre_len`) and this test panics.
        let full_output = "emoji 🎉 then more output"; // 🎉 is 4 bytes
        let emoji_start = full_output.find('🎉').unwrap();
        let pre_len = emoji_start + 2; // mid-emoji byte offset
        assert!(!full_output.is_char_boundary(pre_len));
        let slice = &full_output[char_boundary_floor(full_output, pre_len)..];
        assert!(slice.starts_with('🎉'));
    }

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
