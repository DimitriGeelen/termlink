---
id: T-1674
name: "fleet doctor --include-pin-check: bound probe_cert wall time via tokio::time::timeout (T-1666 follow-up)"
description: >
  fleet doctor --include-pin-check: bound probe_cert wall time via tokio::time::timeout (T-1666 follow-up)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-17T20:49:33Z
last_update: 2026-05-17T20:57:38Z
date_finished: 2026-05-17T20:57:38Z
---

# T-1674: fleet doctor --include-pin-check: bound probe_cert wall time via tokio::time::timeout (T-1666 follow-up)

## Context

T-1666 parallelizes `probe_cert` via `tokio::spawn` so total wall time is bounded by the slowest probe rather than the sum. But `probe_cert` itself has no timeout — it relies on OS-level TCP retry budget when the host is unreachable (e.g. laptop-141 powered off). A single unreachable host stretches the slowest-probe to 30-60+s, observable as long `--watch` cycle delays.

Fix: wrap the inner `probe_cert` call site in `tokio::time::timeout(Duration::from_secs(timeout_secs), ...)`. The `timeout_secs` value (default 10s) is already the hub-RPC timeout — extending it to TLS probes keeps a single bound for "how long do I wait per hub". On timeout, classify as `probe-fail` with a clear message — semantically identical to the existing "TCP connect failed" path.

## Acceptance Criteria

### Agent
- [x] `probe_cert` call inside `cmd_fleet_doctor`'s parallel spawn is wrapped with `tokio::time::timeout(Duration::from_secs(timeout_secs), ...)` — verified `remote.rs:3279-3286`
- [x] On timeout, the entry is classified as `probe-fail` with error message containing `timeout` and the duration — verified live: `"TLS probe to 192.168.10.141:9100 timeout after 10s"`
- [x] `cargo check --workspace` passes — clean (1 pre-existing unrelated `termlink-mcp` warning)
- [x] Default behavior unchanged on reachable hubs (verified via live smoke against local hub) — `match` / `no-pin` outcomes preserved for reachable hubs; only laptop-141 now bails at the 10s bound. Total fleet wall time: **20.19s** (was reported ~2m16s pre-fix in T-1667 commit notes)

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

cargo check --workspace
bash -c "grep -q 'tokio::time::timeout' crates/termlink-cli/src/commands/remote.rs && grep -B2 -A2 'probe_cert' crates/termlink-cli/src/commands/remote.rs | grep -q 'timeout'"

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

### 2026-05-17T20:49:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1674-fleet-doctor---include-pin-check-bound-p.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-6a1c1617
- **Timestamp:** 2026-05-17T20:57:38Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-17T20:57:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
