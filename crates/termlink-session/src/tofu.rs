//! TOFU (Trust On First Use) TLS certificate verifier for cross-hub connections.
//!
//! On first connection to a remote hub, the certificate fingerprint is accepted and
//! stored in `~/.termlink/known_hubs`. On subsequent connections, the fingerprint is
//! verified against the stored value. If it changes, the connection is rejected
//! (like SSH's known_hosts).

use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use rustls::client::danger::{HandshakeSignatureValid, ServerCertVerified, ServerCertVerifier};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use sha2::{Digest, Sha256};

/// Return the path to the known_hubs file: `~/.termlink/known_hubs`.
pub fn known_hubs_path() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(home).join(".termlink").join("known_hubs")
}

/// Compute SHA-256 fingerprint of a DER-encoded certificate.
pub fn cert_fingerprint(cert_der: &[u8]) -> String {
    let hash = Sha256::digest(cert_der);
    let hex: String = hash.iter().map(|b| format!("{b:02x}")).collect();
    format!("sha256:{hex}")
}

/// Simple UTC timestamp string (ISO 8601, no external deps).
fn now_utc() -> String {
    let dur = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default();
    let secs = dur.as_secs();
    // Approximate UTC breakdown (no leap second handling — good enough for timestamps)
    let days = secs / 86400;
    let time_secs = secs % 86400;
    let h = time_secs / 3600;
    let m = (time_secs % 3600) / 60;
    let s = time_secs % 60;
    // Days since epoch to Y-M-D (simplified)
    let mut y = 1970i64;
    let mut remaining = days as i64;
    loop {
        let ydays = if y % 4 == 0 && (y % 100 != 0 || y % 400 == 0) { 366 } else { 365 };
        if remaining < ydays { break; }
        remaining -= ydays;
        y += 1;
    }
    let leap = y % 4 == 0 && (y % 100 != 0 || y % 400 == 0);
    let mdays = [31, if leap { 29 } else { 28 }, 31, 30, 31, 30, 31, 31, 30, 31, 30, 31];
    let mut mo = 0usize;
    for (i, &md) in mdays.iter().enumerate() {
        if remaining < md as i64 { mo = i; break; }
        remaining -= md as i64;
    }
    format!("{y:04}-{:02}-{:02}T{h:02}:{m:02}:{s:02}Z", mo + 1, remaining + 1)
}

/// An entry in the known_hubs store.
#[derive(Clone, Debug)]
pub struct KnownHub {
    pub host_port: String,
    pub fingerprint: String,
    pub first_seen: String,
    pub last_seen: String,
}

/// In-memory + file-backed store for known hub fingerprints.
#[derive(Clone, Debug)]
pub struct KnownHubStore {
    entries: Arc<Mutex<HashMap<String, KnownHub>>>,
    path: PathBuf,
}

impl KnownHubStore {
    /// Create a store backed by the given file path.
    pub fn new(path: PathBuf) -> Self {
        let mut store = Self {
            entries: Arc::new(Mutex::new(HashMap::new())),
            path,
        };
        store.load();
        store
    }

    /// Create a store at the default path (`~/.termlink/known_hubs`).
    pub fn default_store() -> Self {
        Self::new(known_hubs_path())
    }

    /// Load entries from disk. Silently ignores missing or malformed files.
    fn load(&mut self) {
        let content = match std::fs::read_to_string(&self.path) {
            Ok(c) => c,
            Err(_) => return,
        };

        let mut entries = self.entries.lock().expect("TOFU store lock poisoned");
        for line in content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            let parts: Vec<&str> = line.split_whitespace().collect();
            if parts.len() >= 4 {
                entries.insert(
                    parts[0].to_string(),
                    KnownHub {
                        host_port: parts[0].to_string(),
                        fingerprint: parts[1].to_string(),
                        first_seen: parts[2].to_string(),
                        last_seen: parts[3].to_string(),
                    },
                );
            }
        }
    }

    /// Write all entries to disk.
    fn save(&self) {
        let entries = self.entries.lock().expect("TOFU store lock poisoned");
        let mut lines = Vec::new();
        lines.push("# TermLink known hubs (TOFU)".to_string());
        lines.push("# host:port fingerprint first_seen last_seen".to_string());
        for entry in entries.values() {
            lines.push(format!(
                "{} {} {} {}",
                entry.host_port, entry.fingerprint, entry.first_seen, entry.last_seen
            ));
        }

        // Ensure parent dir exists
        if let Some(parent) = self.path.parent() {
            let _ = std::fs::create_dir_all(parent);
        }
        let _ = std::fs::write(&self.path, lines.join("\n") + "\n");
    }

    /// Look up a stored fingerprint for a host:port.
    pub fn get(&self, host_port: &str) -> Option<String> {
        self.entries
            .lock()
            .expect("TOFU store lock poisoned")
            .get(host_port)
            .map(|e| e.fingerprint.clone())
    }

    /// List all known hub entries.
    pub fn list_all(&self) -> Vec<KnownHub> {
        let entries = self.entries.lock().expect("TOFU store lock poisoned");
        let mut hubs: Vec<KnownHub> = entries.values().cloned().collect();
        hubs.sort_by(|a, b| a.host_port.cmp(&b.host_port));
        hubs
    }

    /// Remove a specific host:port entry. Returns true if it existed.
    pub fn remove(&self, host_port: &str) -> bool {
        let mut entries = self.entries.lock().expect("TOFU store lock poisoned");
        let existed = entries.remove(host_port).is_some();
        if existed {
            drop(entries);
            self.save();
        }
        existed
    }

    /// Remove all entries. Returns the count removed.
    pub fn clear_all(&self) -> usize {
        let mut entries = self.entries.lock().expect("TOFU store lock poisoned");
        let count = entries.len();
        entries.clear();
        drop(entries);
        self.save();
        count
    }

    /// Store or update a fingerprint. Returns `Ok(true)` if new, `Ok(false)` if updated,
    /// `Err` if fingerprint changed (MITM).
    pub fn accept(&self, host_port: &str, fingerprint: &str) -> Result<bool, String> {
        let mut entries = self.entries.lock().expect("TOFU store lock poisoned");
        let now = now_utc();

        if let Some(existing) = entries.get_mut(host_port) {
            if existing.fingerprint == fingerprint {
                // Known and matching — update last_seen
                existing.last_seen = now;
                drop(entries);
                self.save();
                return Ok(false);
            } else {
                // FINGERPRINT CHANGED — potential MITM
                return Err(format!(
                    "TOFU VIOLATION: Hub {} fingerprint changed!\n  Expected: {}\n  Got:      {}\n  \
                     This could indicate a man-in-the-middle attack or hub cert regeneration.\n  \
                     To accept the new cert: termlink tofu clear {}",
                    host_port,
                    existing.fingerprint,
                    fingerprint,
                    host_port,
                ));
            }
        }

        // New hub — trust on first use
        entries.insert(
            host_port.to_string(),
            KnownHub {
                host_port: host_port.to_string(),
                fingerprint: fingerprint.to_string(),
                first_seen: now.clone(),
                last_seen: now,
            },
        );
        drop(entries);
        self.save();
        Ok(true)
    }
}

/// A rustls ServerCertVerifier that implements TOFU.
#[derive(Debug)]
pub struct TofuVerifier {
    store: KnownHubStore,
    /// The host:port being connected to (set before each connection).
    host_port: String,
}

impl TofuVerifier {
    pub fn new(store: KnownHubStore, host_port: String) -> Self {
        Self { store, host_port }
    }
}

impl ServerCertVerifier for TofuVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        let fp = cert_fingerprint(end_entity.as_ref());

        match self.store.accept(&self.host_port, &fp) {
            Ok(is_new) => {
                if is_new {
                    tracing::info!(
                        host = %self.host_port,
                        fingerprint = %fp,
                        "TOFU: Trusted new hub certificate"
                    );
                } else {
                    tracing::debug!(
                        host = %self.host_port,
                        "TOFU: Known hub, fingerprint matches"
                    );
                }
                Ok(ServerCertVerified::assertion())
            }
            Err(msg) => {
                tracing::error!("{}", msg);
                Err(Error::General(msg))
            }
        }
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        // TOFU trusts the cert — accept any valid TLS signature
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }

    fn root_hint_subjects(&self) -> Option<&[rustls::DistinguishedName]> {
        Some(&[])
    }
}

/// Build a TLS connector that uses TOFU verification for a specific host:port.
pub fn build_tofu_connector(host_port: &str) -> tokio_rustls::TlsConnector {
    let store = KnownHubStore::default_store();
    let verifier = TofuVerifier::new(store, host_port.to_string());

    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();

    tokio_rustls::TlsConnector::from(Arc::new(config))
}

/// T-1658: capturing verifier — accepts any cert and stashes the leaf DER.
///
/// Used by `probe_cert()` to extract the remote hub's certificate during a
/// TLS handshake WITHOUT pinning, WITHOUT auth, WITHOUT mutating
/// `KnownHubStore`. The captured DER is read once by the caller after the
/// handshake completes.
#[derive(Debug)]
struct ProbeVerifier {
    captured: Arc<Mutex<Option<Vec<u8>>>>,
}

impl ServerCertVerifier for ProbeVerifier {
    fn verify_server_cert(
        &self,
        end_entity: &CertificateDer<'_>,
        _intermediates: &[CertificateDer<'_>],
        _server_name: &ServerName<'_>,
        _ocsp_response: &[u8],
        _now: UnixTime,
    ) -> Result<ServerCertVerified, Error> {
        let der = end_entity.as_ref().to_vec();
        if let Ok(mut slot) = self.captured.lock() {
            *slot = Some(der);
        }
        Ok(ServerCertVerified::assertion())
    }

    fn verify_tls12_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn verify_tls13_signature(
        &self,
        _message: &[u8],
        _cert: &CertificateDer<'_>,
        _dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        Ok(HandshakeSignatureValid::assertion())
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        vec![
            SignatureScheme::RSA_PKCS1_SHA256,
            SignatureScheme::RSA_PKCS1_SHA384,
            SignatureScheme::RSA_PKCS1_SHA512,
            SignatureScheme::ECDSA_NISTP256_SHA256,
            SignatureScheme::ECDSA_NISTP384_SHA384,
            SignatureScheme::ECDSA_NISTP521_SHA512,
            SignatureScheme::RSA_PSS_SHA256,
            SignatureScheme::RSA_PSS_SHA384,
            SignatureScheme::RSA_PSS_SHA512,
            SignatureScheme::ED25519,
            SignatureScheme::ED448,
        ]
    }

    fn root_hint_subjects(&self) -> Option<&[rustls::DistinguishedName]> {
        Some(&[])
    }
}

/// T-1658: open TCP to `addr`, complete a TLS handshake accepting any cert,
/// return the remote leaf cert's DER bytes and its `sha256:<hex>` fingerprint.
///
/// Companion to the local-side `hub fingerprint` (T-1657). Does NOT mutate
/// `~/.termlink/known_hubs` — pure read-only diagnostic. `addr` must be
/// `host:port` (e.g. `192.168.10.122:9100`). Accepts IPv4 and DNS names.
///
/// The TLS server name sent in the handshake is the host portion of `addr`
/// or `localhost` if `addr` is a bare IP — most hub certs are self-signed
/// CN=localhost so SNI rarely matters, but we set it for completeness.
pub async fn probe_cert(addr: &str) -> Result<(Vec<u8>, String), String> {
    // Split host:port — accept IPv4-style addresses only (no IPv6 brackets);
    // hub addresses in the wild are all host:port with at most one ':'.
    let (host, _port) = match addr.rsplit_once(':') {
        Some((h, p)) if !h.is_empty() && !p.is_empty() => (h, p),
        _ => return Err(format!(
            "malformed address '{addr}' — expected host:port (e.g. 192.168.10.122:9100)"
        )),
    };

    // TCP connect
    let stream = tokio::net::TcpStream::connect(addr)
        .await
        .map_err(|e| format!("TCP connect to {addr} failed: {e}"))?;

    // Build a one-shot capturing connector
    let captured: Arc<Mutex<Option<Vec<u8>>>> = Arc::new(Mutex::new(None));
    let verifier = ProbeVerifier { captured: Arc::clone(&captured) };
    let config = rustls::ClientConfig::builder()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
    let connector = tokio_rustls::TlsConnector::from(Arc::new(config));

    // ServerName for SNI — host portion of addr; if it's a bare IP rustls is
    // happy to accept it. Fall back to "localhost" if conversion fails (this
    // is extremely unlikely; rustls's IpAddr/DnsName accept any non-empty
    // string that parses as either).
    let server_name = ServerName::try_from(host.to_string())
        .or_else(|_| ServerName::try_from("localhost".to_string()))
        .map_err(|e| format!("invalid SNI for '{host}': {e}"))?;

    // Drive the handshake — we don't care about the resulting stream,
    // only the captured cert.
    let _tls = connector
        .connect(server_name, stream)
        .await
        .map_err(|e| format!("TLS handshake to {addr} failed: {e}"))?;

    let der = captured
        .lock()
        .map_err(|e| format!("verifier lock poisoned: {e}"))?
        .take()
        .ok_or_else(|| format!("no cert captured from {addr} (handshake completed without verifier call)"))?;

    let fp = cert_fingerprint(&der);
    Ok((der, fp))
}

/// T-1675: timeout-bounded wrapper around [`probe_cert`].
///
/// Without this, a probe of an unreachable host holds open the underlying
/// `tokio::net::TcpStream::connect` for the OS TCP retry budget (30–60+s),
/// which when used in parallel-spawn paths (e.g. `fleet doctor
/// --include-pin-check`, `fleet verify`) determines slowest-probe and
/// therefore total wall time. With this wrapper, slowest-probe is bounded
/// to `timeout`.
///
/// On timeout, returns an `Err` whose message contains the literal `timeout`
/// substring and the elapsed seconds, so downstream classification (e.g.
/// `probe-fail`) can preserve operator-visible error provenance.
pub async fn probe_cert_with_timeout(
    addr: &str,
    timeout: std::time::Duration,
) -> Result<(Vec<u8>, String), String> {
    match tokio::time::timeout(timeout, probe_cert(addr)).await {
        Ok(inner) => inner,
        Err(_) => Err(format!(
            "TLS probe to {addr} timeout after {}s",
            timeout.as_secs()
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicU32, Ordering};

    static COUNTER: AtomicU32 = AtomicU32::new(0);

    fn test_store() -> (KnownHubStore, PathBuf) {
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let dir = PathBuf::from(format!(
            "/tmp/tl-tofu-{}-{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let path = dir.join("known_hubs");
        (KnownHubStore::new(path.clone()), path)
    }

    #[test]
    fn tofu_accepts_new_cert() {
        let (store, _path) = test_store();
        let result = store.accept("192.168.1.1:9100", "sha256:abc123");
        assert!(result.is_ok());
        assert!(result.unwrap()); // is_new = true
    }

    #[test]
    fn tofu_accepts_known_cert() {
        let (store, _path) = test_store();
        store.accept("10.0.0.1:9100", "sha256:def456").unwrap();
        let result = store.accept("10.0.0.1:9100", "sha256:def456");
        assert!(result.is_ok());
        assert!(!result.unwrap()); // is_new = false (update)
    }

    #[test]
    fn tofu_rejects_changed_fingerprint() {
        let (store, _path) = test_store();
        store.accept("10.0.0.1:9100", "sha256:original").unwrap();
        let result = store.accept("10.0.0.1:9100", "sha256:changed");
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("TOFU VIOLATION"));
    }

    #[test]
    fn store_persists_to_disk() {
        let (store, path) = test_store();
        store.accept("host1:9100", "sha256:fp1").unwrap();
        store.accept("host2:9200", "sha256:fp2").unwrap();

        // Reload from disk
        let store2 = KnownHubStore::new(path);
        assert_eq!(store2.get("host1:9100"), Some("sha256:fp1".to_string()));
        assert_eq!(store2.get("host2:9200"), Some("sha256:fp2".to_string()));
    }

    #[test]
    fn cert_fingerprint_deterministic() {
        let cert_bytes = b"fake-cert-data-for-testing";
        let fp1 = cert_fingerprint(cert_bytes);
        let fp2 = cert_fingerprint(cert_bytes);
        assert_eq!(fp1, fp2);
        assert!(fp1.starts_with("sha256:"));
    }

    #[test]
    fn verifier_accepts_unknown_cert() {
        let (store, _path) = test_store();
        let verifier = TofuVerifier::new(store, "test:9100".to_string());
        let cert_der = CertificateDer::from(vec![1u8, 2, 3, 4]);
        let server_name = ServerName::try_from("test").unwrap();
        let result = verifier.verify_server_cert(
            &cert_der,
            &[],
            &server_name,
            &[],
            UnixTime::now(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn verifier_rejects_changed_cert() {
        let (store, _path) = test_store();

        // First connection
        let fp = cert_fingerprint(&[1, 2, 3, 4]);
        store.accept("test:9100", &fp).unwrap();

        // Second connection with different cert
        let verifier = TofuVerifier::new(store, "test:9100".to_string());
        let different_cert = CertificateDer::from(vec![5u8, 6, 7, 8]);
        let server_name = ServerName::try_from("test").unwrap();
        let result = verifier.verify_server_cert(
            &different_cert,
            &[],
            &server_name,
            &[],
            UnixTime::now(),
        );
        assert!(result.is_err());
    }

    #[test]
    fn list_all_returns_sorted() {
        let (store, _path) = test_store();
        store.accept("z-host:9100", "sha256:zzz").unwrap();
        store.accept("a-host:9100", "sha256:aaa").unwrap();
        store.accept("m-host:9100", "sha256:mmm").unwrap();

        let entries = store.list_all();
        assert_eq!(entries.len(), 3);
        assert_eq!(entries[0].host_port, "a-host:9100");
        assert_eq!(entries[1].host_port, "m-host:9100");
        assert_eq!(entries[2].host_port, "z-host:9100");
    }

    #[test]
    fn remove_existing_entry() {
        let (store, _path) = test_store();
        store.accept("host1:9100", "sha256:fp1").unwrap();
        store.accept("host2:9200", "sha256:fp2").unwrap();

        assert!(store.remove("host1:9100"));
        assert_eq!(store.list_all().len(), 1);
        assert!(store.get("host1:9100").is_none());
        assert!(store.get("host2:9200").is_some());
    }

    #[test]
    fn remove_nonexistent_returns_false() {
        let (store, _path) = test_store();
        assert!(!store.remove("nonexistent:9100"));
    }

    #[test]
    fn clear_all_empties_store() {
        let (store, path) = test_store();
        store.accept("h1:9100", "sha256:f1").unwrap();
        store.accept("h2:9200", "sha256:f2").unwrap();
        store.accept("h3:9300", "sha256:f3").unwrap();

        assert_eq!(store.clear_all(), 3);
        assert_eq!(store.list_all().len(), 0);

        // Verify persisted to disk
        let store2 = KnownHubStore::new(path);
        assert_eq!(store2.list_all().len(), 0);
    }

    #[test]
    fn remove_persists_to_disk() {
        let (store, path) = test_store();
        store.accept("h1:9100", "sha256:f1").unwrap();
        store.accept("h2:9200", "sha256:f2").unwrap();

        store.remove("h1:9100");

        let store2 = KnownHubStore::new(path);
        assert_eq!(store2.list_all().len(), 1);
        assert_eq!(store2.get("h2:9200"), Some("sha256:f2".to_string()));
    }

    // T-1675: probe_cert_with_timeout returns a timeout error whose message
    // contains the literal "timeout" substring and the elapsed-seconds value
    // when the inner probe outlives the bound. Use a routeable but
    // non-connectable address (RFC 5737 TEST-NET-1 192.0.2.0/24, reserved for
    // documentation — guaranteed to drop the SYN at any internet boundary)
    // and a 1s bound so the test stays under cargo's default per-test budget.
    #[tokio::test]
    async fn probe_cert_with_timeout_errors_on_unreachable() {
        let res = super::probe_cert_with_timeout(
            "192.0.2.1:9100",
            std::time::Duration::from_secs(1),
        )
        .await;
        let err = res.expect_err("expected probe to fail (unreachable + 1s bound)");
        // Either the timeout fires first (slow networks) or the OS bails
        // sooner with EHOSTUNREACH/ECONNREFUSED; both are acceptable failure
        // paths for the operator. Assert the timeout-path produces the
        // documented substring iff it fires.
        if err.contains("timeout") {
            assert!(err.contains("1s"), "timeout error should include duration: {err}");
            assert!(err.contains("192.0.2.1"), "timeout error should include addr: {err}");
        }
    }
}
