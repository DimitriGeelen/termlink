pub mod control;
pub mod data;
pub mod error;

pub use error::ProtocolError;

/// Protocol version for the data plane binary frames.
pub const DATA_PLANE_VERSION: u8 = 1;

/// Magic bytes for data plane frame sync: "TL" (0x54, 0x4C).
pub const FRAME_MAGIC: [u8; 2] = [0x54, 0x4C];

/// Fixed header size for data plane frames (including magic).
pub const FRAME_HEADER_SIZE: usize = 22;

/// Maximum payload size: 16 MiB.
pub const MAX_PAYLOAD_SIZE: u32 = 16 * 1024 * 1024;
