//! Transport abstraction layer for provider-agnostic session I/O.
//!
//! Defines traits for creating connections and listeners independent of the
//! underlying transport (Unix socket, TCP, etc.). The Unix socket adapter
//! wraps `tokio::net::UnixStream`/`UnixListener` as the default implementation.
//!
//! ## Design Notes (T-122)
//!
//! - `Connection` is a blanket trait over `AsyncRead + AsyncWrite + Send + Unpin`
//! - `TransportListener` abstracts accept loops
//! - `Transport` is the factory that creates connections and listeners
//! - `LivenessProbe` is separate from transport (strategy differs per transport)
//! - No `async_trait` dependency: uses manual `Pin<Box<dyn Future>>` returns

use std::future::Future;
use std::io;
use std::pin::Pin;

use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::{TcpListener, TcpStream, UnixListener, UnixStream};

use termlink_protocol::TransportAddr;

// ---------------------------------------------------------------------------
// Trait: Connection
// ---------------------------------------------------------------------------

/// A bidirectional byte stream. Blanket-implemented for anything that is
/// `AsyncRead + AsyncWrite + Send + Unpin`.
pub trait Connection: AsyncRead + AsyncWrite + Send + Unpin {}

impl<T: AsyncRead + AsyncWrite + Send + Unpin> Connection for T {}

/// Boxed future returning a connection.
type ConnectFuture<'a> = Pin<Box<dyn Future<Output = io::Result<Box<dyn Connection>>> + Send + 'a>>;

/// Boxed future returning a listener.
type BindFuture<'a> = Pin<Box<dyn Future<Output = io::Result<Box<dyn TransportListener>>> + Send + 'a>>;

// ---------------------------------------------------------------------------
// Trait: TransportListener
// ---------------------------------------------------------------------------

/// A listener that accepts incoming connections.
pub trait TransportListener: Send {
    /// Accept the next incoming connection.
    fn accept(&self) -> ConnectFuture<'_>;

    /// The local address this listener is bound to.
    fn local_addr(&self) -> TransportAddr;
}

// ---------------------------------------------------------------------------
// Trait: Transport
// ---------------------------------------------------------------------------

/// Factory for creating connections and listeners.
pub trait Transport: Send + Sync {
    /// Connect to a remote address.
    fn connect<'a>(&'a self, addr: &'a TransportAddr) -> ConnectFuture<'a>;

    /// Bind a listener to a local address.
    fn bind<'a>(&'a self, addr: &'a TransportAddr) -> BindFuture<'a>;
}

// ---------------------------------------------------------------------------
// Trait: LivenessProbe
// ---------------------------------------------------------------------------

/// Check whether a session at a given address is alive.
///
/// Separated from `Transport` because liveness strategy differs per transport:
/// - Unix: check socket file existence + PID
/// - TCP: attempt connection or ping
pub trait LivenessProbe: Send + Sync {
    /// Returns `true` if the session at `addr` appears to be alive.
    fn check(&self, addr: &TransportAddr) -> bool;
}

// ===========================================================================
// Unix socket adapter
// ===========================================================================

/// Unix socket transport adapter — wraps `tokio::net::UnixStream`/`UnixListener`.
#[derive(Debug, Default)]
pub struct UnixTransport;

impl Transport for UnixTransport {
    fn connect<'a>(&'a self, addr: &'a TransportAddr) -> ConnectFuture<'a> {
        Box::pin(async move {
            let path = addr.as_unix_path().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "expected Unix address")
            })?;
            let stream = UnixStream::connect(path).await?;
            Ok(Box::new(stream) as Box<dyn Connection>)
        })
    }

    fn bind<'a>(&'a self, addr: &'a TransportAddr) -> BindFuture<'a> {
        Box::pin(async move {
            let path = addr.as_unix_path().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "expected Unix address")
            })?;
            // Remove leftover socket file
            let _ = std::fs::remove_file(path);
            let listener = UnixListener::bind(path)?;
            let addr = TransportAddr::unix(path);
            Ok(Box::new(UnixTransportListener { listener, addr }) as Box<dyn TransportListener>)
        })
    }
}

/// Unix socket listener adapter.
struct UnixTransportListener {
    listener: UnixListener,
    addr: TransportAddr,
}

impl TransportListener for UnixTransportListener {
    fn accept(&self) -> ConnectFuture<'_> {
        Box::pin(async move {
            let (stream, _) = self.listener.accept().await?;
            Ok(Box::new(stream) as Box<dyn Connection>)
        })
    }

    fn local_addr(&self) -> TransportAddr {
        self.addr.clone()
    }
}

/// Unix socket liveness probe — checks if the socket file exists on disk.
///
/// This replicates the current behavior from `liveness::is_alive` for the
/// file-existence check portion. PID checking remains in `liveness.rs`.
#[derive(Debug, Default)]
pub struct UnixLivenessProbe;

impl LivenessProbe for UnixLivenessProbe {
    fn check(&self, addr: &TransportAddr) -> bool {
        match addr.as_unix_path() {
            Some(path) => path.exists(),
            None => false,
        }
    }
}

// ===========================================================================
// TCP transport adapter
// ===========================================================================

/// TCP transport adapter — wraps `tokio::net::TcpStream`/`TcpListener`.
#[derive(Debug, Default)]
pub struct TcpTransport;

impl Transport for TcpTransport {
    fn connect<'a>(&'a self, addr: &'a TransportAddr) -> ConnectFuture<'a> {
        Box::pin(async move {
            let (host, port) = addr.as_tcp().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "expected TCP address")
            })?;
            let stream = TcpStream::connect((host, port)).await?;
            stream.set_nodelay(true)?;
            Ok(Box::new(stream) as Box<dyn Connection>)
        })
    }

    fn bind<'a>(&'a self, addr: &'a TransportAddr) -> BindFuture<'a> {
        Box::pin(async move {
            let (host, port) = addr.as_tcp().ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidInput, "expected TCP address")
            })?;
            let listener = TcpListener::bind((host, port)).await?;
            let local = listener.local_addr()?;
            let addr = TransportAddr::tcp(local.ip().to_string(), local.port());
            Ok(Box::new(TcpTransportListener { listener, addr }) as Box<dyn TransportListener>)
        })
    }
}

/// TCP listener adapter.
struct TcpTransportListener {
    listener: TcpListener,
    addr: TransportAddr,
}

impl TransportListener for TcpTransportListener {
    fn accept(&self) -> ConnectFuture<'_> {
        Box::pin(async move {
            let (stream, _) = self.listener.accept().await?;
            stream.set_nodelay(true)?;
            Ok(Box::new(stream) as Box<dyn Connection>)
        })
    }

    fn local_addr(&self) -> TransportAddr {
        self.addr.clone()
    }
}

/// TCP liveness probe — attempts a TCP connect with a short timeout.
#[derive(Debug, Default)]
pub struct TcpLivenessProbe;

impl LivenessProbe for TcpLivenessProbe {
    fn check(&self, addr: &TransportAddr) -> bool {
        match addr.as_tcp() {
            Some((host, port)) => {
                // Use a std::net blocking connect with a short timeout.
                // LivenessProbe::check is sync, so we can't use async here.
                use std::net::{TcpStream as StdTcpStream, ToSocketAddrs};
                use std::time::Duration;
                let addr_str = format!("{}:{}", host, port);
                if let Ok(mut addrs) = addr_str.to_socket_addrs()
                    && let Some(sock_addr) = addrs.next() {
                        return StdTcpStream::connect_timeout(&sock_addr, Duration::from_millis(500)).is_ok();
                    }
                false
            }
            None => false,
        }
    }
}

// ===========================================================================
// Tests
// ===========================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::sync::atomic::{AtomicU32, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_socket() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        PathBuf::from(format!("/tmp/tl-transport-{}-{}.sock", std::process::id(), n))
    }

    #[tokio::test]
    async fn unix_transport_connect_and_accept() {
        let path = test_socket();
        let _ = std::fs::remove_file(&path);
        let addr = TransportAddr::unix(&path);

        let transport = UnixTransport;
        let listener = transport.bind(&addr).await.unwrap();
        assert_eq!(listener.local_addr(), addr);

        // Spawn accept
        let accept_handle = tokio::spawn(async move {
            let mut conn = listener.accept().await.unwrap();
            let mut buf = [0u8; 5];
            conn.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"hello");
        });

        // Connect and write
        let mut conn = transport.connect(&addr).await.unwrap();
        conn.write_all(b"hello").await.unwrap();
        drop(conn);

        accept_handle.await.unwrap();
        let _ = std::fs::remove_file(&path);
    }

    #[tokio::test]
    async fn unix_transport_invalid_addr() {
        let transport = UnixTransport;
        let tcp_addr = TransportAddr::tcp("localhost", 8080);

        let result = transport.connect(&tcp_addr).await;
        assert!(result.is_err());

        let result = transport.bind(&tcp_addr).await;
        assert!(result.is_err());
    }

    #[test]
    fn unix_liveness_probe_missing_socket() {
        let probe = UnixLivenessProbe;
        let addr = TransportAddr::unix("/tmp/definitely-not-a-real-socket.sock");
        assert!(!probe.check(&addr));
    }

    #[test]
    fn unix_liveness_probe_existing_file() {
        let path = std::env::temp_dir().join(format!(
            "tl-probe-test-{}.sock",
            std::process::id()
        ));
        std::fs::write(&path, b"fake").unwrap();

        let probe = UnixLivenessProbe;
        let addr = TransportAddr::unix(&path);
        assert!(probe.check(&addr));

        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unix_liveness_probe_tcp_returns_false() {
        let probe = UnixLivenessProbe;
        let addr = TransportAddr::tcp("localhost", 8080);
        assert!(!probe.check(&addr));
    }

    #[test]
    fn connection_blanket_impl() {
        // Verify that UnixStream and TcpStream implement Connection
        fn _assert_connection<T: Connection>() {}
        _assert_connection::<UnixStream>();
        _assert_connection::<TcpStream>();
    }

    #[tokio::test]
    async fn tcp_transport_connect_and_accept() {
        let addr = TransportAddr::tcp("127.0.0.1", 0); // port 0 = OS picks

        let transport = TcpTransport;
        let listener = transport.bind(&addr).await.unwrap();
        let bound_addr = listener.local_addr();
        assert!(bound_addr.is_tcp());

        let accept_handle = tokio::spawn(async move {
            let mut conn = listener.accept().await.unwrap();
            let mut buf = [0u8; 5];
            conn.read_exact(&mut buf).await.unwrap();
            assert_eq!(&buf, b"hello");
        });

        let mut conn = transport.connect(&bound_addr).await.unwrap();
        conn.write_all(b"hello").await.unwrap();
        drop(conn);

        accept_handle.await.unwrap();
    }

    #[tokio::test]
    async fn tcp_transport_invalid_addr() {
        let transport = TcpTransport;
        let unix_addr = TransportAddr::unix("/tmp/test.sock");

        let result = transport.connect(&unix_addr).await;
        assert!(result.is_err());

        let result = transport.bind(&unix_addr).await;
        assert!(result.is_err());
    }

    #[test]
    fn tcp_liveness_probe_no_listener() {
        let probe = TcpLivenessProbe;
        // Port 1 should not be listening
        let addr = TransportAddr::tcp("127.0.0.1", 1);
        assert!(!probe.check(&addr));
    }

    #[test]
    fn tcp_liveness_probe_unix_returns_false() {
        let probe = TcpLivenessProbe;
        let addr = TransportAddr::unix("/tmp/test.sock");
        assert!(!probe.check(&addr));
    }
}
