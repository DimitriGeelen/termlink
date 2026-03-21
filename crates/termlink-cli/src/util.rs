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
}
