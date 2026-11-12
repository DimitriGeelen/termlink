//! The arc-011 blob wrapper header (TLBLOB, B-11, ARC4 6.4.3/6.4.5, 06a AL-26/AL-28): the bytes every object a
//! non-baseline artifact writer stores must begin with. Format only; the crypto lives in `termlink-sidecar::enckey`.
//!
//! Object layout: `"TLBLOB"` (6) || format version (1) || algorithm id (1) || nonce (24) || AEAD ciphertext + tag.
//! Format version is 2 (06a 2.10.c, CR-6.14: the associated data covers the immutable envelope fields; version 1
//! never shipped and is refused). Algorithm 1 is XChaCha20-Poly1305 with the content key in an X25519 sealed box.
//! A new algorithm needs a new id, never a silent change (06a 2.10.h).

/// Magic bytes at offset 0.
pub const MAGIC: &[u8; 6] = b"TLBLOB";
/// The only format version this build writes and accepts.
pub const VERSION: u8 = 2;
/// XChaCha20-Poly1305, content key in an X25519 sealed box.
pub const ALG_XCHACHA20POLY1305_SEALEDBOX: u8 = 1;
/// Known algorithm ids.
pub const KNOWN_ALGS: &[u8] = &[ALG_XCHACHA20POLY1305_SEALEDBOX];
/// XChaCha20 nonce length.
pub const NONCE_LEN: usize = 24;
/// Poly1305 tag length.
pub const TAG_LEN: usize = 16;
/// Header length: magic, version, algorithm, nonce.
pub const HEADER_LEN: usize = 6 + 1 + 1 + NONCE_LEN;

/// `artifact.put` parameter naming the writer class (AL-28). Absent: one of the baseline writers, accepted as today
/// (NID). Present with any value: the object must carry the wrapper header (FCW-117).
pub const WRITER_PARAM: &str = "writer";
/// The writer class the arc-011 sidecar and `artifact put --encrypt-to` declare.
pub const WRITER_ARC011: &str = "arc011-wrapper";

/// Check that `obj` starts with a complete wrapper header of a known version and algorithm and is long enough to
/// hold an AEAD tag. `Err` carries the refusal reason (FCW-117).
pub fn check_header(obj: &[u8]) -> Result<(), String> {
    if obj.len() < HEADER_LEN + TAG_LEN {
        return Err(format!(
            "unwrapped object: {} bytes is shorter than the TLBLOB header and tag ({} bytes)",
            obj.len(),
            HEADER_LEN + TAG_LEN
        ));
    }
    if &obj[..6] != MAGIC {
        return Err("unwrapped object: does not start with the TLBLOB magic".into());
    }
    if obj[6] != VERSION {
        return Err(format!("unwrapped object: TLBLOB format version {} is not {VERSION}", obj[6]));
    }
    if !KNOWN_ALGS.contains(&obj[7]) {
        return Err(format!("unwrapped object: unknown TLBLOB algorithm id {}", obj[7]));
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn good() -> Vec<u8> {
        let mut v = MAGIC.to_vec();
        v.push(VERSION);
        v.push(ALG_XCHACHA20POLY1305_SEALEDBOX);
        v.extend_from_slice(&[0u8; NONCE_LEN + TAG_LEN]);
        v
    }

    #[test]
    fn header_accepts_v2_alg1() {
        assert!(check_header(&good()).is_ok());
    }

    #[test]
    fn header_refuses_plaintext_short_v1_and_unknown_alg() {
        assert!(check_header(b"hello world, plaintext canary 0123456789abcdef").is_err());
        assert!(check_header(&good()[..HEADER_LEN]).is_err());
        let mut v1 = good();
        v1[6] = 1;
        assert!(check_header(&v1).unwrap_err().contains("version 1"));
        let mut a9 = good();
        a9[7] = 9;
        assert!(check_header(&a9).unwrap_err().contains("algorithm id 9"));
    }
}
