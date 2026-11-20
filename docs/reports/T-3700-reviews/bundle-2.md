# T-3700 review bundle 2 — confirmation of round-1 fixes

# T-3700 review round 1 — dispositions

Seats (isolated, both `independent`): claude-code (`review-1-claude-code.md`) FAIL, 1 BLOCKER, 4 MAJOR, 10 MINOR;
opencode (`review-1-opencode.md`) FAIL, 2 MAJOR, 5 MINOR. A = accept (fixed in round 2), R = reject (reason),
D = defer (risk stated). CC n / OC n = the seat's finding number.

## Where the seats agree

| # | Finding | Seats | Disp. | Change |
|---|---|---|---|---|
| 1 | Self-spoof: an unsigned reader naming the owner (instance `-` or the owner's instance) merges into or is taken for the owner, never flagged before a bind | CC 1 (BLOCKER), OC 1, OC 4 | A | Hub keys entries by (agent, instance, strength, key_fp): a claimed reader never merges into a proven entry. `parse_reader` refuses instance `-` and empty. Sidecar counts as itself only `proven` entries with its OWN key fingerprint. |
| 2 | Recorded before the topic exists; topic length unbounded; eviction rows not rate-capped; full-table rewrite per junk call | CC 3, OC 2 | A | Record only for topics that exist on the bus; topic ≤ 256 bytes; eviction rows rate-capped (10/min, a `suppressed` count on the next row); an inbox with a reader inside the 24 h window is never evicted for a new topic (the new one is not recorded; a capped `inbox-consumer-table-full` row). |
| 3 | Liveness falls back to a content read on any `first_at` error | CC 9, OC 3 | A | Fallback only when the error is not from the new handler (an old hub answers -32001 "Missing 'target'", measured in `router.rs::forward_to_target`); a transport error (no code) is Unreadable. |
| 4 | Corrupt/unknown-schema table overwritten; `load_error` never cleared | CC 4, OC 5 | A | On a load error the file is moved to `inbox-consumers.json.corrupt-<ms>` with an admin row; the table restarts with `incomplete_since_ms`; `inbox.consumers` answers an error ("history incomplete since …") until the 24 h window has passed it; `load_error` cleared by that move. |
| 5 | Design 4.1 names `deliver.rs`; no claim path is signed or tested | CC 10, OC 7 | A | Measured: the sidecar never calls `channel.claim` (its "claim" is a local `O_EXCL` file, `deliver.rs:10`). Design corrected. Claim path tested at L4 through the hub (signed claim passes a bind, unsigned refused). |

## claude-code only

| # | Finding | Disp. | Change / reason |
|---|---|---|---|
| CC 2 | Status line has no age: a stale `one (proven)` survives a restart, early return or failed write | A | State carries `checked_ms`; `status` shows `unknown (no check since …)` after 2 × CHECK_EVERY_MS; cleared at start. |
| CC 5 | Tests: claim path, ack/claim refusal through the hub, replay across hub/method, stale ts after bind, bounds, mutant per gap | A | Added L4: signed/unsigned claim before and after bind, receipt post refused after bind, stale-ts reader refused after bind, self-spoof flagged; unit: replay across hub and method, topic length, table-full, overflow ts. New mutants: claim gate removed, bindings fail open, strength merge, self by name. Each L4 run rebuilds the binary first (the runner already rebuilds at the end). |
| CC 6 | Long-poll opened before a bind still delivers after it | A | Subscribe re-checks the binding (no recording) after a long-poll returns and refuses when the reader no longer passes. |
| CC 7 | A verbatim replay of the owner's old receipt passes a bound inbox | A | Ack proof needs the post's `ts` within ±120 s of the hub clock; outside → `claimed (stale ack)`, refused after a bind. |
| CC 8 | `limit` / `timeout_ms` / `conversation_id` unsigned; a captured reader opens reads at its cursor for 120 s | D | Ruling E accepts a 120 s proof window; capturing a reader object needs the hub's traffic (same host, root on .107 today). Risk: a root process that sees a sidecar's request can re-read that page within 120 s; it gains nothing root cannot read from disk. Revisit with PH-4 accounts. |
| CC 11 | One global mutex; bindings/cards/revocation read from disk per read; one corrupt bindings file refuses every inbox | D (perf) / R (fail closed) | Global lock is stronger than per-topic (correctness holds); revocation and key cards are cached (len, mtime) already. Bindings read per call: deferred, measured cost to be taken on .107 (sidecar reads every few seconds). A corrupt bindings file refusing all inbox reads is deliberate fail-closed (FCW-124): with unknown bindings no read can be judged. |
| CC 12 | `(now − ts).abs()` overflows on `ts = i64::MIN` | A | `abs_diff`. |
| CC 13 | Refused/eviction rows: failure ignored; global cap lets junk hide a real probe; suppressed count lost | A (partly) | Per-kind cap with a `suppressed` count carried on the next row. A failed row does not undo a refusal (refusing is the safe side); an eviction proceeds (the table bound wins). Per-topic caps: D (any caller can pick topics, so a per-topic cap is unbounded in total). |
| CC 14 | Codes and counters differ from design 3.1; revoked ack only claimed; bind ignores revocation | A (partly) | Design 3.1 corrected to the codes used (SIGNATURE_INVALID for a bad signature is the more precise code). `bind` refuses a revoked card key. A revoked ack key stays `claimed (key revoked)` (refused after a bind; a post path change is P-15's, not this unit's). Refusal counter: D (status P-24 reads counts). |
| CC 15 | `first_at` / `receipts` leak a bound inbox's metadata | D | Within ruling D (content-free). Metadata of who acked is visible to Observe callers as before P-12. Raised to the operator in the round-2 report as an open item, not decided by the agent. |

## opencode only

| # | Finding | Disp. | Change |
|---|---|---|---|
| OC 6 | An uncarded ack is listed `key:<fp>`, not its claimed `from_agent` as the design says | A | Uncarded ack: the payload's `from_agent` if a plain name, else `(unnamed)`; the key prefix moves into `why`. |

## Design section 7 (changes after round 1) and corrected 3.1/4.1
## 7. Changes after review round 1 (dispositions: `T-3700-reviews/r1-dispositions.md`)

1. Entries are keyed by (agent, instance, strength, key): a claimed reader never merges into a proven entry. A reader
   object needs an instance, and `-` (the ack sentinel) cannot be claimed. The sidecar counts as itself only an entry
   proven by its own key; anything else under its name is "claims this agent" (or "second copy" when proven by
   another key).
2. Only topics that exist on the bus are recorded; topics over 256 bytes are not recorded or bound; eviction and
   refusal rows are capped at 10 a minute per kind with a `suppressed` count; a full table never evicts an inbox read
   inside the 24 h window (the new topic goes unrecorded, with a capped `inbox-consumer-table-full` row).
3. A corrupt or unknown-schema table is moved to `inbox-consumers.json.corrupt-<ms>` (admin row) and the list is an
   error ("history incomplete since …") until the window has passed the reset.
4. The sidecar's `consumers` line carries `checked_ms`; older than 60 s it reads `unknown (no check since …)`; it is
   cleared at start.
5. A long-poll re-checks the binding when it returns. An ack proves only when the post's `ts` is within ±120 s.
   `bind` refuses a revoked card key. Liveness falls back to the content read only for an old hub (-32001).

## Code and test diff since bundle 1
```diff
diff --git a/crates/termlink-bus/src/lib.rs b/crates/termlink-bus/src/lib.rs
index 959741e57..13cdd63ec 100644
--- a/crates/termlink-bus/src/lib.rs
+++ b/crates/termlink-bus/src/lib.rs
@@ -668,6 +668,11 @@ impl Bus {
     /// The iterator is backed by the SQLite records index plus positional
     /// reads into the log file, so records trimmed by `sweep` are not
     /// yielded even if the underlying file still contains their bytes.
+    /// T-3700: whether `topic` exists (the hub records inbox consumers only for real topics).
+    pub fn topic_exists(&self, topic: &str) -> Result<bool> {
+        self.meta.topic_exists(topic)
+    }
+
     pub fn subscribe(&self, topic: &str, cursor: Offset) -> Result<SubscribeIter> {
         if !self.meta.topic_exists(topic)? {
             return Err(BusError::UnknownTopic(topic.to_string()));
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index a48e63c87..9806b6b32 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -985,7 +985,7 @@ pub(crate) async fn handle_channel_post_with_peer(
 
     // T-3700 (AL-30, FCW-209): an ack (receipt) on an inbox records its poster as a consumer and is
     // refused on a bound inbox unless it is the bound agent's bound key.
-    if msg_type == "receipt" && crate::inbox_consumers::applies(&topic) {
+    if msg_type == "receipt" && crate::inbox_consumers::applies(&topic) && topic_exists(&topic) {
         use crate::inbox_consumers as ic;
         let dir = termlink_session::discovery::runtime_dir();
         let reader = {
@@ -998,7 +998,8 @@ pub(crate) async fn handle_channel_post_with_peer(
                     .filter_map(|c| crate::key_card::parse_signing_key(&c.signing_key).map(|k| (c.agent, k)))
                     .collect())
             };
-            ic::assess_ack(&verifying_key, &cards, &l)
+            let claimed = crate::key_card::claimed_agent(&payload);
+            ic::assess_ack(&verifying_key, claimed.as_deref(), ts_unix_ms, now_unix_ms(), &cards, &l)
         };
         let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
         if let Err(f) = ic::record(&dir, &topic, &reader, ic::Action::Ack, now_unix_ms().max(0) as u64, &mut log) {
@@ -1505,14 +1506,25 @@ fn activity_json(act: &termlink_bus::TopicActivity, now_ms: i64, days: u64) -> (
 
 pub async fn handle_channel_subscribe(id: Value, params: &Value, peer_addr: Option<&str>) -> RpcResponse {
     // T-3700 (AL-30): an inbox read records its reader and is refused on a bound inbox unless proven
-    if let Some(r) = consumer_gate(&id, params, "channel.subscribe", "cursor", crate::inbox_consumers::Action::Read, peer_addr) {
-        return r;
-    }
+    let reader = match consumer_gate(&id, params, "channel.subscribe", "cursor", crate::inbox_consumers::Action::Read, peer_addr) {
+        Ok(r) => r,
+        Err(refused) => return refused,
+    };
     let bus = match bus_or_err(id.clone()) {
         Ok(b) => b,
         Err(r) => return r,
     };
-    handle_channel_subscribe_with(bus, id, params).await
+    let resp = handle_channel_subscribe_with(bus, id.clone(), params).await;
+    // r1 CC 6: a long-poll that waited across a bind does not deliver what was posted after it
+    let waited = params.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(0) > 0;
+    if let (true, Some(r), Some(topic)) = (waited, reader.as_ref(), param_str(params, "topic")) {
+        let dir = termlink_session::discovery::runtime_dir();
+        let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
+        if let Err(f) = crate::inbox_consumers::check_bound(&dir, topic, r, crate::inbox_consumers::Action::Read, now_unix_ms().max(0) as u64, &mut log) {
+            return consumer_refusal(&id, f);
+        }
+    }
+    resp
 }
 
 /// T-3700: `channel.first_at` — offset and timestamp of the first record at or after `from_offset`, no
@@ -1541,7 +1553,9 @@ pub fn handle_channel_first_at(id: Value, params: &Value) -> RpcResponse {
 }
 
 /// T-3700 (06a AL-30, operator rulings E and D 2026-11-19): record the reader of an inbox read or claim
-/// and enforce a bind. `None` lets the call proceed. Non-inbox topics are untouched.
+/// and enforce a bind. `Ok(Some(reader))` lets the call proceed (the reader, for a later re-check);
+/// `Ok(None)` when the gate does not apply (not an inbox topic, or a topic that does not exist: nothing to
+/// read, nothing recorded, r1 CC 3); `Err` refuses the call.
 pub(crate) fn consumer_gate(
     id: &Value,
     params: &Value,
@@ -1549,16 +1563,16 @@ pub(crate) fn consumer_gate(
     position_key: &str,
     action: crate::inbox_consumers::Action,
     peer_addr: Option<&str>,
-) -> Option<RpcResponse> {
+) -> Result<Option<crate::inbox_consumers::Reader>, RpcResponse> {
     use crate::inbox_consumers as ic;
-    let topic = param_str(params, "topic")?;
-    if !ic::applies(topic) {
-        return None;
+    let Some(topic) = param_str(params, "topic") else { return Ok(None) };
+    if !ic::applies(topic) || !topic_exists(topic) {
+        return Ok(None);
     }
     let position = params.get(position_key).and_then(|v| v.as_u64()).unwrap_or(0);
     let proof = match ic::parse_reader(params) {
         Ok(p) => p,
-        Err(e) => return Some(ErrorResponse::new(id.clone(), -32602, &format!("reader: {e}")).into()),
+        Err(e) => return Err(ErrorResponse::new(id.clone(), -32602, &format!("reader: {e}")).into()),
     };
     let transport = match peer_addr {
         Some(a) => format!("tcp:{}", a.rsplit_once(':').map(|(h, _)| h).unwrap_or(a)),
@@ -1573,12 +1587,20 @@ pub(crate) fn consumer_gate(
     };
     let reader = match reader {
         Ok(r) => r,
-        Err(f) => return Some(consumer_refusal(id, f)),
+        Err(f) => return Err(consumer_refusal(id, f)),
     };
     let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
     match ic::record(&dir, topic, &reader, action, now.max(0) as u64, &mut log) {
-        Ok(()) => None,
-        Err(f) => Some(consumer_refusal(id, f)),
+        Ok(()) => Ok(Some(reader)),
+        Err(f) => Err(consumer_refusal(id, f)),
+    }
+}
+
+/// Whether a topic exists on the hub's bus; an unknown bus answers "exists" so the gate still applies.
+fn topic_exists(topic: &str) -> bool {
+    match bus_or_err(Value::Null) {
+        Ok(b) => b.topic_exists(topic).unwrap_or(true),
+        Err(_) => true,
     }
 }
 
@@ -2279,8 +2301,8 @@ pub(crate) async fn handle_channel_list_with(
 /// `claimed_until` to retain the claim.
 pub async fn handle_channel_claim(id: Value, params: &Value, peer_addr: Option<&str>) -> RpcResponse {
     // T-3700 (AL-30): a claim on an inbox is a consumer action like a read
-    if let Some(r) = consumer_gate(&id, params, "channel.claim", "offset", crate::inbox_consumers::Action::Claim, peer_addr) {
-        return r;
+    if let Err(refused) = consumer_gate(&id, params, "channel.claim", "offset", crate::inbox_consumers::Action::Claim, peer_addr) {
+        return refused;
     }
     let bus = match bus_or_err(id.clone()) {
         Ok(b) => b,
diff --git a/crates/termlink-hub/src/inbox_consumers.rs b/crates/termlink-hub/src/inbox_consumers.rs
index c6dc70276..5d05de095 100644
--- a/crates/termlink-hub/src/inbox_consumers.rs
+++ b/crates/termlink-hub/src/inbox_consumers.rs
@@ -18,11 +18,15 @@ use termlink_protocol::control::error_code;
 /// AL-30 2.14.c: entries kept per inbox; the oldest by `last_ms` is evicted with an admin row.
 pub const MAX_PER_TOPIC: usize = 16;
 /// Bound on the number of inboxes tracked, so caller-chosen topic names cannot grow the table without limit.
-pub const MAX_TOPICS: usize = 4096;
+pub const MAX_TOPICS: usize = if cfg!(test) { 64 } else { 4096 };
 /// A read proof's timestamp must be this close to the hub clock to prove (P-23 skew bound).
 pub const PROOF_WINDOW_MS: i64 = 120_000;
 /// `last_ms` is persisted when it moved by at least this much since the last write (design 3.2.2).
 pub const PERSIST_EVERY_MS: u64 = 60_000;
+/// Longest inbox topic recorded or bound (r1 CC 3 / OC 2).
+pub const MAX_TOPIC_LEN: usize = 256;
+/// Admin rows of one kind written per minute; the rest are counted and reported on the next row (r1 CC 13).
+pub const ROWS_PER_MINUTE: u32 = 10;
 /// V-11: the default consumer window.
 pub const DEFAULT_WINDOW_S: u64 = 86_400;
 /// The instance recorded for an ack (a receipt post carries no instance).
@@ -60,8 +64,9 @@ pub fn parse_reader(params: &serde_json::Value) -> Result<Option<ReadProof>, Str
         return Err(format!("reader.agent {:?} is not a plain agent name", agent.chars().take(64).collect::<String>()));
     }
     let instance = s("instance").unwrap_or_default();
-    if instance.len() > 128 || instance.chars().any(|c| c.is_control() || c == '\n') {
-        return Err("reader.instance is longer than 128 bytes or holds a control character".into());
+    // r1 CC 1 / OC 1: `-` is the ack sentinel and an empty instance names nobody; neither may be claimed
+    if instance.is_empty() || instance == ACK_INSTANCE || instance.len() > 128 || instance.chars().any(|c| c.is_control()) {
+        return Err("reader.instance is missing, is \"-\", is longer than 128 bytes or holds a control character".into());
     }
     let ts_ms = r.get("ts_ms").and_then(|v| v.as_i64()).unwrap_or(0);
     let signer_pk = s("signer_pk");
@@ -167,27 +172,37 @@ pub fn assess(
         Ok(_) => {}
         Err(e) => return Ok(claimed(&p.agent, &p.instance, &format!("revocation list unreadable: {e}"))),
     }
-    if (now_ms - p.ts_ms).abs() > PROOF_WINDOW_MS {
+    if now_ms.abs_diff(p.ts_ms) > PROOF_WINDOW_MS as u64 {
         return Ok(claimed(&p.agent, &p.instance, "clock skew"));
     }
     Ok(Reader { agent: p.agent.clone(), instance: p.instance.clone(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) })
 }
 
-/// The reader of a verified receipt post: proven when the signer's key is on a card and not revoked.
-pub fn assess_ack(signer: &VerifyingKey, cards: &dyn Fn() -> Result<Vec<(String, [u8; 32])>, String>, l: &Lookups<'_>) -> Reader {
+/// The reader of a verified receipt post: proven when the signer's key is on a card, not revoked, and the post's
+/// `ts` is within the proof window (a replayed old receipt is only claimed, r1 CC 7). An uncarded ack is listed
+/// under the payload's claimed `from_agent`, else `(unnamed)` (r1 OC 6).
+pub fn assess_ack(
+    signer: &VerifyingKey,
+    claimed_agent: Option<&str>,
+    post_ts_ms: i64,
+    now_ms: i64,
+    cards: &dyn Fn() -> Result<Vec<(String, [u8; 32])>, String>,
+    l: &Lookups<'_>,
+) -> Reader {
     let fp = termlink_session::signed_state::key_fingerprint(signer);
+    let short = fp.chars().skip(8).take(16).collect::<String>();
+    let named = claimed_agent.filter(|a| crate::key_card::valid_name(a)).unwrap_or(UNNAMED).to_string();
     let agent = match cards() {
         Ok(cs) => cs.into_iter().find(|(_, k)| k == signer.as_bytes()).map(|(a, _)| a),
-        Err(e) => return claimed(&format!("key:{}", &fp[8..24.min(fp.len())]), ACK_INSTANCE, &format!("key cards unreadable: {e}")),
+        Err(e) => return claimed(&named, ACK_INSTANCE, &format!("key cards unreadable: {e} (key {short})")),
     };
     let Some(agent) = agent else {
-        return claimed(&format!("key:{}", &fp[8..24.min(fp.len())]), ACK_INSTANCE, "no key card");
+        return claimed(&named, ACK_INSTANCE, &format!("no key card (key {short})"));
     };
     match (l.revoked)() {
-        Ok(keys) if !keys.contains(&fp) => {
-            Reader { agent, instance: ACK_INSTANCE.into(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) }
-        }
-        Ok(_) => claimed(&agent, ACK_INSTANCE, "key revoked"),
+        Ok(keys) if keys.contains(&fp) => claimed(&agent, ACK_INSTANCE, "key revoked"),
+        Ok(_) if now_ms.abs_diff(post_ts_ms) > PROOF_WINDOW_MS as u64 => claimed(&agent, ACK_INSTANCE, "stale ack"),
+        Ok(_) => Reader { agent, instance: ACK_INSTANCE.into(), strength: Strength::Proven, why: String::new(), key_fp: Some(fp) },
         Err(e) => claimed(&agent, ACK_INSTANCE, &format!("revocation list unreadable: {e}")),
     }
 }
@@ -243,6 +258,9 @@ pub struct Binding {
 struct ConsumersFile {
     schema: String,
     topics: BTreeMap<String, Vec<Entry>>,
+    /// Set when an unreadable table was moved aside: readers before this time are lost (r1 CC 4).
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    incomplete_since_ms: Option<u64>,
 }
 
 #[derive(Debug, Default, Serialize, Deserialize)]
@@ -266,10 +284,14 @@ pub fn bindings_path(dir: &Path) -> PathBuf {
 #[derive(Debug, Default)]
 struct State {
     topics: BTreeMap<String, Vec<Entry>>,
+    incomplete_since_ms: Option<u64>,
     /// The last persist of the consumer table failed: `inbox.consumers` answers an error (FCW-125).
     persist_error: Option<String>,
-    /// The consumer file could not be read at load: same.
+    /// The consumer file could not be read nor moved aside: nothing is saved over it, and
+    /// `inbox.consumers` answers an error.
     load_error: Option<String>,
+    /// Admin rows found at load (a corrupt table moved aside), written by the next [`record`].
+    pending_rows: Vec<serde_json::Value>,
 }
 
 static STATES: Mutex<Option<HashMap<PathBuf, State>>> = Mutex::new(None);
@@ -281,8 +303,15 @@ fn with_state<R>(dir: &Path, f: impl FnOnce(&mut State) -> R) -> R {
     f(st)
 }
 
+fn now_wall_ms() -> u64 {
+    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
+}
+
+/// Load the table. An unreadable or unknown-schema file is never saved over: it is moved to
+/// `<name>.corrupt-<ms>` (admin row on the next record) and the table restarts marked incomplete (r1 CC 4).
 fn load_state(dir: &Path) -> State {
-    match std::fs::read(consumers_path(dir)) {
+    let path = consumers_path(dir);
+    let why = match std::fs::read(&path) {
         Ok(b) => match serde_json::from_slice::<ConsumersFile>(&b) {
             Ok(f) if f.schema == CONSUMERS_SCHEMA => {
                 let mut topics = f.topics;
@@ -291,13 +320,26 @@ fn load_state(dir: &Path) -> State {
                         e.persisted_last_ms = e.last_ms;
                     }
                 }
-                State { topics, ..Default::default() }
+                return State { topics, incomplete_since_ms: f.incomplete_since_ms, ..Default::default() };
             }
-            Ok(f) => State { load_error: Some(format!("unknown schema {:?}", f.schema)), ..Default::default() },
-            Err(e) => State { load_error: Some(format!("unparseable: {e}")), ..Default::default() },
+            Ok(f) => format!("unknown schema {:?}", f.schema),
+            Err(e) => format!("unparseable: {e}"),
         },
-        Err(e) if e.kind() == std::io::ErrorKind::NotFound => State::default(),
-        Err(e) => State { load_error: Some(e.to_string()), ..Default::default() },
+        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return State::default(),
+        Err(e) => format!("unreadable: {e}"),
+    };
+    let now = now_wall_ms();
+    let aside = path.with_file_name(format!("inbox-consumers.json.corrupt-{now}"));
+    crate::key_card::guard_test_write(dir);
+    match std::fs::rename(&path, &aside) {
+        Ok(()) => State {
+            incomplete_since_ms: Some(now),
+            pending_rows: vec![serde_json::json!({
+                "event": "inbox-consumers-reset", "attention": true, "why": why, "moved_to": aside.display().to_string(),
+            })],
+            ..Default::default()
+        },
+        Err(e) => State { load_error: Some(format!("consumer table {why}; not moved aside: {e}")), ..Default::default() },
     }
 }
 
@@ -315,8 +357,8 @@ fn load_bindings(dir: &Path) -> Result<BTreeMap<String, Binding>, String> {
     }
 }
 
-fn save_consumers(dir: &Path, topics: &BTreeMap<String, Vec<Entry>>) -> Result<(), String> {
-    let f = ConsumersFile { schema: CONSUMERS_SCHEMA.into(), topics: topics.clone() };
+fn save_consumers(dir: &Path, st: &State) -> Result<(), String> {
+    let f = ConsumersFile { schema: CONSUMERS_SCHEMA.into(), topics: st.topics.clone(), incomplete_since_ms: st.incomplete_since_ms };
     let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
     crate::key_card::atomic_write(&consumers_path(dir), &text, 0o600).map_err(|e| e.to_string())
 }
@@ -324,59 +366,120 @@ fn save_consumers(dir: &Path, topics: &BTreeMap<String, Vec<Entry>>) -> Result<(
 /// Admin-log writer, injectable for tests.
 pub type AdminLog<'a> = &'a mut dyn FnMut(serde_json::Value) -> Result<(), String>;
 
+/// Write a capped admin row of one kind: at most [`ROWS_PER_MINUTE`] a minute; the rest are counted and the
+/// count rides on the next written row as `suppressed` (r1 CC 3, CC 13).
+fn capped_row(kind: &'static str, mut row: serde_json::Value, now_ms: u64, log: AdminLog<'_>) {
+    static WIN: Mutex<Option<HashMap<&'static str, (u64, u32, u64)>>> = Mutex::new(None);
+    let mut g = WIN.lock().unwrap_or_else(|p| p.into_inner());
+    let w = g.get_or_insert_with(HashMap::new).entry(kind).or_insert((0, 0, 0));
+    let minute = now_ms / 60_000;
+    if w.0 != minute {
+        w.0 = minute;
+        w.1 = 0;
+    }
+    w.1 += 1;
+    if w.1 > ROWS_PER_MINUTE {
+        w.2 += 1;
+        return;
+    }
+    if w.2 > 0 {
+        row["suppressed"] = serde_json::json!(w.2);
+        w.2 = 0;
+    }
+    let _ = log(row);
+}
+
+/// Whether `reader` passes the binding of `topic` (proven, the bound agent, the bound key).
+fn passes(b: &Binding, reader: &Reader) -> bool {
+    reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str())
+}
+
+fn bound_refusal(topic: &str, b: &Binding, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Refusal {
+    let shown = if reader.strength == Strength::Proven {
+        format!("{} (proven)", reader.agent)
+    } else {
+        format!("{} (claimed: {})", reader.agent, reader.why)
+    };
+    capped_row(
+        "inbox-bound-refused",
+        serde_json::json!({
+            "event": "inbox-bound-refused", "attention": true, "topic": topic, "bound": b.agent,
+            "reader": reader.agent, "instance": reader.instance, "strength": reader.strength, "action": action.name(),
+        }),
+        now_ms,
+        log,
+    );
+    Refusal { code: error_code::INBOX_BOUND, fcw: "FCW-124", reason: format!("inbox bound to {}; {} by {shown} refused", b.agent, action.name()) }
+}
+
+fn bindings_refusal(e: String) -> Refusal {
+    Refusal { code: error_code::INBOX_BOUND, fcw: "FCW-124", reason: format!("{e}; inbox reads are refused until it is repaired") }
+}
+
+/// Re-check only the binding (no recording): a long-poll that waited across a bind must not deliver (r1 CC 6).
+pub fn check_bound(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
+    let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
+    let bindings = load_bindings(dir).map_err(bindings_refusal)?;
+    match bindings.get(topic) {
+        Some(b) if !passes(b, reader) => Err(bound_refusal(topic, b, reader, action, now_ms, log)),
+        _ => Ok(()),
+    }
+}
+
 /// Check the bind and record the reader in one locked step (2.14.f(4)). `Err` refuses the call.
+/// The caller records only topics that exist on the bus (r1 CC 3 / OC 2).
 pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
     let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
     let map = g.get_or_insert_with(HashMap::new);
     let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
+    for row in std::mem::take(&mut st.pending_rows) {
+        let _ = log(row);
+    }
     // the bind is read under the same lock the bind verb writes under
-    let bindings = load_bindings(dir).map_err(|e| Refusal {
-        code: error_code::INBOX_BOUND,
-        fcw: "FCW-124",
-        reason: format!("{e}; inbox reads are refused until it is repaired"),
-    })?;
-    if let Some(b) = bindings.get(topic) {
-        let ok = reader.strength == Strength::Proven && reader.agent == b.agent && reader.key_fp.as_deref() == Some(b.key_fp.as_str());
-        if !ok {
-            let shown = if reader.strength == Strength::Proven {
-                format!("{} (proven)", reader.agent)
-            } else {
-                format!("{} (claimed: {})", reader.agent, reader.why)
-            };
-            if refused_row_allowed(now_ms) {
-                let _ = log(serde_json::json!({
-                    "event": "inbox-bound-refused", "attention": true, "topic": topic, "bound": b.agent,
-                    "reader": reader.agent, "instance": reader.instance, "strength": reader.strength,
-                    "action": action.name(),
-                }));
-            }
-            return Err(Refusal {
-                code: error_code::INBOX_BOUND,
-                fcw: "FCW-124",
-                reason: format!("inbox bound to {}; {} by {shown} refused", b.agent, action.name()),
-            });
-        }
+    let bindings = load_bindings(dir).map_err(bindings_refusal)?;
+    if let Some(b) = bindings.get(topic)
+        && !passes(b, reader)
+    {
+        return Err(bound_refusal(topic, b, reader, action, now_ms, log));
+    }
+    if topic.len() > MAX_TOPIC_LEN || st.load_error.is_some() {
+        // nothing is recorded over an unreadable table (it stays reported unreadable)
+        return Ok(());
     }
     let mut dirty = false;
     if !st.topics.contains_key(topic) && st.topics.len() >= MAX_TOPICS {
-        // evict the inbox whose newest reader is oldest
-        if let Some(old) = st
+        // evict the inbox whose newest reader is oldest, but never one with a reader inside the window
+        let window_start = now_ms.saturating_sub(DEFAULT_WINDOW_S * 1000);
+        let oldest = st
             .topics
             .iter()
-            .min_by_key(|(_, v)| v.iter().map(|e| e.last_ms).max().unwrap_or(0))
-            .map(|(k, _)| k.clone())
-        {
-            st.topics.remove(&old);
-            let _ = log(serde_json::json!({"event": "inbox-consumer-topic-evicted", "topic": old}));
-            dirty = true;
+            .map(|(k, v)| (k.clone(), v.iter().map(|e| e.last_ms).max().unwrap_or(0)))
+            .min_by_key(|(_, last)| *last);
+        match oldest {
+            Some((old, last)) if last < window_start => {
+                st.topics.remove(&old);
+                capped_row("inbox-consumer-topic-evicted", serde_json::json!({"event": "inbox-consumer-topic-evicted", "topic": old}), now_ms, log);
+                dirty = true;
+            }
+            _ => {
+                capped_row(
+                    "inbox-consumer-table-full",
+                    serde_json::json!({"event": "inbox-consumer-table-full", "attention": true, "topic": topic}),
+                    now_ms,
+                    log,
+                );
+                return Ok(());
+            }
         }
     }
     let list = st.topics.entry(topic.to_string()).or_default();
-    match list.iter_mut().find(|e| e.agent == reader.agent && e.instance == reader.instance) {
+    // r1 CC 1 / OC 4: keyed by strength and key too, so a claimed reader never merges into a proven entry
+    match list
+        .iter_mut()
+        .find(|e| e.agent == reader.agent && e.instance == reader.instance && e.strength == reader.strength && e.key_fp == reader.key_fp)
+    {
         Some(e) => {
-            if e.strength != reader.strength || e.key_fp != reader.key_fp || e.why != reader.why {
-                e.strength = reader.strength.clone();
-                e.key_fp = reader.key_fp.clone();
+            if e.why != reader.why {
                 e.why = reader.why.clone();
                 dirty = true;
             }
@@ -389,14 +492,19 @@ pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms:
             }
         }
         None => {
-            if list.len() >= MAX_PER_TOPIC {
-                if let Some(i) = list.iter().enumerate().min_by_key(|(_, e)| e.last_ms).map(|(i, _)| i) {
-                    let gone = list.remove(i);
-                    let _ = log(serde_json::json!({
+            if list.len() >= MAX_PER_TOPIC
+                && let Some(i) = list.iter().enumerate().min_by_key(|(_, e)| e.last_ms).map(|(i, _)| i)
+            {
+                let gone = list.remove(i);
+                capped_row(
+                    "inbox-consumer-evicted",
+                    serde_json::json!({
                         "event": "inbox-consumer-evicted", "topic": topic, "agent": gone.agent,
                         "instance": gone.instance, "last_ms": gone.last_ms,
-                    }));
-                }
+                    }),
+                    now_ms,
+                    log,
+                );
             }
             list.push(Entry {
                 agent: reader.agent.clone(),
@@ -413,7 +521,7 @@ pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms:
         }
     }
     if dirty {
-        match save_consumers(dir, &st.topics) {
+        match save_consumers(dir, st) {
             Ok(()) => {
                 st.persist_error = None;
                 for v in st.topics.values_mut() {
@@ -430,7 +538,8 @@ pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms:
 }
 
 /// `inbox.consumers`: readers seen within `window_s`, plus the binding. `Err` when the list cannot be
-/// trusted (FCW-125): the caller shows `consumers: unknown`, never "one".
+/// trusted (FCW-125): the caller shows `consumers: unknown`, never "one". A table restarted after a corrupt
+/// one stays an error until the window no longer reaches back before the restart (r1 CC 4).
 pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<serde_json::Value, String> {
     let bound = load_bindings(dir)?.get(topic).cloned();
     with_state(dir, |st| {
@@ -438,16 +547,29 @@ pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<
             return Err(e.clone());
         }
         let since = now_ms.saturating_sub(window_s.saturating_mul(1000));
+        if let Some(t) = st.incomplete_since_ms
+            && t > since
+        {
+            return Err(format!("consumer history incomplete since {t} (an unreadable table was moved aside)"));
+        }
         let list: Vec<&Entry> = st.topics.get(topic).map(|v| v.iter().filter(|e| e.last_ms >= since).collect()).unwrap_or_default();
         Ok(serde_json::json!({"topic": topic, "window_s": window_s, "consumers": list, "bound": bound}))
     })
 }
 
 /// `hub.inbox_bind`: admin row first, then the durable binding (2.14.c). The bound key is the agent's
-/// card key now; an agent without a card cannot be bound.
-pub fn bind(dir: &Path, topic: &str, agent: &str, card_key: Option<[u8; 32]>, now_ms: u64, log: AdminLog<'_>) -> Result<Binding, String> {
-    if !applies(topic) {
-        return Err(format!("{topic:?} is not an inbox topic"));
+/// card key now; an agent without a card, or whose card key is revoked (r1 CC 14), cannot be bound.
+pub fn bind(
+    dir: &Path,
+    topic: &str,
+    agent: &str,
+    card_key: Option<[u8; 32]>,
+    revoked: &dyn Fn() -> Result<BTreeSet<String>, String>,
+    now_ms: u64,
+    log: AdminLog<'_>,
+) -> Result<Binding, String> {
+    if !applies(topic) || topic.len() > MAX_TOPIC_LEN {
+        return Err(format!("{topic:?} is not an inbox topic of at most {MAX_TOPIC_LEN} bytes"));
     }
     if !crate::key_card::valid_name(agent) {
         return Err(format!("{agent:?} is not a plain agent name"));
@@ -455,6 +577,9 @@ pub fn bind(dir: &Path, topic: &str, agent: &str, card_key: Option<[u8; 32]>, no
     let Some(k) = card_key else { return Err(format!("agent {agent:?} has no key card; bind refused")) };
     let vk = VerifyingKey::from_bytes(&k).map_err(|e| format!("agent {agent:?}'s card key is invalid: {e}"))?;
     let b = Binding { agent: agent.into(), key_fp: termlink_session::signed_state::key_fingerprint(&vk), at_ms: now_ms };
+    if revoked().map_err(|e| format!("revocation list unreadable: {e}"))?.contains(&b.key_fp) {
+        return Err(format!("agent {agent:?}'s card key {} is revoked; bind refused", b.key_fp));
+    }
     let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
     let mut all = load_bindings(dir)?;
     log(serde_json::json!({"event": "inbox-bind", "topic": topic, "agent": agent, "key_fp": b.key_fp}))?;
@@ -482,18 +607,6 @@ fn write_bindings(dir: &Path, bindings: BTreeMap<String, Binding>) -> Result<(),
     crate::key_card::atomic_write(&bindings_path(dir), &text, 0o600).map_err(|e| e.to_string())
 }
 
-/// At most 10 refused-read admin rows a minute (any caller can provoke them; r2 CC 12 of T-3703).
-fn refused_row_allowed(now_ms: u64) -> bool {
-    static WIN: Mutex<(u64, u32)> = Mutex::new((0, 0));
-    let mut w = WIN.lock().unwrap_or_else(|p| p.into_inner());
-    let minute = now_ms / 60_000;
-    if w.0 != minute {
-        *w = (minute, 0);
-    }
-    w.1 += 1;
-    w.1 <= 10
-}
-
 /// Forget the cached state of `dir` (tests; a hub restart starts from disk anyway).
 #[cfg(test)]
 pub(crate) fn reset(dir: &Path) {
diff --git a/crates/termlink-hub/src/inbox_consumers_tests.rs b/crates/termlink-hub/src/inbox_consumers_tests.rs
index 26ae05232..b13fb8b2f 100644
--- a/crates/termlink-hub/src/inbox_consumers_tests.rs
+++ b/crates/termlink-hub/src/inbox_consumers_tests.rs
@@ -129,6 +129,9 @@ fn parse_reader_rules() {
     assert_eq!(parse_reader(&serde_json::json!({})).unwrap(), None);
     assert!(parse_reader(&serde_json::json!({"reader": {"agent": "Bad Name"}})).is_err());
     assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "sig": "00"}})).is_err());
+    // r1 CC 1 / OC 1: the instance is required and may not be the ack sentinel
+    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a"}})).is_err());
+    assert!(parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "instance": "-"}})).is_err());
     let p = parse_reader(&serde_json::json!({"reader": {"agent": "fx-a", "instance": "i", "ts_ms": 5}})).unwrap().unwrap();
     assert_eq!((p.agent.as_str(), p.ts_ms), ("fx-a", 5));
 }
@@ -229,7 +232,7 @@ fn unreadable_list_is_an_error_never_empty() {
     let d = dir();
     std::fs::write(consumers_path(d.path()), b"{not json").unwrap();
     reset(d.path());
-    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, NOW as u64).is_err());
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now_wall_ms()).is_err());
 }
 
 #[test]
@@ -242,7 +245,7 @@ fn bind_refuses_everyone_but_the_bound_key() {
             logged.push(r);
             Ok(())
         };
-        bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), n, &mut log).unwrap();
+        bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, n, &mut log).unwrap();
         // the bound agent, proven with the bound key
         record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
         // another agent, proven
@@ -266,15 +269,15 @@ fn bind_refuses_everyone_but_the_bound_key() {
 fn bind_needs_a_card_and_an_inbox_topic() {
     let d = dir();
     let (_, mut log) = rows();
-    assert!(bind(d.path(), T, "fx-a", None, NOW as u64, &mut log).is_err());
-    assert!(bind(d.path(), "chat:x", "fx-a", Some(*key(1).verifying_key().as_bytes()), NOW as u64, &mut log).is_err());
+    assert!(bind(d.path(), T, "fx-a", None, &no_rev, NOW as u64, &mut log).is_err());
+    assert!(bind(d.path(), "chat:x", "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut log).is_err());
 }
 
 #[test]
 fn bind_writes_its_admin_row_first() {
     let d = dir();
     let mut failing = |_r: serde_json::Value| Err("admin log full".to_string());
-    assert!(bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), NOW as u64, &mut failing).is_err());
+    assert!(bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut failing).is_err());
     assert!(!bindings_path(d.path()).exists(), "no row, no bind");
 }
 
@@ -283,7 +286,7 @@ fn unbind_reopens_and_unreadable_bindings_fail_closed() {
     let d = dir();
     let (_, mut log) = rows();
     let n = NOW as u64;
-    bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), n, &mut log).unwrap();
+    bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, n, &mut log).unwrap();
     assert!(unbind(d.path(), T, &mut log).unwrap());
     record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
     std::fs::write(bindings_path(d.path()), b"garbage").unwrap();
@@ -293,8 +296,143 @@ fn unbind_reopens_and_unreadable_bindings_fail_closed() {
 #[test]
 fn ack_reader_from_cards() {
     let cards = || -> Result<Vec<(String, [u8; 32])>, String> { Ok(vec![("fx-a".into(), *key(1).verifying_key().as_bytes())]) };
-    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(1).verifying_key(), &cards, l));
+    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(1).verifying_key(), Some("fx-a"), NOW, NOW, &cards, l));
     assert_eq!((r.agent.as_str(), r.strength), ("fx-a", Strength::Proven));
-    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(9).verifying_key(), &cards, l));
-    assert_eq!(r.strength, Strength::Claimed);
+    // r1 OC 6: an uncarded ack is listed under its claimed from_agent
+    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(9).verifying_key(), Some("fx-z"), NOW, NOW, &cards, l));
+    assert_eq!((r.agent.as_str(), r.strength), ("fx-z", Strength::Claimed));
+    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(9).verifying_key(), Some("Not A Name"), NOW, NOW, &cards, l));
+    assert_eq!(r.agent, UNNAMED);
+    // r1 CC 7: a replayed old receipt by the carded key is only claimed
+    let r = with_lookups(BTreeSet::new(), |l| assess_ack(&key(1).verifying_key(), Some("fx-a"), NOW - PROOF_WINDOW_MS - 1, NOW, &cards, l));
+    assert_eq!((r.strength, r.why.as_str()), (Strength::Claimed, "stale ack"));
+}
+
+fn no_rev() -> Result<BTreeSet<String>, String> {
+    Ok(BTreeSet::new())
+}
+
+#[test]
+fn a_claimed_reader_never_merges_into_a_proven_entry() {
+    // r1 CC 1 / OC 4: an unsigned reader naming the owner's agent AND instance is its own entry
+    let d = dir();
+    let (_, mut log) = rows();
+    let n = NOW as u64;
+    record(d.path(), T, &proven("fx-a", "inst-a", 1), Action::Read, n, &mut log).unwrap();
+    record(d.path(), T, &claimed("fx-a", "inst-a", "unsigned"), Action::Read, n + 1, &mut log).unwrap();
+    record(d.path(), T, &proven("fx-a", "inst-a", 1), Action::Read, n + 2, &mut log).unwrap();
+    let l = listed(d.path(), T, n + 3);
+    assert_eq!(l.len(), 2, "{l:?}");
+    assert!(l.contains(&("fx-a".into(), "inst-a".into(), "proven".into())));
+    assert!(l.contains(&("fx-a".into(), "inst-a".into(), "claimed".into())));
+}
+
+#[test]
+fn replay_across_hub_or_method_does_not_verify() {
+    let k = key(1);
+    let other_hub = read_sign_bytes("ffffffffffffffff", T, "channel.subscribe", 7, NOW, "fx-a", "i");
+    let other_method = read_sign_bytes(HUB, T, "channel.claim", 7, NOW, "fx-a", "i");
+    for b in [other_hub, other_method] {
+        let p = ReadProof {
+            agent: "fx-a".into(),
+            instance: "i".into(),
+            ts_ms: NOW,
+            signer_pk: Some(hex(k.verifying_key().as_bytes())),
+            sig: Some(hex(&k.sign(&b).to_bytes())),
+        };
+        assert_eq!(assess_one(Some(&p)).unwrap_err().code, error_code::CHANNEL_SIGNATURE_INVALID);
+    }
+}
+
+#[test]
+fn extreme_timestamp_does_not_overflow() {
+    let p = proof(&key(1), "fx-a", "i", T, 7, i64::MIN);
+    assert_eq!(assess_one(Some(&p)).unwrap().why, "clock skew");
+}
+
+#[test]
+fn skewed_proof_is_refused_after_a_bind() {
+    let d = dir();
+    let (_, mut log) = rows();
+    bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut log).unwrap();
+    let r = assess_one(Some(&proof(&key(1), "fx-a", "i", T, 7, NOW - PROOF_WINDOW_MS - 1))).unwrap();
+    assert_eq!(record(d.path(), T, &r, Action::Read, NOW as u64, &mut log).unwrap_err().code, error_code::INBOX_BOUND);
+}
+
+#[test]
+fn bind_refuses_a_revoked_key_and_a_long_topic() {
+    let d = dir();
+    let (_, mut log) = rows();
+    let fp = termlink_session::signed_state::key_fingerprint(&key(1).verifying_key());
+    let rev = move || -> Result<BTreeSet<String>, String> { Ok([fp.clone()].into_iter().collect()) };
+    assert!(bind(d.path(), T, "fx-a", Some(*key(1).verifying_key().as_bytes()), &rev, NOW as u64, &mut log).is_err());
+    let long = format!("inbox:{}", "x".repeat(MAX_TOPIC_LEN));
+    assert!(bind(d.path(), &long, "fx-a", Some(*key(1).verifying_key().as_bytes()), &no_rev, NOW as u64, &mut log).is_err());
+}
+
+#[test]
+fn a_long_topic_is_not_recorded() {
+    let d = dir();
+    let (_, mut log) = rows();
+    let long = format!("inbox:{}", "x".repeat(MAX_TOPIC_LEN));
+    record(d.path(), &long, &claimed_r("fx-z"), Action::Read, NOW as u64, &mut log).unwrap();
+    assert!(listed(d.path(), &long, NOW as u64).is_empty());
+}
+
+#[test]
+fn a_full_table_never_evicts_an_inbox_read_inside_the_window() {
+    let d = dir();
+    let logged = std::cell::RefCell::new(Vec::new());
+    let mut log = |r: serde_json::Value| {
+        logged.borrow_mut().push(r);
+        Ok(())
+    };
+    let n = NOW as u64;
+    for i in 0..MAX_TOPICS {
+        record(d.path(), &format!("inbox:h/t{i}"), &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
+    }
+    record(d.path(), "inbox:h/new", &claimed_r("fx-z"), Action::Read, n + 1, &mut log).unwrap();
+    assert!(listed(d.path(), "inbox:h/new", n + 1).is_empty(), "not recorded");
+    assert_eq!(listed(d.path(), "inbox:h/t0", n + 1).len(), 1, "the oldest active inbox kept");
+    assert!(logged.borrow().iter().any(|r| r["event"] == "inbox-consumer-table-full"));
+    // a day later the oldest inbox is out of the window and may be evicted
+    let later = n + 2 * 86_400_000;
+    record(d.path(), "inbox:h/new", &claimed_r("fx-z"), Action::Read, later, &mut log).unwrap();
+    assert_eq!(listed(d.path(), "inbox:h/new", later).len(), 1);
+}
+
+#[test]
+fn a_corrupt_table_is_moved_aside_and_reported_incomplete() {
+    let d = dir();
+    std::fs::write(consumers_path(d.path()), b"{not json").unwrap();
+    reset(d.path());
+    let mut logged = Vec::new();
+    let mut log = |r: serde_json::Value| {
+        logged.push(r);
+        Ok(())
+    };
+    let now = now_wall_ms();
+    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, now, &mut log).unwrap();
+    assert!(logged.iter().any(|r| r["event"] == "inbox-consumers-reset"), "{logged:?}");
+    let aside: Vec<_> = std::fs::read_dir(d.path()).unwrap().filter_map(|e| e.ok()).filter(|e| e.file_name().to_string_lossy().contains(".corrupt-")).collect();
+    assert_eq!(aside.len(), 1, "the corrupt file is kept, not overwritten");
+    // inside the window the history is incomplete: an error, never a clean list
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now).is_err());
+    // once the window no longer reaches back before the reset, the list is served again
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now + DEFAULT_WINDOW_S * 1000 + 60_000).is_ok());
+}
+
+#[test]
+fn a_failed_save_makes_the_list_an_error_never_a_list() {
+    // E-169 / FCW-125 (m4 survived the first round-2 mutant run: nothing made a SAVE fail after load)
+    let d = dir();
+    let (_, mut log) = rows();
+    let n = now_wall_ms();
+    record(d.path(), T, &proven("fx-a", "w1", 1), Action::Read, n, &mut log).unwrap();
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, n).is_ok());
+    // the table path becomes a directory: the next save fails
+    std::fs::remove_file(consumers_path(d.path())).unwrap();
+    std::fs::create_dir(consumers_path(d.path())).unwrap();
+    record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, n).is_err(), "a list that could not be saved is not served");
 }
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index a64e4a202..4ac8e37fc 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -1078,7 +1078,12 @@ fn handle_inbox_bind(id: serde_json::Value, params: &serde_json::Value, peer_add
         Err(e) => return ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("bind refused: key cards unreadable: {e}")).into(),
     };
     let mut log = |row: serde_json::Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
-    match crate::inbox_consumers::bind(&dir, &topic, &agent, key, now_ms(), &mut log) {
+    let ident = crate::hub_identity::current();
+    let revoked = || match &ident {
+        Some(i) => crate::revocation::revoked_keys(&dir, &i.hub_id, i.signing_key()),
+        None => Err("the hub identity is not loaded".to_string()),
+    };
+    match crate::inbox_consumers::bind(&dir, &topic, &agent, key, &revoked, now_ms(), &mut log) {
         Ok(b) => Response::success(id, serde_json::json!({"ok": true, "topic": topic, "agent": b.agent, "key_fp": b.key_fp})).into(),
         Err(e) => ErrorResponse::new(id, control::error_code::INBOX_BOUND, &format!("bind refused: {e}")).into(),
     }
diff --git a/crates/termlink-sidecar/src/api.rs b/crates/termlink-sidecar/src/api.rs
index 111992802..d494e6782 100644
--- a/crates/termlink-sidecar/src/api.rs
+++ b/crates/termlink-sidecar/src/api.rs
@@ -1278,7 +1278,7 @@ fn status_at(
         // T-3703 (OBS-41, design 5.5): `revocation` and the `refused_revoked` count
         "revocation": crate::revocation::status(store),
         // T-3700 (AL-30, OBS-56): who consumes this inbox; `unknown`, never "one", when unreadable
-        "consumers": crate::consumers::status(store),
+        "consumers": crate::consumers::status(store, now_ms),
         "stages": STAGES,
         "rows": rows,
         "messages": msgs,
diff --git a/crates/termlink-sidecar/src/consumers.rs b/crates/termlink-sidecar/src/consumers.rs
index de11f0a24..1c20646a5 100644
--- a/crates/termlink-sidecar/src/consumers.rs
+++ b/crates/termlink-sidecar/src/consumers.rs
@@ -30,6 +30,7 @@ pub struct Seen {
     pub instance: String,
     pub strength: String,
     pub why: String,
+    pub key_fp: String,
 }
 
 impl Seen {
@@ -50,8 +51,10 @@ pub struct Verdict {
     pub bound: Option<String>,
 }
 
-/// Classify the hub's answer to `inbox.consumers` (`Err((code, message))` when the call failed).
-pub fn verdict(me_agent: &str, me_instance: &str, answer: Result<&Value, (i64, &str)>) -> Verdict {
+/// Classify the hub's answer to `inbox.consumers` (`Err((code, message))` when the call failed). `me_fp` is this
+/// sidecar's own key fingerprint: only an entry PROVEN by that key is this sidecar (r1 CC 1 / OC 1); a name, an
+/// instance or the ack sentinel alone never is.
+pub fn verdict(me_agent: &str, me_instance: &str, me_fp: &str, answer: Result<&Value, (i64, &str)>) -> Verdict {
     let unknown = |why: String| Verdict { line: format!("unknown ({why})"), others: vec![], bound: None };
     let v = match answer {
         Ok(v) => v,
@@ -67,8 +70,11 @@ pub fn verdict(me_agent: &str, me_instance: &str, answer: Result<&Value, (i64, &
     let mut others: Vec<Seen> = Vec::new();
     for e in list {
         let s = |k: &str| e.get(k).and_then(Value::as_str).unwrap_or("").to_string();
-        let seen = Seen { agent: s("agent"), instance: s("instance"), strength: s("strength"), why: s("why") };
-        let is_me = seen.agent == me_agent && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);
+        let seen = Seen { agent: s("agent"), instance: s("instance"), strength: s("strength"), why: s("why"), key_fp: s("key_fp") };
+        let is_me = seen.agent == me_agent
+            && seen.strength == "proven"
+            && seen.key_fp == me_fp
+            && (seen.instance == me_instance || seen.instance == ACK_INSTANCE);
         if is_me { mine.push(seen) } else { others.push(seen) }
     }
     let line = if !others.is_empty() {
@@ -82,8 +88,11 @@ pub fn verdict(me_agent: &str, me_instance: &str, answer: Result<&Value, (i64, &
         let count = if n == 2 { "two".to_string() } else { n.to_string() };
         let names: Vec<String> = std::iter::once(me_shown)
             .chain(others.iter().map(|o| {
-                if o.agent == me_agent {
+                if o.agent == me_agent && o.strength == "proven" {
+                    // proven by a key that is not this sidecar's: another copy of this agent (V-12)
                     format!("{} [second copy, instance {}]", o.shown(), o.instance)
+                } else if o.agent == me_agent {
+                    format!("{} [claims this agent, instance {}]", o.shown(), o.instance)
                 } else {
                     o.shown()
                 }
@@ -100,9 +109,12 @@ pub fn verdict(me_agent: &str, me_instance: &str, answer: Result<&Value, (i64, &
     Verdict { line, others, bound }
 }
 
+/// How old a check may be before `status` shows `unknown` (r1 CC 2).
+pub const STALE_AFTER_MS: u64 = 2 * CHECK_EVERY_MS;
+
 /// Record a verdict: the status line, and one needs-attention entry per other reader per UTC day.
 pub fn record(store: &Store, topic: &str, v: &Verdict, now_ms: u64) {
-    let state = json!({"consumers": v.line, "bound": v.bound}).to_string();
+    let state = json!({"consumers": v.line, "bound": v.bound, "checked_ms": now_ms}).to_string();
     if let Err(e) = store.write_state(STATUS_STATE, &state) {
         eprintln!("termlink-sidecar: consumers status not written: {e}");
     }
@@ -116,22 +128,32 @@ pub fn record(store: &Store, topic: &str, v: &Verdict, now_ms: u64) {
     }
 }
 
-/// The `consumers` status (and `bound`), `unknown (no check yet)` before the first check.
-pub fn status(store: &Store) -> Value {
-    store
-        .read_state(STATUS_STATE)
-        .ok()
-        .flatten()
-        .and_then(|s| serde_json::from_str::<Value>(&s).ok())
-        .unwrap_or_else(|| json!({"consumers": "unknown (no check yet)", "bound": null}))
+/// Forget the last check (at start: a line from before a restart is never shown, r1 CC 2).
+pub fn reset(store: &Store) {
+    let _ = store.write_state(STATUS_STATE, "{}");
+}
+
+/// The `consumers` status (and `bound`): `unknown (no check yet)` before the first check, and
+/// `unknown (no check since …)` when the last one is older than [`STALE_AFTER_MS`] (r1 CC 2).
+pub fn status(store: &Store, now_ms: u64) -> Value {
+    let v = store.read_state(STATUS_STATE).ok().flatten().and_then(|s| serde_json::from_str::<Value>(&s).ok());
+    let checked = v.as_ref().and_then(|v| v.get("checked_ms")).and_then(Value::as_u64);
+    match (v, checked) {
+        (Some(v), Some(t)) if now_ms.saturating_sub(t) <= STALE_AFTER_MS => v,
+        (_, Some(t)) => json!({"consumers": format!("unknown (no check since {t})"), "bound": null, "checked_ms": t}),
+        _ => json!({"consumers": "unknown (no check yet)", "bound": null}),
+    }
 }
 
 #[cfg(test)]
 mod tests {
     use super::*;
 
+    const FP: &str = "ed25519:mine";
+
     fn e(agent: &str, instance: &str, strength: &str, why: &str) -> Value {
-        json!({"agent": agent, "instance": instance, "strength": strength, "why": why})
+        let fp = if strength == "proven" { FP } else { "" };
+        json!({"agent": agent, "instance": instance, "strength": strength, "why": why, "key_fp": fp})
     }
 
     fn v(list: Vec<Value>) -> Value {
@@ -141,7 +163,7 @@ mod tests {
     #[test]
     fn only_me_proven_is_one() {
         let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "-", "proven", "")]);
-        let r = verdict("fx-a", "w1", Ok(&a));
+        let r = verdict("fx-a", "w1", FP, Ok(&a));
         assert_eq!(r.line, "one (proven)");
         assert!(r.others.is_empty());
     }
@@ -149,7 +171,7 @@ mod tests {
     #[test]
     fn a_second_identity_is_named_with_its_strength() {
         let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
-        let r = verdict("fx-a", "w1", Ok(&a));
+        let r = verdict("fx-a", "w1", FP, Ok(&a));
         assert_eq!(r.line, "inbox consumed by two identities: fx-a (proven), fx-z (claimed: unsigned)");
         assert_eq!(r.others.len(), 1);
     }
@@ -157,21 +179,25 @@ mod tests {
     #[test]
     fn same_agent_other_instance_is_a_second_copy() {
         let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", "w2", "proven", "")]);
-        let r = verdict("fx-a", "w1", Ok(&a));
+        let r = verdict("fx-a", "w1", FP, Ok(&a));
         assert!(r.line.contains("second copy, instance w2"), "{}", r.line);
     }
 
     #[test]
     fn unreadable_is_unknown_never_one() {
-        assert!(verdict("fx-a", "w1", Err((-32049, "consumer list unreadable"))).line.starts_with("unknown ("));
-        assert_eq!(verdict("fx-a", "w1", Err((-32601, "no such method"))).line, "unknown (hub predates the consumer list)");
-        assert!(verdict("fx-a", "w1", Ok(&json!({"bound": null}))).line.starts_with("unknown ("));
+        assert!(verdict("fx-a", "w1", FP, Err((-32049, "consumer list unreadable"))).line.starts_with("unknown ("));
+        assert_eq!(verdict("fx-a", "w1", FP, Err((-32601, "no such method"))).line, "unknown (hub predates the consumer list)");
+        assert!(verdict("fx-a", "w1", FP, Ok(&json!({"bound": null}))).line.starts_with("unknown ("));
     }
 
     #[test]
-    fn my_skewed_clock_shows_as_claimed() {
+    fn my_skewed_clock_is_flagged_not_hidden() {
+        // an unproven read under my name cannot be told from a spoof (r1 CC 1), so it is flagged; a skewed
+        // clock deserves the attention anyway (P-23 reports the skew)
         let a = v(vec![e("fx-a", "w1", "claimed", "clock skew")]);
-        assert_eq!(verdict("fx-a", "w1", Ok(&a)).line, "one (claimed: clock skew)");
+        let r = verdict("fx-a", "w1", FP, Ok(&a));
+        assert_eq!(r.line, "inbox consumed by two identities: fx-a (not seen), fx-a (claimed: clock skew) [claims this agent, instance w1]");
+        assert_eq!(r.others.len(), 1);
     }
 
     #[test]
@@ -179,11 +205,29 @@ mod tests {
         let est = crate::testkit::Estate::create_without_hub().unwrap();
         let store = Store::open(&est.store_root(), "fx-a").unwrap();
         let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-z", "local", "claimed", "unsigned")]);
-        let r = verdict("fx-a", "w1", Ok(&a));
+        let r = verdict("fx-a", "w1", FP, Ok(&a));
         record(&store, "inbox:h/fx-a", &r, 1_000);
         record(&store, "inbox:h/fx-a", &r, 2_000);
         let n = store.attention_entries().iter().filter(|l| l.contains(ATTENTION_KIND)).count();
         assert_eq!(n, 1);
-        assert_eq!(status(&store)["consumers"], r.line);
+        assert_eq!(status(&store, 2_000)["consumers"], r.line);
+        // r1 CC 2: an old check is not shown as current; a reset forgets it
+        assert!(status(&store, 2_000 + STALE_AFTER_MS + 1)["consumers"].as_str().unwrap().starts_with("unknown (no check since"));
+        reset(&store);
+        assert_eq!(status(&store, 2_000)["consumers"], "unknown (no check yet)");
+    }
+
+    #[test]
+    fn the_owners_name_without_its_key_is_not_me() {
+        // r1 CC 1 / OC 1: an unsigned reader using my agent and my instance, or the ack sentinel, is flagged
+        for inst in ["w1", "-"] {
+            let a = v(vec![e("fx-a", "w1", "proven", ""), e("fx-a", inst, "claimed", "unsigned")]);
+            let r = verdict("fx-a", "w1", FP, Ok(&a));
+            assert_eq!(r.others.len(), 1, "{inst}: {}", r.line);
+        }
+        // a proven entry by another key under my name is a second copy, not me
+        let a = json!({"consumers": [e("fx-a", "w1", "proven", ""),
+            {"agent": "fx-a", "instance": "w1", "strength": "proven", "why": "", "key_fp": "ed25519:other"}], "bound": null});
+        assert_eq!(verdict("fx-a", "w1", FP, Ok(&a)).others.len(), 1);
     }
 }
diff --git a/crates/termlink-sidecar/src/liveness.rs b/crates/termlink-sidecar/src/liveness.rs
index e90688739..e61701283 100644
--- a/crates/termlink-sidecar/src/liveness.rs
+++ b/crates/termlink-sidecar/src/liveness.rs
@@ -215,11 +215,14 @@ pub fn read(hub: &dyn Hub, topic: &str, inbox: &str) -> Read {
         Ok(n) => match hub.call(method::CHANNEL_FIRST_AT, json!({"topic": inbox, "from_offset": n.drained_up_to})) {
             Ok(v) => v.get("ts").and_then(Value::as_u64).filter(|_| v.get("offset").and_then(Value::as_u64).is_some()),
             Err((Some(code), _)) if code == error_code::CHANNEL_TOPIC_UNKNOWN => None,
-            // a hub that predates channel.first_at: the one-message read (recorded as a consumer there)
-            Err(_) => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {
+            // r1 CC 9 / OC 3: only a hub that predates channel.first_at falls back to the one-message read. A
+            // T-3700 hub's own errors name the method; an old hub routes the unknown method to a session
+            // (-32001 "Missing 'target'"). A transport error (no code) is Unreadable, never a content read.
+            Err((Some(_), m)) if !m.contains("channel.first_at") => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {
                 Ok(t) => t,
                 Err(e) => return Read::Unreadable(e),
             },
+            Err((_, e)) => return Read::Unreadable(format!("channel.first_at {inbox}: {e}")),
         },
     };
     Read::Note { note, signer, oldest_waiting_ms }
@@ -508,6 +511,22 @@ mod tests {
         assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"));
     }
 
+    /// r1 CC 9 / OC 3: only an old hub (-32001, the unknown method routed to a session) falls back to the
+    /// content read; a transport error or a T-3700 hub's own first_at error is Unreadable.
+    #[test]
+    fn read_falls_back_only_for_an_old_hub() {
+        let n = note(NOW, 2);
+        let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
+        f.replies.insert(("channel.first_at".into(), "I".into()), Err((Some(-32001), "Missing 'target' in params".into())));
+        assert!(matches!(read(&f, "L", "I"), Read::Note { oldest_waiting_ms: Some(1234), .. }));
+        for err in [(None, "connection reset".to_string()), (Some(-32603), "channel.first_at: io".to_string())] {
+            let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
+            f.replies.insert(("channel.first_at".into(), "I".into()), Err(err.clone()));
+            assert!(matches!(read(&f, "L", "I"), Read::Unreadable(_)), "{err:?}");
+            assert!(!f.calls.borrow().iter().any(|c| c == "channel.subscribe I"), "{err:?}");
+        }
+    }
+
     /// A fake hub: canned replies per (method, topic).
     struct Fake {
         replies: BTreeMap<(String, String), HubResult>,
@@ -534,6 +553,8 @@ mod tests {
                 (("channel.list".into(), "L".into()), Ok(json!({"topics": [{"name": "L", "latest_offset": 4}, {"name": "L2"}]}))),
                 (("channel.subscribe".into(), "L".into()), Ok(json!({"messages": [envelope(4, NOTE_TYPE, &body, n.sample_ms)]}))),
                 (("channel.subscribe".into(), "I".into()), inbox),
+                // an old hub by default: the unknown method is routed to a session (T-3700 fallback)
+                (("channel.first_at".into(), "I".into()), Err((Some(-32001), "Missing 'target' in params".into()))),
             ]),
             calls: RefCell::new(vec![]),
         }
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index 227dc2516..6646063ac 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -186,6 +186,8 @@ impl Receiver {
         let rt = tokio::runtime::Builder::new_multi_thread().worker_threads(1).enable_all().build()?;
         let interval = cfg.poll_interval;
         let revocation = Mutex::new(crate::revocation::Tracker::new(&store, &cfg.declared_hub_id, crate::revocation::PN13));
+        // T-3700 (r1 CC 2): a consumers line from before this start is never shown
+        crate::consumers::reset(&store);
         let inner = Arc::new(Inner {
             cfg,
             store,
@@ -404,10 +406,12 @@ impl Inner {
         mem.consumers_checked_ms = now;
         let topic = inbox_topic(&self.cfg.declared_hub_id, &self.cfg.agent);
         let answer = self.rpc("inbox.consumers", json!({"topic": topic, "window_s": crate::consumers::WINDOW_S}));
+        let fp = self.sender.key_fp();
+        let (me, inst) = (&self.cfg.agent, &self.cfg.instance);
         let v = match &answer {
-            Ok(RpcResponse::Success(s)) => crate::consumers::verdict(&self.cfg.agent, &self.cfg.instance, Ok(&s.result)),
-            Ok(RpcResponse::Error(e)) => crate::consumers::verdict(&self.cfg.agent, &self.cfg.instance, Err((e.error.code, &e.error.message))),
-            Err(e) => crate::consumers::verdict(&self.cfg.agent, &self.cfg.instance, Err((0, &format!("hub unreachable: {e}")))),
+            Ok(RpcResponse::Success(s)) => crate::consumers::verdict(me, inst, &fp, Ok(&s.result)),
+            Ok(RpcResponse::Error(e)) => crate::consumers::verdict(me, inst, &fp, Err((e.error.code, &e.error.message))),
+            Err(e) => crate::consumers::verdict(me, inst, &fp, Err((0, &format!("hub unreachable: {e}")))),
         };
         crate::consumers::record(&self.store, &topic, &v, now);
     }
diff --git a/crates/termlink-sidecar/src/send.rs b/crates/termlink-sidecar/src/send.rs
index 8ab4b13d9..ceca5fba5 100644
--- a/crates/termlink-sidecar/src/send.rs
+++ b/crates/termlink-sidecar/src/send.rs
@@ -316,6 +316,12 @@ impl Sender {
         })
     }
 
+    /// T-3700: this agent's signing key fingerprint as the hub records it (`ed25519:` + sha256 of the key).
+    pub fn key_fp(&self) -> String {
+        let pk = self.inner.cfg.identity.public_key_hex();
+        termlink_session::signed_state::fingerprint_of_hex(&pk).unwrap_or_default()
+    }
+
     /// The agent's own fingerprint.
     pub fn fingerprint(&self) -> &str {
         self.inner.cfg.identity.fingerprint()
diff --git a/crates/termlink-sidecar/tests/t3700_consumers.rs b/crates/termlink-sidecar/tests/t3700_consumers.rs
index 5d02d3c61..d4d110714 100644
--- a/crates/termlink-sidecar/tests/t3700_consumers.rs
+++ b/crates/termlink-sidecar/tests/t3700_consumers.rs
@@ -80,7 +80,7 @@ fn put_card(e: &Estate, agent: &str, who: &str) {
 }
 
 fn line(store: &Store) -> String {
-    termlink_sidecar::consumers::status(store)["consumers"].as_str().unwrap_or("").to_string()
+    termlink_sidecar::consumers::status(store, now_ms() as u64)["consumers"].as_str().unwrap_or("").to_string()
 }
 
 fn attention_count(store: &Store) -> usize {
@@ -144,7 +144,7 @@ fn e66_second_consumer_flagged_then_refused_after_bind() {
     assert_eq!(admin_rows(&e, "inbox-bind"), 1);
 
     // fx-z unsigned, fx-z naming fx-a unsigned, an unnamed peek: all refused with a reason
-    for reader in [Some(json!({"agent": "fx-z"})), Some(json!({"agent": "fx-a", "instance": "x"})), None] {
+    for reader in [Some(json!({"agent": "fx-z", "instance": "raw"})), Some(json!({"agent": "fx-a", "instance": "x"})), None] {
         let r = raw_read(&e, &topic, reader.clone());
         assert_eq!(r["error"]["code"], INBOX_BOUND, "{reader:?}: {r}");
         assert!(r["error"]["message"].as_str().unwrap().contains("inbox bound to fx-a"), "{r}");
@@ -196,7 +196,7 @@ fn e66_ack_is_a_consumer_and_17_readers_evict_the_oldest() {
     // 17 more named readers: 16 kept, the oldest (fx-b's ack) evicted with an admin row
     for i in 0..17 {
         std::thread::sleep(Duration::from_millis(2));
-        let _ = raw_read(&e, &topic, Some(json!({"agent": format!("fx-r{i}")})));
+        let _ = raw_read(&e, &topic, Some(json!({"agent": format!("fx-r{i}"), "instance": "raw"})));
     }
     let c = consumers(&e, &topic);
     assert_eq!(c.len(), 16, "{c:?}");
@@ -251,3 +251,118 @@ fn liveness_first_at_is_not_a_consumer() {
     assert!(r["result"].get("messages").is_none(), "no content: {r}");
     assert!(consumers(&e, &topic).is_empty(), "first_at records no consumer");
 }
+
+/// A signed `receipt` post on `topic` by `key` naming `from_agent`, at hub-clock time `ts`.
+fn post_receipt(e: &Estate, topic: &str, key: &Identity, from_agent: &str, ts: i64) -> Value {
+    let payload = format!(r#"{{"from_agent":"{from_agent}","up_to":0}}"#);
+    let sig = key.sign_hex(&termlink_protocol::control::channel::canonical_sign_bytes(topic, "receipt", payload.as_bytes(), None, ts));
+    rpc(e, "channel.post", json!({"topic": topic, "msg_type": "receipt",
+        "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()), "ts": ts,
+        "sender_id": key.fingerprint(), "sender_pubkey_hex": key.public_key_hex(), "signature_hex": sig}))
+}
+
+fn now_ms() -> i64 {
+    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_millis() as i64
+}
+
+fn signed_claim(e: &Estate, id: &str, topic: &str, key: &Identity, agent: &str, offset: u64) -> Value {
+    let ts = now_ms();
+    let b = termlink_session::inbox_read::read_sign_bytes(id, topic, "channel.claim", offset, ts, agent, "inst");
+    let reader = json!({"agent": agent, "instance": "inst", "ts_ms": ts, "signer_pk": key.public_key_hex(), "sig": key.sign_hex(&b)});
+    rpc(e, "channel.claim", json!({"topic": topic, "offset": offset, "claimer": agent, "reader": reader}))
+}
+
+#[test]
+fn e66_claims_and_acks_through_the_hub_before_and_after_a_bind() {
+    let e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    let topic = format!("inbox:{id}/fx-a");
+    let _ = rpc(&e, "channel.create", json!({"name": topic}));
+    put_card(&e, "fx-a", "a");
+    put_card(&e, "fx-b", "b");
+    let (ka, kb) = (e.identity("a").unwrap(), e.identity("b").unwrap());
+    let a = side(&e, &id, "a", "inst-a");
+    let _ = a.sender.send("fx-a", "inst-a", Some("c1"), "one message to claim", None);
+
+    // before a bind: fx-b claims fx-a's inbox (signed) and acks it; both are recorded proven as fx-b
+    let c = signed_claim(&e, &id, &topic, &kb, "fx-b", 0);
+    assert!(c.get("result").is_some(), "{c}");
+    assert!(post_receipt(&e, &topic, &kb, "fx-b", now_ms()).get("result").is_some());
+    let list = consumers(&e, &topic);
+    assert!(list.iter().any(|x| x["agent"] == "fx-b" && x["actions"].as_array().unwrap().iter().any(|a| a == "claim")), "{list:?}");
+
+    // after the bind: fx-a's signed claim and fresh ack pass; fx-b's are refused; an unsigned claim is refused
+    let b = rpc(&e, "hub.inbox_bind", json!({"topic": topic, "agent": "fx-a"}));
+    assert_eq!(b["result"]["ok"], true, "{b}");
+    let claim_id = c["result"]["claim_id"].clone();
+    let rel = rpc(&e, "channel.release", json!({"topic": topic, "claim_id": claim_id, "claimer": "fx-b"}));
+    let ca = signed_claim(&e, &id, &topic, &ka, "fx-a", 0);
+    assert!(ca.get("result").is_some(), "release: {rel}\nclaim: {ca}");
+    assert!(post_receipt(&e, &topic, &ka, "fx-a", now_ms()).get("result").is_some());
+    assert_eq!(signed_claim(&e, &id, &topic, &kb, "fx-b", 0)["error"]["code"], INBOX_BOUND);
+    assert_eq!(post_receipt(&e, &topic, &kb, "fx-b", now_ms())["error"]["code"], INBOX_BOUND);
+    let unsigned = rpc(&e, "channel.claim", json!({"topic": topic, "offset": 0, "claimer": "fx-a", "reader": {"agent": "fx-a", "instance": "x"}}));
+    assert_eq!(unsigned["error"]["code"], INBOX_BOUND, "{unsigned}");
+    // r1 CC 7: a replay of fx-a's OLD receipt (ts outside the window) is refused after the bind
+    assert_eq!(post_receipt(&e, &topic, &ka, "fx-a", now_ms() - 300_000)["error"]["code"], INBOX_BOUND);
+    println!("E-66 PASS (claims and acks recorded, and refused after bind unless proven by the bound key)");
+}
+
+#[test]
+fn e66_an_unsigned_reader_using_the_owners_name_and_instance_is_flagged() {
+    let e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    let topic = format!("inbox:{id}/fx-a");
+    let _ = rpc(&e, "channel.create", json!({"name": topic}));
+    put_card(&e, "fx-a", "a");
+    let a = side(&e, &id, "a", "inst-a");
+    let _ = a.rx.pass();
+    // r1 CC 1 / OC 1: the owner's own name and instance, unsigned
+    let r = raw_read(&e, &topic, Some(json!({"agent": "fx-a", "instance": "inst-a"})));
+    assert!(r.get("result").is_some(), "{r}");
+    // the ack sentinel may not be claimed at all
+    let r = raw_read(&e, &topic, Some(json!({"agent": "fx-a", "instance": "-"})));
+    assert_eq!(r["error"]["code"], -32602, "{r}");
+    let _ = a.rx.pass(); // the owner's signed read again (it must not absorb the claimed entry)
+    a.rx.check_consumers_now();
+    let l = line(&a.store);
+    assert_eq!(l, "inbox consumed by two identities: fx-a (proven), fx-a (claimed: unsigned) [claims this agent, instance inst-a]", "{l}");
+    println!("E-66 PASS (a name-only reader using the owner's identity is flagged)");
+}
+
+#[test]
+fn a_long_poll_opened_before_a_bind_does_not_deliver_after_it() {
+    // r1 CC 6
+    let e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    let topic = format!("inbox:{id}/fx-a");
+    let _ = rpc(&e, "channel.create", json!({"name": topic}));
+    put_card(&e, "fx-a", "a");
+    let addr = e.hub_addr();
+    let t = topic.clone();
+    let poller = std::thread::spawn(move || {
+        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
+        let p = json!({"topic": t, "cursor": 0, "limit": 5, "timeout_ms": 4000, "reader": {"agent": "fx-z", "instance": "raw"}});
+        let r = rt.block_on(termlink_session::client::rpc_call_addr_with_timeout(&addr, "channel.subscribe", p, Duration::from_secs(10))).unwrap();
+        serde_json::to_value(&r).unwrap()
+    });
+    std::thread::sleep(Duration::from_millis(500));
+    let b = rpc(&e, "hub.inbox_bind", json!({"topic": topic, "agent": "fx-a"}));
+    assert_eq!(b["result"]["ok"], true, "{b}");
+    let a = side(&e, &id, "a", "inst-a");
+    let _ = a.sender.send("fx-a", "inst-a", Some("c1"), "posted after the bind", None);
+    let r = poller.join().unwrap();
+    assert_eq!(r["error"]["code"], INBOX_BOUND, "{r}");
+    println!("E-66 PASS (a long-poll that waited across the bind is refused)");
+}
+
+#[test]
+fn a_read_of_a_topic_that_does_not_exist_records_nothing() {
+    // r1 CC 3 / OC 2: junk inbox names cannot grow the hub's table
+    let e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    let ghost = format!("inbox:{id}/no-such-agent");
+    let r = raw_read(&e, &ghost, Some(json!({"agent": "fx-z", "instance": "raw"})));
+    assert!(r.get("error").is_some(), "{r}");
+    assert!(consumers(&e, &ghost).is_empty());
+}
```

## Measured
hub+bus suites 909 passed 0 failed; sidecar suite 375 passed 0 failed; hub unit inbox_consumers 30 passed;
E-66 PASS (ack recorded; 17 readers keep 16, eviction logged)
E-66 PASS (a name-only reader using the owner's identity is flagged)
E-169 PASS (unreadable consumer list shown unknown, never one)
E-133 PASS (the per-tick check fired on the injected second consumer)
E-66 PASS (claims and acks recorded, and refused after bind unless proven by the bound key)
E-66 PASS (a long-poll that waited across the bind is refused)
E-66 PASS (V-12 second copy seen within one check)
E-66 PASS (flagged naming both, read-only consumer counted, refused after bind with reason and row)
test result: ok. 9 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 3.19s

# T-3700 mutants — P-12 / B-2 (AL-30)

Runner: `python3 crates/termlink-session/tests/mutants/run-t3700.py` (scratch runtime dir, T-3822; rebuilds the clean
binary at the end). Every mutant must be killed (FP 3.1.b). m4/m6/m7 are the E-169.5, E-66.5-style and E-66.6
controls; m16–m26 cover the review-round-1 fixes.

## Round-2 run, 2026-11-20

| mutant | what | verdict |
|---|---|---|
| m1-skip-signature-check | a read proof is accepted without verifying its signature | killed |
| m2-ignore-ts-window | a proof with a skewed timestamp still proves | killed |
| m3-bind-by-name | a bind admits any reader naming the bound agent (ruling E broken) | killed |
| m4-hub-unreadable-is-empty | E-169.5 control: an unreadable consumer list answers as a list | SURVIVED, then killed (see below) |
| m5-sidecar-malformed-is-one | a malformed hub answer is shown as one consumer | killed |
| m6-acks-not-recorded | E-66.5-style control: an ack (receipt) does not record its poster | killed |
| m7-reads-not-recorded | E-66.6 control: only acks and claims record; a read-only consumer is missed | killed |
| m8-no-eviction-row | the 17th reader evicts silently | killed |
| m9-hub-instance-ignored | V-12: the hub keys consumers without the instance (a second copy merges) | killed |
| m10-sidecar-instance-ignored | V-12: the sidecar treats its own key at any instance as itself | killed |
| m11-card-mismatch-proves | a proof signed by a key not on the named card proves | killed |
| m12-revoked-proves | a revoked key still proves | killed |
| m13-bind-without-row | a bind takes effect when its admin row could not be written | killed |
| m14-attention-every-check | a second consumer raises an entry on every check | killed |
| m15-liveness-reads-content | liveness reads the main's inbox content instead of first_at | killed |
| m16-claim-gate-removed | a claim on an inbox is neither recorded nor refused after a bind | killed |
| m17-bindings-fail-open | unreadable bindings are treated as none (every bound inbox opens) | killed |
| m18-strength-merge | r1 CC 1: a claimed reader merges into the proven entry of the same name and instance | killed |
| m19-self-by-name | r1 CC 1: the sidecar takes any entry under its name and instance for itself | killed |
| m20-no-long-poll-recheck | r1 CC 6: a long-poll that waited across a bind still delivers | killed |
| m21-liveness-fallback-any-error | r1 CC 9: liveness falls back to a content read on any first_at error | killed |
| m22-ack-replay-proves | r1 CC 7: a replayed old receipt proves | killed |
| m23-corrupt-overwritten | r1 CC 4: a corrupt table is not moved aside (saved over) | killed |
| m24-stale-status-shown | r1 CC 2: an old consumers line is shown as current | killed |
| m25-unknown-topics-recorded | r1 CC 3: reads of inbox topics that do not exist are recorded | killed |
| m26-full-table-evicts-active | r1 CC 3: a full table evicts an inbox read inside the window | killed |

m4 survived the full run: after round 1 an unreadable table is moved aside (the "history incomplete" path), so no
test made a SAVE fail any more. Added `a_failed_save_makes_the_list_an_error_never_a_list`; re-run of m4 alone: killed.

**26/26 killed**

## Round-1 run, 2026-11-19: 15/15 killed (m1–m15)

## Finding during the round-1 run

A mutant killed after `cargo build -p termlink` (m9) left its mutated hub binary in target/debug; L4 tests start
that binary without rebuilding, so V-12 then failed on the clean tree. run-t3700.py and run-t3823.py now rebuild the
clean binary at the end. Production is unaffected: runme installs its own `target/local-fast` build.
