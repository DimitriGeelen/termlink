//! T-3703 (arc-011 P-15, B-5; AL-20): canonical form, digest and signature of signed hub state.
//!
//! 06a 2.4: a signed JSON object verifies identically whatever its byte encoding, and a signature made for one
//! purpose never verifies for another. RFC 8785 (JCS) on a restricted value set, SHA-256 of the canonical bytes,
//! ed25519 over the domain-separated string `<schema>\n<hex digest>`.
//!
//! 1. [`parse_strict`] is the strict-JSON gate (FCW-88): valid UTF-8, at most 1 MiB, depth at most 8, no duplicate
//!    key at any level, numbers only integers in [-(2^53-1), 2^53-1] written without fraction, exponent or `-0`.
//!    It is its own parser on purpose: serde_json keeps the last of two duplicate keys silently.
//! 2. [`canonical`] drops the top-level `digest` and `signature` and serialises by RFC 8785: members sorted by the
//!    UTF-16 code units of their names, no insignificant whitespace, minimal string escapes (lower-case `\u00xx`
//!    for other controls), integers in shortest decimal form. No Unicode normalisation (RFC 8785 applies none).
//! 3. [`verify`] recomputes the digest (never trusts the object's), requires it equal to `digest` as written, and
//!    verifies `signature` (128 lower-case hex) over `<schema>\n<digest>` with the expected key (FCW-89).
//!
//! The fixture is `tests/fixtures/signed-state-vectors.json` (06a 2.4.c.1 plus V-REQ-1).

use ed25519_dalek::{Signature, Signer, SigningKey, VerifyingKey};
use sha2::{Digest, Sha256};
use std::fmt;

/// Largest accepted document (FCW-88).
pub const MAX_BYTES: usize = 1 << 20;
/// Deepest accepted nesting of arrays and objects (FCW-88); the top-level object is depth 1.
pub const MAX_DEPTH: usize = 8;
/// Largest integer magnitude (2^53 - 1).
pub const MAX_INT: i64 = (1i64 << 53) - 1;

/// A JSON value of the restricted set. Object members keep their order as parsed; [`canonical`] sorts.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum J {
    Null,
    Bool(bool),
    Int(i64),
    Str(String),
    Arr(Vec<J>),
    Obj(Vec<(String, J)>),
}

/// Why a document or a signature was refused.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Refused {
    TooLarge,
    Utf8,
    Syntax(usize),
    Duplicate(String),
    Depth,
    Number(usize),
    NotObject,
    Field(&'static str),
    Schema,
    DigestMismatch,
    BadSignature,
}

impl fmt::Display for Refused {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Refused::TooLarge => write!(f, "larger than {MAX_BYTES} bytes"),
            Refused::Utf8 => write!(f, "not valid UTF-8"),
            Refused::Syntax(p) => write!(f, "not JSON at byte {p}"),
            Refused::Duplicate(k) => write!(f, "duplicate key {k:?}"),
            Refused::Depth => write!(f, "nested deeper than {MAX_DEPTH}"),
            Refused::Number(p) => write!(f, "number at byte {p} is not an integer in range"),
            Refused::NotObject => write!(f, "not a JSON object"),
            Refused::Field(n) => write!(f, "field {n} missing or of the wrong type"),
            Refused::Schema => write!(f, "unexpected schema"),
            Refused::DigestMismatch => write!(f, "digest does not recompute"),
            Refused::BadSignature => write!(f, "signature does not verify"),
        }
    }
}

impl std::error::Error for Refused {}

impl J {
    /// The member `name` of an object, if any.
    pub fn get(&self, name: &str) -> Option<&J> {
        match self {
            J::Obj(m) => m.iter().find(|(k, _)| k == name).map(|(_, v)| v),
            _ => None,
        }
    }

    pub fn as_str(&self) -> Option<&str> {
        match self {
            J::Str(s) => Some(s),
            _ => None,
        }
    }

    pub fn as_int(&self) -> Option<i64> {
        match self {
            J::Int(i) => Some(*i),
            _ => None,
        }
    }

    /// Set (or replace) a top-level member; `self` must be an object.
    pub fn set(&mut self, name: &str, v: J) {
        if let J::Obj(m) = self {
            if let Some(slot) = m.iter_mut().find(|(k, _)| k == name) {
                slot.1 = v;
            } else {
                m.push((name.to_string(), v));
            }
        }
    }

    /// From a serde_json value built by a producer; refuses floats, out-of-range integers and over-deep values.
    pub fn from_serde(v: &serde_json::Value) -> Result<J, Refused> {
        fn go(v: &serde_json::Value, depth: usize) -> Result<J, Refused> {
            use serde_json::Value as V;
            Ok(match v {
                V::Null => J::Null,
                V::Bool(b) => J::Bool(*b),
                V::Number(n) => {
                    let i = n.as_i64().ok_or(Refused::Number(0))?;
                    if !(-MAX_INT..=MAX_INT).contains(&i) {
                        return Err(Refused::Number(0));
                    }
                    J::Int(i)
                }
                V::String(s) => J::Str(s.clone()),
                V::Array(a) => {
                    if depth + 1 > MAX_DEPTH {
                        return Err(Refused::Depth);
                    }
                    J::Arr(a.iter().map(|x| go(x, depth + 1)).collect::<Result<_, _>>()?)
                }
                V::Object(o) => {
                    if depth + 1 > MAX_DEPTH {
                        return Err(Refused::Depth);
                    }
                    J::Obj(o.iter().map(|(k, x)| Ok((k.clone(), go(x, depth + 1)?))).collect::<Result<_, _>>()?)
                }
            })
        }
        go(v, 0)
    }

    /// To a serde_json value (for RPC answers). Member order is not significant there.
    pub fn to_serde(&self) -> serde_json::Value {
        use serde_json::Value as V;
        match self {
            J::Null => V::Null,
            J::Bool(b) => V::Bool(*b),
            J::Int(i) => V::from(*i),
            J::Str(s) => V::String(s.clone()),
            J::Arr(a) => V::Array(a.iter().map(J::to_serde).collect()),
            J::Obj(m) => V::Object(m.iter().map(|(k, v)| (k.clone(), v.to_serde())).collect()),
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// Strict parser (FCW-88)

/// Parse `bytes` through the strict-JSON gate.
pub fn parse_strict(bytes: &[u8]) -> Result<J, Refused> {
    if bytes.len() > MAX_BYTES {
        return Err(Refused::TooLarge);
    }
    let text = std::str::from_utf8(bytes).map_err(|_| Refused::Utf8)?;
    let mut p = Parser { s: text.as_bytes(), i: 0 };
    p.ws();
    let v = p.value(0)?;
    p.ws();
    if p.i != p.s.len() {
        return Err(Refused::Syntax(p.i));
    }
    Ok(v)
}

struct Parser<'a> {
    s: &'a [u8],
    i: usize,
}

impl Parser<'_> {
    fn ws(&mut self) {
        while self.i < self.s.len() && matches!(self.s[self.i], b' ' | b'\t' | b'\n' | b'\r') {
            self.i += 1;
        }
    }

    fn peek(&self) -> Option<u8> {
        self.s.get(self.i).copied()
    }

    fn eat(&mut self, c: u8) -> Result<(), Refused> {
        if self.peek() == Some(c) {
            self.i += 1;
            Ok(())
        } else {
            Err(Refused::Syntax(self.i))
        }
    }

    fn lit(&mut self, word: &[u8], v: J) -> Result<J, Refused> {
        if self.s[self.i..].starts_with(word) {
            self.i += word.len();
            Ok(v)
        } else {
            Err(Refused::Syntax(self.i))
        }
    }

    fn value(&mut self, depth: usize) -> Result<J, Refused> {
        match self.peek() {
            Some(b'{') => self.object(depth + 1),
            Some(b'[') => self.array(depth + 1),
            Some(b'"') => Ok(J::Str(self.string()?)),
            Some(b't') => self.lit(b"true", J::Bool(true)),
            Some(b'f') => self.lit(b"false", J::Bool(false)),
            Some(b'n') => self.lit(b"null", J::Null),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(Refused::Syntax(self.i)),
        }
    }

    fn object(&mut self, depth: usize) -> Result<J, Refused> {
        if depth > MAX_DEPTH {
            return Err(Refused::Depth);
        }
        self.eat(b'{')?;
        let mut m: Vec<(String, J)> = Vec::new();
        self.ws();
        if self.peek() == Some(b'}') {
            self.i += 1;
            return Ok(J::Obj(m));
        }
        loop {
            self.ws();
            if self.peek() != Some(b'"') {
                return Err(Refused::Syntax(self.i));
            }
            let k = self.string()?;
            if m.iter().any(|(x, _)| *x == k) {
                return Err(Refused::Duplicate(k));
            }
            self.ws();
            self.eat(b':')?;
            self.ws();
            let v = self.value(depth)?;
            m.push((k, v));
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b'}') => {
                    self.i += 1;
                    return Ok(J::Obj(m));
                }
                _ => return Err(Refused::Syntax(self.i)),
            }
        }
    }

    fn array(&mut self, depth: usize) -> Result<J, Refused> {
        if depth > MAX_DEPTH {
            return Err(Refused::Depth);
        }
        self.eat(b'[')?;
        let mut a = Vec::new();
        self.ws();
        if self.peek() == Some(b']') {
            self.i += 1;
            return Ok(J::Arr(a));
        }
        loop {
            self.ws();
            a.push(self.value(depth)?);
            self.ws();
            match self.peek() {
                Some(b',') => self.i += 1,
                Some(b']') => {
                    self.i += 1;
                    return Ok(J::Arr(a));
                }
                _ => return Err(Refused::Syntax(self.i)),
            }
        }
    }

    fn number(&mut self) -> Result<J, Refused> {
        let start = self.i;
        let neg = self.peek() == Some(b'-');
        if neg {
            self.i += 1;
        }
        let ds = self.i;
        while matches!(self.peek(), Some(b'0'..=b'9')) {
            self.i += 1;
        }
        let digits = &self.s[ds..self.i];
        if digits.is_empty() {
            return Err(Refused::Syntax(start));
        }
        // A fraction or an exponent makes it a non-integer form, refused even when its value is whole (V-NEG-1).
        if matches!(self.peek(), Some(b'.' | b'e' | b'E')) {
            return Err(Refused::Number(start));
        }
        // Leading zero is not JSON; `-0` is refused (06a 2.4.c step 1).
        if digits.len() > 1 && digits[0] == b'0' {
            return Err(Refused::Syntax(start));
        }
        if neg && digits == b"0" {
            return Err(Refused::Number(start));
        }
        if digits.len() > 16 {
            return Err(Refused::Number(start));
        }
        let mag: i64 = std::str::from_utf8(digits)
            .ok()
            .and_then(|t| t.parse().ok())
            .ok_or(Refused::Number(start))?;
        if mag > MAX_INT {
            return Err(Refused::Number(start));
        }
        Ok(J::Int(if neg { -mag } else { mag }))
    }

    fn hex4(&mut self) -> Result<u32, Refused> {
        let h = self.s.get(self.i..self.i + 4).ok_or(Refused::Syntax(self.i))?;
        let t = std::str::from_utf8(h).map_err(|_| Refused::Syntax(self.i))?;
        if !t.bytes().all(|c| c.is_ascii_hexdigit()) {
            return Err(Refused::Syntax(self.i));
        }
        let v = u32::from_str_radix(t, 16).map_err(|_| Refused::Syntax(self.i))?;
        self.i += 4;
        Ok(v)
    }

    fn string(&mut self) -> Result<String, Refused> {
        self.eat(b'"')?;
        let mut out = String::new();
        loop {
            let at = self.i;
            let c = self.peek().ok_or(Refused::Syntax(at))?;
            match c {
                b'"' => {
                    self.i += 1;
                    return Ok(out);
                }
                b'\\' => {
                    self.i += 1;
                    let e = self.peek().ok_or(Refused::Syntax(self.i))?;
                    self.i += 1;
                    match e {
                        b'"' => out.push('"'),
                        b'\\' => out.push('\\'),
                        b'/' => out.push('/'),
                        b'b' => out.push('\u{8}'),
                        b'f' => out.push('\u{c}'),
                        b'n' => out.push('\n'),
                        b'r' => out.push('\r'),
                        b't' => out.push('\t'),
                        b'u' => {
                            let hi = self.hex4()?;
                            let cp = if (0xD800..0xDC00).contains(&hi) {
                                // A high surrogate must be followed by an escaped low surrogate.
                                if self.s.get(self.i..self.i + 2) != Some(b"\\u") {
                                    return Err(Refused::Syntax(self.i));
                                }
                                self.i += 2;
                                let lo = self.hex4()?;
                                if !(0xDC00..0xE000).contains(&lo) {
                                    return Err(Refused::Syntax(self.i));
                                }
                                0x10000 + ((hi - 0xD800) << 10) + (lo - 0xDC00)
                            } else if (0xDC00..0xE000).contains(&hi) {
                                return Err(Refused::Syntax(self.i));
                            } else {
                                hi
                            };
                            out.push(char::from_u32(cp).ok_or(Refused::Syntax(self.i))?);
                        }
                        _ => return Err(Refused::Syntax(at)),
                    }
                }
                0x00..=0x1f => return Err(Refused::Syntax(at)),
                _ => {
                    // The input is valid UTF-8 (checked up front): copy one whole character.
                    let rest = std::str::from_utf8(&self.s[self.i..]).map_err(|_| Refused::Utf8)?;
                    let ch = rest.chars().next().ok_or(Refused::Syntax(at))?;
                    out.push(ch);
                    self.i += ch.len_utf8();
                }
            }
        }
    }
}

// ---------------------------------------------------------------------------------------------------------------
// RFC 8785 serialisation, digest, sign, verify

fn write_str(out: &mut Vec<u8>, s: &str) {
    out.push(b'"');
    for ch in s.chars() {
        match ch {
            '"' => out.extend_from_slice(b"\\\""),
            '\\' => out.extend_from_slice(b"\\\\"),
            '\u{8}' => out.extend_from_slice(b"\\b"),
            '\u{c}' => out.extend_from_slice(b"\\f"),
            '\n' => out.extend_from_slice(b"\\n"),
            '\r' => out.extend_from_slice(b"\\r"),
            '\t' => out.extend_from_slice(b"\\t"),
            c if (c as u32) < 0x20 => out.extend_from_slice(format!("\\u{:04x}", c as u32).as_bytes()),
            c => {
                let mut b = [0u8; 4];
                out.extend_from_slice(c.encode_utf8(&mut b).as_bytes());
            }
        }
    }
    out.push(b'"');
}

fn write_value(out: &mut Vec<u8>, v: &J) {
    match v {
        J::Null => out.extend_from_slice(b"null"),
        J::Bool(true) => out.extend_from_slice(b"true"),
        J::Bool(false) => out.extend_from_slice(b"false"),
        J::Int(i) => out.extend_from_slice(i.to_string().as_bytes()),
        J::Str(s) => write_str(out, s),
        J::Arr(a) => {
            out.push(b'[');
            for (n, x) in a.iter().enumerate() {
                if n > 0 {
                    out.push(b',');
                }
                write_value(out, x);
            }
            out.push(b']');
        }
        J::Obj(m) => write_members(out, m.iter().collect()),
    }
}

fn write_members(out: &mut Vec<u8>, mut m: Vec<&(String, J)>) {
    // RFC 8785 3.2.3: sort by the UTF-16 code units of the names.
    m.sort_by(|a, b| a.0.encode_utf16().cmp(b.0.encode_utf16()));
    out.push(b'{');
    for (n, (k, x)) in m.into_iter().enumerate() {
        if n > 0 {
            out.push(b',');
        }
        write_str(out, k);
        out.push(b':');
        write_value(out, x);
    }
    out.push(b'}');
}

/// Schemas whose `digest` member is content, not the object's own digest (T-3703 design gap 19): the revocation
/// head names the list's digest (V-HEAD-1 is 151 canonical bytes WITH it). Such an object carries no self-digest;
/// its signature covers `<schema>\n<recomputed digest>` and the verifier always recomputes.
pub const CONTENT_DIGEST_SCHEMAS: &[&str] = &["termlink.revocation-head/1"];

fn content_digest(obj: &J) -> bool {
    obj.get("schema").and_then(J::as_str).is_some_and(|s| CONTENT_DIGEST_SCHEMAS.contains(&s))
}

/// Canonical bytes of a top-level object without its `signature` member and, unless the schema names `digest` as
/// content, without its own `digest`.
pub fn canonical(obj: &J) -> Result<Vec<u8>, Refused> {
    let J::Obj(m) = obj else { return Err(Refused::NotObject) };
    let keep_digest = content_digest(obj);
    let mut out = Vec::new();
    write_members(&mut out, m.iter().filter(|(k, _)| k != "signature" && (keep_digest || k != "digest")).collect());
    Ok(out)
}

/// Lower-case hex SHA-256 of [`canonical`].
pub fn digest(obj: &J) -> Result<String, Refused> {
    Ok(hex(&Sha256::digest(canonical(obj)?)))
}

fn schema_of(obj: &J) -> Result<&str, Refused> {
    obj.get("schema").and_then(J::as_str).ok_or(Refused::Field("schema"))
}

/// The bytes the signature covers: `<schema>\n<digest>` (the domain prefix, FP 10.3.2.c).
pub fn signing_input(schema: &str, digest_hex: &str) -> Vec<u8> {
    format!("{schema}\n{digest_hex}").into_bytes()
}

/// Return `obj` with `digest` and `signature` set by `key`.
pub fn seal(key: &SigningKey, mut obj: J) -> Result<J, Refused> {
    let d = digest(&obj)?;
    let sig = key.sign(&signing_input(schema_of(&obj)?, &d));
    if !content_digest(&obj) {
        obj.set("digest", J::Str(d));
    }
    obj.set("signature", J::Str(hex(&sig.to_bytes())));
    Ok(obj)
}

/// FCW-89: the digest recomputes and equals the one written; the signature verifies over `<schema>\n<digest>`
/// with `key`. `want_schema`, when given, must equal the object's schema (a list never verifies as a head).
pub fn verify(key: &VerifyingKey, obj: &J, want_schema: Option<&str>) -> Result<(), Refused> {
    let schema = schema_of(obj)?;
    if want_schema.is_some_and(|w| w != schema) {
        return Err(Refused::Schema);
    }
    let recomputed = digest(obj)?;
    if !content_digest(obj) {
        let written = obj.get("digest").and_then(J::as_str).ok_or(Refused::Field("digest"))?;
        if recomputed != written {
            return Err(Refused::DigestMismatch);
        }
    }
    let sig_hex = obj.get("signature").and_then(J::as_str).ok_or(Refused::Field("signature"))?;
    let sig: [u8; 64] = unhex_lower(sig_hex)
        .and_then(|b| b.try_into().ok())
        .ok_or(Refused::Field("signature"))?;
    key.verify_strict(&signing_input(schema, &recomputed), &Signature::from_bytes(&sig))
        .map_err(|_| Refused::BadSignature)
}

/// Strict parse then [`verify`]: the one call a receiver of signed hub state makes.
pub fn parse_and_verify(bytes: &[u8], key: &VerifyingKey, want_schema: &str) -> Result<J, Refused> {
    let obj = parse_strict(bytes)?;
    verify(key, &obj, Some(want_schema))?;
    Ok(obj)
}

/// `ed25519:` + lower-case hex SHA-256 of the 32-byte public key (AL-18 2.2.h; revocation entry `key`).
pub fn key_fingerprint(key: &VerifyingKey) -> String {
    format!("ed25519:{}", hex(&Sha256::digest(key.as_bytes())))
}

pub fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

/// Lower-case hex only (an upper-case signature or digest is a different string, refused).
pub fn unhex_lower(s: &str) -> Option<Vec<u8>> {
    if s.len() % 2 != 0 || !s.bytes().all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)) {
        return None;
    }
    (0..s.len()).step_by(2).map(|i| u8::from_str_radix(&s[i..i + 2], 16).ok()).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn obj(t: &str) -> J {
        parse_strict(t.as_bytes()).unwrap()
    }

    #[test]
    fn strict_gate_refuses_the_fcw_88_classes() {
        assert!(matches!(parse_strict(br#"{"a":1.0}"#), Err(Refused::Number(_))));
        assert!(matches!(parse_strict(br#"{"a":1e0}"#), Err(Refused::Number(_))));
        assert!(matches!(parse_strict(br#"{"a":-0}"#), Err(Refused::Number(_))));
        assert!(matches!(parse_strict(br#"{"a":9007199254740992}"#), Err(Refused::Number(_))));
        assert_eq!(parse_strict(br#"{"a":-9007199254740991}"#).unwrap().get("a"), Some(&J::Int(-MAX_INT)));
        assert!(matches!(parse_strict(br#"{"a":1,"a":1}"#), Err(Refused::Duplicate(_))));
        assert!(matches!(parse_strict(br#"{"x":{"a":1,"a":2}}"#), Err(Refused::Duplicate(_))));
        assert!(matches!(parse_strict(b"{\"a\":\"\xff\"}"), Err(Refused::Utf8)));
        assert!(matches!(parse_strict(br#"{"a":"\ud800"}"#), Err(Refused::Syntax(_))));
        assert!(matches!(parse_strict(br#"{"a":01}"#), Err(Refused::Syntax(_))));
        assert!(matches!(parse_strict(br#"{"a":1} x"#), Err(Refused::Syntax(_))));
        let deep8 = "[".repeat(7) + "1" + &"]".repeat(7);
        assert!(parse_strict(format!("{{\"a\":{deep8}}}").as_bytes()).is_ok(), "depth 8 accepted");
        let deep9 = "[".repeat(8) + "1" + &"]".repeat(8);
        assert!(matches!(parse_strict(format!("{{\"a\":{deep9}}}").as_bytes()), Err(Refused::Depth)));
        let big = format!("{{\"a\":\"{}\"}}", "x".repeat(MAX_BYTES));
        assert!(matches!(parse_strict(big.as_bytes()), Err(Refused::TooLarge)));
    }

    #[test]
    fn canonical_sorts_by_utf16_and_escapes_minimally() {
        // U+FF61 (BMP, high) sorts after U+1F600 (astral) in UTF-16 order, the opposite of code-point order.
        let o = J::Obj(vec![("\u{ff61}".into(), J::Int(1)), ("\u{1f600}".into(), J::Int(2))]);
        assert_eq!(String::from_utf8(canonical(&o).unwrap()).unwrap(), "{\"\u{1f600}\":2,\"\u{ff61}\":1}");
        let s = obj(r#"{"s":"a\u0001\/é\n","digest":"x","signature":"y"}"#);
        assert_eq!(String::from_utf8(canonical(&s).unwrap()).unwrap(), "{\"s\":\"a\\u0001/\u{e9}\\n\"}");
    }

    #[test]
    fn seal_then_verify_and_domain_separation() {
        let k = SigningKey::from_bytes(&[7u8; 32]);
        let o = seal(&k, obj(r#"{"schema":"termlink.revocation/1","seq":1}"#)).unwrap();
        verify(&k.verifying_key(), &o, Some("termlink.revocation/1")).unwrap();
        assert_eq!(verify(&k.verifying_key(), &o, Some("termlink.revocation-head/1")), Err(Refused::Schema));
        let mut moved = o.clone();
        moved.set("schema", J::Str("termlink.revocation-head/1".into()));
        assert!(verify(&k.verifying_key(), &moved, None).is_err(), "a signature for schema A never verifies as B");
        let other = SigningKey::from_bytes(&[8u8; 32]);
        assert_eq!(verify(&other.verifying_key(), &o, None), Err(Refused::BadSignature));
    }
}
