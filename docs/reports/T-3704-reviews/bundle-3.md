# T-3704 slice A confirmation round (round 3)

Round 2 reviewed slice A's code; one seat PASS with findings, one FAIL (1 BLOCKER). Below: the round-2 findings' dispositions, the code changes made since round 2, the updated tests and the measured results. Judge whether each round-2 finding is closed as its disposition says, and whether the changes introduce a new defect.

## Measured results (2026-11-18)
- hub lib 627 passed, 0 failed; sidecar lib 73/0; real-hub t3704_key_card 2/2; t3704_impersonation 3/3
- mutants 21 of 21 killed (table below)

# T-3704 code review round 2 — dispositions

Seats: opencode (`review-2-opencode.md`, PASS with 2 MAJOR, 3 MINOR), claude-code (`review-2-claude-code.md`, FAIL:
1 BLOCKER, 7 MAJOR, 4 MINOR). Neither seat found a forged arc-011 post that passes the hub in enforce mode. A = accepted
and changed, R = rejected with reason, D = deferred with reason. O-n / C-n = the seats' finding numbers.

| # | Finding | Disp. | Change |
|---|---|---|---|
| C-1 BLOCKER, O-1 | the put persisted the card first and logged after, best effort; no needs-attention consumer | A | `key_card::put` now takes the row writer and calls it BEFORE the store is written; a failed row refuses the put and writes nothing (unit `r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing`; real hub: admin log made unwritable → put refused, card unchanged). Same for the setting (`set_mode`). The row names old and new project, signing key, encryption key, instances, seq and the card digest. Consumer: every sidecar raises a `key-card-written` needs-attention entry per (agent, seq) from `hub.key_card_list` (real-hub test). |
| C-2 | a direct file edit with the same seq passes until restart and reconcile compares seq only | A | the store digest is baselined at start and after each write; a store that differs is refused for every gated post and every put ("changed outside hub.key_card_put"), until a restart re-baselines it with an attention row. Start-up compares each card's digest with its last logged `card_digest`. Unit `r2_an_outside_edit_with_the_same_seq_is_refused_until_restart`. |
| C-3 | encryption key / instances change without replace | A | any field change needs `replace: true`; a signing key may be on one card only. Unit `r2_every_field_change_needs_replace_and_a_key_has_one_card`. |
| C-4 | receiver name rule applies with an empty roster; no log window | A (partly R) | `check_agent` checks nothing while the roster has no named entry, so an upgraded receiver refuses nothing new until it is configured with names (the named roster is the receiver's switch). R: a separate off/log/enforce at the receiver and an "unbound" status counter — the named-roster switch already gives a no-change roll-out; recorded as a deviation from design v2 item 11. |
| C-5, O-2 | missing proofs: receiver type filter, unwritable-row test, receiver E-69 on a real hub, two projects at L4 | A (one D) | added: `an_unlisted_type_on_the_inbox_is_skipped_never_handled_as_mail` (real hub) + mutant m20; the unwritable-row case (unit + real hub) + mutant m18; Q's key claiming P's agent on the real hub. D: the lock-free-read mutant — the store uses one global lock, and a concurrency mutant cannot be made deterministic; noted as a test limit. The receiver half stays at unit level (wire tests with real keys and signatures). |
| C-6 | no test sends a genuine sidecar Msg/Stage through the hub in enforce | A | `real_sidecar_msg_and_stage_pass_enforce_with_their_card_and_an_impostor_does_not`: a real `Sender` sends a msg and a stage with its card in enforce, both stored; an impostor sidecar with another key configured as that agent is refused. |
| C-7 | counters and coalescing keyed on attacker-chosen names | A | refusals naming an agent without a card share one `_uncarded` bucket; per-agent signer lists are capped at 16; refusal rows are capped at 60 per minute in all. Unit `r2_refusals_for_uncarded_names_share_one_bucket_and_rows_are_capped`. |
| C-8, O-3 | mode and counts promised in `hub.status`, delivered in `hub.key_card_list` | A (design amended) | the design text is amended: they are in `hub.key_card_list` (observe scope), which the sidecar reads. `hub.status` is a session-level verb unrelated to the hub's card state. |
| C-9 | alias refused; duplicate names silently overwrite | A | a key bound to several names may speak as any of them; `roster_of` refuses a name given twice (unit). |
| C-10 | setting only a file; deletion unlogged; write race; read on every post | A | start-up compares the setting with the last logged change and reports a difference; `write_mode` runs under the store lock; the gate tests topic and type before reading the setting. Unit `r2_mode_changes_are_logged_first_and_reconciled_at_start`. |
| C-11 | restart hides refusals; own refused posts reported as impersonation | A | the sidecar keys its seen count by `hub_start_ms` from the list; refusals signed by the agent's own key are excluded (per-signer counts from the hub). |
| C-12 | `by: "runme"` hard-coded; mode read is admin; per-agent lock; same key on two cards; test count; CLI name | A / R | `by` now says `hub.key_card_put (local socket)` (the peer is not authenticated beyond the socket; the stated limit); `hub.key_card_mode` requires `mode` (reads go through `hub.key_card_list`); same key on two cards refused. R: per-agent locking — the store is one small file; one lock is simpler and correct (no nesting). The CLI verb is `key-card mode`; the design text said `key-card-check` (the file name); design amended. |
| O-4 | E-69 L4 trimmed | A | two-project case added at the real hub; the hub-secret-only case is the local-socket post with a fresh key (the same code path the secret grants). |
| O-5 | global lock latency on put | R | puts are rare runme acts; the fsync is milliseconds; recorded. |

## Mutants
| FC | Mutant | What it breaks | Test that must go red | Result |
|---|---|---|---|---|
| FCW-133 / E-69.5 | `t3704-m1-fc-01-verify-accepts-any-signature` | the card key is not compared with the signer (plan control fc-01) | `key_card::tests::e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes` | killed |
| FCW-128 | `t3704-m10-put-not-admin` | card writes are not admin RPCs (no intent row first) | `router::tests::t3704_key_card_writes_are_refused_off_the_local_socket` | killed |
| FCW-98 / E-69.4 | `t3704-m11-receiver-ignores-bound-name` | the receiver does not check a bound from_agent against the key | `wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key` | killed |
| FCW-98 | `t3704-m12-open-path-skips-names` | an unrostered key may claim a bound agent name | `wire::wire_tests::t3704_named_roster_binds_from_agent_to_its_key` | killed |
| design v2 C-12 | `t3704-m13-no-refusal-counts` | refusals are not counted for the impersonated agent | `e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused` | killed |
| AL-33 step 2 (admin_log) | `t3704-m14-refusals-never-logged` | no admin row is ever written for a refusal | `key_card::tests::refusal_rows_are_coalesced_per_agent_key_minute` | killed |
| design v2 C-12 (sidecar) | `t3704-m15-impersonation-silent` | the impersonated agent's sidecar never raises the attention entry | `a_refused_post_claiming_to_be_me_raises_one_attention_entry` | killed |
| AL-33 2.17.f(2) | `t3704-m16-no-startup-reconcile` | a card ahead of the admin log is never reported at start | `key_card::tests::al33_startup_reconcile_flags_a_card_the_admin_log_does_not_show` | killed |
| r2 MAJOR 2 | `t3704-m17-outside-edit-accepted` | a card store edited outside hub.key_card_put is used | `key_card::tests::r2_an_outside_edit_with_the_same_seq_is_refused_until_restart` | killed |
| r2 BLOCKER (operator ruling A) | `t3704-m18-row-not-first` | the card is written without its admin row | `key_card::tests::r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing` | killed |
| r2 MAJOR 3 | `t3704-m19-encryption-key-silent` | the encryption key changes without replace | `key_card::tests::r2_every_field_change_needs_replace_and_a_key_has_one_card` | killed |
| FCW-135 / E-169 | `t3704-m2-no-card-passes` | a poster without a card passes | `key_card::tests::e169_fcw135_a_poster_without_a_card_is_refused` | killed |
| r2 MAJOR 2/5 (O-4/C-4) | `t3704-m20-receiver-admits-other-type` | the receiver handles an unlisted type as mail | `an_unlisted_type_on_the_inbox_is_skipped_never_handled_as_mail` | killed |
| r2 MINOR 10 | `t3704-m21-mode-not-reconciled` | a setting changed outside the RPC is not reported at start | `key_card::tests::r2_mode_changes_are_logged_first_and_reconciled_at_start` | killed |
| design v2 C-M1 | `t3704-m3-no-name-check` | from_agent is used without the plain-name check | `key_card::tests::a_traversal_name_is_refused_and_never_read_as_a_path` | killed |
| design v2 O-M3 | `t3704-m4-corrupt-store-as-empty` | a corrupt card store reads as no cards | `key_card::tests::an_unreadable_store_refuses_never_reads_as_empty` | killed |
| design v2 C-3 | `t3704-m5-replace-not-required` | a card key or project changes without replace | `key_card::tests::a_key_or_project_change_needs_replace_and_seq_rises` | killed |
| design v2 (fail closed) | `t3704-m6-bad-setting-is-off` | an unreadable or invalid setting reads as off | `key_card::tests::mode_defaults_off_and_a_bad_setting_enforces` | killed |
| FCW-133 at the hub | `t3704-m7-enforce-stores` | in enforce mode a refused post is stored anyway | `e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused` | killed |
| NID | `t3704-m8-gate-every-type` | the check also gates other message types and topics | `key_card::tests::other_types_and_topics_are_never_gated` | killed |
| operator ruling A (local socket only) | `t3704-m9-put-over-tcp` | a card write is accepted from a TCP peer | `router::tests::t3704_key_card_writes_are_refused_off_the_local_socket` | killed |

PASS: all 21 mutants turned their test red. Run 2026-11-18T01:46Z on 3d8e0cd01.

## Code changes since round 2 (crates/*/src)
```diff
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index 66fe80a01..b07611e81 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -2900,16 +2900,19 @@ pub(crate) fn key_card_gate(
     signer_fp: &str,
 ) -> Option<RpcResponse> {
     use crate::key_card::{self, Check, Mode};
+    if !key_card::is_gated(topic, msg_type) {
+        return None; // other types and topics never read the setting or the store (NID)
+    }
     let mode = key_card::effective_mode(dir);
     if mode == Mode::Off {
         return None;
     }
-    let Check::Refuse { code, fcw, agent, reason } = key_card::check_post(dir, topic, msg_type, payload, signer.as_bytes())
+    let Check::Refuse { code, fcw, agent, reason, carded } = key_card::check_post(dir, topic, msg_type, payload, signer.as_bytes())
     else {
         return None;
     };
     let enforced = mode == Mode::Enforce;
-    key_card::note_refusal(&agent, enforced);
+    key_card::note_refusal(&agent, signer_fp, enforced, carded);
     let now = std::time::SystemTime::now()
         .duration_since(std::time::UNIX_EPOCH)
         .map(|d| d.as_millis() as u64)
diff --git a/crates/termlink-hub/src/key_card.rs b/crates/termlink-hub/src/key_card.rs
index e4bd115b9..3d5d9d03e 100644
--- a/crates/termlink-hub/src/key_card.rs
+++ b/crates/termlink-hub/src/key_card.rs
@@ -75,9 +75,24 @@ pub fn effective_mode(dir: &Path) -> Mode {
 }
 
 pub fn write_mode(dir: &Path, mode: Mode) -> std::io::Result<()> {
+    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
     atomic_write(&dir.join(MODE_FILE), format!("{}\n", mode.as_str()).as_bytes(), 0o644)
 }
 
+/// Change the setting with its admin row written FIRST (r2 BLOCKER: refused if the row cannot be
+/// written). `log(before, after)` writes the row.
+pub fn set_mode(dir: &Path, mode: Mode, log: impl FnOnce(&str, &str) -> Result<(), String>) -> Result<String, String> {
+    let before = read_mode(dir).map(|m| m.as_str().to_string()).unwrap_or_else(|_| "invalid".into());
+    log(&before, mode.as_str())?;
+    write_mode(dir, mode).map_err(|e| format!("setting not written: {e}"))?;
+    Ok(before)
+}
+
+/// The arc-011 types on `inbox:*` the check applies to (tested before anything is read).
+pub fn is_gated(topic: &str, msg_type: &str) -> bool {
+    topic.starts_with("inbox:") && GATED_TYPES.contains(&msg_type)
+}
+
 #[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
 pub struct Card {
     pub schema: String,
@@ -134,21 +149,57 @@ pub fn store_path(dir: &Path) -> PathBuf {
     dir.join(STORE_FILE)
 }
 
+pub fn sha256_hex(b: &[u8]) -> String {
+    use sha2::{Digest, Sha256};
+    Sha256::digest(b).iter().map(|x| format!("{x:02x}")).collect()
+}
+
+/// sha256 of a card's serialized form (the admin row carries it; start-up compares it).
+pub fn card_digest(c: &Card) -> String {
+    sha256_hex(&serde_json::to_vec(c).unwrap_or_default())
+}
+
+const ABSENT: &str = "absent";
+
+/// The store digest this hub last wrote or baselined, per runtime dir (r2 MAJOR 2: a store changed
+/// outside `hub.key_card_put` is refused until the next start re-baselines it, with attention).
+/// No entry = not baselined (unit tests; a hub that never ran the start-up reconcile).
+static KNOWN_DIGEST: Mutex<BTreeMap<PathBuf, String>> = Mutex::new(BTreeMap::new());
+
+fn set_known(dir: &Path, digest: String) {
+    KNOWN_DIGEST.lock().unwrap_or_else(|p| p.into_inner()).insert(dir.to_path_buf(), digest);
+}
+
 /// Absent store = no cards. A store that exists but cannot be read or parsed is an `Err`, never
-/// "no cards" (design v2: an unreadable store refuses every gated post).
-fn load(dir: &Path) -> Result<Store, String> {
+/// "no cards" (design v2: an unreadable store refuses every gated post). Returns the file digest.
+fn load_raw(dir: &Path) -> Result<(Store, String), String> {
     let p = store_path(dir);
-    let text = match std::fs::read_to_string(&p) {
-        Ok(t) => t,
+    let bytes = match std::fs::read(&p) {
+        Ok(b) => b,
         Err(e) if e.kind() == std::io::ErrorKind::NotFound => {
-            return Ok(Store { schema: STORE_SCHEMA.into(), cards: BTreeMap::new() });
+            return Ok((Store { schema: STORE_SCHEMA.into(), cards: BTreeMap::new() }, ABSENT.into()));
         }
         Err(e) => return Err(format!("key-card store unreadable: {e}")),
     };
-    let s: Store = serde_json::from_str(&text).map_err(|e| format!("key-card store corrupt: {e}"))?;
+    let s: Store = serde_json::from_slice(&bytes).map_err(|e| format!("key-card store corrupt: {e}"))?;
     if s.schema != STORE_SCHEMA {
         return Err(format!("key-card store has schema {:?}, not {STORE_SCHEMA}", s.schema));
     }
+    Ok((s, sha256_hex(&bytes)))
+}
+
+/// `load_raw` plus the outside-edit check against the baselined digest.
+fn load(dir: &Path) -> Result<Store, String> {
+    let (s, d) = load_raw(dir)?;
+    if let Some(k) = KNOWN_DIGEST.lock().unwrap_or_else(|p| p.into_inner()).get(dir)
+        && *k != d
+    {
+        return Err(format!(
+            "key-card store changed outside hub.key_card_put (digest {} expected {}); restart the hub to re-baseline (raises attention)",
+            &d[..12.min(d.len())],
+            &k[..12.min(k.len())]
+        ));
+    }
     Ok(s)
 }
 
@@ -173,7 +224,9 @@ fn atomic_write(path: &Path, bytes: &[u8], mode: u32) -> std::io::Result<()> {
 
 fn save(dir: &Path, s: &Store) -> std::io::Result<()> {
     let text = serde_json::to_string_pretty(s).map_err(std::io::Error::other)?;
-    atomic_write(&store_path(dir), text.as_bytes(), 0o600)
+    atomic_write(&store_path(dir), text.as_bytes(), 0o600)?;
+    set_known(dir, sha256_hex(text.as_bytes()));
+    Ok(())
 }
 
 /// The outcome of the post-time check.
@@ -182,11 +235,13 @@ pub enum Check {
     /// Not an arc-011 type on `inbox:*`: the check does not apply.
     NotGated,
     Pass { agent: String },
-    Refuse { code: i64, fcw: &'static str, agent: String, reason: String },
+    /// `carded`: the claimed agent has a card (refusal counts are kept per agent only then; the
+    /// rest share one bucket, so attacker-chosen names cannot grow the map, r2 MAJOR 7).
+    Refuse { code: i64, fcw: &'static str, agent: String, reason: String, carded: bool },
 }
 
 fn refuse_no_card(agent: &str, reason: String) -> Check {
-    Check::Refuse { code: error_code::CHANNEL_NO_KEY_CARD, fcw: "FCW-135", agent: agent.to_string(), reason }
+    Check::Refuse { code: error_code::CHANNEL_NO_KEY_CARD, fcw: "FCW-135", agent: agent.to_string(), reason, carded: false }
 }
 
 /// The claimed agent of an arc-011 payload: its `from_agent` (msg and stage records both carry it).
@@ -197,7 +252,7 @@ pub fn claimed_agent(payload: &[u8]) -> Option<String> {
 
 /// The card check for one post whose transport signature has already verified with `signer`.
 pub fn check_post(dir: &Path, topic: &str, msg_type: &str, payload: &[u8], signer: &[u8; 32]) -> Check {
-    if !topic.starts_with("inbox:") || !GATED_TYPES.contains(&msg_type) {
+    if !is_gated(topic, msg_type) {
         return Check::NotGated;
     }
     let agent = match claimed_agent(payload) {
@@ -223,6 +278,7 @@ pub fn check_post(dir: &Path, topic: &str, msg_type: &str, payload: &[u8], signe
             fcw: "FCW-133",
             agent: agent.clone(),
             reason: format!("the post is not signed by the key on agent {agent:?}'s card"),
+            carded: true,
         },
         None => refuse_no_card(&agent, format!("agent {agent:?}'s card holds no valid signing key")),
     }
@@ -242,9 +298,16 @@ pub struct PutRequest {
     pub replace: bool,
 }
 
-/// Write or replace one card. Returns the new card and the one it replaced. A change of an
-/// existing card's signing key or project needs `replace: true`, so it is never silent.
-pub fn put(dir: &Path, req: &PutRequest, now_ms: u64) -> Result<(Card, Option<Card>), String> {
+/// Write or replace one card. Returns the new card and the one it replaced. Any change to an
+/// existing card (key, project, encryption key, instances) needs `replace: true`, so it is never
+/// silent (r2 MAJOR 3). A signing key may sit on one card only. `log(new, old)` writes the admin
+/// row and runs BEFORE the store is written; if it fails nothing is written (r2 BLOCKER).
+pub fn put(
+    dir: &Path,
+    req: &PutRequest,
+    now_ms: u64,
+    log: impl FnOnce(&Card, Option<&Card>) -> Result<(), String>,
+) -> Result<(Card, Option<Card>), String> {
     if !valid_name(&req.agent) {
         return Err(format!("agent {:?} is not a plain name", req.agent));
     }
@@ -266,14 +329,20 @@ pub fn put(dir: &Path, req: &PutRequest, now_ms: u64) -> Result<(Card, Option<Ca
     let mut store = load(dir)?;
     let old = store.cards.get(&req.agent).cloned();
     if let Some(o) = &old
-        && (o.signing_key != req.signing_key || o.project != req.project)
+        && (o.signing_key != req.signing_key
+            || o.project != req.project
+            || o.encryption_key != req.encryption_key
+            || o.instances != req.instances)
         && !req.replace
     {
         return Err(format!(
-            "agent {:?} already has a card (project {:?}, key {}); changing its key or project needs replace: true",
+            "agent {:?} already has a card (project {:?}, key {}); changing any of its fields needs replace: true",
             req.agent, o.project, o.signing_key
         ));
     }
+    if let Some(other) = store.cards.values().find(|c| c.agent != req.agent && c.signing_key == req.signing_key) {
+        return Err(format!("signing key {} is already on agent {:?}'s card; a key belongs to one agent", req.signing_key, other.agent));
+    }
     let card = Card {
         schema: CARD_SCHEMA.into(),
         agent: req.agent.clone(),
@@ -282,9 +351,11 @@ pub fn put(dir: &Path, req: &PutRequest, now_ms: u64) -> Result<(Card, Option<Ca
         encryption_key: req.encryption_key.clone(),
         instances: req.instances.clone(),
         seq: old.as_ref().map_or(1, |o| o.seq + 1),
-        by: "runme".into(),
+        // what wrote it; the peer is not authenticated beyond the local socket (operator ruling A)
+        by: "hub.key_card_put (local socket)".into(),
         at_ms: now_ms,
     };
+    log(&card, old.as_ref())?;
     store.schema = STORE_SCHEMA.into();
     store.cards.insert(card.agent.clone(), card.clone());
     save(dir, &store).map_err(|e| format!("key-card store not written: {e}"))?;
@@ -306,55 +377,80 @@ pub fn list(dir: &Path) -> Result<Vec<Card>, String> {
 /// card written outside `hub.key_card_put`) or an unreadable store is reported; the caller raises
 /// attention. Known limit: an admin log pruned past a card's last put reports that card too.
 pub fn startup_reconcile(dir: &Path, admin_log: &Path) -> Vec<String> {
-    let cards = match list(dir) {
-        Ok(c) => c,
-        Err(e) => return vec![e],
-    };
-    if cards.is_empty() {
-        return vec![];
-    }
-    let mut logged: BTreeMap<String, u64> = BTreeMap::new();
+    let mut out = Vec::new();
+    // the last logged card digest per agent, and the last logged setting
+    let mut logged: BTreeMap<String, String> = BTreeMap::new();
+    let mut last_mode: Option<String> = None;
     if let Ok(text) = std::fs::read_to_string(admin_log) {
         for v in text.lines().filter_map(|l| serde_json::from_str::<serde_json::Value>(l).ok()) {
-            if v.get("event").and_then(|e| e.as_str()) == Some("key-card-put")
-                && let (Some(a), Some(s)) = (v.get("agent").and_then(|a| a.as_str()), v.get("seq").and_then(|s| s.as_u64()))
-            {
-                let e = logged.entry(a.to_string()).or_insert(0);
-                *e = (*e).max(s);
+            match v.get("event").and_then(|e| e.as_str()) {
+                Some("key-card-put") => {
+                    if let (Some(a), Some(d)) = (v.get("agent").and_then(|a| a.as_str()), v.get("card_digest").and_then(|s| s.as_str())) {
+                        logged.insert(a.to_string(), d.to_string());
+                    }
+                }
+                Some("key-card-mode") => last_mode = v.get("to").and_then(|m| m.as_str()).map(str::to_string),
+                _ => {}
             }
         }
     }
-    cards
-        .iter()
-        .filter(|c| logged.get(&c.agent).copied().unwrap_or(0) != c.seq)
-        .map(|c| {
-            format!(
-                "card {} has seq {} but the admin log's last key-card-put for it shows {}",
-                c.agent,
-                c.seq,
-                logged.get(&c.agent).map_or("none".to_string(), |s| s.to_string())
-            )
-        })
-        .collect()
+    // the setting: deleting or editing the file is not an admin act (r2 MINOR 10)
+    let now = read_mode(dir).map(|m| m.as_str().to_string()).unwrap_or_else(|e| format!("invalid ({e})"));
+    match &last_mode {
+        Some(m) if *m != now => out.push(format!("key-card check setting is {now} but the admin log's last change set {m}")),
+        None if now != "off" => out.push(format!("key-card check setting is {now} with no logged change")),
+        _ => {}
+    }
+    let _g = STORE_LOCK.lock().unwrap_or_else(|p| p.into_inner());
+    match load_raw(dir) {
+        Err(e) => out.push(e),
+        Ok((store, digest)) => {
+            for c in store.cards.values() {
+                let d = card_digest(c);
+                match logged.get(&c.agent) {
+                    Some(l) if *l == d => {}
+                    Some(_) => out.push(format!("card {} (seq {}) differs from its last logged key-card-put", c.agent, c.seq)),
+                    None => out.push(format!("card {} (seq {}) has no logged key-card-put", c.agent, c.seq)),
+                }
+            }
+            set_known(dir, digest);
+        }
+    }
+    out
 }
 
-/// Refusals per claimed agent since hub start (the impersonated agent's sidecar reads these,
-/// design v2 C-12), and would-be refusals in `log` mode.
-static REFUSALS: Mutex<BTreeMap<String, (u64, u64)>> = Mutex::new(BTreeMap::new());
+/// The bucket for refusals naming an agent without a card (bounded: attacker-chosen names).
+pub const UNCARDED: &str = "_uncarded";
+const MAX_SIGNERS: usize = 16;
 
-/// Count one refusal (`enforced`) or one log-mode would-refuse for `agent`.
-pub fn note_refusal(agent: &str, enforced: bool) {
+#[derive(Default)]
+struct Count {
+    refused: u64,
+    would: u64,
+    signers: BTreeMap<String, u64>,
+}
+
+/// Refusals per claimed carded agent since hub start (the impersonated agent's sidecar reads these,
+/// design v2 C-12), with the signer fingerprints (at most 16 named, the rest "other"), so an agent
+/// does not report its own refused posts as someone else's.
+static REFUSALS: Mutex<BTreeMap<String, Count>> = Mutex::new(BTreeMap::new());
+
+/// Count one refusal (`enforced`) or one log-mode would-refuse.
+pub fn note_refusal(agent: &str, signer_fp: &str, enforced: bool, carded: bool) {
+    let key = if carded { agent } else { UNCARDED };
     let mut m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
-    let e = m.entry(agent.to_string()).or_insert((0, 0));
-    if enforced { e.0 += 1 } else { e.1 += 1 }
+    let e = m.entry(key.to_string()).or_default();
+    if enforced { e.refused += 1 } else { e.would += 1 }
+    let s = if e.signers.contains_key(signer_fp) || e.signers.len() < MAX_SIGNERS { signer_fp } else { "other" };
+    *e.signers.entry(s.to_string()).or_insert(0) += 1;
 }
 
-/// `{agent: {refused, would_refuse}}`.
+/// `{agent: {refused, would_refuse, signers: {fp: n}}}`.
 pub fn refusal_counts() -> serde_json::Value {
     let m = REFUSALS.lock().unwrap_or_else(|p| p.into_inner());
     serde_json::Value::Object(
         m.iter()
-            .map(|(a, (r, w))| (a.clone(), serde_json::json!({"refused": r, "would_refuse": w})))
+            .map(|(a, c)| (a.clone(), serde_json::json!({"refused": c.refused, "would_refuse": c.would, "signers": c.signers})))
             .collect(),
     )
 }
@@ -362,6 +458,8 @@ pub fn refusal_counts() -> serde_json::Value {
 /// Admin rows for refusals are coalesced per (claimed agent, poster key, minute) so a token
 /// holder cannot flood the admin log (design v2, C-12). True = write a row for this one.
 static LOGGED: Mutex<(u64, Option<HashSet<(String, String)>>)> = Mutex::new((0, None));
+/// At most this many refusal rows per minute in all (r2 MAJOR 7: varied names cannot flood).
+pub const MAX_ROWS_PER_MINUTE: usize = 60;
 
 pub fn first_in_minute(agent: &str, signer_fp: &str, now_ms: u64) -> bool {
     let minute = now_ms / 60_000;
@@ -369,7 +467,11 @@ pub fn first_in_minute(agent: &str, signer_fp: &str, now_ms: u64) -> bool {
     if g.0 != minute || g.1.is_none() {
         *g = (minute, Some(HashSet::new()));
     }
-    g.1.as_mut().map(|s| s.insert((agent.to_string(), signer_fp.to_string()))).unwrap_or(true)
+    match g.1.as_mut() {
+        Some(s) if s.len() >= MAX_ROWS_PER_MINUTE => false,
+        Some(s) => s.insert((agent.to_string(), signer_fp.to_string())),
+        None => true,
+    }
 }
 
 #[cfg(test)]
@@ -428,8 +530,8 @@ mod tests {
     #[test]
     fn e69_fcw133_signed_by_a_sibling_key_is_refused_own_key_passes() {
         let d = tempfile::tempdir().unwrap();
-        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
-        put(d.path(), &put_req("fx-b", "p", 2), 1).unwrap();
+        put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
+        put(d.path(), &put_req("fx-b", "p", 2), 1, |_, _| Ok(())).unwrap();
         // fx-b's key claiming to be fx-a
         match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[2; 32]) {
             Check::Refuse { code, fcw, .. } => assert_eq!((code, fcw), (error_code::CHANNEL_KEY_CARD_MISMATCH, "FCW-133")),
@@ -444,8 +546,8 @@ mod tests {
     #[test]
     fn r75e_another_projects_agent_cannot_post_as_ps_agent() {
         let d = tempfile::tempdir().unwrap();
-        put(d.path(), &put_req("p-agent", "p", 1), 1).unwrap();
-        put(d.path(), &put_req("q-agent", "q", 2), 1).unwrap();
+        put(d.path(), &put_req("p-agent", "p", 1), 1, |_, _| Ok(())).unwrap();
+        put(d.path(), &put_req("q-agent", "q", 2), 1, |_, _| Ok(())).unwrap();
         // q's session, signing with its own key, asserting it is p's agent
         assert!(matches!(
             check_post(d.path(), "inbox:h/x", "sidecar.stage.v1", &msg_payload("p-agent"), &[2; 32]),
@@ -484,24 +586,24 @@ mod tests {
             Check::Refuse { fcw: "FCW-135", reason, .. } => assert!(reason.contains("corrupt"), "{reason}"),
             o => panic!("{o:?}"),
         }
-        assert!(put(d.path(), &put_req("fx-a", "p", 1), 1).is_err(), "a put never overwrites a corrupt store");
+        assert!(put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).is_err(), "a put never overwrites a corrupt store");
     }
 
     #[test]
     fn a_key_or_project_change_needs_replace_and_seq_rises() {
         let d = tempfile::tempdir().unwrap();
-        let (c1, old) = put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
+        let (c1, old) = put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
         assert_eq!((c1.seq, old), (1, None));
         // same key and project: an idempotent re-put moves seq, no replace needed
-        let (c2, _) = put(d.path(), &put_req("fx-a", "p", 1), 2).unwrap();
+        let (c2, _) = put(d.path(), &put_req("fx-a", "p", 1), 2, |_, _| Ok(())).unwrap();
         assert_eq!(c2.seq, 2);
-        let e = put(d.path(), &put_req("fx-a", "p", 9), 3).unwrap_err();
+        let e = put(d.path(), &put_req("fx-a", "p", 9), 3, |_, _| Ok(())).unwrap_err();
         assert!(e.contains("replace: true"), "{e}");
-        let e = put(d.path(), &put_req("fx-a", "q", 1), 3).unwrap_err();
+        let e = put(d.path(), &put_req("fx-a", "q", 1), 3, |_, _| Ok(())).unwrap_err();
         assert!(e.contains("replace: true"), "{e}");
         let mut r = put_req("fx-a", "p", 9);
         r.replace = true;
-        let (c3, old) = put(d.path(), &r, 4).unwrap();
+        let (c3, old) = put(d.path(), &r, 4, |_, _| Ok(())).unwrap();
         assert_eq!(c3.seq, 3);
         assert_eq!(old.unwrap().signing_key, key_hex(1));
     }
@@ -510,30 +612,109 @@ mod tests {
     fn the_store_is_0600_and_keys_are_validated() {
         use std::os::unix::fs::PermissionsExt;
         let d = tempfile::tempdir().unwrap();
-        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
+        put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
         assert_eq!(std::fs::metadata(store_path(d.path())).unwrap().permissions().mode() & 0o777, 0o600);
         let mut r = put_req("fx-b", "p", 1);
         r.signing_key = "ed25519:".to_string() + &"01".repeat(32);
-        assert!(put(d.path(), &r, 1).is_err(), "the fingerprint form is not a card key");
+        assert!(put(d.path(), &r, 1, |_, _| Ok(())).is_err(), "the fingerprint form is not a card key");
         r.signing_key = key_hex(1).to_uppercase();
-        assert!(put(d.path(), &r, 1).is_err());
+        assert!(put(d.path(), &r, 1, |_, _| Ok(())).is_err());
     }
 
     #[test]
-    fn al33_startup_reconcile_flags_a_card_ahead_of_the_admin_log() {
+    fn al33_startup_reconcile_flags_a_card_the_admin_log_does_not_show() {
         let d = tempfile::tempdir().unwrap();
         let log = d.path().join("rpc-admin.jsonl");
-        put(d.path(), &put_req("fx-a", "p", 1), 1).unwrap();
-        put(d.path(), &put_req("fx-b", "p", 2), 1).unwrap();
-        // fx-a's put was logged; fx-b's crashed between persist and log
-        std::fs::write(&log, "{\"event\":\"key-card-put\",\"agent\":\"fx-a\",\"seq\":1}\n").unwrap();
+        let (a, _) = put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
+        put(d.path(), &put_req("fx-b", "p", 2), 1, |_, _| Ok(())).unwrap();
+        // fx-a's put was logged; fx-b's was not (crash between persist and log, or an outside write)
+        std::fs::write(&log, format!("{{\"event\":\"key-card-put\",\"agent\":\"fx-a\",\"card_digest\":\"{}\"}}\n", card_digest(&a))).unwrap();
         let m = startup_reconcile(d.path(), &log);
         assert_eq!(m.len(), 1, "{m:?}");
-        assert!(m[0].contains("fx-b") && m[0].contains("none"), "{m:?}");
-        std::fs::write(&log, "{\"event\":\"key-card-put\",\"agent\":\"fx-a\",\"seq\":1}\n{\"event\":\"key-card-put\",\"agent\":\"fx-b\",\"seq\":1}\n").unwrap();
-        assert!(startup_reconcile(d.path(), &log).is_empty());
+        assert!(m[0].contains("fx-b") && m[0].contains("no logged"), "{m:?}");
         std::fs::write(store_path(d.path()), "{x").unwrap();
-        assert_eq!(startup_reconcile(d.path(), &log).len(), 1, "an unreadable store is reported");
+        assert!(startup_reconcile(d.path(), &log).iter().any(|s| s.contains("corrupt")), "an unreadable store is reported");
+    }
+
+    #[test]
+    fn r2_an_outside_edit_with_the_same_seq_is_refused_until_restart() {
+        let d = tempfile::tempdir().unwrap();
+        put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Ok(())).unwrap();
+        // a root agent swaps fx-a's key in the file, seq unchanged
+        let text = std::fs::read_to_string(store_path(d.path())).unwrap().replace(&"01".repeat(32), &"07".repeat(32));
+        std::fs::write(store_path(d.path()), text).unwrap();
+        match check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[7; 32]) {
+            Check::Refuse { fcw: "FCW-135", reason, .. } => assert!(reason.contains("changed outside"), "{reason}"),
+            o => panic!("the edited key must not pass: {o:?}"),
+        }
+        assert!(put(d.path(), &put_req("fx-b", "p", 2), 2, |_, _| Ok(())).is_err(), "no put over an outside edit");
+        // a restart re-baselines and reports it
+        let log = d.path().join("rpc-admin.jsonl");
+        assert!(!startup_reconcile(d.path(), &log).is_empty());
+        assert!(matches!(check_post(d.path(), "inbox:h/x", "sidecar.msg.v1", &msg_payload("fx-a"), &[7; 32]), Check::Pass { .. }));
+    }
+
+    #[test]
+    fn r2_the_admin_row_is_written_first_and_a_failed_row_writes_nothing() {
+        let d = tempfile::tempdir().unwrap();
+        let e = put(d.path(), &put_req("fx-a", "p", 1), 1, |_, _| Err("admin log unwritable".into())).unwrap_err();
+        assert!(e.contains("unwritable"), "{e}");
+        assert!(!store_path(d.path()).exists(), "nothing persisted when the row fails");
+        let mut seen = None;
+        put(d.path(), &put_req("fx-a", "p", 1), 1, |new, old| {
+            assert!(!store_path(d.path()).exists(), "the row is written before the store");
+            seen = Some((new.seq, old.is_none()));
+            Ok(())
+        })
+        .unwrap();
+        assert_eq!(seen, Some((1, true)));
+    }
+
+    #[test]
+    fn r2_every_field_change_needs_replace_and_a_key_has_one_card() {
+        let d = tempfile::tempdir().unwrap();
+        let mut r = put_req("fx-a", "p", 1);
+        r.encryption_key = Some(format!("{ENCRYPTION_PREFIX}{}", "02".repeat(32)));
+        put(d.path(), &r, 1, |_, _| Ok(())).unwrap();
+        let mut swap = r.clone();
+        swap.encryption_key = Some(format!("{ENCRYPTION_PREFIX}{}", "03".repeat(32)));
+        assert!(put(d.path(), &swap, 2, |_, _| Ok(())).unwrap_err().contains("replace: true"));
+        let mut drop_enc = r.clone();
+        drop_enc.encryption_key = None;
+        assert!(put(d.path(), &drop_enc, 2, |_, _| Ok(())).unwrap_err().contains("replace: true"));
+        let mut inst = r.clone();
+        inst.instances = vec!["i1".into()];
+        assert!(put(d.path(), &inst, 2, |_, _| Ok(())).unwrap_err().contains("replace: true"));
+        // fx-a's key on fx-b's card is refused
+        assert!(put(d.path(), &put_req("fx-b", "p", 1), 2, |_, _| Ok(())).unwrap_err().contains("one agent"));
+    }
+
+    #[test]
+    fn r2_mode_changes_are_logged_first_and_reconciled_at_start() {
+        let d = tempfile::tempdir().unwrap();
+        let log = d.path().join("rpc-admin.jsonl");
+        assert!(set_mode(d.path(), Mode::Enforce, |_, _| Err("no row".into())).is_err());
+        assert_eq!(read_mode(d.path()), Ok(Mode::Off), "a failed row changes nothing");
+        assert_eq!(set_mode(d.path(), Mode::Enforce, |_, _| Ok(())).unwrap(), "off");
+        std::fs::write(&log, "{\"event\":\"key-card-mode\",\"to\":\"enforce\"}\n").unwrap();
+        assert!(startup_reconcile(d.path(), &log).is_empty());
+        std::fs::remove_file(d.path().join(MODE_FILE)).unwrap(); // the file deleted outside the RPC
+        let m = startup_reconcile(d.path(), &log);
+        assert!(m.iter().any(|s| s.contains("setting is off") && s.contains("enforce")), "{m:?}");
+    }
+
+    #[test]
+    fn r2_refusals_for_uncarded_names_share_one_bucket_and_rows_are_capped() {
+        note_refusal("t-r2-x1", "fpz", true, false);
+        note_refusal("t-r2-x2", "fpz", true, false);
+        note_refusal("t-r2-carded", "fpz", true, true);
+        let c = refusal_counts();
+        assert!(c.get("t-r2-x1").is_none() && c.get("t-r2-x2").is_none(), "{c}");
+        assert!(c[UNCARDED]["refused"].as_u64().unwrap() >= 2);
+        assert_eq!(c["t-r2-carded"]["signers"]["fpz"], 1);
+        let base = 600_000_000u64;
+        let n = (0..200).filter(|i| first_in_minute(&format!("cap-{i}"), "fp", base)).count();
+        assert_eq!(n, MAX_ROWS_PER_MINUTE);
     }
 
     #[test]
diff --git a/crates/termlink-hub/src/router.rs b/crates/termlink-hub/src/router.rs
index 0e8122c2e..86d8da887 100644
--- a/crates/termlink-hub/src/router.rs
+++ b/crates/termlink-hub/src/router.rs
@@ -932,52 +932,52 @@ fn handle_key_card_put(id: serde_json::Value, params: &serde_json::Value, peer_a
         .duration_since(std::time::UNIX_EPOCH)
         .map(|d| d.as_millis() as u64)
         .unwrap_or(0);
-    match crate::key_card::put(&dir, &req, now) {
-        Ok((card, old)) => {
-            // Every put raises attention (operator ruling A: visible, not impossible); the row
-            // names old and new key so a silent replacement cannot happen.
-            let row = serde_json::json!({
-                "event": "key-card-put", "attention": true, "agent": card.agent, "project": card.project,
-                "seq": card.seq, "signing_key": card.signing_key,
-                "old_signing_key": old.as_ref().map(|o| o.signing_key.clone()),
-                "old_project": old.as_ref().map(|o| o.project.clone()), "replace": req.replace,
-            });
-            if let Err(e) = crate::rpc_audit::admin_note(row) {
-                tracing::error!(error = %e, "T-3704: key-card put admin row not written");
-            }
-            Response::success(id, serde_json::json!({"ok": true, "card": card, "replaced": old})).into()
-        }
+    // The row naming old and new values is written BEFORE the store; if it cannot be written the
+    // put is refused and nothing changes (operator ruling A; review r2 BLOCKER). Every sidecar
+    // raises the operator's needs-attention entry from the card list (seq/digest change).
+    let replace = req.replace;
+    let log = |card: &crate::key_card::Card, old: Option<&crate::key_card::Card>| {
+        let row = serde_json::json!({
+            "event": "key-card-put", "attention": true, "agent": card.agent, "project": card.project,
+            "seq": card.seq, "signing_key": card.signing_key, "encryption_key": card.encryption_key,
+            "instances": card.instances, "card_digest": crate::key_card::card_digest(card), "replace": replace,
+            "old": old.map(|o| serde_json::json!({"project": o.project, "signing_key": o.signing_key,
+                "encryption_key": o.encryption_key, "instances": o.instances, "seq": o.seq})),
+        });
+        crate::rpc_audit::admin_note(row).map_err(|e| format!("admin-log row not written ({e}); nothing changed"))
+    };
+    match crate::key_card::put(&dir, &req, now, log) {
+        Ok((card, old)) => Response::success(id, serde_json::json!({"ok": true, "card": card, "replaced": old})).into(),
         Err(e) => key_card_refused(id, &e),
     }
 }
 
 fn handle_key_card_mode(id: serde_json::Value, params: &serde_json::Value, peer_addr: Option<&str>) -> RpcResponse {
-    let dir = termlink_session::discovery::runtime_dir();
-    let Some(want) = params.get("mode").and_then(|v| v.as_str()) else {
-        let cur = crate::key_card::read_mode(&dir);
-        return Response::success(
-            id,
-            serde_json::json!({"mode": cur.as_ref().map(|m| m.as_str()).unwrap_or("enforce"), "error": cur.err()}),
-        )
-        .into();
-    };
     if let Some(r) = key_card_local_only(&id, "hub.key_card_mode", peer_addr) {
         return r;
     }
+    let dir = termlink_session::discovery::runtime_dir();
+    // reading the setting is hub.key_card_list (observe); this RPC only changes it
+    let Some(want) = params.get("mode").and_then(|v| v.as_str()) else {
+        return key_card_refused(id, "missing 'mode' (off|log|enforce); read it with hub.key_card_list");
+    };
     let Some(mode) = crate::key_card::Mode::parse(want) else {
         return key_card_refused(id, &format!("mode {want:?} is not off|log|enforce"));
     };
-    let before = crate::key_card::read_mode(&dir).map(|m| m.as_str()).unwrap_or("invalid");
-    if let Err(e) = crate::key_card::write_mode(&dir, mode) {
-        return key_card_refused(id, &format!("setting not written: {e}"));
-    }
-    let row = serde_json::json!({"event": "key-card-mode", "attention": true, "from": before, "to": mode.as_str()});
-    if let Err(e) = crate::rpc_audit::admin_note(row) {
-        tracing::error!(error = %e, "T-3704: key-card mode admin row not written");
+    let log = |from: &str, to: &str| {
+        let row = serde_json::json!({"event": "key-card-mode", "attention": true, "from": from, "to": to});
+        crate::rpc_audit::admin_note(row).map_err(|e| format!("admin-log row not written ({e}); nothing changed"))
+    };
+    match crate::key_card::set_mode(&dir, mode, log) {
+        Ok(before) => Response::success(id, serde_json::json!({"ok": true, "mode": mode.as_str(), "was": before})).into(),
+        Err(e) => key_card_refused(id, &e),
     }
-    Response::success(id, serde_json::json!({"ok": true, "mode": mode.as_str(), "was": before})).into()
 }
 
+static HUB_START_MS: std::sync::LazyLock<u64> = std::sync::LazyLock::new(|| {
+    std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).map(|d| d.as_millis() as u64).unwrap_or(0)
+});
+
 fn handle_key_card_get(id: serde_json::Value, params: &serde_json::Value) -> RpcResponse {
     let Some(agent) = params.get("agent").and_then(|v| v.as_str()) else {
         return key_card_refused(id, "missing 'agent'");
@@ -999,6 +999,8 @@ fn handle_key_card_list(id: serde_json::Value) -> RpcResponse {
                 "mode_error": mode.err(),
                 "cards": cards,
                 "refusals": crate::key_card::refusal_counts(),
+                // the sidecar keys its "seen" counts by this, so a restart's reset counts are not lost
+                "hub_start_ms": *HUB_START_MS,
             }),
         )
         .into(),
diff --git a/crates/termlink-sidecar/src/daemon.rs b/crates/termlink-sidecar/src/daemon.rs
index 1d4b8e3dd..5ef419a09 100644
--- a/crates/termlink-sidecar/src/daemon.rs
+++ b/crates/termlink-sidecar/src/daemon.rs
@@ -109,6 +109,7 @@ impl Drop for Running {
 
 fn roster_of(keys: &[String]) -> Result<Roster, String> {
     let mut r = Roster::new();
+    let mut seen = std::collections::HashSet::new();
     for entry in keys {
         // T-3704 (B-6): `agent=<hex>` binds the key to that agent name; bare `<hex>` stays unbound.
         let (name, k) = match entry.split_once('=') {
@@ -128,6 +129,7 @@ fn roster_of(keys: &[String]) -> Result<Roster, String> {
         let arr: [u8; 32] = bytes.try_into().map_err(|_| format!("roster key is not 32 bytes: {k}"))?;
         let pk = VerifyingKey::from_bytes(&arr).map_err(|e| format!("roster key: {e}"))?;
         match name {
+            Some(n) if !seen.insert(n.to_string()) => return Err(format!("roster names agent {n:?} twice")),
             Some(n) => r.bind(n, pk),
             None => r.add(pk),
         };
@@ -478,3 +480,19 @@ pub fn main_start(config: &Path) -> i32 {
     println!("termlink-sidecar: {} stopped", cfg.agent);
     0
 }
+
+#[cfg(test)]
+mod t3704_roster_tests {
+    use super::roster_of;
+
+    #[test]
+    fn named_entries_bind_and_a_name_given_twice_is_refused() {
+        let k1 = "11".repeat(32);
+        let k2 = "22".repeat(32);
+        let r = roster_of(&[format!("fx-a={k1}"), k2.clone()]).unwrap();
+        assert_eq!(r.bound_count(), 1, "one named, one legacy entry");
+        let e = roster_of(&[format!("fx-a={k1}"), format!("fx-a={k2}")]).unwrap_err();
+        assert!(e.contains("twice"), "{e}");
+        assert!(roster_of(&[format!("../x={k1}")]).is_err(), "a non-plain name is refused");
+    }
+}
diff --git a/crates/termlink-sidecar/src/pull.rs b/crates/termlink-sidecar/src/pull.rs
index fca74de63..411440d85 100644
--- a/crates/termlink-sidecar/src/pull.rs
+++ b/crates/termlink-sidecar/src/pull.rs
@@ -262,34 +262,47 @@ impl Inner {
     fn observe_impersonation(&self) {
         let Ok(resp) = self.rpc("hub.key_card_list", json!({})) else { return };
         let Ok(v) = unwrap_result(resp) else { return };
-        let r = &v["refusals"][self.cfg.agent.as_str()];
-        let (refused, would) = (r["refused"].as_u64().unwrap_or(0), r["would_refuse"].as_u64().unwrap_or(0));
-        let total = refused + would;
-        let seen: u64 = self
-            .store
-            .read_state(IMPERSONATION_STATE)
-            .ok()
-            .flatten()
-            .and_then(|s| s.trim().parse().ok())
-            .unwrap_or(0);
-        if total == seen {
-            return;
-        }
-        let _ = self.store.write_state(IMPERSONATION_STATE, &total.to_string());
-        if total < seen {
-            return; // the hub restarted and its counts began again
-        }
         let day = std::time::SystemTime::now()
             .duration_since(std::time::UNIX_EPOCH)
             .map(|d| d.as_secs() / 86_400)
             .unwrap_or(0);
-        let detail = format!(
-            "posts claiming to be agent {} were refused at the hub by its key card: {refused} refused, \
-             {would} would be refused (log mode); see `termlink hub key-card list` and rpc-admin.jsonl",
-            self.cfg.agent
-        );
-        if let Err(e) = self.store.attention_coalesced("posts-claiming-to-be-you-refused", &day.to_string(), &detail) {
-            eprintln!("termlink-sidecar: needs attention (posts-claiming-to-be-you-refused): {detail} [could not record it: {e}]");
+        let start = v["hub_start_ms"].as_u64().unwrap_or(0);
+        // (1) refusals of posts claiming to be this agent, NOT signed by this agent's own key
+        //     (its own posts refused during the P-31 roll-out are not an impersonation, r2 MINOR 11)
+        let me = self.sender.fingerprint().to_string();
+        let r = &v["refusals"][self.cfg.agent.as_str()];
+        let total: u64 = r["signers"]
+            .as_object()
+            .map(|m| m.iter().filter(|(fp, _)| **fp != me).filter_map(|(_, n)| n.as_u64()).sum())
+            .unwrap_or(0);
+        let now = format!("{start}:{total}");
+        let seen = self.store.read_state(IMPERSONATION_STATE).ok().flatten().unwrap_or_default();
+        if now != seen.trim() {
+            let _ = self.store.write_state(IMPERSONATION_STATE, &now);
+            let (seen_start, seen_total) = seen
+                .trim()
+                .split_once(':')
+                .map(|(a, b)| (a.parse::<u64>().unwrap_or(0), b.parse::<u64>().unwrap_or(0)))
+                .unwrap_or((0, 0));
+            // a new hub start resets its counts: compare against 0, so none are lost
+            let base = if seen_start == start { seen_total } else { 0 };
+            if total > base {
+                let detail = format!(
+                    "posts claiming to be agent {} were refused at the hub by its key card ({} since the hub started, signed by keys other than this agent's); see `termlink hub key-card list` and rpc-admin.jsonl",
+                    self.cfg.agent, total
+                );
+                if let Err(e) = self.store.attention_coalesced("posts-claiming-to-be-you-refused", &day.to_string(), &detail) {
+                    eprintln!("termlink-sidecar: needs attention (posts-claiming-to-be-you-refused): {detail} [could not record it: {e}]");
+                }
+            }
+        }
+        // (2) every card write is the operator's to know (ruling A): one entry per (agent, seq)
+        for c in v["cards"].as_array().into_iter().flatten() {
+            let (Some(agent), Some(seq), Some(key)) = (c["agent"].as_str(), c["seq"].as_u64(), c["signing_key"].as_str()) else {
+                continue;
+            };
+            let detail = format!("key card for agent {agent} is at seq {seq} (key {key}); written via hub.key_card_put, see rpc-admin.jsonl for old and new values");
+            let _ = self.store.attention_coalesced("key-card-written", &format!("{agent}:{seq}"), &detail);
         }
     }
 
diff --git a/crates/termlink-sidecar/src/wire.rs b/crates/termlink-sidecar/src/wire.rs
index 3335b9c31..d18bb6797 100644
--- a/crates/termlink-sidecar/src/wire.rs
+++ b/crates/termlink-sidecar/src/wire.rs
@@ -312,6 +312,11 @@ impl Roster {
     /// T-3704 (FCW-98): `from_agent` must be a plain name; if it is bound, the signer must be its
     /// key; and a key bound to some agent may not speak as another one.
     pub fn check_agent(&self, from_agent: &str, pk: &VerifyingKey) -> Result<(), Refusal> {
+        // No named entry: the PH-1 behaviour, nothing checked (r2 MAJOR 4: an upgraded receiver
+        // refuses nothing new until its roster is configured with names).
+        if self.1.is_empty() {
+            return Ok(());
+        }
         if !plain_agent_name(from_agent) {
             return Err(Refusal::Malformed("from_agent is not a plain agent name"));
         }
@@ -320,7 +325,9 @@ impl Roster {
         {
             return Err(Refusal::AgentKeyMismatch);
         }
-        if self.1.iter().any(|(name, k)| k.as_bytes() == pk.as_bytes() && name != from_agent) {
+        // a bound key may speak only as a name it is bound to (aliases allowed, r2 MINOR 9)
+        let bound: Vec<&String> = self.1.iter().filter(|(_, k)| k.as_bytes() == pk.as_bytes()).map(|(n, _)| n).collect();
+        if !bound.is_empty() && !bound.iter().any(|n| n.as_str() == from_agent) {
             return Err(Refusal::AgentKeyMismatch);
         }
         Ok(())
@@ -846,6 +853,13 @@ mod wire_tests {
         let eve = ident(13);
         let s = sign(&eve, "inbox:b", 42, &msg_from(&eve, "inst-b"));
         assert_eq!(verify_open(&s, &r, "inst-b").map(|_| ()), Err(Refusal::AgentKeyMismatch));
+        // aliases: one key bound to two names may speak as either
+        let mut ra = Roster::new();
+        ra.bind("alice", alice.verifying_key());
+        ra.bind("alice2", alice.verifying_key());
+        let mut m = msg_from(&alice, "inst-b");
+        m.from_agent = "alice2".into();
+        assert!(verify(&sign(&alice, "inbox:b", 42, &m), &ra, "inst-b").is_ok());
         // a traversal-style name is refused before any lookup
         let mut m = msg_from(&alice, "inst-b");
         m.from_agent = "../x".into();
```

## Test changes since round 2
```diff
diff --git a/crates/termlink-cli/tests/t3704_key_card.rs b/crates/termlink-cli/tests/t3704_key_card.rs
index 59f3156c2..56ef5affb 100644
--- a/crates/termlink-cli/tests/t3704_key_card.rs
+++ b/crates/termlink-cli/tests/t3704_key_card.rs
@@ -118,6 +118,24 @@ fn e69_posts_are_checked_against_the_card_and_e169_no_card_is_refused() {
     );
     assert!(admin.contains("\"key-card-refused\""), "refusals are admin-logged:\n{admin}");
 
+    // review r2 BLOCKER: when the admin row cannot be written the put is refused and the card
+    // is unchanged (the log is made unwritable by putting a directory in its place)
+    let admin_path = dir.path.join("rpc-admin.jsonl");
+    let saved = std::fs::read(&admin_path).unwrap();
+    std::fs::remove_file(&admin_path).unwrap();
+    std::fs::create_dir(&admin_path).unwrap();
+    let r = put(&dir, "fx-a", "proj-p", &a, true);
+    assert!(!r.status.success(), "a put whose admin row cannot be written is refused");
+    std::fs::remove_dir(&admin_path).unwrap();
+    std::fs::write(&admin_path, saved).unwrap();
+    let got = rpc(&sock, "hub.key_card_get", serde_json::json!({"agent": "fx-a"}));
+    assert_eq!(got["result"]["card"]["signing_key"], format!("ed25519-pub:{}", c.public_key_hex()), "card unchanged: {got}");
+    // another project's agent cannot post as fx-a
+    let q = Identity::generate();
+    assert!(put(&dir, "q-agent", "proj-q", &q, false).status.success());
+    assert!(cmd(&dir, &["hub", "key-card", "mode", "enforce"]).status().unwrap().success());
+    assert_eq!(code(&post(&sock, &q, "fx-a", "sidecar.msg.v1")), Some(-32044), "project Q's key claiming P's agent");
+
     // per-agent refusal counts for the impersonated agent
     let list = rpc(&sock, "hub.key_card_list", serde_json::json!({}));
     let refused = list["result"]["refusals"]["fx-a"]["refused"].as_u64().unwrap_or(0);
diff --git a/crates/termlink-sidecar/tests/t3704_impersonation.rs b/crates/termlink-sidecar/tests/t3704_impersonation.rs
index e47c9e5d9..ed8f0e4f2 100644
--- a/crates/termlink-sidecar/tests/t3704_impersonation.rs
+++ b/crates/termlink-sidecar/tests/t3704_impersonation.rs
@@ -94,12 +94,155 @@ fn a_refused_post_claiming_to_be_me_raises_one_attention_entry() {
     );
     assert_eq!(refused["error"]["code"], -32043, "{refused}");
 
+    // fx-a has no card yet, so the forgery counts in the shared uncarded bucket, not as fx-a:
+    // give fx-a a card, then forge again with another key (a carded-agent mismatch)
+    let put = rpc(&e, "hub.key_card_put", json!({"agent": "fx-a", "project": "proj-p",
+        "signing_key": format!("ed25519-pub:{}", e.identity("a").unwrap().public_key_hex())}));
+    assert_eq!(put["result"]["ok"], true, "{put}");
+    let sig = forger.sign_hex(&canonical_sign_bytes(&topic, "sidecar.msg.v1", payload.as_bytes(), None, ts + 1));
+    let refused = rpc(
+        &e,
+        "channel.post",
+        json!({"topic": topic, "msg_type": "sidecar.msg.v1",
+               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
+               "ts": ts + 1, "sender_id": forger.fingerprint(), "sender_pubkey_hex": forger.public_key_hex(),
+               "signature_hex": sig}),
+    );
+    assert_eq!(refused["error"]["code"], -32044, "{refused}");
+
     let _ = rx.pass();
     let hits: Vec<_> = kinds(&store).into_iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").collect();
-    assert_eq!(hits.len(), 1, "{hits:?}");
-    assert!(hits[0].1.contains("fx-a") && hits[0].1.contains("1 refused"), "{}", hits[0].1);
+    let list = rpc(&e, "hub.key_card_list", json!({}));
+    assert_eq!(hits.len(), 1, "{hits:?} list={list} all={:?}", kinds(&store));
+    assert!(hits[0].1.contains("fx-a") && hits[0].1.contains("(1 since"), "{}", hits[0].1);
+    // the card write itself is the operator's to see (ruling A)
+    assert!(kinds(&store).iter().any(|(k, d)| k == "key-card-written" && d.contains("fx-a") && d.contains("seq 1")));
 
     // the next pass with no new refusal adds nothing
     let _ = rx.pass();
     assert_eq!(kinds(&store).iter().filter(|(k, _)| k == "posts-claiming-to-be-you-refused").count(), 1);
 }
+
+/// Review r2 MAJOR 6: a genuine sidecar-built message and stage record pass the hub's check in
+/// enforce mode when the sender's card holds its key, and a sidecar with another key claiming that
+/// agent is refused. Proves the hub reads `from_agent` where the real wire puts it.
+#[test]
+fn real_sidecar_msg_and_stage_pass_enforce_with_their_card_and_an_impostor_does_not() {
+    let e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    std::fs::write(e.runtime_dir().join("key-card-check"), "enforce\n").unwrap();
+    let a_key = e.identity("a").unwrap();
+    let put = rpc(&e, "hub.key_card_put", json!({"agent": "fx-a", "project": "proj-p",
+        "signing_key": format!("ed25519-pub:{}", a_key.public_key_hex())}));
+    assert_eq!(put["result"]["ok"], true, "{put}");
+    let topic = format!("inbox:{id}/fx-b");
+    let _ = rpc(&e, "channel.create", json!({"name": topic}));
+    let mk = |who: &str, agent: &str| {
+        let store = Arc::new(Store::open(&e.store_root(), &format!("{agent}-{who}")).unwrap());
+        Sender::new(
+            SendConfig {
+                agent: agent.into(),
+                instance: format!("inst-{who}"),
+                identity: e.identity(who).unwrap(),
+                hub: e.hub_addr(),
+                declared_hub_id: id.clone(),
+                queue_path: e.root().join(format!("outbox-{who}.sqlite")),
+                timeout: Duration::from_secs(10),
+                pump_interval: None,
+            },
+            store,
+        )
+        .unwrap()
+    };
+    let count = |e: &Estate| -> usize {
+        let v = rpc(e, "channel.subscribe", json!({"topic": topic, "cursor": 0, "limit": 100}));
+        v["result"]["messages"].as_array().map(|a| a.len()).unwrap_or(0)
+    };
+    let before = count(&e);
+    let a = mk("a", "fx-a");
+    a.send("fx-b", "inst-b", Some("c1"), "real message", None).unwrap();
+    let st = termlink_sidecar::wire::Stage {
+        from: a.fingerprint().to_string(),
+        from_agent: "fx-a".into(),
+        to_agent: "fx-b".into(),
+        conv: "c1".into(),
+        msg_id: "00112233445566778899aabbccddeeff".into(),
+        stage: "STORED".into(),
+        ts_ms: 1_800_000_000_000,
+        chain_head: "0".repeat(64),
+    };
+    a.post_stage("fx-b", &st).unwrap();
+    let deadline = std::time::Instant::now() + Duration::from_secs(10);
+    while count(&e) < before + 2 && std::time::Instant::now() < deadline {
+        std::thread::sleep(Duration::from_millis(100));
+    }
+    assert_eq!(count(&e), before + 2, "the real msg and stage of fx-a were both stored in enforce");
+    // an impostor sidecar with its own key, configured as fx-a
+    let z = mk("z", "fx-a");
+    z.send("fx-b", "inst-b", Some("c2"), "impostor", None).unwrap();
+    std::thread::sleep(Duration::from_millis(500));
+    assert_eq!(count(&e), before + 2, "the impostor's post was refused");
+}
+
+/// Review r2 MAJOR 2/5: the hub never gates types other than the two arc-011 ones (NID), so the
+/// receiver must be the backstop: a record of any other type on the inbox, claiming any agent, is
+/// skipped with an entry, never handled as a message.
+#[test]
+fn an_unlisted_type_on_the_inbox_is_skipped_never_handled_as_mail() {
+    let e = Estate::create().unwrap();
+    let id = e.hub_id().unwrap();
+    let store = Arc::new(Store::open(&e.store_root(), "fx-a").unwrap());
+    let sender = Arc::new(
+        Sender::new(
+            SendConfig {
+                agent: "fx-a".into(),
+                instance: "inst-a".into(),
+                identity: e.identity("a").unwrap(),
+                hub: e.hub_addr(),
+                declared_hub_id: id.clone(),
+                queue_path: e.root().join("outbox-a.sqlite"),
+                timeout: Duration::from_secs(10),
+                pump_interval: None,
+            },
+            store.clone(),
+        )
+        .unwrap(),
+    );
+    let rx = Receiver::new(
+        ReceiveConfig {
+            agent: "fx-a".into(),
+            instance: "inst-a".into(),
+            hub: e.hub_addr(),
+            declared_hub_id: id.clone(),
+            roster: Roster::new(),
+            timeout: Duration::from_secs(10),
+            page: 50,
+            poll_interval: None,
+        },
+        store.clone(),
+        sender,
+        Box::new(BuiltIn::new(Roster::new())),
+    )
+    .unwrap();
+    let topic = format!("inbox:{id}/fx-a");
+    let _ = rpc(&e, "channel.create", json!({"name": topic}));
+    let forger = Identity::generate();
+    let payload = json!({"v": 1, "kind": "msg", "from_agent": "fx-b", "to_agent": "fx-a", "body": "forged under another type"}).to_string();
+    let ts = 1_800_000_000_000i64;
+    let sig = forger.sign_hex(&canonical_sign_bytes(&topic, "sidecar.receipt", payload.as_bytes(), None, ts));
+    let ok = rpc(
+        &e,
+        "channel.post",
+        json!({"topic": topic, "msg_type": "sidecar.receipt",
+               "payload_b64": base64::engine::general_purpose::STANDARD.encode(payload.as_bytes()),
+               "ts": ts, "sender_id": forger.fingerprint(), "sender_pubkey_hex": forger.public_key_hex(),
+               "signature_hex": sig}),
+    );
+    assert!(ok.get("result").is_some(), "the hub stores other types (NID): {ok}");
+    let _ = rx.pass();
+    assert!(
+        kinds(&store).iter().any(|(k, d)| k == "inbox-unknown-type" && d.contains("sidecar.receipt")),
+        "{:?}",
+        kinds(&store)
+    );
+}
```
