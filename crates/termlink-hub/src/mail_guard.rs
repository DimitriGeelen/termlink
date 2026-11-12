//! The hub's mail guard: decides from the hub record whether each inbox or conversation record's message is final.
//! T-3701 (arc-011 plan unit P-13, build item B-3; 04 ARC4 6.6.3 and V-3; 06a AL-31; E-67, E-169).
//!
//! Installed on the channel bus at start ([`install`]). The bus asks it, before
//! any sweep, trim, retention change, compaction or ceiling removal on an
//! `inbox:*` or `conv:*` topic, whether the records it would remove belong to
//! final messages. The hub record of a message's state is the set of signed
//! stage records (`sidecar.stage.v1`) the receiver posts to the sender's inbox
//! (04 section 5.3). A message is final once any of its stage records names a
//! final stage (REPLIED, ACKNOWLEDGED_NO_ACTION, REFUSED, FORWARDED, DEAD, or the
//! operator's DENIED).
//!
//! | record on a guarded topic              | final when                                  | cannot tell (FCW-127) |
//! |----------------------------------------|---------------------------------------------|-----------------------|
//! | `sidecar.msg.v1`                       | a final stage for its `sidecar_msg_id` exists | no message id         |
//! | `sidecar.stage.v1`                     | its own message is final                    | payload not readable  |
//! | anything else (not arc-011 mail)       | always (nothing open to protect)            | -                     |
//!
//! Finality is monotone (a final stage never becomes open again), so the set
//! of final message ids is cached and extended incrementally: each call reads
//! only the stage records posted since the previous call. The cache also keeps
//! a message final after its REPLIED record itself has been trimmed, for the
//! life of the process. A restarted hub re-reads what is still live; a message
//! whose final stage was already trimmed then reads as open, which only ever
//! makes a removal refuse, never removes open mail.
//!
//! Ceiling removals are written to the admin log; a forced removal of an open
//! message (last resort) is a needs-attention row and an error-level log line.

use std::collections::{HashMap, HashSet};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex};

use termlink_bus::{Bus, CeilingRemoval, Envelope, Offset, RemovalGuard};

/// Hub-visible type of a sidecar message (mirror of `termlink_sidecar::wire::MSG_TYPE`;
/// the hub does not depend on the sidecar crate).
pub const MSG_TYPE: &str = "sidecar.msg.v1";
/// Hub-visible type of a stage record (mirror of `termlink_sidecar::wire::STAGE_TYPE`).
pub const STAGE_TYPE: &str = "sidecar.stage.v1";
/// Post metadata key carrying the message id (set by the sidecar's send path).
pub const META_MSG_ID: &str = "sidecar_msg_id";

/// Final stages (06a AL-31: REPLIED, ACK_NO_ACTION, REFUSED, FORWARDED, DEAD;
/// R-35.o names ACKNOWLEDGED_NO_ACTION; DENIED is the operator's refusal of a held message).
pub const FINAL_STAGES: &[&str] = &[
    "REPLIED",
    "ACKNOWLEDGED_NO_ACTION",
    "ACK_NO_ACTION",
    "REFUSED",
    "FORWARDED",
    "DEAD",
    "DENIED",
];

/// T-3701: JSON-RPC code of a removal refused on open mail (FCW-126) or on
/// unreadable finality (FCW-127). Hub-local: `termlink-protocol`'s error table
/// gains it in a follow-up (that crate was outside this unit's write set).
pub const MAIL_REMOVAL_REFUSED: i64 = -32040;
/// T-3701: JSON-RPC code of an admin RPC refused because its admin-log row
/// could not be written (FCW-128). Hub-local, as above.
pub const ADMIN_LOG_UNWRITABLE: i64 = -32041;

/// The refusal response for a bus error that is a mail refusal, else `None`.
pub fn refusal(
    id: serde_json::Value,
    method: &str,
    e: &termlink_bus::BusError,
) -> Option<termlink_protocol::jsonrpc::RpcResponse> {
    use termlink_bus::BusError;
    let (fcw, data) = match e {
        BusError::OpenMail { topic, offset } => ("FCW-126", serde_json::json!({"topic": topic, "offset": offset, "fcw": "FCW-126"})),
        BusError::FinalityUnreadable { topic, reason } => {
            ("FCW-127", serde_json::json!({"topic": topic, "reason": reason, "fcw": "FCW-127"}))
        }
        _ => return None,
    };
    tracing::warn!(method, fcw, error = %e, "T-3701: removal refused on a mail topic");
    Some(termlink_protocol::jsonrpc::ErrorResponse::with_data(id, MAIL_REMOVAL_REFUSED, &format!("{method}: {e}"), data).into())
}

/// Topics guarded: the arc-011 inboxes and conversation topics.
pub fn is_guarded_topic(topic: &str) -> bool {
    topic.starts_with("inbox:") || topic.starts_with("conv:")
}

/// The message id a record is about, or `None` when it is not arc-011 mail.
/// `Err` when it is mail but its id cannot be read.
pub fn message_id_of(env: &Envelope) -> Result<Option<String>, String> {
    match env.msg_type.as_str() {
        MSG_TYPE => {
            if let Some(id) = env.metadata.get(META_MSG_ID).filter(|s| !s.is_empty()) {
                return Ok(Some(id.clone()));
            }
            payload_field(&env.payload, "msg_id").map(Some).ok_or_else(|| "message without a message id".to_string())
        }
        STAGE_TYPE => payload_field(&env.payload, "msg_id")
            .map(Some)
            .ok_or_else(|| "stage record whose payload cannot be read".to_string()),
        _ => Ok(None),
    }
}

fn payload_field(payload: &[u8], key: &str) -> Option<String> {
    let v: serde_json::Value = serde_json::from_slice(payload).ok()?;
    v.get(key)?.as_str().filter(|s| !s.is_empty()).map(str::to_owned)
}

/// The final message id a stage record proves, if any.
fn final_stage_of(env: &Envelope) -> Option<String> {
    if env.msg_type != STAGE_TYPE {
        return None;
    }
    let v: serde_json::Value = serde_json::from_slice(&env.payload).ok()?;
    let stage = v.get("stage")?.as_str()?;
    if !FINAL_STAGES.contains(&stage) {
        return None;
    }
    v.get("msg_id")?.as_str().filter(|s| !s.is_empty()).map(str::to_owned)
}

#[derive(Default)]
struct Cache {
    /// Next offset to read, per guarded topic.
    cursors: HashMap<String, Offset>,
    finals: HashSet<String>,
}

/// The hub's [`RemovalGuard`].
#[derive(Default)]
pub struct HubMailGuard {
    cache: Mutex<Cache>,
    /// Test seam (E-169): every read of the hub record fails.
    fail_reads: AtomicBool,
    forced_total: AtomicU64,
    ceiling_removed_total: AtomicU64,
}

impl HubMailGuard {
    pub fn new() -> Self {
        Self::default()
    }

    /// Test seam (E-169): make the hub record unreadable.
    #[doc(hidden)]
    pub fn set_fail_reads(&self, on: bool) {
        self.fail_reads.store(on, Ordering::SeqCst);
    }

    /// Open messages removed as a last resort since start (needs-attention count).
    pub fn forced_total(&self) -> u64 {
        self.forced_total.load(Ordering::Relaxed)
    }

    pub fn ceiling_removed_total(&self) -> u64 {
        self.ceiling_removed_total.load(Ordering::Relaxed)
    }

    /// Read the stage records posted since the last call on every guarded topic.
    fn refresh(&self, bus: &Bus, cache: &mut Cache) -> Result<(), String> {
        if self.fail_reads.load(Ordering::SeqCst) {
            return Err("cannot read the hub record (test seam)".into());
        }
        let topics = bus.list_topics().map_err(|e| format!("cannot list topics: {e}"))?;
        for t in topics.into_iter().filter(|t| is_guarded_topic(t)) {
            let mut cur = cache.cursors.get(&t).copied().unwrap_or(0);
            // A deleted and re-created topic restarts at offset 0.
            let latest = bus.latest_offset(&t).map_err(|e| format!("cannot read {t}: {e}"))?;
            if latest.is_none_or(|l| l + 1 < cur) {
                cur = 0;
            }
            let iter = bus.subscribe(&t, cur).map_err(|e| format!("cannot read {t}: {e}"))?;
            for item in iter {
                let (off, env) = item.map_err(|e| format!("cannot read {t}: {e}"))?;
                if let Some(id) = final_stage_of(&env) {
                    cache.finals.insert(id);
                }
                cur = off + 1;
            }
            cache.cursors.insert(t, cur);
        }
        Ok(())
    }
}

impl RemovalGuard for HubMailGuard {
    fn guards(&self, topic: &str) -> bool {
        is_guarded_topic(topic)
    }

    fn finality(&self, bus: &Bus, topic: &str, records: &[(Offset, Envelope)]) -> Result<Vec<bool>, String> {
        let mut cache = self.cache.lock().unwrap_or_else(|p| p.into_inner());
        self.refresh(bus, &mut cache)?;
        records
            .iter()
            .map(|(off, env)| match message_id_of(env) {
                Ok(None) => Ok(true),
                Ok(Some(id)) => Ok(cache.finals.contains(&id)),
                Err(why) => Err(format!("{topic} offset {off}: {why}")),
            })
            .collect()
    }

    fn on_ceiling_removal(&self, r: &CeilingRemoval) {
        let n = (r.final_offsets.len() + r.forced_open_offsets.len()) as u64;
        self.ceiling_removed_total.fetch_add(n, Ordering::Relaxed);
        let needs_attention = !r.forced_open_offsets.is_empty();
        if needs_attention {
            self.forced_total.fetch_add(r.forced_open_offsets.len() as u64, Ordering::Relaxed);
            tracing::error!(
                topic = %r.topic,
                reason = r.reason,
                open_offsets = ?r.forced_open_offsets,
                "T-3701 NEEDS ATTENTION: V-3 ceiling removed OPEN message record(s) as a last resort (no final record left to remove)"
            );
        } else {
            tracing::warn!(topic = %r.topic, reason = r.reason, removed = n, "T-3701 V-3 ceiling removed final record(s)");
        }
        crate::rpc_audit::admin_event(serde_json::json!({
            "kind": "ceiling-removal",
            "topic": r.topic,
            "reason": r.reason,
            "final_offsets": r.final_offsets,
            "forced_open_offsets": r.forced_open_offsets,
            "needs_attention": needs_attention,
        }));
    }
}

/// Install the guard on the hub's bus. Returns it so status can read its counters.
pub fn install(bus: &Bus) -> Arc<HubMailGuard> {
    let g = Arc::new(HubMailGuard::new());
    bus.set_removal_guard(g.clone());
    g
}

/// The daily V-3 pass: per-topic ceilings on every guarded topic, then the
/// per-hub ceiling. Each failure is logged and the pass continues.
pub fn ceiling_pass(bus: &Bus) -> u64 {
    let mut removed = 0;
    match bus.list_topics() {
        Ok(ts) => {
            for t in ts.into_iter().filter(|t| bus.is_guarded(t)) {
                match bus.apply_mail_ceiling(&t) {
                    Ok(n) => removed += n,
                    Err(e) => tracing::warn!(topic = %t, error = %e, "T-3701 V-3 ceiling: topic skipped, nothing removed"),
                }
            }
        }
        Err(e) => tracing::warn!(error = %e, "T-3701 V-3 ceiling: cannot list topics; pass skipped"),
    }
    match bus.apply_mail_hub_ceiling() {
        Ok((n, errs)) => {
            removed += n;
            for e in errs {
                tracing::warn!(error = %e, "T-3701 V-3 hub ceiling: topic skipped, nothing of it removed");
            }
        }
        Err(e) => tracing::warn!(error = %e, "T-3701 V-3 hub ceiling failed"),
    }
    removed
}

/// Run [`ceiling_pass`] daily (first pass one minute after start).
pub async fn run_daily(mut shutdown: tokio::sync::watch::Receiver<bool>) {
    let mut wait = std::time::Duration::from_secs(60);
    loop {
        tokio::select! {
            _ = tokio::time::sleep(wait) => {}
            _ = shutdown.changed() => return,
        }
        if let Some(bus) = crate::channel::bus() {
            match tokio::task::spawn_blocking(move || ceiling_pass(bus)).await {
                Ok(n) => tracing::info!(removed = n, "T-3701 V-3 daily ceiling pass done"),
                Err(e) => tracing::warn!(error = %e, "T-3701 V-3 daily ceiling pass panicked"),
            }
        }
        wait = std::time::Duration::from_secs(86_400);
    }
}

#[cfg(test)]
mod tests {
    //! T-3701 fixtures, named by the E-n they carry (tests/t3701-b3-fixtures.sh
    //! prints one PASS line per E-n from these names). Records are shaped as the
    //! sidecar writes them (send.rs: message metadata `sidecar_msg_id`; stage
    //! payload `{msg_id, stage, ...}` posted to the sender's inbox).
    use super::*;
    use serde_json::json;
    use termlink_bus::{BusError, MailCeilings, Retention};
    use termlink_protocol::jsonrpc::{Request, RpcResponse};

    fn bus() -> (tempfile::TempDir, Bus, Arc<HubMailGuard>) {
        let d = tempfile::TempDir::new().unwrap();
        let b = Bus::open(d.path()).unwrap();
        let g = install(&b);
        for t in ["inbox:a", "inbox:b"] {
            b.create_topic(t, Retention::Forever).unwrap();
        }
        (d, b, g)
    }

    fn msg(topic: &str, id: &str, ts: i64) -> Envelope {
        Envelope {
            topic: topic.into(),
            sender_id: "fp-a".into(),
            msg_type: MSG_TYPE.into(),
            payload: serde_json::to_vec(&json!({"v":1,"kind":"msg","msg_id":id,"body":"hi"})).unwrap(),
            artifact_ref: None,
            ts_unix_ms: ts,
            metadata: [(META_MSG_ID.to_string(), id.to_string())].into_iter().collect(),
            signature: None,
        }
    }

    fn stage(topic: &str, id: &str, st: &str, ts: i64) -> Envelope {
        Envelope {
            topic: topic.into(),
            sender_id: "fp-b".into(),
            msg_type: STAGE_TYPE.into(),
            payload: serde_json::to_vec(&json!({"v":1,"kind":"stage","msg_id":id,"stage":st})).unwrap(),
            artifact_ref: None,
            ts_unix_ms: ts,
            metadata: Default::default(),
            signature: None,
        }
    }

    /// inbox:b = [m1 (final: REPLIED in inbox:a), m2 (open: only STORED), m3 (final)].
    async fn mailbox(b: &Bus) {
        b.post("inbox:b", &msg("inbox:b", "m1", 1)).await.unwrap();
        b.post("inbox:b", &msg("inbox:b", "m2", 2)).await.unwrap();
        b.post("inbox:b", &msg("inbox:b", "m3", 3)).await.unwrap();
        b.post("inbox:a", &stage("inbox:a", "m1", "REPLIED", 4)).await.unwrap();
        b.post("inbox:a", &stage("inbox:a", "m2", "STORED", 5)).await.unwrap();
        b.post("inbox:a", &stage("inbox:a", "m3", "ACKNOWLEDGED_NO_ACTION", 6)).await.unwrap();
    }

    fn live(b: &Bus, t: &str) -> Vec<u64> {
        b.record_index(t).unwrap().iter().map(|r| r.offset).collect()
    }

    fn err_of(r: RpcResponse) -> (i64, String) {
        match r {
            RpcResponse::Error(e) => (e.error.code, e.error.message),
            RpcResponse::Success(s) => panic!("expected a refusal, got {}", s.result),
        }
    }

    #[test]
    fn e67_finality_is_read_from_the_stage_records() {
        let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
        rt.block_on(async {
            let (_d, b, g) = bus();
            mailbox(&b).await;
            let recs: Vec<_> = b.subscribe("inbox:b", 0).unwrap().map(|r| r.unwrap()).collect();
            assert_eq!(g.finality(&b, "inbox:b", &recs).unwrap(), vec![true, false, true]);
            // Stage records follow their message: the STORED stage of open m2 is open.
            let st: Vec<_> = b.subscribe("inbox:a", 0).unwrap().map(|r| r.unwrap()).collect();
            assert_eq!(g.finality(&b, "inbox:a", &st).unwrap(), vec![true, false, true]);
        });
    }

    #[tokio::test]
    async fn e67_sweep_trim_set_retention_refused_across_the_open_offset_with_reason() {
        let (_d, b, _g) = bus();
        mailbox(&b).await;
        let (code, m) = err_of(crate::channel::handle_channel_trim_with(&b, json!(1), &json!({"topic":"inbox:b"})).await);
        assert_eq!(code, MAIL_REMOVAL_REFUSED);
        assert!(m.contains("open message at offset 1") && m.contains("FCW-126"), "{m}");
        let (code, m) = err_of(
            crate::channel::handle_channel_set_retention_with(&b, json!(2), &json!({"name":"inbox:b","retention":{"kind":"messages","value":1}})).await,
        );
        assert_eq!(code, MAIL_REMOVAL_REFUSED, "{m}");
        assert_eq!(b.topic_retention("inbox:b").unwrap(), Some(Retention::Forever));
        // Messages(3) removes nothing; then a sweep under Messages(1) would remove m1 and open m2.
        b.set_topic_retention("inbox:b", Retention::Messages(3)).unwrap();
        b.post("inbox:b", &msg("inbox:b", "m4", 7)).await.unwrap();
        b.post("inbox:b", &msg("inbox:b", "m5", 8)).await.unwrap();
        let (code, m) = err_of(crate::channel::handle_channel_sweep_with(&b, json!(3), &json!({"topic":"inbox:b"})).await);
        assert_eq!(code, MAIL_REMOVAL_REFUSED);
        assert!(m.contains("open message at offset 1"), "{m}");
        assert_eq!(live(&b, "inbox:b"), vec![0, 1, 2, 3, 4], "nothing removed by any refused call");
        // Below the open offset the trim is allowed and removes exactly m1.
        let ok = crate::channel::handle_channel_trim_with(&b, json!(4), &json!({"topic":"inbox:b","before_offset":1})).await;
        assert!(matches!(ok, RpcResponse::Success(_)));
        assert_eq!(live(&b, "inbox:b"), vec![1, 2, 3, 4]);
    }

    /// E-67.5 CONTROL: the mutant without the open-offset check (a guard that
    /// calls everything final) removes the open message. Proves the refusal
    /// above is the check, not an accident of the fixture.
    #[tokio::test]
    async fn e67_control_mutant_without_the_open_offset_check_removes_open_mail() {
        struct AllFinal;
        impl RemovalGuard for AllFinal {
            fn guards(&self, t: &str) -> bool {
                is_guarded_topic(t)
            }
            fn finality(&self, _: &Bus, _: &str, r: &[(Offset, Envelope)]) -> Result<Vec<bool>, String> {
                Ok(vec![true; r.len()])
            }
        }
        let (_d, b, _g) = bus();
        mailbox(&b).await;
        b.set_removal_guard(Arc::new(AllFinal));
        assert_eq!(b.trim_topic("inbox:b", None).unwrap(), 3, "mutant removed open m2");
    }

    #[tokio::test]
    async fn e169_finality_unreadable_is_refused() {
        let (_d, b, g) = bus();
        mailbox(&b).await;
        // (1) the hub record cannot be read. inbox:c holds three plain records
        // under messages(1): a sweep would remove two.
        b.create_topic("inbox:c", Retention::Messages(1)).unwrap();
        for i in 0..3 {
            let mut e = msg("inbox:c", "n", i);
            e.msg_type = "note".into();
            b.post("inbox:c", &e).await.unwrap();
        }
        g.set_fail_reads(true);
        let (code, m) = err_of(crate::channel::handle_channel_sweep_with(&b, json!(1), &json!({"topic":"inbox:c"})).await);
        let (code2, m2) = err_of(crate::channel::handle_channel_trim_with(&b, json!(2), &json!({"topic":"inbox:b","before_offset":1})).await);
        assert_eq!((code, code2), (MAIL_REMOVAL_REFUSED, MAIL_REMOVAL_REFUSED));
        assert!(m.contains("cannot read finality") && m2.contains("FCW-127"), "{m} / {m2}");
        assert_eq!(live(&b, "inbox:c"), vec![0, 1, 2]);
        g.set_fail_reads(false);
        assert_eq!(b.sweep("inbox:c", 0).unwrap(), 2, "control: readable, the same sweep removes the two");
        g.set_fail_reads(true);
        g.set_fail_reads(false);
        // (2) a record whose message id cannot be read.
        let mut bad = stage("inbox:b", "x", "REPLIED", 9);
        bad.payload = b"not json".to_vec();
        b.post("inbox:b", &bad).await.unwrap();
        let mut no_id = msg("inbox:b", "y", 10);
        no_id.metadata.clear();
        no_id.payload = b"{}".to_vec();
        b.post("inbox:b", &no_id).await.unwrap();
        // Offset 3 (unreadable stage) and 4 (message without an id): any removal
        // covering either is refused as unreadable, even next to the open m2.
        for before in [4u64, 5] {
            let err = b.trim_topic("inbox:b", Some(before)).unwrap_err();
            assert!(matches!(err, BusError::FinalityUnreadable { .. }), "{err}");
        }
        assert_eq!(live(&b, "inbox:b"), vec![0, 1, 2, 3, 4], "nothing removed");
    }

    #[tokio::test]
    async fn e67_finality_survives_the_trim_of_its_own_replied_record() {
        let (_d, b, g) = bus();
        mailbox(&b).await;
        let recs: Vec<_> = b.subscribe("inbox:b", 0).unwrap().map(|r| r.unwrap()).collect();
        g.finality(&b, "inbox:b", &recs).unwrap();
        // inbox:a offset 0 is m1's REPLIED: final, removable.
        b.trim_topic("inbox:a", Some(1)).unwrap();
        assert_eq!(g.finality(&b, "inbox:b", &recs[..1]).unwrap(), vec![true], "cache keeps m1 final");
    }

    #[tokio::test]
    async fn e67_ceiling_final_first_then_open_last_resort_raises_needs_attention() {
        let d = tempfile::TempDir::new().unwrap();
        let log = d.path().join("admin.jsonl");
        crate::rpc_audit::ADMIN_TEST_PATH.with(|c| *c.borrow_mut() = Some(log.clone()));
        let (_d, b, g) = bus();
        mailbox(&b).await;
        b.set_mail_ceilings(MailCeilings { enabled: true, topic_max_records: 2, topic_max_bytes: u64::MAX, hub_max_bytes: u64::MAX });
        // 3 records, ceiling 2: m1 (oldest final) goes; open m2 stays.
        assert_eq!(b.apply_ceiling("inbox:b", 0).unwrap(), 1);
        assert_eq!(live(&b, "inbox:b"), vec![1, 2]);
        assert_eq!(g.forced_total(), 0);
        // Ceiling 0 with only... ceiling 1: m3 (final) goes before open m2.
        b.set_mail_ceilings(MailCeilings { enabled: true, topic_max_records: 1, topic_max_bytes: u64::MAX, hub_max_bytes: u64::MAX });
        b.apply_ceiling("inbox:b", 0).unwrap();
        assert_eq!(live(&b, "inbox:b"), vec![1], "final m3 removed while open m2 remained");
        // Only open left and still over: the oldest open goes, with needs-attention.
        b.post("inbox:b", &msg("inbox:b", "m5", 11)).await.unwrap();
        b.apply_ceiling("inbox:b", 0).unwrap();
        assert_eq!(live(&b, "inbox:b"), vec![3]);
        assert_eq!(g.forced_total(), 1);
        let rows = std::fs::read_to_string(&log).unwrap();
        let na: Vec<_> = rows.lines().filter(|l| l.contains("\"needs_attention\":true")).collect();
        assert_eq!(na.len(), 1, "{rows}");
        assert!(na[0].contains("\"forced_open_offsets\":[1]") && na[0].contains("\"admin\":true"), "{}", na[0]);
        assert_eq!(rows.lines().filter(|l| l.contains("ceiling-removal")).count(), 3, "each removal logged");
        crate::rpc_audit::ADMIN_TEST_PATH.with(|c| *c.borrow_mut() = None);
    }

    #[tokio::test]
    async fn t3382_forever_inbox_is_bounded_on_post() {
        let (_d, b, _g) = bus();
        b.set_mail_ceilings(MailCeilings { enabled: true, topic_max_records: 16, topic_max_bytes: u64::MAX, hub_max_bytes: u64::MAX });
        for i in 0..128 {
            b.post("inbox:a", &stage("inbox:a", &format!("z{i}"), "REPLIED", i)).await.unwrap();
        }
        assert_eq!(b.topic_record_count("inbox:a").unwrap(), 16, "forever inbox held at the V-3 ceiling");
    }

    fn req(method: &str, params: serde_json::Value) -> Request {
        serde_json::from_value(json!({"jsonrpc":"2.0","id":1,"method":method,"params":params})).unwrap()
    }

    #[tokio::test]
    async fn e67_every_admin_rpc_gets_one_intent_and_one_result_row() {
        let d = tempfile::TempDir::new().unwrap();
        let log = d.path().join("admin.jsonl");
        crate::rpc_audit::ADMIN_TEST_PATH.with(|c| *c.borrow_mut() = Some(log.clone()));
        let caller = crate::rpc_audit::AdminCaller { from: Some("op"), sender_id: None, peer_addr: Some("10.0.0.1:5"), peer_pid: None };
        for m in crate::rpc_audit::ADMIN_METHODS {
            let r = req(m, json!({"topic":"inbox:b"}));
            crate::server::dispatch_admin_with(&r, &caller, || async { None }).await;
        }
        for m in ["channel.sweep", "channel.set_retention", "channel.trim", "channel.delete", "channel.force_release"] {
            assert!(crate::rpc_audit::is_admin_method(m), "{m} is an admin RPC (BP-7)");
        }
        assert!(!crate::rpc_audit::is_admin_method("channel.post"));
        assert!(!crate::rpc_audit::is_admin_method("channel.create"));
        let rows: Vec<serde_json::Value> =
            std::fs::read_to_string(&log).unwrap().lines().map(|l| serde_json::from_str(l).unwrap()).collect();
        assert_eq!(rows.len(), 2 * crate::rpc_audit::ADMIN_METHODS.len());
        for (i, m) in crate::rpc_audit::ADMIN_METHODS.iter().enumerate() {
            let (a, z) = (&rows[2 * i], &rows[2 * i + 1]);
            assert_eq!((a["phase"].as_str(), z["phase"].as_str()), (Some("intent"), Some("result")));
            assert_eq!(a["admin"], json!(true));
            assert_eq!(a["method"], json!(m));
            assert_eq!(a["caller"]["from"], json!("op"));
            assert_eq!(a["params_digest"].as_str().unwrap().len(), 64);
            assert_eq!(a["row"], z["row"], "result refers to its intent");
        }
        crate::rpc_audit::ADMIN_TEST_PATH.with(|c| *c.borrow_mut() = None);
    }

    #[tokio::test]
    async fn e169_admin_rpc_whose_log_row_cannot_be_written_is_refused_and_never_acts() {
        crate::rpc_audit::ADMIN_TEST_FAIL.with(|c| c.set(true));
        let acted = std::cell::Cell::new(false);
        let r = req("channel.trim", json!({"topic":"inbox:b"}));
        let resp = crate::server::dispatch_admin_with(&r, &Default::default(), || async {
            acted.set(true);
            None
        })
        .await;
        crate::rpc_audit::ADMIN_TEST_FAIL.with(|c| c.set(false));
        let (code, m) = err_of(resp.unwrap());
        assert_eq!(code, ADMIN_LOG_UNWRITABLE);
        assert!(m.contains("FCW-128"), "{m}");
        assert!(!acted.get(), "the RPC must not act without its admin row");
    }

    #[test]
    fn e67_admin_rows_survive_rotation_for_30_days() {
        let d = tempfile::TempDir::new().unwrap();
        let p = d.path().join("admin.jsonl");
        let day: u128 = 86_400_000;
        let now = 100 * day;
        let row = |age_days: u128, n: u32| json!({"ts": (now - age_days * day) as u64, "admin": true, "n": n});
        // 31-day-old, 29-day-old and 1-day-old rows, then the file passes the size cap.
        for (age, n) in [(31, 1), (29, 2), (1, 3)] {
            crate::rpc_audit::append_admin_row_at(&p, &row(age, n), now, u64::MAX, crate::rpc_audit::ADMIN_KEEP_MS).unwrap();
        }
        crate::rpc_audit::append_admin_row_at(&p, &row(0, 4), now, 1, crate::rpc_audit::ADMIN_KEEP_MS).unwrap();
        let text = std::fs::read_to_string(&p).unwrap();
        let ns: Vec<u64> = text.lines().map(|l| serde_json::from_str::<serde_json::Value>(l).unwrap()["n"].as_u64().unwrap()).collect();
        assert_eq!(ns, vec![2, 3, 4], "only the row past 30 days went, whatever the size");
        // Control: a size-only rotation (the rpc-audit.jsonl kind) would have lost the 29-day row.
        termlink_session::audit_append::append_line_capped(&p, "{}", 1).unwrap();
        let rotated = std::fs::read_to_string(&p).unwrap();
        assert!(!rotated.contains("\"n\":2"), "control: size rotation drops young admin rows");
    }
}
