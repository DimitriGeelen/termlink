---
id: T-1458
name: "Fix hub.version RPC — returns hardcoded 0.9.0 instead of actual build version"
description: >
  Fix hub.version RPC — returns hardcoded 0.9.0 instead of actual build version

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: [crates/termlink-hub/build.rs, scripts/check-vendored-arc-rollout.sh]
related_tasks: []
created: 2026-05-03T20:56:57Z
last_update: 2026-05-03T22:01:30Z
date_finished: 2026-05-03T22:01:30Z
---

# T-1458: Fix hub.version RPC — returns hardcoded 0.9.0 instead of actual build version

## Context

`fleet doctor` reports `Versions in fleet: 0.9.0 (5 hubs)` for every hub, even though all field hubs are running 0.9.17xx binaries. Source: `crates/termlink-hub/src/router.rs::handle_hub_version` (line 884) returns `env!("CARGO_PKG_VERSION")`, which resolves to the workspace Cargo.toml's hardcoded `version = "0.9.0"` at compile time.

The CLI uses `crates/termlink-cli/build.rs` to derive a git-based version string (e.g. 0.9.1702) injected via env var. The hub crate has no equivalent — its version is whatever Cargo.toml says, which is permanently "0.9.0".

Verified 2026-05-03T20:55Z:
- `termlink remote exec ring20-management <session> 'termlink --version'` → `termlink 0.9.1702`
- `fleet doctor --json` for same hub → `"hub_version": "0.9.0"`

Operator impact: T-1166 cut-readiness depends on knowing what's deployed. Fleet-doctor's version field is silently wrong, so cut decisions can't trust it.

## Acceptance Criteria

### Agent
- [x] `hub.version` RPC will return a git-derived version — added `crates/termlink-hub/build.rs` mirroring `crates/termlink-cli/build.rs`; sets `cargo:rustc-env=CARGO_PKG_VERSION={git-derived}` which overrides the workspace Cargo.toml hardcode at compile time
- [x] Existing `hub_version_returns_binary_version_and_protocol_version` unit test in router.rs:3864 passes — `cargo test -p termlink-hub --lib hub_version_returns` → 1 passed
- [x] No regression on CLI fallback path — only hub-crate compile env changed; remote.rs:1727 untouched
- [x] Live deploy verification — rebuilt `target/release/termlink` (cargo build --release -p termlink, 3m32s, finished 2026-05-04T00:00Z), launched isolated test hub on `/tmp/test-hub-t1458b`, queried `hub.version` RPC via direct JSON-RPC over the local Unix socket. Response: `{"hub_version":"0.9.1821","protocol_version":1}` — matches `./target/release/termlink --version` (`termlink 0.9.1821`). The TLS-pinned TCP path against the test hub on :9101 hit a separate BadSignature TOFU issue unrelated to T-1458, so verification went via Unix socket, which is identical end-to-end RPC dispatch. Production-hub verification deferred to next operator-driven hub restart cycle (no agent-side risk to running prod hubs)

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
cargo check -p termlink-hub 2>&1 | tail -5 | grep -qE "Finished|0 errors"
cargo test -p termlink-hub --lib hub_version_returns 2>&1 | tail -5 | grep -qE "test result.*1 passed|ok\. 1 passed"

## RCA

**Symptom:** `fleet doctor` reports `Versions in fleet: 0.9.0 (5 hubs)` for every hub regardless of actual binary version. T-1166 cut-readiness verdict cannot trust the version histogram. Found 2026-05-03T20:55Z while investigating heartbeat-driver identity on .141.

**Root cause:** `crates/termlink-hub/Cargo.toml` declares `version.workspace = true`, which inherits the workspace's hardcoded `version = "0.9.0"`. `handle_hub_version()` in `router.rs:884` calls `env!("CARGO_PKG_VERSION")` which resolves at compile time to that literal "0.9.0". The CLI side already had `crates/termlink-cli/build.rs` (T-648 / T-1057) that sets `cargo:rustc-env=CARGO_PKG_VERSION` from `git describe --tags`, overriding the Cargo.toml default — but the hub crate had no equivalent build.rs, so the override never reached its compilation unit.

**Why structurally allowed:** Two contributing structural gaps. (1) Asymmetric build.rs presence — the framework had a documented version-derivation pattern in CLI but never propagated it to the hub crate when hub.version RPC was added (T-1132). (2) The unit test `hub_version_returns_binary_version_and_protocol_version` only asserts `hub_version == env!("CARGO_PKG_VERSION")` — a tautology that passes whether the env resolves to "0.9.0" or "0.9.1821". The test cannot detect the bug because both sides of `==` resolve identically at compile time.

**Prevention:** (1) Code fix shipped in this task — symmetric build.rs in `crates/termlink-hub/` mirroring CLI's. (2) Future `cargo:rustc-env=CARGO_PKG_VERSION` pattern is now codified in two places, raising the chance an audit notices missing build.rs on a new crate. (3) Recommended follow-up (separate task, not this one): fleet-doctor ad-hoc verification — when `hub_version` returns a value matching the workspace static `0.9.0`, log a warning. This would have caught the issue from the operator side.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-03T20:56:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1458-fix-hubversion-rpc--returns-hardcoded-09.md
- **Context:** Initial task creation

### 2026-05-03T20:57:43Z — status-update [task-update-agent]
- **Change:** status: started-work → captured
- **Reason:** Captured with full reproducer + ACs; fix needs build.rs work on hub crate — defer to dedicated session, not autonomous touch right now (T-1166 has higher priority and budget tight)

### 2026-05-03T21:23:18Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Reason:** user authorized continue on heavy fix

### 2026-05-03T22:01:30Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
