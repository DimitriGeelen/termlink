---
id: T-1415
name: "T-1166 post-cut cleanup: delete retired primitive handlers + fallback paths"
description: >
  After Tier-2 cut (LEGACY_PRIMITIVES_ENABLED=false has been baked >=7d in production), delete the retired-primitive code entirely. Replaces the const + cfg-feature mechanism with permanent removal.

status: captured
workflow_type: decommission
owner: human
horizon: later
tags: []
components: []
related_tasks: [T-1166, T-1411, T-1413]
created: 2026-04-30T07:07:28Z
last_update: 2026-04-30T07:07:28Z
date_finished: null
---

# T-1415: T-1166 post-cut cleanup: delete retired primitive handlers + fallback paths

## Context

This is the **delayed source-cleanup follow-up** to T-1166 staged via the
PL-094 destructive-cut pattern (T-1411 + T-1413). It must NOT run until:

1. **Tier-2 cut authorized** — `LEGACY_PRIMITIVES_ENABLED = false` flipped in
   `crates/termlink-hub/src/router.rs` (or built with cargo feature
   `legacy_primitives_disabled`), hub deployed to all production hosts
2. **Bake window passed** — ≥7 days of all production hubs running flag-off
   with `fw metrics api-usage` showing zero attributable legacy traffic
   (per `legacy_callers_by_ip` and `legacy_callers_by_pid` post-T-1414)
3. **Roll-back window closed** — operator confirms no consumers have hit
   the -32601 method-not-found rejection from `legacy_method_retired_response`

Once those gates are satisfied, this task removes the retired-primitive code
entirely so the const + cfg-feature mechanism can also go (the codebase has
one less abstraction to carry).

## Inventory of code to delete

Confirmed via `grep -rn 'handle_event_broadcast\|handle_inbox_status\|handle_inbox_list\|handle_inbox_clear'` plus the LEGACY callers list:

**Hub** (`crates/termlink-hub/src/`):
- `router.rs`:
  - `handle_event_broadcast` (~line 320) — async fn body
  - `handle_inbox_list` / `handle_inbox_status` / `handle_inbox_clear` (~lines 1663–1700)
  - 4 match arms in `route()` (after deleting the legacy_method_retired arms)
  - The `LEGACY_PRIMITIVES_ENABLED` const + `cfg!(...)` shim itself (no longer needed)
  - The `is_retired_legacy_method()` helper + `legacy_method_retired_response()` helper
  - The `cut_path` test module (gated under cfg-feature; entire module goes)
  - `handle_hub_capabilities` filter logic that drops retired methods
- `Cargo.toml`: `[features] legacy_primitives_disabled = []` line

**Session shim** (`crates/termlink-session/src/inbox_channel.rs`):
- `status_with_fallback` legacy-fallback else-branch (~line 277–286)
- `list_with_fallback` legacy-fallback (parallel)
- `clear_with_fallback` legacy-fallback (parallel)
- `call_legacy_inbox_status_via_client` / `_list_via_client` / `_clear_via_client`
- `flag_legacy_only` / `is_legacy_only` on `FallbackCtx` (no longer needed)
- The "warn_once" telemetry path for `inbox.status` / `inbox.list` / `inbox.clear`

**CLI** (`crates/termlink-cli/src/`):
- `commands/file.rs` (T-1166 list of file.* primitives — full removal of the
  send/receive paths if the user wants the CLI command also retired; verify
  with operator first because file.* may have UX-visible callers).

**Protocol constants** (`crates/termlink-protocol/src/control.rs`):
- `EVENT_BROADCAST`, `INBOX_LIST`, `INBOX_STATUS`, `INBOX_CLEAR`,
  `FILE_SEND`, `FILE_RECEIVE` const definitions (and any chunked variants)

**Tests:**
- `crates/termlink-hub/tests/no_legacy_callers.rs` — keep but tighten (the
  test now asserts these handlers don't exist; rename and update message)
- All tests under `#[cfg(not(feature = "legacy_primitives_disabled"))]`
  (T-1413) become live — gate removed
- Migration-doc references stay; add a "completed YYYY-MM-DD" line

**MCP / topic_lint** (`crates/termlink-mcp/src/tools.rs`,
`crates/termlink-hub/src/topic_lint.rs`): grep for the method names and
delete any remaining references.

## Acceptance Criteria

### Agent
- [ ] `grep -rn 'handle_event_broadcast\|handle_inbox_status\|handle_inbox_list\|handle_inbox_clear' crates/termlink-hub/src/` returns 0 matches (excluding migration doc + this task file)
- [ ] `grep -rn 'LEGACY_PRIMITIVES_ENABLED\|legacy_primitives_disabled' crates/` returns 0 matches
- [ ] `grep -rn 'call_legacy_inbox_\|status_with_fallback\|list_with_fallback\|clear_with_fallback' crates/` returns 0 matches in non-test code
- [ ] `cargo build -p termlink-hub` builds clean (no unused imports, no dead-code warnings)
- [ ] `cargo test -p termlink-hub --lib` passes (no `--features` flag needed)
- [ ] `cargo test -p termlink-session --lib` passes
- [ ] `cargo test -p termlink-cli --lib` passes
- [ ] `crates/termlink-hub/Cargo.toml` no longer has the `[features]` legacy_primitives_disabled entry
- [ ] `docs/migrations/T-1166-retire-legacy-primitives.md` updated with "Source cleanup completed YYYY-MM-DD (T-1415)" line
- [ ] No new clippy warnings introduced (`cargo clippy -p termlink-hub -p termlink-session -p termlink-cli -- -D warnings`)

### Human
- [ ] [REVIEW] Verify production hubs have been running flag-off for ≥7 days
  **Steps:**
  1. SSH to each production hub (.107, .121/.143, .122)
  2. Check that the running binary was built with the cut applied: `termlink hub status` should show no `event.broadcast`/`inbox.*` in capabilities `methods`
  3. Check `journalctl -u termlink-hub --since "7 days ago" | grep -E '(event.broadcast|inbox.(list|status|clear))'` — should be empty (no rejections firing means no callers retrying)
  **Expected:** All production hubs report retired methods absent from capabilities; no rejection log lines in 7-day window
  **If not:** Bake window incomplete — defer this task; investigate the caller that's still hitting the gate

- [ ] [REVIEW] Confirm bake metric is clean
  **Steps:**
  1. Run `fw metrics api-usage --last-Nd 7 --json` on each production hub
  2. Verify `legacy_attributable == 0` in the JSON output
  **Expected:** All hubs report 0 attributable legacy calls
  **If not:** A caller is still hitting legacy methods (will get -32601 rejections); investigate before deleting code

## Verification

# When this task fires, verification will run automatically. The greps below
# would all pass once the cleanup is correct.
# (Can't run pre-cut — these would currently FAIL because the code still exists.)
test -f docs/migrations/T-1166-retire-legacy-primitives.md
test -f .tasks/active/T-1415-t-1166-post-cut-cleanup-delete-retired-p.md || test -f .tasks/completed/T-1415-t-1166-post-cut-cleanup-delete-retired-p.md

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

### 2026-04-30T07:07:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1415-t-1166-post-cut-cleanup-delete-retired-p.md
- **Context:** Initial task creation
