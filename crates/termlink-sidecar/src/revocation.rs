//! T-3703 (arc-011 P-15, B-5; AL-22): the revocation pull, its verification and the stale rule.
//!
//! Rules (06a 2.6; 06 4.7; design note v2 `docs/reports/T-3703-design.md` section 5, r1 dispositions):
//! 1. At the start of every receive pass (before any message is handled) the hub's head is read and verified;
//!    the list is fetched only when the head moved (design 5.4). PN-3's 30 s tick is met a fortiori.
//! 2. Verified means one of the two cases of 06 4.7: the head verifies under the pinned key and equals the held
//!    list, or a higher head whose list verifies, hashes to the head's digest, has the head's seq, chains when
//!    seq = held + 1, and names this hub. **Everything else is a failed pull**, never an empty list: a transport
//!    error, a hub refusal (REVOCATION_RECONCILE included), `method not found` (status `hub predates revocation
//!    list`, design 5.10), a bad signature, an unpinned key, a lower seq (`ROLLBACK refused, kept seq N`), a fork.
//! 3. The hub's public key comes from `hub.version` (`signing_public_key`, design 5.1) and is used only when
//!    `ed25519:` + sha256(key) equals the stored pin (`hub_key_pin::verified_hub_key`).
//! 4. Keys on the held list stay refused whatever happens later (lists only grow); the held list is re-verified
//!    on load (r1 M1): a stored list that fails is tamper, its keys stay refused, its seq stays the floor.
//! 5. A started sidecar is UNVERIFIED_SINCE_START until its first verified pull (FCW-96); PN-13 without one makes
//!    it STALE (FCW-95) with one needs-attention entry. While either holds, `narrowing()` is true (DR-8 A: urgent
//!    from outside the own project's roster is downgraded; in PH-2 urgent already needs the roster, so this is
//!    the gate any later widening must pass).
//! 6. Every list with new entries raises ONE coalesced `key-revoked` entry (design 5.9); STORED mail from a newly
//!    revoked key that is not handed over is REFUSED at once (FCW-200), so no ring names it afterwards.
//! 7. The switch `revocation-pull` (state `off`) returns to PH-1 trust; a flip raises attention (design 5.10).

use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{Duration, Instant};
use termlink_session::signed_state::{self as ss, J};

use crate::store::{LineExtra, Store};

pub const SCHEMA: &str = "termlink.revocation/1";
pub const HEAD_SCHEMA: &str = "termlink.revocation-head/1";
/// PN-13: the stale limit, equal to PN-1 (1 hour).
pub const PN13: Duration = Duration::from_secs(3600);
/// Chain reason (and notice) for a message refused because its signer is revoked (FCW-99, FCW-200).
pub const REVOKED: &str = "REVOKED";
pub const REASON_REVOKED: &str = "revoked";

const LIST_STATE: &str = "revocation-list";
const HUB_PK_STATE: &str = "revocation-hub-pk";
const ATTENTION_STATE: &str = "revocation-attention-seq";
const STATUS_STATE: &str = "revocation-status";
const SWITCH_STATE: &str = "revocation-pull";
const SWITCH_SEEN_STATE: &str = "revocation-pull-seen";
const REFUSED_COUNT_STATE: &str = "revocation-refused-count";
const RAISED_KEYS_STATE: &str = "revocation-attention-keys";

/// What every other part of the sidecar reads: the refused keys and whether the stale rule narrows.
#[derive(Debug, Clone, Default)]
struct Snap {
    keys: BTreeSet<String>,
    narrowing: bool,
}

static SNAPS: Mutex<BTreeMap<PathBuf, Snap>> = Mutex::new(BTreeMap::new());

fn publish(store: &Store, keys: &BTreeSet<String>, narrowing: bool) {
    SNAPS
        .lock()
        .unwrap_or_else(|p| p.into_inner())
        .insert(store.dir().to_path_buf(), Snap { keys: keys.clone(), narrowing });
}

/// The held keys: the published snapshot, or, before any tracker has published for this store (another process,
/// the window before `Tracker::new`), the keys of the stored list read without verification: fail closed, never
/// "nothing revoked" (r2 CC 9).
fn held_keys(store: &Store) -> BTreeSet<String> {
    if let Some(s) = SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()) {
        return s.keys.clone();
    }
    // T-3823 (T-3703 r3 OC 4): parse the stored list once per (len, mtime), not on every lookup
    let path = store.dir().join(format!("state.{LIST_STATE}"));
    let stamp = std::fs::metadata(&path).ok().and_then(|m| {
        let t = m.modified().ok()?.duration_since(std::time::UNIX_EPOCH).ok()?.as_nanos();
        Some((m.len(), t))
    });
    let Some(stamp) = stamp else { return BTreeSet::new() };
    let mut cache = FALLBACK.lock().unwrap_or_else(|p| p.into_inner());
    if let Some((st, keys)) = cache.get(store.dir())
        && *st == stamp
    {
        return keys.clone();
    }
    let keys = store
        .read_state(LIST_STATE)
        .ok()
        .flatten()
        .and_then(|t| ss::parse_strict(t.as_bytes()).ok())
        .map(|l| keys_of(&entries_of(&l)))
        .unwrap_or_default();
    cache.insert(store.dir().to_path_buf(), (stamp, keys.clone()));
    keys
}

/// The no-snapshot fallback's parsed keys per store dir, keyed by the stored list's (len, mtime ns).
#[allow(clippy::type_complexity)]
static FALLBACK: Mutex<BTreeMap<PathBuf, ((u64, u128), BTreeSet<String>)>> = Mutex::new(BTreeMap::new());

/// FCW-99: the full fingerprint `ed25519:<64 hex>` of a signer is on the held list.
pub fn revoked(store: &Store, fingerprint: &str) -> bool {
    held_keys(store).contains(fingerprint)
}

// ---- T-3834 (RK-1): the sender sees that its recipient's key is revoked ---------------------------------
/// `<msg_id> <recipient agent>` per sent message, newest kept. Outside the chain: chain lines are hashed.
pub const SENT_TO_STATE: &str = "sent-recipients";
/// `<agent> <fingerprint>` per key card on the hub, as of the last pull pass that read the card list.
pub const RECIPIENT_KEYS_STATE: &str = "recipient-keys";
/// The sender-side reason shown on a message whose recipient's current key is held as revoked.
pub const RECIPIENT_REVOKED_REASON: &str = "key revoked, operator action needed";
/// The attention kind raised once per (recipient, key).
pub const RECIPIENT_REVOKED_KIND: &str = "recipient-key-revoked";
const SENT_TO_KEEP: usize = 4096;

/// Record who a sent message is addressed to (called right after its SENT line).
pub fn record_sent_to(store: &Store, msg_id: &str, to: &str) {
    if let Err(e) = store.append_state_line(SENT_TO_STATE, &format!("{msg_id} {to}"), SENT_TO_KEEP) {
        eprintln!("termlink-sidecar: could not record the recipient of {msg_id} ({e}); its status cannot name a revoked recipient key");
    }
}

/// msg_id -> recipient agent, for every recorded send.
pub fn sent_to(store: &Store) -> BTreeMap<String, String> {
    store.read_state(SENT_TO_STATE).ok().flatten().unwrap_or_default().lines()
        .filter_map(|l| l.split_once(' ').map(|(m, a)| (m.to_string(), a.to_string())))
        .collect()
}

/// agent -> fingerprint of its current card key.
pub fn recipient_keys(store: &Store) -> BTreeMap<String, String> {
    store.read_state(RECIPIENT_KEYS_STATE).ok().flatten().unwrap_or_default().lines()
        .filter_map(|l| l.split_once(' ').map(|(a, f)| (a.to_string(), f.to_string())))
        .collect()
}

/// Store the cards' `(agent, fingerprint)` pairs, only when they changed.
pub fn store_recipient_keys(store: &Store, keys: &BTreeMap<String, String>) {
    if &recipient_keys(store) == keys {
        return;
    }
    let text: String = keys.iter().map(|(a, f)| format!("{a} {f}\n")).collect();
    let _ = store.write_state(RECIPIENT_KEYS_STATE, &text);
}

/// The fingerprint of `agent`'s current key when that key is held as revoked here, else None.
pub fn recipient_key_revoked(keys: &BTreeMap<String, String>, store: &Store, agent: &str) -> Option<String> {
    keys.get(agent).filter(|fp| revoked(store, fp)).cloned()
}

/// One coalesced needs-attention entry per (recipient, revoked key) this sidecar has sent mail to.
pub fn raise_recipients_revoked(store: &Store) {
    let keys = recipient_keys(store);
    let recipients: BTreeSet<String> = sent_to(store).into_values().collect();
    for agent in recipients {
        if let Some(fp) = recipient_key_revoked(&keys, store, &agent) {
            let detail = format!(
                "mail to {agent} waits: its current key {fp} is on the revocation list, so it cannot be delivered until the \
                 operator issues {agent} a new key (`termlink hub key-card put`) or lifts the revocation"
            );
            let _ = store.attention_coalesced(RECIPIENT_REVOKED_KIND, &format!("{agent}:{fp}"), &detail);
        }
    }
}

/// The short sender id of a stored message (`from`, the first 16 hex of sha256(key)) matches a held key. Used
/// where only the stored message is at hand (the hand-over re-check); a prefix match can only over-refuse.
pub fn revoked_from(store: &Store, from: &str) -> bool {
    if from.len() < 16 {
        return false;
    }
    held_keys(store).iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from)))
}

/// T-3708 (E-127's reader): when this sidecar first held each key as revoked (its own time), one
/// `<key> <ms>` line per key. A message stored before then was stored lawfully (the list was not known
/// here yet); one stored after is a broken invariant.
const HELD_SINCE_STATE: &str = "revocation-held-since";

/// Add the keys not recorded yet, at `now`.
pub fn record_held_since(store: &Store, keys: &BTreeSet<String>, now: u64) {
    let mut m = held_since(store);
    let before = m.len();
    for k in keys {
        if !m.contains_key(k) {
            crate::fields::register(now);
            m.insert(k.clone(), now);
        }
    }
    if m.len() != before {
        let text: String = m.iter().map(|(k, t)| format!("{k} {t}\n")).collect();
        if let Err(e) = store.write_state(HELD_SINCE_STATE, &text) {
            // r2 MINOR 8: never fail open: E-127 then says its invariant (b) cannot be checked
            HELD_SINCE_UNWRITTEN.store(true, std::sync::atomic::Ordering::SeqCst);
            eprintln!("termlink-sidecar: revocation held-since record not written: {e}");
        } else {
            HELD_SINCE_UNWRITTEN.store(false, std::sync::atomic::Ordering::SeqCst);
        }
    }
}

static HELD_SINCE_UNWRITTEN: std::sync::atomic::AtomicBool = std::sync::atomic::AtomicBool::new(false);

/// Whether the last held-since write failed (E-127's invariant (b) then cannot be checked).
pub fn held_since_unwritten() -> bool {
    HELD_SINCE_UNWRITTEN.load(std::sync::atomic::Ordering::SeqCst)
}

/// Key -> the time this sidecar first held it as revoked.
pub fn held_since(store: &Store) -> std::collections::BTreeMap<String, u64> {
    store
        .read_state(HELD_SINCE_STATE)
        .ok()
        .flatten()
        .unwrap_or_default()
        .lines()
        .filter_map(|l| {
            let (k, t) = l.split_once(' ')?;
            Some((k.to_string(), t.trim().parse().ok()?))
        })
        .collect()
}

/// E-127's reader (T-3703 r1 finding 17, carried into T-3708): the invariants that must never break.
/// (a) the list unverified or stale for longer than PN-13 while the narrowing is not applied;
/// (b) a message from a revoked key STORED after this sidecar held that key as revoked.
/// `stored` is (msg_id, short sender id, stored at). Returns one line per broken invariant.
pub fn e127_broken(
    status_line: &str,
    narrowing_now: bool,
    held: &std::collections::BTreeMap<String, u64>,
    stored: &[(String, String, u64)],
    now: u64,
    age: &dyn Fn(u64) -> u64,
) -> Vec<String> {
    let mut out = vec![];
    let since = ["unverified since ", "stale since "]
        .iter()
        .find_map(|p| status_line.strip_prefix(p))
        .and_then(|r| r.split_whitespace().next())
        .and_then(|n| n.parse::<u64>().ok());
    if let Some(t) = since
        && now.saturating_sub(t) > PN13.as_millis() as u64
        && !narrowing_now
    {
        out.push(format!("revocation list unverified since {t}, longer than PN-13, and the narrowing is not applied"));
    }
    for (id, from, at) in stored {
        if from.len() < 16 {
            continue;
        }
        // r3 MINOR 4: "stored after it was held" compares jump-corrected ages, never two raw wall stamps
        let hit = held.iter().find(|(k, t)| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from.as_str())) && age(*at) < age(**t));
        if let Some((k, t)) = hit {
            out.push(format!("message {id} from revoked key {k} was STORED at {at}, after this sidecar held it as revoked at {t}"));
        }
    }
    out
}

/// DR-8 A: true while the list is STALE or not yet verified since start.
pub fn narrowing(store: &Store) -> bool {
    SNAPS.lock().unwrap_or_else(|p| p.into_inner()).get(store.dir()).is_none_or(|s| s.narrowing)
}

/// Status (OBS-41): the line the pull last wrote, and the count of messages refused as revoked.
pub fn status(store: &Store) -> Value {
    let line = store.read_state(STATUS_STATE).ok().flatten().unwrap_or_else(|| "unverified since start (no pull yet)".into());
    let n: u64 = store.read_state(REFUSED_COUNT_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    serde_json::json!({"revocation": line.trim(), "refused_revoked": n})
}

/// Count one refusal of a revoked key (status `refused_revoked`, for E-127's reader).
pub fn count_refusal(store: &Store) {
    let n: u64 = store.read_state(REFUSED_COUNT_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
    let _ = store.write_state(REFUSED_COUNT_STATE, &(n + 1).to_string());
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum State {
    UnverifiedSinceStart,
    Verified,
    Stale,
    Off,
}

#[derive(Debug, Clone)]
struct Held {
    seq: i64,
    digest: String,
    keys: BTreeSet<String>,
    entries: Vec<J>,
}

fn keys_of(entries: &[J]) -> BTreeSet<String> {
    entries.iter().filter_map(|e| e.get("key").and_then(J::as_str).map(str::to_string)).collect()
}

fn entries_of(l: &J) -> Vec<J> {
    match l.get("entries") {
        Some(J::Arr(a)) => a.clone(),
        _ => vec![],
    }
}

/// The per-hub pull state machine (one per receiver).
pub struct Tracker {
    hub_id: String,
    stale_after: Duration,
    started: Instant,
    last_verified: Option<Instant>,
    state: State,
    held: Option<Held>,
    stale_raised: bool,
    /// Wall-clock ms of the last verified pull (or of the start): the stable `since T` (r2 CC 11).
    since_wall: u64,
    /// The last `verified` status write: (seq, when), so an unchanged list is re-stamped at most once a minute.
    status_written: Option<(i64, Instant)>,
    /// The held list failed its load check: its keys are refused until the first verified fetch replaces them,
    /// and no floor, chain or subset check is made against it (r2 CC 7).
    held_unverified: bool,
}

/// One RPC to the home hub: Ok(result) or Err((code, message)). Code 0 = transport.
pub type Rpc<'a> = &'a dyn Fn(&str) -> Result<Value, (i64, String)>;

impl Tracker {
    /// On start: the last verified list from the store, re-verified (r1 M1); state UNVERIFIED_SINCE_START.
    pub fn new(store: &Store, hub_id: &str, stale_after: Duration) -> Tracker {
        let mut t = Tracker {
            hub_id: hub_id.to_string(),
            stale_after,
            started: Instant::now(),
            last_verified: None,
            state: State::UnverifiedSinceStart,
            held: None,
            stale_raised: false,
            since_wall: crate::store::now_ms(),
            status_written: None,
            held_unverified: false,
        };
        if let Ok(Some(text)) = store.read_state(LIST_STATE) {
            let pk = store.read_state(HUB_PK_STATE).ok().flatten().and_then(|h| ss::verifying_key_of_hex(h.trim()));
            let pin_ok = pk.as_ref().is_some_and(|k| {
                store.read_state(&crate::hub_key_pin::pin_state_name(hub_id)).ok().flatten().map(|p| p.trim().to_string())
                    == Some(ss::key_fingerprint(k))
            });
            match (pk, pin_ok) {
                (Some(k), true) => match ss::parse_and_verify(text.as_bytes(), &k, SCHEMA) {
                    Ok(l) => t.held = Some(held_of(&l)),
                    Err(e) => t.tamper_load(store, &text, &format!("stored list does not verify: {e}")),
                },
                _ => t.tamper_load(store, &text, "stored list cannot be checked against the pinned hub key"),
            }
        }
        t.publish(store);
        t.write_status(store, "unverified since start (no pull yet)");
        if !t.held_unverified {
            t.raise_new_entries(store); // a crash between install and raise (design 5.9)
        }
        t
    }

    /// A stored list that fails verification: tamper; its keys stay refused and its seq stays the floor.
    fn tamper_load(&mut self, store: &Store, text: &str, why: &str) {
        let hour = crate::store::now_ms() / 3_600_000;
        let _ = store.attention_coalesced("revocation-tamper", &hour.to_string(), &format!("revocation list in the store: {why}"));
        if let Ok(l) = ss::parse_strict(text.as_bytes()) {
            let mut h = held_of(&l);
            h.digest = String::new(); // never matches a head: the next pull fetches the full list
            self.held = Some(h);
            self.held_unverified = true;
        }
    }

    /// The hub could not be asked at all this pass (`hub.version` failed, or names another hub): a failed pull
    /// for the stale rule (r2 CC 1).
    pub fn outage(&mut self, store: &Store, now: Instant, why: &str) {
        if self.state == State::Off {
            return;
        }
        self.failed(store, now, why, false);
    }

    pub fn set_stale_after(&mut self, d: Duration) {
        self.stale_after = d;
    }

    pub fn state(&self) -> &State {
        &self.state
    }

    pub fn held_seq(&self) -> Option<i64> {
        self.held.as_ref().map(|h| h.seq)
    }

    fn narrowing(&self) -> bool {
        matches!(self.state, State::Stale | State::UnverifiedSinceStart)
    }

    fn publish(&self, store: &Store) {
        let keys = self.held.as_ref().map(|h| h.keys.clone()).unwrap_or_default();
        publish(store, &keys, self.narrowing());
    }

    fn write_status(&self, store: &Store, line: &str) {
        if store.read_state(STATUS_STATE).ok().flatten().as_deref().map(str::trim) != Some(line) {
            let _ = store.write_state(STATUS_STATE, line);
        }
    }

    fn since(&self) -> Instant {
        self.last_verified.unwrap_or(self.started)
    }

    /// One pull. `version` is this pass's `hub.version` answer (already hub-id checked).
    pub fn tick(&mut self, store: &Store, version: &Value, rpc: Rpc<'_>, now: Instant, sender: Option<&crate::send::Sender>) {
        // the switch (design 5.10): off = PH-1 trust, said so; a flip either way raises attention
        let off = store.read_state(SWITCH_STATE).ok().flatten().is_some_and(|s| s.trim() == "off");
        let seen = store.read_state(SWITCH_SEEN_STATE).ok().flatten().map(|s| s.trim().to_string());
        let now_s = if off { "off" } else { "on" };
        if seen.as_deref() != Some(now_s) {
            if seen.is_some() || off {
                let _ = store.attention(
                    "revocation-pull-switched",
                    &format!("revocation pull switched {now_s}{}", if off { ": PH-1 trust, revoked keys are no longer refused" } else { "" }),
                );
            }
            let _ = store.write_state(SWITCH_SEEN_STATE, now_s);
        }
        if off {
            // T-3708 (FCW-120): "off (PH-1 trust)" is this pass's evaluation too; unstamped it would read unknown
            crate::fields::touch(crate::fields::REVOCATION);
            self.state = State::Off;
            publish(store, &BTreeSet::new(), false);
            self.write_status(store, "off (PH-1 trust)");
            return;
        }
        if self.state == State::Off {
            self.state = State::UnverifiedSinceStart;
            self.last_verified = None;
            self.started = now;
        }
        match self.pull(store, version, rpc) {
            Ok(()) => self.verified(store, now, sender),
            Err((why, tamper)) => self.failed(store, now, &why, tamper),
        }
    }

    fn pull(&mut self, store: &Store, version: &Value, rpc: Rpc<'_>) -> Result<(), (String, bool)> {
        let pk_hex = version.get("signing_public_key").and_then(Value::as_str);
        let pinned = crate::hub_key_pin::verified_hub_key(store, &self.hub_id, version);
        let pin_stored = store.read_state(&crate::hub_key_pin::pin_state_name(&self.hub_id)).ok().flatten().is_some();
        let pk = match (pk_hex.and_then(ss::verifying_key_of_hex), pinned) {
            (Some(k), Some(fp)) if ss::key_fingerprint(&k) == fp => k,
            (Some(_), Some(_)) => return Err(("hub public key does not hash to the pinned fingerprint".into(), true)),
            // a pinned hub offering a public key that is not one is interference (r3 CC N6); a missing key is not:
            // every pre-T-3703 hub offers its pinned fingerprint without the key, so it cannot be told apart
            (None, _) if pin_stored && pk_hex.is_some() => return Err(("hub offers a malformed signing public key".into(), true)),
            // a hub older than T-3703 offers no public key: the same class as `method not found` (r2 CC 6, OC 5)
            (None, _) => return Err(("hub predates revocation list".into(), false)),
            // a pin that differs is interference; no pin yet (first contact not over a trusted path) is not
            (Some(_), None) if pin_stored => return Err(("hub signing key differs from the pin".into(), true)),
            (Some(_), None) => return Err(("hub signing key not pinned yet".into(), false)),
        };
        let h = match rpc("hub.revocation_head") {
            Ok(v) => v,
            Err((-32601, _)) => return Err(("hub predates revocation list".into(), false)),
            Err((0, e)) => return Err((format!("head: {e}"), false)),
            Err((c, e)) => return Err((format!("head: hub refused ({c}): {e}"), false)),
        };
        let text = h.get("text").and_then(Value::as_str).ok_or(("head: no text".to_string(), true))?;
        let head = ss::parse_and_verify(text.as_bytes(), &pk, HEAD_SCHEMA).map_err(|e| (format!("head-signature: {e}"), true))?;
        if head.get("hub_id").and_then(J::as_str) != Some(self.hub_id.as_str()) {
            return Err(("head names another hub".into(), true));
        }
        let hseq = head.get("seq").and_then(J::as_int).ok_or(("head without seq".to_string(), true))?;
        let hdig = head.get("digest").and_then(J::as_str).unwrap_or("").to_string();
        if self.held_unverified {
            return self.fetch(store, &pk, hseq, &hdig, rpc); // replaces an unverifiable stored list (r2 CC 7)
        }
        if let Some(held) = &self.held {
            if hseq < held.seq {
                return Err((format!("ROLLBACK refused, kept seq {}", held.seq), true));
            }
            if hseq == held.seq {
                return if hdig == held.digest {
                    Ok(())
                } else {
                    Err((format!("fork: seq {hseq} with another digest"), true))
                };
            }
        }
        self.fetch(store, &pk, hseq, &hdig, rpc)
    }

    fn fetch(&mut self, store: &Store, pk: &ss::VerifyingKey, hseq: i64, hdig: &str, rpc: Rpc<'_>) -> Result<(), (String, bool)> {
        let g = match rpc("hub.revocation_get") {
            Ok(v) => v,
            Err((c, e)) => return Err((format!("get: ({c}) {e}"), false)),
        };
        let text = g.get("text").and_then(Value::as_str).ok_or(("get: no text".to_string(), true))?;
        let l = ss::parse_and_verify(text.as_bytes(), pk, SCHEMA).map_err(|e| (format!("list: {e}"), true))?;
        let lseq = l.get("seq").and_then(J::as_int).unwrap_or(-1);
        let ldig = l.get("digest").and_then(J::as_str).unwrap_or("");
        if lseq != hseq || ldig != hdig {
            // a revocation between head and get: benign, the next pass reads both again (r2 CC 11)
            return Err(("list does not match its head (changed between head and get)".into(), false));
        }
        if l.get("hub_id").and_then(J::as_str) != Some(self.hub_id.as_str()) {
            return Err(("list names another hub".into(), true));
        }
        if let Some(held) = self.held.as_ref().filter(|_| !self.held_unverified) {
            if lseq == held.seq + 1
                && l.get("prev_digest").and_then(J::as_str) != Some(held.digest.as_str())
            {
                return Err(("list does not chain to the held one".into(), true));
            }
            // lists only grow: a list that drops a held key is tamper
            let new_keys = keys_of(&entries_of(&l));
            if !held.keys.is_subset(&new_keys) {
                return Err(("list drops a key this sidecar holds as revoked".into(), true));
            }
        }
        store.write_state(LIST_STATE, text).map_err(|e| (format!("list not stored: {e}"), false))?;
        let _ = store.write_state(HUB_PK_STATE, &ss::hex(pk.as_bytes()));
        self.held = Some(held_of(&l));
        self.held_unverified = false;
        if let Some(h) = &self.held {
            record_held_since(store, &h.keys, crate::store::now_ms());
        }
        Ok(())
    }

    fn verified(&mut self, store: &Store, now: Instant, sender: Option<&crate::send::Sender>) {
        // T-3708 (FCW-120, r1 MAJOR 4): the revocation field is fresh from this tracker's own evaluation
        crate::fields::touch(crate::fields::REVOCATION);
        self.last_verified = Some(now);
        self.since_wall = crate::store::now_ms();
        let was = std::mem::replace(&mut self.state, State::Verified);
        self.stale_raised = false;
        self.publish(store);
        let seq = self.held.as_ref().map(|h| h.seq).unwrap_or(0);
        let fresh = matches!(self.status_written, Some((s, t)) if s == seq && now.duration_since(t) < Duration::from_secs(60));
        if !fresh || was != State::Verified {
            self.write_status(store, &format!("verified seq {seq} at {}", self.since_wall));
            self.status_written = Some((seq, now));
        }
        self.raise_new_entries(store);
        if let Some(h) = &self.held {
            refuse_stored(store, &h.keys, sender);
        }
    }

    /// Design 5.9: one coalesced `key-revoked` entry for the entries above the last raised seq.
    fn raise_new_entries(&self, store: &Store) {
        let Some(h) = &self.held else { return };
        let raised: i64 = store.read_state(ATTENTION_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0);
        if h.seq <= raised {
            return;
        }
        // only the keys not named before (r2 CC 10, OC 7); a reissue with nothing new raises nothing
        let named: BTreeSet<String> =
            store.read_state(RAISED_KEYS_STATE).ok().flatten().unwrap_or_default().lines().map(str::to_string).collect();
        let new: Vec<&J> = h.entries.iter().filter(|e| e.get("key").and_then(J::as_str).is_some_and(|k| !named.contains(k))).collect();
        let all_keys: Vec<String> = h.keys.iter().cloned().collect();
        if new.is_empty() {
            let _ = store.write_state(ATTENTION_STATE, &h.seq.to_string());
            return;
        }
        let total = new.len();
        let names: Vec<String> = new
            .iter()
            .take(20)
            .map(|e| {
                let s = |n: &str| e.get(n).and_then(J::as_str).unwrap_or("?").to_string();
                format!("{}/{} ({}, by {}: {})", s("project"), s("agent"), s("key"), s("by"), s("reason"))
            })
            .collect();
        let more = if total > names.len() { format!(" and {} more", total - names.len()) } else { String::new() };
        let detail = format!(
            "hub {} revocation list is at seq {}: {} key(s) newly revoked: {}{more}; messages signed by them are refused",
            self.hub_id,
            h.seq,
            total,
            names.join("; ")
        );
        if store.attention_coalesced("key-revoked", &h.seq.to_string(), &detail).is_ok() {
            let _ = store.write_state(RAISED_KEYS_STATE, &all_keys.join("\n"));
            let _ = store.write_state(ATTENTION_STATE, &h.seq.to_string());
        }
    }

    fn failed(&mut self, store: &Store, now: Instant, why: &str, tamper: bool) {
        let hour = crate::store::now_ms() / 3_600_000;
        if tamper {
            let _ = store.attention_coalesced("revocation-tamper", &hour.to_string(), &format!("hub {}: {why}", self.hub_id));
        }
        if why == "hub predates revocation list" {
            let day = hour / 24;
            let _ = store.attention_coalesced(
                "revocation-hub-predates",
                &day.to_string(),
                &format!("hub {} serves no revocation list (it predates T-3703); revoked keys are not known here", self.hub_id),
            );
        }
        let since_ms = self.since_wall;
        if self.state != State::Stale && now.duration_since(self.since()) > self.stale_after {
            self.state = State::Stale;
            if !self.stale_raised {
                self.stale_raised = true;
                let _ = store.attention(
                    "revocation-stale",
                    &format!(
                        "hub {}: no verified revocation pull since {since_ms} ({why}); urgent mail from outside the own project is downgraded, never dropped",
                        self.hub_id
                    ),
                );
            }
        }
        self.status_written = None;
        self.publish(store);
        let line = if why.starts_with("ROLLBACK") || why == "hub predates revocation list" {
            why.to_string()
        } else if self.state == State::Stale {
            format!("stale since {since_ms} ({why})")
        } else {
            format!("unverified since {since_ms} ({why})")
        };
        self.write_status(store, &line);
        // r3 finding 1: stamped after the line is rewritten (pinned by e41_a_failed_pull_is_not_an_empty_list…)
        crate::fields::touch(crate::fields::REVOCATION);
    }
}

fn held_of(l: &J) -> Held {
    let entries = entries_of(l);
    Held {
        seq: l.get("seq").and_then(J::as_int).unwrap_or(0),
        digest: l.get("digest").and_then(J::as_str).unwrap_or("").to_string(),
        keys: keys_of(&entries),
        entries,
    }
}

/// [`refuse_stored`] with the keys currently held (the hand-over re-check calls this).
pub fn refuse_stored_now(store: &Store, sender: Option<&crate::send::Sender>) -> usize {
    refuse_stored(store, &held_keys(store), sender)
}

/// FCW-200 sweep: every STORED message from a revoked key that is not handed over, refused or held open by an
/// attempt is REFUSED (reason `revoked`) at once, with one attention entry and a signed notice to its sender
/// when a sender is at hand. Returns how many were refused.
pub fn refuse_stored(store: &Store, keys: &BTreeSet<String>, sender: Option<&crate::send::Sender>) -> usize {
    if keys.is_empty() {
        return 0;
    }
    let mut n = 0;
    for conv in store.conversations() {
        let Ok(lines) = store.chain_lines(&conv) else { continue };
        for st in lines.iter().filter(|l| l.event == "STORED") {
            let from = st.sender.as_str();
            let is_revoked = from.len() >= 16 && keys.iter().any(|k| k.strip_prefix("ed25519:").is_some_and(|h| h.starts_with(from)));
            if !is_revoked {
                continue;
            }
            let done = |ev: &str| lines.iter().any(|l| l.event == ev && l.msg_id == st.msg_id && l.sender == st.sender);
            if done("HANDED_OVER") || done("REFUSED") {
                continue;
            }
            // an open hand-over attempt is already in the agent's hands; its finalize decides (r2 CC 9)
            let closed: BTreeSet<String> = lines
                .iter()
                .filter(|l| l.event == "ABSENT" && l.msg_id == st.msg_id)
                .filter_map(|l| l.reason.as_deref().and_then(|r| r.split_whitespace().next()).map(str::to_string))
                .collect();
            let open = store
                .claims_for(&st.msg_id)
                .map(|a| a.iter().any(|n| !closed.contains(&format!("attempt={n}"))))
                .unwrap_or(true);
            if open {
                continue;
            }
            let extra = LineExtra { reason: Some(REASON_REVOKED.to_owned()), ..Default::default() };
            let (id, sd, dg) = (st.msg_id.clone(), st.sender.clone(), st.digest.clone());
            let wrote = store.append_event_unless(&conv, "REFUSED", &st.msg_id, &st.sender, &st.digest, extra, move |ls| {
                ls.iter().any(|l| l.event == "REFUSED" && l.msg_id == id && l.sender == sd && l.digest == dg)
            });
            if let Ok(Some(_)) = wrote {
                n += 1;
                count_refusal(store);
                if let (Some(s), Ok((_, payload))) = (sender, store.read_record(&conv, st.seq, &st.msg_id))
                    && let Ok(msg) = crate::wire::parse_payload(&payload)
                    && let Ok(stage) = crate::pull::build_stage(store, s.fingerprint(), s.agent().0, &msg, REVOKED)
                {
                    let _ = s.post_stage(&msg.from_agent, &stage);
                }
            }
        }
    }
    if n > 0 {
        // one entry per sweep, not one per message (r2 OC 8)
        let _ = store.attention(
            "refused-revoked",
            &format!("{n} stored message(s) from revoked key(s) refused before hand-over (FCW-200); see the chains' REFUSED lines (reason revoked)"),
        );
    }
    n
}


#[cfg(test)]
mod tests {
    //! AL-22 at L2 (E-40 (a)(b), E-41, 06 4.7 classes) with signed test lists and a stub hub; the real hub is
    //! `tests/t3703_revocation_sidecar.rs` (E-39).
    use super::*;
    use ed25519_dalek::SigningKey;
    use std::cell::RefCell;

    const HUB: &str = "cacc73ea32b121dd";

    struct Fx {
        _e: crate::testkit::Estate,
        store: Store,
        k: SigningKey,
        version: Value,
    }

    fn fx() -> Fx {
        let e = crate::testkit::Estate::create_without_hub().unwrap();
        let store = Store::open(&e.store_root(), "fx-b").unwrap();
        let k = SigningKey::from_bytes(&[5u8; 32]);
        let fp = ss::key_fingerprint(&k.verifying_key());
        store.write_state(&crate::hub_key_pin::pin_state_name(HUB), &fp).unwrap();
        let version = serde_json::json!({"hub_id": HUB, "signing_key": fp, "signing_public_key": ss::hex(k.verifying_key().as_bytes())});
        Fx { _e: e, store, k, version }
    }

    fn fp(n: u8) -> String {
        ss::key_fingerprint(&SigningKey::from_bytes(&[n; 32]).verifying_key())
    }

    fn list(k: &SigningKey, seq: i64, prev: &str, keys: &[String]) -> J {
        let entries = keys
            .iter()
            .map(|key| {
                J::Obj(vec![
                    ("scope".into(), J::Str("agent_key".into())),
                    ("key".into(), J::Str(key.clone())),
                    ("project".into(), J::Str("p".into())),
                    ("agent".into(), J::Str("fx-a".into())),
                    ("reason".into(), J::Str("r".into())),
                    ("revoked_at".into(), J::Int(1)),
                    ("by".into(), J::Str("operator".into())),
                    ("event".into(), J::Str("admin-revoke".into())),
                ])
            })
            .collect();
        ss::seal(
            k,
            J::Obj(vec![
                ("schema".into(), J::Str(SCHEMA.into())),
                ("hub_id".into(), J::Str(HUB.into())),
                ("seq".into(), J::Int(seq)),
                ("issued_at".into(), J::Int(1)),
                ("prev_digest".into(), J::Str(prev.into())),
                ("entries".into(), J::Arr(entries)),
            ]),
        )
        .unwrap()
    }

    fn head_of(k: &SigningKey, l: &J) -> J {
        ss::seal(
            k,
            J::Obj(vec![
                ("schema".into(), J::Str(HEAD_SCHEMA.into())),
                ("hub_id".into(), J::Str(HUB.into())),
                ("seq".into(), l.get("seq").unwrap().clone()),
                ("digest".into(), l.get("digest").unwrap().clone()),
            ]),
        )
        .unwrap()
    }

    fn text(o: &J) -> Value {
        serde_json::json!({"text": serde_json::to_string(&o.to_serde()).unwrap()})
    }

    /// A stub hub serving `served` (head, list), or failing with `fail`.
    struct Hub {
        served: RefCell<Option<(J, J)>>,
        fail: RefCell<Option<(i64, String)>>,
        gets: RefCell<u32>,
    }

    impl Hub {
        fn new() -> Hub {
            Hub { served: RefCell::new(None), fail: RefCell::new(None), gets: RefCell::new(0) }
        }
        fn serve(&self, k: &SigningKey, l: &J) {
            *self.served.borrow_mut() = Some((head_of(k, l), l.clone()));
            *self.fail.borrow_mut() = None;
        }
        fn call(&self, m: &str) -> Result<Value, (i64, String)> {
            if let Some(f) = self.fail.borrow().clone() {
                return Err(f);
            }
            let s = self.served.borrow();
            let (h, l) = s.as_ref().expect("served");
            if m == "hub.revocation_get" {
                *self.gets.borrow_mut() += 1;
                Ok(text(l))
            } else {
                Ok(text(h))
            }
        }
    }

    fn kinds(store: &Store) -> Vec<String> {
        store
            .attention_entries()
            .iter()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .map(|v| v["kind"].as_str().unwrap_or("").to_string())
            .collect()
    }

    fn line(store: &Store) -> String {
        status(store)["revocation"].as_str().unwrap().to_string()
    }

    const Z: &str = "0000000000000000000000000000000000000000000000000000000000000000";

    #[test]
    fn empty_list_then_a_revocation_is_verified_installed_and_raised_once() {
        let f = fx();
        let hub = Hub::new();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        assert!(narrowing(&f.store), "unverified since start narrows (FCW-96)");
        let l0 = list(&f.k, 0, Z, &[]);
        hub.serve(&f.k, &l0);
        let now = Instant::now();
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(t.state(), &State::Verified);
        assert!(line(&f.store).starts_with("verified seq 0 at"), "{}", line(&f.store));
        assert!(!narrowing(&f.store));
        let l1 = list(&f.k, 1, l0.get("digest").unwrap().as_str().unwrap(), &[fp(1)]);
        hub.serve(&f.k, &l1);
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(revoked(&f.store, &fp(1)));
        assert!(revoked_from(&f.store, &fp(1)["ed25519:".len()..][..16]));
        let gets = *hub.gets.borrow();
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(*hub.gets.borrow(), gets, "an unchanged head fetches nothing");
        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "key-revoked").count(), 1, "one entry per list");
        // a fresh tracker (restart) does not raise it again
        let mut t2 = Tracker::new(&f.store, HUB, PN13);
        assert!(revoked(&f.store, &fp(1)), "held keys refused from the start (load verified)");
        t2.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "key-revoked").count(), 1);
    }

    #[test]
    fn e40a_rollback_and_fork_are_refused_and_the_list_kept() {
        let f = fx();
        let hub = Hub::new();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        let l7 = list(&f.k, 7, Z, &[fp(1)]);
        hub.serve(&f.k, &l7);
        let now = Instant::now();
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(t.held_seq(), Some(7));
        hub.serve(&f.k, &list(&f.k, 5, Z, &[]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(line(&f.store), "ROLLBACK refused, kept seq 7");
        assert_eq!(t.held_seq(), Some(7));
        assert!(revoked(&f.store, &fp(1)), "keys on the last verified list stay refused");
        assert!(kinds(&f.store).contains(&"revocation-tamper".to_string()));
        hub.serve(&f.k, &list(&f.k, 7, Z, &[fp(2)])); // same seq, another digest
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("fork"), "{}", line(&f.store));
        assert!(!revoked(&f.store, &fp(2)));
    }

    #[test]
    fn e40b_a_restored_store_detects_the_higher_head_and_refuses_the_key() {
        let f = fx();
        let hub = Hub::new();
        let l5 = list(&f.k, 5, Z, &[]);
        hub.serve(&f.k, &l5);
        let now = Instant::now();
        {
            let mut t = Tracker::new(&f.store, HUB, PN13);
            t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        }
        // the hub moved on to seq 7 with F revoked at 6; the store still holds seq 5 (the snapshot)
        let l6 = list(&f.k, 6, l5.get("digest").unwrap().as_str().unwrap(), &[fp(1)]);
        let l7 = list(&f.k, 7, l6.get("digest").unwrap().as_str().unwrap(), &[fp(1)]);
        hub.serve(&f.k, &l7);
        let mut t = Tracker::new(&f.store, HUB, PN13);
        assert!(!revoked(&f.store, &fp(1)) && narrowing(&f.store), "restored: unreconciled, narrowed");
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(t.held_seq(), Some(7), "first head read shows 7, list fetched (seq 5 -> 7 skips the chain check)");
        assert!(revoked(&f.store, &fp(1)));
        assert!(!narrowing(&f.store));
    }

    #[test]
    fn e41_a_failed_pull_is_not_an_empty_list_and_goes_stale_after_pn13() {
        let f = fx();
        let hub = Hub::new();
        let stale = Duration::from_secs(3600);
        let mut t = Tracker::new(&f.store, HUB, stale);
        hub.serve(&f.k, &list(&f.k, 1, Z, &[fp(1)]));
        let t0 = Instant::now();
        t.tick(&f.store, &f.version, &|m| hub.call(m), t0, None);
        *hub.fail.borrow_mut() = Some((0, "connection refused".into()));
        t.tick(&f.store, &f.version, &|m| hub.call(m), t0 + Duration::from_secs(3599), None);
        assert_eq!(t.state(), &State::Verified, "59:59: not yet stale");
        assert!(line(&f.store).starts_with("unverified since"), "{}", line(&f.store));
        assert!(!narrowing(&f.store));
        assert!(revoked(&f.store, &fp(1)), "a failed pull is not an empty list");
        t.tick(&f.store, &f.version, &|m| hub.call(m), t0 + Duration::from_secs(3601), None);
        assert_eq!(t.state(), &State::Stale, "60:01: stale");
        assert!(line(&f.store).starts_with("stale since"), "{}", line(&f.store));
        assert!(narrowing(&f.store));
        assert!(revoked(&f.store, &fp(1)), "keys on the last verified list stay refused");
        t.tick(&f.store, &f.version, &|m| hub.call(m), t0 + Duration::from_secs(4000), None);
        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "revocation-stale").count(), 1, "one stale entry");
        // a hub refusal counts the same (RECONCILE)
        *hub.fail.borrow_mut() = Some((-32048, "RECONCILE".into()));
        t.tick(&f.store, &f.version, &|m| hub.call(m), t0 + Duration::from_secs(4100), None);
        assert_eq!(t.state(), &State::Stale);
        // a verified pull ends it
        hub.serve(&f.k, &list(&f.k, 1, Z, &[fp(1)]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), t0 + Duration::from_secs(4200), None);
        assert_eq!(t.state(), &State::Verified);
        assert!(!narrowing(&f.store));
    }

    #[test]
    fn the_06_4_7_failure_classes_never_verify() {
        let f = fx();
        let hub = Hub::new();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        let now = Instant::now();
        // method not found: a hub that predates the list
        *hub.fail.borrow_mut() = Some((-32601, "method not found".into()));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(line(&f.store), "hub predates revocation list");
        assert_eq!(t.state(), &State::UnverifiedSinceStart);
        // a list signed by another key
        let other = SigningKey::from_bytes(&[6u8; 32]);
        hub.serve(&other, &list(&other, 1, Z, &[fp(1)]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("head-signature"), "{}", line(&f.store));
        assert!(!revoked(&f.store, &fp(1)));
        // an offered public key that does not hash to the pin
        let mut v = f.version.clone();
        v["signing_public_key"] = Value::from(ss::hex(other.verifying_key().as_bytes()));
        hub.serve(&f.k, &list(&f.k, 1, Z, &[fp(1)]));
        t.tick(&f.store, &v, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("pin"), "{}", line(&f.store));
        // a head for another hub
        let foreign = ss::seal(
            &f.k,
            J::Obj(vec![
                ("schema".into(), J::Str(HEAD_SCHEMA.into())),
                ("hub_id".into(), J::Str("ffffffffffffffff".into())),
                ("seq".into(), J::Int(1)),
                ("digest".into(), J::Str(Z.into())),
            ]),
        )
        .unwrap();
        *hub.served.borrow_mut() = Some((foreign, list(&f.k, 1, Z, &[])));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("another hub"), "{}", line(&f.store));
        // a list that breaks the chain at held + 1
        let l1 = list(&f.k, 1, Z, &[fp(1)]);
        hub.serve(&f.k, &l1);
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(t.state(), &State::Verified);
        hub.serve(&f.k, &list(&f.k, 2, &"1".repeat(64), &[fp(1), fp(2)]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("chain"), "{}", line(&f.store));
        // a list that drops a held key
        hub.serve(&f.k, &list(&f.k, 2, l1.get("digest").unwrap().as_str().unwrap(), &[fp(2)]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("drops a key"), "{}", line(&f.store));
        assert!(revoked(&f.store, &fp(1)));
    }

    #[test]
    fn m1_a_tampered_stored_list_is_tamper_and_refetched() {
        let f = fx();
        let hub = Hub::new();
        let l = list(&f.k, 3, Z, &[fp(1), fp(2)]);
        hub.serve(&f.k, &l);
        let now = Instant::now();
        Tracker::new(&f.store, HUB, PN13).tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        // edit the stored list: drop an entry, keep the digest and signature
        let mut edited = l.clone();
        edited.set("entries", J::Arr(vec![]));
        f.store.write_state(LIST_STATE, &serde_json::to_string(&edited.to_serde()).unwrap()).unwrap();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        assert!(kinds(&f.store).contains(&"revocation-tamper".to_string()));
        assert_eq!(t.held_seq(), Some(3), "seq kept as the floor");
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(t.state(), &State::Verified, "the full list is fetched again");
        assert!(revoked(&f.store, &fp(1)) && revoked(&f.store, &fp(2)));
    }

    #[test]
    fn the_switch_off_is_ph1_trust_said_so_and_a_flip_is_raised() {
        let f = fx();
        let hub = Hub::new();
        hub.serve(&f.k, &list(&f.k, 1, Z, &[fp(1)]));
        let mut t = Tracker::new(&f.store, HUB, PN13);
        let now = Instant::now();
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(revoked(&f.store, &fp(1)));
        f.store.write_state(SWITCH_STATE, "off").unwrap();
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(line(&f.store), "off (PH-1 trust)");
        assert!(!revoked(&f.store, &fp(1)));
        assert!(kinds(&f.store).contains(&"revocation-pull-switched".to_string()));
        f.store.write_state(SWITCH_STATE, "on").unwrap();
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(revoked(&f.store, &fp(1)));
        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "revocation-pull-switched").count(), 2);
    }

    // ---- round 2 (docs/reports/T-3703-reviews/r2-dispositions.md) ----

    #[test]
    fn r2_the_entry_names_only_new_keys_and_a_reissue_raises_nothing() {
        let f = fx();
        let hub = Hub::new();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        let now = Instant::now();
        let l1 = list(&f.k, 1, Z, &[fp(1)]);
        hub.serve(&f.k, &l1);
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        let l2 = list(&f.k, 2, l1.get("digest").unwrap().as_str().unwrap(), &[fp(1), fp(2)]);
        hub.serve(&f.k, &l2);
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        let details: Vec<String> = f
            .store
            .attention_entries()
            .iter()
            .filter_map(|l| serde_json::from_str::<Value>(l).ok())
            .filter(|v| v["kind"] == "key-revoked")
            .map(|v| v["detail"].as_str().unwrap_or("").to_string())
            .collect();
        assert_eq!(details.len(), 2);
        assert!(details[1].contains(&fp(2)) && !details[1].contains(&fp(1)), "{}", details[1]);
        // a reissue: seq 3, same keys -> nothing new, no entry
        hub.serve(&f.k, &list(&f.k, 3, l2.get("digest").unwrap().as_str().unwrap(), &[fp(1), fp(2)]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "key-revoked").count(), 2);
    }

    #[test]
    fn r2_without_a_tracker_the_stored_list_still_refuses() {
        let f = fx();
        let l = list(&f.k, 1, Z, &[fp(1)]);
        f.store.write_state(LIST_STATE, &serde_json::to_string(&l.to_serde()).unwrap()).unwrap();
        assert!(revoked(&f.store, &fp(1)), "no snapshot published for this store: fail closed from the stored list");
        assert!(revoked_from(&f.store, &fp(1)["ed25519:".len()..][..16]));
    }

    #[test]
    fn r2_a_corrupted_stored_list_heals_on_the_first_verified_fetch() {
        let f = fx();
        let hub = Hub::new();
        let l = list(&f.k, 9, Z, &[fp(1)]);
        hub.serve(&f.k, &l);
        let now = Instant::now();
        Tracker::new(&f.store, HUB, PN13).tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        // corrupt the stored copy: another key and a huge seq (neither signed)
        let mut bad = l.clone();
        bad.set("seq", J::Int(1_000_000));
        bad.set("entries", J::Arr(vec![J::Obj(vec![("key".into(), J::Str(fp(7)))])]));
        f.store.write_state(LIST_STATE, &serde_json::to_string(&bad.to_serde()).unwrap()).unwrap();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        assert!(revoked(&f.store, &fp(7)), "until verified, the stored keys are refused (fail closed)");
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert_eq!(t.state(), &State::Verified, "{}", line(&f.store));
        assert_eq!(t.held_seq(), Some(9), "no floor kept from an unverified list");
        assert!(revoked(&f.store, &fp(1)) && !revoked(&f.store, &fp(7)), "the hub's list replaces it");
    }

    #[test]
    fn r2_an_old_hub_without_a_public_key_is_predates_and_no_pin_yet_is_not_tamper() {
        let f = fx();
        let hub = Hub::new();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        let now = Instant::now();
        let mut old = f.version.clone();
        old.as_object_mut().unwrap().remove("signing_public_key");
        t.tick(&f.store, &old, &|m| hub.call(m), now, None);
        assert_eq!(line(&f.store), "hub predates revocation list");
        assert!(kinds(&f.store).contains(&"revocation-hub-predates".to_string()));
        f.store.remove_state(&crate::hub_key_pin::pin_state_name(HUB)).unwrap();
        hub.serve(&f.k, &list(&f.k, 1, Z, &[]));
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        assert!(line(&f.store).contains("not pinned yet"), "{}", line(&f.store));
        assert!(!kinds(&f.store).contains(&"revocation-tamper".to_string()), "no pin yet is not interference");
    }

    #[test]
    fn r2_the_sweep_leaves_an_open_attempt_to_its_finalize() {
        let f = fx();
        let key = fp(1);
        let from = &key["ed25519:".len()..][..16];
        let keys: BTreeSet<String> = [key.clone()].into();
        f.store.put(from, "c1", "m1", b"payload").unwrap();
        f.store.claim("m1", 1, "n1", 1).unwrap();
        assert_eq!(refuse_stored(&f.store, &keys, None), 0, "attempt 1 is open");
        f.store
            .append_event_unless("c1", "ABSENT", "m1", from, "d", LineExtra { reason: Some("attempt=1 nonce=n1".into()), ..Default::default() }, |_| false)
            .unwrap();
        assert_eq!(refuse_stored(&f.store, &keys, None), 1, "closed attempt: refused");
        assert_eq!(kinds(&f.store).iter().filter(|k| *k == "refused-revoked").count(), 1, "one entry per sweep");
    }

    #[test]
    fn r3_a_pinned_hub_offering_a_malformed_key_is_tamper() {
        let f = fx();
        let hub = Hub::new();
        let mut t = Tracker::new(&f.store, HUB, PN13);
        let mut v = f.version.clone();
        v["signing_public_key"] = Value::from("not-a-key");
        t.tick(&f.store, &v, &|m| hub.call(m), Instant::now(), None);
        assert!(line(&f.store).contains("malformed"), "{}", line(&f.store));
        assert!(kinds(&f.store).contains(&"revocation-tamper".to_string()));
    }

    /// E-40 (c): a sidecar restored from a backup taken before F was revoked, with the hub unreachable, cannot know
    /// about F: verdict LIMIT (RR-20). What it must do: stay unreconciled (narrowing, so no urgent delivery from
    /// outside the own project) and never claim a verified list.
    #[test]
    fn e40c_limit_rr20_restored_while_the_hub_is_unreachable() {
        let f = fx();
        let hub = Hub::new();
        let l5 = list(&f.k, 5, Z, &[]);
        hub.serve(&f.k, &l5);
        let now = Instant::now();
        Tracker::new(&f.store, HUB, PN13).tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        // restore (the snapshot holds seq 5, F not on it) and the hub is unreachable
        *hub.fail.borrow_mut() = Some((0, "connection refused".into()));
        let mut t = Tracker::new(&f.store, HUB, PN13);
        t.tick(&f.store, &f.version, &|m| hub.call(m), now, None);
        let limit = !revoked(&f.store, &fp(1)); // RR-20: normal delivery from F is the stated limit
        assert!(limit, "LIMIT RR-20: a key revoked after the backup is unknown while cut off");
        assert_eq!(t.state(), &State::UnverifiedSinceStart);
        assert!(narrowing(&f.store), "no urgent delivery from outside the own project while unreconciled");
        assert!(line(&f.store).starts_with("unverified since"), "{}", line(&f.store));
        println!("E-40 (c) verdict: LIMIT (RR-20)");
    }

    /// T-3823 (T-3703 r3 OC 4): the no-snapshot fallback is cached per (len, mtime) and follows a new list.
    #[test]
    fn t3823_the_fallback_cache_follows_the_stored_list() {
        let f = fx();
        let write = |keys: &[String]| {
            let l = list(&f.k, 1, Z, keys);
            f.store.write_state(LIST_STATE, &serde_json::to_string(&l.to_serde()).unwrap()).unwrap();
        };
        write(&[fp(1)]);
        assert!(revoked(&f.store, &fp(1)));
        assert!(revoked(&f.store, &fp(1)), "second lookup from the cache");
        write(&[fp(2), fp(3)]);
        assert!(!revoked(&f.store, &fp(1)) && revoked(&f.store, &fp(3)), "a changed list is read again");
    }
}
