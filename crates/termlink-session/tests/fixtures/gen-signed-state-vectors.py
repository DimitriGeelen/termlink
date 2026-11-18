#!/usr/bin/env python3
"""T-3703 (arc-011 P-15, AL-20): generate signed-state-vectors.json.

Reproduces the vectors of docs/design/interactive-agent-communication-06a-pseudo-code-ph2-hardening.md
2.4.c.1 (first made by the gitignored .context/working/t3355/vectors.py) and adds V-REQ-1 (a self-revocation
request, T-3703 design note gap 8). Key: the public RFC 8032 test key 1, never an estate key.

Run: python3 gen-signed-state-vectors.py > signed-state-vectors.json   (needs PyNaCl)
The JCS here is valid for the restricted value set (strings, integers, objects, arrays, ASCII/BMP keys).
"""
import base64
import hashlib
import json

from nacl.signing import SigningKey

SEED = bytes.fromhex("9d61b19deffd5a60ba844af492ec2cc44449c5697b326919703bac031cae7f60")
sk = SigningKey(SEED)
PK = sk.verify_key.encode()


def jcs(o):
    return json.dumps(o, sort_keys=True, separators=(",", ":"), ensure_ascii=False).encode("utf-8")


def vector(vid, obj, alt_text=None):
    c = jcs(obj)
    d = hashlib.sha256(c).hexdigest()
    schema = obj["schema"]
    v = {
        "id": vid,
        "object": obj,
        "canonical": c.decode("utf-8"),
        "canonical_len": len(c),
        "digest": d,
        "signature": sk.sign((schema + "\n" + d).encode()).signature.hex(),
        "signature_without_domain": sk.sign(d.encode()).signature.hex(),
    }
    if alt_text is not None:
        assert jcs(json.loads(alt_text)) == c
        v["alt_text"] = alt_text
    return v


rev = {"schema": "termlink.revocation/1", "hub_id": "cacc73ea32b121dd", "seq": 7, "issued_at": 1762862400000,
       "prev_digest": "0" * 64,
       "entries": [{"scope": "agent_key",
                    "key": "ed25519:3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29",
                    "project": "010-termlink", "agent": "claude-termlink-alt", "reason": "key exposed",
                    "revoked_at": 1762862399000, "by": "operator", "event": "admin-revoke"}]}
rev_alt = ('{ "seq" : 7, "schema":"termlink.revocation/1", "hub_id":"cacc73ea32b121dd","issued_at":1762862400000,'
           '"prev_digest":"' + "0" * 64 + '","entries":[{"by":"operator","event":"admin-revoke","key":"ed25519:'
           '3b6a27bcceb6a42d62a3a8d02a6f0d73653215771de243a63ac048a18b59da29","project":"010-termlink","agent":'
           '"claude-termlink-\\u0061lt","reason":"key exposed","revoked_at":1762862399000,"scope":"agent_key"}] }')
head = {"schema": "termlink.revocation-head/1", "hub_id": "cacc73ea32b121dd", "seq": 7,
        "digest": hashlib.sha256(jcs(rev)).hexdigest()}
inner = b"<original signed message bytes>"
fwd = {"schema": "termlink.forward/1", "project": "010-termlink", "generation": 12, "hop": 1,
       "via": "claude-termlink-alt.w1-20261013", "inner_topic": "inbox:cacc73ea32b121dd/claude-termlink-alt",
       "inner_sha256": hashlib.sha256(inner).hexdigest(), "inner_b64": base64.b64encode(inner).decode()}
req = {"schema": "termlink.revocation-request/1", "key": "ed25519:" + hashlib.sha256(PK).hexdigest(),
       "public_key": PK.hex(), "project": "010-termlink", "agent": "fx-a", "reason": "retired",
       "at": 1762862400000}

canon_rev = jcs(rev).decode()
out = {
    "generator": "crates/termlink-session/tests/fixtures/gen-signed-state-vectors.py",
    "seed_rfc8032_test1": SEED.hex(),
    "public_key": PK.hex(),
    "vectors": [vector("V-REV-1", rev, rev_alt), vector("V-HEAD-1", head), vector("V-FWD-1", fwd),
                vector("V-REQ-1", req)],
    "negative": [
        {"id": "V-NEG-1a", "text": canon_rev.replace('"seq":7', '"seq":7.0'), "refusal": "number"},
        {"id": "V-NEG-1b", "text": canon_rev.replace('"seq":7', '"seq":7e0'), "refusal": "number"},
        {"id": "V-NEG-2", "text": canon_rev.replace('"seq":7', '"seq":7,"seq":7'), "refusal": "duplicate"},
        {"id": "V-NEG-4", "text": canon_rev.replace('"seq":7', '"seq":9007199254740992'), "refusal": "number"},
    ],
}
print(json.dumps(out, indent=1, ensure_ascii=False))
