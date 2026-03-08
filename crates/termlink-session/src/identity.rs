use serde::{Deserialize, Serialize};

/// Unique session identifier: `tl-{random8}` using base32 alphabet.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct SessionId(String);

const BASE32_ALPHABET: &[u8] = b"abcdefghijklmnopqrstuvwxyz234567";

impl SessionId {
    /// Generate a new random session ID.
    pub fn generate() -> Self {
        let mut id = String::with_capacity(11); // "tl-" + 8 chars
        id.push_str("tl-");

        // Use thread-local RNG for randomness
        let mut bytes = [0u8; 5]; // 40 bits = 8 base32 chars
        getrandom(&mut bytes);

        // Encode 5 bytes as 8 base32 characters
        let bits = u64::from(bytes[0]) << 32
            | u64::from(bytes[1]) << 24
            | u64::from(bytes[2]) << 16
            | u64::from(bytes[3]) << 8
            | u64::from(bytes[4]);

        for i in (0..8).rev() {
            let idx = ((bits >> (i * 5)) & 0x1F) as usize;
            id.push(BASE32_ALPHABET[idx] as char);
        }

        Self(id)
    }

    /// Create from an existing ID string (validates format).
    pub fn from_str(s: &str) -> Option<Self> {
        if s.len() == 11
            && s.starts_with("tl-")
            && s[3..].chars().all(|c| BASE32_ALPHABET.contains(&(c as u8)))
        {
            Some(Self(s.to_string()))
        } else {
            None
        }
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for SessionId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

fn getrandom(buf: &mut [u8]) {
    use std::io::Read;
    let mut f = std::fs::File::open("/dev/urandom").expect("failed to open /dev/urandom");
    f.read_exact(buf).expect("failed to read random bytes");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generate_has_correct_format() {
        let id = SessionId::generate();
        let s = id.as_str();
        assert_eq!(s.len(), 11);
        assert!(s.starts_with("tl-"));
        assert!(s[3..].chars().all(|c| BASE32_ALPHABET.contains(&(c as u8))));
    }

    #[test]
    fn two_ids_are_different() {
        let a = SessionId::generate();
        let b = SessionId::generate();
        assert_ne!(a, b);
    }

    #[test]
    fn from_str_valid() {
        assert!(SessionId::from_str("tl-abcd2345").is_some());
    }

    #[test]
    fn from_str_invalid() {
        assert!(SessionId::from_str("bad").is_none());
        assert!(SessionId::from_str("tl-ABCD2345").is_none()); // uppercase
        assert!(SessionId::from_str("tl-abcd234").is_none()); // too short
    }

    #[test]
    fn filesystem_safe() {
        let id = SessionId::generate();
        // Should be safe for socket filenames
        assert!(!id.as_str().contains('/'));
        assert!(!id.as_str().contains('\0'));
    }
}
