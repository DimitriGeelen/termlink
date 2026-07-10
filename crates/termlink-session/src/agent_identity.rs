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

    /// Load the identity at `path` directly (no `identity.key` suffix
    /// appended), or generate a new one atomically if the file is missing.
    /// Created files are chmod 600; the parent directory is created if
    /// needed (mirroring `write_seed_atomic`). T-1700 — backs the
    /// `termlink register --identity-key <PATH>` flag so co-resident agents
    /// on a shared host can present distinct envelope identities (PL-166
    /// resolution path; T-1693 Shape 1 wiring).
    pub fn load_or_create_from_file(path: &Path) -> Result<Self> {
        match fs::read(path) {
            Ok(bytes) => Self::from_seed_bytes(&bytes),
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
                let ident = Self::generate();
                write_seed_atomic(path, ident.seed_bytes())?;
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

/// T-2292: resolve the per-agent identity key path `<base>/identities/<id>.key`
/// for a logical `agent_id`. This is the default key location for a session
/// that declares an `agent_id` (via `TERMLINK_AGENT_ID`), so co-resident agents
/// on a shared host get DISTINCT keys — and therefore distinct fingerprints and
/// distinct `dm:` topics — instead of collapsing onto one shared host key
/// (RC1 of the T-2291 reliable-comms inception). `agent_id` is sanitized to a
/// filesystem-safe slug so a logical id containing path separators or other
/// unsafe characters cannot escape the `identities/` directory.
pub fn per_agent_identity_path(base: &Path, agent_id: &str) -> PathBuf {
    base.join("identities")
        .join(format!("{}.key", sanitize_agent_id(agent_id)))
}

/// T-2399: resolve the SIGNING identity honoring the same env precedence as the
/// CLI post path (`termlink-cli/src/commands/channel.rs::load_identity_or_create`)
/// and `registration::resolve_identity_key_path`:
///
///   1. `TERMLINK_IDENTITY_FILE` — explicit key file (T-1700).
///   2. `TERMLINK_IDENTITY_DIR/identity.key` — base-dir override (T-1159).
///   3. `TERMLINK_AGENT_ID` → `$HOME/.termlink/identities/<id>.key` — per-agent (T-2292).
///   4. `fallback_base/identity.key` — shared host default (backward compatible).
///
/// The MCP tool handlers historically hardcoded `load_or_create($HOME/.termlink)`
/// (rule 4 only), so a per-agent armed session (`TERMLINK_AGENT_ID=aef`) posting
/// via MCP signed as the shared host key instead of its per-agent fingerprint —
/// collapsing every co-resident agent's OUTBOUND envelopes onto one identity and
/// breaking `dm:` routing/attribution (the T-2399 leak; sibling to PL-236).
/// Routing them through this keeps the MCP wire `sender_id` in lockstep with the
/// registration fingerprint. `fallback_base` is the handler's existing
/// `$HOME/.termlink` dir, used only when no env override applies.
pub fn resolve_signing_identity(fallback_base: &Path) -> Result<Identity> {
    let path = resolve_signing_identity_path(fallback_base, |k| std::env::var(k).ok());
    Identity::load_or_create_from_file(&path)
}

/// Pure path-resolution core for [`resolve_signing_identity`], parameterized over
/// an env accessor so the precedence is hermetically testable without mutating
/// global process env (mirrors `registration::resolve_identity_key_path`). All
/// four branches resolve to a concrete key file; `load_or_create_from_file`
/// (no suffix) is then applied uniformly by the caller — equivalent to the
/// previous `load_or_create(base)` for rules 2 and 4 since `identity_path`
/// appends `identity.key`.
fn resolve_signing_identity_path<F>(fallback_base: &Path, get_env: F) -> PathBuf
where
    F: Fn(&str) -> Option<String>,
{
    if let Some(file) = get_env("TERMLINK_IDENTITY_FILE") {
        return PathBuf::from(file);
    }
    if let Some(dir) = get_env("TERMLINK_IDENTITY_DIR") {
        return identity_path(Path::new(&dir));
    }
    if let Some(agent_id) = get_env("TERMLINK_AGENT_ID").filter(|s| !s.trim().is_empty()) {
        if let Some(home) = get_env("HOME") {
            return per_agent_identity_path(&PathBuf::from(home).join(".termlink"), &agent_id);
        }
    }
    identity_path(fallback_base)
}

/// Sanitize a logical agent id into a single filesystem-safe filename
/// component: any character that is not ASCII alphanumeric, '-', '_', or '.'
/// is replaced with '_'. An empty result (or one that would be a bare '.'/'..'
/// traversal token) collapses to "_". The caller appends a `.key` suffix, so
/// the returned slug is never interpreted as a path itself.
pub fn sanitize_agent_id(agent_id: &str) -> String {
    let slug: String = agent_id
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' || c == '.' {
                c
            } else {
                '_'
            }
        })
        .collect();
    if slug.is_empty() || slug == "." || slug == ".." {
        "_".to_string()
    } else {
        slug
    }
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

    // T-2399: the MCP-signing resolver must honor FILE > DIR > AGENT_ID > shared,
    // so a per-agent armed session posting via MCP signs with its OWN key (distinct
    // fingerprint) instead of collapsing onto the shared host key.
    #[test]
    fn resolve_signing_identity_path_precedence() {
        fn env_from(pairs: &[(&str, &str)]) -> impl Fn(&str) -> Option<String> {
            let m: std::collections::HashMap<String, String> = pairs
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_string()))
                .collect();
            move |k: &str| m.get(k).cloned()
        }
        let base = Path::new("/home/u/.termlink");

        // Rule 1: explicit FILE wins over agent-id.
        assert_eq!(
            resolve_signing_identity_path(
                base,
                env_from(&[
                    ("TERMLINK_IDENTITY_FILE", "/keys/x.key"),
                    ("TERMLINK_AGENT_ID", "aef"),
                    ("HOME", "/home/u"),
                ])
            ),
            PathBuf::from("/keys/x.key")
        );
        // Rule 2: DIR override → <dir>/identity.key.
        assert_eq!(
            resolve_signing_identity_path(base, env_from(&[("TERMLINK_IDENTITY_DIR", "/d")])),
            identity_path(Path::new("/d"))
        );
        // Rule 3: per-agent key when TERMLINK_AGENT_ID is set — the leak fix.
        assert_eq!(
            resolve_signing_identity_path(
                base,
                env_from(&[("TERMLINK_AGENT_ID", "aef"), ("HOME", "/home/u")])
            ),
            per_agent_identity_path(&PathBuf::from("/home/u/.termlink"), "aef")
        );
        // Blank agent id is treated as absent → shared default (no identities/_.key).
        assert_eq!(
            resolve_signing_identity_path(
                base,
                env_from(&[("TERMLINK_AGENT_ID", "  "), ("HOME", "/home/u")])
            ),
            identity_path(base)
        );
        // Rule 4: nothing set → shared host default (backward compatible).
        assert_eq!(
            resolve_signing_identity_path(base, env_from(&[])),
            identity_path(base)
        );
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

    // T-1700: explicit-file constructor — backs `termlink register --identity-key`.
    #[test]
    fn load_or_create_from_file_writes_seed_0600_at_arbitrary_filename() {
        let tmp = TempDir::new().unwrap();
        let path = tmp.path().join("nested").join("agent-a.key");
        let ident = Identity::load_or_create_from_file(&path).unwrap();
        assert!(path.is_file(), "key file at custom path must exist");
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        assert_eq!(mode, 0o600, "explicit-path key file must be chmod 600");
        let ident2 = Identity::load_or_create_from_file(&path).unwrap();
        assert_eq!(
            ident.public_key_hex(),
            ident2.public_key_hex(),
            "round-trip must yield the same public key"
        );
    }

    #[test]
    fn load_or_create_from_file_two_paths_produce_distinct_identities() {
        // PL-166 resolution path: two paths on the same host must give
        // structurally-distinct fingerprints (the cohort-letter ask).
        let tmp = TempDir::new().unwrap();
        let a_path = tmp.path().join("agent-a.key");
        let b_path = tmp.path().join("agent-b.key");
        let a = Identity::load_or_create_from_file(&a_path).unwrap();
        let b = Identity::load_or_create_from_file(&b_path).unwrap();
        assert_ne!(a.fingerprint(), b.fingerprint());
        assert_ne!(a.public_key_hex(), b.public_key_hex());
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

    // T-2292: per-agent path layout under <base>/identities/<id>.key.
    #[test]
    fn per_agent_identity_path_layout() {
        let base = PathBuf::from("/home/x/.termlink");
        assert_eq!(
            per_agent_identity_path(&base, "claude-alpha"),
            PathBuf::from("/home/x/.termlink/identities/claude-alpha.key")
        );
    }

    // T-2292: distinct agent ids → distinct key paths under a shared base.
    #[test]
    fn per_agent_identity_path_distinct_per_agent() {
        let base = PathBuf::from("/shared/.termlink");
        assert_ne!(
            per_agent_identity_path(&base, "agent-a"),
            per_agent_identity_path(&base, "agent-b")
        );
    }

    // T-2292: unsafe characters (path separators, ':') cannot escape the
    // identities/ directory — they are slugged to '_'.
    #[test]
    fn sanitize_agent_id_neutralizes_path_traversal() {
        // Separators slugged to '_'; the leading dots are harmless inside a
        // single filename component (a '.key' suffix is appended by the caller).
        assert_eq!(sanitize_agent_id("../../etc/passwd"), ".._.._etc_passwd");
        assert_eq!(sanitize_agent_id("a/b"), "a_b");
        assert_eq!(sanitize_agent_id("host:9100"), "host_9100");
        // A resulting path always has exactly one component beyond identities/.
        let p = per_agent_identity_path(Path::new("/b/.termlink"), "../../x");
        assert_eq!(p.parent().unwrap(), Path::new("/b/.termlink/identities"));
    }

    // T-2292: empty / bare-dot ids collapse to '_' (never a traversal token).
    #[test]
    fn sanitize_agent_id_edge_cases() {
        assert_eq!(sanitize_agent_id(""), "_");
        assert_eq!(sanitize_agent_id("."), "_");
        assert_eq!(sanitize_agent_id(".."), "_");
        assert_eq!(sanitize_agent_id("ok.name"), "ok.name");
    }

    // T-2292: two distinct per-agent paths create genuinely distinct
    // identities (distinct fingerprints) — the RC1 antidote.
    #[test]
    fn per_agent_paths_produce_distinct_fingerprints() {
        let dir = std::env::temp_dir().join(format!(
            "tl-peragent-{}",
            std::process::id()
        ));
        let base = dir.join(".termlink");
        let pa = per_agent_identity_path(&base, "agent-a");
        let pb = per_agent_identity_path(&base, "agent-b");
        let a = Identity::load_or_create_from_file(&pa).unwrap();
        let b = Identity::load_or_create_from_file(&pb).unwrap();
        assert_ne!(a.fingerprint(), b.fingerprint());
        let _ = std::fs::remove_dir_all(&dir);
    }
}
