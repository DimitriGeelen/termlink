//! T-2703 — tripwire for the architecture doc's decisive invariant: **spokes never
//! connect to one another**.
//!
//! # What this guards, and what T-2569 does NOT
//!
//! `docs/architecture/parallel-execution-substrate.md` §10 lists five invariants that
//! "must not be violated". The first is *"Strict star; no peer-to-peer surfaces"*, and
//! §3 states it plainly: **"The hub mediates all coordination; spokes never connect to
//! one another."**
//!
//! It is easy to assume `termlink-hub/tests/no_federation_tripwire.rs` (T-2569) already
//! covers this. It does not — it guards a **different edge**. That test scans only
//! `termlink-hub/src` and forbids the *hub* from building a hub-speaking client, i.e.
//! hub↔hub **federation** (charter non-goal #1). The invariant here is spoke↔spoke
//! **mesh**, on the client side, and until T-2702 nothing guarded it at all.
//!
//! # Why the mesh is forbidden (§3's decisive argument)
//!
//! §3 spends forty lines rejecting an agent-to-agent mesh and names the third argument
//! decisive — and it is worth restating here, because "just open a direct channel
//! between these two agents" will look like an obvious optimisation to someone who has
//! not read it:
//!
//! A star has *one* failure point: visible, diagnosable, and — because channel logs and
//! the inbox spool are durable — **recoverable**. A hub crash is a pause, not data loss.
//! A mesh distributes fragility across up to N² links, each with independent liveness
//! state, with **no central durable replay** (a dropped direct message is simply gone)
//! and, worst, **silent partial-partition divergence**: A↔B and C↔B survive while A and
//! C hold inconsistent coordination state with no authority to reconcile against. On a
//! flaky homelab, partial partitions are the *normal* failure mode.
//!
//! The efficiency a mesh reaches for is already in the star: a persistent shared topic
//! is about as cheap as a direct socket and stays durable, ordered, hub-visible, and
//! survives either party dying.
//!
//! # The structural property this pins
//!
//! Every production connection in this crate is constructed in one of **four
//! transport/probe modules**, each with a documented purpose (see `TRANSPORT_MODULES`).
//! That matters more than the raw count: it means there is a small, fixed set of files
//! to audit when asking "what can this process dial?". A mesh would appear either as a
//! new connect site inside one of them (caught by check 2) or — far more likely — as a
//! socket opened somewhere else entirely, a presence or dispatch module deciding to
//! reach a peer directly (caught by check 1).
//!
//! *Corrected during construction (T-2703).* This tripwire was first written asserting
//! that all connects live in `client.rs` alone. That was wrong: the premise came from a
//! `grep` truncated at ten results, and the check failed on its first run against
//! `transport.rs`, `tofu.rs` and `ws_consumer.rs`. The enumeration below is the real
//! surface, which is a stronger invariant than the guess — it is the actual answer to
//! "what can dial out from a spoke".
//!
//! # What is explicitly LEGAL and must not trip this
//!
//! * **spoke → hub.** The star requires it. This test bans neither outbound
//!   connections nor TCP.
//! * **an operator tool → a LOCAL session's control plane.** §3 forbids *agents
//!   meshing with each other*; a CLI reaching a session on the same host over a unix
//!   socket is not that. Banning it would be a false positive that makes the guard
//!   unusable, which is how guards get deleted.
//! * **the hub router → a local proxy** (`connect_addr_raw`). The hub is still the
//!   mediator; that is the star working, not a bypass of it.
//!
//! # Scope and residual risk (stated so nobody over-reads a green result)
//!
//! This is a source-level invariant over one crate, not a runtime proof.
//!
//! * It cannot tell *where a connection is pointed at run time*. `client.rs` takes a
//!   `TransportAddr`; a caller that passes another spoke's address would still mesh.
//!   Closing that needs the connection target to be authorized, which is the per-agent
//!   authorization work (T-2422 / G-064), not a static check.
//! * It scans `termlink-session` only. `termlink-cli` also dials sockets; a mesh
//!   introduced purely there would be missed. That crate is the operator surface, where
//!   local-session dialling is legitimate, so a naive extension would false-positive —
//!   it needs its own predicate, filed rather than bolted on here.
//!
//! # If this test fails
//!
//! Do not relax the assertion. Either the change genuinely opens a spoke-to-spoke
//! channel — in which case it contradicts an invariant the architecture doc calls
//! decisive, and needs that document amended first — or it adds a benign connection, in
//! which case extend the enumeration below with a comment saying what it dials and why
//! that target is the hub, a local session, or a proxy rather than a peer spoke.

use std::path::{Path, PathBuf};

/// The modules allowed to construct connections, with their site count and what each
/// one dials. Every entry was read and confirmed during T-2703 — none dials a peer
/// spoke.
///
/// * **`client.rs` (5)** — the RPC client.
///   `connect_addr` Unix → a session control plane on THIS host (operator surface);
///   `connect_addr` Tcp → the hub; `connect_tls_stream` Tcp → the hub over TLS;
///   `connect_addr_raw` Unix/Tcp → hub-router forwarding to a LOCAL proxy (its own doc
///   says "the target is a local proxy, not an external TLS-speaking hub" — the hub is
///   still the mediator, which is the star working).
/// * **`transport.rs` (3)** — the `Transport` trait impls (`UnixTransport::connect`,
///   `TcpTransport::connect`) plus a synchronous `connect_timeout` liveness probe.
///   Generic plumbing: it dials whatever address it is handed, so the *target* is the
///   caller's responsibility (see residual risk above).
/// * **`tofu.rs` (1)** — a one-shot TCP connect to capture a HUB's TLS leaf certificate
///   for TOFU pinning (`hub probe`). Spoke → hub.
/// * **`ws_consumer.rs` (1)** — the WebSocket consumer's Unix arm, connecting to the
///   hub's socket. Spoke → hub.
const TRANSPORT_MODULES: &[(&str, usize)] = &[
    ("client.rs", 5),
    ("transport.rs", 3),
    ("tofu.rs", 1),
    ("ws_consumer.rs", 1),
];

fn is_transport_module(name: &str) -> bool {
    TRANSPORT_MODULES.iter().any(|(m, _)| *m == name)
}

fn session_src_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("src")
}

/// Remove `#[cfg(test)]` items by BRACE-COUNTING their body, keeping everything after.
///
/// Unit tests legitimately dial scratch sockets to exercise the transport, so they must
/// not be mistaken for production mesh paths — without this the check fires on
/// `auth.rs` and `data_server.rs`, whose only connects are in their test modules.
///
/// **Why not just truncate at the first `#[cfg(test)]`.** That was this file's first
/// implementation and it was wrong: `discovery.rs` puts its test module at line 89 of
/// 176, so truncating discarded *half the file unscanned*. The flaw was caught by the
/// load-bearing probe — a simulated peer-to-peer connect appended to `discovery.rs`
/// did not trip the tripwire, because it landed after the cut. A guard that silently
/// reads less than it claims is the exact failure mode this review series keeps
/// finding elsewhere; skipping only the guarded item's body is the honest fix.
fn strip_test_modules(full: &str) -> String {
    let mut out = String::new();
    let mut lines = full.lines();
    while let Some(line) = lines.next() {
        if !line.trim_start().starts_with("#[cfg(test)]") {
            out.push_str(line);
            out.push('\n');
            continue;
        }
        // Skip the guarded item: consume until its braces balance.
        let mut depth: i32 = 0;
        let mut opened = false;
        for inner in lines.by_ref() {
            depth += inner.matches('{').count() as i32;
            depth -= inner.matches('}').count() as i32;
            if inner.contains('{') {
                opened = true;
            }
            if opened && depth <= 0 {
                break;
            }
        }
    }
    out
}

/// Every `.rs` file in this crate's `src/`, as (file-name, production-only body).
fn session_sources() -> Vec<(String, String)> {
    let dir = session_src_dir();
    let mut out = Vec::new();
    let entries = std::fs::read_dir(&dir)
        .unwrap_or_else(|e| panic!("cannot read session src dir {}: {e}", dir.display()));
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
        let full = std::fs::read_to_string(&path)
            .unwrap_or_else(|e| panic!("cannot read {}: {e}", path.display()));
        out.push((name, strip_test_modules(&full)));
    }
    assert!(
        !out.is_empty(),
        "found no session sources to scan — the test would vacuously pass"
    );
    out
}

/// Strip `//` comments and doc lines so prose mentioning a connect (including this
/// file's own explanations) never trips a check. Only real code is scanned.
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

fn is_connect(line: &str) -> bool {
    line.contains("UnixStream::connect") || line.contains("TcpStream::connect")
}

/// CHECK 1 — connection construction stays in the transport module.
///
/// This is the load-bearing half. A spoke-to-spoke channel is overwhelmingly likely to
/// appear as a socket opened *somewhere else* — a presence or dispatch module deciding
/// to reach a peer directly — rather than as a sixth arm inside the transport layer.
#[test]
fn connections_are_constructed_only_in_transport_modules() {
    let mut strays = Vec::new();
    for (name, body) in session_sources() {
        if is_transport_module(&name) {
            continue;
        }
        for (lineno, line) in code_lines(&body) {
            if is_connect(line) {
                strays.push(format!("{name}:{lineno}: {}", line.trim()));
            }
        }
    }
    assert!(
        strays.is_empty(),
        "spoke↔spoke mesh tripwire (T-2703): connection(s) constructed outside the \
         enumerated transport modules:\n  {}\n\n\
         The architecture doc's §3 invariant is \"spokes never connect to one another\", \
         and §10 lists it as must-not-be-violated. Production connections in this crate \
         are confined to a small fixed set of transport/probe modules so there is a \
         bounded surface to audit when asking what this process can dial.\n\
         If the new site dials the HUB, a LOCAL session control plane, or a hub-router \
         proxy, add its module to TRANSPORT_MODULES with a comment naming the target. \
         If it dials a PEER SPOKE, it contradicts an invariant the architecture doc \
         calls decisive — amend that document first.",
        strays.join("\n  ")
    );
}

/// CHECK 2 — the transport module's connect sites are enumerated.
///
/// Pins the count so a new arm inside the chokepoint is a deliberate act with a
/// comment, not a quiet addition.
#[test]
fn transport_module_connect_sites_are_enumerated() {
    let sources = session_sources();
    for (module, expected) in TRANSPORT_MODULES {
        let (_, body) = sources
            .iter()
            .find(|(n, _)| n == module)
            .unwrap_or_else(|| panic!("{module} not found — has a transport module moved or been renamed?"));

        let sites: Vec<String> = code_lines(body)
            .filter(|(_, line)| is_connect(line))
            .map(|(lineno, line)| format!("{lineno}: {}", line.trim()))
            .collect();

        assert_eq!(
            sites.len(),
            *expected,
            "spoke↔spoke mesh tripwire (T-2703): {module} has {} connect site(s), \
             expected {expected}:\n  {}\n\n\
             A new outbound connection is not automatically wrong — but it must be a \
             deliberate act. Confirm the target is the hub, a local session control \
             plane, or a hub-router proxy (NOT a peer spoke), then update this module's \
             count in TRANSPORT_MODULES and the comment describing what it dials.",
            sites.len(),
            sites.join("\n  ")
        );
    }
}

/// CHECK 3 — the scan is not vacuous.
///
/// A tripwire that silently stops finding anything is worse than no tripwire: it
/// reports green forever. This asserts the transport module is still being read.
#[test]
fn tripwire_is_not_vacuous() {
    let sources = session_sources();
    let total: usize = TRANSPORT_MODULES
        .iter()
        .map(|(module, _)| {
            sources
                .iter()
                .find(|(n, _)| n == module)
                .map(|(_, body)| code_lines(body).filter(|(_, l)| is_connect(l)).count())
                .unwrap_or(0)
        })
        .sum();
    assert!(
        total > 0,
        "found zero connect sites across every transport module — the scan is reading \
         nothing, so checks 1 and 2 would pass vacuously"
    );
}
