//! T-3703 (arc-011 P-15, B-5; AL-21): the hub's signed revocation list.
//!
//! `<runtime_dir>/revocations.json` holds ONE signed object, schema `termlink.revocation/1` (06a 2.4 AL-20 via
//! `termlink_session::signed_state`): `{schema, hub_id, seq, issued_at, prev_digest, entries, digest, signature}`.
//! Absent file = the empty list (seq 0, `prev_digest` 64 zeros), served signed but not written.
//!
//! Rules (design note v2, `docs/reports/T-3703-design.md` section 5; operator ruling A 2026-11-18):
//! 1. One writer under [`LOCK`]. The intent admin row is written by the RPC layer BEFORE the call runs
//!    (`rpc_audit::ADMIN_METHODS`, FCW-91); here the list is built, sealed with the hub signing key, written
//!    atomically (FCW-92: a failed write keeps the previous list and fails the call), and only then is the
//!    `revocation-committed` row written with seq, digest and the full entry.
//! 2. `seq` rises by exactly one per change; `prev_digest` chains every version; re-revoking a listed key is
//!    idempotent (no new seq). There is no un-revoke (lists only grow, AL-22 invariant 2).
//! 3. A self-revocation (`termlink.revocation-request/1`) is accepted only for a key that is the current
//!    signing key of a hub key card (T-3704); project and agent come from the card, never from the request
//!    (r1 finding 8). The request carries its `public_key` inside the signed object, must hash to `key`, must
//!    verify under it, and its `at` must be within [`REQUEST_WINDOW_MS`] of the hub clock.
//! 4. Start-up reconcile compares the file with the last committed row: a file behind it (a restore) makes
//!    `head` refuse with REVOCATION_RECONCILE until [`reissue`]; a file ahead of it (a lost commit row) or an
//!    unreadable file is reported for attention.
//! 5. A list whose canonical form would exceed 1 MiB is not issued (06a FCW-88 would refuse it downstream).

use ed25519_dalek::{SigningKey, VerifyingKey};
use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;
use termlink_session::signed_state::{self as ss, J};

pub const LIST_FILE: &str = "revocations.json";
pub const SCHEMA: &str = "termlink.revocation/1";
pub const HEAD_SCHEMA: &str = "termlink.revocation-head/1";
pub const REQUEST_SCHEMA: &str = "termlink.revocation-request/1";
/// A self-revocation request's `at` must lie within this many ms of the hub's clock (design gap 8).
pub const REQUEST_WINDOW_MS: i64 = 10 * 60 * 1000;
/// Longest accepted reason text.
pub const MAX_REASON: usize = 200;
/// Admin-log event written after a list is persisted.
pub const COMMITTED_EVENT: &str = "revocation-committed";

static LOCK: Mutex<()> = Mutex::new(());
/// Directories whose file is behind the admin log (set by [`startup_reconcile`], cleared by [`reissue`]).
static RECONCILE: Mutex<BTreeMap<PathBuf, String>> = Mutex::new(BTreeMap::new());

const ZERO: &str = "0000000000000000000000000000000000000000000000000000000000000000";

pub fn list_path(dir: &Path) -> PathBuf {
    dir.join(LIST_FILE)
}

/// `ed25519:` + 64 lower hex: the AL-18 fingerprint form of a revoked key (design 5.2 / r1 finding 2).
pub fn valid_key(s: &str) -> bool {
    s.strip_prefix("ed25519:").is_some_and(|h| h.len() == 64 && ss::unhex_lower(h).is_some())
}

fn lock() -> std::sync::MutexGuard<'static, ()> {
    LOCK.lock().unwrap_or_else(|p| p.into_inner())
}

fn reconcile_reason(dir: &Path) -> Option<String> {
    RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).get(dir).cloned()
}

fn empty(hub_id: &str) -> J {
    J::Obj(vec![
        ("schema".into(), J::Str(SCHEMA.into())),
        ("hub_id".into(), J::Str(hub_id.into())),
        ("seq".into(), J::Int(0)),
        ("issued_at".into(), J::Int(0)),
        ("prev_digest".into(), J::Str(ZERO.into())),
        ("entries".into(), J::Arr(vec![])),
    ])
}

/// The current list, verified under the hub's own key. Absent file = the empty list, sealed now.
fn load(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
    let bytes = match std::fs::read(list_path(dir)) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return ss::seal(key, empty(hub_id)).map_err(|e| e.to_string());
        }
        Err(e) => return Err(format!("revocation list unreadable: {e}")),
    };
    let list = ss::parse_and_verify(&bytes, &key.verifying_key(), SCHEMA)
        .map_err(|e| format!("revocation list does not verify under the hub key: {e}"))?;
    if list.get("hub_id").and_then(J::as_str) != Some(hub_id) {
        return Err("revocation list names another hub".into());
    }
    Ok(list)
}

fn seq_of(l: &J) -> i64 {
    l.get("seq").and_then(J::as_int).unwrap_or(-1)
}

fn digest_of(l: &J) -> String {
    l.get("digest").and_then(J::as_str).unwrap_or("").to_string()
}

fn entries_of(l: &J) -> Vec<J> {
    match l.get("entries") {
        Some(J::Arr(a)) => a.clone(),
        _ => vec![],
    }
}

fn entry_key(e: &J) -> Option<&str> {
    e.get("key").and_then(J::as_str)
}

/// The text written to disk and served: the sealed object, members in canonical order.
pub fn text_of(l: &J) -> String {
    serde_json::to_string(&l.to_serde()).unwrap_or_default()
}

fn atomic_write(path: &Path, bytes: &[u8]) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    crate::key_card::guard_test_write(dir);
    let tmp = dir.join(format!(".{LIST_FILE}.tmp.{}", std::process::id()));
    {
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(0o644).open(&tmp)?;
        f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(0o644))?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

/// Build, seal, size-check and persist the next version. `log` writes the committed row afterwards.
#[allow(clippy::too_many_arguments)]
fn issue(
    dir: &Path,
    hub_id: &str,
    key: &SigningKey,
    cur: &J,
    entries: Vec<J>,
    now_ms: u64,
    log: impl FnOnce(&J, &[J]) -> Result<(), String>,
    added: &[J],
) -> Result<J, String> {
    let next = J::Obj(vec![
        ("schema".into(), J::Str(SCHEMA.into())),
        ("hub_id".into(), J::Str(hub_id.into())),
        ("seq".into(), J::Int(seq_of(cur) + 1)),
        ("issued_at".into(), J::Int(now_ms as i64)),
        ("prev_digest".into(), J::Str(digest_of(cur))),
        ("entries".into(), J::Arr(entries)),
    ]);
    let sealed = ss::seal(key, next).map_err(|e| e.to_string())?;
    let text = text_of(&sealed);
    if text.len() > ss::MAX_BYTES {
        return Err(format!("the new list would be {} bytes, over the 1 MiB limit; nothing issued", text.len()));
    }
    atomic_write(&list_path(dir), text.as_bytes())
        .map_err(|e| format!("revocation list not written ({e}); the previous list stays (FCW-92)"))?;
    // The list is in force. A lost committed row is reported at the next start (file ahead of the log).
    if let Err(e) = log(&sealed, added) {
        tracing::error!(error = %e, "T-3703: revocation-committed row not written");
    }
    Ok(sealed)
}

/// One revocation to add.
#[derive(Debug, Clone)]
pub struct Revoke {
    pub key: String,
    pub project: String,
    pub agent: String,
    pub reason: String,
    /// `operator` (hub.revoke) or `self` (a signed request).
    pub by: &'static str,
    pub event: &'static str,
}

/// The outcome of a revocation call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Issued {
    pub seq: i64,
    pub digest: String,
    pub already: bool,
}

fn check_names(r: &Revoke) -> Result<(), String> {
    if !valid_key(&r.key) {
        return Err(format!("key {:?} is not ed25519:<64 lower hex> (the key fingerprint)", r.key));
    }
    if !crate::key_card::valid_name(&r.project) {
        return Err(format!("project {:?} is not a plain name", r.project));
    }
    if !crate::key_card::valid_name(&r.agent) {
        return Err(format!("agent {:?} is not a plain name", r.agent));
    }
    if r.reason.trim().is_empty() || r.reason.chars().count() > MAX_REASON || r.reason.chars().any(char::is_control) {
        return Err(format!("reason must be 1..{MAX_REASON} characters without control characters"));
    }
    Ok(())
}

/// AL-21: add `r` to the list (idempotent). `log(new_list, added)` writes the committed admin row.
pub fn revoke(
    dir: &Path,
    hub_id: &str,
    key: &SigningKey,
    r: &Revoke,
    now_ms: u64,
    log: impl FnOnce(&J, &[J]) -> Result<(), String>,
) -> Result<Issued, String> {
    check_names(r)?;
    let _g = lock();
    if let Some(why) = reconcile_reason(dir) {
        return Err(format!("revocation list needs reconcile first ({why}); run hub.revocation_reissue"));
    }
    let cur = load(dir, hub_id, key)?;
    let mut entries = entries_of(&cur);
    if entries.iter().any(|e| entry_key(e) == Some(r.key.as_str())) {
        return Ok(Issued { seq: seq_of(&cur), digest: digest_of(&cur), already: true });
    }
    let entry = J::Obj(vec![
        ("scope".into(), J::Str("agent_key".into())),
        ("key".into(), J::Str(r.key.clone())),
        ("project".into(), J::Str(r.project.clone())),
        ("agent".into(), J::Str(r.agent.clone())),
        ("reason".into(), J::Str(r.reason.clone())),
        ("revoked_at".into(), J::Int(now_ms as i64)),
        ("by".into(), J::Str(r.by.into())),
        ("event".into(), J::Str(r.event.into())),
    ]);
    entries.push(entry.clone());
    let sealed = issue(dir, hub_id, key, &cur, entries, now_ms, log, &[entry])?;
    Ok(Issued { seq: seq_of(&sealed), digest: digest_of(&sealed), already: false })
}

/// The card lookup a self-revocation needs: `(agent, project)` of the card whose current signing key is `pk`.
pub type CardOf<'a> = &'a dyn Fn(&[u8; 32]) -> Result<Option<(String, String)>, String>;

/// FCW-90: verify a `termlink.revocation-request/1` and turn it into a [`Revoke`] by the card that holds the key.
pub fn check_request(req: &J, now_ms: u64, card_of: CardOf<'_>) -> Result<Revoke, String> {
    let pk_hex = req.get("public_key").and_then(J::as_str).ok_or("request has no public_key")?;
    let pk_bytes: [u8; 32] = ss::unhex_lower(pk_hex)
        .and_then(|b| b.try_into().ok())
        .ok_or("public_key is not 64 lower hex")?;
    let pk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| "public_key is not an ed25519 key")?;
    let key = req.get("key").and_then(J::as_str).ok_or("request has no key")?;
    if ss::key_fingerprint(&pk) != key {
        return Err("key is not the fingerprint of public_key: a request revokes only the key that signs it".into());
    }
    ss::verify(&pk, req, Some(REQUEST_SCHEMA)).map_err(|e| format!("request not signed by the key it revokes: {e} (FCW-90)"))?;
    let at = req.get("at").and_then(J::as_int).ok_or("request has no integer at")?;
    if (at - now_ms as i64).abs() > REQUEST_WINDOW_MS {
        return Err(format!("request time {at} is more than 10 minutes from the hub clock {now_ms}"));
    }
    let reason = req.get("reason").and_then(J::as_str).unwrap_or("self-revocation").to_string();
    let Some((agent, project)) = card_of(&pk_bytes)? else {
        return Err("no key card holds this key; only a carded key may revoke itself (the operator revokes others)".into());
    };
    Ok(Revoke { key: key.to_string(), project, agent, reason, by: "self", event: "self-revoke" })
}

/// `revocation.head`: `{schema: termlink.revocation-head/1, hub_id, seq, digest}` signed.
pub fn head(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
    let _g = lock();
    if let Some(why) = reconcile_reason(dir) {
        return Err(format!("RECONCILE: {why}"));
    }
    let l = load(dir, hub_id, key)?;
    let h = J::Obj(vec![
        ("schema".into(), J::Str(HEAD_SCHEMA.into())),
        ("hub_id".into(), J::Str(hub_id.into())),
        ("seq".into(), J::Int(seq_of(&l))),
        ("digest".into(), J::Str(digest_of(&l))),
    ]);
    ss::seal(key, h).map_err(|e| e.to_string())
}

/// `revocation.get`: the current list.
pub fn get(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<J, String> {
    let _g = lock();
    if let Some(why) = reconcile_reason(dir) {
        return Err(format!("RECONCILE: {why}"));
    }
    load(dir, hub_id, key)
}

/// Verified key sets per directory, keyed by the list file's (len, mtime): the post check does not re-verify an
/// unchanged list on every post (r2 OC 9 / CC 12).
/// (len, mtime ns, sha256 of the bytes): the digest closes a same-length rewrite inside one mtime tick (T-3823,
/// T-3703 r3 CC N5); the bytes are hashed, never parsed or signature-checked, on a cache hit.
type Stamp = (u64, u128, String);
static KEY_CACHE: Mutex<BTreeMap<PathBuf, (Stamp, BTreeSet<String>)>> = Mutex::new(BTreeMap::new());
/// Committed-row keys per admin log, by the log's (len, mtime ns): a pending reconcile no longer re-reads the whole
/// log on every carded post (T-3823, T-3703 r3 CC N5).
static COMMITTED_CACHE: Mutex<BTreeMap<PathBuf, ((u64, u128), BTreeSet<String>)>> = Mutex::new(BTreeMap::new());

fn len_mtime(p: &Path) -> Option<(u64, u128)> {
    let m = std::fs::metadata(p).ok()?;
    Some((m.len(), m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos()))
}

fn file_stamp(dir: &Path) -> Option<Stamp> {
    let (len, t) = len_mtime(&list_path(dir))?;
    let bytes = std::fs::read(list_path(dir)).ok()?;
    use sha2::{Digest, Sha256};
    Some((len, t, ss::hex(&Sha256::digest(&bytes))))
}

fn committed_keys(log: &Path) -> BTreeSet<String> {
    let stamp = len_mtime(log);
    if let Some(st) = stamp
        && let Some((s, k)) = COMMITTED_CACHE.lock().unwrap_or_else(|p| p.into_inner()).get(log)
        && *s == st
    {
        return k.clone();
    }
    let keys: BTreeSet<String> = committed_rows(log)
        .iter()
        .flat_map(|(_, _, added)| added.iter().filter_map(|e| e.get("key").and_then(|k| k.as_str()).map(str::to_string)))
        .collect();
    if let Some(st) = stamp {
        COMMITTED_CACHE.lock().unwrap_or_else(|p| p.into_inner()).insert(log.to_path_buf(), (st, keys.clone()));
    }
    keys
}

/// The fingerprints the hub's own post check refuses. An unreadable list is an `Err` (the caller fails closed).
/// While a reconcile is pending (the file is behind the admin log), the keys of the committed rows are added, so
/// a key whose entry the restore lost is still refused (r2 OC 1 / CC 12).
pub fn revoked_keys(dir: &Path, hub_id: &str, key: &SigningKey) -> Result<BTreeSet<String>, String> {
    let _g = lock();
    let stamp = file_stamp(dir);
    let cached = stamp.as_ref().and_then(|st| {
        KEY_CACHE.lock().unwrap_or_else(|p| p.into_inner()).get(dir).filter(|(s, _)| s == st).map(|(_, k)| k.clone())
    });
    let mut keys = match cached {
        Some(k) => k,
        None => {
            let l = load(dir, hub_id, key)?;
            let k: BTreeSet<String> = entries_of(&l).iter().filter_map(|e| entry_key(e).map(str::to_string)).collect();
            if let Some(st) = stamp {
                KEY_CACHE.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), (st, k.clone()));
            }
            k
        }
    };
    if reconcile_reason(dir).is_some() {
        keys.extend(committed_keys(&dir.join(crate::rpc_audit::ADMIN_FILE_NAME)));
    }
    Ok(keys)
}

type Committed = (i64, String, Vec<serde_json::Value>);

/// Committed rows in the admin log, in order: (seq, digest, entries added), and how many lines named the event
/// but could not be read (a torn row; reconcile treats any as a block, r2 OC 6).
/// A line counts as torn only if it is not JSON at all (a write cut short; it may stop before the event name, r3 OC
/// 2) AND it comes after the last readable committed row: a later committed row (a reissue) supersedes it, so a torn
/// line cannot block every later start (r3 CC N1). Readable rows of other events are ignored, whatever their text.
fn committed_rows_checked(admin_log: &Path) -> (Vec<Committed>, usize) {
    let Ok(text) = std::fs::read_to_string(admin_log) else { return (vec![], 0) };
    let mut rows = Vec::new();
    let mut torn_since_last = 0;
    for l in text.lines().filter(|l| !l.trim().is_empty()) {
        let Ok(v) = serde_json::from_str::<serde_json::Value>(l) else {
            torn_since_last += 1;
            continue;
        };
        if v.get("event").and_then(|e| e.as_str()) != Some(COMMITTED_EVENT) {
            continue;
        }
        let row = (|| {
            Some((
                v.get("seq")?.as_i64()?,
                v.get("digest")?.as_str()?.to_string(),
                v.get("added").and_then(|a| a.as_array()).cloned().unwrap_or_default(),
            ))
        })();
        match row {
            Some(r) => {
                rows.push(r);
                torn_since_last = 0;
            }
            None => torn_since_last += 1, // a committed row without its seq or digest
        }
    }
    (rows, torn_since_last)
}

fn committed_rows(admin_log: &Path) -> Vec<Committed> {
    committed_rows_checked(admin_log).0
}

/// Design v2 5.7: compare the file with the last committed row. Returns messages for attention rows; a file
/// behind the log (or unreadable while the log shows commits) blocks `head`/`get`/`revoke` until [`reissue`].
pub fn startup_reconcile(dir: &Path, admin_log: &Path, hub_id: &str, key: &SigningKey) -> Vec<String> {
    startup_reconcile_with(dir, admin_log, hub_id, key, |_| Ok(()))
}

/// [`startup_reconcile`] with `repair`: when the file is ahead of the log (a committed row was lost), the missing
/// committed row is written through `repair`, so a later restore to before it is still detected (r2 CC 12).
pub fn startup_reconcile_with(
    dir: &Path,
    admin_log: &Path,
    hub_id: &str,
    key: &SigningKey,
    repair: impl Fn(serde_json::Value) -> Result<(), String>,
) -> Vec<String> {
    let mut out = Vec::new();
    let (rows, torn) = committed_rows_checked(admin_log);
    let last = rows.last().map(|(s, d, _)| (*s, d.clone()));
    let _g = lock();
    let block = |why: String, out: &mut Vec<String>| {
        RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), why.clone());
        out.push(why);
    };
    if torn > 0 {
        block(format!("{torn} revocation-committed row(s) in the admin log cannot be read; the floor is unknown"), &mut out);
        return out;
    }
    let repair_row = |l: &J| {
        let known: BTreeSet<String> = rows
            .iter()
            .flat_map(|(_, _, a)| a.iter().filter_map(|e| e.get("key").and_then(|k| k.as_str()).map(str::to_string)))
            .collect();
        let added: Vec<J> = entries_of(l).into_iter().filter(|e| entry_key(e).is_some_and(|k| !known.contains(k))).collect();
        repair(committed_row(l, &added, "startup-repair"))
    };
    match (load(dir, hub_id, key), last) {
        (Err(e), Some((s, _))) => block(format!("{e}; the admin log shows seq {s} committed"), &mut out),
        (Err(e), None) => block(e, &mut out),
        (Ok(l), Some((s, d))) => {
            let fs = seq_of(&l);
            if fs < s {
                block(format!("revocation list is at seq {fs}, behind the committed seq {s} (a restore?)"), &mut out);
            } else if fs == s && digest_of(&l) != d {
                block(format!("revocation list seq {fs} has another digest than the committed one (a fork)"), &mut out);
            } else if fs > s {
                let r = repair_row(&l).err().map(|e| format!(" (repair row not written: {e})")).unwrap_or_default();
                out.push(format!("revocation list seq {fs} is ahead of the last committed row (seq {s}): a commit row was lost; repaired{r}"));
            }
        }
        (Ok(l), None) if seq_of(&l) > 0 => {
            let r = repair_row(&l).err().map(|e| format!(" (repair row not written: {e})")).unwrap_or_default();
            out.push(format!("revocation list seq {} has no committed row in the admin log; repaired{r}", seq_of(&l)))
        }
        (Ok(_), None) => {}
    }
    out
}

/// Rebuild after a restore: every entry in the committed rows and in a readable file, at
/// `seq = max(file, committed) + 1`, chained to the file's digest when readable, else the last committed one.
pub fn reissue(
    dir: &Path,
    admin_log: &Path,
    hub_id: &str,
    key: &SigningKey,
    now_ms: u64,
    log: impl FnOnce(&J, &[J]) -> Result<(), String>,
) -> Result<Issued, String> {
    let _g = lock();
    let rows = committed_rows(admin_log);
    let file = load(dir, hub_id, key).ok();
    let mut seen = BTreeSet::new();
    let mut entries = Vec::new();
    let logged = rows.iter().flat_map(|(_, _, a)| a.iter()).filter_map(|v| J::from_serde(v).ok());
    for e in file.iter().flat_map(entries_of).chain(logged) {
        if let Some(k) = entry_key(&e)
            && valid_key(k)
            && seen.insert(k.to_string())
        {
            entries.push(e);
        }
    }
    let logged_top = rows.last().map(|(s, _, _)| *s).unwrap_or(0);
    let file_top = file.as_ref().map(seq_of).unwrap_or(-1);
    let top = logged_top.max(file_top).max(0);
    // chain to the highest version any sidecar may hold, so one holding it sees seq + 1 with a matching prev
    let prev = if file_top >= logged_top {
        file.as_ref().map(digest_of).unwrap_or(ZERO.into())
    } else {
        rows.last().map(|(_, d, _)| d.clone()).unwrap_or(ZERO.into())
    };
    let base = J::Obj(vec![("seq".into(), J::Int(top)), ("digest".into(), J::Str(prev))]);
    let sealed = issue(dir, hub_id, key, &base, entries, now_ms, log, &[])?;
    RECONCILE.lock().unwrap_or_else(|p| p.into_inner()).remove(dir);
    Ok(Issued { seq: seq_of(&sealed), digest: digest_of(&sealed), already: false })
}

/// The committed row for a new list (written by the RPC layer through `rpc_audit::admin_note`).
pub fn committed_row(list: &J, added: &[J], by: &str) -> serde_json::Value {
    serde_json::json!({
        "event": COMMITTED_EVENT, "attention": true, "by": by,
        "seq": seq_of(list), "digest": digest_of(list),
        "added": added.iter().map(J::to_serde).collect::<Vec<_>>(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use ed25519_dalek::Signer;

    const HUB: &str = "cacc73ea32b121dd";

    fn hub_key() -> SigningKey {
        SigningKey::from_bytes(&[3u8; 32])
    }

    fn tmp() -> tempfile::TempDir {
        tempfile::tempdir().unwrap()
    }

    fn op(key: &str) -> Revoke {
        Revoke {
            key: key.into(),
            project: "010-termlink".into(),
            agent: "fx-a".into(),
            reason: "key exposed".into(),
            by: "operator",
            event: "admin-revoke",
        }
    }

    fn fp(n: u8) -> String {
        ss::key_fingerprint(&SigningKey::from_bytes(&[n; 32]).verifying_key())
    }

    /// Record committed rows into `log_path` like the RPC layer does.
    fn logger(log_path: &Path) -> impl FnOnce(&J, &[J]) -> Result<(), String> + '_ {
        move |l: &J, added: &[J]| {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().create(true).append(true).open(log_path).unwrap();
            writeln!(f, "{}", committed_row(l, added, "test")).unwrap();
            Ok(())
        }
    }

    #[test]
    fn empty_list_is_served_signed_at_seq_0() {
        let d = tmp();
        let k = hub_key();
        let h = head(d.path(), HUB, &k).unwrap();
        ss::verify(&k.verifying_key(), &h, Some(HEAD_SCHEMA)).unwrap();
        assert_eq!(h.get("seq"), Some(&J::Int(0)));
        let l = get(d.path(), HUB, &k).unwrap();
        assert_eq!(l.get("prev_digest").and_then(J::as_str), Some(ZERO));
        assert_eq!(h.get("digest").and_then(J::as_str), l.get("digest").and_then(J::as_str));
        assert!(!list_path(d.path()).exists(), "the empty list is not written");
    }

    #[test]
    fn revoke_rises_by_one_chains_and_is_idempotent() {
        let d = tmp();
        let log = d.path().join("admin.jsonl");
        let k = hub_key();
        let l0 = get(d.path(), HUB, &k).unwrap();
        let a = revoke(d.path(), HUB, &k, &op(&fp(1)), 1000, logger(&log)).unwrap();
        assert_eq!((a.seq, a.already), (1, false));
        let l1 = get(d.path(), HUB, &k).unwrap();
        assert_eq!(l1.get("prev_digest").and_then(J::as_str), l0.get("digest").and_then(J::as_str));
        let again = revoke(d.path(), HUB, &k, &op(&fp(1)), 2000, logger(&log)).unwrap();
        assert_eq!((again.seq, again.already), (1, true), "re-revoke: no new seq");
        let b = revoke(d.path(), HUB, &k, &op(&fp(2)), 3000, logger(&log)).unwrap();
        assert_eq!(b.seq, 2);
        let l2 = get(d.path(), HUB, &k).unwrap();
        assert_eq!(l2.get("prev_digest").and_then(J::as_str), Some(a.digest.as_str()));
        assert_eq!(revoked_keys(d.path(), HUB, &k).unwrap().len(), 2);
        assert_eq!(committed_rows(&log).len(), 2, "one committed row per change, none for the repeat");
    }

    #[test]
    fn bad_names_and_keys_are_refused() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join("a");
        let mut r = op("ed25519-pub:00");
        assert!(revoke(d.path(), HUB, &k, &r, 1, logger(&log)).is_err());
        r = op(&fp(1));
        r.agent = "../x".into();
        assert!(revoke(d.path(), HUB, &k, &r, 1, logger(&log)).is_err());
        r = op(&fp(1));
        r.reason = "a\nb".into();
        assert!(revoke(d.path(), HUB, &k, &r, 1, logger(&log)).is_err());
    }

    #[test]
    fn e167_a_list_that_cannot_be_written_leaves_the_previous_one() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join("admin.jsonl");
        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
        // a directory where the temporary file goes: the write fails for any uid, root included
        let blocker = d.path().join(format!(".{LIST_FILE}.tmp.{}", std::process::id()));
        std::fs::create_dir(&blocker).unwrap();
        std::fs::write(blocker.join("x"), b"x").unwrap();
        let e = revoke(d.path(), HUB, &k, &op(&fp(2)), 2, logger(&log)).unwrap_err();
        assert!(e.contains("not written") && e.contains("FCW-92"), "{e}");
        assert_eq!(get(d.path(), HUB, &k).unwrap().get("seq"), Some(&J::Int(1)), "previous list served");
        assert_eq!(committed_rows(&log).len(), 1, "no committed row for the failed call");
    }

    #[test]
    fn a_list_signed_by_another_key_or_hub_is_refused_on_load() {
        let d = tmp();
        let k = hub_key();
        let other = SigningKey::from_bytes(&[9u8; 32]);
        revoke(d.path(), HUB, &other, &op(&fp(1)), 1, |_: &J, _: &[J]| Ok(())).unwrap();
        assert!(get(d.path(), HUB, &k).unwrap_err().contains("does not verify"));
        assert!(get(d.path(), "ffffffffffffffff", &other).unwrap_err().contains("another hub"));
    }

    fn request(sk: &SigningKey, at: i64) -> J {
        let pk = sk.verifying_key();
        let o = J::Obj(vec![
            ("schema".into(), J::Str(REQUEST_SCHEMA.into())),
            ("key".into(), J::Str(ss::key_fingerprint(&pk))),
            ("public_key".into(), J::Str(ss::hex(pk.as_bytes()))),
            ("project".into(), J::Str("claimed-project".into())),
            ("agent".into(), J::Str("claimed-agent".into())),
            ("reason".into(), J::Str("retired".into())),
            ("at".into(), J::Int(at)),
        ]);
        ss::seal(sk, o).unwrap()
    }

    #[test]
    fn e43_self_request_needs_its_own_signature_a_card_and_a_fresh_time() {
        let a = SigningKey::from_bytes(&[1u8; 32]);
        let b = SigningKey::from_bytes(&[2u8; 32]);
        let apk = a.verifying_key().to_bytes();
        let cards = move |pk: &[u8; 32]| Ok(if *pk == apk { Some(("fx-a".to_string(), "010-termlink".to_string())) } else { None });
        let r = check_request(&request(&a, 1_000_000), 1_000_000, &cards).unwrap();
        assert_eq!((r.agent.as_str(), r.project.as_str(), r.by), ("fx-a", "010-termlink", "self"), "names from the card");
        // fx-b signs a request naming fx-a's key: the fingerprint is not of the signing key
        let mut forged = request(&b, 1_000_000);
        forged.set("key", J::Str(ss::key_fingerprint(&a.verifying_key())));
        let forged = ss::seal(&b, forged).unwrap();
        assert!(check_request(&forged, 1_000_000, &cards).unwrap_err().contains("only the key that signs it"));
        // fx-b's own key has no card
        assert!(check_request(&request(&b, 1_000_000), 1_000_000, &cards).unwrap_err().contains("no key card"));
        // a signature over the request by another key, with a's public key claimed
        let mut wrong = request(&a, 1_000_000);
        let d = ss::digest(&wrong).unwrap();
        wrong.set("signature", J::Str(ss::hex(&b.sign(&ss::signing_input(REQUEST_SCHEMA, &d)).to_bytes())));
        assert!(check_request(&wrong, 1_000_000, &cards).unwrap_err().contains("FCW-90"));
        // stale
        assert!(check_request(&request(&a, 0), 1_000_000_000, &cards).unwrap_err().contains("10 minutes"));
    }

    #[test]
    fn restore_behind_the_log_blocks_head_until_reissue() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join("admin.jsonl");
        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
        let backup = std::fs::read(list_path(d.path())).unwrap();
        revoke(d.path(), HUB, &k, &op(&fp(2)), 2, logger(&log)).unwrap();
        std::fs::write(list_path(d.path()), &backup).unwrap(); // the restore
        let msgs = startup_reconcile(d.path(), &log, HUB, &k);
        assert!(msgs.iter().any(|m| m.contains("behind the committed seq 2")), "{msgs:?}");
        assert!(head(d.path(), HUB, &k).unwrap_err().starts_with("RECONCILE"));
        assert!(revoke(d.path(), HUB, &k, &op(&fp(3)), 3, logger(&log)).is_err());
        let r = reissue(d.path(), &log, HUB, &k, 4, logger(&log)).unwrap();
        assert_eq!(r.seq, 3, "above every seq any sidecar may hold");
        let keys = revoked_keys(d.path(), HUB, &k).unwrap();
        assert!(keys.contains(&fp(1)) && keys.contains(&fp(2)), "the entry lost by the restore is back");
        assert!(head(d.path(), HUB, &k).is_ok());
        assert!(startup_reconcile(d.path(), &log, HUB, &k).is_empty());
    }

    #[test]
    fn a_lost_committed_row_is_reported_not_blocking() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join("admin.jsonl");
        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, |_: &J, _: &[J]| Ok(())).unwrap();
        let msgs = startup_reconcile(d.path(), &log, HUB, &k);
        assert!(msgs.iter().any(|m| m.contains("no committed row")), "{msgs:?}");
        assert!(head(d.path(), HUB, &k).is_ok());
    }

    #[test]
    fn a_list_over_one_mib_is_not_issued() {
        let d = tmp();
        let k = hub_key();
        let cur = get(d.path(), HUB, &k).unwrap();
        let big: Vec<J> = (0..20_000u32)
            .map(|i| J::Obj(vec![("key".into(), J::Str(format!("ed25519:{i:064x}"))), ("reason".into(), J::Str("x".repeat(40)))]))
            .collect();
        let e = issue(d.path(), HUB, &k, &cur, big, 1, |_: &J, _: &[J]| Ok(()), &[]).unwrap_err();
        assert!(e.contains("1 MiB"), "{e}");
        assert!(!list_path(d.path()).exists());
    }

    #[test]
    fn r2_a_key_lost_by_a_restore_is_still_refused_and_reissue_chains_to_the_committed_digest() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join(crate::rpc_audit::ADMIN_FILE_NAME);
        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
        let backup = std::fs::read(list_path(d.path())).unwrap();
        let two = revoke(d.path(), HUB, &k, &op(&fp(2)), 2, logger(&log)).unwrap();
        std::fs::write(list_path(d.path()), &backup).unwrap();
        startup_reconcile(d.path(), &log, HUB, &k);
        let keys = revoked_keys(d.path(), HUB, &k).unwrap();
        assert!(keys.contains(&fp(2)), "the post check still refuses the key the restore lost (r2 OC 1)");
        reissue(d.path(), &log, HUB, &k, 3, logger(&log)).unwrap();
        let l = get(d.path(), HUB, &k).unwrap();
        assert_eq!(l.get("seq"), Some(&J::Int(3)));
        assert_eq!(l.get("prev_digest").and_then(J::as_str), Some(two.digest.as_str()), "a sidecar holding seq 2 chains (r2 CC 5)");
    }

    #[test]
    fn r2_a_torn_committed_row_blocks_and_a_lost_row_is_repaired() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join("admin.jsonl");
        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
        {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).open(&log).unwrap();
            writeln!(f, "{{\"event\":\"{COMMITTED_EVENT}\",\"seq\":2,\"dig").unwrap();
        }
        let msgs = startup_reconcile(d.path(), &log, HUB, &k);
        assert!(msgs.iter().any(|m| m.contains("cannot be read")), "{msgs:?}");
        assert!(head(d.path(), HUB, &k).is_err());
        RECONCILE.lock().unwrap().remove(d.path());
        // a list written with no committed row: the start-up check writes the missing row
        let d2 = tmp();
        let log2 = d2.path().join("admin.jsonl");
        revoke(d2.path(), HUB, &k, &op(&fp(1)), 1, |_: &J, _: &[J]| Ok(())).unwrap();
        let rows = std::cell::RefCell::new(vec![]);
        let msgs = startup_reconcile_with(d2.path(), &log2, HUB, &k, |r| {
            rows.borrow_mut().push(r);
            Ok(())
        });
        assert!(msgs.iter().any(|m| m.contains("repaired")), "{msgs:?}");
        let rows = rows.into_inner();
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0]["seq"], 1);
        assert_eq!(rows[0]["added"][0]["key"], fp(1));
    }

    #[test]
    fn r3_a_torn_row_stops_blocking_after_reissue_and_a_tear_before_the_event_name_still_blocks() {
        let d = tmp();
        let k = hub_key();
        let log = d.path().join("admin.jsonl");
        revoke(d.path(), HUB, &k, &op(&fp(1)), 1, logger(&log)).unwrap();
        let tear = |text: &str| {
            use std::io::Write;
            let mut f = std::fs::OpenOptions::new().append(true).open(&log).unwrap();
            writeln!(f, "{text}").unwrap();
        };
        tear("{\"event\":\"revoc"); // cut before the event name is complete (r3 OC 2)
        assert!(!startup_reconcile(d.path(), &log, HUB, &k).is_empty());
        assert!(head(d.path(), HUB, &k).is_err(), "a trailing torn line blocks");
        reissue(d.path(), &log, HUB, &k, 2, logger(&log)).unwrap();
        // a restart: the torn line is still in the append-only log, but a committed row now follows it (r3 CC N1)
        let msgs = startup_reconcile(d.path(), &log, HUB, &k);
        assert!(msgs.is_empty(), "{msgs:?}");
        assert!(head(d.path(), HUB, &k).is_ok());
        // a readable row of another event that merely mentions the event name is not torn
        tear(&serde_json::json!({"event": "revocation-reconcile", "detail": format!("1 {COMMITTED_EVENT} row(s) cannot be read")}).to_string());
        assert!(startup_reconcile(d.path(), &log, HUB, &k).is_empty());
    }

    /// T-3823 (T-3703 r3 CC N5): a same-length rewrite inside one mtime tick is not served from the cache.
    #[test]
    fn t3823_the_post_check_cache_sees_a_same_length_rewrite() {
        let k = hub_key();
        let (a, b) = (tmp(), tmp());
        revoke(a.path(), HUB, &k, &op(&fp(1)), 1, |_: &J, _: &[J]| Ok(())).unwrap();
        revoke(b.path(), HUB, &k, &op(&fp(2)), 1, |_: &J, _: &[J]| Ok(())).unwrap();
        assert!(revoked_keys(a.path(), HUB, &k).unwrap().contains(&fp(1)));
        let (la, lb) = (std::fs::read(list_path(a.path())).unwrap(), std::fs::read(list_path(b.path())).unwrap());
        assert_eq!(la.len(), lb.len(), "same length by construction");
        let mtime = std::fs::metadata(list_path(a.path())).unwrap().modified().unwrap();
        std::fs::write(list_path(a.path()), &lb).unwrap();
        std::fs::File::options().write(true).open(list_path(a.path())).unwrap().set_modified(mtime).unwrap();
        let keys = revoked_keys(a.path(), HUB, &k).unwrap();
        assert!(keys.contains(&fp(2)) && !keys.contains(&fp(1)), "{keys:?}");
    }
}
