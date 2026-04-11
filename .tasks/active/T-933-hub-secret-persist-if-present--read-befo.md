---
id: T-933
name: "Hub secret persist-if-present — read before generate"
description: >
  crates/termlink-hub/src/server.rs:46 generate_and_write_hub_secret() unconditionally generates a fresh secret on every start, and server.rs:154/210 deletes the file on clean shutdown. Rotation is incidental, not deliberate (no comment asserts it as a security property — T-930 Spike 3). Fix: read existing hex if present and valid (64 chars, mode 0600, parses), otherwise generate. Remove remove_file(hub_secret_path()) from both cleanup paths. Add integration test that starts hub twice and asserts the same secret is used. Zero network security delta. From T-930 decomposition.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-930, T-931]
created: 2026-04-11T22:29:26Z
last_update: 2026-04-11T22:56:24Z
date_finished: null
---

# T-933: Hub secret persist-if-present — read before generate

## Context

T-930 Spike 3 discovered that `generate_and_write_hub_secret()` in
`crates/termlink-hub/src/server.rs:46` unconditionally generates a fresh
HMAC secret on every hub start, and `server.rs:152-158` / `server.rs:209-213`
delete the secret file on clean shutdown. Rotation is incidental, not
deliberate — no comment asserts it as a security property. Consequence:
every hub restart invalidates any secret a cross-host agent has cached,
causing `-32010 Token validation failed` until the new secret is
redistributed.

## Acceptance Criteria

### Agent
- [x] `generate_and_write_hub_secret()` reads the existing hub.secret file if it exists AND parses as valid 64-char hex. If valid, it reuses the existing secret; otherwise it generates a fresh one. (via new `load_existing_hub_secret()` helper, commit c071297)
- [x] Both `remove_file(hub_secret_path())` calls removed from clean-shutdown paths (`run_with_tcp` and `run_blocking`) — the secret persists at rest across hub restarts.
- [x] `cargo build --workspace` clean.
- [x] New unit test `hub_secret_persists_across_calls`: two calls return the same secret; corrupted file triggers regeneration.
- [x] `cargo test -p termlink-hub` 177/177 pass (no regressions).
- [x] Live test on .107: second `sudo systemctl restart termlink-hub` preserved secret `b3076eb7…`; journalctl shows `"Hub secret loaded from disk (persist-if-present, T-933)"`.

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

### 2026-04-11T22:29:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-933-hub-secret-persist-if-present--read-befo.md
- **Context:** Initial task creation

### 2026-04-11T22:51:11Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
