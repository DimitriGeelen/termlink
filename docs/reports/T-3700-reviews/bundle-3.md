# T-3700 review bundle 3 — final confirmation of round-2 fixes

# T-3700 review round 2 — dispositions

Seats (isolated, both `independent`): opencode (`review-2-opencode.md`) **PASS**: every round-1 A finding CLOSED, every
R/D SOUND, 3 new MINOR. claude-code (`review-2-claude-code.md`) **FAIL**: every round-1 BLOCKER/MAJOR CLOSED, every R/D
SOUND, one new MAJOR (N1) introduced by the round-1 fix for finding 2, 7 new MINOR. A = fixed for round 3, R = reject
(reason), D = defer (risk). Round 3 is the last (cap 2–3); its findings become accepted, rejected or deferred, not
re-reviewed.

| # | Finding | Seat | Disp. | Change / reason |
|---|---|---|---|---|
| N1 | Full table: an unrecorded inbox is answered as a clean empty list (FCW-125 broken) | CC (MAJOR) | A | The skip sets a persisted `dropped_since_ms`; `inbox.consumers` for an inbox absent from the table answers an error ("consumer table full since …") while the window reaches it. Test asserts the error; mutant m27. |
| N2 | A missing topic skips the gate, so a bound inbox is ungated until created | CC | A | The gate (reads, claims, receipts) also applies when the topic is bound; unreadable bindings count as bound. Test + mutant m29. |
| N3 | Liveness fallback looser than "-32001 only" | CC | A | Exactly `SESSION_NOT_FOUND` with "Missing 'target'"; test adds a generic coded error; m21 updated. |
| N4 / OC 3 | Reset marker and row only in memory until a record; a dropped row lost | CC, OC | A | The empty table with `incomplete_since_ms` is saved at the move; the row stays queued until written. Restart test + mutant m30. |
| N5 | Suppressed count lost when its row fails; `capped_row` untested | CC | A | Cleared only after a successful write; unit test + mutant m28. |
| N6 | `why` change forces a full rewrite per call | CC | A | `why` updates in memory and is persisted on the 60 s cadence. |
| N7 / OC 1 | Re-check only when `timeout_ms` parses as u64 > 0 | CC, OC | A | Every gated read re-checks the binding before delivering; m20 updated. |
| N8 | Stale-ack rule refuses honest late receipts | CC | R | Measured: the sidecar posts no `receipt` (`grep '"receipt"' crates/termlink-sidecar/src` is empty); receipts come from operator/agent tools that sign at post time. |
| OC 2 | `key_fp` failure → `""` → own entry shown as a second copy | OC | A | An empty own fingerprint makes the line `unknown (this sidecar's own key fingerprint is unavailable)`; unit test. |
| CC 5 gap | `capped_row` untested | CC | A | See N5. |
| CC 5 gap | Unsigned claim before a bind not exercised | CC | D | The claim gate is the same `consumer_gate` as reads, whose unsigned-before-bind path is tested (E-66.6); m16 kills removing it. |

## Code and test diff since bundle 2
```diff
diff --git a/crates/termlink-hub/src/channel.rs b/crates/termlink-hub/src/channel.rs
index 9806b6b32..4fbe6db26 100644
--- a/crates/termlink-hub/src/channel.rs
+++ b/crates/termlink-hub/src/channel.rs
@@ -985,7 +985,10 @@ pub(crate) async fn handle_channel_post_with_peer(
 
     // T-3700 (AL-30, FCW-209): an ack (receipt) on an inbox records its poster as a consumer and is
     // refused on a bound inbox unless it is the bound agent's bound key.
-    if msg_type == "receipt" && crate::inbox_consumers::applies(&topic) && topic_exists(&topic) {
+    if msg_type == "receipt"
+        && crate::inbox_consumers::applies(&topic)
+        && (topic_exists(&topic) || crate::inbox_consumers::is_bound(&termlink_session::discovery::runtime_dir(), &topic))
+    {
         use crate::inbox_consumers as ic;
         let dir = termlink_session::discovery::runtime_dir();
         let reader = {
@@ -1515,9 +1518,9 @@ pub async fn handle_channel_subscribe(id: Value, params: &Value, peer_addr: Opti
         Err(r) => return r,
     };
     let resp = handle_channel_subscribe_with(bus, id.clone(), params).await;
-    // r1 CC 6: a long-poll that waited across a bind does not deliver what was posted after it
-    let waited = params.get("timeout_ms").and_then(|v| v.as_u64()).unwrap_or(0) > 0;
-    if let (true, Some(r), Some(topic)) = (waited, reader.as_ref(), param_str(params, "topic")) {
+    // r1 CC 6 / r2 CC N7, OC 1: every gated read re-checks the binding before delivering, so a read that
+    // waited across a bind (however its timeout was given) does not deliver what was posted after it
+    if let (Some(r), Some(topic)) = (reader.as_ref(), param_str(params, "topic")) {
         let dir = termlink_session::discovery::runtime_dir();
         let mut log = |row: Value| crate::rpc_audit::admin_note(row).map_err(|e| e.to_string());
         if let Err(f) = crate::inbox_consumers::check_bound(&dir, topic, r, crate::inbox_consumers::Action::Read, now_unix_ms().max(0) as u64, &mut log) {
@@ -1566,7 +1569,8 @@ pub(crate) fn consumer_gate(
 ) -> Result<Option<crate::inbox_consumers::Reader>, RpcResponse> {
     use crate::inbox_consumers as ic;
     let Some(topic) = param_str(params, "topic") else { return Ok(None) };
-    if !ic::applies(topic) || !topic_exists(topic) {
+    // r2 CC N2: a bound inbox is gated even before its topic exists
+    if !ic::applies(topic) || (!topic_exists(topic) && !ic::is_bound(&termlink_session::discovery::runtime_dir(), topic)) {
         return Ok(None);
     }
     let position = params.get(position_key).and_then(|v| v.as_u64()).unwrap_or(0);
diff --git a/crates/termlink-hub/src/inbox_consumers.rs b/crates/termlink-hub/src/inbox_consumers.rs
index 5d05de095..8f7bb022a 100644
--- a/crates/termlink-hub/src/inbox_consumers.rs
+++ b/crates/termlink-hub/src/inbox_consumers.rs
@@ -261,6 +261,9 @@ struct ConsumersFile {
     /// Set when an unreadable table was moved aside: readers before this time are lost (r1 CC 4).
     #[serde(default, skip_serializing_if = "Option::is_none")]
     incomplete_since_ms: Option<u64>,
+    /// Last time a new inbox went unrecorded because the table was full (r2 CC N1).
+    #[serde(default, skip_serializing_if = "Option::is_none")]
+    dropped_since_ms: Option<u64>,
 }
 
 #[derive(Debug, Default, Serialize, Deserialize)]
@@ -285,6 +288,7 @@ pub fn bindings_path(dir: &Path) -> PathBuf {
 struct State {
     topics: BTreeMap<String, Vec<Entry>>,
     incomplete_since_ms: Option<u64>,
+    dropped_since_ms: Option<u64>,
     /// The last persist of the consumer table failed: `inbox.consumers` answers an error (FCW-125).
     persist_error: Option<String>,
     /// The consumer file could not be read nor moved aside: nothing is saved over it, and
@@ -320,7 +324,12 @@ fn load_state(dir: &Path) -> State {
                         e.persisted_last_ms = e.last_ms;
                     }
                 }
-                return State { topics, incomplete_since_ms: f.incomplete_since_ms, ..Default::default() };
+                return State {
+                    topics,
+                    incomplete_since_ms: f.incomplete_since_ms,
+                    dropped_since_ms: f.dropped_since_ms,
+                    ..Default::default()
+                };
             }
             Ok(f) => format!("unknown schema {:?}", f.schema),
             Err(e) => format!("unparseable: {e}"),
@@ -332,13 +341,20 @@ fn load_state(dir: &Path) -> State {
     let aside = path.with_file_name(format!("inbox-consumers.json.corrupt-{now}"));
     crate::key_card::guard_test_write(dir);
     match std::fs::rename(&path, &aside) {
-        Ok(()) => State {
-            incomplete_since_ms: Some(now),
-            pending_rows: vec![serde_json::json!({
-                "event": "inbox-consumers-reset", "attention": true, "why": why, "moved_to": aside.display().to_string(),
-            })],
-            ..Default::default()
-        },
+        Ok(()) => {
+            let mut st = State {
+                incomplete_since_ms: Some(now),
+                pending_rows: vec![serde_json::json!({
+                    "event": "inbox-consumers-reset", "attention": true, "why": why, "moved_to": aside.display().to_string(),
+                })],
+                ..Default::default()
+            };
+            // r2 CC N4: the marker reaches disk at once, so a restart before any record keeps it
+            if let Err(e) = save_consumers(dir, &st) {
+                st.persist_error = Some(format!("consumer list not persisted: {e}"));
+            }
+            st
+        }
         Err(e) => State { load_error: Some(format!("consumer table {why}; not moved aside: {e}")), ..Default::default() },
     }
 }
@@ -358,7 +374,12 @@ fn load_bindings(dir: &Path) -> Result<BTreeMap<String, Binding>, String> {
 }
 
 fn save_consumers(dir: &Path, st: &State) -> Result<(), String> {
-    let f = ConsumersFile { schema: CONSUMERS_SCHEMA.into(), topics: st.topics.clone(), incomplete_since_ms: st.incomplete_since_ms };
+    let f = ConsumersFile {
+        schema: CONSUMERS_SCHEMA.into(),
+        topics: st.topics.clone(),
+        incomplete_since_ms: st.incomplete_since_ms,
+        dropped_since_ms: st.dropped_since_ms,
+    };
     let text = serde_json::to_vec_pretty(&f).map_err(|e| e.to_string())?;
     crate::key_card::atomic_write(&consumers_path(dir), &text, 0o600).map_err(|e| e.to_string())
 }
@@ -384,9 +405,11 @@ fn capped_row(kind: &'static str, mut row: serde_json::Value, now_ms: u64, log:
     }
     if w.2 > 0 {
         row["suppressed"] = serde_json::json!(w.2);
+    }
+    // r2 CC N5: the suppressed count is cleared only once a row carrying it is written
+    if log(row).is_ok() {
         w.2 = 0;
     }
-    let _ = log(row);
 }
 
 /// Whether `reader` passes the binding of `topic` (proven, the bound agent, the bound key).
@@ -416,6 +439,11 @@ fn bindings_refusal(e: String) -> Refusal {
     Refusal { code: error_code::INBOX_BOUND, fcw: "FCW-124", reason: format!("{e}; inbox reads are refused until it is repaired") }
 }
 
+/// Whether `topic` has a binding; unreadable bindings count as bound (fail closed, FCW-124).
+pub fn is_bound(dir: &Path, topic: &str) -> bool {
+    load_bindings(dir).map(|b| b.contains_key(topic)).unwrap_or(true)
+}
+
 /// Re-check only the binding (no recording): a long-poll that waited across a bind must not deliver (r1 CC 6).
 pub fn check_bound(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms: u64, log: AdminLog<'_>) -> Result<(), Refusal> {
     let _g = STATES.lock().unwrap_or_else(|p| p.into_inner());
@@ -432,9 +460,8 @@ pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms:
     let mut g = STATES.lock().unwrap_or_else(|p| p.into_inner());
     let map = g.get_or_insert_with(HashMap::new);
     let st = map.entry(dir.to_path_buf()).or_insert_with(|| load_state(dir));
-    for row in std::mem::take(&mut st.pending_rows) {
-        let _ = log(row);
-    }
+    // r2 CC N4: a load-time row stays queued until it is written
+    st.pending_rows.retain(|row| log(row.clone()).is_err());
     // the bind is read under the same lock the bind verb writes under
     let bindings = load_bindings(dir).map_err(bindings_refusal)?;
     if let Some(b) = bindings.get(topic)
@@ -468,6 +495,13 @@ pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms:
                     now_ms,
                     log,
                 );
+                // r2 CC N1: an inbox missing from a full table is answered as an error, never a clean list
+                if st.dropped_since_ms.is_none_or(|t| now_ms >= t + PERSIST_EVERY_MS) {
+                    st.dropped_since_ms = Some(now_ms);
+                    if let Err(e) = save_consumers(dir, st) {
+                        st.persist_error = Some(format!("consumer list not persisted: {e}"));
+                    }
+                }
                 return Ok(());
             }
         }
@@ -479,10 +513,8 @@ pub fn record(dir: &Path, topic: &str, reader: &Reader, action: Action, now_ms:
         .find(|e| e.agent == reader.agent && e.instance == reader.instance && e.strength == reader.strength && e.key_fp == reader.key_fp)
     {
         Some(e) => {
-            if e.why != reader.why {
-                e.why = reader.why.clone();
-                dirty = true;
-            }
+            // r2 CC N6: a changed reason alone is persisted on the PERSIST_EVERY_MS cadence, not per call
+            e.why.clone_from(&reader.why);
             if e.actions.insert(action.name().to_string()) {
                 dirty = true;
             }
@@ -552,6 +584,12 @@ pub fn consumers(dir: &Path, topic: &str, window_s: u64, now_ms: u64) -> Result<
         {
             return Err(format!("consumer history incomplete since {t} (an unreadable table was moved aside)"));
         }
+        if let Some(t) = st.dropped_since_ms
+            && t > since
+            && !st.topics.contains_key(topic)
+        {
+            return Err(format!("consumer table full since {t}: this inbox's readers are not recorded"));
+        }
         let list: Vec<&Entry> = st.topics.get(topic).map(|v| v.iter().filter(|e| e.last_ms >= since).collect()).unwrap_or_default();
         Ok(serde_json::json!({"topic": topic, "window_s": window_s, "consumers": list, "bound": bound}))
     })
diff --git a/crates/termlink-hub/src/inbox_consumers_tests.rs b/crates/termlink-hub/src/inbox_consumers_tests.rs
index b13fb8b2f..ce4123950 100644
--- a/crates/termlink-hub/src/inbox_consumers_tests.rs
+++ b/crates/termlink-hub/src/inbox_consumers_tests.rs
@@ -392,7 +392,9 @@ fn a_full_table_never_evicts_an_inbox_read_inside_the_window() {
         record(d.path(), &format!("inbox:h/t{i}"), &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
     }
     record(d.path(), "inbox:h/new", &claimed_r("fx-z"), Action::Read, n + 1, &mut log).unwrap();
-    assert!(listed(d.path(), "inbox:h/new", n + 1).is_empty(), "not recorded");
+    // r2 CC N1: the unrecorded inbox is answered as an error, never a clean empty list
+    let e = consumers(d.path(), "inbox:h/new", DEFAULT_WINDOW_S, n + 1).unwrap_err();
+    assert!(e.contains("table full"), "{e}");
     assert_eq!(listed(d.path(), "inbox:h/t0", n + 1).len(), 1, "the oldest active inbox kept");
     assert!(logged.borrow().iter().any(|r| r["event"] == "inbox-consumer-table-full"));
     // a day later the oldest inbox is out of the window and may be evicted
@@ -436,3 +438,43 @@ fn a_failed_save_makes_the_list_an_error_never_a_list() {
     record(d.path(), T, &claimed_r("fx-z"), Action::Read, n, &mut log).unwrap();
     assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, n).is_err(), "a list that could not be saved is not served");
 }
+
+#[test]
+fn capped_rows_keep_the_suppressed_count_until_written() {
+    // r2 CC N5
+    let minute = 1_800_000_000_000u64 / 60_000 * 60_000;
+    let mut rows_written: Vec<serde_json::Value> = Vec::new();
+    let mut ok = |r: serde_json::Value| {
+        rows_written.push(r);
+        Ok(())
+    };
+    for _ in 0..12 {
+        capped_row("t3700-test-kind", serde_json::json!({"event": "x"}), minute, &mut ok);
+    }
+    let mut fail = |_r: serde_json::Value| Err("log full".to_string());
+    capped_row("t3700-test-kind", serde_json::json!({"event": "x"}), minute + 60_000, &mut fail);
+    capped_row("t3700-test-kind", serde_json::json!({"event": "x"}), minute + 60_000, &mut ok);
+    assert_eq!(rows_written.len(), ROWS_PER_MINUTE as usize + 1);
+    assert_eq!(rows_written.last().unwrap()["suppressed"], 2, "{:?}", rows_written.last());
+}
+
+#[test]
+fn unreadable_bindings_count_as_bound() {
+    // r2 CC N2: the gate still applies to a missing topic when it is bound, and fails closed
+    let d = dir();
+    assert!(!is_bound(d.path(), T));
+    std::fs::write(bindings_path(d.path()), b"garbage").unwrap();
+    assert!(is_bound(d.path(), T));
+}
+
+#[test]
+fn the_reset_marker_survives_a_restart_before_any_record() {
+    // r2 CC N4
+    let d = dir();
+    std::fs::write(consumers_path(d.path()), b"{not json").unwrap();
+    reset(d.path());
+    let now = now_wall_ms();
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now).is_err());
+    reset(d.path()); // a hub restart
+    assert!(consumers(d.path(), T, DEFAULT_WINDOW_S, now).is_err(), "still incomplete after the restart");
+}
diff --git a/crates/termlink-sidecar/src/consumers.rs b/crates/termlink-sidecar/src/consumers.rs
index 1c20646a5..f6ce1e98d 100644
--- a/crates/termlink-sidecar/src/consumers.rs
+++ b/crates/termlink-sidecar/src/consumers.rs
@@ -56,6 +56,10 @@ pub struct Verdict {
 /// instance or the ack sentinel alone never is.
 pub fn verdict(me_agent: &str, me_instance: &str, me_fp: &str, answer: Result<&Value, (i64, &str)>) -> Verdict {
     let unknown = |why: String| Verdict { line: format!("unknown ({why})"), others: vec![], bound: None };
+    if me_fp.is_empty() {
+        // r2 OC 2: without its own fingerprint the sidecar cannot recognise itself; never guess
+        return unknown("this sidecar's own key fingerprint is unavailable".into());
+    }
     let v = match answer {
         Ok(v) => v,
         Err((METHOD_NOT_FOUND, _)) => return unknown("hub predates the consumer list".into()),
@@ -188,6 +192,7 @@ mod tests {
         assert!(verdict("fx-a", "w1", FP, Err((-32049, "consumer list unreadable"))).line.starts_with("unknown ("));
         assert_eq!(verdict("fx-a", "w1", FP, Err((-32601, "no such method"))).line, "unknown (hub predates the consumer list)");
         assert!(verdict("fx-a", "w1", FP, Ok(&json!({"bound": null}))).line.starts_with("unknown ("));
+        assert!(verdict("fx-a", "w1", "", Ok(&v(vec![e("fx-a", "w1", "proven", "")]))).line.starts_with("unknown ("));
     }
 
     #[test]
diff --git a/crates/termlink-sidecar/src/liveness.rs b/crates/termlink-sidecar/src/liveness.rs
index e61701283..6f5ac4f5c 100644
--- a/crates/termlink-sidecar/src/liveness.rs
+++ b/crates/termlink-sidecar/src/liveness.rs
@@ -218,7 +218,8 @@ pub fn read(hub: &dyn Hub, topic: &str, inbox: &str) -> Read {
             // r1 CC 9 / OC 3: only a hub that predates channel.first_at falls back to the one-message read. A
             // T-3700 hub's own errors name the method; an old hub routes the unknown method to a session
             // (-32001 "Missing 'target'"). A transport error (no code) is Unreadable, never a content read.
-            Err((Some(_), m)) if !m.contains("channel.first_at") => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {
+            // r2 CC N3: exactly the old hub's answer (the unknown method routed to a session), nothing looser
+            Err((Some(code), m)) if code == error_code::SESSION_NOT_FOUND && m.contains("Missing 'target'") => match oldest_by_subscribe(hub, inbox, n.drained_up_to) {
                 Ok(t) => t,
                 Err(e) => return Read::Unreadable(e),
             },
@@ -519,7 +520,7 @@ mod tests {
         let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
         f.replies.insert(("channel.first_at".into(), "I".into()), Err((Some(-32001), "Missing 'target' in params".into())));
         assert!(matches!(read(&f, "L", "I"), Read::Note { oldest_waiting_ms: Some(1234), .. }));
-        for err in [(None, "connection reset".to_string()), (Some(-32603), "channel.first_at: io".to_string())] {
+        for err in [(None, "connection reset".to_string()), (Some(-32603), "channel.first_at: io".to_string()), (Some(-32603), "bus unavailable".to_string())] {
             let mut f = fake(&n, Ok(json!({"messages": [envelope(2, "sidecar.msg.v1", b"x", 1234)]})));
             f.replies.insert(("channel.first_at".into(), "I".into()), Err(err.clone()));
             assert!(matches!(read(&f, "L", "I"), Read::Unreadable(_)), "{err:?}");
```

## Measured
hub+bus+sidecar suites: 1289 passed, 0 failed; hub unit inbox_consumers: 33 passed.

## Round-3 run, 2026-11-20 (after review round 2; m27–m30 cover its fixes)

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
| m21-liveness-fallback-any-error | r1 CC 9 / r2 N3: liveness falls back to a content read on any coded first_at error | killed |
| m22-ack-replay-proves | r1 CC 7: a replayed old receipt proves | killed |
| m23-corrupt-overwritten | r1 CC 4: a corrupt table is not moved aside (saved over) | killed |
| m24-stale-status-shown | r1 CC 2: an old consumers line is shown as current | killed |
| m25-unknown-topics-recorded | r1 CC 3: reads of inbox topics that do not exist are recorded | killed |
| m27-full-table-clean-list | r2 N1: an inbox missing from a full table is answered as a clean empty list | killed |
| m28-suppressed-count-lost | r2 N5: the suppressed count is dropped when the row carrying it fails | killed |
| m29-bindings-unreadable-unbound | r2 N2: unreadable bindings count as unbound for a missing topic | killed |
| m30-reset-marker-not-saved | r2 N4: the reset marker stays in memory until a record | killed |
| m26-full-table-evicts-active | r1 CC 3: a full table evicts an inbox read inside the window | killed |


**30/30 killed**
