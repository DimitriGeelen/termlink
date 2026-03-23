use std::io::IsTerminal;

/// Strip ANSI escape sequences from a string.
pub(crate) fn strip_ansi_codes(s: &str) -> String {
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

pub(crate) fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        format!("{}…", &s[..max - 1])
    }
}

pub(crate) fn parse_signal(s: &str) -> Option<i32> {
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

/// Generate a request ID for agent protocol correlation.
pub(crate) fn generate_request_id() -> String {
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis();
    format!("req-{}-{}", std::process::id(), ts)
}

/// Get the current terminal size (cols, rows).
pub(crate) fn terminal_size() -> (u16, u16) {
    let mut ws: libc::winsize = unsafe { std::mem::zeroed() };
    let ret = unsafe { libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) };
    if ret == 0 && ws.ws_col > 0 && ws.ws_row > 0 {
        (ws.ws_col, ws.ws_row)
    } else {
        (80, 24) // sensible default
    }
}

/// Encode terminal dimensions as a 4-byte Resize payload (big-endian cols + rows).
pub(crate) fn resize_payload(cols: u16, rows: u16) -> [u8; 4] {
    let mut buf = [0u8; 4];
    buf[0..2].copy_from_slice(&cols.to_be_bytes());
    buf[2..4].copy_from_slice(&rows.to_be_bytes());
    buf
}

/// Escape a string for use in a shell command.
pub(crate) fn shell_escape(s: &str) -> String {
    if s.contains(|c: char| c.is_whitespace() || c == '\'' || c == '"' || c == '\\' || c == '$') {
        format!("'{}'", s.replace('\'', "'\\''"))
    } else {
        s.to_string()
    }
}

/// Resolve a session target: if provided, return it; if None, prompt the user
/// to pick from available sessions (interactive picker).
pub(crate) fn resolve_target(target: Option<String>) -> anyhow::Result<String> {
    if let Some(t) = target {
        return Ok(t);
    }
    pick_session()
}

/// Interactive session picker — lists available sessions and lets the user choose.
/// Auto-selects if only one session exists. Errors if no sessions or stdin is not a TTY.
fn pick_session() -> anyhow::Result<String> {
    use termlink_session::manager;

    if !std::io::stdin().is_terminal() {
        anyhow::bail!("No target specified and stdin is not a terminal (cannot prompt)");
    }

    let sessions = manager::list_sessions(false)
        .map_err(|e| anyhow::anyhow!("Failed to list sessions: {}", e))?;

    if sessions.is_empty() {
        anyhow::bail!("No active sessions found. Register one with: termlink register --name <name> --shell");
    }

    if sessions.len() == 1 {
        let s = &sessions[0];
        eprintln!(
            "Auto-selecting: {} ({})",
            s.display_name, s.id
        );
        return Ok(s.display_name.clone());
    }

    // Print numbered list
    eprintln!("Available sessions:");
    eprintln!(
        "  {:<4} {:<20} {:<12} {:<10} {}",
        "#", "NAME", "STATE", "PID", "TAGS"
    );
    eprintln!("  {}", "-".repeat(60));
    for (i, s) in sessions.iter().enumerate() {
        let tags = if s.tags.is_empty() {
            String::new()
        } else {
            s.tags.join(", ")
        };
        eprintln!(
            "  {:<4} {:<20} {:<12} {:<10} {}",
            i + 1,
            truncate(&s.display_name, 19),
            format!("{:?}", s.state).to_lowercase(),
            s.pid,
            tags
        );
    }
    eprintln!();
    eprint!("Select session [1-{}]: ", sessions.len());

    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .map_err(|e| anyhow::anyhow!("Failed to read input: {}", e))?;

    let choice: usize = input
        .trim()
        .parse()
        .map_err(|_| anyhow::anyhow!("Invalid selection: '{}'", input.trim()))?;

    if choice < 1 || choice > sessions.len() {
        anyhow::bail!("Selection out of range: {} (expected 1-{})", choice, sessions.len());
    }

    let selected = &sessions[choice - 1];
    eprintln!("→ {} ({})", selected.display_name, selected.id);
    Ok(selected.display_name.clone())
}

/// Default chunk size for file transfers (48KB raw → ~64KB base64).
pub(crate) const DEFAULT_CHUNK_SIZE: usize = 49152;

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

    // --- truncate tests ---

    #[test]
    fn truncate_short_string_unchanged() {
        assert_eq!(truncate("hello", 10), "hello");
    }

    #[test]
    fn truncate_exact_length_unchanged() {
        assert_eq!(truncate("hello", 5), "hello");
    }

    #[test]
    fn truncate_long_string_with_ellipsis() {
        let result = truncate("hello world", 5);
        assert_eq!(result, "hell…");
        assert_eq!(result.chars().count(), 5);
    }

    #[test]
    fn truncate_empty_string() {
        assert_eq!(truncate("", 10), "");
    }

    // --- parse_signal tests ---

    #[test]
    fn parse_signal_numeric() {
        assert_eq!(parse_signal("15"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("9"), Some(libc::SIGKILL));
        assert_eq!(parse_signal("2"), Some(libc::SIGINT));
    }

    #[test]
    fn parse_signal_named() {
        assert_eq!(parse_signal("TERM"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("KILL"), Some(libc::SIGKILL));
        assert_eq!(parse_signal("INT"), Some(libc::SIGINT));
        assert_eq!(parse_signal("HUP"), Some(libc::SIGHUP));
        assert_eq!(parse_signal("QUIT"), Some(libc::SIGQUIT));
        assert_eq!(parse_signal("USR1"), Some(libc::SIGUSR1));
        assert_eq!(parse_signal("USR2"), Some(libc::SIGUSR2));
        assert_eq!(parse_signal("STOP"), Some(libc::SIGSTOP));
        assert_eq!(parse_signal("CONT"), Some(libc::SIGCONT));
    }

    #[test]
    fn parse_signal_with_sig_prefix() {
        assert_eq!(parse_signal("SIGTERM"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("SIGKILL"), Some(libc::SIGKILL));
    }

    #[test]
    fn parse_signal_case_insensitive() {
        assert_eq!(parse_signal("term"), Some(libc::SIGTERM));
        assert_eq!(parse_signal("sigkill"), Some(libc::SIGKILL));
        assert_eq!(parse_signal("Hup"), Some(libc::SIGHUP));
    }

    #[test]
    fn parse_signal_invalid() {
        assert_eq!(parse_signal("INVALID"), None);
        assert_eq!(parse_signal(""), None);
        assert_eq!(parse_signal("SIGFOO"), None);
    }

    // --- shell_escape tests ---

    #[test]
    fn shell_escape_safe_string() {
        assert_eq!(shell_escape("hello"), "hello");
        assert_eq!(shell_escape("foo-bar_baz.txt"), "foo-bar_baz.txt");
    }

    #[test]
    fn shell_escape_whitespace() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
        assert_eq!(shell_escape("a\tb"), "'a\tb'");
    }

    #[test]
    fn shell_escape_single_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
    }

    #[test]
    fn shell_escape_special_chars() {
        assert_eq!(shell_escape("$HOME"), "'$HOME'");
        assert_eq!(shell_escape("a\\b"), "'a\\b'");
        assert_eq!(shell_escape("a\"b"), "'a\"b'");
    }

    // --- resize_payload tests ---

    #[test]
    fn resize_payload_standard_terminal() {
        let buf = resize_payload(80, 24);
        // 80 = 0x0050, 24 = 0x0018
        assert_eq!(buf, [0x00, 0x50, 0x00, 0x18]);
    }

    #[test]
    fn resize_payload_large_terminal() {
        let buf = resize_payload(300, 100);
        // 300 = 0x012C, 100 = 0x0064
        assert_eq!(buf, [0x01, 0x2C, 0x00, 0x64]);
    }

    #[test]
    fn resize_payload_roundtrip() {
        let cols: u16 = 132;
        let rows: u16 = 43;
        let buf = resize_payload(cols, rows);
        let decoded_cols = u16::from_be_bytes([buf[0], buf[1]]);
        let decoded_rows = u16::from_be_bytes([buf[2], buf[3]]);
        assert_eq!(decoded_cols, cols);
        assert_eq!(decoded_rows, rows);
    }

    // --- generate_request_id tests ---

    #[test]
    fn generate_request_id_format() {
        let id = generate_request_id();
        assert!(id.starts_with("req-"), "Expected 'req-' prefix, got: {}", id);
        let parts: Vec<&str> = id.splitn(3, '-').collect();
        assert_eq!(parts.len(), 3, "Expected req-PID-TIMESTAMP format");
        assert!(parts[1].parse::<u32>().is_ok(), "PID should be numeric");
        assert!(parts[2].parse::<u128>().is_ok(), "Timestamp should be numeric");
    }

    #[test]
    fn generate_request_id_unique_with_delay() {
        let id1 = generate_request_id();
        std::thread::sleep(std::time::Duration::from_millis(2));
        let id2 = generate_request_id();
        assert_ne!(id1, id2, "Request IDs with delay should differ");
    }
}
