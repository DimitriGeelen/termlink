//! Charter non-goal #1 tripwire — "TermLink is NOT an inter-hub federation layer."
//!
//! T-2569 (filed by the T-2468 non-goal-guard review; built under the T-2678
//! charter guard-coverage review, finding F1).
//!
//! # Why this file exists
//!
//! `docs/CHARTER.md` non-goal #1 states that each hub owns independent topic state
//! and that cross-hub visibility is *explicit, client-driven cross-posting — never
//! automatic* (G-060). Before this test that invariant was enforced by nothing:
//! G-060 existed only as prose in doc comments, and the property held purely by
//! absence-of-feature. A future change adding an auto-relay in the hub router would
//! have tripped no test, no canary, and no lint.
//!
//! That mattered more than the usual "untested invariant", because non-goal #1 is
//! the most *assumed-away* of the five: peer projects have repeatedly believed
//! federation exists (`docs/operations/channel-topic-semantics.md` was written to
//! explain it away, and a live ring20 RCA — T-2229 — named cross-hub "federation"
//! as a root cause). The property most likely to be broken by a well-meaning change
//! was the one with no guard.
//!
//! # What "federation" means mechanically, and why these three checks catch it
//!
//! For the hub to federate it must do all three of: (a) learn a peer hub's address,
//! (b) build a client that speaks *hub* RPC, and (c) originate an outbound call to
//! it while handling inbound traffic. Each check below removes one leg:
//!
//! 1. `hub_cannot_learn_peer_hub_addresses` — the hub never reads the client-side
//!    fleet config (`hubs.toml`, `KnownHubStore`, hub profiles). It has no way to
//!    know a peer hub exists.
//! 2. `hub_never_builds_a_hub_speaking_client` — the hub never constructs a
//!    `BusClient` (the type that speaks hub RPC to a hub). This is the literal
//!    "hub-side has no BusClient-to-peer-hub call" T-2569 asked for.
//! 3. `hub_outbound_connects_are_the_known_session_forwards` — the complete set of
//!    outbound connections in non-test hub code is enumerated. Every one targets a
//!    *session registration* address (hub → its own spoke, which is the strict star
//!    working as designed), not a hub. A NEW outbound connect anywhere in the crate
//!    fails this test and forces its author to justify the addition.
//!
//! # Scope and residual risk (stated so nobody over-reads a green result)
//!
//! These are source-level invariants over the hub crate, not a runtime proof. One
//! residual path is NOT covered and cannot be by a static check: `orchestrator.route`
//! forwards to addresses drawn from session registrations, so a client that registers
//! a "session" whose `host:port` is actually a peer hub could induce a relay. That is
//! misuse of the registration surface rather than a designed federation path, and
//! closing it belongs with the per-agent authorization work (T-2422 / G-064), not here.
//!
//! # If this test fails
//!
//! Do not "fix" it by relaxing the assertion. Either the change genuinely introduces
//! cross-hub relay — in which case it contradicts a human-blessed charter non-goal and
//! needs a charter amendment first — or it adds a benign outbound call, in which case
//! extend the enumeration in check 3 with a comment saying what it targets and why it
//! is not a hub.

use std::path::{Path, PathBuf};

fn hub_src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Every `.rs` file in the hub crate's `src/`, as (relative-name, contents).
fn hub_sources() -> Vec<(String, String)> {
    let dir = hub_src_dir();
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read hub src dir {}: {e}", dir.display()));
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let name = path
            .file_name()
            .and_then(|n| n.to_str())
            .unwrap_or_default()
            .to_string();
        let body = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        out.push((name, body));
    }
    assert!(
        !out.is_empty(),
        "found no hub sources to scan — the test would vacuously pass"
    );
    out
}

/// Strip `//`-comments and `//!`/`///` doc lines so prose mentioning a forbidden
/// term (including this file's own explanations, and the G-060 notes scattered
/// through the hub) never trips a check. Only real code is scanned.
fn code_lines(body: &str) -> impl Iterator<Item = (usize, &str)> {
    body.lines().enumerate().filter_map(|(i, raw)| {
        let trimmed = raw.trim_start();
        if trimmed.starts_with("//") {
            return None;
        }
        let code = match raw.find("//") {
            Some(idx) => &raw[..idx],
            None => raw,
        };
        if code.trim().is_empty() {
            None
        } else {
            Some((i + 1, code))
        }
    })
}

#[test]
fn hub_cannot_learn_peer_hub_addresses() {
    // Leg (a): if the hub never reads the client-side fleet configuration, it has
    // no mechanism to discover that another hub exists at all. `hubs.toml` /
    // `KnownHubStore` / hub profiles are strictly client-side concepts.
    const PEER_HUB_CONFIG: &[&str] = &["hubs.toml", "KnownHubStore", "known_hubs", "HubProfile"];

    let mut found = Vec::new();
    for (file, body) in hub_sources() {
        for (line, code) in code_lines(&body) {
            for needle in PEER_HUB_CONFIG {
                if code.contains(needle) {
                    found.push(format!("{file}:{line}: {needle} — {}", code.trim()));
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "charter non-goal #1 (no inter-hub federation): the hub crate now reads \
         client-side peer-hub configuration, which is the first step toward \
         federation — it gives the hub a way to learn peer hub addresses.\n\
         Offending sites:\n  {}\n\
         See docs/CHARTER.md non-goal #1 and G-060 before changing this.",
        found.join("\n  ")
    );
}

#[test]
fn hub_never_builds_a_hub_speaking_client() {
    // Leg (b): `BusClient` is the client that speaks hub RPC (channel.post,
    // hub.auth, ...) TO a hub. The hub forwards to sessions with
    // `client::Client`, a different type speaking session RPC. Constructing a
    // BusClient inside the hub would mean the hub had become a hub's client —
    // the definition of a federation hop.
    const HUB_SPEAKING_CLIENT: &[&str] = &["BusClient", "bus_client::"];

    let mut found = Vec::new();
    for (file, body) in hub_sources() {
        for (line, code) in code_lines(&body) {
            for needle in HUB_SPEAKING_CLIENT {
                if code.contains(needle) {
                    found.push(format!("{file}:{line}: {needle} — {}", code.trim()));
                }
            }
        }
    }

    assert!(
        found.is_empty(),
        "charter non-goal #1 (no inter-hub federation): the hub crate now constructs \
         a hub-speaking client. The hub must only ever originate calls to its own \
         SESSIONS (client::Client), never to another hub.\n\
         Offending sites:\n  {}\n\
         See docs/CHARTER.md non-goal #1 and G-060 before changing this.",
        found.join("\n  ")
    );
}

#[test]
fn hub_outbound_connects_are_the_known_session_forwards() {
    // Leg (c): enumerate every outbound connection the hub can originate.
    //
    // As of T-2569 the complete non-test set is four `Client::connect_addr_raw`
    // calls, all in router.rs, all forwarding to a SESSION address:
    //   - three on the `orchestrator.route` path (local candidate, remote-session
    //     candidate, failover retry)
    //   - one in `forward_to_target`, the catch-all session forward
    //
    // `RemoteStore` entries are remote SESSIONS that registered with THIS hub via
    // `session.register_remote` — this hub's own spokes reached over TCP. Forwarding
    // to them is hub → spoke, the strict star working as designed, not a hub hop.
    //
    // Test-only `TcpStream::connect("127.0.0.1:...")` calls are excluded: they are
    // fixtures dialling the hub under test, not hub behaviour.
    const EXPECTED_OUTBOUND_SITES: usize = 4;

    let mut sites = Vec::new();
    for (file, body) in hub_sources() {
        for (line, code) in code_lines(&body) {
            if code.contains("connect_addr_raw") {
                sites.push(format!("{file}:{line}"));
            }
            // A raw TCP dial outside test fixtures would be a new, unreviewed
            // outbound path. Loopback dials in #[cfg(test)] blocks are fixtures.
            if code.contains("TcpStream::connect") && !code.contains("127.0.0.1") {
                sites.push(format!("{file}:{line} (raw TcpStream::connect)"));
            }
        }
    }

    assert_eq!(
        sites.len(),
        EXPECTED_OUTBOUND_SITES,
        "charter non-goal #1 (no inter-hub federation): the set of outbound \
         connections originated by the hub changed.\n\
         Expected {EXPECTED_OUTBOUND_SITES} known session-forward sites, found {}:\n  {}\n\
         If you added a benign outbound call to a SESSION, bump EXPECTED_OUTBOUND_SITES \
         and document what it targets. If it targets another HUB, stop — that \
         contradicts a human-blessed charter non-goal and needs a charter amendment \
         first. See docs/CHARTER.md non-goal #1 and G-060.",
        sites.len(),
        sites.join("\n  ")
    );

    assert!(
        sites.iter().all(|s| s.starts_with("router.rs")),
        "charter non-goal #1: outbound connections must originate only from the \
         router's session-forward paths, but found one elsewhere:\n  {}",
        sites.join("\n  ")
    );
}
