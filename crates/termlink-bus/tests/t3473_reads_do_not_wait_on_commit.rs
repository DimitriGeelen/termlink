//! T-3473: a meta read must not queue behind a commit that is holding the write
//! connection through its fsync. After T-3461 every meta access shared one mutex,
//! and on .107 one btrfs fsync took over 60 s while channel.list / channel.info
//! timed out behind it (ring20 report, 2026-10-18).
//!
//! Own test binary on purpose: the slow-durable seam is process-global.

use std::sync::Arc;
use std::time::{Duration, Instant};

use termlink_bus::{Bus, Envelope, Retention, set_test_slow_durable_ms};

/// Both tests run one at a time: the slow-durable seam would otherwise slow the
/// other test's setup.
static SERIAL: std::sync::Mutex<()> = std::sync::Mutex::new(());

fn env(topic: &str) -> Envelope {
    Envelope {
        topic: topic.into(),
        sender_id: "s".into(),
        msg_type: "note".into(),
        payload: b"x".to_vec(),
        artifact_ref: None,
        ts_unix_ms: 0,
        metadata: std::collections::BTreeMap::new(),
        signature: None,
    }
}

#[test]
fn reads_return_while_a_commit_holds_the_write_connection() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Arc::new(Bus::open(dir.path()).unwrap());
    bus.create_topic("t", Retention::Forever).unwrap();
    let rt = tokio::runtime::Builder::new_multi_thread()
        .worker_threads(2)
        .enable_all()
        .build()
        .unwrap();

    // Every durable step of the post now takes 1.5 s, with the meta commit
    // holding the write connection for its whole 1.5 s.
    set_test_slow_durable_ms(1500);
    let post = {
        let bus = bus.clone();
        rt.spawn(async move { bus.post("t", &env("t")).await })
    };

    // Sample the read paths channel.list and channel.info use, from a plain
    // thread, against the wall clock, while the post is still in progress.
    let mut worst = Duration::ZERO;
    let window = Instant::now();
    while window.elapsed() < Duration::from_millis(2500) {
        let t = Instant::now();
        bus.topic_summaries().unwrap();
        bus.topic_record_count("t").unwrap();
        bus.topic_retention("t").unwrap();
        worst = worst.max(t.elapsed());
        std::thread::sleep(Duration::from_millis(100));
    }
    let overlapped = !post.is_finished();
    set_test_slow_durable_ms(0);

    assert!(overlapped, "the post finished before sampling ended; the test proved nothing");
    assert!(
        worst < Duration::from_millis(700),
        "a meta read waited {worst:?} behind a slow commit"
    );
    rt.block_on(post).unwrap().expect("post must still succeed");
    assert_eq!(bus.topic_record_count("t").unwrap(), 1, "read connection sees the committed post");
}

#[test]
fn topic_summaries_match_the_per_topic_reads() {
    let _g = SERIAL.lock().unwrap_or_else(|e| e.into_inner());
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Bus::open(dir.path()).unwrap();
    bus.create_topic("a", Retention::Forever).unwrap();
    bus.create_topic("b", Retention::Messages(5)).unwrap();
    let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
    for _ in 0..3 {
        rt.block_on(bus.post("a", &env("a"))).unwrap();
    }
    let got = bus.topic_summaries().unwrap();
    assert_eq!(got.len(), 2);
    for s in &got {
        assert_eq!(s.retention, bus.topic_retention(&s.name).unwrap());
        assert_eq!(s.count, bus.topic_record_count(&s.name).unwrap());
        assert_eq!(s.latest_offset, bus.latest_offset(&s.name).unwrap());
    }
    assert_eq!(got[0].name, "a");
    assert_eq!((got[0].count, got[0].latest_offset), (3, Some(2)));
    assert_eq!((got[1].count, got[1].latest_offset), (0, None));
}
