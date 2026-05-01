---
id: T-1442
name: "Populate model_used and fallback_used in dispatch meta.json"
description: >
  Populate model_used and fallback_used in dispatch meta.json

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-01T20:42:49Z
last_update: 2026-05-01T21:03:11Z
date_finished: 2026-05-01T21:03:11Z
---

# T-1442: Populate model_used and fallback_used in dispatch meta.json

## Context

U-005 from upstream framework. /opt/termlink is the substrate that knows the truth
about which model a dispatch worker actually ran with, and whether routing chose a
fallback. The framework's `fw termlink dispatch` writes meta.json with
`model_used: null` and `fallback_used: null`, intentionally leaving the substrate
to populate them. Closing the value loop here unblocks the orchestrator-rethink arc
on the framework side (Watchtower /orchestrator's "Recent dispatches" panel renders
n/a until both fields are non-null).

Scope = /opt/termlink's own dispatch path: `scripts/tl-dispatch.sh` (the substrate's
shell-driven dispatcher that the framework's adapter mirrors). Mirror the framework's
`task_type` / `model_used` / `fallback_used` schema and populate the latter two from
the substrate's resolution decision.

## Acceptance Criteria

### Agent
- [x] `scripts/tl-dispatch.sh cmd_spawn` accepts `--model <m>` and `--task-type <t>` flags (matching the framework's `agents/termlink/termlink.sh` contract).
- [x] After cmd_spawn writes meta.json, `/tmp/tl-dispatch/<name>/meta.json` contains `task_type`, `model`, `model_used`, `fallback_used` keys (schema parity with the framework).
- [x] When `--model haiku` is passed, `model_used = "haiku"` and `fallback_used = false` (explicit choice — no fallback).
- [x] When `--model` is omitted but `DISPATCH_MODEL_FOR_<TYPE>` is set in the environment for the resolved task-type, `model_used` is that env value and `fallback_used` is `false`.
- [x] When `--model` is omitted, no per-type override exists, but `DISPATCH_MODEL_DEFAULT` is set, `model_used` is the default and `fallback_used` is `true` (no per-type specialist → fell back).
- [x] When no model is resolvable at all (no flag, no env), `model_used` and `fallback_used` are both JSON `null` (don't lie about state we don't know).
- [x] A regression test in `tests/test_tl_dispatch_meta.sh` pins the schema and the four resolution branches.
- [x] `cargo check --workspace` passes.
- [x] `bash -n scripts/tl-dispatch.sh` passes.
- [x] `tests/test_tl_dispatch_meta.sh` exits 0.

### Human
- [ ] [REVIEW] Spot-check by running cmd_spawn against a live hub and confirm `cat /tmp/tl-dispatch/spot/meta.json | python3 -m json.tool` shows `model_used: "haiku"`, `fallback_used: false`, `task_type: "build"`.
  **Steps:**
  1. `cd /opt/termlink && cargo build --release -p termlink-cli`
  2. `./target/release/termlink hub start &` (if no hub is running)
  3. `bash scripts/tl-dispatch.sh --name spot --prompt 'echo hi' --model haiku --task-type build`
  4. `cat /tmp/tl-dispatch/spot/meta.json | python3 -m json.tool`
  5. `bash scripts/tl-dispatch.sh cleanup`
  **Expected:** JSON shows `task_type: "build"`, `model: "haiku"`, `model_used: "haiku"`, `fallback_used: false`.
  **If not:** Re-run the regression test (`bash tests/test_tl_dispatch_meta.sh`) — if THAT passes but live spawn doesn't, the bug is in spawn argument plumbing.

## Verification

bash -n scripts/tl-dispatch.sh
bash tests/test_tl_dispatch_meta.sh
cargo check --workspace

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

### 2026-05-01T20:42:49Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1442-populate-modelused-and-fallbackused-in-d.md
- **Context:** Initial task creation

### 2026-05-01T20:45:56Z — status-update [task-update-agent]
- **Change:** horizon: now → later
- **Change:** status: started-work → captured (auto-sync)

### 2026-05-01T20:47:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: later → now

### 2026-05-01T21:03:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
