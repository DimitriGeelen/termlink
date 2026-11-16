//! T-3366 (arc-011 P-10, B-1; AL-17 + AL-18 hub half): the hub's canonical id and signing key.
//!
//! `hub.id` holds 16 lowercase hex, minted ONCE per runtime directory and never recomputed: a
//! certificate rotation no longer renames `inbox:<hub-id>/…`. `hub.signing.key` holds an ed25519
//! seed (64 hex, mode 0600) for hub-signed state (revocation list, dead letters, cards).
//!
//! Rules (docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md 2.1, 2.2):
//! 1. `hub.id` present: it must be 16 lowercase hex, else refuse (FCW-80).
//! 2. `hub.id` absent beside prior hub state: refuse (FCW-79). The only way past is
//!    [`adopt_legacy`] (`termlink hub adopt-id`), which runme runs before a restart.
//! 3. Empty runtime directory: mint from the certificate if one exists NOW, else 64 random bits
//!    (CR-6.42). Written via temp + fsync + rename; a failed write leaves no file (FCW-81).
//! 4. `hub.signing.key`: generated only on a first start (same rule); otherwise it must exist,
//!    be mode 0600, owned by this user, and hold a 32-byte seed (FCW-82). Never printed.
//!
//! Every function takes the runtime directory explicitly, so tests use temp dirs and never the
//! live hub's directory.

use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::RwLock;

use ed25519_dalek::SigningKey;
use sha2::{Digest, Sha256};

pub const ID_FILE: &str = "hub.id";
pub const KEY_FILE: &str = "hub.signing.key";

/// The identity a started hub answers with. Set once per start, read by `hub.version`.
#[derive(Clone)]
pub struct HubIdentity {
    pub hub_id: String,
    signing: SigningKey,
}

impl HubIdentity {
    /// `ed25519:` + hex SHA-256 of the 32-byte public key (AL-18 2.2.h).
    pub fn signing_key_fingerprint(&self) -> String {
        signing_fingerprint(&self.signing)
    }

    /// The signing key, for hub-signed objects (P-15, P-16). Never serialise it.
    pub fn signing_key(&self) -> &SigningKey {
        &self.signing
    }
}

impl std::fmt::Debug for HubIdentity {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        // SI-25: the private key never appears in any log or debug print.
        f.debug_struct("HubIdentity")
            .field("hub_id", &self.hub_id)
            .field("signing_key", &self.signing_key_fingerprint())
            .finish()
    }
}

pub fn signing_fingerprint(k: &SigningKey) -> String {
    let digest = Sha256::digest(k.verifying_key().as_bytes());
    format!("ed25519:{}", hex(&digest))
}

static CURRENT: RwLock<Option<HubIdentity>> = RwLock::new(None);

/// Record the identity the running hub answers with (called by the server at start).
pub fn set_current(id: HubIdentity) {
    if let Ok(mut g) = CURRENT.write() {
        *g = Some(id);
    }
}

/// The running hub's identity, if it has started. `None` means `hub.version` states null:
/// the hub never falls back to the certificate (AL-17 invariant 3).
pub fn current() -> Option<HubIdentity> {
    CURRENT.read().ok().and_then(|g| g.clone())
}

fn refuse(msg: impl Into<String>) -> std::io::Error {
    std::io::Error::new(std::io::ErrorKind::InvalidData, msg.into())
}

fn hex(bytes: &[u8]) -> String {
    bytes.iter().map(|b| format!("{b:02x}")).collect()
}

fn is_id(s: &str) -> bool {
    s.len() == 16 && s.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

/// Prior hub state in `dir`: a secret, a signing key, or any topic log — the things that make
/// an id visible to peers. A certificate alone is not: E-36 starts a "cert only (legacy hub)"
/// directory and expects `first16(fingerprint)`, which AL-17's first-start branch gives (the
/// spec's step 1 also lists the certificate; E-36 is the tested reading, logged in T-3366).
pub fn has_prior_state(dir: &Path) -> bool {
    ["hub.secret", KEY_FILE].iter().any(|f| dir.join(f).exists())
        || std::fs::read_dir(dir.join("bus")).map(|mut r| r.next().is_some()).unwrap_or(false)
}

/// Read `hub.id`: `Ok(None)` when absent, `Err` when unreadable or malformed (FCW-80).
pub fn read_id(dir: &Path) -> std::io::Result<Option<String>> {
    let path = dir.join(ID_FILE);
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(refuse(format!("hub.id unreadable or malformed ({}: {e})", path.display()))),
    };
    let s = raw.strip_suffix('\n').unwrap_or(&raw);
    if !is_id(s) {
        return Err(refuse(format!(
            "hub.id unreadable or malformed ({}: not 16 lowercase hex)",
            path.display()
        )));
    }
    Ok(Some(s.to_string()))
}

/// Write `contents` to `dir/name` atomically: temp (with `mode`) + fsync + rename + dir fsync.
/// On any error the temp is removed and no partial file is left (FCW-81).
#[cfg(test)]
thread_local! {
    /// E-166 fault injector (tests only): fail after the temp is written, before the rename.
    static FAIL_BEFORE_RENAME: std::cell::Cell<bool> = const { std::cell::Cell::new(false) };
}

fn atomic_write(dir: &Path, name: &str, contents: &[u8], mode: u32) -> std::io::Result<()> {
    let path = dir.join(name);
    // Crash table (AL-17 2.1.f): a temp left by a start that died before its rename is removed.
    if let Ok(rd) = std::fs::read_dir(dir) {
        let prefix = format!(".{name}.tmp-");
        for e in rd.flatten() {
            if e.file_name().to_string_lossy().starts_with(&prefix) {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    let tmp: PathBuf = dir.join(format!(".{name}.tmp-{}", std::process::id()));
    let res = (|| {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new().create_new(true).write(true).mode(mode).open(&tmp)?;
        f.write_all(contents)?;
        f.sync_all()?;
        drop(f);
        #[cfg(test)]
        if FAIL_BEFORE_RENAME.with(|c| c.get()) {
            return Err(std::io::Error::other("injected: crash before rename"));
        }
        std::fs::rename(&tmp, &path)?;
        if let Ok(d) = std::fs::File::open(dir) {
            let _ = d.sync_all();
        }
        Ok(())
    })();
    if res.is_err() {
        let _ = std::fs::remove_file(&tmp);
    }
    res
}

/// The id a fresh mint takes: the certificate's when one is present now, else random.
/// Returns (id, from_certificate).
fn mint_value(dir: &Path) -> (String, bool) {
    if let Some(id) = crate::tls::own_cert_fingerprint_at(&dir.join("hub.cert.pem"))
        .as_deref()
        .and_then(crate::tls::hub_id_from_fingerprint)
    {
        return (id, true);
    }
    let r = termlink_session::auth::generate_secret();
    (hex(&r[..8]), false)
}

fn write_id(dir: &Path, id: &str) -> std::io::Result<()> {
    atomic_write(dir, ID_FILE, format!("{id}\n").as_bytes(), 0o644)
        .map_err(|e| std::io::Error::new(e.kind(), format!("hub.id could not be written atomically: {e}")))
}

fn generate_key(dir: &Path) -> std::io::Result<SigningKey> {
    let seed = termlink_session::auth::generate_secret();
    atomic_write(dir, KEY_FILE, format!("{}\n", hex(&seed)).as_bytes(), 0o600)
        .map_err(|e| std::io::Error::new(e.kind(), format!("hub.signing.key could not be written atomically: {e}")))?;
    Ok(SigningKey::from_bytes(&seed))
}

/// Read `hub.signing.key`: `Ok(None)` when absent; `Err` on wrong mode, owner or contents (FCW-82).
pub fn read_key(dir: &Path) -> std::io::Result<Option<SigningKey>> {
    use std::os::unix::fs::MetadataExt;
    let path = dir.join(KEY_FILE);
    let meta = match std::fs::metadata(&path) {
        Ok(m) => m,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(refuse(format!("hub.signing.key unreadable ({}: {e})", path.display()))),
    };
    let mode = meta.mode() & 0o777;
    if mode != 0o600 {
        return Err(refuse(format!("hub.signing.key has mode {mode:o}, must be 600 ({})", path.display())));
    }
    // SAFETY: geteuid has no preconditions.
    let euid = unsafe { libc::geteuid() };
    if meta.uid() != euid {
        return Err(refuse(format!(
            "hub.signing.key is owned by uid {}, the hub runs as uid {euid} ({})",
            meta.uid(),
            path.display()
        )));
    }
    let raw = std::fs::read_to_string(&path)
        .map_err(|e| refuse(format!("hub.signing.key unreadable ({}: {e})", path.display())))?;
    let s = raw.trim_end_matches('\n');
    let bytes: Option<Vec<u8>> = (s.len() == 64)
        .then(|| (0..32).map(|i| u8::from_str_radix(&s[2 * i..2 * i + 2], 16).ok()).collect())
        .flatten();
    let Some(bytes) = bytes else {
        // The contents are never echoed (SI-25).
        return Err(refuse(format!("hub.signing.key is malformed: not a 32-byte hex seed ({})", path.display())));
    };
    let mut seed = [0u8; 32];
    seed.copy_from_slice(&bytes);
    Ok(Some(SigningKey::from_bytes(&seed)))
}

/// Hub start (AL-17 step 2, AL-18): read or mint the id, read or generate the key.
/// Must run BEFORE the start writes its own secret or certificate, so a first start is
/// recognised as one.
pub fn load_or_mint(dir: &Path) -> std::io::Result<HubIdentity> {
    let prior = has_prior_state(dir);
    let (hub_id, first_start) = match read_id(dir)? {
        Some(id) => (id, false),
        None if prior => {
            return Err(refuse(format!(
                "hub.id missing beside existing state in {}; restore it, or run `termlink hub adopt-id` (runme does this before a restart)",
                dir.display()
            )));
        }
        None => {
            std::fs::create_dir_all(dir)?;
            let (id, from_cert) = mint_value(dir);
            write_id(dir, &id)?;
            tracing::info!(hub_id = %id, from_certificate = from_cert, "hub.id minted (first start, T-3366)");
            (id, true)
        }
    };
    let signing = match read_key(dir)? {
        Some(k) => k,
        None if first_start => generate_key(dir)?,
        None => {
            return Err(refuse(format!(
                "hub.signing.key missing beside existing state in {}; run `termlink hub adopt-id` (runme does this before a restart)",
                dir.display()
            )));
        }
    };
    Ok(HubIdentity { hub_id, signing })
}

/// What [`adopt_legacy`] did.
#[derive(Debug, PartialEq, Eq)]
pub struct Adopted {
    pub hub_id: String,
    /// "kept" (already present), "certificate" (first16 of the current fingerprint) or
    /// "random" (no certificate: peers will see a new id and refuse it until re-pinned).
    pub id_source: &'static str,
    /// true when this call generated the signing key.
    pub key_generated: bool,
    pub signing_key: String,
}

/// The legacy migration (AL-17, "the only path past FCW-79"): give a hub that predates
/// T-3366 its id and signing key, without renaming any inbox. Idempotent; never changes an
/// id already on disk; refuses a malformed one or a bad key file rather than replacing it.
pub fn adopt_legacy(dir: &Path, dry_run: bool) -> std::io::Result<Adopted> {
    let (hub_id, id_source) = match read_id(dir)? {
        Some(id) => (id, "kept"),
        None => {
            let (id, from_cert) = mint_value(dir);
            if !dry_run {
                std::fs::create_dir_all(dir)?;
                write_id(dir, &id)?;
            }
            (id, if from_cert { "certificate" } else { "random" })
        }
    };
    let (signing_key, key_generated) = match read_key(dir)? {
        Some(k) => (signing_fingerprint(&k), false),
        None if dry_run => ("(would be generated)".to_string(), true),
        None => (signing_fingerprint(&generate_key(dir)?), true),
    };
    Ok(Adopted { hub_id, id_source, key_generated, signing_key })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;

    fn tmp(tag: &str) -> PathBuf {
        let d = std::env::temp_dir().join(format!("t3366-{tag}-{}-{}", std::process::id(), rand_suffix()));
        std::fs::create_dir_all(&d).unwrap();
        d
    }
    fn rand_suffix() -> String {
        hex(&termlink_session::auth::generate_secret()[..4])
    }
    fn write_cert(dir: &Path) -> String {
        let ck = rcgen::generate_simple_self_signed(vec!["localhost".into()]).unwrap();
        std::fs::write(dir.join("hub.cert.pem"), ck.cert.pem()).unwrap();
        std::fs::write(dir.join("hub.key.pem"), ck.key_pair.serialize_pem()).unwrap();
        crate::tls::own_cert_fingerprint_at(&dir.join("hub.cert.pem"))
            .as_deref()
            .and_then(crate::tls::hub_id_from_fingerprint)
            .unwrap()
    }

    // E-36: empty dir mints random, file 0644, 16 hex; restart keeps it.
    #[test]
    fn e36_empty_dir_mints_once_and_restart_keeps_it() {
        let d = tmp("empty");
        let a = load_or_mint(&d).unwrap();
        assert!(is_id(&a.hub_id));
        let m = std::fs::metadata(d.join(ID_FILE)).unwrap().permissions().mode() & 0o777;
        assert_eq!(m, 0o644);
        assert_eq!(std::fs::read_to_string(d.join(ID_FILE)).unwrap(), format!("{}\n", a.hub_id));
        let k = std::fs::metadata(d.join(KEY_FILE)).unwrap().permissions().mode() & 0o777;
        assert_eq!(k, 0o600);
        let b = load_or_mint(&d).unwrap();
        assert_eq!(a.hub_id, b.hub_id);
        assert_eq!(a.signing_key_fingerprint(), b.signing_key_fingerprint());
        let _ = std::fs::remove_dir_all(&d);
    }

    // E-36: a cert present at first start gives first16(fingerprint).
    #[test]
    fn e36_first_start_with_cert_takes_the_certificate_id() {
        let d = tmp("cert");
        let want = write_cert(&d);
        assert_eq!(load_or_mint(&d).unwrap().hub_id, want, "cert only (legacy hub) keeps first16(fingerprint)");
        assert_eq!(load_or_mint(&d).unwrap().hub_id, want);
        // and adopt on a secret+cert legacy hub gives the same certificate id
        let d2 = tmp("cert2");
        let want2 = write_cert(&d2);
        std::fs::write(d2.join("hub.secret"), "ab".repeat(32)).unwrap();
        let ad = adopt_legacy(&d2, false).unwrap();
        assert_eq!((ad.hub_id.as_str(), ad.id_source, ad.key_generated), (want2.as_str(), "certificate", true));
        assert_eq!(load_or_mint(&d2).unwrap().hub_id, want2);
        let _ = std::fs::remove_dir_all(&d2);
        let _ = std::fs::remove_dir_all(&d);
    }

    // E-36 control (ARC4 6.1.7): rotation changes neither id nor key — kills "derive each call".
    #[test]
    fn e36_certificate_rotation_changes_neither_id_nor_key() {
        let d = tmp("rot");
        write_cert(&d);
        let a = load_or_mint(&d).unwrap();
        let new = write_cert(&d);
        assert_ne!(new, a.hub_id, "the fixture must really rotate");
        let b = load_or_mint(&d).unwrap();
        assert_eq!(b.hub_id, a.hub_id);
        assert_eq!(b.signing_key_fingerprint(), a.signing_key_fingerprint());
        let _ = std::fs::remove_dir_all(&d);
    }

    // E-36 / FCW-79: secret + cert, no hub.id → refuse, and write nothing.
    #[test]
    fn e36_fcw79_missing_id_beside_state_refuses_and_writes_nothing() {
        let d = tmp("fcw79");
        write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("hub.id missing beside existing state"), "{e}");
        assert!(!d.join(ID_FILE).exists());
        assert!(!d.join(KEY_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    // FCW-79 also on a topic log alone.
    #[test]
    fn e36_fcw79_topic_log_counts_as_prior_state() {
        let d = tmp("bus");
        std::fs::create_dir_all(d.join("bus/x")).unwrap();
        assert!(load_or_mint(&d).is_err());
        let _ = std::fs::remove_dir_all(&d);
    }

    // FCW-80: malformed id refuses (uppercase, short, junk).
    #[test]
    fn e36_fcw80_malformed_id_refuses() {
        for bad in ["CACC73EA32B121DD\n", "cacc73ea\n", "zzzzzzzzzzzzzzzz\n", "cacc73ea32b121dd\n\n"] {
            let d = tmp("fcw80");
            std::fs::write(d.join(ID_FILE), bad).unwrap();
            let e = load_or_mint(&d).unwrap_err().to_string();
            assert!(e.contains("hub.id unreadable or malformed"), "{bad:?}: {e}");
            let _ = std::fs::remove_dir_all(&d);
        }
    }

    // E-166 / FCW-81: an unwritable directory refuses and leaves no file or temp.
    #[test]
    fn e166_fcw81_mint_that_cannot_be_written_leaves_nothing() {
        if unsafe { libc::geteuid() } == 0 {
            // root ignores directory modes; use a path under a regular file instead
            let d = tmp("fcw81");
            let f = d.join("notadir");
            std::fs::write(&f, "x").unwrap();
            assert!(load_or_mint(&f.join("rt")).is_err());
            let names: Vec<_> = std::fs::read_dir(&d).unwrap().map(|e| e.unwrap().file_name()).collect();
            assert_eq!(names.len(), 1, "{names:?}");
            let _ = std::fs::remove_dir_all(&d);
            return;
        }
        let d = tmp("fcw81");
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o500)).unwrap();
        assert!(load_or_mint(&d).is_err());
        std::fs::set_permissions(&d, std::fs::Permissions::from_mode(0o700)).unwrap();
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 0);
        let _ = std::fs::remove_dir_all(&d);
    }

    // E-166 control (mutant "write hub.id in place"): a failure after the bytes are written but
    // before the rename leaves NO hub.id and no temp; an in-place writer leaves the file.
    #[test]
    fn e166_failure_before_rename_leaves_no_id_and_no_temp() {
        let d = tmp("inject");
        FAIL_BEFORE_RENAME.with(|c| c.set(true));
        let r = load_or_mint(&d);
        FAIL_BEFORE_RENAME.with(|c| c.set(false));
        assert!(r.is_err());
        assert_eq!(std::fs::read_dir(&d).unwrap().count(), 0, "no hub.id, no temp");
        // and the next start mints normally (crash table row 1)
        assert!(load_or_mint(&d).is_ok());
        let _ = std::fs::remove_dir_all(&d);
    }

    // Crash table: a temp left by a dead start is removed by the next write.
    #[test]
    fn e166_stale_temp_from_a_dead_start_is_removed() {
        let d = tmp("stale");
        std::fs::write(d.join(".hub.id.tmp-999999"), "cacc").unwrap();
        load_or_mint(&d).unwrap();
        assert!(!d.join(".hub.id.tmp-999999").exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn e166_atomic_write_never_leaves_a_temp() {
        let d = tmp("atomic");
        load_or_mint(&d).unwrap();
        let temps: Vec<_> = std::fs::read_dir(&d)
            .unwrap()
            .filter_map(|e| e.ok())
            .filter(|e| e.file_name().to_string_lossy().contains(".tmp-"))
            .collect();
        assert!(temps.is_empty());
        let _ = std::fs::remove_dir_all(&d);
    }

    // E-37 hub half / FCW-82: key mode, contents.
    #[test]
    fn e37_fcw82_bad_key_file_refuses() {
        let d = tmp("fcw82");
        load_or_mint(&d).unwrap();
        std::fs::set_permissions(d.join(KEY_FILE), std::fs::Permissions::from_mode(0o644)).unwrap();
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("mode 644"), "{e}");
        std::fs::write(d.join(KEY_FILE), "nothex\n").unwrap();
        std::fs::set_permissions(d.join(KEY_FILE), std::fs::Permissions::from_mode(0o600)).unwrap();
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("malformed") && !e.contains("nothex"), "{e}");
        let _ = std::fs::remove_dir_all(&d);
    }

    // AL-18: key missing beside an existing id refuses (not regenerated silently).
    #[test]
    fn e37_key_missing_beside_id_refuses() {
        let d = tmp("nokey");
        load_or_mint(&d).unwrap();
        std::fs::remove_file(d.join(KEY_FILE)).unwrap();
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("hub.signing.key missing"), "{e}");
        let _ = std::fs::remove_dir_all(&d);
    }

    // E-37 / SI-25: the debug form and fingerprint carry no seed bytes.
    #[test]
    fn e37_private_key_is_never_printed() {
        let d = tmp("si25");
        let id = load_or_mint(&d).unwrap();
        let seed = std::fs::read_to_string(d.join(KEY_FILE)).unwrap();
        let shown = format!("{id:?} {}", id.signing_key_fingerprint());
        assert!(!shown.contains(seed.trim()));
        assert!(id.signing_key_fingerprint().starts_with("ed25519:"));
        assert_eq!(id.signing_key_fingerprint().len(), "ed25519:".len() + 64);
        let _ = std::fs::remove_dir_all(&d);
    }

    // adopt: idempotent, keeps an existing id, socket-only legacy gets random, dry-run writes nothing.
    #[test]
    fn adopt_is_idempotent_and_never_changes_an_existing_id() {
        let d = tmp("adopt");
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        let dry = adopt_legacy(&d, true).unwrap();
        assert_eq!(dry.id_source, "random");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        let a = adopt_legacy(&d, false).unwrap();
        assert_eq!((a.id_source, a.key_generated), ("random", true));
        let b = adopt_legacy(&d, false).unwrap();
        assert_eq!((b.hub_id.as_str(), b.id_source, b.key_generated), (a.hub_id.as_str(), "kept", false));
        assert_eq!(b.signing_key, a.signing_key);
        write_cert(&d); // a later cert never changes the adopted id
        assert_eq!(adopt_legacy(&d, false).unwrap().hub_id, a.hub_id);
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn adopt_refuses_a_malformed_id_rather_than_replacing_it() {
        let d = tmp("adoptbad");
        std::fs::write(d.join(ID_FILE), "nope\n").unwrap();
        assert!(adopt_legacy(&d, false).is_err());
        assert_eq!(std::fs::read_to_string(d.join(ID_FILE)).unwrap(), "nope\n");
        let _ = std::fs::remove_dir_all(&d);
    }
}
