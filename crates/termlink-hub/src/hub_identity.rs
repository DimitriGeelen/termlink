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

    /// T-3703 (design v2 5.1): the 32-byte public key as 64 lower hex, offered in `hub.version` so a
    /// sidecar can verify hub-signed state after checking `sha256(key)` against its pin.
    pub fn signing_public_key_hex(&self) -> String {
        hex(self.signing.verifying_key().as_bytes())
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

/// Prior hub state in `dir`: a secret, a certificate, a signing key, or any topic log
/// (AL-17 step 1). Operator ruling B, 2026-11-16 (T-3366): a certificate alone IS prior state,
/// so a certificate-only directory refuses (FCW-79) until `hub adopt-id` gives it the same
/// certificate-derived id E-36 expects. Fail-closed over E-36's unreachable "start" reading.
pub fn has_prior_state(dir: &Path) -> bool {
    ["hub.secret", "hub.cert.pem", KEY_FILE].iter().any(|f| dir.join(f).exists())
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

#[cfg(test)]
thread_local! {
    /// E-166 fault injector (tests only): fail after the temp is written, before the rename.
    static FAIL_BEFORE_RENAME: std::cell::Cell<Option<&'static str>> = const { std::cell::Cell::new(None) };
}

/// Write `contents` to `dir/name` atomically: temp (exact `mode`) + fsync + no-clobber publish +
/// dir fsync. On any error the temp is removed and no partial file is left (FCW-81).
fn atomic_write(dir: &Path, name: &str, contents: &[u8], mode: u32) -> std::io::Result<()> {
    let path = dir.join(name);
    // Crash table (AL-17 2.1.f): a temp left by a start that died before its publish is removed.
    // Only a DEAD writer's temp: a concurrent start may be writing its own right now.
    if let Ok(rd) = std::fs::read_dir(dir) {
        let prefix = format!(".{name}.tmp-");
        for e in rd.flatten() {
            let fname = e.file_name().to_string_lossy().into_owned();
            let Some(rest) = fname.strip_prefix(&prefix) else { continue };
            let pid: Option<i32> = rest.split('-').next().and_then(|p| p.parse().ok());
            // SAFETY: kill(pid, 0) only probes for existence.
            let dead = pid.is_none_or(|p| p <= 0 || (unsafe { libc::kill(p, 0) } != 0
                && std::io::Error::last_os_error().raw_os_error() == Some(libc::ESRCH)));
            if dead {
                let _ = std::fs::remove_file(e.path());
            }
        }
    }
    static SEQ: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let tmp: PathBuf = dir.join(format!(
        ".{name}.tmp-{}-{}",
        std::process::id(),
        SEQ.fetch_add(1, std::sync::atomic::Ordering::Relaxed)
    ));
    let res = (|| {
        use std::os::unix::fs::OpenOptionsExt;
        let mut f = std::fs::OpenOptions::new().create_new(true).write(true).mode(mode).open(&tmp)?;
        {
            // the mode argument is masked by the umask; set it exactly (review r3 #5: E-36.4 asks 0644)
            use std::os::unix::fs::PermissionsExt;
            f.set_permissions(std::fs::Permissions::from_mode(mode))?;
        }
        f.write_all(contents)?;
        f.sync_all()?;
        drop(f);
        #[cfg(test)]
        if FAIL_BEFORE_RENAME.with(|c| c.get()) == Some(name) {
            return Err(std::io::Error::other("injected: crash before rename"));
        }
        // No-clobber publish (review T-3366 r1 #3): link the complete temp to the final name;
        // `AlreadyExists` when another start got there first, and that file is never replaced.
        std::fs::hard_link(&tmp, &path)?;
        let _ = std::fs::remove_file(&tmp);
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
    // Exactly 64 hex digits and at most one trailing newline. The digit check comes first:
    // from_str_radix accepts a leading '+', and slicing a non-ASCII byte string would panic
    // instead of refusing (review confirmation round, claude-code #3).
    let s = raw.strip_suffix('\n').unwrap_or(&raw);
    let bytes: Option<Vec<u8>> = (s.len() == 64 && s.bytes().all(|b| b.is_ascii_hexdigit()))
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

pub const PENDING_FILE: &str = ".hub-identity.pending";

/// Finish a mint the journal says was started (review T-3366 r2 #5): the key and the id are two
/// files, so a crash between their writes would otherwise leave a state every later start refuses.
/// The marker holds the chosen id; the key is written first, then the id, then the marker goes.
/// A complete identity already on disk wins over the marker (a racing start finished first).
/// `Ok(None)` when no mint is pending.
fn finish_pending(dir: &Path) -> std::io::Result<Option<HubIdentity>> {
    let marker = dir.join(PENDING_FILE);
    let raw = match std::fs::read_to_string(&marker) {
        Ok(s) => s,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
        Err(e) => return Err(refuse(format!("{PENDING_FILE} unreadable ({e})"))),
    };
    let want = raw.trim_end_matches('\n');
    if !is_id(want) {
        return Err(refuse(format!("{PENDING_FILE} is malformed; remove it only after checking hub.id by hand")));
    }
    // Review r3 #1: a marker that outlived a finished mint must not re-key a hub that later lost
    // its key. The key is always published before the id, so an id on disk with no key is a loss.
    let id_now = read_id(dir)?;
    let key_now = read_key(dir)?;
    if id_now.is_some() && key_now.is_none() {
        return Err(refuse(format!(
            "hub.signing.key missing beside hub.id in {} ({PENDING_FILE} is stale); not re-keying (a new key is an operator action)",
            dir.display()
        )));
    }
    let signing = match key_now {
        Some(k) => k,
        None => match generate_key(dir) {
            Ok(k) => k,
            Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read_key(dir)?.ok_or(e)?,
            Err(e) => return Err(e),
        },
    };
    let hub_id = match write_id(dir, want) {
        Ok(()) => want.to_string(),
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => read_id(dir)?.ok_or(e)?,
        Err(e) => return Err(e),
    };
    if let Err(e) = std::fs::remove_file(&marker)
        && e.kind() != std::io::ErrorKind::NotFound
    {
        tracing::warn!(error = %e, "{PENDING_FILE} could not be removed after the mint finished");
    }
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(Some(HubIdentity { hub_id, signing }))
}

/// Journal a mint of `id`, then finish it. A marker already there (a concurrent mint) is used as is.
fn mint_both(dir: &Path, id: &str) -> std::io::Result<HubIdentity> {
    std::fs::create_dir_all(dir)?;
    match atomic_write(dir, PENDING_FILE, format!("{id}\n").as_bytes(), 0o644) {
        Ok(()) => {}
        Err(e) if e.kind() == std::io::ErrorKind::AlreadyExists => {}
        Err(e) => return Err(std::io::Error::new(e.kind(), format!("hub.id could not be written atomically: {e}"))),
    }
    if let Some(done) = finish_pending(dir)? {
        return Ok(done);
    }
    // A concurrent start finished the mint and removed the marker first: take its result.
    match (read_id(dir)?, read_key(dir)?) {
        (Some(hub_id), Some(signing)) => Ok(HubIdentity { hub_id, signing }),
        _ => Err(refuse("hub identity mint vanished while being finished")),
    }
}

/// Hub start (AL-17 step 2, AL-18): read the id and key, finish an interrupted mint, or mint on a
/// first start. Must run BEFORE the start writes its own secret or certificate, so a first start
/// is recognised as one.
/// Review r3 #5: before FCW-79 refuses, look once more — a concurrent first start may have written
/// its marker and key between our reads. A finished identity found now is used, not refused.
fn finish_pending_again(dir: &Path) -> std::io::Result<Option<HubIdentity>> {
    if let Some(done) = finish_pending(dir)? {
        return Ok(Some(done));
    }
    match (read_id(dir)?, read_key(dir)?) {
        (Some(hub_id), Some(signing)) => Ok(Some(HubIdentity { hub_id, signing })),
        _ => Ok(None),
    }
}

pub fn load_or_mint(dir: &Path) -> std::io::Result<HubIdentity> {
    if let Some(done) = finish_pending(dir)? {
        tracing::info!(hub_id = %done.hub_id, "hub identity: an interrupted mint was finished (T-3366)");
        return Ok(done);
    }
    match (read_id(dir)?, read_key(dir)?) {
        (Some(hub_id), Some(signing)) => Ok(HubIdentity { hub_id, signing }),
        // The key is always written before the id, so an id without a key is a loss, not a crash.
        (Some(_), None) => Err(refuse(format!(
            "hub.signing.key missing beside existing state in {}; restore it (a new key is an operator action)",
            dir.display()
        ))),
        (None, _) if has_prior_state(dir) => match finish_pending_again(dir)? {
            Some(done) => Ok(done),
            None => Err(refuse(format!(
                "hub.id missing beside existing state in {}; restore it, or run `termlink hub adopt-id` on a hub that predates T-3366 (runme does this before a restart)",
                dir.display()
            ))),
        },
        (None, _) => {
            let (id, from_cert) = mint_value(dir);
            let done = mint_both(dir, &id)?;
            tracing::info!(hub_id = %done.hub_id, from_certificate = from_cert, "hub.id minted (first start, T-3366)");
            Ok(done)
        }
    }
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
/// T-3366 its id and signing key, without renaming any inbox. Idempotent. It acts ONLY on a
/// pre-T-3366 directory (neither `hub.id` nor `hub.signing.key`); a directory that has one
/// without the other was already migrated and lost a file, and is refused, never re-minted or
/// re-keyed (review T-3366 r1 #1: that would undo FCW-79 / AL-18 through the deploy path).
/// `expect`: the id the hub serves today; the call refuses, writing nothing, when the id it
/// would write (or finds) differs, so a wrong id never lands on disk.
pub fn adopt_legacy(dir: &Path, dry_run: bool, expect: Option<&str>) -> std::io::Result<Adopted> {
    let check = |found: &str| -> std::io::Result<()> {
        match expect {
            Some(e) if e != found => Err(refuse(format!(
                "hub.id would be {found} but the hub serves {e}; nothing written (an inbox would be renamed)"
            ))),
            _ => Ok(()),
        }
    };
    // An interrupted migration (or first start) is finished, not refused (review r2 #5).
    if let Ok(raw) = std::fs::read_to_string(dir.join(PENDING_FILE)) {
        let id = raw.trim_end_matches('\n').to_string();
        check(&id)?;
        if dry_run {
            return Ok(Adopted { hub_id: id, id_source: "pending", key_generated: false, signing_key: "(an interrupted migration would be finished)".into() });
        }
        let had_key = read_key(dir)?.is_some();
        let done = finish_pending(dir)?.ok_or_else(|| refuse("pending migration vanished"))?;
        return Ok(Adopted { hub_id: done.hub_id.clone(), id_source: "pending", key_generated: !had_key, signing_key: done.signing_key_fingerprint() });
    }
    match (read_id(dir)?, read_key(dir)?) {
        (Some(id), Some(k)) => {
            check(&id)?;
            Ok(Adopted { hub_id: id, id_source: "kept", key_generated: false, signing_key: signing_fingerprint(&k) })
        }
        (Some(_), None) => Err(refuse(format!(
            "hub.signing.key missing beside hub.id in {}: this hub was already migrated and lost its key; adopt-id does not re-key (a new key is an operator action)",
            dir.display()
        ))),
        (None, Some(_)) => Err(refuse(format!(
            "hub.id missing beside hub.signing.key in {}: this hub was already migrated and lost its id; restore hub.id, adopt-id does not re-mint",
            dir.display()
        ))),
        (None, None) => {
            let (id, from_cert) = mint_value(dir);
            check(&id)?;
            let id_source = if from_cert { "certificate" } else { "random" };
            if dry_run {
                return Ok(Adopted { hub_id: id, id_source, key_generated: true, signing_key: "(would be generated)".into() });
            }
            let done = mint_both(dir, &id)?;
            check(&done.hub_id)?;
            Ok(Adopted { hub_id: done.hub_id.clone(), id_source, key_generated: true, signing_key: done.signing_key_fingerprint() })
        }
    }
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

    // E-36 under ruling B: a cert-only dir is prior state; adopt-id gives it first16(fingerprint).
    #[test]
    fn e36_cert_only_dir_refuses_until_adopt_gives_the_certificate_id() {
        let d = tmp("cert");
        let want = write_cert(&d);
        // ruling B: a certificate-only directory is prior state — refused, nothing written ...
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("hub.id missing beside existing state"), "{e}");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        // ... until adopt-id gives it first16(fingerprint) (E-36: no inbox renamed)
        let ad = adopt_legacy(&d, false, None).unwrap();
        assert_eq!((ad.hub_id.as_str(), ad.id_source), (want.as_str(), "certificate"));
        assert_eq!(load_or_mint(&d).unwrap().hub_id, want);
        // and adopt on a secret+cert legacy hub gives the same certificate id
        let d2 = tmp("cert2");
        let want2 = write_cert(&d2);
        std::fs::write(d2.join("hub.secret"), "ab".repeat(32)).unwrap();
        let ad = adopt_legacy(&d2, false, None).unwrap();
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
        adopt_legacy(&d, false, None).unwrap();
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
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("hub.id missing beside existing state"), "{e}");
        assert!(!d.join(ID_FILE).exists());
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

    // E-166 control (mutant "write hub.id in place"): a failure after hub.id's bytes are written but
    // before its publish leaves NO hub.id and no temp; the journal (marker + key) stays, and the next
    // start finishes the mint with the journalled id. An in-place writer leaves hub.id behind.
    #[test]
    fn e166_failure_before_rename_leaves_no_id_and_no_temp() {
        let d = tmp("inject");
        FAIL_BEFORE_RENAME.with(|c| c.set(Some(ID_FILE)));
        let r = load_or_mint(&d);
        FAIL_BEFORE_RENAME.with(|c| c.set(None));
        assert!(r.is_err());
        assert!(!d.join(ID_FILE).exists(), "no hub.id");
        let temps = std::fs::read_dir(&d).unwrap().flatten().filter(|e| e.file_name().to_string_lossy().contains(".tmp-")).count();
        assert_eq!(temps, 0, "no temp");
        let journalled = std::fs::read_to_string(d.join(PENDING_FILE)).unwrap().trim().to_string();
        let i = load_or_mint(&d).unwrap();
        assert_eq!(i.hub_id, journalled, "the next start finishes the journalled mint");
        assert!(!d.join(PENDING_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r3 #5: a failure at the KEY publish leaves the marker only; the next start finishes it.
    #[test]
    fn e166_failure_at_the_key_write_is_finished_by_the_next_start() {
        let d = tmp("injectkey");
        FAIL_BEFORE_RENAME.with(|c| c.set(Some(KEY_FILE)));
        let r = load_or_mint(&d);
        FAIL_BEFORE_RENAME.with(|c| c.set(None));
        assert!(r.is_err());
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        assert!(d.join(PENDING_FILE).exists());
        let i = load_or_mint(&d).unwrap();
        assert!(d.join(KEY_FILE).exists() && read_id(&d).unwrap().as_deref() == Some(i.hub_id.as_str()));
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r3 #1: a STALE marker beside an id whose key was lost later never re-keys the hub.
    #[test]
    fn e37_stale_marker_beside_an_id_without_key_is_refused_not_rekeyed() {
        let d = tmp("stalemark");
        let first = load_or_mint(&d).unwrap();
        std::fs::write(d.join(PENDING_FILE), format!("{}\n", first.hub_id)).unwrap(); // outlived the mint
        std::fs::remove_file(d.join(KEY_FILE)).unwrap(); // key lost later
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("not re-keying"), "{e}");
        assert!(!d.join(KEY_FILE).exists(), "no new key written");
        assert!(adopt_legacy(&d, false, None).is_err(), "adopt refuses too");
        assert!(!d.join(KEY_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r3 #5: hub.id is 0644 whatever the umask (E-36.4).
    #[test]
    fn e36_id_mode_is_0644_under_a_strict_umask() {
        let d = tmp("umask");
        // SAFETY: umask has no preconditions; restored at once.
        let old = unsafe { libc::umask(0o077) };
        let r = load_or_mint(&d);
        unsafe { libc::umask(old) };
        r.unwrap();
        assert_eq!(std::fs::metadata(d.join(ID_FILE)).unwrap().permissions().mode() & 0o777, 0o644);
        assert_eq!(std::fs::metadata(d.join(KEY_FILE)).unwrap().permissions().mode() & 0o777, 0o600);
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

    // FCW-82 (confirmation round #3): a '+'-prefixed, non-ASCII or multi-newline key file is
    // refused with the stated message, never accepted and never a panic.
    #[test]
    fn e37_fcw82_malformed_key_variants_are_refused_not_panicking() {
        let d = tmp("keyvar");
        load_or_mint(&d).unwrap();
        for bad in ["+a".repeat(32) + "\n", "é".repeat(32) + "\n", "ab".repeat(32) + "\n\n"] {
            std::fs::write(d.join(KEY_FILE), &bad).unwrap();
            std::fs::set_permissions(d.join(KEY_FILE), std::fs::Permissions::from_mode(0o600)).unwrap();
            let e = load_or_mint(&d).unwrap_err().to_string();
            assert!(e.contains("malformed"), "{bad:?}: {e}");
        }
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
        let dry = adopt_legacy(&d, true, None).unwrap();
        assert_eq!(dry.id_source, "random");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        let a = adopt_legacy(&d, false, None).unwrap();
        assert_eq!((a.id_source, a.key_generated), ("random", true));
        let b = adopt_legacy(&d, false, None).unwrap();
        assert_eq!((b.hub_id.as_str(), b.id_source, b.key_generated), (a.hub_id.as_str(), "kept", false));
        assert_eq!(b.signing_key, a.signing_key);
        write_cert(&d); // a later cert never changes the adopted id
        assert_eq!(adopt_legacy(&d, false, None).unwrap().hub_id, a.hub_id);
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r1 #1: a migrated hub that lost ONE file is refused, never re-keyed or re-minted.
    #[test]
    fn adopt_refuses_a_lost_key_beside_an_id_and_writes_no_key() {
        let d = tmp("lostkey");
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        adopt_legacy(&d, false, None).unwrap();
        std::fs::remove_file(d.join(KEY_FILE)).unwrap();
        let e = adopt_legacy(&d, false, None).unwrap_err().to_string();
        assert!(e.contains("does not re-key"), "{e}");
        assert!(!d.join(KEY_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn adopt_refuses_a_lost_id_beside_a_key_and_writes_no_id() {
        let d = tmp("lostid");
        write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        adopt_legacy(&d, false, None).unwrap();
        std::fs::remove_file(d.join(ID_FILE)).unwrap();
        write_cert(&d); // a rotation since: a re-mint would take the NEW certificate's id
        let e = adopt_legacy(&d, false, None).unwrap_err().to_string();
        assert!(e.contains("does not re-mint"), "{e}");
        assert!(!d.join(ID_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    // `--expect`: a differing id is refused before anything is written; a matching one passes.
    #[test]
    fn adopt_expect_mismatch_writes_nothing() {
        let d = tmp("expect");
        let want = write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        let e = adopt_legacy(&d, false, Some("0123456789abcdef")).unwrap_err().to_string();
        assert!(e.contains("nothing written"), "{e}");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        assert_eq!(adopt_legacy(&d, false, Some(&want)).unwrap().hub_id, want);
        assert!(adopt_legacy(&d, false, Some("0123456789abcdef")).is_err(), "kept id checked too");
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r1 #3: the publish never replaces an existing file.
    #[test]
    fn e166_write_never_clobbers_an_existing_id() {
        let d = tmp("noclobber");
        write_id(&d, "aaaaaaaaaaaaaaaa").unwrap();
        let e = write_id(&d, "bbbbbbbbbbbbbbbb").unwrap_err();
        assert_eq!(e.kind(), std::io::ErrorKind::AlreadyExists);
        assert_eq!(read_id(&d).unwrap().as_deref(), Some("aaaaaaaaaaaaaaaa"));
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r1 #3: concurrent first starts on one directory all end with the SAME id and key.
    #[test]
    fn e166_concurrent_first_starts_converge_on_one_id() {
        let d = tmp("race");
        let hs: Vec<_> = (0..8)
            .map(|_| {
                let d = d.clone();
                std::thread::spawn(move || load_or_mint(&d).map(|i| (i.hub_id.clone(), i.signing_key_fingerprint())))
            })
            .collect();
        let rs: Vec<_> = hs.into_iter().map(|h| h.join().unwrap().unwrap()).collect();
        assert!(rs.iter().all(|r| r == &rs[0]), "{rs:?}");
        assert_eq!(read_id(&d).unwrap().as_deref(), Some(rs[0].0.as_str()));
        let _ = std::fs::remove_dir_all(&d);
    }

    // Review r2 #5: a crash between the key and the id leaves a journal; the next start finishes it.
    #[test]
    fn e166_interrupted_first_start_is_finished_by_the_next_start() {
        let d = tmp("pend1");
        std::fs::write(d.join(PENDING_FILE), "aaaaaaaaaaaaaaaa\n").unwrap();
        let k = generate_key(&d).unwrap(); // the crash happened after the key, before the id
        let i = load_or_mint(&d).unwrap();
        assert_eq!(i.hub_id, "aaaaaaaaaaaaaaaa");
        assert_eq!(i.signing_key_fingerprint(), signing_fingerprint(&k), "the key already written is kept");
        assert!(!d.join(PENDING_FILE).exists());
        assert_eq!(load_or_mint(&d).unwrap().hub_id, "aaaaaaaaaaaaaaaa");
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn e166_interrupted_migration_is_finished_by_adopt_and_checked_against_expect() {
        let d = tmp("pend2");
        let want = write_cert(&d);
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        std::fs::write(d.join(PENDING_FILE), format!("{want}\n")).unwrap(); // crashed before the key
        assert!(adopt_legacy(&d, false, Some("0123456789abcdef")).is_err(), "expect still checked");
        assert!(!d.join(ID_FILE).exists() && !d.join(KEY_FILE).exists());
        let a = adopt_legacy(&d, false, Some(&want)).unwrap();
        assert_eq!((a.hub_id.as_str(), a.id_source, a.key_generated), (want.as_str(), "pending", true));
        assert!(!d.join(PENDING_FILE).exists());
        assert_eq!(load_or_mint(&d).unwrap().hub_id, want);
        let _ = std::fs::remove_dir_all(&d);
    }

    // Without a journal, a key without an id is a LOSS (the key is always written first): refused.
    #[test]
    fn e36_key_without_id_and_no_journal_is_refused() {
        let d = tmp("keyonly");
        std::fs::write(d.join("hub.secret"), "ab".repeat(32)).unwrap();
        generate_key(&d).unwrap();
        let e = load_or_mint(&d).unwrap_err().to_string();
        assert!(e.contains("hub.id missing beside existing state"), "{e}");
        assert!(!d.join(ID_FILE).exists());
        let _ = std::fs::remove_dir_all(&d);
    }

    #[test]
    fn adopt_refuses_a_malformed_id_rather_than_replacing_it() {
        let d = tmp("adoptbad");
        std::fs::write(d.join(ID_FILE), "nope\n").unwrap();
        assert!(adopt_legacy(&d, false, None).is_err());
        assert_eq!(std::fs::read_to_string(d.join(ID_FILE)).unwrap(), "nope\n");
        let _ = std::fs::remove_dir_all(&d);
    }
}
