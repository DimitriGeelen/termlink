---
id: T-1463
name: "fleet doctor --save-snapshot <path> companion to --diff for routine decay-rate sampling"
description: >
  fleet doctor --save-snapshot <path> companion to --diff for routine decay-rate sampling

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/remote.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-04T05:37:31Z
last_update: 2026-05-04T05:43:31Z
date_finished: 2026-05-04T05:43:31Z
---

# T-1463: fleet doctor --save-snapshot <path> companion to --diff for routine decay-rate sampling

## Context

T-1462 added `--diff <path>` so two snapshots can be compared. Today the
operator captures a snapshot via redirect: `--legacy-usage --json > snap.json`
— but that suppresses the human-readable summary, forcing two runs (one to
read, one to save). `--save-snapshot <PATH>` writes the same JSON document
to PATH while leaving stdout/stderr untouched, so a single invocation both
shows the verdict and persists today's data point. Clean ergonomics for
the daily cron pattern documented in T-1166 migration doc.

## Acceptance Criteria

### Agent
- [x] `--save-snapshot <PATH>` flag added to `fleet doctor`. Optional. When provided WITHOUT `--json`, human output still prints; the JSON document is written to PATH atomically (write to PATH.tmp, fsync, rename).
- [x] When `--save-snapshot` is combined with `--json`, the same JSON is both printed to stdout AND saved to disk (verified: ts_ms 1777873367060 matches in both).
- [x] If PATH's parent directory does not exist, the command exits non-zero with a clear error before doing fleet-doctor work (fail fast — verified).
- [x] The saved JSON contains `_snapshot_ts_ms` and `legacy_summary` (jq has() returned true for both).
- [x] T-1166 migration doc gains a "Decay-rate sampling" subsection with the recommended daily workflow + rate-sign reference table.
- [x] `cargo build --release -p termlink` succeeds
- [x] Smoke: `--legacy-usage --save-snapshot /tmp/t1463-snaps/today.json` → file written, parses, both keys present.
- [x] Round-trip smoke: `--diff` against just-saved snapshot prints "no change" with rate 0.00 (flat).
- [x] Error-path smoke: `--save-snapshot /nonexistent/dir/x.json` exits non-zero with the expected parent-missing message.

## Verification

cargo build --release -p termlink
grep -q "save_snapshot: Option<std::path::PathBuf>" crates/termlink-cli/src/cli.rs
grep -q "Decay-rate sampling" docs/migrations/T-1166-retire-legacy-primitives.md
grep -q "save_snapshot" crates/termlink-cli/src/commands/remote.rs

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

### 2026-05-04T05:37:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1463-fleet-doctor---save-snapshot-path-compan.md
- **Context:** Initial task creation

### 2026-05-04T05:43:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
