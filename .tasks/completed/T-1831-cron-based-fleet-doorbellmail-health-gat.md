---
id: T-1831
name: "Cron-based fleet doorbell+mail health gate — daily selftest per hub in hubs.toml"
description: >
  Cron-based fleet doorbell+mail health gate — daily selftest per hub in hubs.toml

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T12:43:50Z
last_update: 2026-05-28T12:43:50Z
date_finished: null
---

# T-1831: Cron-based fleet doorbell+mail health gate — daily selftest per hub in hubs.toml

## Context

T-1829 shipped `agent-conversation-selftest.sh` (loopback doorbell+mail validation, fleet-wide PASS 2026-05-28). T-1829 Recommendation block listed three follow-ups; this task closes #1: a daily cron canary that runs the selftest against every profile in `~/.termlink/hubs.toml` and only writes to a log when something is non-pass. Same shape as `scripts/check-mirror-freshness.sh` + `.context/cron/release-mirror-canary.crontab` (T-1140/T-1696). Empty log = healthy fleet. Persistent observability becomes load-bearing when T-1830 adoption push lands and real conversations start flowing — drift in runtime health must be visible BEFORE conversations break.

## Acceptance Criteria

### Agent
- [ ] `scripts/check-fleet-doorbell-mail-health.sh` exists, executable, with `--quiet` / `--json` / `--no-heartbeat` flags mirroring `check-mirror-freshness.sh`
- [ ] Script reads `~/.termlink/hubs.toml` (or `--hubs-file <path>`) and iterates profiles; for each `[hubs.NAME]` runs `scripts/agent-conversation-selftest.sh --hub <address> --json` and parses `.verdict`
- [ ] Exit 0 only when every reachable profile reports `verdict=pass`; exit 1 on any non-pass (drift) or unreachable; exit 2 on tooling error (missing hubs.toml, jq missing, etc.)
- [ ] `--quiet` suppresses output on full-pass (cron-friendly — log only grows on drift, same as release-mirror-canary)
- [ ] `--json` emits a single envelope: `{ok, profiles:[{name,address,verdict,elapsed_ms,error?}], summary:{total,pass,fail,unreachable}}`
- [ ] Heartbeat file `.context/working/.fleet-doorbell-mail-canary.heartbeat` is touched before the network sweep (same convention as release-mirror-canary, suppressible via `--no-heartbeat`)
- [ ] `.context/cron/fleet-doorbell-mail-canary.crontab` exists with USER-field syntax, runs daily at a non-colliding minute, append-only to `.context/working/.fleet-doorbell-mail-canary.log`
- [ ] `scripts/test-check-fleet-doorbell-mail-health.sh` exists with at least: T1 `--help` exits 0 with usage, T2 unknown arg exits 2, T3 `--json` against `--hubs-file` with one local-only profile returns ok=true verdict=pass, T4 `--json` against `--hubs-file` with one unreachable profile returns ok=false summary.unreachable>=1
- [ ] All tests pass
- [ ] `fw audit` PASS — including cron-sync check picking up the new crontab
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
test -x scripts/check-fleet-doorbell-mail-health.sh
test -x scripts/test-check-fleet-doorbell-mail-health.sh
test -f .context/cron/fleet-doorbell-mail-canary.crontab
bash scripts/test-check-fleet-doorbell-mail-health.sh
bash scripts/check-fleet-doorbell-mail-health.sh --help >/dev/null

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

### 2026-05-28T12:43:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1831-cron-based-fleet-doorbellmail-health-gat.md
- **Context:** Initial task creation
