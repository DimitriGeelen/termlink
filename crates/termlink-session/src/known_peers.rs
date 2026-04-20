//! Known-peer keyring with TOFU semantics (T-1159).
//!
//! On first observation of a peer's ed25519 public key, `learn()` pins it.
//! Subsequent `learn()` calls with a different key for the same `peer_id`
//! return `TofuViolation` — the caller logs and decides whether to rotate
//! the pin or reject the message. Mirrors `termlink_session::tofu` but for
//! agent identity rather than hub transport.

use std::collections::BTreeMap;
use std::fs;
use std::io::Write;
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use ed25519_dalek::{Signature, VerifyingKey};
use serde::{Deserialize, Serialize};
use thiserror::Error;

use crate::agent_identity::{Identity, fingerprint_of};

#[derive(Debug, Error)]
pub enum PeersError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("toml serialize: {0}")]
    TomlSer(#[from] toml::ser::Error),
    #[error("toml parse: {0}")]
    TomlDe(#[from] toml::de::Error),
    #[error("TOFU violation for peer {peer_id:?}: pinned {pinned} but got {got}")]
    TofuViolation {
        peer_id: String,
        pinned: String,
        got: String,
    },
    #[error("invalid public key hex: {0}")]
    BadPubKey(String),
    #[error("unknown peer {0:?}")]
    UnknownPeer(String),
}

pub type Result<T> = std::result::Result<T, PeersError>;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PeerEntry {
    pub pubkey_hex: String,
    pub first_seen_ms: i64,
    pub last_seen_ms: i64,
    pub fingerprint: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
struct PeersFile {
    peers: BTreeMap<String, PeerEntry>,
}

/// TOFU keyring for peer agent public keys, persisted as TOML.
pub struct KnownPeers {
    path: PathBuf,
    inner: PeersFile,
}

impl KnownPeers {
    /// Open (or create) the keyring at `<base>/known_peers.toml`. Missing
    /// files are an empty keyring — they are materialized on first save.
    pub fn open(base: &Path) -> Result<Self> {
        let path = base.join("known_peers.toml");
        let inner = match fs::read_to_string(&path) {
            Ok(s) => toml::from_str(&s)?,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => PeersFile::default(),
            Err(e) => return Err(PeersError::Io(e)),
        };
        Ok(Self { path, inner })
    }

    /// Learn a peer's public key via TOFU. First observation pins; later
    /// calls with the same hex are a no-op; later calls with a different
    /// hex return `TofuViolation`.
    pub fn learn(&mut self, peer_id: &str, pubkey_hex: &str) -> Result<()> {
        let now = now_unix_ms();
        if let Some(existing) = self.inner.peers.get_mut(peer_id) {
            if existing.pubkey_hex != pubkey_hex {
                return Err(PeersError::TofuViolation {
                    peer_id: peer_id.to_string(),
                    pinned: existing.fingerprint.clone(),
                    got: fingerprint_hex(pubkey_hex),
                });
            }
            existing.last_seen_ms = now;
        } else {
            self.inner.peers.insert(
                peer_id.to_string(),
                PeerEntry {
                    pubkey_hex: pubkey_hex.to_string(),
                    first_seen_ms: now,
                    last_seen_ms: now,
                    fingerprint: fingerprint_hex(pubkey_hex),
                },
            );
        }
        self.save()
    }

    /// Fetch a peer's entry.
    pub fn get(&self, peer_id: &str) -> Option<&PeerEntry> {
        self.inner.peers.get(peer_id)
    }

    /// Verify a signature against a pinned peer's key.
    pub fn verify_from(&self, peer_id: &str, msg: &[u8], sig: &Signature) -> Result<bool> {
        let entry = self
            .inner
            .peers
            .get(peer_id)
            .ok_or_else(|| PeersError::UnknownPeer(peer_id.to_string()))?;
        let pk = parse_pubkey_hex(&entry.pubkey_hex)?;
        Ok(Identity::verify(&pk, msg, sig))
    }

    /// All pinned peer ids, sorted.
    pub fn peer_ids(&self) -> Vec<String> {
        self.inner.peers.keys().cloned().collect()
    }

    fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let tmp = self.path.with_extension("toml.tmp");
        let body = toml::to_string_pretty(&self.inner)?;
        {
            let mut f = fs::OpenOptions::new()
                .create(true)
                .truncate(true)
                .write(true)
                .mode(0o600)
                .open(&tmp)?;
            f.write_all(body.as_bytes())?;
            f.flush()?;
        }
        let mut perms = fs::metadata(&tmp)?.permissions();
        perms.set_mode(0o600);
        fs::set_permissions(&tmp, perms)?;
        fs::rename(&tmp, &self.path)?;
        Ok(())
    }
}

fn parse_pubkey_hex(hex: &str) -> Result<VerifyingKey> {
    if hex.len() != 64 {
        return Err(PeersError::BadPubKey(format!(
            "expected 64 hex chars, got {}",
            hex.len()
        )));
    }
    let mut bytes = [0u8; 32];
    for i in 0..32 {
        let b = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
            .map_err(|_| PeersError::BadPubKey("non-hex byte".into()))?;
        bytes[i] = b;
    }
    VerifyingKey::from_bytes(&bytes).map_err(|e| PeersError::BadPubKey(e.to_string()))
}

fn fingerprint_hex(pubkey_hex: &str) -> String {
    match parse_pubkey_hex(pubkey_hex) {
        Ok(pk) => fingerprint_of(&pk),
        Err(_) => pubkey_hex
            .chars()
            .take(16)
            .collect::<String>(),
    }
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn empty_on_missing_file() {
        let tmp = TempDir::new().unwrap();
        let kp = KnownPeers::open(tmp.path()).unwrap();
        assert!(kp.peer_ids().is_empty());
    }

    #[test]
    fn learn_pins_on_first_observation() {
        let tmp = TempDir::new().unwrap();
        let mut kp = KnownPeers::open(tmp.path()).unwrap();
        let ident = Identity::generate();
        kp.learn("agent-A", &ident.public_key_hex()).unwrap();
        let entry = kp.get("agent-A").unwrap();
        assert_eq!(entry.pubkey_hex, ident.public_key_hex());
        assert_eq!(entry.fingerprint.len(), 16);
    }

    #[test]
    fn learn_same_key_is_idempotent() {
        let tmp = TempDir::new().unwrap();
        let mut kp = KnownPeers::open(tmp.path()).unwrap();
        let ident = Identity::generate();
        kp.learn("agent-A", &ident.public_key_hex()).unwrap();
        kp.learn("agent-A", &ident.public_key_hex()).unwrap();
        assert_eq!(kp.peer_ids().len(), 1);
    }

    #[test]
    fn learn_different_key_is_tofu_violation() {
        let tmp = TempDir::new().unwrap();
        let mut kp = KnownPeers::open(tmp.path()).unwrap();
        let first = Identity::generate();
        let second = Identity::generate();
        kp.learn("agent-A", &first.public_key_hex()).unwrap();
        let err = kp.learn("agent-A", &second.public_key_hex()).unwrap_err();
        assert!(matches!(err, PeersError::TofuViolation { .. }));
    }

    #[test]
    fn verify_from_known_peer() {
        let tmp = TempDir::new().unwrap();
        let mut kp = KnownPeers::open(tmp.path()).unwrap();
        let ident = Identity::generate();
        kp.learn("agent-A", &ident.public_key_hex()).unwrap();
        let msg = b"hello";
        let sig = ident.sign(msg);
        assert!(kp.verify_from("agent-A", msg, &sig).unwrap());
        assert!(!kp.verify_from("agent-A", b"tampered", &sig).unwrap());
    }

    #[test]
    fn verify_from_unknown_peer_errors() {
        let tmp = TempDir::new().unwrap();
        let kp = KnownPeers::open(tmp.path()).unwrap();
        let ident = Identity::generate();
        let sig = ident.sign(b"x");
        let err = kp.verify_from("nobody", b"x", &sig).unwrap_err();
        assert!(matches!(err, PeersError::UnknownPeer(_)));
    }

    #[test]
    fn persists_across_open() {
        let tmp = TempDir::new().unwrap();
        let ident = Identity::generate();
        {
            let mut kp = KnownPeers::open(tmp.path()).unwrap();
            kp.learn("agent-A", &ident.public_key_hex()).unwrap();
        }
        let kp = KnownPeers::open(tmp.path()).unwrap();
        assert_eq!(kp.peer_ids(), vec!["agent-A".to_string()]);
        // File is chmod 600.
        let mode = fs::metadata(tmp.path().join("known_peers.toml"))
            .unwrap()
            .permissions()
            .mode()
            & 0o777;
        assert_eq!(mode, 0o600);
    }
}
