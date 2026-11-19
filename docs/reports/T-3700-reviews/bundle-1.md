# T-3700 review bundle 1 — P-12 / B-2 inbox consumer list

## Design (operator rulings at the top)
# T-3700 design — P-12 / B-2: inbox consumer list, proven/claimed readers, bind by key

Prepared 2026-11-19. Implements AL-30 (06a 2.14) with operator ruling E on F-1 (T-3700 `## Decisions`): every inbox
read and claim names its reader; a reader with a key card signs; consumers are `proven` or `claimed`; a bind holds by
key. Spec sources and F-1 evidence: `T-3700-spec-extract.md`.

## 1. Scope

1. Topics: `inbox:*` only. arc-011 conversations ride inside inbox topics (`conv` is a payload field,
   `crates/termlink-sidecar/src/wire.rs:55`), so AL-30's "or conversation topic" adds no other topic today.
2. Consumer actions on an inbox (all three are recorded, FCW-209):
   1. read: `channel.subscribe` (poll, long-poll, read; the only read RPC);
   2. claim: `channel.claim`;
   3. ack: `channel.post` with `msg_type: receipt` (`termlink-mcp/src/tools.rs`, the ack tool), already signed.
3. Out of scope: posts of other types (senders post into inboxes; never refused by a bind), cross-hub copies (PH-3).

## 2. Wire (additive, old clients keep working)

1. `channel.subscribe` and `channel.claim` gain an optional `reader` object:
   `{agent, instance, ts_ms, signer_pk?, sig?}`.
   1. `agent`: plain agent name (`key_card::valid_name`); `instance`: the sidecar instance id (as `from_instance`).
   2. `sig` (ed25519, hex) over the domain-separated bytes
      `termlink-inbox-read/v1\n<hub_id>\n<topic>\n<method>\n<cursor-or-offset>\n<ts_ms>\n<agent>\n<instance>`.
      The prefix makes a read signature useless as a post signature and vice versa; `hub_id` stops replay on another
      hub; `ts_ms` must be within ±120 s of the hub clock (P-23 skew bound) and is not otherwise remembered.
2. A call with no `reader` is recorded as reader `(unnamed)`, instance = the transport kind (`local` / `tcp:<peer ip>`).
3. Receipt posts need no new field: the poster's `sender_pubkey_hex` is already verified; the hub maps it to an agent
   through the key cards (`proven`) or records the payload's claimed `from_agent`/`(unnamed)` (`claimed`).

## 3. Hub

1. **Proof.** A reader is `proven` when all hold: signature verifies; `ts_ms` in window; the agent's key card exists
   and its signing key equals `signer_pk`; the key is not on the revocation list (P-15). Otherwise `claimed`.
   A `sig` that is present but fails to verify, or a revoked key, **refuses the call** (code CHANNEL_KEY_CARD_MISMATCH /
   KEY_REVOKED, refusal counted like a post refusal) — a broken proof is never downgraded to a claim.
   A valid signature with `ts_ms` outside the window is **not** refused (a skewed clock must not cut a sidecar off its
   inbox; P-23 reports the skew): the reader is listed `claimed (clock skew)`, and after a bind it is refused like any
   unproven reader.
2. **Consumer table.** Per topic, keyed by (agent, instance): `{agent, instance, strength, first_ms, last_ms, actions}`.
   1. 16 entries per topic; on overflow evict the oldest by `last_ms` and write an admin-log row `inbox-consumer-evicted`.
   2. Durable: `<runtime>/inbox-consumers.json`, atomic write. Written when an entry is new, changes strength or action
      set, or `last_ms` moved by ≥ 60 s since the last write (deviation from 2.14.f(3) "written with every RPC": the
      sidecar long-polls every tick; a crash loses ≤ 60 s of `last_ms`, inside the 24 h window and the 60 s flag).
   3. Updated and checked under the same per-topic lock as the bind (2.14.f(4)).
3. **RPC `inbox.consumers {topic, window_s?}`** (Observe; default window 86 400 s): entries with `last_ms ≥ now − window`,
   plus `bound: {agent, key_fp} | null`. An unreadable table answers an error, never an empty list (FCW-125).
4. **Bind.** `hub.inbox_bind {topic, agent}` / `hub.inbox_unbind {topic}` (Execute, local socket only, in ADMIN_METHODS):
   admin-log row first, then `<runtime>/inbox-bindings.json` (durable, atomic). The binding stores the agent and the
   sha256 fingerprint of its card's signing key at bind time; binding an agent with no card is refused.
   CLI: `termlink hub inbox consumers|bind|unbind`. Binds reach the operator through runme only.
5. **Refusal after bind (FCW-124).** On a bound topic, a read, claim or receipt post passes only when it is `proven` as
   the bound agent with the bound key fingerprint; everything else is refused with
   `inbox bound to <agent>` and an admin-log row (rate-capped like the T-3703 refused rows, 10/min).
   A card rotation of the bound agent therefore needs a re-bind (stated, visible in the refusal reason).

## 4. Sidecar

1. Every inbox `channel.subscribe` / `channel.claim` it makes carries a signed `reader` (pull.rs, liveness.rs, deliver.rs).
2. Each tick: `inbox.consumers(MY_INBOX, 24 h)`.
   1. Err → status `consumers: unknown (<why>)` (never "one", FCW-125).
   2. Entries other than (MY_AGENT, MY_INSTANCE) → status `consumers: inbox consumed by two identities: A (proven),
      B (claimed)` and needs-attention coalesced on (topic, other agent+instance, UTC day). A same-agent, other-instance
      entry is the V-12 second copy and is named as such.
   3. Only own entry → `consumers: one (proven)`; a hub without the RPC → `consumers: unknown (hub predates consumer list)`.
3. Status also shows `bound: <agent>` when the hub reports a binding.
4. **Liveness is not a consumer (agent choice, 2026-11-19, found while building).** Every sidecar with an address book
   read one message of the *main's* inbox to judge "stuck" (`liveness.rs`, T-3646). As a content read by another
   agent it would flag the main's inbox and be refused after a bind. It now calls `channel.first_at {topic,
   from_offset}` (new, Observe), which returns only the offset and timestamp of the first record at or after
   `from_offset`, no content, and records no consumer. On a hub without it, the old one-message read is the fallback.
   No content is exposed, so this opens no BP-15 route. Reported to the operator; overrulable.

## 5. Diagnostic peeks (operator ruling D, 2026-11-19)

1. The sidecar gains a `peek` verb (CLI `termlink sidecar peek` and the local API): it reads the inbox from the hub with
   its own signed `reader`, returns the messages, and changes no cursor, stage or store state.
2. Direct key-less reads (CLI `channel subscribe`, MCP `channel_state`, AEF `fw sidecar inbox --peek` today) get no
   exception: listed `(unnamed)`/`claimed`, flagged, refused after a bind.
3. AEF is asked over TermLink to route its `--peek` through the sidecar (its code, a request only).

## 6. Evidence plan

1. E-66 (L4, real hub in a temp runtime dir, never the live one): fx-z subscribes and acks fx-a's inbox → flagged
   within 60 s naming both; bind fx-a → fx-z refused with reason + admin-log row; 17 identities → oldest evicted and
   logged; second instance of fx-a → flagged within one tick. E-66.6: read-only fx-z listed and flagged, refused after
   bind. Controls: mutants recording only subscribers / only acks+claims miss fx-z.
2. E-133: the per-tick check fires on injection (the E-66 run under the sidecar tick loop).
3. E-169 (B-2 part, L2): consumer RPC failing → `consumers: unknown`; control mutant defaulting to empty reports one.
4. Ruling-E cases: unsigned name equal to the bound agent refused after bind; bad signature refused before bind;
   revoked key refused; stale `ts_ms` (> 120 s) listed `claimed (clock skew)` before a bind and refused after it;
   a signature replayed on another topic or hub does not verify.
5. Mutant list (all must be killed): skip-signature-check, ignore-ts-window, bind-by-name, default-unreadable-to-empty,
   record-subscribers-only, record-acks-only, no-eviction-log, instance-ignored (V-12).

## Spec extract and F-1
## Spec sources

1. Plan row `08-plan.md:155`: P-12, B-2, R-49, R-60, E-66, E-133, E-169; exit "fixture suite exits 0 with a PASS line per
   listed E-n, mutant list all killed"; rollback "flag-only mode (DR-11 A before W4)".
2. Pseudo-code AL-30, `06a-pseudo-code-ph2-hardening.md:541-571` (2.14): record every subscribe/poll/read/ack/claim of an
   inbox or conversation topic per identity (first, last; 16 kept, oldest evicted with an admin-log row); RPC
   `inbox.consumers(topic, window)`; `admin inbox bind <topic> <identity>` (root-only socket, admin log first, durable);
   sidecar tick: other identity within 24 h → status `inbox consumed by two identities: A, B` + attention coalesced on
   (topic, identity, day); unreadable list → `consumers: unknown` (FCW-125). After a bind every read, ack, claim by
   another identity is refused and logged (FCW-124). A read-only consumer is a consumer (FCW-209, CR-6.33). Table and
   bind written under the per-topic lock in one step. 2.14.h: "the identity is the authenticated caller of the RPC [B]".
3. Architecture `04-architecture.md:495-504` (6.6.1-6.6.2), DR-11 A (`:1413`, flag during W1..W3, refuse after W4 bind),
   V-11 (24 h window, `:949`), V-12 (second live copy on the same hub within one tick, `:950`).
4. Evidence `05-evidence.md:723-729` E-66 (L4; flag within 60 s, refuse after bind, eviction logged, second copy within
   one tick; controls: subscriber-only and ack/claim-only mutants miss `fx-z`), E-133 (`:1310`, continuous check), E-169
   (`:960`, B-2 part only: unreadable list → `consumers: unknown`, never "one"; control: mutant defaulting to empty).

## Finding F-1: the "[B]" premise of 2.14.h does not hold for reads

`[B]` marks "a fact about what is built" (`04-architecture.md:11`). Checked in the code, 2026-11-19:

1. `channel.subscribe` (the only read/poll RPC) takes `topic, cursor, limit, timeout_ms, conversation_id, ...` and no
   caller identity (`crates/termlink-hub/src/channel.rs:1491-1530`). The router passes no caller to channel handlers
   (`router.rs:59`, `route(req, peer_addr)`).
2. Local-socket callers are trusted by OS uid; on .107 every sidecar, CLI and MCP reader is root (TB-1 separates nobody
   until PH-4 accounts, `04-architecture.md:1142`).
3. `channel.claim` carries a self-declared `claimer` string (`channel.rs:2179`).
4. Only posts (acks, stage rows, messages) are authenticated: signed, `sender_id` derived from the key (B-6 key cards).
5. The only per-connection principal is the T-3760 registration claim, used for `session.*` registration only
   (`router.rs:1505-1545`).

So AL-30 can key acks on an authenticated identity, but reads (the BP-15 shape that CR-6.33 added) and claims only on a
caller-declared name. How to identify readers is an operator decision (brief in the T-3700 `## Decisions` once ruled).

## Dialogue Log

## Code diff (crates)
```diff
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index 12bb0d95e..a48e63c87 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -983,6 +983,29 @@ pub(crate) async fn handle_channel_post_with_peer(
         return resp;
     }
 
+    // T-3700 (AL-30, FCW-209): an ack (receipt) on an inbox records its poster as a consumer and is
+    // refused on a bound inbox unless it is the bound agent's bound key.
+    if msg_type == "receipt" && crate::inbox_consumers::applies(&topic) {
+        use crate::inbox_consumers as ic;
+        let dir = termlink_session::discovery::runtime_dir();
+        let reader = {
+            let (ident, card_key, revoked) = consumer_lookups(&dir);
+            let l = ic::Lookups { hub_id: ident.as_ref().map(|i| i.hub_id.as_str()), card_key: &card_key, revoked: &revoked };
+            let d = dir.clone();
+            let cards = move || -> Result<Vec<(String, [u8; 32])>, String> {
+                Ok(crate::key_card::list(&d)?
+                    .into_iter()
+                    .filter_map(|c| crate::key_card::parse_signing_key(&c.signing_key).map(|k| (c.agent, k)))
+                    .collect())
+            };
+            ic::assess_ack(&verifying_key, &cards, &l)
+        };
+        let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
+        if let Err(f) = ic::record(&dir, &topic, &reader, ic::Action::Ack, now_unix_ms().max(0) as u64, &mut log) {
+            return consumer_refusal(&id, f);
+        }
+    }
+
     // T-1287: optional metadata routing-hint map. NOT included in canonical
     // signed bytes — trusted-mesh threat model treats it as routing only.
     // Well-known keys: conversation_id, event_type (per T-1288 catalog).
@@ -1480,7 +1503,11 @@ fn activity_json(act: &termlink_bus::TopicActivity, now_ms: i64, days: u64) -> (
     )
 }
 
-pub async fn handle_channel_subscribe(id: Value, params: &Value) -> RpcResponse {
+pub async fn handle_channel_subscribe(id: Value, params: &Value, peer_addr: Option<&str>) -> RpcResponse {
+    // T-3700 (AL-30): an inbox read records its reader and is refused on a bound inbox unless proven
+    if let Some(r) = consumer_gate(&id, params, "channel.subscribe", "cursor", crate::inbox_consumers::Action::Read, peer_addr) {
+        return r;
+    }
     let bus = match bus_or_err(id.clone()) {
         Ok(b) => b,
         Err(r) => return r,
@@ -1488,6 +1515,101 @@ pub async fn handle_channel_subscribe(id: Value, params: &Value) -> RpcResponse
     handle_channel_subscribe_with(bus, id, params).await
 }
 
+/// T-3700: `channel.first_at` — offset and timestamp of the first record at or after `from_offset`, no
+/// content. Liveness checks of another agent's inbox use it, so they are not consumers of that inbox.
+pub fn handle_channel_first_at(id: Value, params: &Value) -> RpcResponse {
+    let bus = match bus_or_err(id.clone()) {
+        Ok(b) => b,
+        Err(r) => return r,
+    };
+    let topic = match param_str(params, "topic") {
+        Some(t) if !t.is_empty() => t,
+        _ => return ErrorResponse::new(id, -32602, "Missing 'topic' in params").into(),
+    };
+    let from = params.get("from_offset").and_then(|v| v.as_u64()).unwrap_or(0);
+    match bus.subscribe(topic, from) {
+        Ok(mut it) => match it.next() {
+            Some(Ok((offset, env))) => Response::success(id, json!({"offset": offset, "ts": env.ts_unix_ms})).into(),
+            None => Response::success(id, json!({"offset": null, "ts": null})).into(),
+            Some(Err(e)) => ErrorResponse::new(id, -32603, &format!("channel.first_at: {e}")).into(),
+        },
+        Err(termlink_bus::BusError::UnknownTopic(t)) => {
+            ErrorResponse::new(id, error_code::CHANNEL_TOPIC_UNKNOWN, &format!("channel.first_at: unknown topic '{t}'")).into()
+        }
+        Err(e) => ErrorResponse::new(id, -32603, &format!("channel.first_at: {e}")).into(),
+    }
+}
+
+/// T-3700 (06a AL-30, operator rulings E and D 2026-11-19): record the reader of an inbox read or claim
+/// and enforce a bind. `None` lets the call proceed. Non-inbox topics are untouched.
+pub(crate) fn consumer_gate(
+    id: &Value,
+    params: &Value,
+    method: &str,
+    position_key: &str,
+    action: crate::inbox_consumers::Action,
+    peer_addr: Option<&str>,
+) -> Option<RpcResponse> {
+    use crate::inbox_consumers as ic;
+    let topic = param_str(params, "topic")?;
+    if !ic::applies(topic) {
+        return None;
+    }
+    let position = params.get(position_key).and_then(|v| v.as_u64()).unwrap_or(0);
+    let proof = match ic::parse_reader(params) {
+        Ok(p) => p,
+        Err(e) => return Some(ErrorResponse::new(id.clone(), -32602, &format!("reader: {e}")).into()),
+    };
+    let transport = match peer_addr {
+        Some(a) => format!("tcp:{}", a.rsplit_once(':').map(|(h, _)| h).unwrap_or(a)),
+        None => "local".to_string(),
+    };
+    let dir = termlink_session::discovery::runtime_dir();
+    let now = now_unix_ms();
+    let reader = {
+        let (ident, card_key, revoked) = consumer_lookups(&dir);
+        let l = ic::Lookups { hub_id: ident.as_ref().map(|i| i.hub_id.as_str()), card_key: &card_key, revoked: &revoked };
+        ic::assess(topic, method, position, proof.as_ref(), &transport, now, &l)
+    };
+    let reader = match reader {
+        Ok(r) => r,
+        Err(f) => return Some(consumer_refusal(id, f)),
+    };
+    let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
+    match ic::record(&dir, topic, &reader, action, now.max(0) as u64, &mut log) {
+        Ok(()) => None,
+        Err(f) => Some(consumer_refusal(id, f)),
+    }
+}
+
+type CardKeyFn = Box<dyn Fn(&str) -> Result<Option<[u8; 32]>, String>>;
+type RevokedFn = Box<dyn Fn() -> Result<std::collections::BTreeSet<String>, String>>;
+
+/// The live hub's lookups for [`crate::inbox_consumers::assess`]: identity, card keys, revocation list.
+fn consumer_lookups(dir: &std::path::Path) -> (Option<crate::hub_identity::HubIdentity>, CardKeyFn, RevokedFn) {
+    let ident = crate::hub_identity::current();
+    let d1 = dir.to_path_buf();
+    let card_key: CardKeyFn = Box::new(move |a: &str| {
+        crate::key_card::get(&d1, a).map(|o| o.and_then(|c| crate::key_card::parse_signing_key(&c.signing_key)))
+    });
+    let d2 = dir.to_path_buf();
+    let id2 = ident.clone();
+    let revoked: RevokedFn = Box::new(move || match &id2 {
+        Some(i) => crate::revocation::revoked_keys(&d2, &i.hub_id, i.signing_key()),
+        None => Err("the hub identity is not loaded".to_string()),
+    });
+    (ident, card_key, revoked)
+}
+
+fn consumer_refusal(id: &Value, f: crate::inbox_consumers::Refusal) -> RpcResponse {
+    ErrorResponse::with_data(id.clone(), f.code, &format!("refused ({}): {}", f.fcw, f.reason), serde_json::json!({"fcw": f.fcw}))
+        .into()
+}
+
+fn now_unix_ms() -> i64 {
+    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as i64).unwrap_or(0)
+}
+
 pub(crate) async fn handle_channel_subscribe_with(
     bus: &Bus,
     id: Value,
@@ -2155,7 +2277,11 @@ pub(crate) async fn handle_channel_list_with(
 /// Issues an exclusive lease over `(topic, offset)` for `claimer`. Default TTL is 30s; the
 /// client must call `channel.release` (or the future `channel.renew`, Slice 2) before
 /// `claimed_until` to retain the claim.
-pub async fn handle_channel_claim(id: Value, params: &Value) -> RpcResponse {
+pub async fn handle_channel_claim(id: Value, params: &Value, peer_addr: Option<&str>) -> RpcResponse {
+    // T-3700 (AL-30): a claim on an inbox is a consumer action like a read
+    if let Some(r) = consumer_gate(&id, params, "channel.claim", "offset", crate::inbox_consumers::Action::Claim, peer_addr) {
+        return r;
+    }
     let bus = match bus_or_err(id.clone()) {
         Ok(b) => b,
         Err(r) => return r,
diff --git a/crates/termlink-hub/src/inbox_consumers.rs b/crates/termlink-hub/src/inbox_consumers.rs
new file mode 100644
index 000000000..c6dc70276
--- /dev/null
+++ b/crates/termlink-hub/src/inbox_consumers.rs
@@ -0,0 +1,507 @@
+//! T-3700 (arc-011 P-12, B-2, 06a AL-30): who consumes each inbox, `proven` or `claimed`, and the bind.
+//!
+//! Every read (`channel.subscribe`), claim (`channel.claim`) and ack (`channel.post` of a `receipt`) on an
+//! `inbox:*` topic records its reader. A reader names itself in an optional `reader` object; one with a key
+//! card signs it, and is `proven` when the signature, the card and the revocation list all agree
+//! (operator ruling E, 2026-11-19). A broken proof is refused, never downgraded. A bound inbox
+//! (`hub.inbox_bind`, operator runme) admits only reads proven by the bound agent's bound key (FCW-124).
+//! Design: `docs/reports/T-3700-design.md`.
+
+use std::collections::{BTreeMap, BTreeSet, HashMap};
+use std::path::{Path, PathBuf};
+use std::sync::Mutex;
+
+use ed25519_dalek::{Signature, Verifier, VerifyingKey};
+use serde::{Deserialize, Serialize};
+use termlink_protocol::control::error_code;
+
+/// AL-30 2.14.c: entries kept per inbox; the oldest by `last_ms` is evicted with an admin row.
+pub const MAX_PER_TOPIC: usize = 16;
+/// Bound on the number of inboxes tracked, so caller-chosen topic names cannot grow the table without limit.
+pub const MAX_TOPICS: usize = 4096;
+/// A read proof's timestamp must be this close to the hub clock to prove (P-23 skew bound).
+pub const PROOF_WINDOW_MS: i64 = 120_000;
+/// `last_ms` is persisted when it moved by at least this much since the last write (design 3.2.2).
+pub const PERSIST_EVERY_MS: u64 = 60_000;
+/// V-11: the default consumer window.
+pub const DEFAULT_WINDOW_S: u64 = 86_400;
+/// The instance recorded for an ack (a receipt post carries no instance).
+pub const ACK_INSTANCE: &str = "-";
+/// The agent recorded for a caller that names no reader.
+pub const UNNAMED: &str = "(unnamed)";
+/// Domain prefix of the read-proof bytes (shared with the sidecar through `termlink-session`).
+pub use termlink_session::inbox_read::{READ_DOMAIN, read_sign_bytes};
+
+/// Topics this module applies to.
+pub fn applies(topic: &str) -> bool {
+    topic.starts_with("inbox:")
+}
+
+/// The `reader` object of a read or claim.
+#[derive(Debug, Clone, Default, PartialEq, Eq)]
+pub struct ReadProof {
+    pub agent: String,
+    pub instance: String,
+    pub ts_ms: i64,
+    pub signer_pk: Option<String>,
+    pub sig: Option<String>,
+}
+
+/// Parse `params.reader`. `Ok(None)` when absent; `Err` when present but malformed (refused: a caller that
+/// sends a reader object must send a valid one).
+pub fn parse_reader(params: &serde_json::Value) -> Result<Option<ReadProof>, String> {
+    let Some(r) = params.get("reader") else { return Ok(None) };
+    if r.is_null() {
+        return Ok(None);
+    }
+    let s = |k: &str| r.get(k).and_then(|v| v.as_str()).map(str::to_string);
+    let agent = s("agent").ok_or("reader.agent is missing")?;
+    if !crate::key_card::valid_name(&agent) {
+        return Err(format!("reader.agent {:?} is not a plain agent name", agent.chars().take(64).collect::<String>()));
+    }
+    let instance = s("instance").unwrap_or_default();
+    if instance.len() > 128 || instance.chars().any(|c| c.is_control() || c == '\n') {
+        return Err("reader.instance is longer than 128 bytes or holds a control character".into());
+    }
+    let ts_ms = r.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
+    let signer_pk = s("signer_pk");
+    let sig = s("sig");
+    if signer_pk.is_some() != sig.is_some() {
+        return Err("reader.signer_pk and reader.sig come together or not at all".into());
+    }
+    Ok(Some(ReadProof { agent, instance, ts_ms, signer_pk, sig }))
+}
+
+/// How strongly the hub knows who a reader is.
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+#[serde(rename_all = "lowercase")]
+pub enum Strength {
+    Proven,
+    Claimed,
+}
+
+/// A reader as the hub records it.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Reader {
+    pub agent: String,
+    pub instance: String,
+    pub strength: Strength,
+    /// Why a reader is only `claimed` (empty when proven).
+    pub why: String,
+    /// The signing key fingerprint of a proven reader.
+    pub key_fp: Option<String>,
+}
+
+/// A refusal: code, FCW id, reason.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Refusal {
+    pub code: i64,
+    pub fcw: &'static str,
+    pub reason: String,
+}
+
+/// What [`assess`] needs from the hub, injectable for tests.
+pub struct Lookups<'a> {
+    /// The hub's id; `None` when the identity is not loaded (nothing can be proven).
+    pub hub_id: Option<&'a str>,
+    /// The signing key (raw 32 bytes) on an agent's card, `Ok(None)` without a card.
+    pub card_key: &'a dyn Fn(&str) -> Result<Option<[u8; 32]>, String>,
+    /// The fingerprints on the hub's revocation list.
+    pub revoked: &'a dyn Fn() -> Result<BTreeSet<String>, String>,
+}
+
+fn claimed(agent: &str, instance: &str, why: &str) -> Reader {
+    Reader { agent: agent.into(), instance: instance.into(), strength: Strength::Claimed, why: why.into(), key_fp: None }
+}
+
+/// Decide who a read or claim comes from (design 3.1). `transport` names a caller with no reader
+/// (`local` or `tcp:<ip>`).
+pub fn assess(
+    topic: &str,
+    method: &str,
+    position: u64,
+    proof: Option<&ReadProof>,
+    transport: &str,
+    now_ms: i64,
+    l: &Lookups<'_>,
+) -> Result<Reader, Refusal> {
+    let Some(p) = proof else { return Ok(claimed(UNNAMED, transport, "no reader named")) };
+    let (Some(pk_hex), Some(sig_hex)) = (p.signer_pk.as_deref(), p.sig.as_deref()) else {
+        return Ok(claimed(&p.agent, &p.instance, "unsigned"));
+    };
+    let bad = |reason: String| Refusal { code: error_code::CHANNEL_SIGNATURE_INVALID, fcw: "FCW-124", reason };
+    let Some(vk) = termlink_session::signed_state::verifying_key_of_hex(pk_hex) else {
+        return Err(bad("reader.signer_pk is not a lower-case 32-byte ed25519 public key".into()));
+    };
+    let Some(sig) = parse_sig(sig_hex) else {
+        return Err(bad("reader.sig is not a 64-byte ed25519 signature".into()));
+    };
+    let Some(hub_id) = l.hub_id else {
+        return Ok(claimed(&p.agent, &p.instance, "the hub identity is not loaded"));
+    };
+    let bytes = read_sign_bytes(hub_id, topic, method, position, p.ts_ms, &p.agent, &p.instance);
+    if vk.verify(&bytes, &sig).is_err() {
+        return Err(bad(format!("the read signature of {:?} failed verification", p.agent)));
+    }
+    match (l.card_key)(&p.agent) {
+        Ok(Some(k)) if &k == vk.as_bytes() => {}
+        Ok(Some(_)) => {
+            return Err(Refusal {
+                code: error_code::CHANNEL_KEY_CARD_MISMATCH,
+                fcw: "FCW-124",
+                reason: format!("the read is not signed by the key on agent {:?}'s card", p.agent),
+            })
+        }
+        Ok(None) => return Ok(claimed(&p.agent, &p.instance, "no key card")),
+        Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("key cards unreadable: {e}"))),
+    }
+    let fp = termlink_session::signed_state::key_fingerprint(&vk);
+    match (l.revoked)() {
+        Ok(keys) if keys.contains(&fp) => {
+            return Err(Refusal {
+                code: error_code::KEY_REVOKED,
+                fcw: "FCW-99",
+                reason: format!("agent {:?}'s card key {fp} is on the hub's revocation list", p.agent),
+            })
+        }
+        Ok(_) => {}
+        Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("revocation list unreadable: {e}"))),
+    }
+    if (now_ms - p.ts_ms).abs() > PROOF_WINDOW_MS {
+        return Ok(claimed(&p.agent, &p.instance, "clock skew"));
+    }
+    Ok(Reader { agent: p.agent.clone(), instance: p.instance.clone(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) })
+}
+
+/// The reader of a verified receipt post: proven when the signer's key is on a card and not revoked.
+pub fn assess_ack(signer: &VerifyingKey, cards: &dyn Fn() -> Result<Vec<(String, [u8; 32])>, String>, l: &Lookups<'_>) -> Reader {
+    let fp = termlink_session::signed_state::key_fingerprint(signer);
+    let agent = match cards() {
+        Ok(cs) => cs.into_iter().find(|(_, k)| k == signer.as_bytes()).map(|(a, _)| a),
+        Err(e) => return claimed(&format!("key:{}", &fp[8..24.min(fp.len())]), ACK_INSTANCE, &format!("key cards unreadable: {e}")),
+    };
+    let Some(agent) = agent else {
+        return claimed(&format!("key:{}", &fp[8..24.min(fp.len())]), ACK_INSTANCE, "no key card");
+    };
+    match (l.revoked)() {
+        Ok(keys) if !keys.contains(&fp) => {
+            Reader { agent, instance: ACK_INSTANCE.into(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) }
+        }
+        Ok(_) => claimed(&agent, ACK_INSTANCE, "key revoked"),
+        Err(e) => claimed(&agent, ACK_INSTANCE, &format!("revocation list unreadable: {e}")),
+    }
+}
+
+fn parse_sig(hex: &str) -> Option<Signature> {
+    let b = termlink_session::signed_state::unhex_lower(hex)?;
+    let arr: [u8; 64] = b.try_into().ok()?;
+    Some(Signature::from_bytes(&arr))
+}
+
+/// A consumer action (FCW-209: every one records).
+#[derive(Debug, Clone, Copy, PartialEq, Eq)]
+pub enum Action {
+    Read,
+    Claim,
+    Ack,
+}
+
+impl Action {
+    fn name(self) -> &'static str {
+        match self {
+            Action::Read => "read",
+            Action::Claim => "claim",
+            Action::Ack => "ack",
+        }
+    }
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct Entry {
+    pub agent: String,
+    pub instance: String,
+    pub strength: Strength,
+    #[serde(default, skip_serializing_if = "String::is_empty")]
+    pub why: String,
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    pub key_fp: Option<String>,
+    pub first_ms: u64,
+    pub last_ms: u64,
+    pub actions: BTreeSet<String>,
+    #[serde(skip)]
+    persisted_last_ms: u64,
+}
+
+#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
+pub struct Binding {
+    pub agent: String,
+    pub key_fp: String,
+    pub at_ms: u64,
+}
+
+#[derive(Debug, Default, Serialize, Deserialize)]
+struct ConsumersFile {
+    schema: String,
+    topics: BTreeMap<String, Vec<Entry>>,
+}
+
+#[derive(Debug, Default, Serialize, Deserialize)]
+struct BindingsFile {
+    schema: String,
+    bindings: BTreeMap<String, Binding>,
+}
+
+const CONSUMERS_SCHEMA: &str = "termlink.inbox-consumers/v1";
+const BINDINGS_SCHEMA: &str = "termlink.inbox-bindings/v1";
+
+pub fn consumers_path(dir: &Path) -> PathBuf {
+    dir.join("inbox-consumers.json")
+}
+
+pub fn bindings_path(dir: &Path) -> PathBuf {
+    dir.join("inbox-bindings.json")
+}
+
+/// In-memory state of one runtime dir.
+#[derive(Debug, Default)]
+struct State {
+    topics: BTreeMap<String, Vec<Entry>>,
+    /// The last persist of the consumer table failed: `inbox.consumers` answers an error (FCW-125).
+    persist_error: Option<String>,
+    /// The consumer file could not be read at load: same.
+    load_error: Option<String>,
+}
+
+static STATES: Mutex<Option<HashMap<PathBuf, State>>> = Mutex::new(None);
+
+fn with_state<R>(dir: &Path, f: impl FnOnce(&mut State) -> R) -> R {
+    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
+    let map = g.get_or_insert_with(HashMap::new);
+    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
+    f(st)
+}
+
+fn load_state(dir: &Path) -> State {
+    match std::fs::read(consumers_path(dir)) {
+        Ok(b) => match serde_json::from_slice::<ConsumersFile>(&b) {
+            Ok(f) if f.schema == CONSUMERS_SCHEMA => {
+                let mut topics = f.topics;
+                for v in topics.values_mut() {
+                    for e in v.iter_mut() {
+                        e.persisted_last_ms = e.last_ms;
+                    }
+                }
+                State { topics, ..Default::default() }
+            }
+            Ok(f) => State { load_error: Some(format!("unknown schema {:?}", f.schema)), ..Default::default() },
+            Err(e) => State { load_error: Some(format!("unparseable: {e}")), ..Default::default() },
+        },
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
+        Err(e) => State { load_error: Some(e.to_string()), ..Default::default() },
+    }
+}
+
+fn load_bindings(dir: &Path) -> Result<BTreeMap<String, Binding>, String> {
+    match std::fs::read(bindings_path(dir)) {
+        Ok(b) => {
+            let f: BindingsFile = serde_json::from_slice(&b).map_err(|e| format!("inbox bindings unparseable: {e}"))?;
+            if f.schema != BINDINGS_SCHEMA {
+                return Err(format!("inbox bindings: unknown schema {:?}", f.schema));
+            }
+            Ok(f.bindings)
+        }
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(BTreeMap::new()),
+        Err(e) => Err(format!("inbox bindings unreadable: {e}")),
+    }
+}
+
+fn save_consumers(dir: &Path, topics: &BTreeMap<String, Vec<Entry>>) -> Result<(), String> {
+    let f = ConsumersFile { schema: CONSUMERS_SCHEMA.into(), topics: topics.clone() };
+    let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
+    crate::key_card::atomic_write(&consumers_path(dir), &text, 0o600).map_err(|e| e.to_string())
+}
+
+/// Admin-log writer, injectable for tests.
+pub type AdminLog<'a> = &'a mut dyn FnMut(serde_json::Value) -> Result<(), String>;
+
+/// Check the bind and record the reader in one locked step (2.14.f(4)). `Err` refuses the call.
+pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
+    let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
+    let map = g.get_or_insert_with(HashMap::new);
+    let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
+    // the bind is read under the same lock the bind verb writes under
+    let bindings = load_bindings(dir).map_err(|e| Refusal {
+        code: error_code::INBOX_BOUND,
+        fcw: "FCW-124",
+        reason: format!("{e}; inbox reads are refused until it is repaired"),
+    })?;
+    if let Some(b) = bindings.get(topic) {
+        let ok = reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str());
+        if !ok {
+            let shown = if reader.strength == Strength::Proven {
+                format!("{} (proven)", reader.agent)
+            } else {
+                format!("{} (claimed: {})", reader.agent, reader.why)
+            };
+            if refused_row_allowed(now_ms) {
+                let _ = log(serde_json::json!({
+                    "event": "inbox-bound-refused", "attention": true, "topic": topic, "bound": b.agent,
+                    "reader": reader.agent, "instance": reader.instance, "strength": reader.strength,
+                    "action": action.name(),
+                }));
+            }
+            return Err(Refusal {
+                code: error_code::INBOX_BOUND,
+                fcw: "FCW-124",
+                reason: format!("inbox bound to {}; {} by {shown} refused", b.agent, action.name()),
+            });
+        }
+    }
+    let mut dirty = false;
+    if !st.topics.contains_key(topic) && st.topics.len() >= MAX_TOPICS {
+        // evict the inbox whose newest reader is oldest
+        if let Some(old) = st
+            .topics
+            .iter()
+            .min_by_key(|(_, v)| v.iter().map(|e| e.last_ms).max().unwrap_or(0))
+            .map(|(k, _)| k.clone())
+        {
+            st.topics.remove(&old);
+            let _ = log(serde_json::json!({"event": "inbox-consumer-topic-evicted", "topic": old}));
+            dirty = true;
+        }
+    }
+    let list = st.topics.entry(topic.to_string()).or_default();
+    match list.iter_mut().find(|e| e.agent == reader.agent && e.instance == reader.instance) {
+        Some(e) => {
+            if e.strength != reader.strength || e.key_fp != reader.key_fp || e.why != reader.why {
+                e.strength = reader.strength.clone();
+                e.key_fp = reader.key_fp.clone();
+                e.why = reader.why.clone();
+                dirty = true;
+            }
+            if e.actions.insert(action.name().to_string()) {
+                dirty = true;
+            }
+            e.last_ms = e.last_ms.max(now_ms);
+            if e.last_ms >= e.persisted_last_ms + PERSIST_EVERY_MS {
+                dirty = true;
+            }
+        }
+        None => {
+            if list.len() >= MAX_PER_TOPIC {
+                if let Some(i) = list.iter().enumerate().min_by_key(|(_, e)| e.last_ms).map(|(i, _)| i) {
+                    let gone = list.remove(i);
+                    let _ = log(serde_json::json!({
+                        "event": "inbox-consumer-evicted", "topic": topic, "agent": gone.agent,
+                        "instance": gone.instance, "last_ms": gone.last_ms,
+                    }));
+                }
+            }
+            list.push(Entry {
+                agent: reader.agent.clone(),
+                instance: reader.instance.clone(),
+                strength: reader.strength.clone(),
+                why: reader.why.clone(),
+                key_fp: reader.key_fp.clone(),
+                first_ms: now_ms,
+                last_ms: now_ms,
+                actions: [action.name().to_string()].into_iter().collect(),
+                persisted_last_ms: 0,
+            });
+            dirty = true;
+        }
+    }
+    if dirty {
+        match save_consumers(dir, &st.topics) {
+            Ok(()) => {
+                st.persist_error = None;
+                for v in st.topics.values_mut() {
+                    for e in v.iter_mut() {
+                        e.persisted_last_ms = e.last_ms;
+                    }
+                }
+            }
+            // the read itself goes on; the list is now reported unreadable (FCW-125), never "one"
+            Err(e) => st.persist_error = Some(format!("consumer list not persisted: {e}")),
+        }
+    }
+    Ok(())
+}
+
+/// `inbox.consumers`: readers seen within `window_s`, plus the binding. `Err` when the list cannot be
+/// trusted (FCW-125): the caller shows `consumers: unknown`, never "one".
+pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<serde_json::Value, String> {
+    let bound = load_bindings(dir)?.get(topic).cloned();
+    with_state(dir, |st| {
+        if let Some(e) = st.load_error.as_ref().or(st.persist_error.as_ref()) {
+            return Err(e.clone());
+        }
+        let since = now_ms.saturating_sub(window_s.saturating_mul(1000));
+        let list: Vec<&Entry> = st.topics.get(topic).map(|v| v.iter().filter(|e| e.last_ms >= since).collect()).unwrap_or_default();
+        Ok(serde_json::json!({"topic": topic, "window_s": window_s, "consumers": list, "bound": bound}))
+    })
+}
+
+/// `hub.inbox_bind`: admin row first, then the durable binding (2.14.c). The bound key is the agent's
+/// card key now; an agent without a card cannot be bound.
+pub fn bind(dir: &Path, topic: &str, agent: &str, card_key: Option<[u8; 32]>, now_ms: u64, log: AdminLog<'_>) -> Result<Binding, String> {
+    if !applies(topic) {
+        return Err(format!("{topic:?} is not an inbox topic"));
+    }
+    if !crate::key_card::valid_name(agent) {
+        return Err(format!("{agent:?} is not a plain agent name"));
+    }
+    let Some(k) = card_key else { return Err(format!("agent {agent:?} has no key card; bind refused")) };
+    let vk = VerifyingKey::from_bytes(&k).map_err(|e| format!("agent {agent:?}'s card key is invalid: {e}"))?;
+    let b = Binding { agent: agent.into(), key_fp: termlink_session::signed_state::key_fingerprint(&vk), at_ms: now_ms };
+    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
+    let mut all = load_bindings(dir)?;
+    log(serde_json::json!({"event": "inbox-bind", "topic": topic, "agent": agent, "key_fp": b.key_fp}))?;
+    all.insert(topic.to_string(), b.clone());
+    write_bindings(dir, all)?;
+    Ok(b)
+}
+
+/// `hub.inbox_unbind`: admin row first, then remove the binding. Unbinding an unbound inbox is a no-op.
+pub fn unbind(dir: &Path, topic: &str, log: AdminLog<'_>) -> Result<bool, String> {
+    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
+    let mut all = load_bindings(dir)?;
+    if !all.contains_key(topic) {
+        return Ok(false);
+    }
+    log(serde_json::json!({"event": "inbox-unbind", "topic": topic}))?;
+    all.remove(topic);
+    write_bindings(dir, all)?;
+    Ok(true)
+}
+
+fn write_bindings(dir: &Path, bindings: BTreeMap<String, Binding>) -> Result<(), String> {
+    let f = BindingsFile { schema: BINDINGS_SCHEMA.into(), bindings };
+    let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
+    crate::key_card::atomic_write(&bindings_path(dir), &text, 0o600).map_err(|e| e.to_string())
+}
+
+/// At most 10 refused-read admin rows a minute (any caller can provoke them; r2 CC 12 of T-3703).
+fn refused_row_allowed(now_ms: u64) -> bool {
+    static WIN: Mutex<(u64, u32)> = Mutex::new((0, 0));
+    let mut w = WIN.lock().unwrap_or_else(|p| p.into_inner());
+    let minute = now_ms / 60_000;
+    if w.0 != minute {
+        *w = (minute, 0);
+    }
+    w.1 += 1;
+    w.1 <= 10
+}
+
+/// Forget the cached state of `dir` (tests; a hub restart starts from disk anyway).
+#[cfg(test)]
+pub(crate) fn reset(dir: &Path) {
+    if let Some(m) = STATES.lock().unwrap_or_else(|p| p.into_inner()).as_mut() {
+        m.remove(dir);
+    }
+}
+
+#[cfg(test)]
+#[path = "inbox_consumers_tests.rs"]
+mod tests;
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index c8a97a315..a64e4a202 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -96,7 +96,7 @@ pub async fn route(req: &Request, peer_addr: Option<&str>) -> Option<RpcResponse
             crate::channel::handle_channel_post(id, &req.params, peer_addr).await
         }
         control::method::CHANNEL_SUBSCRIBE => {
-            crate::channel::handle_channel_subscribe(id, &req.params).await
+            crate::channel::handle_channel_subscribe(id, &req.params, peer_addr).await
         }
         control::method::CHANNEL_LIST => {
             crate::channel::handle_channel_list(id, &req.params).await
@@ -110,8 +110,9 @@ pub async fn route(req: &Request, peer_addr: Option<&str>) -> Option<RpcResponse
         control::method::CHANNEL_RECEIPTS => {
             crate::channel::handle_channel_receipts(id, &req.params).await
         }
+        control::method::CHANNEL_FIRST_AT => crate::channel::handle_channel_first_at(id, &req.params),
         control::method::CHANNEL_CLAIM => {
-            crate::channel::handle_channel_claim(id, &req.params).await
+            crate::channel::handle_channel_claim(id, &req.params, peer_addr).await
         }
         control::method::DISPATCH_ALLOCATE => {
             crate::dispatch_contract::handle_dispatch_allocate(id, &req.params).await
@@ -177,6 +178,9 @@ pub async fn route(req: &Request, peer_addr: Option<&str>) -> Option<RpcResponse
         "hub.revoke" => handle_revoke(id, &req.params, peer_addr),
         "hub.revocation_request" => handle_revocation_request(id, &req.params),
         "hub.revocation_reissue" => handle_revocation_reissue(id, peer_addr),
+        "inbox.consumers" => handle_inbox_consumers(id, &req.params),
+        "hub.inbox_bind" => handle_inbox_bind(id, &req.params, peer_addr),
+        "hub.inbox_unbind" => handle_inbox_unbind(id, &req.params, peer_addr),
         "hub.legacy_usage" => handle_hub_legacy_usage(id, &req.params),
         "hub.bus_state" => handle_hub_bus_state(id),
         "hub.governor_status" => handle_hub_governor_status(id),
@@ -1042,6 +1046,58 @@ fn handle_revoke(id: serde_json::Value, params: &serde_json::Value, peer_addr: O
     }
 }
 
+/// T-3700 (AL-30, OBS-56): the readers of one inbox within a window, and its binding. An error means the
+/// list cannot be trusted (FCW-125); the caller shows `consumers: unknown`, never "one".
+fn handle_inbox_consumers(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
+    let Some(topic) = params.get("topic").and_then(|v| v.as_str()).filter(|t| crate::inbox_consumers::applies(t)) else {
+        return ErrorResponse::new(id, -32602, "Missing 'topic' (an inbox:* topic)").into();
+    };
+    let window = params
+        .get("window_s")
+        .and_then(|v| v.as_u64())
+        .unwrap_or(crate::inbox_consumers::DEFAULT_WINDOW_S)
+        .clamp(1, 30 * 86_400);
+    let dir = termlink_session::discovery::runtime_dir();
+    match crate::inbox_consumers::consumers(&dir, topic, window, now_ms()) {
+        Ok(v) => Response::success(id, v).into(),
+        Err(e) => ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("consumer list unreadable (FCW-125): {e}")).into(),
+    }
+}
+
+/// T-3700 (AL-30): bind an inbox to one agent's current card key. Operator only (local socket, Execute,
+/// admin row first); reached through runme.
+fn handle_inbox_bind(id: serde_json::Value, params: &serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
+    if let Some(r) = key_card_local_only(&id, "hub.inbox_bind", peer_addr) {
+        return r;
+    }
+    let field = |n: &str| params.get(n).and_then(|v| v.as_str()).unwrap_or("").to_string();
+    let (topic, agent) = (field("topic"), field("agent"));
+    let dir = termlink_session::discovery::runtime_dir();
+    let key = match crate::key_card::get(&dir, &agent) {
+        Ok(c) => c.and_then(|c| crate::key_card::parse_signing_key(&c.signing_key)),
+        Err(e) => return ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("bind refused: key cards unreadable: {e}")).into(),
+    };
+    let mut log = |row: serde_json::Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
+    match crate::inbox_consumers::bind(&dir, &topic, &agent, key, now_ms(), &mut log) {
+        Ok(b) => Response::success(id, serde_json::json!({"ok": true, "topic": topic, "agent": b.agent, "key_fp": b.key_fp})).into(),
+        Err(e) => ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("bind refused: {e}")).into(),
+    }
+}
+
+/// T-3700: remove an inbox binding (operator only, admin row first).
+fn handle_inbox_unbind(id: serde_json::Value, params: &serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
+    if let Some(r) = key_card_local_only(&id, "hub.inbox_unbind", peer_addr) {
+        return r;
+    }
+    let topic = params.get("topic").and_then(|v| v.as_str()).unwrap_or("").to_string();
+    let dir = termlink_session::discovery::runtime_dir();
+    let mut log = |row: serde_json::Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
+    match crate::inbox_consumers::unbind(&dir, &topic, &mut log) {
+        Ok(was) => Response::success(id, serde_json::json!({"ok": true, "topic": topic, "was_bound": was})).into(),
+        Err(e) => ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("unbind refused: {e}")).into(),
+    }
+}
+
 fn handle_revocation_request(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
     let Some(ident) = crate::hub_identity::current() else {
         return revocation_error(id, "the hub identity is not loaded");
diff --git a/crates/termlink-hub/src/rpc_audit.rs b/crates/termlink-hub/src/rpc_audit.rs
index d2b98bac5..10858f9ec 100644
--- a/crates/termlink-hub/src/rpc_audit.rs
+++ b/crates/termlink-hub/src/rpc_audit.rs
@@ -624,6 +624,9 @@ pub const ADMIN_METHODS: &[&str] = &[
     "hub.revoke",
     "hub.revocation_request",
     "hub.revocation_reissue",
+    // T-3700 (AL-30): an inbox bind changes who may read it.
+    "hub.inbox_bind",
+    "hub.inbox_unbind",
 ];
 
 pub fn is_admin_method(method: &str) -> bool {
diff --git a/crates/termlink-hub/src/server.rs b/crates/termlink-hub/src/server.rs
index 13db66397..fcfeeb588 100644
--- a/crates/termlink-hub/src/server.rs
+++ b/crates/termlink-hub/src/server.rs
@@ -798,11 +798,14 @@ fn hub_method_scope(method: &str) -> PermissionScope {
         | "hub.key_card_get"
         | "hub.key_card_list"
         | "hub.revocation_head"
-        | "hub.revocation_get" => PermissionScope::Observe,
+        | "hub.revocation_get"
+        | "inbox.consumers" => PermissionScope::Observe,
 
         // T-3704 (B-6): card writes and the check setting; also local-socket only (router).
         // T-3703 (B-5): operator revocation and reissue, same rule (operator ruling A 2026-11-18).
         "hub.key_card_put" | "hub.key_card_mode" | "hub.revoke" | "hub.revocation_reissue" => PermissionScope::Execute,
+        // T-3700 (AL-30): inbox bind/unbind, operator only, local socket (router).
+        "hub.inbox_bind" | "hub.inbox_unbind" => PermissionScope::Execute,
         // T-3703: a self-revocation is authenticated by its own signature and a key card.
         "hub.revocation_request" => PermissionScope::Interact,
 
@@ -815,6 +818,7 @@ fn hub_method_scope(method: &str) -> PermissionScope {
         // with no side effects and must require only Observe.
         control::method::CHANNEL_LIST
         | control::method::CHANNEL_SUBSCRIBE
+        | control::method::CHANNEL_FIRST_AT
         | control::method::CHANNEL_RECEIPTS
         | control::method::CHANNEL_CLAIMS
         | control::method::CHANNEL_CLAIMS_SUMMARY
@@ -2468,6 +2472,7 @@ mod tests {
             // Observe — pure reads, no side effects
             (m::CHANNEL_LIST, Observe),
             (m::CHANNEL_SUBSCRIBE, Observe),
+            (m::CHANNEL_FIRST_AT, Observe),
             (m::CHANNEL_RECEIPTS, Observe),
             (m::CHANNEL_CLAIMS, Observe),
             (m::CHANNEL_CLAIMS_SUMMARY, Observe),
diff --git a/crates/termlink-protocol/src/control.rs b/crates/termlink-protocol/src/control.rs
index 56a86a54b..2f91351be 100644
--- a/crates/termlink-protocol/src/control.rs
+++ b/crates/termlink-protocol/src/control.rs
@@ -136,6 +136,11 @@ pub mod method {
     /// Params: `{ topic, cursor?, limit? }` → `{ messages: [...], next_cursor }`.
     pub const CHANNEL_SUBSCRIBE: &str = "channel.subscribe";
 
+    /// T-3700 — the offset and timestamp of the first record at or after `from_offset`, without its
+    /// content. Not an inbox read: it does not record a consumer (liveness checks of another agent's inbox).
+    /// Params: `{ topic, from_offset? }` → `{ offset: n|null, ts: ms|null }`.
+    pub const CHANNEL_FIRST_AT: &str = "channel.first_at";
+
     /// Tier-A — list existing topics (optional prefix filter).
     /// Params: `{ prefix? }` → `{ topics: [{name, last_offset, retention}] }`.
     pub const CHANNEL_LIST: &str = "channel.list";
@@ -583,6 +588,9 @@ pub mod error_code {
     /// revocation (a restore); the head is not served until `hub.revocation_reissue`. A sidecar
     /// counts this as a failed pull, never as an empty list.
     pub const REVOCATION_RECONCILE: i64 = -32048;
+    /// T-3700 (AL-30, FCW-124): the inbox is bound to one agent and this read, claim or ack was not
+    /// proven by that agent's bound key; or the bindings could not be read (fail closed).
+    pub const INBOX_BOUND: i64 = -32049;
 }
 
 /// Default `protocol_version` when the field is missing on the wire.
diff --git a/crates/termlink-session/src/inbox_read.rs b/crates/termlink-session/src/inbox_read.rs
new file mode 100644
index 000000000..2e037cdbd
--- /dev/null
+++ b/crates/termlink-session/src/inbox_read.rs
@@ -0,0 +1,14 @@
+//! T-3700 (arc-011 P-12, AL-30, operator ruling E 2026-11-19): the bytes an inbox reader signs.
+//!
+//! A sidecar names itself on every read and claim of its inbox and signs these bytes with its key-card
+//! key; the hub checks them (`termlink_hub::inbox_consumers`). The domain prefix keeps a read proof from
+//! ever verifying as a post signature; the hub id keeps it from being replayed on another hub.
+
+/// Domain prefix of the read-proof bytes.
+pub const READ_DOMAIN: &str = "termlink-inbox-read/v1";
+
+/// The bytes signed for one read (`method` = `channel.subscribe` with the cursor, or `channel.claim` with the
+/// offset).
+pub fn read_sign_bytes(hub_id: &str, topic: &str, method: &str, position: u64, ts_ms: i64, agent: &str, instance: &str) -> Vec<u8> {
+    format!("{READ_DOMAIN}\n{hub_id}\n{topic}\n{method}\n{position}\n{ts_ms}\n{agent}\n{instance}").into_bytes()
+}
diff --git a/crates/termlink-sidecar/src/api.rs b/crates/termlink-sidecar/src/api.rs
index 1d95eca91..111992802 100644
--- a/crates/termlink-sidecar/src/api.rs
+++ b/crates/termlink-sidecar/src/api.rs
@@ -95,6 +95,8 @@ fn op_spec(op: &str) -> Option<(Scope, &'static [&'static str])> {
         // P-26 (AL-39, K-20): the PreToolUse hook reports one vendor-channel use.
         "vendor-use" => (Scope::Send, &["session", "tool", "boot_id", "mono_ns"]),
         "status" => (Scope::Status, &["msg_id", "conv"]),
+        // T-3700 (ruling D): read the own inbox from `from` (default: the receive cursor), changing nothing
+        "peek" => (Scope::Status, &["from", "limit"]),
         "delivery" => (Scope::Admin, &["state", "why"]),
         "held" => (Scope::Admin, &["body"]),
         "allow" => (Scope::Admin, &["key", "why"]),
@@ -481,6 +483,7 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
             }
             v
         }
+        "peek" => peek(obj, sender, store),
         "send" => bound_send(metered_send(send(obj, sender, book, agent), obj, ctx.meter), obj, ctx.deliverer, store),
         "delivery" => delivery(obj, store, who),
         "held" => held(obj, store),
@@ -517,6 +520,25 @@ fn dispatch_inner(req: &Value, scope: Scope, store: &Store, agent: &str, hub_id:
     }
 }
 
+/// T-3700 (ruling D): a diagnostic look at the own inbox through this sidecar's signed reader.
+fn peek(obj: &serde_json::Map<String, Value>, sender: Option<&Sender>, store: &Store) -> Value {
+    let Some(sender) = sender else {
+        return err("not-configured", "this sidecar has no hub path configured", "start it with a hub address and a hub id");
+    };
+    let cursor = || -> u64 {
+        store.read_state(crate::pull::CURSOR_STATE).ok().flatten().and_then(|s| s.trim().parse().ok()).unwrap_or(0)
+    };
+    let from = obj.get("from").and_then(Value::as_u64).unwrap_or_else(cursor);
+    let limit = obj.get("limit").and_then(Value::as_u64).unwrap_or(20);
+    match sender.peek_inbox(from, limit) {
+        Ok(mut v) => {
+            v["ok"] = Value::Bool(true);
+            v
+        }
+        Err(e) => err("hub", &format!("peek: {e}"), "check `status` (hub, revocation, consumers)"),
+    }
+}
+
 /// B-27 S3: a message the session's agent sent moves that session's S3. Only a
 /// send that succeeded and names its session counts (D-3 in T-3687).
 fn metered_send(resp: Value, obj: &serde_json::Map<String, Value>, meter: Option<&crate::meter::Meter>) -> Value {
@@ -1255,6 +1277,8 @@ fn status_at(
         "hub_signing_key": crate::hub_key_pin::status(store, hub_id),
         // T-3703 (OBS-41, design 5.5): `revocation` and the `refused_revoked` count
         "revocation": crate::revocation::status(store),
+        // T-3700 (AL-30, OBS-56): who consumes this inbox; `unknown`, never "one", when unreadable
+        "consumers": crate::consumers::status(store),
         "stages": STAGES,
         "rows": rows,
         "messages": msgs,
diff --git a/crates/termlink-sidecar/src/consumers.rs b/crates/termlink-sidecar/src/consumers.rs
new file mode 100644
index 000000000..de11f0a24
--- /dev/null
+++ b/crates/termlink-sidecar/src/consumers.rs
@@ -0,0 +1,189 @@
+//! T-3700 (arc-011 P-12, B-2, 06a AL-30): the sidecar's per-tick check of who consumes its inbox.
+//!
+//! The hub lists every reader of the inbox (`inbox.consumers`), `proven` (signed by its key-card key) or
+//! `claimed` (operator ruling E, 2026-11-19). Any reader but this sidecar within the V-11 window (24 h) is
+//! "inbox consumed by two identities: A, B" in status and a needs-attention entry, coalesced per
+//! (inbox, reader, UTC day). The same agent at another instance is the V-12 second copy. A list that
+//! cannot be read is `unknown`, never "one" (FCW-125).
+
+use serde_json::{Value, json};
+
+use crate::store::Store;
+
+/// Store state holding the last status line (JSON `{consumers, bound}`).
+pub const STATUS_STATE: &str = "inbox-consumers";
+/// V-11: the window the hub is asked about.
+pub const WINDOW_S: u64 = 86_400;
+/// How often the pass loop asks (well inside E-66's 60 s).
+pub const CHECK_EVERY_MS: u64 = 30_000;
+/// The instance the hub records for an ack (a receipt post names no instance).
+pub const ACK_INSTANCE: &str = "-";
+/// Attention kind for a second consumer.
+pub const ATTENTION_KIND: &str = "inbox-second-consumer";
+/// JSON-RPC "method not found".
+const METHOD_NOT_FOUND: i64 = -32601;
+
+/// One reader as the hub reports it.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Seen {
+    pub agent: String,
+    pub instance: String,
+    pub strength: String,
+    pub why: String,
+}
+
+impl Seen {
+    fn shown(&self) -> String {
+        let s = if self.why.is_empty() { self.strength.clone() } else { format!("{}: {}", self.strength, self.why) };
+        format!("{} ({s})", self.agent)
+    }
+}
+
+/// What one check concluded.
+#[derive(Debug, Clone, PartialEq, Eq)]
+pub struct Verdict {
+    /// The `consumers` status line.
+    pub line: String,
+    /// Readers other than this sidecar (each is a needs-attention entry).
+    pub others: Vec<Seen>,
+    /// The agent the inbox is bound to, if any.
+    pub bound: Option<String>,
+}
+
+/// Classify the hub's answer to `inbox.consumers` (`Err((code, message))` when the call failed).
+pub fn verdict(me_agent: &str, me_instance: &str, answer: Result<&Value, (i64, &str)>) -> Verdict {
+    let unknown = |why: String| Verdict { line: format!("unknown ({why})"), others: vec![], bound: None };
+    let v = match answer {
+        Ok(v) => v,
+        Err((METHOD_NOT_FOUND, _)) => return unknown("hub predates the consumer list".into()),
+        Err((_, m)) => return unknown(m.to_string()),
+    };
+    let Some(list) = v.get("consumers").and_then(Value::as_array) else {
+        // FCW-125: a malformed answer is not "one"
+        return unknown("the hub's answer holds no consumer list".into());
+    };
+    let bound = v.get("bound").and_then(|b| b.get("agent")).and_then(Value::as_str).map(str::to_string);
+    let mut mine: Vec<Seen> = Vec::new();
+    let mut others: Vec<Seen> = Vec::new();
+    for e in list {
+        let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
+        let seen = Seen { agent: s("agent"), instance: s("instance"), strength: s("strength"), why: s("why") };
+        let is_me = seen.agent == me_agent && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);
+        if is_me { mine.push(seen) } else { others.push(seen) }
+    }
+    let line = if !others.is_empty() {
+        let me_shown = mine
+            .iter()
+            .find(|m| m.instance == me_instance)
+            .or(mine.first())
+            .map(Seen::shown)
+            .unwrap_or_else(|| format!("{me_agent} (not seen)"));
+        let n = others.len() + 1;
+        let count = if n == 2 { "two".to_string() } else { n.to_string() };
+        let names: Vec<String> = std::iter::once(me_shown)
+            .chain(others.iter().map(|o| {
+                if o.agent == me_agent {
+                    format!("{} [second copy, instance {}]", o.shown(), o.instance)
+                } else {
+                    o.shown()
+                }
+            }))
+            .collect();
+        format!("inbox consumed by {count} identities: {}", names.join(", "))
+    } else {
+        match mine.iter().find(|m| m.instance == me_instance).or(mine.first()) {
+            Some(m) if m.strength == "proven" => "one (proven)".to_string(),
+            Some(m) => format!("one ({}: {})", m.strength, if m.why.is_empty() { "?" } else { &m.why }),
+            None => "no reader in the window".to_string(),
+        }
+    };
+    Verdict { line, others, bound }
+}
+
+/// Record a verdict: the status line, and one needs-attention entry per other reader per UTC day.
+pub fn record(store: &Store, topic: &str, v: &Verdict, now_ms: u64) {
+    let state = json!({"consumers": v.line, "bound": v.bound}).to_string();
+    if let Err(e) = store.write_state(STATUS_STATE, &state) {
+        eprintln!("termlink-sidecar: consumers status not written: {e}");
+    }
+    let day = now_ms / 86_400_000;
+    for o in &v.others {
+        let key = format!("{topic}|{}|{}|{day}", o.agent, o.instance);
+        let detail = format!("{} (instance {}) also consumes {topic}: {}", o.shown(), o.instance, v.line);
+        if let Err(e) = store.attention_coalesced(ATTENTION_KIND, &key, &detail) {
+            eprintln!("termlink-sidecar: needs attention ({ATTENTION_KIND}): {detail} [could not record it: {e}]");
+        }
+    }
+}
+
+/// The `consumers` status (and `bound`), `unknown (no check yet)` before the first check.
+pub fn status(store: &Store) -> Value {
+    store
+        .read_state(STATUS_STATE)
+        .ok()
+        .flatten()
+        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
+        .unwrap_or_else(|| json!({"consumers": "unknown (no check yet)", "bound": null}))
+}
+
+#[cfg(test)]
+mod tests {
+    use super::*;
+
+    fn e(agent: &str, instance: &str, strength: &str, why: &str) -> Value {
+        json!({"agent": agent, "instance": instance, "strength": strength, "why": why})
+    }
+
+    fn v(list: Vec<Value>) -> Value {
+        json!({"consumers": list, "bound": null})
+    }
+
+    #[test]
+    fn only_me_proven_is_one() {
+        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "-", "proven", "")]);
+        let r = verdict("fx-a", "w1", Ok(&a));
+        assert_eq!(r.line, "one (proven)");
+        assert!(r.others.is_empty());
+    }
+
+    #[test]
+    fn a_second_identity_is_named_with_its_strength() {
+        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
+        let r = verdict("fx-a", "w1", Ok(&a));
+        assert_eq!(r.line, "inbox consumed by two identities: fx-a (proven), fx-z (claimed: unsigned)");
+        assert_eq!(r.others.len(), 1);
+    }
+
+    #[test]
+    fn same_agent_other_instance_is_a_second_copy() {
+        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "w2", "proven", "")]);
+        let r = verdict("fx-a", "w1", Ok(&a));
+        assert!(r.line.contains("second copy, instance w2"), "{}", r.line);
+    }
+
+    #[test]
+    fn unreadable_is_unknown_never_one() {
+        assert!(verdict("fx-a", "w1", Err((-32049, "consumer list unreadable"))).line.starts_with("unknown ("));
+        assert_eq!(verdict("fx-a", "w1", Err((-32601, "no such method"))).line, "unknown (hub predates the consumer list)");
+        assert!(verdict("fx-a", "w1", Ok(&json!({"bound": null}))).line.starts_with("unknown ("));
+    }
+
+    #[test]
+    fn my_skewed_clock_shows_as_claimed() {
+        let a = v(vec![e("fx-a", "w1", "claimed", "clock skew")]);
+        assert_eq!(verdict("fx-a", "w1", Ok(&a)).line, "one (claimed: clock skew)");
+    }
+
+    #[test]
+    fn attention_once_per_reader_per_day() {
+        let est = crate::testkit::Estate::create_without_hub().unwrap();
+        let store = Store::open(&est.store_root(), "fx-a").unwrap();
+        let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
+        let r = verdict("fx-a", "w1", Ok(&a));
+        record(&store, "inbox:h/fx-a", &r, 1_000);
+        record(&store, "inbox:h/fx-a", &r, 2_000);
+        let n = store.attention_entries().iter().filter(|l| l.contains(ATTENTION_KIND)).count();
+        assert_eq!(n, 1);
+        assert_eq!(status(&store)["consumers"], r.line);
+    }
+}
diff --git a/crates/termlink-sidecar/src/liveness.rs b/crates/termlink-sidecar/src/liveness.rs
index cdf369573..e90688739 100644
--- a/crates/termlink-sidecar/src/liveness.rs
+++ b/crates/termlink-sidecar/src/liveness.rs
@@ -211,19 +211,33 @@ pub fn read(hub: &dyn Hub, topic: &str, inbox: &str) -> Read {
     };
     let oldest_waiting_ms = match &note {
         Err(_) => None,
-        Ok(n) => match hub.call(method::CHANNEL_SUBSCRIBE, json!({"topic": inbox, "cursor": n.drained_up_to, "limit": 1})) {
-            Ok(v) => v
-                .get("messages")
-                .and_then(Value::as_array)
-                .and_then(|m| m.iter().find(|e| e.get("offset").and_then(Value::as_u64).is_some_and(|o| o >= n.drained_up_to)))
-                .map(|e| e.get("ts").and_then(Value::as_u64).unwrap_or(0)),
+        // T-3700: the timestamp only, so this check is not a consumer of the main's inbox (AL-30)
+        Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({"topic": inbox, "from_offset": n.drained_up_to})) {
+            Ok(v) => v.get("ts").and_then(Value::as_u64).filter(|_| v.get("offset").and_then(Value::as_u64).is_some()),
             Err((Some(code), _)) if code == error_code::CHANNEL_TOPIC_UNKNOWN => None,
-            Err((_, e)) => return Read::Unreadable(format!("channel.subscribe {inbox}: {e}")),
+            // a hub that predates channel.first_at: the one-message read (recorded as a consumer there)
+            Err(_) => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {
+                Ok(t) => t,
+                Err(e) => return Read::Unreadable(e),
+            },
         },
     };
     Read::Note { note, signer, oldest_waiting_ms }
 }
 
+/// The pre-T-3700 way to find the oldest waiting message: read one message at `from`.
+fn oldest_by_subscribe(hub: &dyn Hub, inbox: &str, from: u64) -> Result<Option<u64>, String> {
+    Ok(match hub.call(method::CHANNEL_SUBSCRIBE, json!({"topic": inbox, "cursor": from, "limit": 1})) {
+        Ok(v) => v
+            .get("messages")
+            .and_then(Value::as_array)
+            .and_then(|m| m.iter().find(|e| e.get("offset").and_then(Value::as_u64).is_some_and(|o| o >= from)))
+            .map(|e| e.get("ts").and_then(Value::as_u64).unwrap_or(0)),
+        Err((Some(code), _)) if code == error_code::CHANNEL_TOPIC_UNKNOWN => None,
+        Err((_, e)) => return Err(format!("channel.subscribe {inbox}: {e}")),
+    })
+}
+
 /// Move the alarm latch for a verdict; returns the needs-attention entry written, if any.
 /// The latch lives in the store, so a restart neither repeats nor forgets an alarm.
 pub fn alarm(store: &Store, main: &str, v: &Liveness) -> Option<&'static str> {
@@ -478,6 +492,22 @@ mod tests {
         }
     }
 
+    /// T-3700 (AL-30): with `channel.first_at` the main's inbox is never read for content, so this
+    /// check is not a consumer of it; the subscribe path is only the fallback for older hubs.
+    #[test]
+    fn read_uses_first_at_and_never_reads_the_inbox_content() {
+        let n = note(NOW, 2);
+        let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 999)]})));
+        f.replies.insert(("channel.first_at".into(), "I".into()), Ok(json!({"offset": 2, "ts": 1234})));
+        assert_eq!(read(&f, "L", "I"), Read::Note { note: Ok(n.clone()), signer: "fp-main".into(), oldest_waiting_ms: Some(1234) });
+        assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"), "{:?}", f.calls.borrow());
+        // nothing waiting at or after the offset
+        let mut f = fake(&n, Ok(json!({"messages": []})));
+        f.replies.insert(("channel.first_at".into(), "I".into()), Ok(json!({"offset": null, "ts": null})));
+        assert_eq!(read(&f, "L", "I"), Read::Note { note: Ok(n), signer: "fp-main".into(), oldest_waiting_ms: None });
+        assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"));
+    }
+
     /// A fake hub: canned replies per (method, topic).
     struct Fake {
         replies: BTreeMap<(String, String), HubResult>,
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index eb0974adb..227dc2516 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -113,6 +113,8 @@ struct Inner {
 
 #[derive(Default)]
 struct PassMemory {
+    /// T-3700: when the consumer list was last checked (ms).
+    consumers_checked_ms: u64,
     held: Option<String>,
     /// The last per-message failure written to attention, so a stuck message
     /// is reported once rather than every pass.
@@ -241,6 +243,13 @@ impl Receiver {
     pub fn revocation_state(&self) -> crate::revocation::State {
         self.inner.revocation.lock().unwrap_or_else(|p| p.into_inner()).state().clone()
     }
+
+    /// T-3700: run the consumer check now, whatever the throttle (tests, and an operator's `status` refresh).
+    pub fn check_consumers_now(&self) {
+        let mut mem = self.inner.pass.lock().unwrap_or_else(|p| p.into_inner());
+        mem.consumers_checked_ms = 0;
+        self.inner.check_consumers(&mut mem);
+    }
 }
 
 impl Drop for Receiver {
@@ -386,6 +395,23 @@ impl Inner {
         t.tick(&self.store, version, &rpc, std::time::Instant::now(), Some(&self.sender));
     }
 
+    /// T-3700 (AL-30, E-66): ask the hub who consumed this inbox within the V-11 window and record it.
+    fn check_consumers(&self, mem: &mut PassMemory) {
+        let now = crate::store::now_ms();
+        if mem.consumers_checked_ms != 0 && now < mem.consumers_checked_ms + crate::consumers::CHECK_EVERY_MS {
+            return;
+        }
+        mem.consumers_checked_ms = now;
+        let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
+        let answer = self.rpc("inbox.consumers", json!({"topic": topic, "window_s": crate::consumers::WINDOW_S}));
+        let v = match &answer {
+            Ok(RpcResponse::Success(s)) => crate::consumers::verdict(&self.cfg.agent, &self.cfg.instance, Ok(&s.result)),
+            Ok(RpcResponse::Error(e)) => crate::consumers::verdict(&self.cfg.agent, &self.cfg.instance, Err((e.error.code, &e.error.message))),
+            Err(e) => crate::consumers::verdict(&self.cfg.agent, &self.cfg.instance, Err((0, &format!("hub unreachable: {e}")))),
+        };
+        crate::consumers::record(&self.store, &topic, &v, now);
+    }
+
     fn cursor(&self) -> u64 {
         self.store
             .read_state(CURSOR_STATE)
@@ -445,13 +471,17 @@ impl Inner {
         };
         // T-3703 (design 5.4): the revocation head is compared before any message is handled.
         self.pull_revocations(&version);
+        // T-3700 (AL-30): who else consumes this inbox (at most every CHECK_EVERY_MS)
+        self.check_consumers(mem);
         let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
         let mut handled = 0;
         loop {
             let cursor = self.cursor();
             let resp = match self.rpc(
                 method::CHANNEL_SUBSCRIBE,
-                json!({"topic": topic, "cursor": cursor, "limit": self.cfg.page}),
+                // T-3700 (AL-30, ruling E): a named, signed read of our own inbox (proven consumer)
+                json!({"topic": topic, "cursor": cursor, "limit": self.cfg.page,
+                       "reader": self.sender.read_proof(&topic, method::CHANNEL_SUBSCRIBE, cursor)}),
             ) {
                 Ok(r) => r,
                 Err(_) => return PassResult::Waiting,
diff --git a/crates/termlink-sidecar/src/send.rs b/crates/termlink-sidecar/src/send.rs
index 7473be176..8ab4b13d9 100644
--- a/crates/termlink-sidecar/src/send.rs
+++ b/crates/termlink-sidecar/src/send.rs
@@ -301,6 +301,21 @@ impl Sender {
         self.inner.post_stage(to_agent, st)
     }
 
+    /// T-3700 (AL-30, ruling E): the signed `reader` object for one read or claim of `topic` at
+    /// `position`, so the hub lists this sidecar as a proven consumer of its own inbox.
+    pub fn read_proof(&self, topic: &str, method: &str, position: u64) -> Value {
+        let c = &self.inner.cfg;
+        let ts = crate::store::now_ms() as i64;
+        let bytes = termlink_session::inbox_read::read_sign_bytes(&c.declared_hub_id, topic, method, position, ts, &c.agent, &c.instance);
+        json!({
+            "agent": c.agent,
+            "instance": c.instance,
+            "ts_ms": ts,
+            "signer_pk": c.identity.public_key_hex(),
+            "sig": c.identity.sign_hex(&bytes),
+        })
+    }
+
     /// The agent's own fingerprint.
     pub fn fingerprint(&self) -> &str {
         self.inner.cfg.identity.fingerprint()
@@ -361,6 +376,24 @@ impl Sender {
     pub fn post_note(&self, topic: &str, msg_type: &str, payload: &[u8], retention: Value) -> Result<(), String> {
         self.inner.post_note(topic, msg_type, payload, retention)
     }
+
+    /// T-3700 (operator ruling D, 2026-11-19): read this agent's own inbox from `from` without changing
+    /// anything — no cursor, stage or store write. The read is signed, so the hub lists it as this
+    /// sidecar (never a second consumer) and lets it through a bind. Refused on a hub that is not the
+    /// declared one, like every other call.
+    pub fn peek_inbox(&self, from: u64, limit: u64) -> Result<Value, String> {
+        self.inner.check_pin().map_err(|e| format!("{e:?}"))?;
+        let topic = inbox_topic(&self.inner.cfg.declared_hub_id, &self.inner.cfg.agent);
+        let params = json!({"topic": topic, "cursor": from, "limit": limit.clamp(1, 100),
+                            "reader": self.read_proof(&topic, termlink_protocol::control::method::CHANNEL_SUBSCRIBE, from)});
+        match self.inner.rpc(termlink_protocol::control::method::CHANNEL_SUBSCRIBE, params)? {
+            RpcResponse::Success(s) => Ok(json!({"topic": topic, "from": from, "result": s.result})),
+            RpcResponse::Error(e) if e.error.code == termlink_protocol::control::error_code::CHANNEL_TOPIC_UNKNOWN => {
+                Ok(json!({"topic": topic, "from": from, "result": {"messages": [], "next_cursor": from}}))
+            }
+            RpcResponse::Error(e) => Err(format!("channel.subscribe: {} ({})", e.error.message, e.error.code)),
+        }
+    }
 }
 
 impl Drop for Sender {
```

## Tests
```rust
//! T-3700 unit tests for the inbox consumer list (AL-30, rulings E and D).

use super::*;
use ed25519_dalek::{Signer, SigningKey};

const HUB: &str = "cacc73ea32b121dd";
const T: &str = "inbox:cacc73ea32b121dd/fx-a";
const NOW: i64 = 1_800_000_000_000;

fn key(n: u8) -> SigningKey {
    SigningKey::from_bytes(&[n; 32])
}

fn hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02x}")).collect()
}

fn proof(k: &SigningKey, agent: &str, instance: &str, topic: &str, pos: u64, ts: i64) -> ReadProof {
    let bytes = read_sign_bytes(HUB, topic, "channel.subscribe", pos, ts, agent, instance);
    ReadProof {
        agent: agent.into(),
        instance: instance.into(),
        ts_ms: ts,
        signer_pk: Some(hex(k.verifying_key().as_bytes())),
        sig: Some(hex(&k.sign(&bytes).to_bytes())),
    }
}

/// Cards: fx-a -> key 1, fx-b -> key 2. Revoked: whatever `rev` holds.
fn with_lookups<R>(rev: BTreeSet<String>, f: impl FnOnce(&Lookups<'_>) -> R) -> R {
    let card_key = |a: &str| -> Result<Option<[u8; 32]>, String> {
        Ok(match a {
            "fx-a" => Some(*key(1).verifying_key().as_bytes()),
            "fx-b" => Some(*key(2).verifying_key().as_bytes()),
            _ => None,
        })
    };
    let revoked = move || -> Result<BTreeSet<String>, String> { Ok(rev.clone()) };
    let l = Lookups { hub_id: Some(HUB), card_key: &card_key, revoked: &revoked };
    f(&l)
}

fn assess_one(p: Option<&ReadProof>) -> Result<Reader, Refusal> {
    with_lookups(BTreeSet::new(), |l| assess(T, "channel.subscribe", 7, p, "local", NOW, l))
}

#[test]
fn signed_read_by_carded_key_is_proven() {
    let r = assess_one(Some(&proof(&key(1), "fx-a", "fx-a.w1", T, 7, NOW))).unwrap();
    assert_eq!(r.strength, Strength::Proven);
    assert_eq!(r.agent, "fx-a");
    assert_eq!(r.key_fp.as_deref(), Some(termlink_session::signed_state::key_fingerprint(&key(1).verifying_key()).as_str()));
}

#[test]
fn no_reader_is_unnamed_claimed_with_transport() {
    let r = assess_one(None).unwrap();
    assert_eq!((r.agent.as_str(), r.instance.as_str(), r.strength.clone()), (UNNAMED, "local", Strength::Claimed));
}

#[test]
fn named_unsigned_reader_is_claimed() {
    let p = ReadProof { agent: "fx-a".into(), instance: "x".into(), ts_ms: NOW, signer_pk: None, sig: None };
    let r = assess_one(Some(&p)).unwrap();
    assert_eq!(r.strength, Strength::Claimed);
    assert_eq!(r.why, "unsigned");
}

#[test]
fn bad_signature_is_refused_never_downgraded() {
    let mut p = proof(&key(1), "fx-a", "i", T, 7, NOW);
    p.ts_ms += 1; // signature no longer covers the bytes
    let e = assess_one(Some(&p)).unwrap_err();
    assert_eq!(e.code, error_code::CHANNEL_SIGNATURE_INVALID);
}

#[test]
fn signature_for_another_topic_or_position_does_not_verify() {
    let p = proof(&key(1), "fx-a", "i", "inbox:cacc73ea32b121dd/fx-b", 7, NOW);
    assert!(assess_one(Some(&p)).is_err());
    let p = proof(&key(1), "fx-a", "i", T, 8, NOW);
    assert!(assess_one(Some(&p)).is_err());
}

#[test]
fn signed_by_a_key_not_on_the_named_card_is_refused() {
    // fx-z signs with its own key but names fx-a
    let e = assess_one(Some(&proof(&key(9), "fx-a", "i", T, 7, NOW))).unwrap_err();
    assert_eq!(e.code, error_code::CHANNEL_KEY_CARD_MISMATCH);
}

#[test]
fn signed_without_a_card_is_claimed() {
    let r = assess_one(Some(&proof(&key(9), "fx-z", "i", T, 7, NOW))).unwrap();
    assert_eq!((r.strength, r.why.as_str()), (Strength::Claimed, "no key card"));
}

#[test]
fn revoked_key_is_refused() {
    let fp = termlink_session::signed_state::key_fingerprint(&key(1).verifying_key());
    let e = with_lookups([fp].into_iter().collect(), |l| {
        assess(T, "channel.subscribe", 7, Some(&proof(&key(1), "fx-a", "i", T, 7, NOW)), "local", NOW, l)
    })
    .unwrap_err();
    assert_eq!(e.code, error_code::KEY_REVOKED);
}

#[test]
fn skewed_clock_is_claimed_not_refused() {
    let p = proof(&key(1), "fx-a", "i", T, 7, NOW - PROOF_WINDOW_MS - 1);
    let r = assess_one(Some(&p)).unwrap();
    assert_eq!((r.strength, r.why.as_str()), (Strength::Claimed, "clock skew"));
    // inside the window proves
    let p = proof(&key(1), "fx-a", "i", T, 7, NOW - PROOF_WINDOW_MS);
    assert_eq!(assess_one(Some(&p)).unwrap().strength, Strength::Proven);
}

#[test]
fn no_hub_identity_means_claimed() {
    let card_key = |_: &str| -> Result<Option<[u8; 32]>, String> { Ok(None) };
    let revoked = || -> Result<BTreeSet<String>, String> { Ok(BTreeSet::new()) };
    let l = Lookups { hub_id: None, card_key: &card_key, revoked: &revoked };
    let r = assess(T, "channel.subscribe", 7, Some(&proof(&key(1), "fx-a", "i", T, 7, NOW)), "local", NOW, &l).unwrap();
    assert_eq!(r.strength, Strength::Claimed);
}

#[test]
fn parse_reader_rules() {
    assert_eq!(parse_reader(&serde_json::json!({})).unwrap(), None);
    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "Bad Name"}})).is_err());
    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "sig": "00"}})).is_err());
    let p = parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "instance": "i", "ts_ms": 5}})).unwrap().unwrap();
    assert_eq!((p.agent.as_str(), p.ts_ms), ("fx-a", 5));
}

// ---- record / consumers / bind ----

fn dir() -> tempfile::TempDir {
    tempfile::tempdir().unwrap()
}

fn proven(agent: &str, instance: &str, n: u8) -> Reader {
    Reader {
        agent: agent.into(),
        instance: instance.into(),
        strength: Strength::Proven,
        why: String::new(),
        key_fp: Some(termlink_session::signed_state::key_fingerprint(&key(n).verifying_key())),
    }
}

fn claimed_r(agent: &str) -> Reader {
    claimed(agent, "local", "unsigned")
}

fn rows() -> (Vec<serde_json::Value>, impl FnMut(serde_json::Value) -> Result<(), String>) {
    (Vec::new(), |_r| Ok(()))
}

fn listed(d: &Path, topic: &str, now: u64) -> Vec<(String, String, String)> {
    let v = consumers(d, topic, DEFAULT_WINDOW_S, now).unwrap();
    v["consumers"]
        .as_array()
        .unwrap()
        .iter()
        .map(|e| {
            (
                e["agent"].as_str().unwrap().to_string(),
                e["instance"].as_str().unwrap().to_string(),
                e["strength"].as_str().unwrap().to_string(),
            )
        })
        .collect()
}

#[test]
fn every_action_records_including_read_only() {
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap(); // reads only, never acks (FCW-209)
    record(d.path(), T, &proven("fx-b", ACK_INSTANCE, 2), Action::Ack, n, &mut log).unwrap();
    let l = listed(d.path(), T, n);
    assert_eq!(l.len(), 3);
    assert!(l.contains(&("fx-z".into(), "local".into(), "claimed".into())));
    assert!(l.contains(&("fx-a".into(), "w1".into(), "proven".into())));
}

#[test]
fn window_hides_old_readers() {
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n - 2 * 86_400_000, &mut log).unwrap();
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
    assert_eq!(listed(d.path(), T, n).len(), 1);
}

#[test]
fn seventeenth_reader_evicts_the_oldest_with_a_row() {
    let d = dir();
    let mut logged = Vec::new();
    let mut log = |r: serde_json::Value| {
        logged.push(r);
        Ok(())
    };
    let n = NOW as u64;
    for i in 0..17u64 {
        record(d.path(), T, &claimed_r(&format!("fx-{i}")), Action::Read, n + i, &mut log).unwrap();
    }
    let l = listed(d.path(), T, n + 100);
    assert_eq!(l.len(), MAX_PER_TOPIC);
    assert!(!l.iter().any(|e| e.0 == "fx-0"));
    assert!(logged.iter().any(|r| r["event"] == "inbox-consumer-evicted" && r["agent"] == "fx-0"));
}

#[test]
fn state_survives_a_restart() {
    let d = dir();
    let (_, mut log) = rows();
    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, NOW as u64, &mut log).unwrap();
    reset(d.path());
    assert_eq!(listed(d.path(), T, NOW as u64).len(), 1);
}

#[test]
fn unreadable_list_is_an_error_never_empty() {
    let d = dir();
    std::fs::write(consumers_path(d.path()), b"{not json").unwrap();
    reset(d.path());
    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, NOW as u64).is_err());
}

#[test]
fn bind_refuses_everyone_but_the_bound_key() {
    let d = dir();
    let mut logged = Vec::new();
    let n = NOW as u64;
    {
        let mut log = |r: serde_json::Value| {
            logged.push(r);
            Ok(())
        };
        bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), n, &mut log).unwrap();
        // the bound agent, proven with the bound key
        record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
        // another agent, proven
        assert_eq!(record(d.path(), T, &proven("fx-b", "w1", 2), Action::Ack, n, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
        // the bound agent's NAME, unsigned (ruling E: a name alone never passes)
        let named = claimed("fx-a", "x", "unsigned");
        assert_eq!(record(d.path(), T, &named, Action::Read, n, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
        // the bound agent's name, proven but by another key (a rotated card needs a re-bind)
        assert!(record(d.path(), T, &proven("fx-a", "w1", 3), Action::Claim, n, &mut log).is_err());
        // a key-less peek (ruling D: no exception)
        assert!(record(d.path(), T, &claimed(UNNAMED, "local", "no reader named"), Action::Read, n, &mut log).is_err());
    }
    assert_eq!(logged[0]["event"], "inbox-bind");
    assert!(logged.iter().any(|r| r["event"] == "inbox-bound-refused" && r["reader"] == "fx-b"));
    // other inboxes are not affected
    let (_, mut log) = rows();
    record(d.path(), "inbox:cacc73ea32b121dd/fx-b", &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
}

#[test]
fn bind_needs_a_card_and_an_inbox_topic() {
    let d = dir();
    let (_, mut log) = rows();
    assert!(bind(d.path(), T, "fx-a", None, NOW as u64, &mut log).is_err());
    assert!(bind(d.path(), "chat:x", "fx-a", Some(*key(1).verifying_key().as_bytes()), NOW as u64, &mut log).is_err());
}

#[test]
fn bind_writes_its_admin_row_first() {
    let d = dir();
    let mut failing = |_r: serde_json::Value| Err("admin log full".to_string());
    assert!(bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), NOW as u64, &mut failing).is_err());
    assert!(!bindings_path(d.path()).exists(), "no row, no bind");
}

#[test]
fn unbind_reopens_and_unreadable_bindings_fail_closed() {
    let d = dir();
    let (_, mut log) = rows();
    let n = NOW as u64;
    bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), n, &mut log).unwrap();
    assert!(unbind(d.path(), T, &mut log).unwrap());
    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
    std::fs::write(bindings_path(d.path()), b"garbage").unwrap();
    assert_eq!(record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
}

#[test]
fn ack_reader_from_cards() {
    let cards = || -> Result<Vec<(String, [u8; 32])>, String> { Ok(vec![("fx-a".into(), *key(1).verifying_key().as_bytes())]) };
    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(1).verifying_key(), &cards, l));
    assert_eq!((r.agent.as_str(), r.strength), ("fx-a", Strength::Proven));
    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(9).verifying_key(), &cards, l));
    assert_eq!(r.strength, Strength::Claimed);
}
//! T-3700 (arc-011 P-12, B-2, 06a AL-30) at L4 on a real hub: E-66 (a second consumer is flagged, refused
//! after bind; 17 readers evict the oldest with a row; a second copy is seen within one check), E-66.6 (a
//! read-only consumer counts), E-133 (the per-tick check fires), E-169 B-2 part (an unreadable list is
//! `unknown`, never "one"), operator rulings E (proven/claimed, bind by key) and D (peek through the sidecar).

use std::sync::Arc;
use std::time::Duration;

use base64::Engine as _;
use serde_json::{Value, json};
use termlink_session::agent_identity::Identity;
use termlink_sidecar::policy::BuiltIn;
use termlink_sidecar::pull::{ReceiveConfig, Receiver};
use termlink_sidecar::send::{SendConfig, Sender};
use termlink_sidecar::store::Store;
use termlink_sidecar::testkit::Estate;
use termlink_sidecar::wire::Roster;

const INBOX_BOUND: i64 = -32049;

fn rpc(e: &Estate, method: &str, params: Value) -> Value {
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    let resp = rt
        .block_on(termlink_session::client::rpc_call_addr_with_timeout(&e.hub_addr(), method, params, Duration::from_secs(10)))
        .unwrap();
    serde_json::to_value(&resp).unwrap()
}

struct Side {
    store: Arc<Store>,
    sender: Arc<Sender>,
    rx: Receiver,
}

fn side(e: &Estate, id: &str, who: &str, instance: &str) -> Side {
    let agent = format!("fx-{who}");
    let root = e.store_root().join(instance);
    std::fs::create_dir_all(&root).unwrap();
    let store = Arc::new(Store::open(&root, &agent).unwrap());
    let sender = Arc::new(
        Sender::new(
            SendConfig {
                agent: agent.clone(),
                instance: instance.to_string(),
                identity: e.identity(who).unwrap(),
                hub: e.hub_addr(),
                declared_hub_id: id.to_string(),
                queue_path: e.root().join(format!("outbox-{instance}.sqlite")),
                timeout: Duration::from_secs(10),
                pump_interval: None,
            },
            store.clone(),
        )
        .unwrap(),
    );
    let roster = Roster::new();
    let rx = Receiver::new(
        ReceiveConfig {
            agent: agent.clone(),
            instance: instance.to_string(),
            hub: e.hub_addr(),
            declared_hub_id: id.to_string(),
            roster: roster.clone(),
            timeout: Duration::from_secs(10),
            page: 50,
            poll_interval: None,
        },
        store.clone(),
        sender.clone(),
        Box::new(BuiltIn::new(roster)),
    )
    .unwrap();
    Side { store, sender, rx }
}

fn put_card(e: &Estate, agent: &str, who: &str) {
    let r = rpc(e, "hub.key_card_put", json!({"agent": agent, "project": "proj-p",
        "signing_key": format!("ed25519-pub:{}", e.identity(who).unwrap().public_key_hex())}));
    assert_eq!(r["result"]["ok"], true, "{r}");
}

fn line(store: &Store) -> String {
    termlink_sidecar::consumers::status(store)["consumers"].as_str().unwrap_or("").to_string()
}

fn attention_count(store: &Store) -> usize {
    store.attention_entries().iter().filter(|l| l.contains(termlink_sidecar::consumers::ATTENTION_KIND)).count()
}

/// A raw read of `topic` by a client that is not a sidecar, with an optional `reader` object.
fn raw_read(e: &Estate, topic: &str, reader: Option<Value>) -> Value {
    let mut p = json!({"topic": topic, "cursor": 0, "limit": 5});
    if let Some(r) = reader {
        p["reader"] = r;
    }
    rpc(e, "channel.subscribe", p)
}

/// A reader object signed by `key` for `agent`/`instance` at cursor 0.
fn signed_reader(id: &str, topic: &str, key: &Identity, agent: &str, instance: &str) -> Value {
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let b = termlink_session::inbox_read::read_sign_bytes(id, topic, "channel.subscribe", 0, ts, agent, instance);
    json!({"agent": agent, "instance": instance, "ts_ms": ts, "signer_pk": key.public_key_hex(), "sig": key.sign_hex(&b)})
}

fn consumers(e: &Estate, topic: &str) -> Vec<Value> {
    let r = rpc(e, "inbox.consumers", json!({"topic": topic}));
    r["result"]["consumers"].as_array().cloned().unwrap_or_else(|| panic!("inbox.consumers: {r}"))
}

fn admin_rows(e: &Estate, event: &str) -> usize {
    let p = e.runtime_dir().join("rpc-admin.jsonl");
    std::fs::read_to_string(p).unwrap_or_default().lines().filter(|l| l.contains(&format!("\"{event}\""))).count()
}

#[test]
fn e66_second_consumer_flagged_then_refused_after_bind() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    put_card(&e, "fx-a", "a");
    let a = side(&e, &id, "a", "inst-a");

    // fx-a's own signed read: proven, one consumer
    let _ = a.rx.pass();
    a.rx.check_consumers_now();
    assert_eq!(line(&a.store), "one (proven)");

    // E-66.6: fx-z only reads (never acks), unsigned; it is a consumer and is flagged at the next check
    let r = raw_read(&e, &topic, Some(json!({"agent": "fx-z", "instance": "raw"})));
    assert!(r.get("result").is_some(), "unbound read passes: {r}");
    a.rx.check_consumers_now();
    let l = line(&a.store);
    assert_eq!(l, "inbox consumed by two identities: fx-a (proven), fx-z (claimed: unsigned)");
    assert_eq!(attention_count(&a.store), 1);
    a.rx.check_consumers_now();
    assert_eq!(attention_count(&a.store), 1, "coalesced per (inbox, reader, day)");
    println!("E-133 PASS (the per-tick check fired on the injected second consumer)");

    // the operator binds fx-a's inbox to fx-a (runme, local socket)
    let b = rpc(&e, "hub.inbox_bind", json!({"topic": topic, "agent": "fx-a"}));
    assert_eq!(b["result"]["ok"], true, "{b}");
    assert_eq!(admin_rows(&e, "inbox-bind"), 1);

    // fx-z unsigned, fx-z naming fx-a unsigned, an unnamed peek: all refused with a reason
    for reader in [Some(json!({"agent": "fx-z"})), Some(json!({"agent": "fx-a", "instance": "x"})), None] {
        let r = raw_read(&e, &topic, reader.clone());
        assert_eq!(r["error"]["code"], INBOX_BOUND, "{reader:?}: {r}");
        assert!(r["error"]["message"].as_str().unwrap().contains("inbox bound to fx-a"), "{r}");
    }
    assert!(admin_rows(&e, "inbox-bound-refused") >= 1);
    // fx-z signing with its own key while naming fx-a: refused as a card mismatch
    let z = e.identity("z").unwrap();
    let r = raw_read(&e, &topic, Some(signed_reader(&id, &topic, &z, "fx-a", "x")));
    assert_eq!(r["error"]["code"], -32044, "{r}");

    // fx-a keeps reading its own inbox, and its peek works through the bind (ruling D)
    let _ = a.sender.send("fx-a", "inst-a", Some("c1"), "to myself", None);
    assert!(matches!(a.rx.pass(), termlink_sidecar::pull::PassResult::CaughtUp { .. }));
    let p = a.sender.peek_inbox(0, 10).unwrap();
    assert!(p["result"]["messages"].as_array().is_some(), "{p}");

    // other inboxes are untouched by the bind
    let other = format!("inbox:{id}/fx-b");
    let _ = rpc(&e, "channel.create", json!({"name": other}));
    assert!(raw_read(&e, &other, None).get("result").is_some());

    // unbind reopens
    let u = rpc(&e, "hub.inbox_unbind", json!({"topic": topic}));
    assert_eq!(u["result"]["was_bound"], true, "{u}");
    assert!(raw_read(&e, &topic, None).get("result").is_some());
    println!("E-66 PASS (flagged naming both, read-only consumer counted, refused after bind with reason and row)");
}

#[test]
fn e66_ack_is_a_consumer_and_17_readers_evict_the_oldest() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    put_card(&e, "fx-b", "b");

    // fx-b acks on fx-a's inbox (a signed receipt post): recorded as proven fx-b
    let kb = e.identity("b").unwrap();
    let ts = std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64;
    let payload = br#"{"up_to":0}"#;
    let sig = kb.sign_hex(&termlink_protocol::control::channel::canonical_sign_bytes(&topic, "receipt", payload, None, ts));
    let r = rpc(&e, "channel.post", json!({"topic": topic, "msg_type": "receipt",
        "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload), "ts": ts,
        "sender_id": kb.fingerprint(), "sender_pubkey_hex": kb.public_key_hex(), "signature_hex": sig}));
    assert!(r.get("result").is_some(), "{r}");
    let c = consumers(&e, &topic);
    assert!(c.iter().any(|x| x["agent"] == "fx-b" && x["strength"] == "proven" && x["actions"][0] == "ack"), "{c:?}");

    // 17 more named readers: 16 kept, the oldest (fx-b's ack) evicted with an admin row
    for i in 0..17 {
        std::thread::sleep(Duration::from_millis(2));
        let _ = raw_read(&e, &topic, Some(json!({"agent": format!("fx-r{i}")})));
    }
    let c = consumers(&e, &topic);
    assert_eq!(c.len(), 16, "{c:?}");
    assert!(!c.iter().any(|x| x["agent"] == "fx-b"), "the oldest is gone");
    assert!(admin_rows(&e, "inbox-consumer-evicted") >= 2);
    println!("E-66 PASS (ack recorded; 17 readers keep 16, eviction logged)");
}

#[test]
fn v12_second_copy_seen_within_one_check() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-a")}));
    put_card(&e, "fx-a", "a");
    let a1 = side(&e, &id, "a", "inst-a1");
    let a2 = side(&e, &id, "a", "inst-a2");
    let _ = a1.rx.pass();
    let _ = a2.rx.pass();
    a1.rx.check_consumers_now();
    let l = line(&a1.store);
    assert!(l.starts_with("inbox consumed by two identities: fx-a (proven), fx-a (proven) [second copy, instance inst-a2]"), "{l}");
    println!("E-66 PASS (V-12 second copy seen within one check)");
}

#[test]
fn e169_unreadable_consumer_list_is_unknown_never_one() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let _ = rpc(&e, "channel.create", json!({"name": format!("inbox:{id}/fx-a")}));
    put_card(&e, "fx-a", "a");
    // the table cannot be written: its path is a directory
    std::fs::create_dir_all(e.runtime_dir().join("inbox-consumers.json")).unwrap();
    let a = side(&e, &id, "a", "inst-a");
    let _ = a.rx.pass();
    a.rx.check_consumers_now();
    let l = line(&a.store);
    assert!(l.starts_with("unknown ("), "{l}");
    println!("E-169 PASS (unreadable consumer list shown unknown, never one)");
}

#[test]
fn liveness_first_at_is_not_a_consumer() {
    let e = Estate::create().unwrap();
    let id = e.hub_id().unwrap();
    let topic = format!("inbox:{id}/fx-a");
    let _ = rpc(&e, "channel.create", json!({"name": topic}));
    put_card(&e, "fx-a", "a");
    let a = side(&e, &id, "a", "inst-a");
    let _ = a.sender.send("fx-a", "inst-a", Some("c1"), "hello", None);
    let r = rpc(&e, "channel.first_at", json!({"topic": topic, "from_offset": 0}));
    assert!(r["result"]["ts"].as_i64().is_some(), "{r}");
    assert!(r["result"].get("messages").is_none(), "no content: {r}");
    assert!(consumers(&e, &topic).is_empty(), "first_at records no consumer");
}
```

## Measured
E-66 PASS (ack recorded; 17 readers keep 16, eviction logged)
E-169 PASS (unreadable consumer list shown unknown, never one)
E-133 PASS (the per-tick check fired on the injected second consumer)
E-66 PASS (V-12 second copy seen within one check)
E-66 PASS (flagged naming both, read-only consumer counted, refused after bind with reason and row)
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 2.45s

# T-3700 mutants — P-12 / B-2 (AL-30)

Run 2026-11-20 with `python3 crates/termlink-session/tests/mutants/run-t3700.py` (scratch runtime dir, T-3822). Every
mutant must be killed (FP 3.1.b). m4/m6/m7 are the E-169.5, E-66.5-style and E-66.6 controls.

| mutant | what | verdict |
|---|---|---|
| m1-skip-signature-check | a read proof is accepted without verifying its signature | killed |
| m2-ignore-ts-window | a proof with a skewed timestamp still proves | killed |
| m3-bind-by-name | a bind admits any reader naming the bound agent (ruling E broken) | killed |
| m4-hub-unreadable-is-empty | E-169.5 control: an unreadable consumer list answers as a list | killed |
| m5-sidecar-malformed-is-one | a malformed hub answer is shown as one consumer | killed |
| m6-acks-not-recorded | E-66.5-style control: an ack (receipt) does not record its poster | killed |
| m7-reads-not-recorded | E-66.6 control: only acks and claims record; a read-only consumer is missed | killed |
| m8-no-eviction-row | the 17th reader evicts silently | killed |
| m9-hub-instance-ignored | V-12: the hub keys consumers by agent only (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own agent at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content (a consumer of it) instead of first_at | killed |

15/15 killed

## Finding during the run

A mutant killed after `cargo build -p termlink` (here m9) left its mutated hub binary in target/debug; the L4 tests
start that binary without rebuilding it, so V-12 failed on the clean tree afterwards. Both run-t3700.py and
run-t3823.py now rebuild the clean binary at the end. Production is not affected: runme installs from its own
`target/local-fast` build.
