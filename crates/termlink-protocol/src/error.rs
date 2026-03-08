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
