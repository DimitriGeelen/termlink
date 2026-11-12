//! Per-agent X25519 encryption keys, the register check at start, and the TLBLOB wrapper (B-11, K-24).
//!
//! 06a AL-25: the private key lives in the agent's store (`<store>/sidecar/<agent>/keys/x25519.key`, 0600,
//! 64 hex) and its public half in the root-owned register `/etc/termlink-sidecar/keys/<agent>.x25519.pub`.
//! A register entry that is missing, not root-owned or group/world-writable refuses start (FCW-110); a private
//! key whose public half differs from the register refuses start with both fingerprints (FCW-109). Production
//! never mints a key (the P7-2 rule): provisioning writes both halves.
//!
//! 06a AL-26 step 4 (format version 2, CR-6.14): a random content key encrypts the plaintext with
//! XChaCha20-Poly1305 under associated data over the immutable envelope fields; the content key is sealed to the
//! recipient's X25519 key (libsodium sealed box, 80 bytes). The object layout is `termlink_protocol::tlblob`.
//!
//! Rollback (08-plan P-6): an agent without a configured register runs with `keys.encryption` "not provisioned"
//! and every blob path refuses; nothing is ever sent in plaintext and no working sidecar is switched off (NID).

use chacha20poly1305::aead::{Aead, KeyInit, Payload};
use chacha20poly1305::{XChaCha20Poly1305, XNonce};
use crypto_box::{PublicKey, SecretKey};
use rand_core::{OsRng, RngCore};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::io::Write;
use std::os::unix::fs::{MetadataExt, OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};
use termlink_protocol::tlblob;

/// Domain string at the head of the associated data (06a 2.10.h).
pub const AAD_DOMAIN: &[u8] = b"termlink.blob/2\n";
/// Sealed-box length of a 32-byte content key: ephemeral public key 32 + tag 16 + key 32.
pub const WRAPPED_KEY_LEN: usize = 80;

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn unhex32(s: &str) -> Option<[u8; 32]> {
    if s.len() != 64 {
        return None;
    }
    let v: Option<Vec<u8>> = (0..64).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect();
    v?.try_into().ok()
}

/// Fingerprint of a public key: the first 16 hex of its sha256 (as AL-18 for signing keys).
pub fn fingerprint_of(public: &[u8; 32]) -> String {
    hex(&Sha256::digest(public))[..16].to_string()
}

/// An agent's X25519 encryption key.
pub struct EncKey {
    secret: SecretKey,
}

impl EncKey {
    pub fn generate() -> EncKey {
        EncKey { secret: SecretKey::generate(&mut OsRng) }
    }
    pub fn public(&self) -> PublicKey {
        self.secret.public_key()
    }
    pub fn public_hex(&self) -> String {
        hex(self.public().as_bytes())
    }
    pub fn fingerprint(&self) -> String {
        fingerprint_of(self.public().as_bytes())
    }
}

/// Where the private key lives inside an agent's store directory.
pub fn key_path(store_dir: &Path) -> PathBuf {
    store_dir.join("keys").join("x25519.key")
}

/// The register entry the operator's provisioning writes for `agent` (ARC4 6.4.2).
pub fn default_register(agent: &str) -> PathBuf {
    PathBuf::from("/etc/termlink-sidecar/keys").join(format!("{agent}.x25519.pub"))
}

/// Load the private key. Production never mints: a missing file is a refused start. A test estate mints one (0600).
/// The file must not be readable or writable by group or others.
pub fn load_key(path: &Path, production: bool) -> Result<EncKey, String> {
    match std::fs::symlink_metadata(path) {
        Ok(md) => {
            if !md.file_type().is_file() {
                return Err(format!("encryption key {} is not a regular file", path.display()));
            }
            if md.permissions().mode() & 0o077 != 0 {
                return Err(format!(
                    "encryption key {} has mode {:o}; it must be 0600 (no group or other access)",
                    path.display(),
                    md.permissions().mode() & 0o777
                ));
            }
            let s = std::fs::read_to_string(path).map_err(|e| format!("encryption key {}: {e}", path.display()))?;
            let b = unhex32(s.trim()).ok_or_else(|| format!("encryption key {} is not 64 hex", path.display()))?;
            Ok(EncKey { secret: SecretKey::from_bytes(b) })
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            if production {
                return Err(format!(
                    "encryption key {} is missing; production never mints a key (re-provision through runme)",
                    path.display()
                ));
            }
            let k = EncKey::generate();
            if let Some(d) = path.parent() {
                std::fs::create_dir_all(d).map_err(|e| format!("encryption key dir {}: {e}", d.display()))?;
                let _ = std::fs::set_permissions(d, std::fs::Permissions::from_mode(0o700));
            }
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .create_new(true)
                .mode(0o600)
                .open(path)
                .map_err(|e| format!("encryption key {}: {e}", path.display()))?;
            f.write_all(hex(&k.secret.to_bytes()).as_bytes())
                .and_then(|_| f.sync_all())
                .map_err(|e| format!("encryption key {}: {e}", path.display()))?;
            Ok(k)
        }
        Err(e) => Err(format!("encryption key {}: {e}", path.display())),
    }
}

/// Read a register entry (FCW-110): it must exist, be a regular file (not a link) owned by `owner_uid`, not
/// group- or world-writable, and hold a 32-byte public key as 64 hex or base64.
pub fn read_register(reg: &Path, owner_uid: u32) -> Result<[u8; 32], String> {
    let md = std::fs::symlink_metadata(reg).map_err(|e| format!("encryption key register {}: {e} (FCW-110)", reg.display()))?;
    if !md.file_type().is_file() {
        return Err(format!("encryption key register {} is not a regular file (FCW-110)", reg.display()));
    }
    if md.uid() != owner_uid {
        return Err(format!(
            "encryption key register {} is owned by uid {}, not {owner_uid} (FCW-110)",
            reg.display(),
            md.uid()
        ));
    }
    if md.permissions().mode() & 0o022 != 0 {
        return Err(format!(
            "encryption key register {} is group- or world-writable (mode {:o}) (FCW-110)",
            reg.display(),
            md.permissions().mode() & 0o777
        ));
    }
    let s = std::fs::read_to_string(reg).map_err(|e| format!("encryption key register {}: {e} (FCW-110)", reg.display()))?;
    let s = s.trim();
    if let Some(b) = unhex32(s) {
        return Ok(b);
    }
    use base64::Engine;
    base64::engine::general_purpose::STANDARD
        .decode(s)
        .ok()
        .and_then(|v| <[u8; 32]>::try_from(v).ok())
        .ok_or_else(|| format!("encryption key register {} does not hold a 32-byte key (64 hex or base64) (FCW-110)", reg.display()))
}

/// The start-up check (AL-25). `register` is the configured entry; `None` leaves the agent without an encryption
/// key (`Ok(None)`, blobs refused), which is the rollback state. With a register, any defect refuses start.
pub fn startup(
    store_dir: &Path,
    register: Option<&Path>,
    production: bool,
    owner_uid: u32,
) -> Result<Option<(EncKey, PathBuf)>, String> {
    let Some(reg) = register else { return Ok(None) };
    let want = read_register(reg, owner_uid)?;
    let key = load_key(&key_path(store_dir), production)?;
    let have = key.public();
    if have.as_bytes() != &want {
        return Err(format!(
            "encryption key {} does not match the register {} (registered fingerprint {}, key file has {}) (FCW-109); a key changes only through a runme action that names the change",
            key_path(store_dir).display(),
            reg.display(),
            fingerprint_of(&want),
            fingerprint_of(have.as_bytes())
        ));
    }
    Ok(Some((key, reg.to_path_buf())))
}

/// Status `keys.encryption` (06a 2.9 OBS).
pub fn status_view(k: Option<&(EncKey, PathBuf)>) -> Value {
    match k {
        Some((key, reg)) => json!({"fingerprint": key.fingerprint(), "register": "match", "register_path": reg.display().to_string()}),
        None => json!({"state": "not provisioned", "why": "no encryption key register configured: blobs are refused, never sent in plaintext"}),
    }
}

/// Record the start-up result in the store (`keys/encryption.json`), rewritten at every start, so `status` can show
/// `keys.encryption` without new API wiring.
pub fn record_status(store_dir: &Path, view: &Value) -> Result<(), String> {
    let d = store_dir.join("keys");
    std::fs::create_dir_all(&d).map_err(|e| format!("{}: {e}", d.display()))?;
    let tmp = d.join(".encryption.json.tmp");
    std::fs::write(&tmp, view.to_string()).map_err(|e| format!("{}: {e}", tmp.display()))?;
    std::fs::rename(&tmp, d.join("encryption.json")).map_err(|e| format!("{}: {e}", d.display()))
}

/// Status `keys.encryption` as recorded at start; `unknown` when this store has no record (never "fine").
pub fn recorded_status(store_dir: &Path) -> Value {
    std::fs::read_to_string(store_dir.join("keys").join("encryption.json"))
        .ok()
        .and_then(|s| serde_json::from_str(&s).ok())
        .unwrap_or_else(|| json!({"state": "unknown", "why": "no start-up record of the encryption key check"}))
}

/// The immutable envelope fields the associated data covers (06a 2.10.c step 4, ARC4 6.4.3.a): never the
/// receiving agent's own name, so a role address and a forwarded message open at the agent holding the key.
pub struct AadFields<'a> {
    pub id: &'a str,
    pub from: &'a str,
    /// `to` verbatim as sent (a role address stays the role).
    pub to: &'a str,
    pub conv: &'a str,
    pub plain_sha256: &'a str,
    pub size: u64,
}

impl AadFields<'_> {
    /// `"termlink.blob/2\n" || sha256(jcs({...}))`. Keys are sorted (BTreeMap) and the values are strings and an
    /// integer, which serde_json renders as RFC 8785 does.
    pub fn bytes(&self) -> Vec<u8> {
        let mut m: BTreeMap<&str, Value> = BTreeMap::new();
        m.insert("id", json!(self.id));
        m.insert("from", json!(self.from));
        m.insert("to", json!(self.to));
        m.insert("conv", json!(self.conv));
        m.insert("plain_sha256", json!(self.plain_sha256));
        m.insert("size", json!(self.size));
        let canon = serde_json::to_vec(&m).expect("string map serializes");
        let mut out = AAD_DOMAIN.to_vec();
        out.extend_from_slice(&Sha256::digest(&canon));
        out
    }
}

/// A sealed blob: the object stored at the hub and the content key sealed to the recipient.
pub struct Sealed {
    pub obj: Vec<u8>,
    pub wrapped_key: Vec<u8>,
    pub cipher_sha256: String,
}

/// Encrypt `plain` to `recipient` (AL-26 step 4).
pub fn seal(plain: &[u8], recipient: &PublicKey, aad: &[u8]) -> Result<Sealed, String> {
    let mut k = [0u8; 32];
    let mut n = [0u8; tlblob::NONCE_LEN];
    OsRng.fill_bytes(&mut k);
    OsRng.fill_bytes(&mut n);
    let cipher = XChaCha20Poly1305::new((&k).into());
    let ct = cipher
        .encrypt(XNonce::from_slice(&n), Payload { msg: plain, aad })
        .map_err(|_| "blob encryption failed".to_string())?;
    let wrapped_key = recipient.seal(&mut OsRng, &k).map_err(|_| "content key seal failed".to_string())?;
    k.fill(0);
    let mut obj = Vec::with_capacity(tlblob::HEADER_LEN + ct.len());
    obj.extend_from_slice(tlblob::MAGIC);
    obj.push(tlblob::VERSION);
    obj.push(tlblob::ALG_XCHACHA20POLY1305_SEALEDBOX);
    obj.extend_from_slice(&n);
    obj.extend_from_slice(&ct);
    let cipher_sha256 = hex(&Sha256::digest(&obj));
    Ok(Sealed { obj, wrapped_key, cipher_sha256 })
}

/// Open an object with this agent's key (AL-27 step 4). Any failure returns `Err` and no plaintext.
pub fn open(obj: &[u8], wrapped_key: &[u8], me: &EncKey, aad: &[u8]) -> Result<Vec<u8>, String> {
    tlblob::check_header(obj)?;
    if wrapped_key.len() != WRAPPED_KEY_LEN {
        return Err(format!("wrapped key is {} bytes, not {WRAPPED_KEY_LEN}", wrapped_key.len()));
    }
    let mut k: [u8; 32] = me
        .secret
        .unseal(wrapped_key)
        .map_err(|_| "content key does not open with this agent's key".to_string())?
        .try_into()
        .map_err(|_| "content key is not 32 bytes".to_string())?;
    let n = &obj[8..tlblob::HEADER_LEN];
    let cipher = XChaCha20Poly1305::new((&k).into());
    let r = cipher
        .decrypt(XNonce::from_slice(n), Payload { msg: &obj[tlblob::HEADER_LEN..], aad })
        .map_err(|_| "authenticated decryption failed".to_string());
    k.fill(0);
    r
}

#[cfg(test)]
#[path = "enckey_tests.rs"]
mod tests;
