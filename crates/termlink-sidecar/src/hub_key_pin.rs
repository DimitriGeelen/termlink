//! T-3366 (arc-011 P-10, B-1; AL-18 sidecar half, E-37): pin the hub's signing key.
//!
//! The hub states `signing_key = "ed25519:<sha256 of the public key>"` in `hub.version`. The
//! sidecar pins it on its first VERIFIED contact and from then on accepts hub-signed data only
//! under that pin. Hub-signed objects arrive with P-15 (revocation list) and P-16 (dead letters);
//! until then the pin is recorded, compared and reported, and [`verified_hub_key`] is the gate
//! those units call.
//!
//! Rules (06a 2.2):
//! 1. No `signing_key` offered (a hub that predates T-3366): nothing pinned, nothing refused.
//! 2. No pin yet: pin only when the contact is over the local hub socket (FCW-84: anything else
//!    pins nothing and hub-signed data stays unverified). TCP sidecar transport has no runtime
//!    implementation today, so "TLS under the pinned certificate" is not reachable here yet.
//! 3. Pin differs from the offer: every hub-signed object is refused, and a needs-attention entry
//!    names both fingerprints, once per day per offered key (FCW-83). A pin is never replaced
//!    here; only an operator re-pin (runme) changes it.
//!
//! Mail is NOT held on a key mismatch: there are no hub-signed objects yet, and stopping mail
//! would switch off a working route (NID rule). The hub-id check (FX-2) still holds mail.

use serde_json::{Value, json};

use crate::store::Store;

/// What one observation of `hub.version` found.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum KeyState {
    /// The hub offers no signing key (pre-T-3366).
    NotOffered,
    /// The offer matches the pin.
    Pinned(String),
    /// First verified contact: the offer was pinned now.
    NewlyPinned(String),
    /// No pin, and this contact was not over a trusted path (FCW-84).
    Unverified(String),
    /// The offer differs from the pin (FCW-83).
    Mismatch { pinned: String, offered: String },
}

impl KeyState {
    pub fn as_str(&self) -> &'static str {
        match self {
            KeyState::NotOffered => "not offered",
            KeyState::Pinned(_) | KeyState::NewlyPinned(_) => "pinned",
            KeyState::Unverified(_) => "unverified",
            KeyState::Mismatch { .. } => "mismatch",
        }
    }
}

pub fn pin_state_name(hub_id: &str) -> String {
    format!("hub-key-pin-{hub_id}")
}

fn seen_state_name(hub_id: &str) -> String {
    format!("hub-key-seen-{hub_id}")
}

fn valid_fp(s: &str) -> bool {
    s.strip_prefix("ed25519:")
        .is_some_and(|h| h.len() == 64 && h.bytes().all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b)))
}

fn offered(version: &Value) -> Option<String> {
    version.get("signing_key").and_then(Value::as_str).map(str::to_owned)
}

fn alert(store: &Store, kind: &str, detail: &str) {
    if let Err(e) = store.attention(kind, detail) {
        eprintln!("termlink-sidecar: needs attention ({kind}): {detail} [could not record it: {e}]");
    }
}

/// Record the last observation so `status` can show it without a hub call; written only on change.
fn remember(store: &Store, hub_id: &str, st: &KeyState) {
    let off = match st {
        KeyState::NotOffered => String::new(),
        KeyState::Pinned(f) | KeyState::NewlyPinned(f) | KeyState::Unverified(f) => f.clone(),
        KeyState::Mismatch { offered, .. } => offered.clone(),
    };
    let line = format!("{} {}", st.as_str().replace(' ', "-"), off);
    let name = seen_state_name(hub_id);
    if store.read_state(&name).ok().flatten().as_deref().map(str::trim) != Some(line.trim()) {
        let _ = store.write_state(&name, &line);
    }
}

/// Observe one `hub.version` reply from the hub `hub_id` (already matched against the declared
/// id). `trusted_path` is true for the local hub socket. Never fails: a problem becomes a state
/// and, where the spec asks for one, a needs-attention entry.
pub fn observe(store: &Store, hub_id: &str, version: &Value, trusted_path: bool) -> KeyState {
    let st = match offered(version) {
        None => KeyState::NotOffered,
        Some(off) => {
            let pin = store.read_state(&pin_state_name(hub_id)).ok().flatten().map(|s| s.trim().to_string());
            match pin {
                Some(p) if p == off => KeyState::Pinned(off),
                Some(p) => {
                    let detail = format!(
                        "hub {hub_id} signing key changed: pinned {p}, offered {off}; every hub-signed object is refused until an operator re-pin"
                    );
                    let day = crate::store::now_ms() / 86_400_000;
                    if let Err(e) = store.attention_coalesced("hub-signing-key-changed", &format!("{day}:{off}"), &detail) {
                        eprintln!("termlink-sidecar: needs attention (hub-signing-key-changed): {detail} [could not record it: {e}]");
                    }
                    KeyState::Mismatch { pinned: p, offered: off }
                }
                None if !trusted_path || !valid_fp(&off) => KeyState::Unverified(off),
                None => match store.write_state(&pin_state_name(hub_id), &off) {
                    Ok(()) => {
                        alert(store, "hub-signing-key-pinned", &format!("hub {hub_id} signing key pinned: {off}"));
                        KeyState::NewlyPinned(off)
                    }
                    Err(e) => {
                        alert(store, "hub-signing-key-pin-failed", &format!("hub {hub_id}: could not record the pin {off}: {e}"));
                        KeyState::Unverified(off)
                    }
                },
            }
        }
    };
    remember(store, hub_id, &st);
    st
}

/// The gate for hub-signed objects (P-15, P-16): the hub's key fingerprint when, and only when,
/// the offer in `version` equals the stored pin. `None` means refuse.
pub fn verified_hub_key(store: &Store, hub_id: &str, version: &Value) -> Option<String> {
    let off = offered(version)?;
    let pin = store.read_state(&pin_state_name(hub_id)).ok().flatten()?;
    (pin.trim() == off).then_some(off)
}

/// `status` view (OBS-36): `{pinned, offered, state}` from the stored pin and last observation.
pub fn status(store: &Store, hub_id: Option<&str>) -> Value {
    let Some(h) = hub_id else {
        return json!({"pinned": null, "offered": null, "state": "no hub id configured"});
    };
    let pinned = store.read_state(&pin_state_name(h)).ok().flatten().map(|s| s.trim().to_string());
    let seen = store.read_state(&seen_state_name(h)).ok().flatten().unwrap_or_default();
    let mut it = seen.trim().splitn(2, ' ');
    let state = it.next().filter(|s| !s.is_empty()).unwrap_or("not observed").replace('-', " ");
    let offered = it.next().filter(|s| !s.is_empty()).map(str::to_owned);
    json!({"pinned": pinned, "offered": offered, "state": state})
}
