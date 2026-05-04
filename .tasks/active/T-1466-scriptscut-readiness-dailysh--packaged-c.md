---
id: T-1466
name: "scripts/cut-readiness-daily.sh — packaged cron wrapper for T-1462/3/5 cut-readiness pattern"
description: >
  scripts/cut-readiness-daily.sh — packaged cron wrapper for T-1462/3/5 cut-readiness pattern

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-04T06:06:31Z
last_update: 2026-05-04T06:06:31Z
date_finished: null
---

# T-1466: scripts/cut-readiness-daily.sh — packaged cron wrapper for T-1462/3/5 cut-readiness pattern

## Context

T-1462 + T-1463 + T-1465 give operators every CLI primitive needed for
cron-based cut-readiness tracking. The migration doc shows the pattern
in case-statement form. This task packages the pattern as a working
script (`scripts/cut-readiness-daily.sh`) so operators install one symlink
into `/etc/cron.daily/` and never have to author the case statement.

Uses --save-snapshot for today, picks up yesterday's snapshot
automatically (alphabetic sort of the snapshots directory), feeds it
back via --diff. Honors --exit-code-on-verdict so cron's MAILTO carries
WAIT/UNCERTAIN signals naturally.

## Acceptance Criteria

### Agent
- [x] `scripts/cut-readiness-daily.sh` exists, is executable, follows scripts/ peer convention (#!/bin/bash + set -u + project-root-relative TL resolution).
- [x] Default invocation captures today's snapshot, diffs against most recent prior, prints verdict + diff to stderr, exits with verdict-mapped code.
- [x] `--snapshots-dir <PATH>` overrides default `/var/lib/termlink/snapshots`.
- [x] First-run safe: no prior snapshot → "no prior snapshot — initial capture" message, exits with capture-step's code (0 for healthy fleet).
- [x] Missing-binary error path (exit 2). Missing-snapshots-dir auto-create with 0700 perms.
- [x] Smoke (a): initial capture in `mktemp -d` → exit 0, file appears.
- [x] Smoke (b): routine run with one prior in dir → DIFF block prints, exit 0 propagated through --exit-code-on-verdict.
- [x] No code changes to crates/ — pure scripts/ addition.

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

test -x scripts/cut-readiness-daily.sh
bash -n scripts/cut-readiness-daily.sh
grep -q "snapshots-dir" scripts/cut-readiness-daily.sh
grep -q "no prior snapshot" scripts/cut-readiness-daily.sh
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

### 2026-05-04T06:06:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1466-scriptscut-readiness-dailysh--packaged-c.md
- **Context:** Initial task creation
