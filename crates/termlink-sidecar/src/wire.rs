//! C-3 wire format and signing (architecture note section 5; R-75, SI-3, SI-4, SI-26).
//!
//! The hub's signed bytes do not cover `metadata`, so every identity-bearing
//! field sits inside `payload`. The sender signs
//! `canonical_sign_bytes(topic, MSG_TYPE, payload, None, ts)` with its own key;
//! the receiver re-verifies and also checks the roster and its own instance id.

use ed25519_dalek::{Signature, VerifyingKey};
use serde_json::Value;
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fmt;
use termlink_protocol::control::channel::canonical_sign_bytes;
use termlink_session::agent_identity::{fingerprint_of, Identity};

/// Hub-visible message type of a sidecar message.
pub const MSG_TYPE: &str = "sidecar.msg.v1";
pub const WIRE_VERSION: i64 = 1;
/// Hub-visible message type of a stage record (section 5.3).
pub const STAGE_TYPE: &str = "sidecar.stage.v1";
/// Post metadata keys that carry the transport signature to the receiver. The
/// hub verifies the signature on post but does not return it on subscribe, and
/// metadata is outside the signed bytes, so carrying it there changes nothing
/// the signature covers: an altered key or signature simply fails `verify`.
pub const META_PK: &str = "sidecar_pk";
pub const META_SIG: &str = "sidecar_sig";
/// Stages a receiver reports back, in their only allowed order (forward-only).
pub const STAGE_ORDER: [&str; 4] = ["RECEIVED", "STORED", "HANDED_OVER", "REPLIED"];
/// Notices a receiver sends about a message from a sender outside its allow-list
/// (OQ-2 ruling E, B15): held for its operator, then admitted or refused by that
/// operator. Since PH-2 row P2 also the receiver's overdue notices (R-44.e): a
/// stored message not handed over, or a handed-over one not answered, while the
/// recipient's agent is ready. They are not stages of the order above; each is
/// recorded once.
pub const NOTICES: [&str; 5] = ["HELD", "ADMITTED", "DENIED", crate::deadline::OVERDUE_HANDOVER, crate::deadline::OVERDUE_ANSWER];

/// The signed message body (section 5.1).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Msg {
    pub from: String,
    pub from_agent: String,
    /// The sender's exact instance: a reply is addressed to it (SI-26, B11).
    pub from_instance: String,
    pub to_agent: String,
    pub to_instance: String,
    pub conv: String,
    pub msg_id: String,
    pub seq: u64,
    pub priority: String,
    pub ts_ms: i64,
    pub body: String,
}

impl Msg {
    fn to_value(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("v".into(), Value::from(WIRE_VERSION));
        m.insert("kind".into(), "msg".into());
        m.insert("from".into(), self.from.clone().into());
        m.insert("from_agent".into(), self.from_agent.clone().into());
        m.insert("from_instance".into(), self.from_instance.clone().into());
        m.insert("to_agent".into(), self.to_agent.clone().into());
        m.insert("to_instance".into(), self.to_instance.clone().into());
        m.insert("conv".into(), self.conv.clone().into());
        m.insert("msg_id".into(), self.msg_id.clone().into());
        m.insert("seq".into(), Value::from(self.seq));
        m.insert("priority".into(), self.priority.clone().into());
        m.insert("ts_ms".into(), Value::from(self.ts_ms));
        m.insert("body".into(), self.body.clone().into());
        Value::Object(m)
    }

    fn from_value(v: &Value) -> Result<Self, Refusal> {
        let o = v.as_object().ok_or(Refusal::Malformed("not an object"))?;
        if o.len() != 13 {
            return Err(Refusal::Malformed("unexpected field set"));
        }
        if o.get("v").and_then(Value::as_i64) != Some(WIRE_VERSION) {
            return Err(Refusal::Malformed("unsupported version"));
        }
        if o.get("kind").and_then(Value::as_str) != Some("msg") {
            return Err(Refusal::Malformed("kind is not msg"));
        }
        let s = |k: &'static str| {
            o.get(k)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(Refusal::Malformed(k))
        };
        Ok(Msg {
            from: s("from")?,
            from_agent: s("from_agent")?,
            from_instance: s("from_instance")?,
            to_agent: s("to_agent")?,
            to_instance: s("to_instance")?,
            conv: s("conv")?,
            msg_id: s("msg_id")?,
            seq: o
                .get("seq")
                .and_then(Value::as_u64)
                .ok_or(Refusal::Malformed("seq"))?,
            priority: s("priority")?,
            ts_ms: o
                .get("ts_ms")
                .and_then(Value::as_i64)
                .ok_or(Refusal::Malformed("ts_ms"))?,
            body: s("body")?,
        })
    }

    /// Canonical payload bytes: UTF-8, keys sorted, no insignificant whitespace.
    pub fn payload_bytes(&self) -> Vec<u8> {
        canonical_json(&self.to_value()).into_bytes()
    }

    /// `sha256(payload bytes)`, lower-case hex.
    pub fn digest(&self) -> String {
        digest_hex(&self.payload_bytes())
    }
}

/// Parse stored payload bytes back into a message: canonical v1 only. A record
/// was verified before it was stored; this re-reads it, it does not re-verify.
pub fn parse_payload(payload: &[u8]) -> Result<Msg, Refusal> {
    let value: Value = serde_json::from_slice(payload).map_err(|_| Refusal::Malformed("payload is not JSON"))?;
    let msg = Msg::from_value(&value)?;
    if msg.payload_bytes() != payload {
        return Err(Refusal::NotCanonical);
    }
    Ok(msg)
}

/// Mint a random 128-bit hex message id (reuses the offline queue's minter).
pub fn mint_msg_id() -> String {
    termlink_session::offline_queue::mint_client_msg_id()
}

pub fn digest_hex(payload: &[u8]) -> String {
    hex(&Sha256::digest(payload))
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// A 64-hex ed25519 public key, or `None` when it is not one.
pub fn parse_public_key(hex: &str) -> Option<VerifyingKey> {
    let b: [u8; 32] = unhex(hex)?.try_into().ok()?;
    VerifyingKey::from_bytes(&b).ok()
}

fn unhex(s: &str) -> Option<Vec<u8>> {
    if !s.len().is_multiple_of(2) || !s.is_ascii() {
        return None;
    }
    (0..s.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok())
        .collect()
}

/// Serialise with explicitly sorted keys, independent of serde_json's
/// `preserve_order` feature (which another crate in the graph may enable).
pub fn canonical_json(v: &Value) -> String {
    let mut out = String::new();
    write_canon(v, &mut out);
    out
}

fn write_canon(v: &Value, out: &mut String) {
    match v {
        Value::Object(m) => {
            let sorted: BTreeMap<&String, &Value> = m.iter().collect();
            out.push('{');
            for (i, (k, val)) in sorted.into_iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                out.push_str(&Value::String(k.clone()).to_string());
                out.push(':');
                write_canon(val, out);
            }
            out.push('}');
        }
        Value::Array(a) => {
            out.push('[');
            for (i, val) in a.iter().enumerate() {
                if i > 0 {
                    out.push(',');
                }
                write_canon(val, out);
            }
            out.push(']');
        }
        other => out.push_str(&other.to_string()),
    }
}

/// A message as posted: payload bytes, timestamp, signer key and signature.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Signed {
    pub topic: String,
    pub ts_ms: i64,
    pub payload: Vec<u8>,
    /// Signer public key, 64 hex chars.
    pub signer_pk: String,
    /// Signature, 128 hex chars.
    pub sig: String,
}

/// Sign `msg` for `topic` at transport timestamp `ts_ms` with the sender's own key.
pub fn sign(id: &Identity, topic: &str, ts_ms: i64, msg: &Msg) -> Signed {
    let payload = msg.payload_bytes();
    let bytes = canonical_sign_bytes(topic, MSG_TYPE, &payload, None, ts_ms);
    let sig = id.sign(&bytes);
    Signed {
        topic: topic.to_owned(),
        ts_ms,
        payload,
        signer_pk: id.public_key_hex(),
        sig: hex(&sig.to_bytes()),
    }
}

/// Why a message was refused. Each variant is a distinct, loud reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refusal {
    /// Payload is not a valid v1 message.
    Malformed(&'static str),
    /// Payload parses but is not byte-identical to its canonical form.
    NotCanonical,
    /// The signer key does not hash to the `from` fingerprint inside the payload.
    FromMismatch,
    /// `from` is not in the roster (this is how the shared host key fails).
    NotInRoster,
    /// Signer key differs from the roster key held for `from`.
    KeyMismatch,
    /// Signature does not verify (altered payload, timestamp or topic).
    BadSignature,
    /// `to_instance` names another or ended instance (SI-26): never redirected.
    WrongInstance { want: String, got: String },
}

impl fmt::Display for Refusal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Malformed(w) => write!(f, "malformed message: {w}"),
            Self::NotCanonical => write!(f, "payload is not canonical"),
            Self::FromMismatch => write!(f, "signer key does not match the from identity"),
            Self::NotInRoster => write!(f, "sender is not in the roster"),
            Self::KeyMismatch => write!(f, "signer key differs from the roster key"),
            Self::BadSignature => write!(f, "signature does not verify"),
            Self::WrongInstance { want, got } => {
                write!(f, "message is for instance {got}, this is {want}")
            }
        }
    }
}

impl std::error::Error for Refusal {}

/// Sender fingerprint -> public key. PH-1: filled by the harness (note 7.2).
#[derive(Debug, Clone, Default)]
pub struct Roster(BTreeMap<String, VerifyingKey>);

impl Roster {
    pub fn new() -> Self {
        Self::default()
    }

    /// Add a key; returns its fingerprint.
    pub fn add(&mut self, pk: VerifyingKey) -> String {
        let fp = fingerprint_of(&pk);
        self.0.insert(fp.clone(), pk);
        fp
    }

    pub fn get(&self, fp: &str) -> Option<&VerifyingKey> {
        self.0.get(fp)
    }

    /// Add every key of `other`.
    pub fn extend(&mut self, other: &Roster) {
        for (fp, pk) in &other.0 {
            self.0.insert(fp.clone(), *pk);
        }
    }
}

/// Signer key -> `from`, roster, signature: the checks shared by messages and stages.
fn check_signer(s: &Signed, roster: &Roster, from: &str, msg_type: &str) -> Result<(), Refusal> {
    let pk_bytes: [u8; 32] = unhex(&s.signer_pk)
        .and_then(|b| b.try_into().ok())
        .ok_or(Refusal::Malformed("signer key"))?;
    let pk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| Refusal::Malformed("signer key"))?;
    if fingerprint_of(&pk) != from {
        return Err(Refusal::FromMismatch);
    }
    let rostered = roster.get(from).ok_or(Refusal::NotInRoster)?;
    if rostered.as_bytes() != pk.as_bytes() {
        return Err(Refusal::KeyMismatch);
    }
    let sig_bytes: [u8; 64] = unhex(&s.sig)
        .and_then(|b| b.try_into().ok())
        .ok_or(Refusal::Malformed("signature"))?;
    let sig = Signature::from_bytes(&sig_bytes);
    let signed = canonical_sign_bytes(&s.topic, msg_type, &s.payload, None, s.ts_ms);
    if !Identity::verify(rostered, &signed, &sig) {
        return Err(Refusal::BadSignature);
    }
    Ok(())
}

/// Verify a posted message as the receiving instance `my_instance`.
pub fn verify(s: &Signed, roster: &Roster, my_instance: &str) -> Result<Msg, Refusal> {
    let value: Value = serde_json::from_slice(&s.payload)
        .map_err(|_| Refusal::Malformed("payload is not JSON"))?;
    let msg = Msg::from_value(&value)?;
    if msg.payload_bytes() != s.payload {
        return Err(Refusal::NotCanonical);
    }
    check_signer(s, roster, &msg.from, MSG_TYPE)?;
    if msg.to_instance != my_instance {
        return Err(Refusal::WrongInstance {
            want: my_instance.to_owned(),
            got: msg.to_instance,
        });
    }
    Ok(msg)
}

/// Who signed a verified message, relative to the roster (B8).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Signer {
    /// The signer is the roster key held for `from`.
    Rostered,
    /// The signer is a well-formed key that is honest about itself (its
    /// fingerprint is `from` and the signature verifies under it), but `from` is
    /// not in the roster. Its identity is proven; its admission is the policy's.
    Unrostered,
}

/// Verify like [`verify`], except that a sender outside the roster is not an
/// error: the signature is then checked under the signer's own key, which must
/// hash to `from`. Every forgery still fails: a key claiming another identity
/// (`FromMismatch`), a roster entry whose key differs (`KeyMismatch`), an altered
/// payload (`BadSignature`), another instance (`WrongInstance`).
pub fn verify_open(s: &Signed, roster: &Roster, my_instance: &str) -> Result<(Msg, Signer), Refusal> {
    match verify(s, roster, my_instance) {
        Ok(m) => return Ok((m, Signer::Rostered)),
        Err(Refusal::NotInRoster) => {}
        Err(e) => return Err(e),
    }
    // NotInRoster is only reached after the signer key was shown to hash to
    // `from`, so checking under that key alone is checking the claimed identity.
    let pk_bytes: [u8; 32] = unhex(&s.signer_pk)
        .and_then(|b| b.try_into().ok())
        .ok_or(Refusal::Malformed("signer key"))?;
    let pk = VerifyingKey::from_bytes(&pk_bytes).map_err(|_| Refusal::Malformed("signer key"))?;
    let mut own = Roster::new();
    own.add(pk);
    let msg = verify(s, &own, my_instance)?;
    Ok((msg, Signer::Unrostered))
}

/// A short, stable reason code for a refusal (B8): goes on the REFUSED chain line.
pub fn refusal_code(r: &Refusal) -> &'static str {
    match r {
        Refusal::Malformed(_) => "malformed",
        Refusal::NotCanonical => "not-canonical",
        Refusal::FromMismatch => "from-mismatch",
        Refusal::NotInRoster => "not-in-roster",
        Refusal::KeyMismatch => "key-mismatch",
        Refusal::BadSignature => "bad-signature",
        Refusal::WrongInstance { .. } => "wrong-instance",
    }
}

// ---- stage records (section 5.3) ------------------------------------------

/// A receiver's signed report that one of the sender's messages reached a stage.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Stage {
    /// The reporting (receiving) agent's fingerprint.
    pub from: String,
    pub from_agent: String,
    /// The agent whose message this is about (the original sender).
    pub to_agent: String,
    pub conv: String,
    pub msg_id: String,
    pub stage: String,
    pub ts_ms: i64,
    /// The receiver's conversation chain head right after the stage line.
    pub chain_head: String,
}

impl Stage {
    fn to_value(&self) -> Value {
        let mut m = serde_json::Map::new();
        m.insert("v".into(), Value::from(WIRE_VERSION));
        m.insert("kind".into(), "stage".into());
        m.insert("from".into(), self.from.clone().into());
        m.insert("from_agent".into(), self.from_agent.clone().into());
        m.insert("to_agent".into(), self.to_agent.clone().into());
        m.insert("conv".into(), self.conv.clone().into());
        m.insert("msg_id".into(), self.msg_id.clone().into());
        m.insert("stage".into(), self.stage.clone().into());
        m.insert("ts_ms".into(), Value::from(self.ts_ms));
        m.insert("chain_head".into(), self.chain_head.clone().into());
        Value::Object(m)
    }

    fn from_value(v: &Value) -> Result<Self, Refusal> {
        let o = v.as_object().ok_or(Refusal::Malformed("not an object"))?;
        if o.len() != 10 {
            return Err(Refusal::Malformed("unexpected field set"));
        }
        if o.get("v").and_then(Value::as_i64) != Some(WIRE_VERSION) {
            return Err(Refusal::Malformed("unsupported version"));
        }
        if o.get("kind").and_then(Value::as_str) != Some("stage") {
            return Err(Refusal::Malformed("kind is not stage"));
        }
        let s = |k: &'static str| {
            o.get(k)
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or(Refusal::Malformed(k))
        };
        let stage = s("stage")?;
        if !STAGE_ORDER.contains(&stage.as_str()) && !NOTICES.contains(&stage.as_str()) {
            return Err(Refusal::Malformed("unknown stage"));
        }
        Ok(Stage {
            from: s("from")?,
            from_agent: s("from_agent")?,
            to_agent: s("to_agent")?,
            conv: s("conv")?,
            msg_id: s("msg_id")?,
            stage,
            ts_ms: o
                .get("ts_ms")
                .and_then(Value::as_i64)
                .ok_or(Refusal::Malformed("ts_ms"))?,
            chain_head: s("chain_head")?,
        })
    }

    pub fn payload_bytes(&self) -> Vec<u8> {
        canonical_json(&self.to_value()).into_bytes()
    }
}

/// Sign a stage record for `topic` (the original sender's inbox) with the receiver's key.
pub fn sign_stage(id: &Identity, topic: &str, ts_ms: i64, st: &Stage) -> Signed {
    let payload = st.payload_bytes();
    let bytes = canonical_sign_bytes(topic, STAGE_TYPE, &payload, None, ts_ms);
    let sig = id.sign(&bytes);
    Signed {
        topic: topic.to_owned(),
        ts_ms,
        payload,
        signer_pk: id.public_key_hex(),
        sig: hex(&sig.to_bytes()),
    }
}

/// Verify a stage record: canonical, signed by the roster key of its `from`.
pub fn verify_stage(s: &Signed, roster: &Roster) -> Result<Stage, Refusal> {
    let value: Value = serde_json::from_slice(&s.payload)
        .map_err(|_| Refusal::Malformed("payload is not JSON"))?;
    let st = Stage::from_value(&value)?;
    if st.payload_bytes() != s.payload {
        return Err(Refusal::NotCanonical);
    }
    check_signer(s, roster, &st.from, STAGE_TYPE)?;
    Ok(st)
}

// ---- address parser (R-32) -------------------------------------------------

/// A five-level circuit address; every level optional (sparse addresses exist).
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Circuit {
    pub host: Option<String>,
    pub hub: Option<String>,
    pub project: Option<String>,
    pub session: Option<String>,
    pub agent: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum AddrError {
    Unparseable,
    /// A level is set below an unset one; path form cannot express it.
    NotRepresentable,
}

impl Circuit {
    /// Parse path form (`//host/hub/project[/session][/@agent]`, or without the
    /// leading `//` starting at hub) or V9 (`aef::host=..::hub=..::@agent::`).
    pub fn parse(addr: &str) -> Result<Circuit, AddrError> {
        let s = addr.trim();
        if s.is_empty() {
            return Err(AddrError::Unparseable);
        }
        let mut c = Circuit::default();
        if s.contains("::") {
            for piece in s.split("::") {
                let piece = piece.trim();
                if piece.is_empty() || piece == "aef" {
                    continue;
                }
                if let Some(a) = piece.strip_prefix('@') {
                    if c.agent.is_some() || a.is_empty() {
                        return Err(AddrError::Unparseable);
                    }
                    c.agent = Some(a.to_owned());
                    continue;
                }
                let (k, v) = piece.split_once('=').ok_or(AddrError::Unparseable)?;
                let (k, v) = (k.trim(), v.trim());
                let v = if k == "agent" { v.trim_start_matches('@') } else { v };
                let slot = match k {
                    "host" => &mut c.host,
                    "hub" => &mut c.hub,
                    "project" => &mut c.project,
                    "session" => &mut c.session,
                    "agent" => &mut c.agent,
                    _ => return Err(AddrError::Unparseable),
                };
                if v.is_empty() || slot.is_some() {
                    return Err(AddrError::Unparseable);
                }
                *slot = Some(v.to_owned());
            }
            return if c == Circuit::default() {
                Err(AddrError::Unparseable)
            } else {
                Ok(c)
            };
        }
        let (rest, with_host) = match s.strip_prefix("//") {
            Some(r) => (r, true),
            None => (s, false),
        };
        let mut parts: Vec<&str> = rest.split('/').collect();
        while parts.last() == Some(&"") {
            parts.pop();
        }
        if parts.is_empty() || parts.contains(&"") {
            return Err(AddrError::Unparseable);
        }
        let mut idx = 0;
        for p in parts {
            if let Some(a) = p.strip_prefix('@') {
                if c.agent.is_some() || a.is_empty() {
                    return Err(AddrError::Unparseable);
                }
                c.agent = Some(a.to_owned());
                continue;
            }
            if c.agent.is_some() {
                return Err(AddrError::Unparseable);
            }
            let v = Some(p.to_owned());
            match (with_host, idx) {
                (true, 0) => c.host = v,
                (true, 1) | (false, 0) => c.hub = v,
                (true, 2) | (false, 1) => c.project = v,
                _ if c.session.is_none() => c.session = v,
                _ => return Err(AddrError::Unparseable),
            }
            idx += 1;
        }
        Ok(c)
    }

    /// Write path form. Fails when a level is set below an unset one.
    pub fn to_path(&self) -> Result<String, AddrError> {
        let levels = [&self.host, &self.hub, &self.project, &self.session];
        let first = levels.iter().position(|l| l.is_some());
        let mut out = String::new();
        if let Some(first) = first {
            // Path form starts at host (`//`) or at hub (no prefix); project and
            // session alone cannot be positioned.
            if first == 0 {
                out.push_str("//");
            } else if first != 1 {
                return Err(AddrError::NotRepresentable);
            }
            let mut gap = false;
            let mut parts = Vec::new();
            for l in &levels[first..] {
                match l {
                    Some(v) if !gap => parts.push(v.as_str()),
                    Some(_) => return Err(AddrError::NotRepresentable),
                    None => gap = true,
                }
            }
            out.push_str(&parts.join("/"));
        } else if self.agent.is_none() {
            return Err(AddrError::NotRepresentable);
        }
        if let Some(a) = &self.agent {
            if !out.is_empty() {
                out.push('/');
            }
            out.push('@');
            out.push_str(a);
        }
        Ok(out)
    }
}

#[cfg(test)]
mod wire_tests {
    use super::*;

    fn ident(tag: u8) -> Identity {
        let dir = std::env::temp_dir().join(format!(
            "wire-seed-{}-{}-{}",
            std::process::id(),
            tag,
            mint_msg_id()
        ));
        std::fs::create_dir_all(&dir).unwrap();
        let p = dir.join("seed");
        std::fs::write(&p, [tag; 32]).unwrap();
        let id = Identity::load_or_create_from_file(&p).unwrap();
        let _ = std::fs::remove_dir_all(&dir);
        id
    }

    fn msg_from(id: &Identity, to_instance: &str) -> Msg {
        Msg {
            from: id.fingerprint().to_owned(),
            from_agent: "alice".into(),
            from_instance: "inst-a".into(),
            to_agent: "bob".into(),
            to_instance: to_instance.into(),
            conv: "conv-1".into(),
            msg_id: "00112233445566778899aabbccddeeff".into(),
            seq: 7,
            priority: "normal".into(),
            ts_ms: 1_700_000_000_123,
            body: "hello \"bob\"\nline2 é".into(),
        }
    }

    /// SI-27 (P9): a message of another wire version is refused with a reason,
    /// never read as this version.
    #[test]
    fn wire_unknown_version_is_refused_with_a_reason() {
        let id = Identity::generate();
        let mut v = msg_from(&id, "inst-b").to_value();
        assert!(Msg::from_value(&v).is_ok());
        v["v"] = Value::from(WIRE_VERSION + 1);
        assert_eq!(Msg::from_value(&v), Err(Refusal::Malformed("unsupported version")));
        v.as_object_mut().unwrap().remove("v");
        assert!(Msg::from_value(&v).is_err(), "a message without a version is refused too");
    }

    fn setup() -> (Identity, Roster) {
        let a = ident(1);
        let mut r = Roster::new();
        r.add(a.verifying_key());
        (a, r)
    }

    #[test]
    fn wire_round_trip_verifies() {
        let (a, r) = setup();
        let m = msg_from(&a, "inst-b");
        let s = sign(&a, "inbox:b", 42, &m);
        assert_eq!(verify(&s, &r, "inst-b"), Ok(m.clone()));
        assert_eq!(m.digest().len(), 64);
        assert_eq!(m.digest(), digest_hex(&s.payload));
    }

    #[test]
    fn wire_payload_is_sorted_compact() {
        let (a, _) = setup();
        let p = String::from_utf8(msg_from(&a, "i").payload_bytes()).unwrap();
        assert!(p.starts_with("{\"body\":"));
        assert!(!p.contains(": ") && !p.contains(", "));
        assert!(p.contains("\"v\":1"));
    }

    #[test]
    fn wire_altered_payload_refused() {
        let (a, r) = setup();
        let mut s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-b"));
        let text = String::from_utf8(s.payload.clone()).unwrap();
        s.payload = text.replace("hello", "hellp").into_bytes();
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::BadSignature));
        // Altered transport fields are caught the same way.
        let mut s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-b"));
        s.ts_ms += 1;
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::BadSignature));
        let mut s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-b"));
        s.topic = "inbox:c".into();
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::BadSignature));
    }

    #[test]
    fn wire_wrong_key_refused() {
        // A sibling signs but claims to be A: its key does not hash to `from`.
        let (a, r) = setup();
        let sibling = ident(2);
        let s = sign(&sibling, "inbox:b", 42, &msg_from(&a, "inst-b"));
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::FromMismatch));
        // Right claimed key but a forged signature.
        let mut s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-b"));
        s.sig = sign(&sibling, "inbox:b", 42, &msg_from(&a, "inst-b")).sig;
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::BadSignature));
    }

    #[test]
    fn wire_shared_key_refused() {
        // The shared host key is honest about who it is but is not rostered.
        let (_a, r) = setup();
        let host = ident(9);
        let s = sign(&host, "inbox:b", 42, &msg_from(&host, "inst-b"));
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::NotInRoster));
    }

    #[test]
    fn wire_wrong_instance_refused() {
        let (a, r) = setup();
        let s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-OTHER"));
        assert_eq!(
            verify(&s, &r, "inst-b"),
            Err(Refusal::WrongInstance {
                want: "inst-b".into(),
                got: "inst-OTHER".into()
            })
        );
    }

    #[test]
    fn wire_reasons_are_distinct() {
        let all = [
            Refusal::NotCanonical,
            Refusal::FromMismatch,
            Refusal::NotInRoster,
            Refusal::KeyMismatch,
            Refusal::BadSignature,
            Refusal::WrongInstance { want: "a".into(), got: "b".into() },
        ];
        let texts: std::collections::BTreeSet<String> =
            all.iter().map(|r| r.to_string()).collect();
        assert_eq!(texts.len(), all.len());
    }

    #[test]
    fn wire_non_canonical_and_malformed_refused() {
        let (a, r) = setup();
        let mut s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-b"));
        s.payload = serde_json::to_vec_pretty(
            &serde_json::from_slice::<Value>(&s.payload).unwrap(),
        )
        .unwrap();
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::NotCanonical));
        s.payload = b"not json".to_vec();
        assert!(matches!(verify(&s, &r, "inst-b"), Err(Refusal::Malformed(_))));
        s.payload = b"{\"v\":2}".to_vec();
        assert!(matches!(verify(&s, &r, "inst-b"), Err(Refusal::Malformed(_))));
    }

    #[test]
    fn wire_roster_key_mismatch_refused() {
        // Roster holds a different key under A's fingerprint slot.
        let (a, _) = setup();
        let other = ident(3);
        let mut r = Roster::new();
        r.0.insert(a.fingerprint().to_owned(), other.verifying_key());
        let s = sign(&a, "inbox:b", 42, &msg_from(&a, "inst-b"));
        assert_eq!(verify(&s, &r, "inst-b"), Err(Refusal::KeyMismatch));
    }

    #[test]
    fn wire_msg_ids_are_random_128_bit_hex() {
        let (x, y) = (mint_msg_id(), mint_msg_id());
        assert_eq!(x.len(), 32);
        assert!(x.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(x, y);
    }

    #[test]
    fn wire_golden_vector() {
        let a = ident(1);
        let m = msg_from(&a, "inst-b");
        let s = sign(&a, "inbox:b", 1_700_000_000_999, &m);
        let got = serde_json::json!({
            "topic": s.topic,
            "ts_ms": s.ts_ms,
            "signer_pk": s.signer_pk,
            "payload": String::from_utf8(s.payload.clone()).unwrap(),
            "digest": m.digest(),
            "sig": s.sig,
        });
        let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/golden/wire_v1.json");
        if std::env::var_os("WIRE_GOLDEN_UPDATE").is_some() {
            std::fs::create_dir_all(path.parent().unwrap()).unwrap();
            std::fs::write(&path, serde_json::to_string_pretty(&got).unwrap() + "\n").unwrap();
        }
        let want: Value =
            serde_json::from_str(&std::fs::read_to_string(&path).expect("golden file")).unwrap();
        assert_eq!(got, want, "wire format changed: golden vector differs");
    }

    #[test]
    fn wire_address_both_grammars_same_levels() {
        let path = Circuit::parse("//host.example/hub1/pid-abc/sess1/@main").unwrap();
        let v9 = Circuit::parse(
            "aef::host=host.example::hub=hub1::project=pid-abc::session=sess1::@main::",
        )
        .unwrap();
        assert_eq!(path, v9);
        assert_eq!(path.agent.as_deref(), Some("main"));
        assert_eq!(path.session.as_deref(), Some("sess1"));
        assert_eq!(v9.to_path().unwrap(), "//host.example/hub1/pid-abc/sess1/@main");
    }

    #[test]
    fn wire_address_short_forms_and_errors() {
        let c = Circuit::parse("hub1/pid-abc/@main").unwrap();
        assert_eq!(c.host, None);
        assert_eq!(c.to_path().unwrap(), "hub1/pid-abc/@main");
        let c = Circuit::parse("hub1/pid-abc/sess").unwrap();
        assert_eq!(c.session.as_deref(), Some("sess"));
        for bad in ["", "//", "a//b", "hub/@x/more", "aef::bogus=1::", "aef::host=::"] {
            assert_eq!(Circuit::parse(bad), Err(AddrError::Unparseable), "{bad}");
        }
        // Sparse V9 that path form cannot carry.
        let c = Circuit::parse("aef::project=p::@a::").unwrap();
        assert_eq!(c.to_path(), Err(AddrError::NotRepresentable));
    }
}
