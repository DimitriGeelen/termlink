//! T-3701 (arc-011 P-13, B-3 / V-3): the bus-side removal guard, driven by a fake guard.
//! E-67 (sweep, trim, set-retention refused across an open offset; ceilings final first,
//! open only as a last resort), E-169 (finality unreadable -> refused). Each refusal test
//! has a negative control: the same removal with no guard installed removes the open record.

use std::sync::{Arc, Mutex};
use termlink_bus::{Bus, BusError, CeilingRemoval, Envelope, MailCeilings, Offset, RemovalGuard, Retention};

/// Records with msg_type "open" are open mail; everything else is final.
#[derive(Default)]
struct FakeGuard {
    fail: bool,
    removals: Mutex<Vec<CeilingRemoval>>,
}

impl RemovalGuard for FakeGuard {
    fn guards(&self, topic: &str) -> bool {
        topic.starts_with("inbox:")
    }
    fn finality(&self, _bus: &Bus, _topic: &str, records: &[(Offset, Envelope)]) -> Result<Vec<bool>, String> {
        if self.fail {
            return Err("hub record unreadable (fake)".into());
        }
        Ok(records.iter().map(|(_, e)| !e.msg_type.starts_with("open")).collect())
    }
    fn on_ceiling_removal(&self, r: &CeilingRemoval) {
        self.removals.lock().unwrap().push(r.clone());
    }
}

fn env(topic: &str, kind: &str, ts: i64) -> Envelope {
    Envelope {
        topic: topic.into(),
        sender_id: "s".into(),
        msg_type: kind.into(),
        payload: vec![b'x'; 10],
        artifact_ref: None,
        ts_unix_ms: ts,
        metadata: Default::default(),
        signature: None,
    }
}

/// A bus with `inbox:b` holding `kinds` at offsets 0.. (ts = offset * 1000).
async fn bus_with(kinds: &[&str], retention: Retention, guard: Option<Arc<FakeGuard>>) -> (tempfile::TempDir, Bus) {
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Bus::open(dir.path()).unwrap();
    no_post_trim(&bus);
    if let Some(g) = guard {
        bus.set_removal_guard(g);
    }
    bus.create_topic("inbox:b", retention).unwrap();
    for (i, k) in kinds.iter().enumerate() {
        bus.post("inbox:b", &env("inbox:b", k, i as i64 * 1000)).await.unwrap();
    }
    (dir, bus)
}

/// The pre-existing post-time ceilings (T-3310) would trim an unguarded
/// Messages(1) topic while the fixture fills it; switch them off so the
/// controls measure the sweep alone.
fn no_post_trim(bus: &Bus) {
    bus.set_ceilings(termlink_bus::Ceilings { on_post: false, ..Default::default() });
}

fn live(bus: &Bus) -> Vec<Offset> {
    bus.record_index("inbox:b").unwrap().iter().map(|r| r.offset).collect()
}

#[tokio::test]
async fn e67_sweep_across_an_open_offset_is_refused_and_removes_nothing() {
    let (_d, bus) = bus_with(&["final", "open", "final", "final"], Retention::Messages(1), Some(Arc::default())).await;
    let err = bus.sweep("inbox:b", 10_000).unwrap_err();
    assert!(matches!(err, BusError::OpenMail { offset: 1, .. }), "{err}");
    assert!(err.to_string().contains("open message at offset 1"), "{err}");
    assert_eq!(live(&bus), vec![0, 1, 2, 3]);
}

#[tokio::test]
async fn e67_control_sweep_without_the_guard_removes_the_open_message() {
    let (_d, bus) = bus_with(&["final", "open", "final", "final"], Retention::Messages(1), None).await;
    assert_eq!(bus.sweep("inbox:b", 10_000).unwrap(), 3);
    assert_eq!(live(&bus), vec![3], "control: unguarded sweep removed open offset 1");
}

#[tokio::test]
async fn e67_sweep_of_only_final_records_removes_exactly_them() {
    let (_d, bus) = bus_with(&["final", "final", "open", "final"], Retention::Messages(2), Some(Arc::default())).await;
    assert_eq!(bus.sweep("inbox:b", 10_000).unwrap(), 2);
    assert_eq!(live(&bus), vec![2, 3]);
}

#[tokio::test]
async fn e67_trim_across_an_open_offset_is_refused() {
    let (_d, bus) = bus_with(&["final", "open", "final"], Retention::Forever, Some(Arc::default())).await;
    assert!(matches!(bus.trim_topic("inbox:b", None).unwrap_err(), BusError::OpenMail { offset: 1, .. }));
    assert!(matches!(bus.trim_topic("inbox:b", Some(2)).unwrap_err(), BusError::OpenMail { offset: 1, .. }));
    assert_eq!(live(&bus), vec![0, 1, 2]);
    // Below the open offset: allowed, exactly offset 0.
    assert_eq!(bus.trim_topic("inbox:b", Some(1)).unwrap(), 1);
    assert_eq!(live(&bus), vec![1, 2]);
}

#[tokio::test]
async fn e67_control_trim_without_the_guard_removes_the_open_message() {
    let (_d, bus) = bus_with(&["final", "open", "final"], Retention::Forever, None).await;
    assert_eq!(bus.trim_topic("inbox:b", None).unwrap(), 3);
}

#[tokio::test]
async fn e67_set_retention_that_would_remove_an_open_message_is_refused() {
    let (_d, bus) = bus_with(&["open", "final", "final"], Retention::Forever, Some(Arc::default())).await;
    let err = bus.set_topic_retention("inbox:b", Retention::Messages(1)).unwrap_err();
    assert!(matches!(err, BusError::OpenMail { offset: 0, .. }), "{err}");
    assert_eq!(bus.topic_retention("inbox:b").unwrap(), Some(Retention::Forever), "policy unchanged");
    // A policy that removes nothing open is allowed.
    assert!(bus.set_topic_retention("inbox:b", Retention::Messages(3)).unwrap());
}

#[tokio::test]
async fn e67_unguarded_topics_are_unchanged() {
    let g: Arc<FakeGuard> = Arc::default();
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Bus::open(dir.path()).unwrap();
    no_post_trim(&bus);
    bus.set_removal_guard(g);
    bus.create_topic("chat", Retention::Messages(1)).unwrap();
    for i in 0..3 {
        bus.post("chat", &env("chat", "open", i)).await.unwrap();
    }
    assert_eq!(bus.sweep("chat", 0).unwrap(), 2, "a non-mail topic sweeps as before");
}

#[tokio::test]
async fn e169_finality_unreadable_refuses_sweep_trim_and_set_retention() {
    let g = Arc::new(FakeGuard { fail: true, ..Default::default() });
    let (_d, bus) = bus_with(&["final", "final", "final"], Retention::Messages(1), Some(g)).await;
    for err in [
        bus.sweep("inbox:b", 0).unwrap_err(),
        bus.trim_topic("inbox:b", None).unwrap_err(),
        bus.set_topic_retention("inbox:b", Retention::Latest).unwrap_err(),
    ] {
        assert!(matches!(err, BusError::FinalityUnreadable { .. }), "{err}");
        assert!(err.to_string().contains("cannot read finality"), "{err}");
    }
    assert_eq!(live(&bus), vec![0, 1, 2]);
}

#[tokio::test]
async fn periodic_sweep_keeps_open_mail_and_removes_final() {
    let (_d, bus) = bus_with(&["open", "final", "final", "final"], Retention::Messages(1), Some(Arc::default())).await;
    assert_eq!(bus.sweep_final_only("inbox:b", 0).unwrap(), (2, 1));
    assert_eq!(live(&bus), vec![0, 3], "open offset 0 never cut while open");
}

fn shrunk(records: u64) -> MailCeilings {
    MailCeilings { enabled: true, topic_max_records: records, topic_max_bytes: u64::MAX, hub_max_bytes: u64::MAX }
}

#[tokio::test]
async fn e67_ceiling_removes_final_first_and_keeps_open_while_a_final_remains() {
    let g: Arc<FakeGuard> = Arc::default();
    // Forever topic (the T-3382 case): the old ceilings only warned here.
    let (_d, bus) = bus_with(&["open", "final", "open", "final", "final"], Retention::Forever, Some(g.clone())).await;
    bus.set_mail_ceilings(shrunk(3));
    assert_eq!(bus.apply_ceiling("inbox:b", 0).unwrap(), 2);
    assert_eq!(live(&bus), vec![0, 2, 4], "the two oldest FINAL records went; open 0 and 2 stay");
    let r = g.removals.lock().unwrap().clone();
    assert_eq!(r.len(), 1);
    assert_eq!(r[0].final_offsets, vec![1, 3]);
    assert!(r[0].forced_open_offsets.is_empty());
}

#[tokio::test]
async fn e67_ceiling_removes_open_only_as_last_resort_with_needs_attention() {
    let g: Arc<FakeGuard> = Arc::default();
    let (_d, bus) = bus_with(&["open", "final", "open", "open"], Retention::Forever, Some(g.clone())).await;
    bus.set_mail_ceilings(shrunk(2));
    assert_eq!(bus.apply_ceiling("inbox:b", 0).unwrap(), 2);
    assert_eq!(live(&bus), vec![2, 3]);
    let r = g.removals.lock().unwrap().clone();
    assert_eq!(r[0].final_offsets, vec![1]);
    assert_eq!(r[0].forced_open_offsets, vec![0], "the oldest open one, reported for needs-attention");
}

#[tokio::test]
async fn e67_ceiling_fires_on_post_for_a_forever_inbox() {
    let g: Arc<FakeGuard> = Arc::default();
    let (_d, bus) = bus_with(&[], Retention::Forever, Some(g.clone())).await;
    bus.set_mail_ceilings(shrunk(10));
    // Checked every 64th post: 64 final posts leave the topic at the ceiling.
    for i in 0..64 {
        bus.post("inbox:b", &env("inbox:b", "final", i)).await.unwrap();
    }
    assert_eq!(bus.topic_record_count("inbox:b").unwrap(), 10, "forever inbox bounded by V-3 (T-3382)");
}

#[tokio::test]
async fn e67_ceilings_off_is_the_rollback() {
    let g: Arc<FakeGuard> = Arc::default();
    let (_d, bus) = bus_with(&["final"; 5], Retention::Forever, Some(g)).await;
    bus.set_mail_ceilings(MailCeilings { enabled: false, ..shrunk(1) });
    assert_eq!(bus.apply_ceiling("inbox:b", 0).unwrap(), 0);
    assert_eq!(live(&bus).len(), 5);
}

#[tokio::test]
async fn e169_ceiling_with_unreadable_finality_removes_nothing() {
    let g = Arc::new(FakeGuard { fail: true, ..Default::default() });
    let (_d, bus) = bus_with(&["final"; 5], Retention::Forever, Some(g)).await;
    bus.set_mail_ceilings(shrunk(1));
    assert!(matches!(bus.apply_ceiling("inbox:b", 0).unwrap_err(), BusError::FinalityUnreadable { .. }));
    assert_eq!(live(&bus).len(), 5, "never trimmed blind");
}

#[tokio::test]
async fn e67_hub_ceiling_takes_the_oldest_final_across_topics() {
    let g: Arc<FakeGuard> = Arc::default();
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Bus::open(dir.path()).unwrap();
    bus.set_removal_guard(g.clone());
    for t in ["inbox:a", "inbox:b"] {
        bus.create_topic(t, Retention::Forever).unwrap();
    }
    // ts order: a0 open(0) b0 final(1) a1 final(2) b1 final(3); each record is the same size.
    bus.post("inbox:a", &env("inbox:a", "opens", 0)).await.unwrap();
    bus.post("inbox:b", &env("inbox:b", "final", 1)).await.unwrap();
    bus.post("inbox:a", &env("inbox:a", "final", 2)).await.unwrap();
    bus.post("inbox:b", &env("inbox:b", "final", 3)).await.unwrap();
    let per = bus.record_index("inbox:a").unwrap()[0].len;
    bus.set_mail_ceilings(MailCeilings { enabled: true, topic_max_records: u64::MAX, topic_max_bytes: u64::MAX, hub_max_bytes: per * 2 });
    let (n, errs) = bus.apply_mail_hub_ceiling().unwrap();
    assert!(errs.is_empty());
    assert_eq!(n, 2);
    let a: Vec<_> = bus.record_index("inbox:a").unwrap().iter().map(|r| r.offset).collect();
    let b: Vec<_> = bus.record_index("inbox:b").unwrap().iter().map(|r| r.offset).collect();
    assert_eq!(a, vec![0], "open a0 kept though oldest");
    assert_eq!(b, vec![1], "b0 (oldest final) and a1 went; b1 kept");
}
