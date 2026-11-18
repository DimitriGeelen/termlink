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
    atomic_write(&dir.join(MODE_FILE), format!("{}\n", mode.as_str()).as_bytes(), 0o644)
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

/// Absent store = no cards. A store that exists but cannot be read or parsed is an `Err`, never
/// "no cards" (design v2: an unreadable store refuses every gated post).
fn load(dir: &Path) -> Result<Store, String> {
    let p = store_path(dir);
    let text = match std::fs::read_to_string(&p) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
            return Ok(Store { schema: STORE_SCHEMA.into(), cards: BTreeMap::new() });
        }
        Err(e) => return Err(format!("key-card store unreadable: {e}")),
    };
    let s: Store = serde_json::from_str(&text).map_err(|e| format!("key-card store corrupt: {e}"))?;
    if s.schema != STORE_SCHEMA {
        return Err(format!("key-card store has schema {:?}, not {STORE_SCHEMA}", s.schema));
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
    atomic_write(&store_path(dir), text.as_bytes(), 0o600)
}

/// The outcome of the post-time check.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Check {
    /// Not an arc-011 type on `inbox:*`: the check does not apply.
    NotGated,
    Pass { agent: String },
    Refuse { code: i64, fcw: &'static str, agent: String, reason: String },
}

fn refuse_no_card(agent: &str, reason: String) -> Check {
    Check::Refuse { code: error_code::CHANNEL_NO_KEY_CARD, fcw: "FCW-135", agent: agent.to_string(), reason }
}

/// The claimed agent of an arc-011 payload: its `from_agent` (msg and stage records both carry it).
pub fn claimed_agent(payload: &[u8]) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(payload).ok()?;
    v.get("from_agent")?.as_str().map(str::to_string)
}

/// The card check for one post whose transport signature has already verified with `signer`.
pub fn check_post(dir: &Path, topic: &str, msg_type: &str, payload: &[u8], signer: &[u8; 32]) -> Check {
    if !topic.starts_with("inbox:") || !GATED_TYPES.contains(&msg_type) {
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

/// Write or replace one card. Returns the new card and the one it replaced. A change of an
/// existing card's signing key or project needs `replace: true`, so it is never silent.
pub fn put(dir: &Path, req: &PutRequest, now_ms: u64) -> Result<(Card, Option<Card>), String> {
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
        && (o.signing_key != req.signing_key || o.project != req.project)
        && !req.replace
    {
        return Err(format!(
            "agent {:?} already has a card (project {:?}, key {}); changing its key or project needs replace: true",
            req.agent, o.project, o.signing_key
        ));
    }
    let card = Card {
        schema: CARD_SCHEMA.into(),
        agent: req.agent.clone(),
        project: req.project.clone(),
        signing_key: req.signing_key.clone(),
        encryption_key: req.encryption_key.clone(),
        instances: req.instances.clone(),
        seq: old.as_ref().map_or(1, |o| o.seq + 1),
        by: "runme".into(),
        at_ms: now_ms,
    };
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

/// Refusals per claimed agent since hub start (the impersonated agent's sidecar reads these,
/// design v2 C-12), and would-be refusals in `log` mode.
static REFUSALS: Mutex<BTreeMap<String, (u64, u64)>> = Mutex::new(BTreeMap::new());

/// Count one refusal (`enforced`) or one log-mode would-refuse for `agent`.
pub fn note_refusal(agent: &str, enforced: bool) {
    let mut m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
    let e = m.entry(agent.to_string()).or_insert((0, 0));
    if enforced { e.0 += 1 } else { e.1 += 1 }
}

/// `{agent: {refused, would_refuse}}`.
pub fn refusal_counts() -> serde_json::Value {
    let m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
    serde_json::Value::Object(
        m.iter()
            .map(|(a, (r, w))| (a.clone(), serde_json::json!({"refused": r, "would_refuse": w})))
            .collect(),
    )
}

/// Admin rows for refusals are coalesced per (claimed agent, poster key, minute) so a token
/// holder cannot flood the admin log (design v2, C-12). True = write a row for this one.
static LOGGED: Mutex<(u64, Option<HashSet<(String, String)>>)> = Mutex::new((0, None));

pub fn first_in_minute(agent: &str, signer_fp: &str, now_ms: u64) -> bool {
    let minute = now_ms / 60_000;
    let mut g = LOGGED.lock().unwrap_or_else(|p| p.into_inner());
    if g.0 != minute || g.1.is_none() {
        *g = (minute, Some(HashSet::new()));
    }
    g.1.as_mut().map(|s| s.insert((agent.to_string(), signer_fp.to_string()))).unwrap_or(true)
}

#[cfg(test)]
mod tests {
    use super::*;

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
        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
        put(d.path(), &put_req("fx-b", "p", 2), 1).unwrap();
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
        put(d.path(), &put_req("p-agent", "p", 1), 1).unwrap();
        put(d.path(), &put_req("q-agent", "q", 2), 1).unwrap();
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
        assert!(put(d.path(), &put_req("fx-a", "p", 1), 1).is_err(), "a put never overwrites a corrupt store");
    }

    #[test]
    fn a_key_or_project_change_needs_replace_and_seq_rises() {
        let d = tempfile::tempdir().unwrap();
        let (c1, old) = put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
        assert_eq!((c1.seq, old), (1, None));
        // same key and project: an idempotent re-put moves seq, no replace needed
        let (c2, _) = put(d.path(), &put_req("fx-a", "p", 1), 2).unwrap();
        assert_eq!(c2.seq, 2);
        let e = put(d.path(), &put_req("fx-a", "p", 9), 3).unwrap_err();
        assert!(e.contains("replace: true"), "{e}");
        let e = put(d.path(), &put_req("fx-a", "q", 1), 3).unwrap_err();
        assert!(e.contains("replace: true"), "{e}");
        let mut r = put_req("fx-a", "p", 9);
        r.replace = true;
        let (c3, old) = put(d.path(), &r, 4).unwrap();
        assert_eq!(c3.seq, 3);
        assert_eq!(old.unwrap().signing_key, key_hex(1));
    }

    #[test]
    fn the_store_is_0600_and_keys_are_validated() {
        use std::os::unix::fs::PermissionsExt;
        let d = tempfile::tempdir().unwrap();
        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
        assert_eq!(std::fs::metadata(store_path(d.path())).unwrap().permissions().mode() & 0o777, 0o600);
        let mut r = put_req("fx-b", "p", 1);
        r.signing_key = "ed25519:".to_string() + &"01".repeat(32);
        assert!(put(d.path(), &r, 1).is_err(), "the fingerprint form is not a card key");
        r.signing_key = key_hex(1).to_uppercase();
        assert!(put(d.path(), &r, 1).is_err());
    }

    #[test]
    fn refusal_rows_are_coalesced_per_agent_key_minute() {
        assert!(first_in_minute("t-agent", "fp1", 120_000));
        assert!(!first_in_minute("t-agent", "fp1", 120_500));
        assert!(first_in_minute("t-agent", "fp2", 120_600));
        assert!(first_in_minute("t-agent", "fp1", 180_000), "a new minute logs again");
    }
}
