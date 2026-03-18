//! TLS certificate generation and configuration for the TCP hub.
//!
//! Generates a self-signed certificate on hub startup, writes cert+key PEM files
//! to the runtime directory, and provides TLS acceptor/connector configurations.

use std::io::BufReader;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use rcgen::{CertifiedKey, generate_simple_self_signed};
use rustls::pki_types::CertificateDer;
use tokio_rustls::TlsAcceptor;

use termlink_session::discovery;

/// Return the hub certificate PEM path: `runtime_dir()/hub.cert.pem`.
pub fn hub_cert_path() -> PathBuf {
    discovery::runtime_dir().join("hub.cert.pem")
}

/// Return the hub key PEM path: `runtime_dir()/hub.key.pem`.
pub fn hub_key_path() -> PathBuf {
    discovery::runtime_dir().join("hub.key.pem")
}

/// Generate a self-signed certificate and write PEM files to the runtime directory.
///
/// Returns a `TlsAcceptor` configured with the generated cert+key.
pub fn generate_and_write_cert() -> std::io::Result<TlsAcceptor> {
    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    let CertifiedKey { cert, key_pair } =
        generate_simple_self_signed(subject_alt_names).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::Other, format!("cert generation failed: {e}"))
        })?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

    let cert_path = hub_cert_path();
    let key_path = hub_key_path();

    // Write cert (readable by anyone on the machine — needed for clients)
    std::fs::write(&cert_path, &cert_pem)?;

    // Write key with restricted permissions (0600)
    std::fs::write(&key_path, &key_pem)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        std::fs::set_permissions(&key_path, std::fs::Permissions::from_mode(0o600))?;
    }

    tracing::info!(
        cert = %cert_path.display(),
        key = %key_path.display(),
        "Hub TLS certificate generated"
    );

    build_acceptor_from_pem(&cert_pem, &key_pem)
}

/// Build a TLS acceptor from PEM-encoded cert and key strings.
fn build_acceptor_from_pem(cert_pem: &str, key_pem: &str) -> std::io::Result<TlsAcceptor> {
    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_pem.as_bytes()))
        .collect::<Result<Vec<CertificateDer<'_>>, _>>()?;

    let key = rustls_pemfile::private_key(&mut BufReader::new(key_pem.as_bytes()))?
        .ok_or_else(|| std::io::Error::new(std::io::ErrorKind::InvalidData, "no private key found in PEM"))?;

    let config = rustls::ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(certs, key)
        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, format!("TLS config error: {e}")))?;

    Ok(TlsAcceptor::from(Arc::new(config)))
}

/// Build a TLS connector that trusts the hub's self-signed certificate.
///
/// Reads the cert from the given PEM file path.
pub fn build_client_connector(cert_pem_path: &Path) -> std::io::Result<tokio_rustls::TlsConnector> {
    let cert_pem = std::fs::read_to_string(cert_pem_path)?;
    let certs = rustls_pemfile::certs(&mut BufReader::new(cert_pem.as_bytes()))
        .collect::<Result<Vec<CertificateDer<'_>>, _>>()?;

    let mut root_store = rustls::RootCertStore::empty();
    for cert in certs {
        root_store.add(cert).map_err(|e| {
            std::io::Error::new(std::io::ErrorKind::InvalidData, format!("failed to add cert: {e}"))
        })?;
    }

    let config = rustls::ClientConfig::builder()
        .with_root_certificates(root_store)
        .with_no_client_auth();

    Ok(tokio_rustls::TlsConnector::from(Arc::new(config)))
}

/// Clean up TLS cert and key files.
pub fn cleanup() {
    let _ = std::fs::remove_file(hub_cert_path());
    let _ = std::fs::remove_file(hub_key_path());
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static TEST_COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_dir() -> PathBuf {
        let n = TEST_COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!("/tmp/tl-tls-{}-{}", std::process::id(), n));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        dir
    }

    #[test]
    fn generate_cert_and_build_connector() {
        // Override runtime dir for this test
        let dir = test_dir();
        let cert_path = dir.join("hub.cert.pem");
        let key_path = dir.join("hub.key.pem");

        // Generate cert manually (not using generate_and_write_cert since it uses runtime_dir)
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let CertifiedKey { cert, key_pair } =
            generate_simple_self_signed(subject_alt_names).unwrap();
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        std::fs::write(&cert_path, &cert_pem).unwrap();
        std::fs::write(&key_path, &key_pem).unwrap();

        // Build acceptor
        let acceptor = build_acceptor_from_pem(&cert_pem, &key_pem);
        assert!(acceptor.is_ok(), "Acceptor should build from valid cert+key");

        // Build client connector
        let connector = build_client_connector(&cert_path);
        assert!(connector.is_ok(), "Connector should build from valid cert");
    }

    #[tokio::test]
    async fn tls_handshake_roundtrip() {
        let dir = test_dir();
        let cert_path = dir.join("hub.cert.pem");
        let key_path = dir.join("hub.key.pem");

        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let CertifiedKey { cert, key_pair } =
            generate_simple_self_signed(subject_alt_names).unwrap();
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        std::fs::write(&cert_path, &cert_pem).unwrap();
        std::fs::write(&key_path, &key_pem).unwrap();

        let acceptor = build_acceptor_from_pem(&cert_pem, &key_pem).unwrap();
        let connector = build_client_connector(&cert_path).unwrap();

        // Start a TCP listener
        let tcp_listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let port = tcp_listener.local_addr().unwrap().port();

        // Server: accept + TLS handshake + read + echo back
        let server_handle = tokio::spawn(async move {
            use tokio::io::{AsyncReadExt, AsyncWriteExt};
            let (tcp_stream, _) = tcp_listener.accept().await.unwrap();
            let mut tls_stream = acceptor.accept(tcp_stream).await.unwrap();
            let mut buf = [0u8; 64];
            let n = tls_stream.read(&mut buf).await.unwrap();
            tls_stream.write_all(&buf[..n]).await.unwrap();
            tls_stream.shutdown().await.unwrap();
        });

        // Client: connect + TLS handshake + send + receive
        let tcp_stream = tokio::net::TcpStream::connect(format!("127.0.0.1:{port}"))
            .await
            .unwrap();
        let server_name = rustls::pki_types::ServerName::try_from("localhost").unwrap();
        let mut tls_stream = connector.connect(server_name, tcp_stream).await.unwrap();

        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        tls_stream.write_all(b"hello TLS").await.unwrap();

        let mut buf = [0u8; 64];
        let n = tls_stream.read(&mut buf).await.unwrap();
        assert_eq!(&buf[..n], b"hello TLS");

        let _ = server_handle.await;
    }
}
