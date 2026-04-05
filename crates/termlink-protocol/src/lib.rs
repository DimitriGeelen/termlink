pub mod control;
pub mod data;
pub mod error;
pub mod events;
pub mod jsonrpc;
pub mod transport;

pub use error::ProtocolError;
pub use transport::TransportAddr;

/// Format a Unix timestamp string (e.g. "1774791796Z") as a human-readable age.
///
/// Returns "5s", "3m", "2h", or "7d" depending on elapsed time.
/// Returns "?" for unparseable timestamps, "0s" for future timestamps.
/// Protocol version for the data plane binary frames.
pub const DATA_PLANE_VERSION: u8 = 1;

/// Magic bytes for data plane frame sync: "TL" (0x54, 0x4C).
pub const FRAME_MAGIC: [u8; 2] = [0x54, 0x4C];

/// Fixed header size for data plane frames (including magic).
pub const FRAME_HEADER_SIZE: usize = 22;

/// Maximum payload size: 16 MiB.
pub const MAX_PAYLOAD_SIZE: u32 = 16 * 1024 * 1024;

/// Shell-escape a string for safe embedding in `sh -c` commands.
///
/// Safe characters (alphanumeric + common path chars) pass through unchanged.
/// Everything else gets wrapped in single quotes with embedded quotes escaped.
/// Empty strings return `''`.
pub fn shell_escape(s: &str) -> String {
    if s.is_empty() {
        return "''".to_string();
    }
    if s.chars().all(|c| {
        c.is_ascii_alphanumeric() || c == '_' || c == '-' || c == '.' || c == '/' || c == ':'
    }) {
        return s.to_string();
    }
    format!("'{}'", s.replace('\'', "'\\''"))
}

pub fn format_age(timestamp_str: &str) -> String {
    let ts_str = timestamp_str.trim_end_matches('Z');
    let ts: u64 = match ts_str.parse() {
        Ok(v) => v,
        Err(_) => return "?".to_string(),
    };
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    if ts > now {
        return "0s".to_string();
    }
    let diff = now - ts;
    if diff < 60 {
        format!("{}s", diff)
    } else if diff < 3600 {
        format!("{}m", diff / 60)
    } else if diff < 86400 {
        format!("{}h", diff / 3600)
    } else {
        format!("{}d", diff / 86400)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn format_age_seconds() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_age(&format!("{}Z", now - 30));
        assert!(result.ends_with('s'), "Expected seconds: {result}");
    }

    #[test]
    fn format_age_minutes() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_age(&format!("{}Z", now - 300));
        assert_eq!(result, "5m");
    }

    #[test]
    fn format_age_hours() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_age(&format!("{}Z", now - 7200));
        assert_eq!(result, "2h");
    }

    #[test]
    fn format_age_days() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let result = format_age(&format!("{}Z", now - 172800));
        assert_eq!(result, "2d");
    }

    #[test]
    fn format_age_invalid() {
        assert_eq!(format_age("not-a-number"), "?");
        assert_eq!(format_age(""), "?");
    }

    #[test]
    fn format_age_future_timestamp() {
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        assert_eq!(format_age(&format!("{}Z", now + 1000)), "0s");
    }

    #[test]
    fn shell_escape_safe_string() {
        assert_eq!(shell_escape("hello"), "hello");
        assert_eq!(shell_escape("/tmp/foo-bar_baz.txt"), "/tmp/foo-bar_baz.txt");
        assert_eq!(shell_escape("abc123"), "abc123");
        assert_eq!(shell_escape("host:port"), "host:port");
    }

    #[test]
    fn shell_escape_special_chars() {
        assert_eq!(shell_escape("hello world"), "'hello world'");
        assert_eq!(shell_escape("a;b"), "'a;b'");
        assert_eq!(shell_escape("$(cmd)"), "'$(cmd)'");
    }

    #[test]
    fn shell_escape_single_quotes() {
        assert_eq!(shell_escape("it's"), "'it'\\''s'");
        assert_eq!(shell_escape("a'b'c"), "'a'\\''b'\\''c'");
    }

    #[test]
    fn shell_escape_empty() {
        assert_eq!(shell_escape(""), "''");
    }
}
