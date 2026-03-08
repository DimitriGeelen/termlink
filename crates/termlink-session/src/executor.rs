use std::time::Duration;

use serde::{Deserialize, Serialize};
use tokio::process::Command;

/// Result of executing a shell command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecResult {
    pub exit_code: i32,
    pub stdout: String,
    pub stderr: String,
}

/// Execute a shell command with optional timeout, working directory, and env vars.
pub async fn execute(
    command: &str,
    cwd: Option<&str>,
    env: Option<&std::collections::HashMap<String, String>>,
    timeout: Option<Duration>,
) -> Result<ExecResult, ExecError> {
    let mut cmd = Command::new("sh");
    cmd.arg("-c").arg(command);

    if let Some(dir) = cwd {
        cmd.current_dir(dir);
    }

    if let Some(env_vars) = env {
        for (k, v) in env_vars {
            cmd.env(k, v);
        }
    }

    cmd.stdout(std::process::Stdio::piped());
    cmd.stderr(std::process::Stdio::piped());

    let timeout_dur = timeout.unwrap_or(Duration::from_secs(30));

    let output = tokio::time::timeout(timeout_dur, cmd.output())
        .await
        .map_err(|_| ExecError::Timeout(timeout_dur))?
        .map_err(ExecError::Spawn)?;

    Ok(ExecResult {
        exit_code: output.status.code().unwrap_or(-1),
        stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
        stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
    })
}

/// Resolve a symbolic key name to its raw byte sequence.
pub fn resolve_key(name: &str) -> Option<Vec<u8>> {
    match name {
        // Control characters
        "Ctrl+A" | "ctrl+a" => Some(vec![0x01]),
        "Ctrl+B" | "ctrl+b" => Some(vec![0x02]),
        "Ctrl+C" | "ctrl+c" => Some(vec![0x03]),
        "Ctrl+D" | "ctrl+d" => Some(vec![0x04]),
        "Ctrl+E" | "ctrl+e" => Some(vec![0x05]),
        "Ctrl+F" | "ctrl+f" => Some(vec![0x06]),
        "Ctrl+G" | "ctrl+g" => Some(vec![0x07]),
        "Ctrl+H" | "ctrl+h" => Some(vec![0x08]),
        "Ctrl+K" | "ctrl+k" => Some(vec![0x0B]),
        "Ctrl+L" | "ctrl+l" => Some(vec![0x0C]),
        "Ctrl+N" | "ctrl+n" => Some(vec![0x0E]),
        "Ctrl+O" | "ctrl+o" => Some(vec![0x0F]),
        "Ctrl+P" | "ctrl+p" => Some(vec![0x10]),
        "Ctrl+Q" | "ctrl+q" => Some(vec![0x11]),
        "Ctrl+R" | "ctrl+r" => Some(vec![0x12]),
        "Ctrl+S" | "ctrl+s" => Some(vec![0x13]),
        "Ctrl+T" | "ctrl+t" => Some(vec![0x14]),
        "Ctrl+U" | "ctrl+u" => Some(vec![0x15]),
        "Ctrl+V" | "ctrl+v" => Some(vec![0x16]),
        "Ctrl+W" | "ctrl+w" => Some(vec![0x17]),
        "Ctrl+X" | "ctrl+x" => Some(vec![0x18]),
        "Ctrl+Y" | "ctrl+y" => Some(vec![0x19]),
        "Ctrl+Z" | "ctrl+z" => Some(vec![0x1A]),
        "Ctrl+\\" | "ctrl+\\" => Some(vec![0x1C]),

        // Special keys
        "Enter" | "enter" | "Return" | "return" => Some(vec![0x0D]),
        "Tab" | "tab" => Some(vec![0x09]),
        "Backspace" | "backspace" => Some(vec![0x7F]),
        "Escape" | "escape" | "Esc" | "esc" => Some(vec![0x1B]),
        "Delete" | "delete" | "Del" | "del" => Some(vec![0x1B, 0x5B, 0x33, 0x7E]),
        "Space" | "space" => Some(vec![0x20]),

        // Arrow keys (ANSI)
        "Up" | "up" => Some(vec![0x1B, 0x5B, 0x41]),
        "Down" | "down" => Some(vec![0x1B, 0x5B, 0x42]),
        "Right" | "right" => Some(vec![0x1B, 0x5B, 0x43]),
        "Left" | "left" => Some(vec![0x1B, 0x5B, 0x44]),

        // Home/End
        "Home" | "home" => Some(vec![0x1B, 0x5B, 0x48]),
        "End" | "end" => Some(vec![0x1B, 0x5B, 0x46]),

        _ => None,
    }
}

/// Resolve a KeyEntry (from T-005 protocol) to raw bytes.
pub fn resolve_key_entry(entry: &termlink_protocol::control::KeyEntry) -> Result<Vec<u8>, String> {
    match entry {
        termlink_protocol::control::KeyEntry::Text(text) => Ok(text.as_bytes().to_vec()),
        termlink_protocol::control::KeyEntry::Key(name) => {
            resolve_key(name).ok_or_else(|| format!("Unknown key: {name}"))
        }
        termlink_protocol::control::KeyEntry::Raw(b64) => {
            // Simple base64 decode (no dependency needed for basic cases)
            base64_decode(b64).map_err(|e| format!("Invalid base64: {e}"))
        }
    }
}

/// Resolve a sequence of KeyEntries to a single byte buffer.
pub fn resolve_keys(entries: &[termlink_protocol::control::KeyEntry]) -> Result<Vec<u8>, String> {
    let mut buf = Vec::new();
    for entry in entries {
        buf.extend(resolve_key_entry(entry)?);
    }
    Ok(buf)
}

/// Send a POSIX signal to a process.
pub fn send_signal(pid: u32, signal: i32) -> Result<(), ExecError> {
    let ret = unsafe { libc::kill(pid as i32, signal) };
    if ret == 0 {
        Ok(())
    } else {
        Err(ExecError::Signal(
            signal,
            std::io::Error::last_os_error(),
        ))
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExecError {
    #[error("command timed out after {0:?}")]
    Timeout(Duration),

    #[error("failed to spawn command: {0}")]
    Spawn(std::io::Error),

    #[error("failed to send signal {0}: {1}")]
    Signal(i32, std::io::Error),
}

/// Minimal base64 decoder (avoids adding a dependency for this).
fn base64_decode(input: &str) -> Result<Vec<u8>, &'static str> {
    let input = input.trim_end_matches('=');
    let mut buf = Vec::with_capacity(input.len() * 3 / 4);
    let mut accum: u32 = 0;
    let mut bits: u32 = 0;

    for c in input.bytes() {
        let val = match c {
            b'A'..=b'Z' => c - b'A',
            b'a'..=b'z' => c - b'a' + 26,
            b'0'..=b'9' => c - b'0' + 52,
            b'+' => 62,
            b'/' => 63,
            b'\n' | b'\r' | b' ' => continue,
            _ => return Err("invalid base64 character"),
        };
        accum = (accum << 6) | val as u32;
        bits += 6;
        if bits >= 8 {
            bits -= 8;
            buf.push((accum >> bits) as u8);
            accum &= (1 << bits) - 1;
        }
    }
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use termlink_protocol::control::KeyEntry;

    #[tokio::test]
    async fn execute_echo() {
        let result = execute("echo hello", None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stdout.trim(), "hello");
        assert!(result.stderr.is_empty());
    }

    #[tokio::test]
    async fn execute_with_cwd() {
        let result = execute("pwd", Some("/tmp"), None, None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        // macOS resolves /tmp to /private/tmp
        assert!(
            result.stdout.trim() == "/tmp" || result.stdout.trim() == "/private/tmp"
        );
    }

    #[tokio::test]
    async fn execute_with_env() {
        let mut env = std::collections::HashMap::new();
        env.insert("MY_VAR".into(), "my_value".into());
        let result = execute("echo $MY_VAR", None, Some(&env), None)
            .await
            .unwrap();
        assert_eq!(result.stdout.trim(), "my_value");
    }

    #[tokio::test]
    async fn execute_captures_stderr() {
        let result = execute("echo err >&2", None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.stderr.trim(), "err");
    }

    #[tokio::test]
    async fn execute_nonzero_exit() {
        let result = execute("exit 42", None, None, None).await.unwrap();
        assert_eq!(result.exit_code, 42);
    }

    #[tokio::test]
    async fn execute_timeout() {
        let result = execute(
            "sleep 10",
            None,
            None,
            Some(Duration::from_millis(100)),
        )
        .await;
        assert!(matches!(result, Err(ExecError::Timeout(_))));
    }

    #[test]
    fn resolve_known_keys() {
        assert_eq!(resolve_key("Enter"), Some(vec![0x0D]));
        assert_eq!(resolve_key("Ctrl+C"), Some(vec![0x03]));
        assert_eq!(resolve_key("Up"), Some(vec![0x1B, 0x5B, 0x41]));
        assert_eq!(resolve_key("Tab"), Some(vec![0x09]));
        assert!(resolve_key("UnknownKey").is_none());
    }

    #[test]
    fn resolve_key_entries() {
        let entries = vec![
            KeyEntry::Text("ls -la".into()),
            KeyEntry::Key("Enter".into()),
        ];
        let bytes = resolve_keys(&entries).unwrap();
        assert_eq!(&bytes[..6], b"ls -la");
        assert_eq!(bytes[6], 0x0D); // Enter
    }

    #[test]
    fn resolve_raw_base64() {
        // 0x03 (Ctrl+C) = "Aw==" in base64
        let entry = KeyEntry::Raw("Aw==".into());
        let bytes = resolve_key_entry(&entry).unwrap();
        assert_eq!(bytes, vec![0x03]);
    }

    #[test]
    fn base64_decode_basic() {
        assert_eq!(base64_decode("SGVsbG8=").unwrap(), b"Hello");
        assert_eq!(base64_decode("Aw==").unwrap(), vec![0x03]);
    }

    #[test]
    fn send_signal_to_self() {
        // Signal 0 checks process existence without actually sending a signal
        send_signal(std::process::id(), 0).unwrap();
    }

    #[test]
    fn send_signal_to_nonexistent() {
        let result = send_signal(4_000_000, 0);
        assert!(result.is_err());
    }
}
