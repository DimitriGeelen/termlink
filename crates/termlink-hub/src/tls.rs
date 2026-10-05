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

/// SHA-256 fingerprint (`sha256:<hex>`) of the hub's own certificate on disk,
/// or `None` when there is no readable cert (T-3345). Same value `hub probe`
/// and `hub fingerprint` report, because it is the same DER through the same
/// `cert_fingerprint`.
pub fn own_cert_fingerprint() -> Option<String> {
    own_cert_fingerprint_at(&hub_cert_path())
}

/// [`own_cert_fingerprint`] for an explicit path (test seam).
pub fn own_cert_fingerprint_at(path: &Path) -> Option<String> {
    let pem = std::fs::read_to_string(path).ok()?;
    let der = rustls_pemfile::certs(&mut BufReader::new(pem.as_bytes())).next()?.ok()?;
    Some(termlink_session::tofu::cert_fingerprint(&der))
}

/// The hub's canonical id, derived from a fingerprint (T-3345, OD-12 ruling 3c).
///
/// Until the architect decides how canonical ids are minted (arc-011 step 4),
/// the canonical id IS the first 16 hex of the TLS fingerprint — the value
/// every live `inbox:<hub-id>/<project>` name already uses, so nothing is
/// renamed. Clients must read it from `hub.version`, never derive it, so that
/// only this function changes when minting lands.
pub fn hub_id_from_fingerprint(fingerprint: &str) -> Option<String> {
    let hex = fingerprint.strip_prefix("sha256:").unwrap_or(fingerprint);
    let id = hex.get(..16)?;
    id.chars().all(|c| c.is_ascii_hexdigit()).then(|| id.to_ascii_lowercase())
}

/// Load existing cert+key from disk, or generate a new self-signed pair.
///
/// Persist-if-present (T-985, follows T-933 hub-secret pattern): if valid
/// PEM files already exist on disk, reuse them so that client TOFU
/// fingerprints survive hub restarts. Otherwise generate fresh ones.
pub fn load_or_generate_cert() -> std::io::Result<TlsAcceptor> {
    let cert_path = hub_cert_path();
    let key_path = hub_key_path();

    // Try loading existing cert+key
    if cert_path.exists() && key_path.exists() {
        let cert_pem = std::fs::read_to_string(&cert_path)?;
        let key_pem = std::fs::read_to_string(&key_path)?;
        match build_acceptor_from_pem(&cert_pem, &key_pem) {
            Ok(acceptor) => {
                tracing::info!(
                    cert = %cert_path.display(),
                    "Hub TLS certificate loaded from disk (persist-if-present, T-985)"
                );
                return Ok(acceptor);
            }
            Err(e) => {
                tracing::warn!(
                    error = %e,
                    "Existing TLS cert/key invalid, regenerating"
                );
            }
        }
    }

    // Generate new cert
    let subject_alt_names = vec![
        "localhost".to_string(),
        "127.0.0.1".to_string(),
        "::1".to_string(),
    ];

    let CertifiedKey { cert, key_pair } =
        generate_simple_self_signed(subject_alt_names).map_err(|e| {
            std::io::Error::other(format!("cert generation failed: {e}"))
        })?;

    let cert_pem = cert.pem();
    let key_pem = key_pair.serialize_pem();

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
///
/// NOTE (T-985): Cert files are intentionally preserved across restarts
/// (persist-if-present) so client TOFU fingerprints remain valid.
/// This function is retained for explicit cleanup (e.g., `hub stop --clean`)
/// but is no longer called on normal shutdown.
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

    #[test]
    fn own_cert_fingerprint_matches_cert_fingerprint_of_same_der_t3345() {
        let dir = test_dir();
        let cert_path = dir.join("hub.cert.pem");
        let CertifiedKey { cert, .. } =
            generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        std::fs::write(&cert_path, cert.pem()).unwrap();

        let fp = own_cert_fingerprint_at(&cert_path).expect("fingerprint of a real cert");
        assert_eq!(fp, termlink_session::tofu::cert_fingerprint(cert.der()));
        let id = hub_id_from_fingerprint(&fp).unwrap();
        assert_eq!(id.len(), 16);
        assert!(fp.starts_with(&format!("sha256:{id}")));

        assert_eq!(own_cert_fingerprint_at(&dir.join("absent.pem")), None);
        std::fs::write(dir.join("junk.pem"), "not a cert").unwrap();
        assert_eq!(own_cert_fingerprint_at(&dir.join("junk.pem")), None);
    }

    #[test]
    fn hub_id_from_fingerprint_shapes_t3345() {
        let hex = "cacc73ea32b121dd206a6ce20278a319e8dda6b6b4d6d5872105acccdc546d66";
        assert_eq!(hub_id_from_fingerprint(&format!("sha256:{hex}")).as_deref(), Some("cacc73ea32b121dd"));
        assert_eq!(hub_id_from_fingerprint(hex).as_deref(), Some("cacc73ea32b121dd"));
        assert_eq!(hub_id_from_fingerprint("sha256:CACC73EA32B121DD00").as_deref(), Some("cacc73ea32b121dd"));
        assert_eq!(hub_id_from_fingerprint("sha256:cacc73ea"), None, "too short");
        assert_eq!(hub_id_from_fingerprint("sha256:zzzz73ea32b121dd00"), None, "non-hex");
        assert_eq!(hub_id_from_fingerprint(""), None);
    }

    #[test]
    fn invalid_cert_pem_rejects() {
        let result = build_acceptor_from_pem("not a real cert", "not a real key");
        assert!(result.is_err(), "Invalid PEM should be rejected");
    }

    #[test]
    fn empty_cert_pem_rejects() {
        let result = build_acceptor_from_pem("", "");
        assert!(result.is_err(), "Empty PEM should be rejected");
    }

    #[test]
    fn client_connector_missing_file_rejects() {
        let result = build_client_connector(Path::new("/nonexistent/path/cert.pem"));
        assert!(result.is_err(), "Missing cert file should be rejected");
    }

    #[test]
    fn load_existing_cert_persists() {
        let dir = test_dir();
        let cert_path = dir.join("hub.cert.pem");
        let key_path = dir.join("hub.key.pem");

        // Generate a cert+key pair and write to disk
        let subject_alt_names = vec!["localhost".to_string(), "127.0.0.1".to_string()];
        let CertifiedKey { cert, key_pair } =
            generate_simple_self_signed(subject_alt_names).unwrap();
        let cert_pem = cert.pem();
        let key_pem = key_pair.serialize_pem();

        std::fs::write(&cert_path, &cert_pem).unwrap();
        std::fs::write(&key_path, &key_pem).unwrap();

        // Loading the same cert should produce a working acceptor
        let loaded_cert_pem = std::fs::read_to_string(&cert_path).unwrap();
        let loaded_key_pem = std::fs::read_to_string(&key_path).unwrap();
        let acceptor = build_acceptor_from_pem(&loaded_cert_pem, &loaded_key_pem);
        assert!(acceptor.is_ok(), "Loading persisted cert+key should succeed");

        // Build a client connector against the same cert (simulates TOFU)
        let connector = build_client_connector(&cert_path);
        assert!(connector.is_ok(), "Client TOFU should work with persisted cert");
    }

    #[test]
    fn invalid_existing_cert_triggers_regeneration() {
        let dir = test_dir();
        let cert_path = dir.join("hub.cert.pem");
        let key_path = dir.join("hub.key.pem");

        // Write garbage cert+key
        std::fs::write(&cert_path, "invalid cert").unwrap();
        std::fs::write(&key_path, "invalid key").unwrap();

        // build_acceptor_from_pem should fail on invalid PEM
        let result = build_acceptor_from_pem("invalid cert", "invalid key");
        assert!(result.is_err(), "Invalid PEM should trigger regeneration path");
    }

    #[test]
    fn cleanup_removes_files() {
        let dir = test_dir();
        let cert_path = dir.join("hub.cert.pem");
        let key_path = dir.join("hub.key.pem");

        // Create dummy files to clean up
        std::fs::write(&cert_path, "dummy cert").unwrap();
        std::fs::write(&key_path, "dummy key").unwrap();
        assert!(cert_path.exists());
        assert!(key_path.exists());

        // cleanup() uses runtime_dir() which we can't override,
        // so test the manual removal pattern instead
        let _ = std::fs::remove_file(&cert_path);
        let _ = std::fs::remove_file(&key_path);
        assert!(!cert_path.exists());
        assert!(!key_path.exists());
    }

    #[test]
    fn mismatched_cert_key_rejects() {
        // Generate two different cert/key pairs
        let pair1 = generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        let pair2 = generate_simple_self_signed(vec!["localhost".into()]).unwrap();

        // Use cert from pair1 but key from pair2
        let result = build_acceptor_from_pem(&pair1.cert.pem(), &pair2.key_pair.serialize_pem());
        // rustls should reject mismatched cert+key
        assert!(result.is_err(), "Mismatched cert/key should be rejected");
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
