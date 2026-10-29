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
use rustls::crypto::{CryptoProvider, WebPkiSupportedAlgorithms};
use rustls::pki_types::{CertificateDer, ServerName, UnixTime};
use rustls::{DigitallySignedStruct, Error, SignatureScheme};
use sha2::{Digest, Sha256};

/// Return the path to the known_hubs trust store, e.g. `~/.termlink/known_hubs`.
///
/// T-2607: resolves through the unified [`crate::identity_dir::resolve_identity_dir`]
/// ladder (`TERMLINK_IDENTITY_DIR` → `$XDG_STATE_HOME/termlink` → `$HOME/.termlink`
/// → loud UID-namespaced tmp), so the trust store now honors
/// `TERMLINK_IDENTITY_DIR` and never silently lands in a world-writable
/// `/tmp/.termlink` when `HOME` is unset.
pub fn known_hubs_path() -> PathBuf {
    crate::identity_dir::resolve_identity_dir().join("known_hubs")
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

/// Path of the append-only pin-event trail (T-3384, RR-18 interim control).
/// `TERMLINK_PIN_EVENTS_FILE` overrides; default `<identity dir>/pin-events.jsonl`.
pub fn pin_events_path() -> PathBuf {
    match std::env::var_os("TERMLINK_PIN_EVENTS_FILE") {
        Some(p) if !p.is_empty() => PathBuf::from(p),
        // Unit tests must never write the operator's real trail.
        _ if cfg!(test) => std::env::temp_dir()
            .join(format!("tl-pin-events-test-{}.jsonl", std::process::id())),
        _ => crate::identity_dir::resolve_identity_dir().join("pin-events.jsonl"),
    }
}

/// Build one pin-event JSON line (no trailing newline).
fn pin_event_line(verb: &str, address: &str, old: Option<&str>, new: Option<&str>) -> String {
    serde_json::json!({
        "ts": now_utc(),
        "verb": verb,
        "address": address,
        "old_fingerprint": old,
        "new_fingerprint": new,
        "pid": std::process::id(),
        "argv0": std::env::args().next(),
        "agent_id": std::env::var("TERMLINK_AGENT_ID").ok().filter(|v| !v.is_empty()),
    })
    .to_string()
}

/// Fallible core of [`record_pin_event`], split out so tests can assert failure.
fn record_pin_event_to(
    path: &std::path::Path,
    verb: &str,
    address: &str,
    old: Option<&str>,
    new: Option<&str>,
) -> std::io::Result<()> {
    use std::io::Write;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let mut f = std::fs::OpenOptions::new().create(true).append(true).open(path)?;
    // One write call so concurrent appenders do not interleave a line.
    f.write_all((pin_event_line(verb, address, old, new) + "\n").as_bytes())
}

/// Record a pin change (`tofu-clear`, `tofu-clear-all`, `tofu-pin`) in the append-only
/// trail. This is a tripwire, not a gate: a write failure warns on stderr and the
/// operation proceeds. After-the-fact detection only; a same-user attacker can edit
/// the file (RR-18).
pub fn record_pin_event(verb: &str, address: &str, old: Option<&str>, new: Option<&str>) {
    let path = pin_events_path();
    if let Err(e) = record_pin_event_to(&path, verb, address, old, new) {
        eprintln!(
            "warning: failed to record TOFU pin event ({verb} {address}) to {} ({e}); \
             the operation proceeds but this change will not appear in needs-attention",
            path.display()
        );
    }
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
    /// Persist the store, warning LOUDLY on failure (T-2438). A lost
    /// pin write is security-relevant: the in-memory pin protects only
    /// this process — the next invocation would silently re-TOFU the
    /// hub (trust-downgrade window), so the operator must hear about
    /// it. Mutating callers' signatures stay infallible; the warning
    /// is the contract.
    fn save(&self) {
        if let Err(e) = self.save_result() {
            eprintln!(
                "warning: failed to persist TOFU pin store to {} ({e}); \
                 fingerprint pins from this session will NOT survive process exit — \
                 the next connection will re-trust-on-first-use (fix the path/permissions)",
                self.path.display()
            );
        }
    }

    /// Fallible core of [`Self::save`], split out so tests can assert
    /// the failure surfaces (T-2438) without capturing stderr.
    fn save_result(&self) -> std::io::Result<()> {
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
            std::fs::create_dir_all(parent)?;
        }
        // T-2501: write atomically (temp file + rename) instead of
        // truncate-in-place. `std::fs::write` truncates the live file BEFORE
        // writing, so a crash / OOM-kill / power-loss / ENOSPC mid-write leaves
        // `known_hubs` truncated on disk → pins vanish → the next connection
        // silently re-TOFUs (trust-downgrade / MITM window). POSIX rename(2) is
        // atomic within one filesystem, so a reader always sees the complete old
        // or complete new file — never a partial one. Mirrors the crate's
        // established idiom (registration.rs::write_atomic, known_peers.rs).
        // T-2438's Err-path warning still applies: any failure here propagates
        // to save()'s loud eprintln.
        let tmp_path = self.path.with_extension("tmp");
        std::fs::write(&tmp_path, lines.join("\n") + "\n")?;
        std::fs::rename(&tmp_path, &self.path)
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
        let removed = entries.remove(host_port);
        let existed = removed.is_some();
        if let Some(old) = removed {
            record_pin_event("tofu-clear", host_port, Some(&old.fingerprint), None);
            drop(entries);
            self.save();
        }
        existed
    }

    /// Remove all entries. Returns the count removed.
    pub fn clear_all(&self) -> usize {
        let mut entries = self.entries.lock().expect("TOFU store lock poisoned");
        let count = entries.len();
        for (hp, old) in entries.iter() {
            record_pin_event("tofu-clear-all", hp, Some(&old.fingerprint), None);
        }
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

        // New hub — trust on first use (after a `tofu clear` this IS the re-pin)
        record_pin_event("tofu-pin", host_port, None, Some(fingerprint));
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

/// T-3629: signature algorithms used to check the server's handshake signature.
///
/// Taken from the process-default crypto provider (the one `ClientConfig::builder()`
/// uses), falling back to aws-lc-rs, the workspace's only provider (T-2333).
fn signature_algorithms() -> WebPkiSupportedAlgorithms {
    match CryptoProvider::get_default() {
        Some(p) => p.signature_verification_algorithms,
        None => rustls::crypto::aws_lc_rs::default_provider().signature_verification_algorithms,
    }
}

/// A rustls ServerCertVerifier that implements TOFU.
#[derive(Debug)]
pub struct TofuVerifier {
    store: KnownHubStore,
    /// The host:port being connected to (set before each connection).
    host_port: String,
    /// T-3629: verifies the handshake signature against the pinned cert's key.
    algs: WebPkiSupportedAlgorithms,
}

impl TofuVerifier {
    pub fn new(store: KnownHubStore, host_port: String) -> Self {
        Self { store, host_port, algs: signature_algorithms() }
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
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        // T-3629: the pin only covers the public cert; this proves the peer holds its key.
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.algs)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.algs)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algs.supported_schemes()
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
    /// T-3629: the probe still checks the handshake signature, so a reported
    /// fingerprint belongs to an endpoint that holds the cert's private key.
    algs: WebPkiSupportedAlgorithms,
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
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        // T-3629: the pin only covers the public cert; this proves the peer holds its key.
        rustls::crypto::verify_tls12_signature(message, cert, dss, &self.algs)
    }

    fn verify_tls13_signature(
        &self,
        message: &[u8],
        cert: &CertificateDer<'_>,
        dss: &DigitallySignedStruct,
    ) -> Result<HandshakeSignatureValid, Error> {
        rustls::crypto::verify_tls13_signature(message, cert, dss, &self.algs)
    }

    fn supported_verify_schemes(&self) -> Vec<SignatureScheme> {
        self.algs.supported_schemes()
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
    let verifier = ProbeVerifier { captured: Arc::clone(&captured), algs: signature_algorithms() };
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
    fn save_result_errs_on_unwritable_path() {
        // T-2438: a failed pin write must SURFACE, not vanish. Parent
        // "directory" is a regular file, so create_dir_all fails.
        let n = COUNTER.fetch_add(1, Ordering::Relaxed);
        let blocker = PathBuf::from(format!(
            "/tmp/tl-tofu-blocker-{}-{}",
            std::process::id(),
            n
        ));
        let _ = std::fs::remove_file(&blocker);
        std::fs::write(&blocker, "not a dir").unwrap();

        let store = KnownHubStore::new(blocker.join("known_hubs"));
        {
            let mut entries = store.entries.lock().unwrap();
            entries.insert(
                "1.2.3.4:9100".to_string(),
                KnownHub {
                    host_port: "1.2.3.4:9100".to_string(),
                    fingerprint: "sha256:xyz".to_string(),
                    first_seen: "t0".to_string(),
                    last_seen: "t0".to_string(),
                },
            );
        }
        assert!(
            store.save_result().is_err(),
            "unwritable pin store must report the failure"
        );
        let _ = std::fs::remove_file(&blocker);
    }

    #[test]
    fn save_result_persists_and_reloads_pin() {
        // T-2438: happy path — the fallible core actually persists.
        let (store, path) = test_store();
        store.accept("5.6.7.8:9100", "sha256:pin1").unwrap();
        store.save_result().expect("writable store must save");

        let reloaded = KnownHubStore::new(path);
        assert_eq!(
            reloaded.get("5.6.7.8:9100").as_deref(),
            Some("sha256:pin1"),
            "pin must survive a reload"
        );
    }

    #[test]
    fn save_result_is_atomic_no_leftover_temp() {
        // T-2501: the write must be atomic (temp file + rename), not
        // truncate-in-place. After a successful save no sibling temp file may
        // remain, and the pin must round-trip. A regression to `fs::write`
        // (truncate-in-place) would not leave a temp file — but the atomic
        // path is what guarantees a crash mid-write can't truncate the live
        // file; this test locks in the temp+rename mechanism + clean round-trip.
        let (store, path) = test_store();
        store.accept("9.9.9.9:9100", "sha256:atomic").unwrap();
        store.save_result().expect("writable store must save");

        let tmp_path = path.with_extension("tmp");
        assert!(
            !tmp_path.exists(),
            "atomic write must leave no leftover temp file at {}",
            tmp_path.display()
        );
        assert!(path.exists(), "the live pin file must exist after save");

        let reloaded = KnownHubStore::new(path);
        assert_eq!(
            reloaded.get("9.9.9.9:9100").as_deref(),
            Some("sha256:atomic"),
            "pin must survive the atomic write + reload"
        );
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

    #[test]
    fn pin_event_has_fields_and_appends() {
        let dir = std::env::temp_dir().join(format!("tl-pinev-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let f = dir.join("sub").join("pin-events.jsonl");
        record_pin_event_to(&f, "tofu-clear", "h:9100", Some("sha256:aa"), None).unwrap();
        record_pin_event_to(&f, "tofu-pin", "h:9100", None, Some("sha256:bb")).unwrap();
        let text = std::fs::read_to_string(&f).unwrap();
        let lines: Vec<serde_json::Value> =
            text.lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(lines.len(), 2);
        assert_eq!(lines[0]["verb"], "tofu-clear");
        assert_eq!(lines[0]["old_fingerprint"], "sha256:aa");
        assert!(lines[0]["new_fingerprint"].is_null());
        assert_eq!(lines[1]["new_fingerprint"], "sha256:bb");
        for k in ["ts", "address", "pid", "argv0", "agent_id"] {
            assert!(lines[0].get(k).is_some(), "missing {k}");
        }
        let _ = std::fs::remove_dir_all(&dir);
    }

    #[test]
    fn pin_event_failure_does_not_block_operation() {
        let dir = std::env::temp_dir().join(format!("tl-pinev-fail-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let blocker = dir.join("blocker");
        std::fs::write(&blocker, "x").unwrap();
        // Parent is a regular file => the append fails.
        assert!(record_pin_event_to(&blocker.join("e.jsonl"), "v", "h", None, None).is_err());
        // record_pin_event returns () by signature: a failure can only warn, never block.
        let _: () = record_pin_event("tofu-pin", "h:1", None, Some("sha256:cc"));
        let _ = std::fs::remove_dir_all(&dir);
    }

    // ── T-3629: handshake signatures are verified, not assumed ──────────────

    /// Loopback TLS server that presents `cert_der` but signs the handshake
    /// with `key_der`. `CertifiedKey::new` does not check that they match, so a
    /// mismatched pair models an endpoint replaying a hub's public certificate
    /// without holding its private key.
    async fn spawn_tls_server(cert_der: Vec<u8>, key_der: Vec<u8>) -> std::net::SocketAddr {
        use rustls::pki_types::{PrivateKeyDer, PrivatePkcs8KeyDer};
        use rustls::server::{ClientHello, ResolvesServerCert};
        use rustls::sign::CertifiedKey;

        #[derive(Debug)]
        struct Fixed(Arc<CertifiedKey>);
        impl ResolvesServerCert for Fixed {
            fn resolve(&self, _hello: ClientHello<'_>) -> Option<Arc<CertifiedKey>> {
                Some(Arc::clone(&self.0))
            }
        }

        let key = PrivateKeyDer::Pkcs8(PrivatePkcs8KeyDer::from(key_der));
        let signer = rustls::crypto::aws_lc_rs::sign::any_supported_type(&key).unwrap();
        let ck = CertifiedKey::new(vec![CertificateDer::from(cert_der)], signer);
        let config = rustls::ServerConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_safe_default_protocol_versions()
        .unwrap()
        .with_no_client_auth()
        .with_cert_resolver(Arc::new(Fixed(Arc::new(ck))));
        let acceptor = tokio_rustls::TlsAcceptor::from(Arc::new(config));

        let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.unwrap();
        let addr = listener.local_addr().unwrap();
        tokio::spawn(async move {
            while let Ok((tcp, _)) = listener.accept().await {
                let acceptor = acceptor.clone();
                tokio::spawn(async move {
                    if let Ok(mut tls) = acceptor.accept(tcp).await {
                        use tokio::io::AsyncWriteExt;
                        let _ = tls.write_all(b"ok").await;
                        let _ = tls.shutdown().await;
                    }
                });
            }
        });
        addr
    }

    /// TOFU handshake to `addr` with the server cert already pinned in a temp
    /// store, restricted to one TLS version.
    async fn tofu_handshake(
        addr: std::net::SocketAddr,
        pinned_fp: &str,
        version: &'static rustls::SupportedProtocolVersion,
    ) -> std::io::Result<()> {
        let (store, _path) = test_store();
        let host_port = addr.to_string();
        store.accept(&host_port, pinned_fp).unwrap();
        let verifier = TofuVerifier::new(store, host_port);
        let config = rustls::ClientConfig::builder_with_provider(Arc::new(
            rustls::crypto::aws_lc_rs::default_provider(),
        ))
        .with_protocol_versions(&[version])
        .unwrap()
        .dangerous()
        .with_custom_certificate_verifier(Arc::new(verifier))
        .with_no_client_auth();
        let connector = tokio_rustls::TlsConnector::from(Arc::new(config));
        let tcp = tokio::net::TcpStream::connect(addr).await?;
        let name = ServerName::try_from("localhost".to_string()).unwrap();
        let mut tls = connector.connect(name, tcp).await?;
        // Reading to EOF makes sure every handshake message has been processed.
        use tokio::io::AsyncReadExt;
        let mut buf = Vec::new();
        tls.read_to_end(&mut buf).await?;
        Ok(())
    }

    fn self_signed() -> (Vec<u8>, Vec<u8>) {
        let ck = rcgen::generate_simple_self_signed(vec!["localhost".to_string()]).unwrap();
        (ck.cert.der().to_vec(), ck.key_pair.serialize_der())
    }

    #[tokio::test]
    async fn t3629_tofu_handshake_with_matching_key_succeeds() {
        let (cert, key) = self_signed();
        let fp = cert_fingerprint(&cert);
        let addr = spawn_tls_server(cert, key).await;
        for v in [&rustls::version::TLS13, &rustls::version::TLS12] {
            tofu_handshake(addr, &fp, v)
                .await
                .unwrap_or_else(|e| panic!("genuine hub must pass ({v:?}): {e}"));
        }
    }

    #[tokio::test]
    async fn t3629_pinned_cert_signed_with_other_key_is_rejected() {
        let (cert, _real_key) = self_signed();
        let (_other_cert, other_key) = self_signed();
        let fp = cert_fingerprint(&cert);
        let addr = spawn_tls_server(cert, other_key).await;
        for v in [&rustls::version::TLS13, &rustls::version::TLS12] {
            let res = tofu_handshake(addr, &fp, v).await;
            assert!(
                res.is_err(),
                "replayed pinned cert without its private key must fail the handshake ({v:?})"
            );
        }
    }

    #[tokio::test]
    async fn t3629_probe_rejects_cert_signed_with_other_key() {
        let (cert, _real_key) = self_signed();
        let (_other_cert, other_key) = self_signed();
        let addr = spawn_tls_server(cert, other_key).await;
        let res = super::probe_cert(&addr.to_string()).await;
        assert!(res.is_err(), "probe must not report a fingerprint the endpoint cannot sign for");
    }

    #[tokio::test]
    async fn t3629_probe_with_matching_key_returns_fingerprint() {
        let (cert, key) = self_signed();
        let fp = cert_fingerprint(&cert);
        let addr = spawn_tls_server(cert, key).await;
        let (_, got) = super::probe_cert(&addr.to_string()).await.unwrap();
        assert_eq!(got, fp);
    }

    #[test]
    fn t3629_verify_schemes_come_from_provider() {
        let (store, _path) = test_store();
        let v = TofuVerifier::new(store, "h:1".into());
        assert_eq!(v.supported_verify_schemes(), signature_algorithms().supported_schemes());
        assert!(!v.supported_verify_schemes().is_empty());
    }
}
