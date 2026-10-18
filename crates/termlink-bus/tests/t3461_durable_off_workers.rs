//! T-3461: a slow durable step (meta commit, payload fsync) must not park the
//! tokio workers. Before the fix, every meta commit ran on a worker under one
//! std mutex; a multi-second btrfs fsync parked all workers and the hub's
//! accept loop went silent for 43-57 s.
//!
//! Own test binary on purpose: the slow-durable seam is process-global.

use std::sync::Arc;
use std::time::{Duration, Instant};

use termlink_bus::{Bus, Envelope, Retention, set_test_slow_durable_ms};

fn env(topic: &str) -> Envelope {
    Envelope {
        topic: topic.into(),
        sender_id: "s".into(),
        msg_type: "note".into(),
        payload: b"x".to_vec(),
        artifact_ref: None,
        ts_unix_ms: 0,
        metadata: std::collections::BTreeMap::new(),
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn slow_durable_step_does_not_park_the_workers() {
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Arc::new(Bus::open(dir.path()).unwrap());
    for t in 0..4 {
        bus.create_topic(&format!("t{t}"), Retention::Forever).unwrap();
    }

    // Four posts, more than there are workers, each stuck on a 1.5 s "fsync".
    set_test_slow_durable_ms(1500);
    let posts: Vec<_> = (0..4)
        .map(|t| {
            let bus = bus.clone();
            tokio::spawn(async move { bus.post(&format!("t{t}"), &env(&format!("t{t}"))).await })
        })
        .collect();
    // Wall-clock throughout, never the tokio timer: with every worker parked the
    // timer driver stops too, so `tokio::time::{sleep,timeout}` would wait as long
    // as the stall and the test could not tell (the T-3461 mutant check caught
    // exactly that). This thread runs `block_on`, not a worker, so blocking it is fine.
    std::thread::sleep(Duration::from_millis(200));

    // Stand-in for the accept loop: an unrelated task must still get a worker.
    let (tx, rx) = std::sync::mpsc::channel();
    let started = Instant::now();
    tokio::spawn(async move {
        let _ = tx.send(42);
    });
    let got = rx.recv_timeout(Duration::from_millis(700));
    let waited = started.elapsed();
    set_test_slow_durable_ms(0);

    assert!(
        got == Ok(42),
        "an unrelated task waited {waited:?} for a worker while posts sat in a slow durable step"
    );
    for p in posts {
        p.await.unwrap().expect("post must still succeed");
    }
}

#[test]
fn meta_db_runs_wal_with_full_sync() {
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Bus::open(dir.path()).unwrap();
    bus.create_topic("t", Retention::Forever).unwrap();
    drop(bus);
    // SQLite file header bytes 18/19 (write/read format version) are 2 in WAL
    // mode and 1 for a rollback journal. synchronous=FULL is per connection and
    // is asserted inside the crate (meta.rs tests).
    let b = std::fs::read(dir.path().join("meta.db")).unwrap();
    assert_eq!((b[18], b[19]), (2, 2), "meta.db is not in WAL mode");
}

/// Measurement, not a gate (run with `--ignored --nocapture`): what a durable
/// post and a snapshot read cost on this host's disk. T-3461 "Open": flush cost.
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[ignore]
async fn measure_post_and_read_latency() {
    let dir = tempfile::TempDir::new().unwrap();
    let bus = Bus::open(dir.path()).unwrap();
    bus.create_topic("m", Retention::Forever).unwrap();
    let pct = |mut v: Vec<Duration>, p: usize| {
        v.sort();
        v[(v.len() * p / 100).min(v.len() - 1)]
    };
    let mut posts = Vec::new();
    for _ in 0..200 {
        let t = Instant::now();
        bus.post("m", &env("m")).await.unwrap();
        posts.push(t.elapsed());
    }
    let mut reads = Vec::new();
    for _ in 0..200 {
        let t = Instant::now();
        let _ = bus.latest_offset("m").unwrap();
        reads.push(t.elapsed());
    }
    println!(
        "post p50={:?} p99={:?} | read p50={:?} p99={:?}",
        pct(posts.clone(), 50),
        pct(posts, 99),
        pct(reads.clone(), 50),
        pct(reads, 99)
    );
}
