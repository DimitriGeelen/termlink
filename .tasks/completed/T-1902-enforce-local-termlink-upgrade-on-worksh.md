---
id: T-1902
name: "Enforce local termlink upgrade on workshop-designer (.107)"
description: >
  Enforce local termlink upgrade on workshop-designer (.107)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-31T19:43:34Z
last_update: 2026-05-31T19:43:34Z
date_finished: 2026-05-31T20:21:37Z
---

# T-1902: Enforce local termlink upgrade on workshop-designer (.107)

## Context

Local termlink CLI on workshop-designer (hostname `dimitrimintdev`, .107) is at 0.11.1 (May 18) while source HEAD is at 0.11.472 — ~470 commits behind. Local hub is even further behind at 0.9.2110. Operator requested upgrade be enforced. Existing release build at `target/release/termlink` is at 0.11.421 (today 00:13) — close to HEAD but not at it. Plan: fresh `cargo build --release`, install over `/root/.cargo/bin/termlink`, restart local hub to pick up new binary, re-verify with `termlink --version` + `fleet doctor`.

## Acceptance Criteria

### Agent
- [x] `cargo build --release --bin termlink` completes successfully (in `/opt/termlink`) — finished in 7m12s, only pre-existing termlink-mcp unused-assignment warning
- [x] Fresh `target/release/termlink --version` reports `0.11.472` or newer — confirmed `termlink 0.11.472`
- [x] `/root/.cargo/bin/termlink` replaced with the new build; `termlink --version` from clean shell reports `0.11.472` or newer — installed via `install -m 755` (29 register PTYs held old inode; install's unlink+create avoids ETXTBSY); fresh shell reports 0.11.472
- [x] Local hub restarted (`termlink hub restart`); `termlink hub status` reports running with the new PID — `systemctl restart termlink-hub.service` (the supervisor); new PID 1875371, Active 21:52:36
- [x] `termlink fleet doctor` reports the local hub (`workstation-107-public` and/or `local-test`) on the new 0.11.x version — both endpoints now report `version: 0.11.472`
- [x] CLI-vs-hub stale-build WARN cleared for local hub — only laptop-141 (0.9.0) and ring20-{management,dashboard} (0.9.2127) remain in the skew list; local hub no longer flagged

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

### 2026-05-31T19:43:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1902-enforce-local-termlink-upgrade-on-worksh.md
- **Context:** Initial task creation
