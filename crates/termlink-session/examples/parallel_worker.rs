//! Runnable demo of the arc-parallel-substrate claim primitive (T-2018, T-2031).
//!
//! Spawns N worker tasks that race to claim sequential offsets on a topic.
//! Each successful claim is "processed" (sleep 100ms) then ack'd; conflicting
//! claims (CLAIM_CONFLICT) are counted and skipped. The summary at the end
//! shows exclusive-delivery in action — `total_wins` equals the number of
//! distinct offsets attempted, and `total_conflicts` shows how often workers
//! raced for the same slot.
//!
//! ## Run
//!
//! ```text
//! # Start a local hub first (uses default runtime_dir):
//! termlink hub start &
//!
//! # Create a topic and add some posts so workers have offsets to claim:
//! termlink channel create demo-work
//! for i in $(seq 1 20); do termlink channel post demo-work "item-$i"; done
//!
//! # Run the demo against the local hub socket:
//! cargo run --release --example parallel_worker -- \
//!     /tmp/termlink-0/hub.sock demo-work 4 20
//! ```
//!
//! Args: `<hub-socket-path> <topic> [worker-count=4] [offset-count=10]`
//!
//! ## Expected output
//!
//! ```text
//! parallel_worker: spawning 4 workers, claiming offsets 0..20 on topic 'demo-work'
//! [worker-0] won claim on offset=0 (claim_id=clm-...)
//! [worker-2] conflict on offset=0 (already claimed) — skipping
//! [worker-1] won claim on offset=1 (claim_id=clm-...)
//! ...
//! ────────────────────────────────────────
//! parallel_worker summary
//!   worker-0: wins=5 conflicts=0
//!   worker-1: wins=5 conflicts=2
//!   worker-2: wins=5 conflicts=3
//!   worker-3: wins=5 conflicts=2
//!   ─────
//!   total_wins=20  total_conflicts=7
//!   each offset processed exactly once (exclusive-delivery)
//! ```
//!
//! ## What this demonstrates
//!
//! - `LeasedClaim::acquire` is the easy in-process shape: acquire +
//!   auto-renew at ttl/2 + Drop fires fire-and-forget nack release.
//! - CLAIM_CONFLICT is the structured "someone else got there first"
//!   signal — the worker doesn't crash, it just skips.
//! - With N workers and M offsets, total_wins == M; total_conflicts grows
//!   with worker contention (more workers = more wasted attempts but
//!   faster overall throughput on a high-latency processor).
//!
//! See `docs/operations/substrate-claim-primitive.md` for the full
//! recipe + diagnostic runbook.

use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Arc;
use std::time::Duration;

use termlink_protocol::TransportAddr;
use termlink_session::{ClaimError, LeasedClaim};

const DEFAULT_WORKERS: usize = 4;
const DEFAULT_OFFSETS: u64 = 10;
const CLAIM_TTL_MS: u32 = 30_000;
const FAKE_WORK_MS: u64 = 100;

#[tokio::main(flavor = "multi_thread")]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    if args.len() < 3 {
        eprintln!(
            "Usage: {} <hub-socket-path> <topic> [worker-count={}] [offset-count={}]",
            args[0], DEFAULT_WORKERS, DEFAULT_OFFSETS
        );
        std::process::exit(2);
    }
    let socket = args[1].clone();
    let topic = args[2].clone();
    let worker_count: usize = args
        .get(3)
        .map(|s| s.parse().expect("worker count must be a positive integer"))
        .unwrap_or(DEFAULT_WORKERS);
    let offset_count: u64 = args
        .get(4)
        .map(|s| s.parse().expect("offset count must be a positive integer"))
        .unwrap_or(DEFAULT_OFFSETS);

    println!(
        "parallel_worker: spawning {worker_count} workers, claiming offsets 0..{offset_count} on topic '{topic}'"
    );

    let addr = TransportAddr::unix(&socket);
    // Shared "next offset" pointer — workers atomically claim a new offset
    // index, then race the hub for it. Workers that lose the race observe
    // CLAIM_CONFLICT and increment their conflict counter.
    let next_offset = Arc::new(AtomicU64::new(0));

    let mut handles = Vec::with_capacity(worker_count);
    for worker_id in 0..worker_count {
        let claimer = format!("worker-{worker_id}");
        let topic = topic.clone();
        let addr = addr.clone();
        let next_offset = next_offset.clone();
        handles.push(tokio::spawn(async move {
            run_worker(worker_id, claimer, addr, topic, next_offset, offset_count).await
        }));
    }

    let mut total_wins = 0u64;
    let mut total_conflicts = 0u64;
    let mut tallies = Vec::with_capacity(worker_count);
    for h in handles {
        let tally = h.await.unwrap_or_else(|e| {
            eprintln!("worker task panicked: {e}");
            WorkerTally::default()
        });
        total_wins += tally.wins;
        total_conflicts += tally.conflicts;
        tallies.push(tally);
    }

    println!("────────────────────────────────────────");
    println!("parallel_worker summary");
    for (i, t) in tallies.iter().enumerate() {
        println!("  worker-{i}: wins={} conflicts={}", t.wins, t.conflicts);
    }
    println!("  ─────");
    println!("  total_wins={total_wins}  total_conflicts={total_conflicts}");
    if total_wins == offset_count {
        println!("  each offset processed exactly once (exclusive-delivery)");
    } else {
        println!(
            "  NOTE: total_wins ({total_wins}) != offset_count ({offset_count}) — \
             some offsets were probably missing from the topic. Make sure the topic has \
             at least {offset_count} posts before running."
        );
    }
}

#[derive(Default, Clone, Copy)]
struct WorkerTally {
    wins: u64,
    conflicts: u64,
}

async fn run_worker(
    worker_id: usize,
    claimer: String,
    addr: TransportAddr,
    topic: String,
    next_offset: Arc<AtomicU64>,
    offset_count: u64,
) -> WorkerTally {
    let mut tally = WorkerTally::default();
    loop {
        let offset = next_offset.fetch_add(1, Ordering::Relaxed);
        if offset >= offset_count {
            break;
        }
        match LeasedClaim::acquire(addr.clone(), &topic, offset, &claimer, CLAIM_TTL_MS).await {
            Ok(lease) => {
                println!(
                    "[worker-{worker_id}] won claim on offset={offset} (claim_id={})",
                    lease.claim_id()
                );
                // Simulate work. The lease auto-renews at ttl/2 in the background,
                // so long sleeps wouldn't lose the slot — see LeasedClaim docs.
                tokio::time::sleep(Duration::from_millis(FAKE_WORK_MS)).await;
                match lease.ack().await {
                    Ok(_) => tally.wins += 1,
                    Err(e) => eprintln!(
                        "[worker-{worker_id}] ack failed on offset={offset}: {e}"
                    ),
                }
            }
            Err(ClaimError::Conflict { .. }) => {
                tally.conflicts += 1;
                println!(
                    "[worker-{worker_id}] conflict on offset={offset} (already claimed) — skipping"
                );
            }
            Err(e) => {
                eprintln!(
                    "[worker-{worker_id}] claim failed on offset={offset}: {e}"
                );
            }
        }
    }
    tally
}
