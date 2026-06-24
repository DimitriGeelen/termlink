---
id: T-1677
name: "Client::connect_addr_with_timeout primitive + migrate operator-facing remote-hub sites (T-1675 follow-up)"
description: >
  Client::connect_addr_with_timeout primitive + migrate operator-facing remote-hub sites (T-1675 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, crates/termlink-session/src/client.rs]
related_tasks: []
created: 2026-05-17T21:47:53Z
last_update: 2026-05-17T21:56:47Z
date_finished: 2026-05-17T21:56:47Z
---

# T-1677: Client::connect_addr_with_timeout primitive + migrate operator-facing remote-hub sites (T-1675 follow-up)

## Context

T-1675 fixed the `probe_cert` unbounded-TCP-wait at 8 detection-verb call sites by introducing `probe_cert_with_timeout`. Same disease, different surface: `termlink_session::client::Client::connect_addr` (the canonical TLS client path) does `tokio::net::TcpStream::connect(...).await` with no timeout (crates/termlink-session/src/client.rs:30). Every operator-facing remote-hub command that goes through `connect_remote_hub` (`termlink remote call/exec/ping`, `termlink channel post/list`, `termlink file send/receive`) inherits that 30-60s OS TCP retry budget when the target is unreachable. Live evidence is the same as T-1675's: a single unreachable laptop-141 stretches wall-time from ~10s to 60-130s.

Fix mirrors T-1675 — add `Client::connect_addr_with_timeout(addr, dur)` as the primitive, then migrate the operator-facing CLI call sites that aren't already wrapped in `tokio::time::timeout` at their caller. Out of scope: hub-internal `connect_addr_raw` (localhost only, no remote-TCP wait), session-level helpers in `inbox_channel.rs` (caller doesn't expose a flag yet), MCP variants (already pre-validate reachability in some paths — separate audit). Default timeout 10s, matching T-1675's `fleet doctor`/`fleet verify` default.

## Acceptance Criteria

### Agent
- [x] `Client::connect_addr_with_timeout(addr: &TransportAddr, timeout: Duration) -> io::Result<Self>` added to `termlink-session/src/client.rs`, wraps `connect_addr` in `tokio::time::timeout`, returns `io::Error::new(TimedOut, ...)` on expiry
- [x] `connect_remote_hub` in `crates/termlink-cli/src/commands/remote.rs` (line ~774) uses the timeout primitive with `Duration::from_secs(10)`
- [x] Live smoke: `time termlink remote ping <unreachable-host>` exits in <12s with a clean error (not 30-60s hang) — verified against blackhole `10.255.255.1:9100`, elapsed 10s with msg `client connect to ... timeout after 10s`
- [x] `cargo check --workspace` passes
- [x] `cargo test -p termlink-session --lib client::tests::connect_addr_with_timeout` passes (unit test using RFC 5737 TEST-NET-1 `192.0.2.1:9100`)

## Verification

cargo check --workspace
bash -c "grep -q 'connect_addr_with_timeout' crates/termlink-session/src/client.rs"
bash -c "grep -q 'connect_addr_with_timeout' crates/termlink-cli/src/commands/remote.rs"
cargo test -p termlink-session --lib connect_addr_with_timeout 2>&1 | grep -qE "test result: ok\. [1-9]"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-05-17T21:47:53Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1677-clientconnectaddrwithtimeout-primitive--.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-9a583a73
- **Timestamp:** 2026-05-17T21:56:58Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T21:56:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
