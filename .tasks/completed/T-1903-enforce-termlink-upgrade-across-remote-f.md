---
id: T-1903
name: "Enforce termlink upgrade across remote fleet (laptop-141, ring20-management, ring20-dashboard)"
description: >
  Enforce termlink upgrade across remote fleet (laptop-141, ring20-management, ring20-dashboard)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-31T20:12:03Z
last_update: 2026-05-31T20:21:38Z
date_finished: 2026-05-31T20:58:47Z
---

# T-1903: Enforce termlink upgrade across remote fleet (laptop-141, ring20-management, ring20-dashboard)

## Context

Companion to T-1902. T-1902 upgraded local hub on .107 to 0.11.472. Three remote hubs remain on 0.9.x:
- laptop-141 (192.168.10.141:9100) — 0.9.0 (pre-T-1458 build.rs era)
- ring20-management (192.168.10.122:9100) — 0.9.2127
- ring20-dashboard (192.168.10.121:9100) — 0.9.2127

Local musl-static binary at `target/x86_64-unknown-linux-musl/release/termlink` is 0.9.2127 (May 15) — must be rebuilt at HEAD before deploy. Fleet deploys MUST use musl-static (PL-100 / T-1422: glibc mismatch crashes glibc-linked builds on Debian 12 LXC hosts like ring20).

Deploy path: `scripts/fleet-deploy-binary.sh <hub> --probe --swap-restart`. The `--probe` flag (T-1423) runs `<staged>/termlink --version` on the remote before any swap, catching architecture/libc failures before they take down the running hub.

## Acceptance Criteria

### Agent
- [x] `cargo build --release --target x86_64-unknown-linux-musl --bin termlink` completes; `target/x86_64-unknown-linux-musl/release/termlink --version` reports 0.11.472 — actual `0.11.473` (HEAD `3546b434` after T-1902 commit landed mid-build)
- [x] laptop-141: `fleet-deploy-binary.sh laptop-141 --probe --swap-restart` succeeds (exit 0); fleet doctor reports laptop-141 on 0.11.472 — probe OK `termlink 0.11.473`, hub UP t=10s after swap, fleet doctor reports `version: 0.11.473` (0.9.0 → 0.11.473)
- [x] ring20-management: `fleet-deploy-binary.sh ring20-management --probe --swap-restart` succeeds (exit 0); fleet doctor reports ring20-management on 0.11.472 — probe OK `0.11.473`, hub UP t=15s, fleet doctor `version: 0.11.473` (0.9.2127 → 0.11.473); 4 active sessions survived restart
- [x] ring20-dashboard: `fleet-deploy-binary.sh ring20-dashboard --probe --swap-restart` succeeds (exit 0); fleet doctor reports ring20-dashboard on 0.11.472 — probe OK `0.11.473`, hub UP t=15s, fleet doctor `version: 0.11.473` (0.9.2127 → 0.11.473)
- [x] Post-deploy fleet-wide check: `fleet doctor` reports zero 0.9.x hubs; the "Version skew" action item is empty — 5/5 PASS, versions 0.11.472 (2 local) / 0.11.473 (3 remote), no 0.9.x left
- [x] Any post-rotation auth-mismatch is healed via `fleet reauth <hub> --bootstrap-from auto` (T-1291); fleet verify reports zero `drift` rows — `fleet verify` reports `match` for all three remotes (persist-if-present held; no PL-021 rotation triggered); no heal needed

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

### 2026-05-31T20:12:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1903-enforce-termlink-upgrade-across-remote-f.md
- **Context:** Initial task creation
