//! Agent cryptographic identity (T-1159, T-1155 S-4).
//!
//! One ed25519 keypair per agent, stored as a 32-byte seed at
//! `<base>/identity.key` with mode 0600. Separates identity trust from
//! transport trust — hub-secret rotations no longer invalidate signed
//! messages.
//!
//! See `known_peers.rs` for the companion TOFU keyring that pins peer
//! public keys on first observation.

use std::fs::{self, File, OpenOptions};
use std::io::{Read, Write};
use std::os::unix::fs::{OpenOptionsExt, PermissionsExt};
use std::path::{Path, PathBuf};

use ed25519_dalek::{Signature, Signer, SigningKey, Verifier, VerifyingKey};
use sha2::{Digest, Sha256};
use thiserror::Error;

/// 32-byte raw ed25519 seed file, stored chmod 600.
const SEED_LEN: usize = 32;

#[derive(Debug, Error)]
pub enum IdentityError {
    #[error("io: {0}")]
    Io(#[from] std::io::Error),
    #[error("invalid seed length: expected {expected}, got {got}")]
    InvalidSeedLen { expected: usize, got: usize },
    #[error("identity file already exists at {0} (use --force to overwrite)")]
    AlreadyExists(PathBuf),
    #[error("signature error: {0}")]
    Signature(ed25519_dalek::SignatureError),
}

impl From<ed25519_dalek::SignatureError> for IdentityError {
    fn from(e: ed25519_dalek::SignatureError) -> Self {
        IdentityError::Signature(e)
    }
}

pub type Result<T> = std::result::Result<T, IdentityError>;

/// Agent keypair and cached fingerprint. Public key is the source of truth
/// for identity; fingerprint is a short human-readable handle.
pub struct Identity {
    signing: SigningKey,
    fingerprint: String,
}

impl std::fmt::Debug for Identity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Identity")
            .field("fingerprint", &self.fingerprint)
            .field("signing", &"<redacted>")
            .finish()
    }
}

impl Identity {
    /// Load the identity at `<base>/identity.key` or generate a new one
    /// atomically if the file is missing. Created files are chmod 600.
    pub fn load_or_create(base: &Path) -> Result<Self> {
        let path = identity_path(base);
        match fs::read(&path) {
            Ok(bytes) => Self::from_seed_bytes(&bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let ident = Self::generate();
                write_seed_atomic(&path, ident.seed_bytes())?;
                Ok(ident)
            }
            Err(e) => Err(IdentityError::Io(e)),
        }
    }

    /// Bootstrap a new identity at `<base>/identity.key`. Refuses to
    /// overwrite an existing file unless `force` is true; when forcing,
    /// the old file is renamed to `identity.key.bak-<unix-ms>`.
    pub fn init(base: &Path, force: bool) -> Result<Self> {
        let path = identity_path(base);
        if path.exists() {
            if !force {
                return Err(IdentityError::AlreadyExists(path));
            }
            let bak = path.with_file_name(format!(
                "identity.key.bak-{}",
                now_unix_ms(),
            ));
            fs::rename(&path, &bak)?;
        }
        let ident = Self::generate();
        write_seed_atomic(&path, ident.seed_bytes())?;
        Ok(ident)
    }

    /// Generate a fresh keypair in memory (no disk touch).
    pub fn generate() -> Self {
        let mut seed = [0u8; SEED_LEN];
        rand_core::OsRng.fill_bytes_wrapper(&mut seed);
        Self::from_seed(seed)
    }

    fn from_seed(seed: [u8; SEED_LEN]) -> Self {
        let signing = SigningKey::from_bytes(&seed);
        let fingerprint = fingerprint_of(&signing.verifying_key());
        Self {
            signing,
            fingerprint,
        }
    }

    fn from_seed_bytes(bytes: &[u8]) -> Result<Self> {
        if bytes.len() != SEED_LEN {
            return Err(IdentityError::InvalidSeedLen {
                expected: SEED_LEN,
                got: bytes.len(),
            });
        }
        let mut arr = [0u8; SEED_LEN];
        arr.copy_from_slice(bytes);
        Ok(Self::from_seed(arr))
    }

    fn seed_bytes(&self) -> &[u8; SEED_LEN] {
        self.signing.as_bytes()
    }

    /// Sign an arbitrary message with the agent's private key.
    pub fn sign(&self, msg: &[u8]) -> Signature {
        self.signing.sign(msg)
    }

    /// Verify a signature against an arbitrary public key.
    pub fn verify(pk: &VerifyingKey, msg: &[u8], sig: &Signature) -> bool {
        pk.verify(msg, sig).is_ok()
    }

    /// 64-hex-char public key.
    pub fn public_key_hex(&self) -> String {
        hex_encode(self.signing.verifying_key().as_bytes())
    }

    /// Short identity fingerprint (first 16 hex chars of sha256(pubkey)),
    /// for operator-facing display.
    pub fn fingerprint(&self) -> &str {
        &self.fingerprint
    }

    /// The `VerifyingKey` — share this with peers over the TOFU channel.
    pub fn verifying_key(&self) -> VerifyingKey {
        self.signing.verifying_key()
    }
}

/// Compute the fingerprint of a public key (first 16 hex chars of sha256).
pub fn fingerprint_of(pk: &VerifyingKey) -> String {
    let mut hasher = Sha256::new();
    hasher.update(pk.as_bytes());
    let digest = hasher.finalize();
    hex_encode(&digest)[..16].to_string()
}

/// Resolve `<base>/identity.key`. Exposed so CLI code can report the exact
/// path the user should protect / back up.
pub fn identity_path(base: &Path) -> PathBuf {
    base.join("identity.key")
}

fn write_seed_atomic(path: &Path, seed: &[u8; SEED_LEN]) -> Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let tmp = path.with_extension("key.tmp");
    {
        let mut f = OpenOptions::new()
            .create(true)
            .truncate(true)
            .write(true)
            .mode(0o600)
            .open(&tmp)?;
        f.write_all(seed)?;
        f.flush()?;
    }
    // Ensure permissions survive any umask weirdness.
    let mut perms = fs::metadata(&tmp)?.permissions();
    perms.set_mode(0o600);
    fs::set_permissions(&tmp, perms)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn hex_encode(bytes: &[u8]) -> String {
    use std::fmt::Write;
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        let _ = write!(&mut s, "{b:02x}");
    }
    s
}

fn now_unix_ms() -> i64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_millis() as i64)
        .unwrap_or(0)
}

/// Small shim so we can call `fill_bytes` on `OsRng` without pulling in the
/// full rand crate just for one trait method.
trait FillBytes {
    fn fill_bytes_wrapper(&mut self, buf: &mut [u8]);
}

impl FillBytes for rand_core::OsRng {
    fn fill_bytes_wrapper(&mut self, buf: &mut [u8]) {
        use rand_core::RngCore;
        self.fill_bytes(buf);
    }
}

/// Read an existing seed file. Shared with `init --force` to preserve the
/// old keypair in `identity.key.bak-*`.
#[allow(dead_code)]
pub(crate) fn read_seed_file(path: &Path) -> Result<[u8; SEED_LEN]> {
    let mut f = File::open(path)?;
    let mut buf = [0u8; SEED_LEN];
    f.read_exact(&mut buf)?;
    Ok(buf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn generate_produces_distinct_keys() {
        let a = Identity::generate();
        let b = Identity::generate();
        assert_ne!(a.public_key_hex(), b.public_key_hex());
        assert_eq!(a.fingerprint().len(), 16);
    }

    #[test]
    fn load_or_create_writes_seed_0600() {
        let tmp = TempDir::new().unwrap();
        let ident = Identity::load_or_create(tmp.path()).unwrap();
        let p = identity_path(tmp.path());
        assert!(p.is_file());
        let mode = fs::metadata(&p).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "identity.key must be chmod 600");
        // Load again — should return the same key.
        let ident2 = Identity::load_or_create(tmp.path()).unwrap();
        assert_eq!(ident.public_key_hex(), ident2.public_key_hex());
    }

    #[test]
    fn sign_and_verify_roundtrip() {
        let ident = Identity::generate();
        let msg = b"hello bus";
        let sig = ident.sign(msg);
        assert!(Identity::verify(&ident.verifying_key(), msg, &sig));
        // Wrong message fails.
        assert!(!Identity::verify(&ident.verifying_key(), b"tampered", &sig));
        // Wrong key fails.
        let other = Identity::generate();
        assert!(!Identity::verify(&other.verifying_key(), msg, &sig));
    }

    #[test]
    fn fingerprint_is_stable_across_reload() {
        let tmp = TempDir::new().unwrap();
        let a = Identity::load_or_create(tmp.path()).unwrap();
        let b = Identity::load_or_create(tmp.path()).unwrap();
        assert_eq!(a.fingerprint(), b.fingerprint());
    }

    #[test]
    fn init_refuses_overwrite_without_force() {
        let tmp = TempDir::new().unwrap();
        Identity::init(tmp.path(), false).unwrap();
        let err = Identity::init(tmp.path(), false).unwrap_err();
        assert!(matches!(err, IdentityError::AlreadyExists(_)));
    }

    #[test]
    fn init_force_rotates_and_backs_up() {
        let tmp = TempDir::new().unwrap();
        let a = Identity::init(tmp.path(), false).unwrap();
        let b = Identity::init(tmp.path(), true).unwrap();
        assert_ne!(a.public_key_hex(), b.public_key_hex());
        // There should now be exactly one .bak-* file.
        let baks: Vec<_> = fs::read_dir(tmp.path())
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.file_name()
                    .to_string_lossy()
                    .starts_with("identity.key.bak-")
            })
            .collect();
        assert_eq!(baks.len(), 1);
    }

    #[test]
    fn short_seed_rejected() {
        let err = Identity::from_seed_bytes(&[0u8; 10]).unwrap_err();
        assert!(matches!(err, IdentityError::InvalidSeedLen { .. }));
    }
}
