---
id: T-1458
name: "Fix hub.version RPC — returns hardcoded 0.9.0 instead of actual build version"
description: >
  Fix hub.version RPC — returns hardcoded 0.9.0 instead of actual build version

status: captured
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-03T20:56:57Z
last_update: 2026-05-03T20:57:43Z
date_finished: null
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
- [ ] `hub.version` RPC returns a version string derived the same way as the CLI's `--version` (git-derived via build.rs or equivalent shared mechanism)
- [ ] `fleet doctor --json` for a freshly-built hub shows `hub_version` matching `termlink --version` output (modulo `"termlink "` prefix)
- [ ] Existing `hub_version_returns_binary_version_and_protocol_version` unit test in router.rs:3864 passes
- [ ] No regression: pre-T-1132 hubs (which return method-not-found) still get classified as `"unknown"` by the CLI fallback (remote.rs:1727)

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
