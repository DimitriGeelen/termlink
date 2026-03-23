//! Data plane server — streams PTY output as binary frames over a Unix socket.
//!
//! Each PTY-backed session can optionally run a data plane server alongside the
//! control plane server. The data plane uses the binary frame protocol for
//! low-latency, high-throughput I/O:
//!
//! - **Output frames** (server → client): PTY output streamed in real-time
//! - **Input frames** (client → server): keystrokes written to PTY master
//! - **Resize frames** (client → server): terminal resize requests
//! - **Ping/Pong** (bidirectional): keepalive

use std::path::{Path, PathBuf};
use std::sync::Arc;

use tokio::net::{UnixListener, UnixStream};
use tokio::sync::broadcast;

use termlink_protocol::data::{FrameFlags, FrameType};

use crate::codec::{FrameReader, FrameWriter};
use crate::pty::PtySession;

/// Data plane socket path: `{session_socket}.data`
/// e.g., `/tmp/termlink-501/sessions/tl-k7mx2b4n.sock.data`
pub fn data_socket_path(control_socket: &Path) -> PathBuf {
    let mut p = control_socket.as_os_str().to_owned();
    p.push(".data");
    PathBuf::from(p)
}

/// Run the data plane server for a PTY session.
///
/// Listens on the data socket and streams PTY output to connected clients.
/// Accepts input frames and writes them to the PTY master.
pub async fn run(
    data_socket: &Path,
    pty: Arc<PtySession>,
    output_rx: broadcast::Receiver<Vec<u8>>,
) -> std::io::Result<()> {
    if let Some(parent) = data_socket.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let _ = std::fs::remove_file(data_socket);

    let listener = UnixListener::bind(data_socket)?;
    tracing::info!(path = %data_socket.display(), "Data plane listening");

    run_data_accept_loop(listener, pty, output_rx).await;
    Ok(())
}

/// Accept loop for data plane connections.
async fn run_data_accept_loop(
    listener: UnixListener,
    pty: Arc<PtySession>,
    output_rx: broadcast::Receiver<Vec<u8>>,
) {
    loop {
        match listener.accept().await {
            Ok((stream, _addr)) => {
                let pty_clone = pty.clone();
                // Each client gets its own receiver from the broadcast channel
                let client_rx = output_rx.resubscribe();
                tokio::spawn(async move {
                    handle_data_connection(stream, pty_clone, client_rx).await;
                });
            }
            Err(e) => {
                tracing::error!(error = %e, "Data plane accept failed");
                break;
            }
        }
    }
}

/// Handle a single data plane client connection.
async fn handle_data_connection(
    stream: UnixStream,
    pty: Arc<PtySession>,
    mut output_rx: broadcast::Receiver<Vec<u8>>,
) {
    let (read_half, write_half) = tokio::io::split(stream);
    let mut reader = FrameReader::new(read_half);
    let mut writer = FrameWriter::new(write_half);

    loop {
        tokio::select! {
            // Stream PTY output to client
            output = output_rx.recv() => {
                match output {
                    Ok(data) => {
                        if let Err(e) = writer.write_frame(
                            FrameType::Output,
                            FrameFlags::empty(),
                            0,
                            &data,
                        ).await {
                            tracing::debug!(error = %e, "Data plane: write failed");
                            break;
                        }
                    }
                    Err(broadcast::error::RecvError::Closed) => break,
                    Err(broadcast::error::RecvError::Lagged(n)) => {
                        tracing::warn!(skipped = n, "Data plane client lagging, frames dropped");
                    }
                }
            }

            // Read input frames from client
            frame = reader.read_frame() => {
                match frame {
                    Ok(Some(frame)) => {
                        match frame.header.frame_type {
                            FrameType::Input => {
                                if let Err(e) = pty.write(&frame.payload).await {
                                    tracing::warn!(error = %e, "Data plane: PTY write failed");
                                }
                            }
                            FrameType::Resize => {
                                if frame.payload.len() >= 4 {
                                    let cols = u16::from_be_bytes([frame.payload[0], frame.payload[1]]);
                                    let rows = u16::from_be_bytes([frame.payload[2], frame.payload[3]]);
                                    if let Err(e) = pty.resize(cols, rows) {
                                        tracing::warn!(error = %e, "Data plane: resize failed");
                                    }
                                }
                            }
                            FrameType::Ping => {
                                let _ = writer.write_frame(
                                    FrameType::Pong,
                                    FrameFlags::empty(),
                                    0,
                                    &frame.payload,
                                ).await;
                            }
                            FrameType::Close => {
                                tracing::debug!("Data plane: client sent Close frame");
                                break;
                            }
                            _ => {
                                tracing::debug!(
                                    frame_type = ?frame.header.frame_type,
                                    "Data plane: ignoring unexpected frame type"
                                );
                            }
                        }
                    }
                    Ok(None) => break, // EOF
                    Err(e) => {
                        tracing::debug!(error = %e, "Data plane: read error");
                        break;
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::sync::broadcast;

    use crate::test_util::PTY_LOCK;

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_data_socket() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(format!("/tmp/tl-data-{}-{}.sock", std::process::id(), n))
    }

    #[test]
    fn data_socket_path_appends_data() {
        let control = PathBuf::from("/tmp/termlink/sessions/tl-abc123.sock");
        let data = data_socket_path(&control);
        assert_eq!(data, PathBuf::from("/tmp/termlink/sessions/tl-abc123.sock.data"));
    }

    /// Helper: start data server, connect client, return handles.
    async fn setup_data_test(
        socket: &Path,
        pty: Arc<PtySession>,
        rx: broadcast::Receiver<Vec<u8>>,
    ) -> (
        tokio::task::JoinHandle<()>,
        FrameReader<tokio::io::ReadHalf<UnixStream>>,
        FrameWriter<tokio::io::WriteHalf<UnixStream>>,
    ) {
        let socket_clone = socket.to_path_buf();
        let pty_clone = pty.clone();
        let handle = tokio::spawn(async move {
            run(&socket_clone, pty_clone, rx).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        let stream = UnixStream::connect(socket).await.unwrap();
        let (read_half, write_half) = tokio::io::split(stream);

        // Give handler time to start its select loop after accepting connection
        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        (handle, FrameReader::new(read_half), FrameWriter::new(write_half))
    }

    #[tokio::test]
    async fn output_broadcast_to_client() {
        let _guard = PTY_LOCK.lock().await;
        let socket = test_data_socket();
        let _ = std::fs::remove_file(&socket);

        let (tx, rx) = broadcast::channel::<Vec<u8>>(64);
        let pty = Arc::new(PtySession::spawn(None, 1024).unwrap());

        let (handle, mut reader, _writer) = setup_data_test(&socket, pty.clone(), rx).await;

        // Broadcast some output — handler should forward as Output frame
        tx.send(b"hello data plane".to_vec()).unwrap();

        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader.read_frame(),
        )
        .await
        .expect("timed out waiting for frame")
        .unwrap()
        .unwrap();
        assert_eq!(frame.header.frame_type, FrameType::Output);
        assert_eq!(frame.payload, b"hello data plane");

        // Second message
        tx.send(b"second chunk".to_vec()).unwrap();
        let frame = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader.read_frame(),
        )
        .await
        .expect("timed out")
        .unwrap()
        .unwrap();
        assert_eq!(frame.payload, b"second chunk");

        handle.abort();
        let _ = pty.signal(libc::SIGTERM);
        let _ = std::fs::remove_file(&socket);
    }

    #[tokio::test]
    async fn input_frame_reaches_pty() {
        let _guard = PTY_LOCK.lock().await;
        let socket = test_data_socket();
        let _ = std::fs::remove_file(&socket);

        let (_tx, rx) = broadcast::channel::<Vec<u8>>(64);
        let pty = Arc::new(PtySession::spawn(None, 4096).unwrap());

        // Start PTY read loop so it captures output
        let pty_read = pty.clone();
        let read_handle = tokio::spawn(async move {
            let _ = pty_read.read_loop().await;
        });

        let (handle, _reader, mut writer) = setup_data_test(&socket, pty.clone(), rx).await;

        // Send "echo test-data-plane\n" as Input frame
        writer
            .write_frame(
                FrameType::Input,
                FrameFlags::empty(),
                0,
                b"echo test-data-plane\n",
            )
            .await
            .unwrap();

        // Give PTY time to process
        tokio::time::sleep(std::time::Duration::from_millis(200)).await;

        // Check scrollback for the output
        let sb = pty.scrollback();
        let sb = sb.lock().await;
        let bytes = sb.last_n_bytes(sb.len());
        let output = String::from_utf8_lossy(&bytes);
        assert!(
            output.contains("test-data-plane"),
            "expected scrollback to contain 'test-data-plane', got: {}",
            output
        );

        handle.abort();
        read_handle.abort();
        let _ = pty.signal(libc::SIGTERM);
        let _ = std::fs::remove_file(&socket);
    }

    #[tokio::test]
    async fn ping_pong() {
        let _guard = PTY_LOCK.lock().await;
        let socket = test_data_socket();
        let _ = std::fs::remove_file(&socket);

        let (_tx, rx) = broadcast::channel::<Vec<u8>>(64);
        let pty = Arc::new(PtySession::spawn(None, 1024).unwrap());

        let (handle, mut reader, mut writer) = setup_data_test(&socket, pty.clone(), rx).await;

        // Send Ping
        writer
            .write_frame(FrameType::Ping, FrameFlags::empty(), 0, b"keepalive")
            .await
            .unwrap();

        // Should get Pong back
        let pong = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader.read_frame(),
        )
        .await
        .expect("timed out waiting for pong")
        .unwrap()
        .unwrap();
        assert_eq!(pong.header.frame_type, FrameType::Pong);
        assert_eq!(pong.payload, b"keepalive");

        handle.abort();
        let _ = pty.signal(libc::SIGTERM);
        let _ = std::fs::remove_file(&socket);
    }

    #[tokio::test]
    async fn mirror_mode_receives_output_read_only() {
        let _guard = PTY_LOCK.lock().await;
        let socket = test_data_socket();
        let _ = std::fs::remove_file(&socket);

        let (tx, rx) = broadcast::channel::<Vec<u8>>(64);
        let pty = Arc::new(PtySession::spawn(None, 1024).unwrap());

        // Start data server
        let socket_clone = socket.clone();
        let pty_clone = pty.clone();
        let server_handle = tokio::spawn(async move {
            run(&socket_clone, pty_clone, rx).await.unwrap();
        });

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Connect two "mirror" clients (read-only — they never send Input frames)
        let stream1 = UnixStream::connect(&socket).await.unwrap();
        let (r1, _w1) = tokio::io::split(stream1);
        let mut reader1 = FrameReader::new(r1);

        let stream2 = UnixStream::connect(&socket).await.unwrap();
        let (r2, _w2) = tokio::io::split(stream2);
        let mut reader2 = FrameReader::new(r2);

        tokio::time::sleep(std::time::Duration::from_millis(20)).await;

        // Broadcast output — both mirrors should receive it
        tx.send(b"mirror test data".to_vec()).unwrap();

        let frame1 = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader1.read_frame(),
        )
        .await
        .expect("mirror 1 timed out")
        .unwrap()
        .unwrap();
        assert_eq!(frame1.header.frame_type, FrameType::Output);
        assert_eq!(frame1.payload, b"mirror test data");

        let frame2 = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            reader2.read_frame(),
        )
        .await
        .expect("mirror 2 timed out")
        .unwrap()
        .unwrap();
        assert_eq!(frame2.header.frame_type, FrameType::Output);
        assert_eq!(frame2.payload, b"mirror test data");

        server_handle.abort();
        let _ = pty.signal(libc::SIGTERM);
        let _ = std::fs::remove_file(&socket);
    }
}
