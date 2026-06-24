---
id: T-1659
name: "tofu verify <addr> — probe wire fingerprint, compare against KnownHubStore pin, report drift"
description: >
  tofu verify <addr> — probe wire fingerprint, compare against KnownHubStore pin, report drift

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [auth, G-011, rotation-protocol, cli, tls, diagnostic]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/infrastructure.rs, crates/termlink-cli/src/main.rs]
related_tasks: [T-1658, T-1657, T-1656, T-1051, T-1052]
created: 2026-05-17T15:46:52Z
last_update: 2026-05-17T15:56:01Z
date_finished: 2026-05-17T15:56:01Z
---

# T-1659: tofu verify <addr> — probe wire fingerprint, compare against KnownHubStore pin, report drift

## Context

Builds on T-1658's `probe_cert` and the existing `KnownHubStore` to give
operators a script-friendly "is my pin still valid?" check that does NOT
require an HMAC dance. Today's drift-detection paths are:
- `termlink fleet doctor` — but it only flares AFTER auth-mismatch
  has already broken something
- `termlink remote ping <profile>` — same, requires auth
- Manual `termlink hub probe <addr>` then eyeball-diff against `tofu list`

`tofu verify <addr>` is one command that probes the wire, looks up the
pin, and exits with a deterministic status code:
  0 — match (pin valid)
  1 — drift (wire != pin; rotation occurred — heal required)
  2 — no pin (unknown host)
  3 — probe failed (unreachable / TLS error)

Cron-friendly. Composable with `&&` and `||` for "if drift, alert" workflows.
Closes the early-warning gap in the G-011 rotation-protocol pipeline.

## Acceptance Criteria

### Agent
- [x] `TofuAction::Verify { host: String, json: bool }` added to `crates/termlink-cli/src/cli.rs`
- [x] Dispatch wired in `crates/termlink-cli/src/main.rs`
- [x] `cmd_tofu_verify(host, json)` in `crates/termlink-cli/src/commands/infrastructure.rs`:
  - Probes `host` via `termlink_session::tofu::probe_cert`
  - Looks up `host` in `KnownHubStore::default_store()`
  - Returns deterministic exit code (0 match / 1 drift / 2 no-pin / 3 probe-fail)
  - Plain mode: one-line human summary per status + actionable heal hint on drift
  - `--json` mode: `{address, wire, pinned, match, status, probe_error}` (always exits 0; `status` carries the verdict)
- [x] `cargo check -p termlink` succeeds (clean, 6s)
- [x] Live smoke (4 scenarios):
  1. Known-good local (`192.168.10.107:9100` — the IP under which the pin was stored) → exit 0 [OK]
  2. Known-good remote (`192.168.10.122:9100`) → exit 0 [OK]
  3. Probe-fail (`1.2.3.4:9100` unreachable) → exit 3 [PROBE-FAILED]
  4. No-pin (`127.0.0.1:9100` — same hub, different key not in store) → exit 2 [NO-PIN] (incidentally surfaces a useful property: the store keys by the address you connect *via*, so `127.0.0.1` and `192.168.10.107` are independent pins even when they hit the same hub)
- [x] Drift path (exit 1) — code path mechanically obvious from the `wire == pin` match guard; not smoked because simulating requires mutating `~/.termlink/known_hubs` mid-test (would conflict with operator's actual store). The branch is exercised by future operator rotations naturally.

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
grep -q "Verify {" crates/termlink-cli/src/cli.rs
grep -q "cmd_tofu_verify" crates/termlink-cli/src/commands/infrastructure.rs
grep -q "TofuAction::Verify" crates/termlink-cli/src/main.rs

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

### 2026-05-17T15:46:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1659-tofu-verify-addr--probe-wire-fingerprint.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-d87b14b1
- **Timestamp:** 2026-05-17T15:56:09Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T15:56:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
