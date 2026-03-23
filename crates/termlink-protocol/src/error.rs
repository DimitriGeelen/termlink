use thiserror::Error;

#[derive(Debug, Error)]
pub enum ProtocolError {
    #[error("invalid magic bytes: expected 0x544C, got 0x{0:04X}")]
    InvalidMagic(u16),

    #[error("payload too large: {0} bytes (max {max})", max = crate::MAX_PAYLOAD_SIZE)]
    PayloadTooLarge(u32),

    #[error("unknown frame type: 0x{0:X}")]
    UnknownFrameType(u8),

    #[error("unsupported protocol version: {0}")]
    UnsupportedVersion(u8),

    #[error("incomplete frame: need {expected} bytes, have {available}")]
    IncompleteFrame { expected: usize, available: usize },

    #[error("JSON-RPC error: {0}")]
    JsonRpc(String),

    #[error("serialization error: {0}")]
    Serialization(#[from] serde_json::Error),

    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn invalid_magic_display() {
        let err = ProtocolError::InvalidMagic(0xDEAD);
        assert_eq!(err.to_string(), "invalid magic bytes: expected 0x544C, got 0xDEAD");
    }

    #[test]
    fn payload_too_large_display() {
        let err = ProtocolError::PayloadTooLarge(20_000_000);
        assert!(err.to_string().contains("20000000 bytes"));
        assert!(err.to_string().contains("max 16777216"));
    }

    #[test]
    fn unknown_frame_type_display() {
        let err = ProtocolError::UnknownFrameType(0xFF);
        assert_eq!(err.to_string(), "unknown frame type: 0xFF");
    }

    #[test]
    fn unsupported_version_display() {
        let err = ProtocolError::UnsupportedVersion(99);
        assert_eq!(err.to_string(), "unsupported protocol version: 99");
    }

    #[test]
    fn incomplete_frame_display() {
        let err = ProtocolError::IncompleteFrame {
            expected: 100,
            available: 42,
        };
        assert_eq!(
            err.to_string(),
            "incomplete frame: need 100 bytes, have 42"
        );
    }

    #[test]
    fn json_rpc_display() {
        let err = ProtocolError::JsonRpc("method not found".into());
        assert_eq!(err.to_string(), "JSON-RPC error: method not found");
    }

    #[test]
    fn from_serde_json_error() {
        let json_err = serde_json::from_str::<serde_json::Value>("not json").unwrap_err();
        let err = ProtocolError::from(json_err);
        assert!(err.to_string().starts_with("serialization error:"));
    }

    #[test]
    fn from_io_error() {
        let io_err = std::io::Error::new(std::io::ErrorKind::BrokenPipe, "pipe broken");
        let err = ProtocolError::from(io_err);
        assert_eq!(err.to_string(), "I/O error: pipe broken");
    }
}
