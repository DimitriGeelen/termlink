---
id: T-1658
name: "hub probe — TLS handshake to <addr>, print remote cert fingerprint (pre-pin, no auth)"
description: >
  hub probe — TLS handshake to <addr>, print remote cert fingerprint (pre-pin, no auth)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [auth, G-011, rotation-protocol, cli, tls]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/main.rs, crates/termlink-session/src/tofu.rs]
related_tasks: [T-1656, T-1657, T-1051, T-1052]
created: 2026-05-17T15:35:26Z
last_update: 2026-05-17T15:46:00Z
date_finished: 2026-05-17T15:46:00Z
---

# T-1658: hub probe — TLS handshake to <addr>, print remote cert fingerprint (pre-pin, no auth)

## Context

Completes the G-011 rotation-protocol primitive set. T-1656 reads the live
local secret; T-1657 reads the live local cert fingerprint. Both are LOCAL
sources of truth — they require shell access to the hub host. There is no
symmetric REMOTE primitive: an operator who wants to verify "is the hub at
192.168.10.122:9100 still pinned to the fingerprint I expect?" must either
trust `fleet doctor` (which only flares after an auth-mismatch already
broke a workload) or shell into the host and run `hub fingerprint`.

`hub probe <addr>` closes the loop: open TCP, complete TLS handshake
accepting any cert, extract the leaf cert DER, print `sha256:<64-hex>` in
the same canonical form as `tofu list` and `hub fingerprint`. No auth, no
profile, no HMAC. Operators can:
- pre-pin verify before adding a profile
- compare-without-trust against KnownHubStore.get(addr) after a suspected rotation
- diagnose "is the hub even up and presenting a cert" without an HMAC dance

Implementation reuses the existing `tokio_rustls` + `cert_fingerprint`
machinery in `termlink_session::tofu`. A throwaway `ProbeVerifier` accepts
every cert and stashes the DER for the caller — no persistence, no
KnownHubStore mutation.

## Acceptance Criteria

### Agent
- [x] CLI surface added: `HubAction::Probe { addr: String, json: bool }` in `crates/termlink-cli/src/cli.rs`
- [x] Dispatch wired in `crates/termlink-cli/src/main.rs` to `commands::infrastructure::cmd_hub_probe`
- [x] `cmd_hub_probe(addr, json)` in `crates/termlink-cli/src/commands/infrastructure.rs`:
  - Parses `addr` (host:port; accepts IPv4 + DNS) — actually delegated to `tofu::probe_cert` which splits + validates
  - Opens TCP connection
  - Runs TLS handshake with a `ProbeVerifier` that accepts all certs and captures the leaf cert DER
  - Computes `sha256:<hex>` via `termlink_session::tofu::cert_fingerprint`
  - Plain mode: prints fingerprint to stdout
  - `--json` mode: `{"address":"...","fingerprint":"sha256:..."}`
  - Does NOT mutate `KnownHubStore`
  - Refuses to connect to obviously-malformed addresses with an actionable error
- [x] `cargo build --release -p termlink` succeeds (1m 48s, clean)
- [x] Live smoke: `termlink hub probe 127.0.0.1:9100` returns
  `sha256:d1bd50f5cb03c4fd11689b77c4d9d6a3d6f8f83ff947a23c2dd586c43abb359f`
  which matches the local `hub fingerprint` output exactly. Cross-verified
  against `tofu list` for both local (`d1bd50f5cb0...` at 192.168.10.107:9100)
  and remote ring20-management (`22c19fedafd...` at 192.168.10.122:9100) —
  remote probe over the wire returns the same value KnownHubStore has pinned.
  Error paths smoke: malformed addr (`localhost`) returns actionable
  "expected host:port" message; unreachable port (127.0.0.1:1) returns
  "TCP connect ... Connection refused".

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
-->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

cargo check -p termlink
grep -q "Probe {" crates/termlink-cli/src/cli.rs
grep -q "cmd_hub_probe" crates/termlink-cli/src/commands/infrastructure.rs
grep -q "HubAction::Probe" crates/termlink-cli/src/main.rs

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

### 2026-05-17T15:35:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1658-hub-probe--tls-handshake-to-addr-print-r.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-1db04ccd
- **Timestamp:** 2026-05-17T15:46:09Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T15:46:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
