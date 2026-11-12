//! T-3696 (B-11) tests: E-47 (ciphertext only, writer header), E-48 (register mismatch refused; A's blob does not
//! open with B's key), E-144 (register entry matches the private key), E-168 (bad register refuses start).
//! Test names carry the E-n so the fixture suite can print a PASS line per E-n.

use super::*;
use std::os::unix::fs::PermissionsExt;

const CANARY: &[u8] = b"T3696-CANARY-0123456789abcdefXYZ";

fn me_uid() -> u32 {
    // SAFETY: geteuid has no preconditions.
    unsafe { libc::geteuid() }
}

fn tmp() -> PathBuf {
    let d = std::env::temp_dir().join(format!("t3696-{}-{}", std::process::id(), OsRng.next_u64()));
    std::fs::create_dir_all(&d).unwrap();
    d
}

fn write_reg(dir: &Path, name: &str, body: &str, mode: u32) -> PathBuf {
    let p = dir.join(name);
    std::fs::write(&p, body).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(mode)).unwrap();
    p
}

fn aad<'a>(to: &'a str, ps: &'a str) -> AadFields<'a> {
    AadFields { id: "m-1", from: "fx-a", to, conv: "c-1", plain_sha256: ps, size: CANARY.len() as u64 }
}

// ── E-144: the register entry matches the private key; start succeeds and status says match ──

#[test]
fn e144_register_matches_private_key_starts() {
    let d = tmp();
    let store = d.join("store");
    let k = load_key(&key_path(&store), false).unwrap(); // test estate mints
    let reg = write_reg(&d, "fx-a.x25519.pub", &format!("{}\n", k.public_hex()), 0o644);
    let got = startup(&store, Some(&reg), true, me_uid()).unwrap().expect("key loaded");
    assert_eq!(got.0.public_hex(), k.public_hex());
    let v = status_view(Some(&got));
    assert_eq!(v["register"], "match");
    assert_eq!(v["fingerprint"], json!(k.fingerprint()));
    // base64 register form is accepted too
    use base64::Engine;
    let reg64 = write_reg(&d, "b64.pub", &base64::engine::general_purpose::STANDARD.encode(k.public().as_bytes()), 0o644);
    assert!(startup(&store, Some(&reg64), true, me_uid()).unwrap().is_some());
}

#[test]
fn e144_key_file_mode_and_production_never_mints() {
    let d = tmp();
    let store = d.join("store");
    let err = load_key(&key_path(&store), true).err().expect("production refuses a missing key");
    assert!(err.contains("never mints"), "{err}");
    assert!(!key_path(&store).exists(), "nothing minted");
    let k = load_key(&key_path(&store), false).unwrap();
    let mode = std::fs::metadata(key_path(&store)).unwrap().permissions().mode() & 0o777;
    assert_eq!(mode, 0o600);
    assert_eq!(load_key(&key_path(&store), true).unwrap().public_hex(), k.public_hex(), "reload is stable");
    std::fs::set_permissions(key_path(&store), std::fs::Permissions::from_mode(0o640)).unwrap();
    assert!(load_key(&key_path(&store), true).err().unwrap().contains("0600"));
}

#[test]
fn e144_no_register_is_rollback_state_not_a_refusal() {
    // NID / rollback: an agent without a configured register starts; blobs are refused, never plaintext.
    let d = tmp();
    assert!(startup(&d.join("store"), None, true, 0).unwrap().is_none());
    assert_eq!(status_view(None)["state"], "not provisioned");
}

// ── E-48: a swapped register entry refuses start with both fingerprints; B's key does not open A's blob ──

#[test]
fn e48_swapped_register_refused_with_both_fingerprints() {
    let d = tmp();
    let (sa, sb) = (d.join("a"), d.join("b"));
    let ka = load_key(&key_path(&sa), false).unwrap();
    let kb = load_key(&key_path(&sb), false).unwrap();
    // A's sidecar with B's register entry (swapped).
    let reg_b = write_reg(&d, "fx-a.x25519.pub", &kb.public_hex(), 0o644);
    let err = startup(&sa, Some(&reg_b), true, me_uid()).err().expect("mismatch refuses start");
    assert!(err.contains("FCW-109"), "{err}");
    assert!(err.contains(&ka.fingerprint()) && err.contains(&kb.fingerprint()), "both fingerprints: {err}");
}

#[test]
fn e48_blob_to_a_does_not_open_with_b() {
    let (ka, kb) = (EncKey::generate(), EncKey::generate());
    let ps = hex(&Sha256::digest(CANARY));
    let a = aad("fx-a", &ps).bytes();
    let s = seal(CANARY, &ka.public(), &a).unwrap();
    assert_eq!(s.wrapped_key.len(), WRAPPED_KEY_LEN);
    assert_eq!(open(&s.obj, &s.wrapped_key, &ka, &a).unwrap(), CANARY, "A opens its own blob");
    let e = open(&s.obj, &s.wrapped_key, &kb, &a).expect_err("B cannot open A's blob");
    assert!(e.contains("does not open"), "{e}");
}

#[test]
fn e48_tamper_and_wrong_aad_fail_authenticated_decryption() {
    let k = EncKey::generate();
    let ps = hex(&Sha256::digest(CANARY));
    let a = aad("main", &ps).bytes();
    let s = seal(CANARY, &k.public(), &a).unwrap();
    let mut flipped = s.obj.clone();
    let last = flipped.len() - 1;
    flipped[last] ^= 1;
    assert!(open(&flipped, &s.wrapped_key, &k, &a).unwrap_err().contains("authenticated decryption"));
    // the role address is bound verbatim: rewriting `to` to the receiving agent's name does not open (CR-4.10)
    let other = aad("fx-a", &ps).bytes();
    assert!(open(&s.obj, &s.wrapped_key, &k, &other).is_err());
    assert_eq!(open(&s.obj, &s.wrapped_key, &k, &a).unwrap(), CANARY);
}

// ── E-168: a register entry missing, not owner-correct, or group/world-writable refuses start naming it ──

#[test]
fn e168_bad_register_refuses_start_naming_the_entry() {
    let d = tmp();
    let store = d.join("store");
    let k = load_key(&key_path(&store), false).unwrap();
    let missing = d.join("missing.x25519.pub");
    let e = startup(&store, Some(&missing), true, me_uid()).err().unwrap();
    assert!(e.contains("FCW-110") && e.contains("missing.x25519.pub"), "{e}");

    let ok = write_reg(&d, "ok.pub", &k.public_hex(), 0o644);
    let e = startup(&store, Some(&ok), true, me_uid().wrapping_add(1)).err().expect("wrong owner refuses");
    assert!(e.contains("owned by uid") && e.contains("ok.pub"), "{e}");

    for (name, mode) in [("gw.pub", 0o664), ("ww.pub", 0o646)] {
        let p = write_reg(&d, name, &k.public_hex(), mode);
        let e = startup(&store, Some(&p), true, me_uid()).err().expect("writable register refuses");
        assert!(e.contains("writable") && e.contains(name), "{e}");
    }

    let garbage = write_reg(&d, "garbage.pub", "not a key", 0o644);
    assert!(startup(&store, Some(&garbage), true, me_uid()).err().unwrap().contains("32-byte"));

    let link = d.join("link.pub");
    std::os::unix::fs::symlink(&ok, &link).unwrap();
    assert!(startup(&store, Some(&link), true, me_uid()).err().unwrap().contains("not a regular file"));
}

// ── E-47: the object a wrapper writer stores holds no plaintext and passes the hub's writer check ──

#[test]
fn e47_sealed_object_is_ciphertext_only_with_wrapper_header() {
    let k = EncKey::generate();
    let ps = hex(&Sha256::digest(CANARY));
    let a = aad("fx-a", &ps).bytes();
    let s = seal(CANARY, &k.public(), &a).unwrap();
    assert!(!s.obj.windows(CANARY.len()).any(|w| w == CANARY), "canary absent from the stored object");
    assert!(!s.wrapped_key.windows(8).any(|w| CANARY.windows(8).any(|c| c == w)));
    assert_eq!(&s.obj[..6], tlblob::MAGIC);
    assert_eq!(s.obj[6], tlblob::VERSION);
    assert!(tlblob::check_header(&s.obj).is_ok(), "the hub writer check accepts it");
    assert!(tlblob::check_header(CANARY).is_err(), "and refuses the plaintext");
    assert_eq!(s.cipher_sha256, hex(&Sha256::digest(&s.obj)));
    assert_eq!(s.obj.len(), tlblob::HEADER_LEN + CANARY.len() + tlblob::TAG_LEN);
}
