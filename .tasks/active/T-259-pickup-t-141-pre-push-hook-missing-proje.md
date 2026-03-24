---
id: T-259
name: "Pickup from fw T-546: Release build fixes — flaky test ENV_LOCK + macOS runner"
description: >
  From framework agent pickup T-546: 1) Flaky test register_remote_and_discover missing
  ENV_LOCK guard (router.rs line 969), 2) macOS x86_64 CI runner macos-13 deprecated,
  change to macos-14 for cross-compile, 3) bump version and tag after fixes.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [pickup, release, ci]
components: []
related_tasks: []
created: 2026-03-24T08:41:40Z
last_update: 2026-03-24T08:42:47Z
date_finished: null
---

# T-259: Pickup from fw T-546 — Release build fixes

## Context

Pickup from framework agent (T-546 on .107). Two TermLink-side fixes needed for release builds.

## Acceptance Criteria

### Agent
- [ ] Flaky test `router::tests::register_remote_and_discover` fixed with ENV_LOCK guard
- [ ] Release workflow macOS x86_64 target uses `macos-14` instead of deprecated `macos-13`
- [ ] Tests pass: `cargo test register_remote_and_discover`

## Verification

grep -q "ENV_LOCK" crates/termlink-hub/src/router.rs

## Decisions

## Updates

### 2026-03-24T08:41:40Z — task-created [pickup from fw-agent on .107]
- **Source:** `/pickup fw-agent T-546` via termlink remote inject
- **Original message:** Flaky test router::tests::register_remote_and_discover — missing ENV_LOCK guard causes race condition with parallel tests clearing REMOTE_STORE. Fix: add `let _lock = ENV_LOCK.lock().await;` at top of test (line 969 in router.rs). Release workflow macOS x86_64 build — macos-13 runner deprecated/cancelled immediately. Fix: change os to macos-14 for x86_64-apple-darwin target (cross-compile). After fixes, bump version and tag for release test.
