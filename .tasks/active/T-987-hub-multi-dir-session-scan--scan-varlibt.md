---
id: T-987
name: "Hub multi-dir session scan — scan /var/lib/termlink + /tmp/termlink-UID + TERMLINK_RUNTIME_DIR"
description: >
  Hub multi-dir session scan — scan /var/lib/termlink + /tmp/termlink-UID + TERMLINK_RUNTIME_DIR

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-12T21:20:11Z
last_update: 2026-04-12T21:20:11Z
date_finished: null
---

# T-987: Hub multi-dir session scan — scan /var/lib/termlink + /tmp/termlink-UID + TERMLINK_RUNTIME_DIR

## Context

Build task from T-942 GO decision. The hub currently scans only one session directory
(`runtime_dir()/sessions`). With the two-pool architecture (T-959: persistent `/var/lib/termlink`
+ ephemeral `/tmp/termlink-UID`), sessions in the non-default pool are invisible to discovery.
Fix: scan all candidate dirs, merge results, deduplicate by session ID.

Related: T-940 (runtime dir RCA), T-959 (two-pool architecture), T-942 (inception)

## Acceptance Criteria

### Agent
- [ ] `discovery.rs` exposes `all_runtime_dirs()` returning Vec<PathBuf> of candidate dirs
- [ ] `manager::list_sessions()` iterates all dirs, deduplicates by session ID
- [ ] `supervisor::sweep()` cleans across all dirs
- [ ] Hub router `session.discover` returns sessions from all dirs
- [ ] Existing `runtime_dir()` remains unchanged (backward compat for session registration)
- [ ] Unit tests for multi-dir listing with sessions in different dirs
- [ ] All existing hub tests pass (`cargo test -p termlink-hub`)

## Verification

# Shell commands that MUST pass before work-completed. One per line.
cargo test -p termlink-hub
cargo test -p termlink-session
cargo clippy -p termlink-hub -p termlink-session -- -D warnings

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

### 2026-04-12T21:20:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-987-hub-multi-dir-session-scan--scan-varlibt.md
- **Context:** Initial task creation
