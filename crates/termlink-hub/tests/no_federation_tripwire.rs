//! Tripwire test for the charter rule that hubs never relay topic messages between hubs.
//! Charter non-goal #1 tripwire, narrowed by T-3643 (adopted purpose, T-3612 draft v0.9
//! §6.5): **hubs own their topic records and never relay topic messages between hubs;
//! signed directory cards are permitted.**
//!
//! T-2569 (filed by the T-2468 non-goal-guard review; built under the T-2678
//! charter guard-coverage review, finding F1). Narrowed by T-3643.
//!
//! # Why this file exists
//!
//! Each hub owns independent topic state, and cross-hub visibility of topic messages
//! is *explicit, client-driven cross-posting — never automatic* (G-060). Before T-2569
//! that invariant was enforced by nothing; the property held purely by
//! absence-of-feature. Peer projects have repeatedly believed federation exists
//! (`docs/operations/channel-topic-semantics.md`; ring20 RCA T-2229), so the property
//! most likely to be broken by a well-meaning change needs a guard.
//!
//! # What changed in T-3643, and why
//!
//! The T-2569 version forbade ALL hub-to-hub contact. The adopted purpose forbids
//! something narrower: **topic-message relay** between hubs. Hubs MAY exchange signed
//! directory cards (who lives where), which needs a hub to know a peer hub and to call
//! it. Left as it was, this test would have blocked that design while guarding the
//! wrong thing.
//!
//! No directory-card type exists in the code yet. The allowance is therefore keyed to
//! the narrowest name the draft gives: a hub module whose file name starts with
//! `directory_card` (see [`DIRECTORY_CARD_MODULE_PREFIX`]). Only such a module may
//! reach a peer hub, and even there every topic-message RPC is forbidden.
//!
//! # The four checks
//!
//! 1. `peer_hub_access_is_confined_to_directory_card_modules` — reading peer-hub
//!    config (`hubs.toml`, `KnownHubStore`, hub profiles) or building a hub-speaking
//!    client (`BusClient`) anywhere OUTSIDE a directory-card module fails. A relay in
//!    the router or any other module is caught here.
//! 2. `directory_card_modules_never_carry_topic_messages` — inside a directory-card
//!    module, naming any topic-message RPC (`channel.post`, `event.broadcast`, …) or
//!    the topic store fails. The allowance is for cards, not for messages.
//! 3. `hub_outbound_connects_are_the_known_session_forwards` — the router's four
//!    session-forward dials are pinned; any other outbound dial outside a
//!    directory-card module fails.
//! 4. `negative_cases_fire` — synthetic sources prove checks 1–3 fire on their
//!    forbidden cases and stay quiet on a card-only module.
//!
//! # Scope and residual risk (stated so nobody over-reads a green result)
//!
//! These are source-level invariants over the hub crate, not a runtime proof.
//! `orchestrator.route` forwards to addresses drawn from session registrations, so a
//! client that registers a "session" whose `host:port` is actually a peer hub could
//! induce a relay. That is misuse of the registration surface; closing it belongs with
//! the per-agent authorization work (T-2422 / G-064). Check 2 is a name scan: a
//! directory-card module that relays topic messages through an indirection that never
//! names a topic RPC or the topic store would not be seen.
//!
//! # If this test fails
//!
//! Do not "fix" it by relaxing the assertion. If the change relays topic messages
//! between hubs, it contradicts the charter and needs a charter amendment first. If it
//! is directory-card exchange, put it in a `directory_card*.rs` module. If it adds a
//! benign outbound call to a SESSION, extend the enumeration in check 3 with a comment.

use std::path::{Path, PathBuf};

/// The narrowest name the adopted draft gives for permitted hub-to-hub traffic
/// ("signed directory cards", T-3612 v0.9 §6.5). A hub module whose file name starts
/// with this prefix may reach a peer hub; no other module may.
const DIRECTORY_CARD_MODULE_PREFIX: &str = "directory_card";

/// Client-side peer-hub configuration: how a hub would learn a peer hub exists.
const PEER_HUB_CONFIG: &[&str] = &["hubs.toml", "KnownHubStore", "known_hubs", "HubProfile"];

/// The client type that speaks hub RPC TO a hub. The hub forwards to sessions with
/// `client::Client`, a different type.
const HUB_SPEAKING_CLIENT: &[&str] = &["BusClient", "bus_client::"];

/// Topic-message RPCs and the topic store. Naming any of these in a module that
/// reaches a peer hub is topic-message relay.
const TOPIC_MESSAGE_SURFACE: &[&str] = &[
    "channel.post",
    "CHANNEL_POST",
    "channel.subscribe",
    "CHANNEL_SUBSCRIBE",
    "channel.forward",
    "event.broadcast",
    "EVENT_BROADCAST",
    "event.emit",
    "EVENT_EMIT",
    "event.collect",
    "EVENT_COLLECT",
    "event.poll",
    "EVENT_POLL",
    "inbox.",
    "channel::",
    "bus::",
];

/// The router's session-forward dials (T-2569): three on `orchestrator.route` (local
/// candidate, remote-session candidate, failover retry) and one in
/// `forward_to_target`. `RemoteStore` entries are remote SESSIONS registered with THIS
/// hub, so forwarding to them is hub → spoke.
const EXPECTED_ROUTER_OUTBOUND_SITES: usize = 4;

fn hub_src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

fn is_directory_card_module(name: &str) -> bool {
    name.starts_with(DIRECTORY_CARD_MODULE_PREFIX)
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
/// term never trips a check. Only real code is scanned.
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

fn scan(sources: &[(String, String)], keep: impl Fn(&str) -> bool, needles: &[&str]) -> Vec<String> {
    let mut found = Vec::new();
    for (file, body) in sources {
        if !keep(file) {
            continue;
        }
        for (line, code) in code_lines(body) {
            for needle in needles {
                if code.contains(needle) {
                    found.push(format!("{file}:{line}: {needle} — {}", code.trim()));
                }
            }
        }
    }
    found
}

/// Check 1: peer-hub access outside a directory-card module.
fn peer_hub_access_outside_card_modules(sources: &[(String, String)]) -> Vec<String> {
    let mut found = scan(sources, |f| !is_directory_card_module(f), PEER_HUB_CONFIG);
    found.extend(scan(sources, |f| !is_directory_card_module(f), HUB_SPEAKING_CLIENT));
    found
}

/// Check 2: topic-message surface inside a directory-card module.
fn topic_relay_in_card_modules(sources: &[(String, String)]) -> Vec<String> {
    scan(sources, is_directory_card_module, TOPIC_MESSAGE_SURFACE)
}

/// Check 3: outbound dials. Returns (router session-forward sites, stray sites).
/// Directory-card modules are excluded: their dials are confined by checks 1 and 2.
fn outbound_sites(sources: &[(String, String)]) -> (Vec<String>, Vec<String>) {
    let mut router = Vec::new();
    let mut strays = Vec::new();
    for (file, body) in sources {
        if is_directory_card_module(file) {
            continue;
        }
        for (line, code) in code_lines(body) {
            let dial = code.contains("connect_addr_raw")
                // Loopback dials are #[cfg(test)] fixtures dialling the hub under test.
                || (code.contains("TcpStream::connect") && !code.contains("127.0.0.1"));
            if !dial {
                continue;
            }
            let site = format!("{file}:{line}: {}", code.trim());
            if file == "router.rs" && code.contains("connect_addr_raw") {
                router.push(site);
            } else {
                strays.push(site);
            }
        }
    }
    (router, strays)
}

#[test]
fn peer_hub_access_is_confined_to_directory_card_modules() {
    let found = peer_hub_access_outside_card_modules(&hub_sources());
    assert!(
        found.is_empty(),
        "charter non-goal #1 (no topic-message relay between hubs): a hub module that \
         is not a `{DIRECTORY_CARD_MODULE_PREFIX}*.rs` module reads peer-hub \
         configuration or builds a hub-speaking client.\n\
         Offending sites:\n  {}\n\
         Only signed directory-card exchange may reach a peer hub (T-3612 §6.5, T-3643). \
         Topic messages must never be relayed between hubs; see docs/CHARTER.md and G-060.",
        found.join("\n  ")
    );
}

#[test]
fn directory_card_modules_never_carry_topic_messages() {
    let found = topic_relay_in_card_modules(&hub_sources());
    assert!(
        found.is_empty(),
        "charter non-goal #1 (no topic-message relay between hubs): a directory-card \
         module names a topic-message RPC or the topic store. Directory cards say who \
         lives where; they never carry topic messages.\n\
         Offending sites:\n  {}",
        found.join("\n  ")
    );
}

#[test]
fn hub_outbound_connects_are_the_known_session_forwards() {
    let (router, strays) = outbound_sites(&hub_sources());
    assert_eq!(
        router.len(),
        EXPECTED_ROUTER_OUTBOUND_SITES,
        "charter non-goal #1: the router's session-forward dials changed.\n\
         Expected {EXPECTED_ROUTER_OUTBOUND_SITES}, found {}:\n  {}\n\
         If you added a benign forward to a SESSION, bump the constant and document \
         what it targets. If it targets another HUB to carry topic messages, stop: that \
         contradicts the charter.",
        router.len(),
        router.join("\n  ")
    );
    assert!(
        strays.is_empty(),
        "charter non-goal #1: outbound connections must originate only from the \
         router's session-forward paths (or a `{DIRECTORY_CARD_MODULE_PREFIX}*.rs` \
         module), but found:\n  {}",
        strays.join("\n  ")
    );
}

/// Negative and positive cases on synthetic sources, so a green run above means the
/// checks can fire, not merely that they found nothing.
#[test]
fn negative_cases_fire() {
    let src = |name: &str, body: &str| vec![(name.to_string(), body.to_string())];

    // A relay built in the router: a hub-speaking client posting a topic message.
    let relay_in_router = src(
        "router.rs",
        "async fn relay(m: Msg) {\n    let c = BusClient::connect(peer).await?;\n    c.call(CHANNEL_POST, m).await?;\n}\n",
    );
    assert!(
        !peer_hub_access_outside_card_modules(&relay_in_router).is_empty(),
        "check 1 must fire on a BusClient built outside a directory-card module"
    );

    // A peer-hub address learned in an ordinary module.
    let config_in_server = src("server.rs", "fn peers() { let s = KnownHubStore::default_store(); }\n");
    assert!(
        !peer_hub_access_outside_card_modules(&config_in_server).is_empty(),
        "check 1 must fire on peer-hub config read outside a directory-card module"
    );

    // A directory-card module that smuggles a topic message: the forbidden case.
    let relay_in_card = src(
        "directory_card.rs",
        "async fn exchange(peer: &str) {\n    let c = BusClient::connect(peer).await?;\n    c.call(\"channel.post\", msg).await?;\n}\n",
    );
    assert!(
        !topic_relay_in_card_modules(&relay_in_card).is_empty(),
        "check 2 must fire on a topic-message RPC inside a directory-card module"
    );
    let store_in_card = src("directory_card_sync.rs", "fn f() { crate::channel::append(&topic, m); }\n");
    assert!(
        !topic_relay_in_card_modules(&store_in_card).is_empty(),
        "check 2 must fire on the topic store named inside a directory-card module"
    );

    // A card-only exchange: permitted by every check.
    let card_only = src(
        "directory_card.rs",
        "async fn exchange(peer: &str, card: SignedCard) {\n    let c = BusClient::connect(peer).await?;\n    let s = KnownHubStore::default_store();\n    c.call(\"hub.directory_card\", card.verify()?).await?;\n    let _t = TcpStream::connect(peer).await?;\n}\n",
    );
    assert!(peer_hub_access_outside_card_modules(&card_only).is_empty(), "card-only exchange must pass check 1");
    assert!(topic_relay_in_card_modules(&card_only).is_empty(), "card-only exchange must pass check 2");
    assert!(outbound_sites(&card_only).1.is_empty(), "card-only dial must pass check 3");

    // A raw dial outside the router and outside a card module.
    let stray_dial = src("server.rs", "async fn f(a: &str) { let _s = TcpStream::connect(a).await; }\n");
    assert!(
        !outbound_sites(&stray_dial).1.is_empty(),
        "check 3 must fire on a raw dial outside the router"
    );

    // Comments never trip a check.
    let prose = src("router.rs", "// BusClient would relay channel.post here — never do this\nfn ok() {}\n");
    assert!(peer_hub_access_outside_card_modules(&prose).is_empty(), "comments must not trip check 1");
}
