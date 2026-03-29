//! Async frame codec for the data plane binary protocol.
//!
//! Provides [`FrameReader`] and [`FrameWriter`] for reading/writing
//! data plane frames over any async stream (typically a Unix socket).

use tokio::io::{AsyncReadExt, AsyncWriteExt};

use termlink_protocol::data::{Frame, FrameFlags, FrameHeader, FrameType};
use termlink_protocol::{ProtocolError, FRAME_HEADER_SIZE};

/// Async frame reader — reads data plane frames from a stream.
pub struct FrameReader<R> {
    reader: R,
    header_buf: [u8; FRAME_HEADER_SIZE],
}

impl<R: AsyncReadExt + Unpin> FrameReader<R> {
    pub fn new(reader: R) -> Self {
        Self {
            reader,
            header_buf: [0u8; FRAME_HEADER_SIZE],
        }
    }

    /// Read the next frame. Returns `None` on clean EOF.
    pub async fn read_frame(&mut self) -> Result<Option<Frame>, ProtocolError> {
        // Read header
        match self.reader.read_exact(&mut self.header_buf).await {
            Ok(_) => {}
            Err(e) if e.kind() == std::io::ErrorKind::UnexpectedEof => return Ok(None),
            Err(e) => return Err(ProtocolError::Io(e)),
        }

        let header = FrameHeader::decode(&self.header_buf)?;
        let payload_len = header.payload_length as usize;

        // Read payload
        let mut payload = vec![0u8; payload_len];
        if payload_len > 0 {
            self.reader
                .read_exact(&mut payload)
                .await
                .map_err(ProtocolError::Io)?;
        }

        Ok(Some(Frame { header, payload }))
    }
}

/// Async frame writer — writes data plane frames to a stream.
pub struct FrameWriter<W> {
    writer: W,
    sequence: u64,
}

impl<W: AsyncWriteExt + Unpin> FrameWriter<W> {
    pub fn new(writer: W) -> Self {
        Self {
            writer,
            sequence: 0,
        }
    }

    /// Write a frame, auto-incrementing the sequence number.
    pub async fn write_frame(
        &mut self,
        frame_type: FrameType,
        flags: FrameFlags,
        channel_id: u32,
        payload: &[u8],
    ) -> Result<(), ProtocolError> {
        let frame = Frame::new(
            frame_type,
            flags,
            channel_id,
            self.sequence,
            payload.to_vec(),
        );
        self.sequence += 1;

        let encoded = frame.encode();
        self.writer
            .write_all(&encoded)
            .await
            .map_err(ProtocolError::Io)?;
        self.writer.flush().await.map_err(ProtocolError::Io)?;
        Ok(())
    }

    /// Write a raw pre-built frame without modifying the sequence.
    pub async fn write_raw_frame(&mut self, frame: &Frame) -> Result<(), ProtocolError> {
        let encoded = frame.encode();
        self.writer
            .write_all(&encoded)
            .await
            .map_err(ProtocolError::Io)?;
        self.writer.flush().await.map_err(ProtocolError::Io)?;
        Ok(())
    }

    /// Current sequence number.
    pub fn sequence(&self) -> u64 {
        self.sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tokio::io::duplex;

    #[tokio::test]
    async fn roundtrip_single_frame() {
        let (client, server) = duplex(4096);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 1, b"hello")
            .await
            .unwrap();

        // Drop writer to signal EOF
        drop(writer);
        drop(client_read);

        let frame = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(frame.header.frame_type, FrameType::Output);
        assert_eq!(frame.header.channel_id, 1);
        assert_eq!(frame.header.sequence, 0);
        assert_eq!(frame.payload, b"hello");

        // Next read should be EOF
        let eof = reader.read_frame().await.unwrap();
        assert!(eof.is_none());
    }

    #[tokio::test]
    async fn roundtrip_multiple_frames() {
        let (client, server) = duplex(8192);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 0, b"frame-1")
            .await
            .unwrap();
        writer
            .write_frame(FrameType::Input, FrameFlags::URGENT, 0, b"frame-2")
            .await
            .unwrap();
        writer
            .write_frame(FrameType::Ping, FrameFlags::empty(), 0, &[])
            .await
            .unwrap();

        drop(writer);
        drop(client_read);

        let f1 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f1.header.frame_type, FrameType::Output);
        assert_eq!(f1.header.sequence, 0);
        assert_eq!(f1.payload, b"frame-1");

        let f2 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f2.header.frame_type, FrameType::Input);
        assert_eq!(f2.header.sequence, 1);
        assert!(f2.header.flags.contains(FrameFlags::URGENT));
        assert_eq!(f2.payload, b"frame-2");

        let f3 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f3.header.frame_type, FrameType::Ping);
        assert_eq!(f3.header.sequence, 2);
        assert!(f3.payload.is_empty());

        assert!(reader.read_frame().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn empty_payload_roundtrip() {
        let (client, server) = duplex(4096);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        writer
            .write_frame(FrameType::Close, FrameFlags::FIN, 0, &[])
            .await
            .unwrap();

        drop(writer);
        drop(client_read);

        let frame = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(frame.header.frame_type, FrameType::Close);
        assert!(frame.header.flags.contains(FrameFlags::FIN));
        assert!(frame.payload.is_empty());
    }

    #[tokio::test]
    async fn write_raw_frame_preserves_sequence() {
        let (client, server) = duplex(4096);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        // write_raw_frame should NOT increment sequence counter
        let raw = Frame::new(FrameType::Pong, FrameFlags::empty(), 0, 999, b"pong".to_vec());
        writer.write_raw_frame(&raw).await.unwrap();
        assert_eq!(writer.sequence(), 0); // unchanged

        // Regular write still starts from 0
        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 0, b"next")
            .await
            .unwrap();
        assert_eq!(writer.sequence(), 1);

        drop(writer);
        drop(client_read);

        let f1 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f1.header.frame_type, FrameType::Pong);
        assert_eq!(f1.header.sequence, 999); // raw sequence preserved
        assert_eq!(f1.payload, b"pong");

        let f2 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f2.header.frame_type, FrameType::Output);
        assert_eq!(f2.header.sequence, 0);
        assert_eq!(f2.payload, b"next");
    }

    #[tokio::test]
    async fn all_frame_types_through_codec() {
        let types = [
            FrameType::Output,
            FrameType::Input,
            FrameType::Resize,
            FrameType::Signal,
            FrameType::Transfer,
            FrameType::Ping,
            FrameType::Pong,
            FrameType::Close,
        ];

        let (client, server) = duplex(8192);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        for (i, ft) in types.iter().enumerate() {
            writer
                .write_frame(*ft, FrameFlags::empty(), i as u32, &[i as u8])
                .await
                .unwrap();
        }

        drop(writer);
        drop(client_read);

        for (i, ft) in types.iter().enumerate() {
            let frame = reader.read_frame().await.unwrap().unwrap();
            assert_eq!(frame.header.frame_type, *ft);
            assert_eq!(frame.header.channel_id, i as u32);
            assert_eq!(frame.payload, vec![i as u8]);
        }

        assert!(reader.read_frame().await.unwrap().is_none());
    }

    #[tokio::test]
    async fn multiple_channels_interleaved() {
        let (client, server) = duplex(8192);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 1, b"ch1-a")
            .await
            .unwrap();
        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 2, b"ch2-a")
            .await
            .unwrap();
        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 1, b"ch1-b")
            .await
            .unwrap();

        drop(writer);
        drop(client_read);

        let f1 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f1.header.channel_id, 1);
        assert_eq!(f1.payload, b"ch1-a");

        let f2 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f2.header.channel_id, 2);
        assert_eq!(f2.payload, b"ch2-a");

        let f3 = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(f3.header.channel_id, 1);
        assert_eq!(f3.payload, b"ch1-b");
    }

    #[tokio::test]
    async fn combined_flags_roundtrip() {
        let (client, server) = duplex(4096);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        let all_flags = FrameFlags::FIN | FrameFlags::COMPRESSED | FrameFlags::BINARY | FrameFlags::URGENT;
        writer
            .write_frame(FrameType::Transfer, all_flags, 42, b"data")
            .await
            .unwrap();

        drop(writer);
        drop(client_read);

        let frame = reader.read_frame().await.unwrap().unwrap();
        assert!(frame.header.flags.contains(FrameFlags::FIN));
        assert!(frame.header.flags.contains(FrameFlags::COMPRESSED));
        assert!(frame.header.flags.contains(FrameFlags::BINARY));
        assert!(frame.header.flags.contains(FrameFlags::URGENT));
    }

    #[tokio::test]
    async fn sequence_auto_increments() {
        let (client, _server) = duplex(4096);
        let (_client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        assert_eq!(writer.sequence(), 0);

        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 0, b"a")
            .await
            .unwrap();
        assert_eq!(writer.sequence(), 1);

        writer
            .write_frame(FrameType::Output, FrameFlags::empty(), 0, b"b")
            .await
            .unwrap();
        assert_eq!(writer.sequence(), 2);
    }

    #[tokio::test]
    async fn large_payload_roundtrip() {
        let payload = vec![0xABu8; 64 * 1024]; // 64 KiB
        let (client, server) = duplex(128 * 1024);
        let (server_read, _server_write) = tokio::io::split(server);
        let (client_read, client_write) = tokio::io::split(client);

        let mut writer = FrameWriter::new(client_write);
        let mut reader = FrameReader::new(server_read);

        writer
            .write_frame(FrameType::Transfer, FrameFlags::BINARY, 99, &payload)
            .await
            .unwrap();

        drop(writer);
        drop(client_read);

        let frame = reader.read_frame().await.unwrap().unwrap();
        assert_eq!(frame.header.frame_type, FrameType::Transfer);
        assert_eq!(frame.header.channel_id, 99);
        assert!(frame.header.flags.contains(FrameFlags::BINARY));
        assert_eq!(frame.payload.len(), 64 * 1024);
        assert_eq!(frame.payload, payload);
    }
}
