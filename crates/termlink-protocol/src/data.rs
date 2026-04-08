use crate::{
    ProtocolError, DATA_PLANE_VERSION, FRAME_HEADER_SIZE, FRAME_MAGIC, MAX_PAYLOAD_SIZE,
};

/// Data plane frame types (4 bits, 0x0–0xF).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum FrameType {
    Output = 0x0,
    Input = 0x1,
    Resize = 0x2,
    Signal = 0x3,
    Transfer = 0x4,
    Ping = 0x5,
    Pong = 0x6,
    Close = 0x7,
    Governance = 0x8,
}

impl FrameType {
    pub fn from_u8(val: u8) -> Result<Self, ProtocolError> {
        match val {
            0x0 => Ok(Self::Output),
            0x1 => Ok(Self::Input),
            0x2 => Ok(Self::Resize),
            0x3 => Ok(Self::Signal),
            0x4 => Ok(Self::Transfer),
            0x5 => Ok(Self::Ping),
            0x6 => Ok(Self::Pong),
            0x7 => Ok(Self::Close),
            0x8 => Ok(Self::Governance),
            other => Err(ProtocolError::UnknownFrameType(other)),
        }
    }
}

bitflags::bitflags! {
    /// Data plane frame flags (8 bits).
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct FrameFlags: u8 {
        const FIN        = 0b0000_0001;
        const COMPRESSED = 0b0000_0010;
        const BINARY     = 0b0000_0100;
        const URGENT     = 0b0000_1000;
    }
}

/// A parsed data plane frame header (22 bytes on wire).
#[derive(Debug, Clone)]
pub struct FrameHeader {
    pub payload_length: u32,
    pub version: u8,
    pub frame_type: FrameType,
    pub flags: FrameFlags,
    pub sequence: u64,
    pub channel_id: u32,
}

/// A complete data plane frame: header + payload.
#[derive(Debug, Clone)]
pub struct Frame {
    pub header: FrameHeader,
    pub payload: Vec<u8>,
}

impl FrameHeader {
    /// Encode this header into a 22-byte buffer.
    pub fn encode(&self, buf: &mut [u8]) {
        assert!(buf.len() >= FRAME_HEADER_SIZE);
        // Magic: 2 bytes
        buf[0..2].copy_from_slice(&FRAME_MAGIC);
        // Payload length: 4 bytes big-endian
        buf[2..6].copy_from_slice(&self.payload_length.to_be_bytes());
        // Version (high nibble) | Type (low nibble): 1 byte
        buf[6] = (self.version << 4) | (self.frame_type as u8 & 0x0F);
        // Flags: 1 byte
        buf[7] = self.flags.bits();
        // Reserved: 2 bytes
        buf[8..10].copy_from_slice(&[0, 0]);
        // Sequence: 8 bytes big-endian
        buf[10..18].copy_from_slice(&self.sequence.to_be_bytes());
        // Channel ID: 4 bytes big-endian
        buf[18..22].copy_from_slice(&self.channel_id.to_be_bytes());
    }

    /// Decode a header from a 22-byte buffer.
    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        if buf.len() < FRAME_HEADER_SIZE {
            return Err(ProtocolError::IncompleteFrame {
                expected: FRAME_HEADER_SIZE,
                available: buf.len(),
            });
        }

        // Verify magic
        let magic = u16::from_be_bytes([buf[0], buf[1]]);
        if magic != 0x544C {
            return Err(ProtocolError::InvalidMagic(magic));
        }

        let payload_length = u32::from_be_bytes([buf[2], buf[3], buf[4], buf[5]]);
        if payload_length > MAX_PAYLOAD_SIZE {
            return Err(ProtocolError::PayloadTooLarge(payload_length));
        }

        let ver_type = buf[6];
        let version = ver_type >> 4;
        if version != DATA_PLANE_VERSION {
            return Err(ProtocolError::UnsupportedVersion(version));
        }

        let frame_type = FrameType::from_u8(ver_type & 0x0F)?;
        let flags = FrameFlags::from_bits_truncate(buf[7]);
        // buf[8..10] reserved, ignored
        let sequence = u64::from_be_bytes([
            buf[10], buf[11], buf[12], buf[13], buf[14], buf[15], buf[16], buf[17],
        ]);
        let channel_id = u32::from_be_bytes([buf[18], buf[19], buf[20], buf[21]]);

        Ok(Self {
            payload_length,
            version,
            frame_type,
            flags,
            sequence,
            channel_id,
        })
    }
}

impl Frame {
    /// Create a new frame.
    pub fn new(frame_type: FrameType, flags: FrameFlags, channel_id: u32, sequence: u64, payload: Vec<u8>) -> Self {
        Self {
            header: FrameHeader {
                payload_length: payload.len() as u32,
                version: DATA_PLANE_VERSION,
                frame_type,
                flags,
                sequence,
                channel_id,
            },
            payload,
        }
    }

    /// Encode the full frame (header + payload) into bytes.
    pub fn encode(&self) -> Vec<u8> {
        let mut buf = vec![0u8; FRAME_HEADER_SIZE + self.payload.len()];
        self.header.encode(&mut buf[..FRAME_HEADER_SIZE]);
        buf[FRAME_HEADER_SIZE..].copy_from_slice(&self.payload);
        buf
    }

    /// Decode a complete frame from bytes.
    pub fn decode(buf: &[u8]) -> Result<Self, ProtocolError> {
        let header = FrameHeader::decode(buf)?;
        let total = FRAME_HEADER_SIZE + header.payload_length as usize;
        if buf.len() < total {
            return Err(ProtocolError::IncompleteFrame {
                expected: total,
                available: buf.len(),
            });
        }
        let payload = buf[FRAME_HEADER_SIZE..total].to_vec();
        Ok(Self { header, payload })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn roundtrip_frame() {
        let frame = Frame::new(
            FrameType::Output,
            FrameFlags::empty(),
            1,
            42,
            b"hello, terminal!".to_vec(),
        );
        let encoded = frame.encode();
        assert_eq!(encoded.len(), FRAME_HEADER_SIZE + 16);

        let decoded = Frame::decode(&encoded).unwrap();
        assert_eq!(decoded.header.frame_type, FrameType::Output);
        assert_eq!(decoded.header.channel_id, 1);
        assert_eq!(decoded.header.sequence, 42);
        assert_eq!(decoded.payload, b"hello, terminal!");
    }

    #[test]
    fn magic_bytes() {
        let frame = Frame::new(FrameType::Ping, FrameFlags::empty(), 0, 0, vec![]);
        let encoded = frame.encode();
        assert_eq!(&encoded[0..2], &[0x54, 0x4C]); // "TL"
    }

    #[test]
    fn invalid_magic_rejected() {
        let mut buf = vec![0u8; FRAME_HEADER_SIZE];
        buf[0] = 0xFF;
        buf[1] = 0xFF;
        assert!(matches!(
            FrameHeader::decode(&buf),
            Err(ProtocolError::InvalidMagic(0xFFFF))
        ));
    }

    #[test]
    fn payload_too_large_rejected() {
        let mut buf = vec![0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&FRAME_MAGIC);
        // Set payload_length to MAX + 1
        let too_large = MAX_PAYLOAD_SIZE + 1;
        buf[2..6].copy_from_slice(&too_large.to_be_bytes());
        buf[6] = DATA_PLANE_VERSION << 4; // version | type=Output
        assert!(matches!(
            FrameHeader::decode(&buf),
            Err(ProtocolError::PayloadTooLarge(_))
        ));
    }

    #[test]
    fn flags_roundtrip() {
        let frame = Frame::new(
            FrameType::Input,
            FrameFlags::URGENT | FrameFlags::BINARY,
            5,
            100,
            vec![0x03], // Ctrl+C
        );
        let encoded = frame.encode();
        let decoded = Frame::decode(&encoded).unwrap();
        assert!(decoded.header.flags.contains(FrameFlags::URGENT));
        assert!(decoded.header.flags.contains(FrameFlags::BINARY));
        assert!(!decoded.header.flags.contains(FrameFlags::COMPRESSED));
    }

    #[test]
    fn all_frame_types() {
        for (ft, expected) in [
            (FrameType::Output, 0x0),
            (FrameType::Input, 0x1),
            (FrameType::Resize, 0x2),
            (FrameType::Signal, 0x3),
            (FrameType::Transfer, 0x4),
            (FrameType::Ping, 0x5),
            (FrameType::Pong, 0x6),
            (FrameType::Close, 0x7),
            (FrameType::Governance, 0x8),
        ] {
            let frame = Frame::new(ft, FrameFlags::empty(), 0, 0, vec![]);
            let encoded = frame.encode();
            let decoded = Frame::decode(&encoded).unwrap();
            assert_eq!(decoded.header.frame_type, ft);
            assert_eq!(ft as u8, expected);
        }
    }

    #[test]
    fn unknown_frame_type_rejected() {
        assert!(matches!(
            FrameType::from_u8(0x09),
            Err(ProtocolError::UnknownFrameType(0x09))
        ));
        assert!(matches!(
            FrameType::from_u8(0xFF),
            Err(ProtocolError::UnknownFrameType(0xFF))
        ));
    }

    #[test]
    fn incomplete_header_rejected() {
        // Too short for header
        let buf = vec![0x54, 0x4C, 0x00];
        assert!(matches!(
            FrameHeader::decode(&buf),
            Err(ProtocolError::IncompleteFrame { expected: 22, available: 3 })
        ));

        // Empty buffer
        assert!(matches!(
            FrameHeader::decode(&[]),
            Err(ProtocolError::IncompleteFrame { expected: 22, available: 0 })
        ));
    }

    #[test]
    fn unsupported_version_rejected() {
        let mut buf = vec![0u8; FRAME_HEADER_SIZE];
        buf[0..2].copy_from_slice(&FRAME_MAGIC);
        // version=2 in high nibble, type=Output in low nibble
        buf[6] = 0x20;
        assert!(matches!(
            FrameHeader::decode(&buf),
            Err(ProtocolError::UnsupportedVersion(2))
        ));
    }

    #[test]
    fn truncated_payload_rejected() {
        let frame = Frame::new(FrameType::Output, FrameFlags::empty(), 0, 0, b"hello".to_vec());
        let mut encoded = frame.encode();
        // Remove last 2 bytes of payload
        encoded.truncate(encoded.len() - 2);
        assert!(matches!(
            Frame::decode(&encoded),
            Err(ProtocolError::IncompleteFrame { .. })
        ));
    }

    #[test]
    fn zero_payload_frame_roundtrip() {
        let frame = Frame::new(FrameType::Ping, FrameFlags::empty(), 0, 0, vec![]);
        let encoded = frame.encode();
        assert_eq!(encoded.len(), FRAME_HEADER_SIZE);
        let decoded = Frame::decode(&encoded).unwrap();
        assert!(decoded.payload.is_empty());
        assert_eq!(decoded.header.payload_length, 0);
    }

    #[test]
    fn max_sequence_and_channel_roundtrip() {
        let frame = Frame::new(
            FrameType::Transfer,
            FrameFlags::BINARY,
            u32::MAX,
            u64::MAX,
            b"data".to_vec(),
        );
        let encoded = frame.encode();
        let decoded = Frame::decode(&encoded).unwrap();
        assert_eq!(decoded.header.sequence, u64::MAX);
        assert_eq!(decoded.header.channel_id, u32::MAX);
    }

    #[test]
    fn frame_header_version_field() {
        let frame = Frame::new(FrameType::Output, FrameFlags::empty(), 0, 0, vec![]);
        let encoded = frame.encode();
        let decoded = Frame::decode(&encoded).unwrap();
        assert_eq!(decoded.header.version, DATA_PLANE_VERSION);
    }
}
