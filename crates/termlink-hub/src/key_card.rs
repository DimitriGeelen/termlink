//! T-3704 (arc-011 P-16, B-6, 06a AL-33): agent key cards at the hub and the post-time check.
//!
//! A card binds an agent name to its signing key and project. Cards live in ONE store file
//! (`<runtime_dir>/key-cards.json`, 0600, atomic replace), never in a path built from a name
//! (design v2, review C-M1). They are written only by `hub.key_card_put`, an admin RPC accepted
//! over the local unix socket (operator ruling A 2026-11-18: while agents run as root this is a
//! visible, logged convention, not a barrier).
//!
//! The check runs on every `channel.post` of an arc-011 type (`sidecar.msg.v1`,
//! `sidecar.stage.v1`) to `inbox:*`; every other type and topic is never seen by it (NID). It is
//! behind a hub setting `<runtime_dir>/key-card-check` = `off` (default, file absent) | `log` |
//! `enforce`. The poster's agent is the `from_agent` of the signed payload; it must be a plain
//! name, have a card, and the card's key must be the key that signed the post (the transport
//! signature is verified before this check, unchanged).

use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use serde::{Deserialize, Serialize};
use termlink_protocol::control::error_code;

pub const STORE_FILE: &str = "key-cards.json";
pub const MODE_FILE: &str = "key-card-check";
pub const CARD_SCHEMA: &str = "termlink.key-card/1";
pub const STORE_SCHEMA: &str = "termlink.key-cards/1";
/// The arc-011 message types the check sees (design v2: liveness posts go to `liveness:*`).
pub const GATED_TYPES: [&str; 2] = ["sidecar.msg.v1", "sidecar.stage.v1"];
pub const SIGNING_PREFIX: &str = "ed25519-pub:";
pub const ENCRYPTION_PREFIX: &str = "x25519-pub:";

/// One lock for the store: a post reads its card under it, and a write is a read-modify-write
/// under it, so a post meets either the old or the new card, never a half-written one (AL-33
/// 2.17.f(3)).
static STORE_LOCK: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Mode {
    Off,
    Log,
    Enforce,
}

impl Mode {
    pub fn as_str(self) -> &'static str {
        match self {
            Mode::Off => "off",
            Mode::Log => "log",
            Mode::Enforce => "enforce",
        }
    }
    pub fn parse(s: &str) -> Option<Mode> {
        match s.trim() {
            "off" => Some(Mode::Off),
            "log" => Some(Mode::Log),
            "enforce" => Some(Mode::Enforce),
            _ => None,
        }
    }
}

/// The setting. Absent file = `off`. A file that cannot be read or holds anything else is an
/// `Err`: the caller treats that as `enforce` (fail closed), never as `off`.
pub fn read_mode(dir: &Path) -> Result<Mode, String> {
    match std::fs::read_to_string(dir.join(MODE_FILE)) {
        Ok(s) => Mode::parse(&s).ok_or_else(|| format!("{MODE_FILE} holds {:?}, not off|log|enforce", s.trim())),
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(Mode::Off),
        Err(e) => Err(format!("{MODE_FILE} unreadable: {e}")),
    }
}

/// The effective mode for a check: an unreadable or invalid setting enforces.
pub fn effective_mode(dir: &Path) -> Mode {
    read_mode(dir).unwrap_or(Mode::Enforce)
}

pub fn write_mode(dir: &Path, mode: Mode) -> std::io::Result<()> {
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    write_mode_unlocked(dir, mode)
}

fn write_mode_unlocked(dir: &Path, mode: Mode) -> std::io::Result<()> {
    atomic_write(&dir.join(MODE_FILE), format!("{}\n", mode.as_str()).as_bytes(), 0o644)
}

/// Change the setting with its admin row written FIRST (r2 BLOCKER: refused if the row cannot be
/// written). `log(before, after)` writes the row.
pub fn set_mode(dir: &Path, mode: Mode, log: impl FnOnce(&str, &str) -> Result<(), String>) -> Result<String, String> {
    // read, log and write under one lock, so the file and the last row agree (r3 MINOR 4)
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let before = read_mode(dir).map(|m| m.as_str().to_string()).unwrap_or_else(|_| "invalid".into());
    log(&before, mode.as_str())?;
    write_mode_unlocked(dir, mode).map_err(|e| format!("setting not written: {e}"))?;
    Ok(before)
}

/// The arc-011 types on `inbox:*` the check applies to (tested before anything is read).
pub fn is_gated(topic: &str, msg_type: &str) -> bool {
    topic.starts_with("inbox:") && GATED_TYPES.contains(&msg_type)
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Card {
    pub schema: String,
    pub agent: String,
    pub project: String,
    pub signing_key: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key: Option<String>,
    #[serde(default)]
    pub instances: Vec<String>,
    pub seq: u64,
    pub by: String,
    pub at_ms: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct Store {
    schema: String,
    cards: BTreeMap<String, Card>,
}

/// A plain agent or project name: `^[a-z0-9][a-z0-9._-]{0,63}$` and no `..` (design v2, C-M1).
pub fn valid_name(s: &str) -> bool {
    let b = s.as_bytes();
    !b.is_empty()
        && b.len() <= 64
        && (b[0].is_ascii_lowercase() || b[0].is_ascii_digit())
        && b.iter().all(|c| c.is_ascii_lowercase() || c.is_ascii_digit() || matches!(c, b'.' | b'_' | b'-'))
        && !s.contains("..")
}

fn parse_prefixed_key(s: &str, prefix: &str) -> Option<[u8; 32]> {
    let hex = s.strip_prefix(prefix)?;
    if hex.len() != 64 || !hex.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
        return None;
    }
    let mut out = [0u8; 32];
    for (i, o) in out.iter_mut().enumerate() {
        *o = u8::from_str_radix(&hex[2 * i..2 * i + 2], 16).ok()?;
    }
    Some(out)
}

/// `ed25519-pub:<64 lower hex>` → the 32 key bytes. Compared as bytes, never as strings.
pub fn parse_signing_key(s: &str) -> Option<[u8; 32]> {
    parse_prefixed_key(s, SIGNING_PREFIX)
}

pub fn parse_encryption_key(s: &str) -> Option<[u8; 32]> {
    parse_prefixed_key(s, ENCRYPTION_PREFIX)
}

pub fn store_path(dir: &Path) -> PathBuf {
    dir.join(STORE_FILE)
}

pub fn sha256_hex(b: &[u8]) -> String {
    use sha2::{Digest, Sha256};
    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
}

/// sha256 of a card's serialized form (the admin row carries it; start-up compares it).
pub fn card_digest(c: &Card) -> String {
    sha256_hex(&serde_json::to_vec(c).unwrap_or_default())
}

const ABSENT: &str = "absent";

/// The store digest this hub last wrote or baselined, per runtime dir (r2 MAJOR 2: a store changed
/// outside `hub.key_card_put` is refused until the next start re-baselines it, with attention).
/// No entry = not baselined (unit tests; a hub that never ran the start-up reconcile).
static KNOWN_DIGEST: Mutex<BTreeMap<PathBuf, String>> = Mutex::new(BTreeMap::new());

fn set_known(dir: &Path, digest: String) {
    KNOWN_DIGEST.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), digest);
}

/// Absent store = no cards. A store that exists but cannot be read or parsed is an `Err`, never
/// "no cards" (design v2: an unreadable store refuses every gated post). Returns the file digest.
fn load_raw(dir: &Path) -> Result<(Store, String), String> {
    let p = store_path(dir);
    let bytes = match std::fs::read(&p) {
        Ok(b) => b,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok((Store { schema: STORE_SCHEMA.into(), cards: BTreeMap::new() }, ABSENT.into()));
        }
        Err(e) => return Err(format!("key-card store unreadable: {e}")),
    };
    let s: Store = serde_json::from_slice(&bytes).map_err(|e| format!("key-card store corrupt: {e}"))?;
    if s.schema != STORE_SCHEMA {
        return Err(format!("key-card store has schema {:?}, not {STORE_SCHEMA}", s.schema));
    }
    Ok((s, sha256_hex(&bytes)))
}

/// `load_raw` plus the outside-edit check against the baselined digest.
fn load(dir: &Path) -> Result<Store, String> {
    let (s, d) = load_raw(dir)?;
    if let Some(k) = KNOWN_DIGEST.lock().unwrap_or_else(|p| p.into_inner()).get(dir)
        && *k != d
    {
        return Err(format!(
            "key-card store changed outside hub.key_card_put (digest {} expected {}); restart the hub to re-baseline (raises attention)",
            &d[..12.min(d.len())],
            &k[..12.min(k.len())]
        ));
    }
    Ok(s)
}

fn atomic_write(path: &Path, bytes: &[u8], mode: u32) -> std::io::Result<()> {
    use std::io::Write;
    use std::os::unix::fs::OpenOptionsExt;
    let dir = path.parent().unwrap_or_else(|| Path::new("."));
    let tmp = dir.join(format!(".{}.tmp.{}", path.file_name().and_then(|n| n.to_str()).unwrap_or("f"), std::process::id()));
    {
        let mut f = std::fs::OpenOptions::new().write(true).create(true).truncate(true).mode(mode).open(&tmp)?;
        // exact mode whatever the umask (T-3366 lesson)
        f.set_permissions(std::os::unix::fs::PermissionsExt::from_mode(mode))?;
        f.write_all(bytes)?;
        f.sync_all()?;
    }
    std::fs::rename(&tmp, path)?;
    if let Ok(d) = std::fs::File::open(dir) {
        let _ = d.sync_all();
    }
    Ok(())
}

fn save(dir: &Path, s: &Store) -> std::io::Result<()> {
    let text = serde_json::to_string_pretty(s).map_err(std::io::Error::other)?;
    atomic_write(&store_path(dir), text.as_bytes(), 0o600)?;
    set_known(dir, sha256_hex(text.as_bytes()));
    Ok(())
}

/// The outcome of the post-time check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// Not an arc-011 type on `inbox:*`: the check does not apply.
    NotGated,
    Pass { agent: String },
    /// `carded`: the claimed agent has a card (refusal counts are kept per agent only then; the
    /// rest share one bucket, so attacker-chosen names cannot grow the map, r2 MAJOR 7).
    Refuse { code: i64, fcw: &'static str, agent: String, reason: String, carded: bool },
}

fn refuse_no_card(agent: &str, reason: String) -> Check {
    Check::Refuse { code: error_code::CHANNEL_NO_KEY_CARD, fcw: "FCW-135", agent: agent.to_string(), reason, carded: false }
}

/// The claimed agent of an arc-011 payload: its `from_agent` (msg and stage records both carry it).
pub fn claimed_agent(payload: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(payload).ok()?;
    v.get("from_agent")?.as_str().map(str::to_string)
}

/// The card check for one post whose transport signature has already verified with `signer`.
pub fn check_post(dir: &Path, topic: &str, msg_type: &str, payload: &[u8], signer: &[u8; 32]) -> Check {
    if !is_gated(topic, msg_type) {
        return Check::NotGated;
    }
    let agent = match claimed_agent(payload) {
        Some(a) if valid_name(&a) => a,
        Some(a) => {
            let shown: String = a.chars().take(64).collect();
            return refuse_no_card("?", format!("from_agent {shown:?} is not a plain agent name"));
        }
        None => return refuse_no_card("?", "the payload names no from_agent".into()),
    };
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let store = match load(dir) {
        Ok(s) => s,
        Err(e) => return refuse_no_card(&agent, e),
    };
    let Some(card) = store.cards.get(&agent) else {
        return refuse_no_card(&agent, format!("no key card for agent {agent:?}"));
    };
    match parse_signing_key(&card.signing_key) {
        Some(k) if &k == signer => Check::Pass { agent },
        Some(_) => Check::Refuse {
            code: error_code::CHANNEL_KEY_CARD_MISMATCH,
            fcw: "FCW-133",
            agent: agent.clone(),
            reason: format!("the post is not signed by the key on agent {agent:?}'s card"),
            carded: true,
        },
        None => refuse_no_card(&agent, format!("agent {agent:?}'s card holds no valid signing key")),
    }
}

/// A card write request (from `hub.key_card_put`).
#[derive(Debug, Clone, Deserialize)]
pub struct PutRequest {
    pub agent: String,
    pub project: String,
    pub signing_key: String,
    #[serde(default)]
    pub encryption_key: Option<String>,
    #[serde(default)]
    pub instances: Vec<String>,
    #[serde(default)]
    pub replace: bool,
}

/// Write or replace one card. Returns the new card and the one it replaced. Any change to an
/// existing card (key, project, encryption key, instances) needs `replace: true`, so it is never
/// silent (r2 MAJOR 3). A signing key may sit on one card only. `log(new, old)` writes the admin
/// row and runs BEFORE the store is written; if it fails nothing is written (r2 BLOCKER).
pub fn put(
    dir: &Path,
    req: &PutRequest,
    now_ms: u64,
    log: impl FnOnce(&Card, Option<&Card>) -> Result<(), String>,
) -> Result<(Card, Option<Card>), String> {
    if !valid_name(&req.agent) {
        return Err(format!("agent {:?} is not a plain name", req.agent));
    }
    if !valid_name(&req.project) {
        return Err(format!("project {:?} is not a plain name", req.project));
    }
    if parse_signing_key(&req.signing_key).is_none() {
        return Err(format!("signing_key must be {SIGNING_PREFIX}<64 lower hex>"));
    }
    if let Some(e) = &req.encryption_key
        && parse_encryption_key(e).is_none()
    {
        return Err(format!("encryption_key must be {ENCRYPTION_PREFIX}<64 lower hex>"));
    }
    if let Some(bad) = req.instances.iter().find(|i| !valid_name(i)) {
        return Err(format!("instance {bad:?} is not a plain name"));
    }
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    let mut store = load(dir)?;
    let old = store.cards.get(&req.agent).cloned();
    if let Some(o) = &old
        && (o.signing_key != req.signing_key
            || o.project != req.project
            || o.encryption_key != req.encryption_key
            || o.instances != req.instances)
        && !req.replace
    {
        return Err(format!(
            "agent {:?} already has a card (project {:?}, key {}); changing any of its fields needs replace: true",
            req.agent, o.project, o.signing_key
        ));
    }
    if let Some(other) = store.cards.values().find(|c| c.agent != req.agent && c.signing_key == req.signing_key) {
        return Err(format!("signing key {} is already on agent {:?}'s card; a key belongs to one agent", req.signing_key, other.agent));
    }
    let card = Card {
        schema: CARD_SCHEMA.into(),
        agent: req.agent.clone(),
        project: req.project.clone(),
        signing_key: req.signing_key.clone(),
        encryption_key: req.encryption_key.clone(),
        instances: req.instances.clone(),
        seq: old.as_ref().map_or(1, |o| o.seq + 1),
        // what wrote it; the peer is not authenticated beyond the local socket (operator ruling A)
        by: "hub.key_card_put (local socket)".into(),
        at_ms: now_ms,
    };
    log(&card, old.as_ref())?;
    store.schema = STORE_SCHEMA.into();
    store.cards.insert(card.agent.clone(), card.clone());
    save(dir, &store).map_err(|e| format!("key-card store not written: {e}"))?;
    Ok((card, old))
}

pub fn get(dir: &Path, agent: &str) -> Result<Option<Card>, String> {
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    Ok(load(dir)?.cards.get(agent).cloned())
}

pub fn list(dir: &Path) -> Result<Vec<Card>, String> {
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    Ok(load(dir)?.cards.into_values().collect())
}

/// AL-33 2.17.f(2): compare each card's `seq` with the highest `seq` the admin log records for a
/// `key-card-put` of that agent. A card ahead of the log (a crash between persist and log, or a
/// card written outside `hub.key_card_put`) or an unreadable store is reported; the caller raises
/// attention. Known limit: an admin log pruned past a card's last put reports that card too.
pub fn startup_reconcile(dir: &Path, admin_log: &Path) -> Vec<String> {
    let mut out = Vec::new();
    // the last logged card digest per agent, and the last logged setting
    let mut logged: BTreeMap<String, String> = BTreeMap::new();
    let mut last_mode: Option<String> = None;
    if let Ok(text) = std::fs::read_to_string(admin_log) {
        for v in text.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok()) {
            match v.get("event").and_then(|e| e.as_str()) {
                Some("key-card-put") => {
                    if let (Some(a), Some(d)) = (v.get("agent").and_then(|a| a.as_str()), v.get("card_digest").and_then(|s| s.as_str())) {
                        logged.insert(a.to_string(), d.to_string());
                    }
                }
                Some("key-card-mode") => last_mode = v.get("to").and_then(|m| m.as_str()).map(str::to_string),
                _ => {}
            }
        }
    }
    // the setting: deleting or editing the file is not an admin act (r2 MINOR 10)
    let now = read_mode(dir).map(|m| m.as_str().to_string()).unwrap_or_else(|e| format!("invalid ({e})"));
    match &last_mode {
        Some(m) if *m != now => out.push(format!("key-card check setting is {now} but the admin log's last change set {m}")),
        None if now != "off" => out.push(format!("key-card check setting is {now} with no logged change")),
        _ => {}
    }
    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
    match load_raw(dir) {
        Err(e) => {
            // no file digest equals this, so a later outside write is still refused (r3 MINOR 2)
            set_known(dir, "unreadable-at-start".into());
            out.push(e)
        }
        Ok((store, digest)) => {
            for c in store.cards.values() {
                let d = card_digest(c);
                match logged.get(&c.agent) {
                    Some(l) if *l == d => {}
                    Some(_) => out.push(format!("card {} (seq {}) differs from its last logged key-card-put", c.agent, c.seq)),
                    None => out.push(format!("card {} (seq {}) has no logged key-card-put", c.agent, c.seq)),
                }
            }
            set_known(dir, digest);
        }
    }
    out
}

/// The bucket for refusals naming an agent without a card (bounded: attacker-chosen names).
pub const UNCARDED: &str = "_uncarded";
const MAX_SIGNERS: usize = 16;

#[derive(Default)]
struct Count {
    refused: u64,
    would: u64,
    signers: BTreeMap<String, u64>,
}

/// Refusals per claimed carded agent since hub start (the impersonated agent's sidecar reads these,
/// design v2 C-12), with the signer fingerprints (at most 16 named, the rest "other"), so an agent
/// does not report its own refused posts as someone else's.
static REFUSALS: Mutex<BTreeMap<String, Count>> = Mutex::new(BTreeMap::new());

/// Count one refusal (`enforced`) or one log-mode would-refuse.
pub fn note_refusal(agent: &str, signer_fp: &str, enforced: bool, carded: bool) {
    let key = if carded { agent } else { UNCARDED };
    let mut m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
    let e = m.entry(key.to_string()).or_default();
    if enforced { e.refused += 1 } else { e.would += 1 }
    let s = if e.signers.contains_key(signer_fp) || e.signers.len() < MAX_SIGNERS { signer_fp } else { "other" };
    *e.signers.entry(s.to_string()).or_insert(0) += 1;
}

/// `{agent: {refused, would_refuse, signers: {fp: n}}}`.
pub fn refusal_counts() -> serde_json::Value {
    let m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
    serde_json::Value::Object(
        m.iter()
            .map(|(a, c)| (a.clone(), serde_json::json!({"refused": c.refused, "would_refuse": c.would, "signers": c.signers})))
            .collect(),
    )
}

/// Admin rows for refusals are coalesced per (claimed agent, poster key, minute) so a token
/// holder cannot flood the admin log (design v2, C-12). True = write a row for this one.
static LOGGED: Mutex<(u64, Option<HashSet<(String, String)>>)> = Mutex::new((0, None));
/// At most this many refusal rows per minute for CARDED agents (r2 MAJOR 7). Refusals naming an
/// agent without a card are coalesced under one key per signer, so made-up names can neither flood
/// the log nor use up the carded budget (r3 MAJOR: a real impersonation is always logged).
pub const MAX_ROWS_PER_MINUTE: usize = 60;

pub fn first_in_minute(agent: &str, signer_fp: &str, now_ms: u64, carded: bool) -> bool {
    let minute = now_ms / 60_000;
    let key = if carded { agent } else { UNCARDED };
    let mut g = LOGGED.lock().unwrap_or_else(|p| p.into_inner());
    if g.0 != minute || g.1.is_none() {
        *g = (minute, Some(HashSet::new()));
    }
    match g.1.as_mut() {
        Some(s) if carded && s.iter().filter(|(a, _)| a != UNCARDED).count() >= MAX_ROWS_PER_MINUTE => false,
        Some(s) => s.insert((key.to_string(), signer_fp.to_string())),
        None => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The two tests that drive the process-wide per-minute row state run one at a time.
    static MINUTE_STATE: Mutex<()> = Mutex::new(());

    fn key_hex(b: u8) -> String {
        format!("{SIGNING_PREFIX}{}", format!("{b:02x}").repeat(32))
    }
    fn msg_payload(agent: &str) -> Vec<u8> {
        serde_json::json!({"v": 1, "kind": "msg", "from_agent": agent, "body": "x"}).to_string().into_bytes()
    }
    fn put_req(agent: &str, project: &str, key: u8) -> PutRequest {
        PutRequest {
            agent: agent.into(),
            project: project.into(),
            signing_key: key_hex(key),
            encryption_key: None,
            instances: vec![],
            replace: false,
        }
    }

    #[test]
    fn names_are_plain_or_refused() {
        for ok in ["fx-a", "claude-termlink", "a", "010-termlink", "a.b_c-d"] {
            assert!(valid_name(ok), "{ok}");
        }
        for bad in ["", "../../tmp/x", "a..b", "A", "-a", ".a", "a/b", "a b", &"a".repeat(65)] {
            assert!(!valid_name(bad), "{bad:?}");
        }
    }

    #[test]
    fn mode_defaults_off_and_a_bad_setting_enforces() {
        let d = tempfile::tempdir().unwrap();
        assert_eq!(read_mode(d.path()), Ok(Mode::Off));
        write_mode(d.path(), Mode::Log).unwrap();
        assert_eq!(read_mode(d.path()), Ok(Mode::Log));
        std::fs::write(d.path().join(MODE_FILE), "maybe\n").unwrap();
        assert!(read_mode(d.path()).is_err());
        assert_eq!(effective_mode(d.path()), Mode::Enforce, "an invalid setting fails closed");
    }

    #[test]
    fn e169_fcw135_a_poster_without_a_card_is_refused() {
        let d = tempfile::tempdir().unwrap();
        match check_post(d.path(), "inbox:h/fx-b", "sidecar.msg.v1", &msg_payload("fx-a"), &[1; 32]) {
            Check::Refuse { code, fcw, agent, .. } => {
                assert_eq!((code, fcw, agent.as_str()), (error_code::CHANNEL_NO_KEY_CARD, "FCW-135", "fx-a"));
            }
            o => panic!("{o:?}"),
        }
    }

    #[test]
    fn e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes() {
        let d = tempfile::tempdir().unwrap();
        put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
        put(d.path(), &put_req("fx-b", "p", 2), 1, |_, _| Ok(())).unwrap();
        // fx-b's key claiming to be fx-a
        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[2; 32]) {
            Check::Refuse { code, fcw, .. } => assert_eq!((code, fcw), (error_code::CHANNEL_KEY_CARD_MISMATCH, "FCW-133")),
            o => panic!("{o:?}"),
        }
        assert_eq!(
            check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[1; 32]),
            Check::Pass { agent: "fx-a".into() }
        );
    }

    #[test]
    fn r75e_another_projects_agent_cannot_post_as_ps_agent() {
        let d = tempfile::tempdir().unwrap();
        put(d.path(), &put_req("p-agent", "p", 1), 1, |_, _| Ok(())).unwrap();
        put(d.path(), &put_req("q-agent", "q", 2), 1, |_, _| Ok(())).unwrap();
        // q's session, signing with its own key, asserting it is p's agent
        assert!(matches!(
            check_post(d.path(), "inbox:h/x", "sidecar.stage.v1", &msg_payload("p-agent"), &[2; 32]),
            Check::Refuse { fcw: "FCW-133", .. }
        ));
    }

    #[test]
    fn a_traversal_name_is_refused_and_never_read_as_a_path() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(d.path().join("x.json"), "{}").unwrap();
        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("../x"), &[1; 32]) {
            Check::Refuse { fcw: "FCW-135", agent, .. } => assert_eq!(agent, "?"),
            o => panic!("{o:?}"),
        }
    }

    #[test]
    fn other_types_and_topics_are_never_gated() {
        let d = tempfile::tempdir().unwrap();
        for (topic, ty) in [
            ("inbox:h/x", "sidecar.receipt"),
            ("inbox:h/x", "chat"),
            ("liveness:h/x", "sidecar.liveness.v1"),
            ("framework:pickup", "sidecar.msg.v1"),
        ] {
            assert_eq!(check_post(d.path(), topic, ty, b"{}", &[1; 32]), Check::NotGated, "{topic} {ty}");
        }
    }

    #[test]
    fn an_unreadable_store_refuses_never_reads_as_empty() {
        let d = tempfile::tempdir().unwrap();
        std::fs::write(store_path(d.path()), "{not json").unwrap();
        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[1; 32]) {
            Check::Refuse { fcw: "FCW-135", reason, .. } => assert!(reason.contains("corrupt"), "{reason}"),
            o => panic!("{o:?}"),
        }
        assert!(put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).is_err(), "a put never overwrites a corrupt store");
    }

    #[test]
    fn a_key_or_project_change_needs_replace_and_seq_rises() {
        let d = tempfile::tempdir().unwrap();
        let (c1, old) = put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
        assert_eq!((c1.seq, old), (1, None));
        // same key and project: an idempotent re-put moves seq, no replace needed
        let (c2, _) = put(d.path(), &put_req("fx-a", "p", 1), 2, |_, _| Ok(())).unwrap();
        assert_eq!(c2.seq, 2);
        let e = put(d.path(), &put_req("fx-a", "p", 9), 3, |_, _| Ok(())).unwrap_err();
        assert!(e.contains("replace: true"), "{e}");
        let e = put(d.path(), &put_req("fx-a", "q", 1), 3, |_, _| Ok(())).unwrap_err();
        assert!(e.contains("replace: true"), "{e}");
        let mut r = put_req("fx-a", "p", 9);
        r.replace = true;
        let (c3, old) = put(d.path(), &r, 4, |_, _| Ok(())).unwrap();
        assert_eq!(c3.seq, 3);
        assert_eq!(old.unwrap().signing_key, key_hex(1));
    }

    #[test]
    fn the_store_is_0600_and_keys_are_validated() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
        assert_eq!(std::fs::metadata(store_path(d.path())).unwrap().permissions().mode() & 0o777, 0o600);
        let mut r = put_req("fx-b", "p", 1);
        r.signing_key = "ed25519:".to_string() + &"01".repeat(32);
        assert!(put(d.path(), &r, 1, |_, _| Ok(())).is_err(), "the fingerprint form is not a card key");
        r.signing_key = key_hex(1).to_uppercase();
        assert!(put(d.path(), &r, 1, |_, _| Ok(())).is_err());
    }

    #[test]
    fn al33_startup_reconcile_flags_a_card_the_admin_log_does_not_show() {
        let d = tempfile::tempdir().unwrap();
        let log = d.path().join("rpc-admin.jsonl");
        let (a, _) = put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
        put(d.path(), &put_req("fx-b", "p", 2), 1, |_, _| Ok(())).unwrap();
        // fx-a's put was logged; fx-b's was not (crash between persist and log, or an outside write)
        std::fs::write(&log, format!("{{\"event\":\"key-card-put\",\"agent\":\"fx-a\",\"card_digest\":\"{}\"}}\n", card_digest(&a))).unwrap();
        let m = startup_reconcile(d.path(), &log);
        assert_eq!(m.len(), 1, "{m:?}");
        assert!(m[0].contains("fx-b") && m[0].contains("no logged"), "{m:?}");
        std::fs::write(store_path(d.path()), "{x").unwrap();
        assert!(startup_reconcile(d.path(), &log).iter().any(|s| s.contains("corrupt")), "an unreadable store is reported");
    }

    #[test]
    fn r2_an_outside_edit_with_the_same_seq_is_refused_until_restart() {
        let d = tempfile::tempdir().unwrap();
        put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
        // a root agent swaps fx-a's key in the file, seq unchanged
        let text = std::fs::read_to_string(store_path(d.path())).unwrap().replace(&"01".repeat(32), &"07".repeat(32));
        std::fs::write(store_path(d.path()), text).unwrap();
        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[7; 32]) {
            Check::Refuse { fcw: "FCW-135", reason, .. } => assert!(reason.contains("changed outside"), "{reason}"),
            o => panic!("the edited key must not pass: {o:?}"),
        }
        assert!(put(d.path(), &put_req("fx-b", "p", 2), 2, |_, _| Ok(())).is_err(), "no put over an outside edit");
        // a restart re-baselines and reports it
        let log = d.path().join("rpc-admin.jsonl");
        assert!(!startup_reconcile(d.path(), &log).is_empty());
        assert!(matches!(check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[7; 32]), Check::Pass { .. }));
    }

    #[test]
    fn r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing() {
        let d = tempfile::tempdir().unwrap();
        let e = put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Err("admin log unwritable".into())).unwrap_err();
        assert!(e.contains("unwritable"), "{e}");
        assert!(!store_path(d.path()).exists(), "nothing persisted when the row fails");
        let mut seen = None;
        put(d.path(), &put_req("fx-a", "p", 1), 1, |new, old| {
            assert!(!store_path(d.path()).exists(), "the row is written before the store");
            seen = Some((new.seq, old.is_none()));
            Ok(())
        })
        .unwrap();
        assert_eq!(seen, Some((1, true)));
    }

    #[test]
    fn r2_every_field_change_needs_replace_and_a_key_has_one_card() {
        let d = tempfile::tempdir().unwrap();
        let mut r = put_req("fx-a", "p", 1);
        r.encryption_key = Some(format!("{ENCRYPTION_PREFIX}{}", "02".repeat(32)));
        put(d.path(), &r, 1, |_, _| Ok(())).unwrap();
        let mut swap = r.clone();
        swap.encryption_key = Some(format!("{ENCRYPTION_PREFIX}{}", "03".repeat(32)));
        assert!(put(d.path(), &swap, 2, |_, _| Ok(())).unwrap_err().contains("replace: true"));
        let mut drop_enc = r.clone();
        drop_enc.encryption_key = None;
        assert!(put(d.path(), &drop_enc, 2, |_, _| Ok(())).unwrap_err().contains("replace: true"));
        let mut inst = r.clone();
        inst.instances = vec!["i1".into()];
        assert!(put(d.path(), &inst, 2, |_, _| Ok(())).unwrap_err().contains("replace: true"));
        // fx-a's key on fx-b's card is refused
        assert!(put(d.path(), &put_req("fx-b", "p", 1), 2, |_, _| Ok(())).unwrap_err().contains("one agent"));
    }

    #[test]
    fn r2_mode_changes_are_logged_first_and_reconciled_at_start() {
        let d = tempfile::tempdir().unwrap();
        let log = d.path().join("rpc-admin.jsonl");
        assert!(set_mode(d.path(), Mode::Enforce, |_, _| Err("no row".into())).is_err());
        assert_eq!(read_mode(d.path()), Ok(Mode::Off), "a failed row changes nothing");
        assert_eq!(set_mode(d.path(), Mode::Enforce, |_, _| Ok(())).unwrap(), "off");
        std::fs::write(&log, "{\"event\":\"key-card-mode\",\"to\":\"enforce\"}\n").unwrap();
        assert!(startup_reconcile(d.path(), &log).is_empty());
        std::fs::remove_file(d.path().join(MODE_FILE)).unwrap(); // the file deleted outside the RPC
        let m = startup_reconcile(d.path(), &log);
        assert!(m.iter().any(|s| s.contains("setting is off") && s.contains("enforce")), "{m:?}");
    }

    #[test]
    fn r2_refusals_for_uncarded_names_share_one_bucket_and_rows_are_capped() {
        note_refusal("t-r2-x1", "fpz", true, false);
        note_refusal("t-r2-x2", "fpz", true, false);
        note_refusal("t-r2-carded", "fpz", true, true);
        let c = refusal_counts();
        assert!(c.get("t-r2-x1").is_none() && c.get("t-r2-x2").is_none(), "{c}");
        assert!(c[UNCARDED]["refused"].as_u64().unwrap() >= 2);
        assert_eq!(c["t-r2-carded"]["signers"]["fpz"], 1);
        let _s = MINUTE_STATE.lock().unwrap_or_else(|p| p.into_inner());
        let base = 600_000_000u64;
        let n = (0..200).filter(|i| first_in_minute(&format!("cap-{i}"), "fp", base, true)).count();
        assert_eq!(n, MAX_ROWS_PER_MINUTE);
        // r3 MAJOR: 60 made-up (uncarded) names in a minute do not suppress a carded refusal row
        let m = base + 60_000;
        let uncarded = (0..60).filter(|i| first_in_minute(&format!("x{i}"), "k1", m, false)).count();
        assert_eq!(uncarded, 1, "uncarded refusals coalesce to one row per signer");
        assert!(first_in_minute("fx-a", "k2", m, true), "the real impersonation is still logged");
    }

    #[test]
    fn refusal_rows_are_coalesced_per_agent_key_minute() {
        let _s = MINUTE_STATE.lock().unwrap_or_else(|p| p.into_inner());
        assert!(first_in_minute("t-agent", "fp1", 120_000, true));
        assert!(!first_in_minute("t-agent", "fp1", 120_500, true));
        assert!(first_in_minute("t-agent", "fp2", 120_600, true));
        assert!(first_in_minute("t-agent", "fp1", 180_000, true), "a new minute logs again");
    }
}
