# Review bundle: a daily canary is the unnamed second reader of two agents' inboxes

## 1. Facts (verified on the host 2026-11-21)

TermLink is a messaging substrate. Each agent has a hub topic `inbox:<hub-id>/<agent>`. Rule P-12: an agent's inbox has
exactly ONE reader, its own sidecar, proven by an ed25519 signature on each read. The hub records every read of an
`inbox:*` topic in a consumer registry (crate code below). A read without a signature is recorded as agent
`(unnamed)`, strength `claimed`, why `no reader named`. The registry lists readers seen within a 24 h window
(DEFAULT_WINDOW_S = 86400). The operator's deploy script (`runme.sh`, action 37 `p12_live_check`) FAILS when an
inbox's consumer list is anything other than "one (proven)". The sidecar's status also reports NOT CLEAR then.

Registry rows for the two production agent inboxes (claude-termlink, claude-termlink-alt), read 2026-11-21 ~11:15Z:
- `(unnamed)`, instance `local`, claimed, "no reader named", first 2026-11-20T05:52:37Z, last 2026-11-21T06:47:08Z
  (the two inboxes 23 ms apart), actions [read]
- the agent itself, proven, key fingerprint, last a few seconds ago.

`/etc/cron.d/termlink-file-shaped-posts-canary`: `47 7 * * * root cd <repo> && bash scripts/check-file-shaped-posts.sh --quiet >> <log>`
(host TZ CET, so 07:47 CET = 06:47Z). The canary's log mtimes show it running at exactly HH:47:08 local. Its script
(below) lists every channel topic (`termlink channel list --json`) and pages through each with
`termlink channel subscribe <topic> --json --cursor N --limit 1000`, unsigned. It never skips `inbox:` topics. So every
morning it becomes the unnamed second reader of both inboxes, the 24 h window never empties, and action 37 can never pass.

The agent earlier told the operator the unnamed reader was residue of one-off manual reads and would age out by
09:41Z; that was wrong (the 06:47Z read proves a recurring reader).

`inbox.consumers` (the RPC behind `termlink hub inbox consumers <topic>`) only READS the registry; it records nothing
(router excerpt below). Reads are recorded only on the channel read paths (subscribe and similar) and on acks.

Standing operator rules that bear on this:
- "Nothing working is switched off until its replacement is proven in the field, and only with explicit documented
  operator approval." Narrowing a guard's coverage counts.
- Exit codes for checks: 0 healthy, 1 firing, 2 tooling; a tooling error is never reported clean.
- A guard cap: guards are added only by retiring one (not directly relevant unless an option adds a new guard).

## 2. The options the agent put to the operator

- A. The canary skips every `inbox:` topic (one filter line plus a fixture). It no longer checks direct mail to any agent.
- B. (agent's recommendation) The canary skips only inboxes whose consumer list shows a proven single reader: before
  reading an inbox it calls `termlink hub inbox consumers <topic>` (which records nothing) and skips the topic if a
  proven reader owns it. Today that is the two production inboxes; every other inbox stays covered, and a newly bound
  inbox drops out automatically.
- C. Make action 37 tolerate the canary. The canary is unnamed, so it cannot be told apart from an intruder; P-12 would
  be blinded.
- D. Do nothing; action 37 stays red permanently.
The agent's scores (weights: antifragility 9, reliability 7, usability 5, portability 3): A +19, B +25, C -20, D -21.

## 3. scripts/check-file-shaped-posts.sh (whole file)
```python
#!/usr/bin/env bash
# check-file-shaped-posts.sh — files sent as ordinary messages, outside the artifact path (T-3432)
#
# WHY: arc-011 FX-5 ruling C (2026-10-17). Measured on .107: peers already move files through
# channel.post — `document` messages with a `filename`, and files cut into 40 KB `artifact-chunk`
# pieces with `part`/`sha256`. That content lands in the message body, a plaintext field the hub
# keeps readable, and no per-message size cap can stop it (the pieces are small). So the hub
# keeps a 512 KB backstop cap (PLAINTEXT_FIELD_TOO_LARGE), and this canary makes the chunked
# bypass visible: which topics, which message types, which senders. Refusing such posts is a
# later decision, once PH-2 row P15 gives an encrypted way to send a file.
#
# FILE-SHAPED: msg_type document | artifact-chunk | attachment | file-chunk, OR metadata carries
# `filename`, OR both `part` and `sha256`. NOT file-shaped: msg_type `artifact`, the sanctioned
# `termlink file send` path (its writers are held by check-artifact-writers.sh).
#
# FIRES (exit 1) on a file-shaped post NEWER than the acknowledged offset for its topic. The
# existing ones are acknowledged with --ack (recorded per topic in the state file), so the log
# stays meaningful: it fills only when someone sends a new file this way.
#
# SCOPE (T-2680): reads the local hub only, and only the shapes above; a file disguised as a plain
# `note` is not seen. Exit 0 healthy · 1 new file-shaped post(s) · 2 tooling (hub unreachable,
# unparseable output; never a false clean).
# Usage: check-file-shaped-posts.sh [--json] [--quiet] [--ack] [--no-heartbeat]
# Seams: FILE_SHAPED_TEST_DIR (list.json + <topic-slug>.ndjson), FILE_SHAPED_STATE, FILE_SHAPED_HEARTBEAT
set -uo pipefail
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
JSON=0 QUIET=0 ACK=0 HB=1
for a in "$@"; do
    case "$a" in
        --json) JSON=1 ;; --quiet) QUIET=1 ;; --ack) ACK=1 ;; --no-heartbeat) HB=0 ;;
        -h|--help) sed -n 2,24p "$0"; exit 0 ;;
        *) echo "check-file-shaped-posts: unknown arg '$a'" >&2; exit 2 ;;
    esac
done
STATE="${FILE_SHAPED_STATE:-$ROOT/.context/working/.file-shaped-posts.seen}"
HBF="${FILE_SHAPED_HEARTBEAT:-$ROOT/.context/working/.file-shaped-posts-canary.heartbeat}"
# The heartbeat proves the run completed (T-2843), so it is written on exit, not at start.
[ "$HB" = 1 ] && trap 'touch "$HBF" 2>/dev/null' EXIT

FILE_SHAPED_JSON="$JSON" FILE_SHAPED_QUIET="$QUIET" FILE_SHAPED_ACK="$ACK" FILE_SHAPED_STATE="$STATE" python3 - <<'PY'
import json, os, re, subprocess, sys

test_dir = os.environ.get("FILE_SHAPED_TEST_DIR")
state_path = os.environ["FILE_SHAPED_STATE"]
FILE_TYPES = {"document", "artifact-chunk", "attachment", "file-chunk"}

def tooling(msg):
    print(f"check-file-shaped-posts: TOOLING: {msg}", file=sys.stderr); sys.exit(2)

def slug(t): return re.sub(r"[^A-Za-z0-9_.-]", "_", t)

def topics():
    if test_dir:
        raw = open(os.path.join(test_dir, "list.json")).read()
    else:
        try:
            raw = subprocess.run(["termlink", "channel", "list", "--json"], capture_output=True,
                                 text=True, timeout=120).stdout
        except Exception as e: tooling(f"channel list: {e}")
    try: d = json.loads(raw)
    except Exception: tooling("channel list output is not JSON")
    ts = d.get("topics", d) if isinstance(d, dict) else d
    return [(t["name"], t.get("count", 0)) for t in ts if isinstance(t, dict) and "name" in t]

def envelopes(topic):
    if test_dir:
        p = os.path.join(test_dir, slug(topic) + ".ndjson")
        if os.path.exists(p):
            for line in open(p):
                if line.strip(): yield json.loads(line)
        return
    cursor = 0
    while True:
        try:
            out = subprocess.run(["termlink", "channel", "subscribe", topic, "--json", "--cursor",
                                  str(cursor), "--limit", "1000"], capture_output=True, text=True,
                                 timeout=120).stdout
        except Exception as e: tooling(f"subscribe {topic}: {e}")
        rows = []
        for line in out.splitlines():
            try: rows.append(json.loads(line))
            except Exception: continue
        for r in rows:
            if isinstance(r.get("offset"), int): yield r
        offs = [r["offset"] for r in rows if isinstance(r.get("offset"), int)]
        if len(rows) < 1000 or not offs: return
        cursor = max(offs) + 1

def file_shaped(env):
    t = env.get("msg_type") or ""
    if t == "artifact": return None
    md = env.get("metadata") or {}
    if t in FILE_TYPES: return t
    if "filename" in md: return f"{t}+filename"
    if "part" in md and "sha256" in md: return f"{t}+part/sha256"
    return None

try: acked = json.load(open(state_path))
except FileNotFoundError: acked = {}
except Exception: tooling(f"state file unreadable: {state_path}")

report, new_acked = [], {}
for name, count in topics():
    if not count: continue
    found = []
    for env in envelopes(name):
        shape = file_shaped(env)
        if shape:
            md = env.get("metadata") or {}
            found.append({"offset": env["offset"], "shape": shape,
                          "from": md.get("from_project") or env.get("sender_id", "?")[:16]})
    if not found: continue
    newest = max(f["offset"] for f in found)
    new_acked[name] = newest
    fresh = [f for f in found if f["offset"] > acked.get(name, -1)]
    report.append({"topic": name, "total": len(found), "new": len(fresh),
                   "shapes": sorted({f["shape"] for f in found}),
                   "senders": sorted({str(f["from"]) for f in found})})

firing = [r for r in report if r["new"]]
if os.environ["FILE_SHAPED_ACK"] == "1":
    os.makedirs(os.path.dirname(state_path) or ".", exist_ok=True)
    json.dump(new_acked, open(state_path, "w"), indent=1, sort_keys=True)
if os.environ["FILE_SHAPED_JSON"] == "1":
    print(json.dumps({"ok": not firing, "acked_now": os.environ["FILE_SHAPED_ACK"] == "1",
                      "topics": report, "firing": [r["topic"] for r in firing]}))
else:
    quiet = os.environ["FILE_SHAPED_QUIET"] == "1"
    if firing and os.environ["FILE_SHAPED_ACK"] != "1":
        print(f"file-shaped-posts canary: FIRING — new file-shaped message(s) in {len(firing)} topic(s), "
              "outside the artifact path (plaintext on the hub, FX-5):")
        for r in firing:
            print(f"  {r['topic']}: {r['new']} new ({r['total']} total)  shapes={','.join(r['shapes'])}  from={','.join(r['senders'])}")
        print("  Send files with `termlink file send` (the artifact path), and once seen and judged,")
        print("  acknowledge with: bash scripts/check-file-shaped-posts.sh --ack")
    elif not quiet:
        tot = sum(r["total"] for r in report)
        verb = "acknowledged now" if os.environ["FILE_SHAPED_ACK"] == "1" else "all acknowledged"
        print(f"file-shaped-posts canary: healthy — {tot} file-shaped message(s) in {len(report)} topic(s), {verb}")
        for r in report:
            print(f"  {r['topic']}: {r['total']}  shapes={','.join(r['shapes'])}  from={','.join(r['senders'])}")
        print("  Scope: local hub; detects the listed shapes only, not a file disguised as a plain note.")
sys.exit(1 if firing and os.environ["FILE_SHAPED_ACK"] != "1" else 0)
PY
```
## 4. crates/termlink-hub/src/inbox_consumers.rs (lines 1-260, 480-697)
```rust
//! T-3700 (arc-011 P-12, B-2, 06a AL-30): who consumes each inbox, `proven` or `claimed`, and the bind.
//!
//! Every read (`channel.subscribe`), claim (`channel.claim`) and ack (`channel.post` of a `receipt`) on an
//! `inbox:*` topic records its reader. A reader names itself in an optional `reader` object; one with a key
//! card signs it, and is `proven` when the signature, the card and the revocation list all agree
//! (operator ruling E, 2026-11-19). A broken proof is refused, never downgraded. A bound inbox
//! (`hub.inbox_bind`, operator runme) admits only reads proven by the bound agent's bound key (FCW-124).
//! Design: `docs/reports/T-3700-design.md`.

use std::collections::{BTreeMap, BTreeSet, HashMap};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use termlink_protocol::control::error_code;

/// AL-30 2.14.c: entries kept per inbox; the oldest by `last_ms` is evicted with an admin row.
pub const MAX_PER_TOPIC: usize = 16;
/// Bound on the number of inboxes tracked, so caller-chosen topic names cannot grow the table without limit.
pub const MAX_TOPICS: usize = if cfg!(test) { 64 } else { 4096 };
/// A read proof's timestamp must be this close to the hub clock to prove (P-23 skew bound).
pub const PROOF_WINDOW_MS: i64 = 120_000;
/// `last_ms` is persisted when it moved by at least this much since the last write (design 3.2.2).
pub const PERSIST_EVERY_MS: u64 = 60_000;
/// Longest inbox topic recorded or bound (r1 CC 3 / OC 2).
pub const MAX_TOPIC_LEN: usize = 256;
/// Admin rows of one kind written per minute; the rest are counted and reported on the next row (r1 CC 13).
pub const ROWS_PER_MINUTE: u32 = 10;
/// V-11: the default consumer window.
pub const DEFAULT_WINDOW_S: u64 = 86_400;
/// The instance recorded for an ack (a receipt post carries no instance).
pub const ACK_INSTANCE: &str = "-";
/// The agent recorded for a caller that names no reader.
pub const UNNAMED: &str = "(unnamed)";
/// Domain prefix of the read-proof bytes (shared with the sidecar through `termlink-session`).
pub use termlink_session::inbox_read::{READ_DOMAIN, read_sign_bytes};

/// Topics this module applies to.
pub fn applies(topic: &str) -> bool {
    topic.starts_with("inbox:")
}

/// The `reader` object of a read or claim.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ReadProof {
    pub agent: String,
    pub instance: String,
    pub ts_ms: i64,
    pub signer_pk: Option<String>,
    pub sig: Option<String>,
}

/// The params field carrying a read proof. NOT `reader`: that name already means a plain reader NAME on
/// `channel.subscribe` (T-3309 read activity) and old clients send it as a string (regression found live
/// 2026-11-20, fixed T-3700).
pub const PROOF_FIELD: &str = "read_proof";

/// Parse `params.read_proof`. `Ok(None)` when absent; `Err` when present but malformed (refused: a caller that
/// sends a proof object must send a valid one).
pub fn parse_reader(params: &serde_json::Value) -> Result<Option<ReadProof>, String> {
    let Some(r) = params.get(PROOF_FIELD) else { return Ok(None) };
    if r.is_null() {
        return Ok(None);
    }
    if !r.is_object() {
        return Err(format!("{PROOF_FIELD} must be an object"));
    }
    let s = |k: &str| r.get(k).and_then(|v| v.as_str()).map(str::to_string);
    let agent = s("agent").ok_or("reader.agent is missing")?;
    if !crate::key_card::valid_name(&agent) {
        return Err(format!("reader.agent {:?} is not a plain agent name", agent.chars().take(64).collect::<String>()));
    }
    let instance = s("instance").unwrap_or_default();
    // r1 CC 1 / OC 1: `-` is the ack sentinel and an empty instance names nobody; neither may be claimed
    if instance.is_empty() || instance == ACK_INSTANCE || instance.len() > 128 || instance.chars().any(|c| c.is_control()) {
        return Err("reader.instance is missing, is \"-\", is longer than 128 bytes or holds a control character".into());
    }
    let ts_ms = r.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
    let signer_pk = s("signer_pk");
    let sig = s("sig");
    if signer_pk.is_some() != sig.is_some() {
        return Err("reader.signer_pk and reader.sig come together or not at all".into());
    }
    Ok(Some(ReadProof { agent, instance, ts_ms, signer_pk, sig }))
}

/// How strongly the hub knows who a reader is.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Strength {
    Proven,
    Claimed,
}

/// A reader as the hub records it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Reader {
    pub agent: String,
    pub instance: String,
    pub strength: Strength,
    /// Why a reader is only `claimed` (empty when proven).
    pub why: String,
    /// The signing key fingerprint of a proven reader.
    pub key_fp: Option<String>,
}

/// A refusal: code, FCW id, reason.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Refusal {
    pub code: i64,
    pub fcw: &'static str,
    pub reason: String,
}

/// Looks up the signing key (raw 32 bytes) on an agent's card: `Ok(None)` without a card.
pub type CardKeyLookup<'a> = &'a dyn Fn(&str) -> Result<Option<[u8; 32]>, String>;

/// What [`assess`] needs from the hub, injectable for tests.
pub struct Lookups<'a> {
    /// The hub's id; `None` when the identity is not loaded (nothing can be proven).
    pub hub_id: Option<&'a str>,
    /// The signing key (raw 32 bytes) on an agent's card, `Ok(None)` without a card.
    pub card_key: CardKeyLookup<'a>,
    /// The fingerprints on the hub's revocation list.
    pub revoked: &'a dyn Fn() -> Result<BTreeSet<String>, String>,
}

fn claimed(agent: &str, instance: &str, why: &str) -> Reader {
    Reader { agent: agent.into(), instance: instance.into(), strength: Strength::Claimed, why: why.into(), key_fp: None }
}

/// Decide who a read or claim comes from (design 3.1). `transport` names a caller with no reader
/// (`local` or `tcp:<ip>`).
pub fn assess(
    topic: &str,
    method: &str,
    position: u64,
    proof: Option<&ReadProof>,
    transport: &str,
    now_ms: i64,
    l: &Lookups<'_>,
) -> Result<Reader, Refusal> {
    let Some(p) = proof else { return Ok(claimed(UNNAMED, transport, "no reader named")) };
    let (Some(pk_hex), Some(sig_hex)) = (p.signer_pk.as_deref(), p.sig.as_deref()) else {
        return Ok(claimed(&p.agent, &p.instance, "unsigned"));
    };
    let bad = |reason: String| Refusal { code: error_code::CHANNEL_SIGNATURE_INVALID, fcw: "FCW-124", reason };
    let Some(vk) = termlink_session::signed_state::verifying_key_of_hex(pk_hex) else {
        return Err(bad("reader.signer_pk is not a lower-case 32-byte ed25519 public key".into()));
    };
    let Some(sig) = parse_sig(sig_hex) else {
        return Err(bad("reader.sig is not a 64-byte ed25519 signature".into()));
    };
    let Some(hub_id) = l.hub_id else {
        return Ok(claimed(&p.agent, &p.instance, "the hub identity is not loaded"));
    };
    let bytes = read_sign_bytes(hub_id, topic, method, position, p.ts_ms, &p.agent, &p.instance);
    if vk.verify(&bytes, &sig).is_err() {
        return Err(bad(format!("the read signature of {:?} failed verification", p.agent)));
    }
    match (l.card_key)(&p.agent) {
        Ok(Some(k)) if &k == vk.as_bytes() => {}
        Ok(Some(_)) => {
            return Err(Refusal {
                code: error_code::CHANNEL_KEY_CARD_MISMATCH,
                fcw: "FCW-124",
                reason: format!("the read is not signed by the key on agent {:?}'s card", p.agent),
            })
        }
        Ok(None) => return Ok(claimed(&p.agent, &p.instance, "no key card")),
        Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("key cards unreadable: {e}"))),
    }
    let fp = termlink_session::signed_state::key_fingerprint(&vk);
    match (l.revoked)() {
        Ok(keys) if keys.contains(&fp) => {
            return Err(Refusal {
                code: error_code::KEY_REVOKED,
                fcw: "FCW-99",
                reason: format!("agent {:?}'s card key {fp} is on the hub's revocation list", p.agent),
            })
        }
        Ok(_) => {}
        Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("revocation list unreadable: {e}"))),
    }
    if now_ms.abs_diff(p.ts_ms) > PROOF_WINDOW_MS as u64 {
        return Ok(claimed(&p.agent, &p.instance, "clock skew"));
    }
    Ok(Reader { agent: p.agent.clone(), instance: p.instance.clone(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) })
}

/// The reader of a verified receipt post: proven when the signer's key is on a card, not revoked, and the post's
/// `ts` is within the proof window (a replayed old receipt is only claimed, r1 CC 7). An uncarded ack is listed
/// under the payload's claimed `from_agent`, else `(unnamed)` (r1 OC 6).
pub fn assess_ack(
    signer: &VerifyingKey,
    claimed_agent: Option<&str>,
    post_ts_ms: i64,
    now_ms: i64,
    cards: &dyn Fn() -> Result<Vec<(String, [u8; 32])>, String>,
    l: &Lookups<'_>,
) -> Reader {
    let fp = termlink_session::signed_state::key_fingerprint(signer);
    let short = fp.chars().skip(8).take(16).collect::<String>();
    let named = claimed_agent.filter(|a| crate::key_card::valid_name(a)).unwrap_or(UNNAMED).to_string();
    let agent = match cards() {
        Ok(cs) => cs.into_iter().find(|(_, k)| k == signer.as_bytes()).map(|(a, _)| a),
        Err(e) => return claimed(&named, ACK_INSTANCE, &format!("key cards unreadable: {e} (key {short})")),
    };
    let Some(agent) = agent else {
        return claimed(&named, ACK_INSTANCE, &format!("no key card (key {short})"));
    };
    match (l.revoked)() {
        Ok(keys) if keys.contains(&fp) => claimed(&agent, ACK_INSTANCE, "key revoked"),
        Ok(_) if now_ms.abs_diff(post_ts_ms) > PROOF_WINDOW_MS as u64 => claimed(&agent, ACK_INSTANCE, "stale ack"),
        Ok(_) => Reader { agent, instance: ACK_INSTANCE.into(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) },
        Err(e) => claimed(&agent, ACK_INSTANCE, &format!("revocation list unreadable: {e}")),
    }
}

fn parse_sig(hex: &str) -> Option<Signature> {
    let b = termlink_session::signed_state::unhex_lower(hex)?;
    let arr: [u8; 64] = b.try_into().ok()?;
    Some(Signature::from_bytes(&arr))
}

/// A consumer action (FCW-209: every one records).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Action {
    Read,
    Claim,
    Ack,
}

impl Action {
    fn name(self) -> &'static str {
        match self {
            Action::Read => "read",
            Action::Claim => "claim",
            Action::Ack => "ack",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Entry {
    pub agent: String,
    pub instance: String,
    pub strength: Strength,
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub why: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub key_fp: Option<String>,
    pub first_ms: u64,
    pub last_ms: u64,
    pub actions: BTreeSet<String>,
    #[serde(skip)]
    persisted_last_ms: u64,
}

// ... (lines 261-479 omitted: persistence and bind/unbind) ...
    }
}

/// Check the bind and record the reader in one locked step (2.14.f(4)). `Err` refuses the call.
/// The caller records only topics that exist on the bus (r1 CC 3 / OC 2).
pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let map = g.get_or_insert_with(HashMap::new);
    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
    // r2 CC N4: a load-time row stays queued until it is written
    st.pending_rows.retain(|row| log(row.clone()).is_err());
    // the bind is read under the same lock the bind verb writes under
    let bindings = load_bindings(dir).map_err(bindings_refusal)?;
    if let Some(b) = bindings.get(topic)
        && !passes(b, reader)
    {
        return Err(bound_refusal(topic, b, reader, action, now_ms, log));
    }
    if topic.len() > MAX_TOPIC_LEN || st.load_error.is_some() {
        // nothing is recorded over an unreadable table (it stays reported unreadable)
        return Ok(());
    }
    let mut dirty = false;
    if !st.topics.contains_key(topic) && st.topics.len() >= MAX_TOPICS {
        // evict the inbox whose newest reader is oldest, but never one with a reader inside the window
        let window_start = now_ms.saturating_sub(DEFAULT_WINDOW_S * 1000);
        let oldest = st
            .topics
            .iter()
            .map(|(k, v)| (k.clone(), v.iter().map(|e| e.last_ms).max().unwrap_or(0)))
            .min_by_key(|(_, last)| *last);
        match oldest {
            Some((old, last)) if last < window_start => {
                st.topics.remove(&old);
                st.topic_since.remove(&old);
                capped_row("inbox-consumer-topic-evicted", serde_json::json!({"event": "inbox-consumer-topic-evicted", "topic": old}), now_ms, log);
                dirty = true;
            }
            _ => {
                capped_row(
                    "inbox-consumer-table-full",
                    serde_json::json!({"event": "inbox-consumer-table-full", "attention": true, "topic": topic}),
                    now_ms,
                    log,
                );
                // r2 CC N1: an inbox missing from a full table is answered as an error, never a clean list.
                // r3 CC 2 / OC 1: the marker moves on every drop; only the save is rate-limited.
                let saved = st.dropped_since_ms;
                if st.dropped_since_ms.is_none_or(|t| t < window_start) {
                    st.dropped_first_ms = Some(now_ms); // a new run of drops
                }
                st.dropped_since_ms = Some(now_ms);
                if saved.is_none_or(|t| now_ms >= t + PERSIST_EVERY_MS)
                    && let Err(e) = save_consumers(dir, st)
                {
                    st.persist_error = Some(format!("consumer list not persisted: {e}"));
                }
                return Ok(());
            }
        }
    }
    if !st.topics.contains_key(topic) {
        st.topic_since.insert(topic.to_string(), now_ms);
    }
    let list = st.topics.entry(topic.to_string()).or_default();
    // r1 CC 1 / OC 4: keyed by strength and key too, so a claimed reader never merges into a proven entry
    match list
        .iter_mut()
        .find(|e| e.agent == reader.agent && e.instance == reader.instance && e.strength == reader.strength && e.key_fp == reader.key_fp)
    {
        Some(e) => {
            // r2 CC N6: a changed reason alone is persisted on the PERSIST_EVERY_MS cadence, not per call
            e.why.clone_from(&reader.why);
            if e.actions.insert(action.name().to_string()) {
                dirty = true;
            }
            e.last_ms = e.last_ms.max(now_ms);
            if e.last_ms >= e.persisted_last_ms + PERSIST_EVERY_MS {
                dirty = true;
            }
        }
        None => {
            if list.len() >= MAX_PER_TOPIC
                && let Some(i) = list.iter().enumerate().min_by_key(|(_, e)| e.last_ms).map(|(i, _)| i)
            {
                let gone = list.remove(i);
                capped_row(
                    "inbox-consumer-evicted",
                    serde_json::json!({
                        "event": "inbox-consumer-evicted", "topic": topic, "agent": gone.agent,
                        "instance": gone.instance, "last_ms": gone.last_ms,
                    }),
                    now_ms,
                    log,
                );
            }
            list.push(Entry {
                agent: reader.agent.clone(),
                instance: reader.instance.clone(),
                strength: reader.strength.clone(),
                why: reader.why.clone(),
                key_fp: reader.key_fp.clone(),
                first_ms: now_ms,
                last_ms: now_ms,
                actions: [action.name().to_string()].into_iter().collect(),
                persisted_last_ms: 0,
            });
            dirty = true;
        }
    }
    if dirty {
        match save_consumers(dir, st) {
            Ok(()) => {
                st.persist_error = None;
                for v in st.topics.values_mut() {
                    for e in v.iter_mut() {
                        e.persisted_last_ms = e.last_ms;
                    }
                }
            }
            // the read itself goes on; the list is now reported unreadable (FCW-125), never "one"
            Err(e) => st.persist_error = Some(format!("consumer list not persisted: {e}")),
        }
    }
    Ok(())
}

/// `inbox.consumers`: readers seen within `window_s`, plus the binding. `Err` when the list cannot be
/// trusted (FCW-125): the caller shows `consumers: unknown`, never "one". A table restarted after a corrupt
/// one stays an error until the window no longer reaches back before the restart (r1 CC 4).
pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<serde_json::Value, String> {
    let bound = load_bindings(dir)?.get(topic).cloned();
    with_state(dir, |st| {
        if let Some(e) = st.load_error.as_ref().or(st.persist_error.as_ref()) {
            return Err(e.clone());
        }
        let since = now_ms.saturating_sub(window_s.saturating_mul(1000));
        if let Some(t) = st.incomplete_since_ms
            && t > since
        {
            return Err(format!("consumer history incomplete since {t} (an unreadable table was moved aside)"));
        }
        // r3 CC 1: an inbox absent from the table, or one that entered it after a drop inside the window (it may be
        // the one whose reader was dropped), is answered as an error, never as a clean list
        let first = st.dropped_first_ms.unwrap_or(0);
        let entered_after_first_drop = st.topic_since.get(topic).is_some_and(|entered| *entered >= first);
        if let Some(t) = st.dropped_since_ms
            && t > since
            && (!st.topics.contains_key(topic) || entered_after_first_drop)
        {
            return Err(format!("consumer table full since {t}: this inbox's readers are not recorded"));
        }
        let list: Vec<&Entry> = st.topics.get(topic).map(|v| v.iter().filter(|e| e.last_ms >= since).collect()).unwrap_or_default();
        Ok(serde_json::json!({"topic": topic, "window_s": window_s, "consumers": list, "bound": bound}))
    })
}

/// `hub.inbox_bind`: admin row first, then the durable binding (2.14.c). The bound key is the agent's
/// card key now; an agent without a card, or whose card key is revoked (r1 CC 14), cannot be bound.
pub fn bind(
    dir: &Path,
    topic: &str,
    agent: &str,
    card_key: Option<[u8; 32]>,
    revoked: &dyn Fn() -> Result<BTreeSet<String>, String>,
    now_ms: u64,
    log: AdminLog<'_>,
) -> Result<Binding, String> {
    if !applies(topic) || topic.len() > MAX_TOPIC_LEN {
        return Err(format!("{topic:?} is not an inbox topic of at most {MAX_TOPIC_LEN} bytes"));
    }
    if !crate::key_card::valid_name(agent) {
        return Err(format!("{agent:?} is not a plain agent name"));
    }
    let Some(k) = card_key else { return Err(format!("agent {agent:?} has no key card; bind refused")) };
    let vk = VerifyingKey::from_bytes(&k).map_err(|e| format!("agent {agent:?}'s card key is invalid: {e}"))?;
    let b = Binding { agent: agent.into(), key_fp: termlink_session::signed_state::key_fingerprint(&vk), at_ms: now_ms };
    if revoked().map_err(|e| format!("revocation list unreadable: {e}"))?.contains(&b.key_fp) {
        return Err(format!("agent {agent:?}'s card key {} is revoked; bind refused", b.key_fp));
    }
    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let mut all = load_bindings(dir)?;
    log(serde_json::json!({"event": "inbox-bind", "topic": topic, "agent": agent, "key_fp": b.key_fp}))?;
    all.insert(topic.to_string(), b.clone());
    write_bindings(dir, all)?;
    Ok(b)
}

/// `hub.inbox_unbind`: admin row first, then remove the binding. Unbinding an unbound inbox is a no-op.
pub fn unbind(dir: &Path, topic: &str, log: AdminLog<'_>) -> Result<bool, String> {
    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
    let mut all = load_bindings(dir)?;
    if !all.contains_key(topic) {
        return Ok(false);
    }
    log(serde_json::json!({"event": "inbox-unbind", "topic": topic}))?;
    all.remove(topic);
    write_bindings(dir, all)?;
    Ok(true)
}

fn write_bindings(dir: &Path, bindings: BTreeMap<String, Binding>) -> Result<(), String> {
    let f = BindingsFile { schema: BINDINGS_SCHEMA.into(), bindings };
    let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
    crate::key_card::atomic_write(&bindings_path(dir), &text, 0o600).map_err(|e| e.to_string())
}

/// Forget the cached state of `dir` (tests; a hub restart starts from disk anyway).
#[cfg(test)]
pub(crate) fn reset(dir: &Path) {
    if let Some(m) = STATES.lock().unwrap_or_else(|p| p.into_inner()).as_mut() {
        m.remove(dir);
    }
}

#[cfg(test)]
#[path = "inbox_consumers_tests.rs"]
mod tests;
```
## 5. Where reads are recorded: crates/termlink-hub/src/channel.rs 1570-1605
```rust
    use crate::inbox_consumers as ic;
    let Some(topic) = param_str(params, "topic") else { return Ok(None) };
    // r2 CC N2: a bound inbox is gated even before its topic exists
    if !ic::applies(topic) || (!topic_exists(topic) && !ic::is_bound(&termlink_session::discovery::runtime_dir(), topic)) {
        return Ok(None);
    }
    let position = params.get(position_key).and_then(|v| v.as_u64()).unwrap_or(0);
    let proof = match ic::parse_reader(params) {
        Ok(p) => p,
        Err(e) => return Err(ErrorResponse::new(id.clone(), -32602, &format!("{}: {e}", ic::PROOF_FIELD)).into()),
    };
    let transport = match peer_addr {
        Some(a) => format!("tcp:{}", a.rsplit_once(':').map(|(h, _)| h).unwrap_or(a)),
        None => "local".to_string(),
    };
    let dir = termlink_session::discovery::runtime_dir();
    let now = now_unix_ms();
    let reader = {
        let (ident, card_key, revoked) = consumer_lookups(&dir);
        let l = ic::Lookups { hub_id: ident.as_ref().map(|i| i.hub_id.as_str()), card_key: &card_key, revoked: &revoked };
        ic::assess(topic, method, position, proof.as_ref(), &transport, now, &l)
    };
    let reader = match reader {
        Ok(r) => r,
        Err(f) => return Err(consumer_refusal(id, f)),
    };
    let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
    match ic::record(&dir, topic, &reader, action, now.max(0) as u64, &mut log) {
        Ok(()) => Ok(Some(reader)),
        Err(f) => Err(consumer_refusal(id, f)),
    }
}

/// Whether a topic exists on the hub's bus; an unknown bus answers "exists" so the gate still applies.
fn topic_exists(topic: &str) -> bool {
    match bus_or_err(Value::Null) {
```
## 6. The consumers RPC: crates/termlink-hub/src/router.rs 1051-1065
```rust
fn handle_inbox_consumers(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
    let Some(topic) = params.get("topic").and_then(|v| v.as_str()).filter(|t| crate::inbox_consumers::applies(t)) else {
        return ErrorResponse::new(id, -32602, "Missing 'topic' (an inbox:* topic)").into();
    };
    let window = params
        .get("window_s")
        .and_then(|v| v.as_u64())
        .unwrap_or(crate::inbox_consumers::DEFAULT_WINDOW_S)
        .clamp(1, 30 * 86_400);
    let dir = termlink_session::discovery::runtime_dir();
    match crate::inbox_consumers::consumers(&dir, topic, window, now_ms()) {
        Ok(v) => Response::success(id, v).into(),
        Err(e) => ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("consumer list unreadable (FCW-125): {e}")).into(),
    }
}
```
## 7. runme.sh action 37 (p12_live_check)
```bash

head2 "4b. W1 sidecars onto the installed binary (T-3647, makes T-3646 live)"
w1_restart_onto_binary

# ---------------------------------------------------------------------------
# 37. P-12 live check: each W1 sidecar sees its inbox's consumer list (T-3700, arc-011 B-2; operator rulings E, D)
# Verification only, changes nothing. After 4 installs the P-12 binary and 4b restarts the W1 sidecars, each one's
# `sidecar status` must carry a live `consumers` line within P12_WAIT seconds (the sidecar checks every 30 s):
#   one (proven)                -> skip (live, one consumer: itself, proven by its own key)
#   inbox consumed by N ...     -> FAILED, the line names every other reader (a second consumer: find it before any
#                                  bind; binding an inbox is its own later action, by the operator's ruling)
#   anything else (unknown, no field) -> FAILED with the line
# No inbox is bound here. Seams (fixtures only): RUNME_P31_UNITS, RUNME_P31_TL (as 35/36), RUNME_P12_WAIT.
# ---------------------------------------------------------------------------
seam P12_WAIT RUNME_P12_WAIT 75
p12_live_check() {
    local unit inst agent out line waited
    if [ -z "$P31_UNITS" ]; then say "  skip    no W1 units configured"; SKIPPED=$((SKIPPED+1)); return; fi
    for unit in $P31_UNITS; do
        inst="$(w1_instance_of "$unit")"; agent="$(p31_env_get "$W1_CONF_DIR/$inst.env" AGENT)"
        if [ "$DRY_RUN" = "1" ]; then say "  [DRY]   would read $agent's consumers status"; DONE=$((DONE+1)); continue; fi
        waited=0
        while :; do
            out="$(TERMLINK_SIDECAR_STATUS_SOCKET="$W1_SOCK_DIR/$agent.status.sock" "$P31_TL" sidecar status --json 2>/dev/null)"
            line="$(p31_json "$out" 'v["consumers"]["consumers"]')"
            case "$line" in "one (proven)"|"inbox consumed by"*) break ;; esac
            [ "$waited" -ge "$P12_WAIT" ] && break
            sleep 5; waited=$((waited+5))
        done
        case "$line" in
            "one (proven)") say "  skip    $agent: consumers one (proven) — P-12 live"; SKIPPED=$((SKIPPED+1)) ;;
            "inbox consumed by"*) say "  FAILED  $agent: $line — find the other reader before any bind"; FAILED=$((FAILED+1)) ;;
            "") say "  FAILED  $agent: status carries no consumers line after ${P12_WAIT}s (sidecar older than P-12?)"; FAILED=$((FAILED+1)) ;;
            *) say "  FAILED  $agent: consumers '$line' after ${P12_WAIT}s"; FAILED=$((FAILED+1)) ;;
        esac
    done
}

head2 "37. P-12 live: each W1 sidecar's inbox consumer list (T-3700)"
p12_live_check

```
