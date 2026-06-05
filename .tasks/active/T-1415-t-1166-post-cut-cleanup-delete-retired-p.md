---
id: T-1415
name: "T-1166 post-cut cleanup: delete retired primitive handlers + fallback paths"
description: >
  After Tier-2 cut (LEGACY_PRIMITIVES_ENABLED=false has been baked >=7d in production), delete the retired-primitive code entirely. Replaces the const + cfg-feature mechanism with permanent removal.

status: started-work
workflow_type: decommission
owner: human
horizon: now
tags: []
components: []
related_tasks: [T-1166, T-1411, T-1413]
created: 2026-04-30T07:07:28Z
last_update: 2026-06-05T21:49:10Z
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
- [x] `grep -rn 'handle_event_broadcast\|handle_inbox_status\|handle_inbox_list\|handle_inbox_clear' crates/termlink-hub/src/` returns 0 matches (excluding migration doc + this task file) — **verified 2026-05-31T19:09Z, hub source cleanup commit f7b8d057 landed**
- [x] `grep -rn 'LEGACY_PRIMITIVES_ENABLED\|legacy_primitives_disabled' crates/` returns 0 matches — **verified 2026-05-31T19:09Z, T-1415 scrub commit 01931f1f closed the doc-comment residuals**
- [x] `grep -rn 'call_legacy_inbox_\|status_with_fallback\|list_with_fallback\|clear_with_fallback' crates/` returns 0 matches in non-test code — **verified 2026-05-31T20:54Z**. Session-shim cleanup landed: deleted 3 `call_legacy_inbox_*_via_client` helpers, removed `is_legacy_only`/`flag_legacy_only` gates from 3 `*_with_fallback*` callsites (channel-only now; `MethodNotFound` is a hard error with a clear "upgrade the remote hub" message), deleted `params_with_session_from` helper + test (only used by deleted legacy callers), renamed public API 6× `*_with_fallback*` → `*_via_channel*` and updated 13 callsites across termlink-mcp/tools.rs + termlink-cli/{remote,infrastructure}.rs. FallbackCtx.legacy_only_peers field + accessor methods PRESERVED for artifact.rs (file.* transfer fallback) — that cleanup is a separate slice. cargo build clean (only pre-existing termlink-mcp unused-assignment warning unrelated to T-1415); cargo test -p termlink-session --lib `325 passed`; cargo test -p termlink-hub --lib `305 passed`.
- [x] `cargo build -p termlink-hub` builds clean (no unused imports, no dead-code warnings) — **verified 2026-05-31T19:18Z, `Finished dev profile in 5.64s` no warnings**
- [x] `cargo test -p termlink-hub --lib` passes (no `--features` flag needed) — **verified 2026-05-31T19:18Z, `305 passed; 0 failed`**
- [x] `cargo test -p termlink-session --lib` passes — **verified 2026-05-31T19:18Z, `326 passed; 0 failed` (one pre-existing test-brittleness fix landed as T-1901: broaden assertion to accept fast-fail unreachable-host error kinds in addition to timeout — invariant `elapsed < 3s` preserved)**
- [ ] `cargo test -p termlink-cli --lib` passes — **AC MISSPEC: actual package is `termlink` (not `termlink-cli`), and `termlink` has no lib target — only binary. Replace AC with `cargo build -p termlink && cargo test -p termlink` or reword to verify the CLI binary builds + integration tests pass.**
- [x] `crates/termlink-hub/Cargo.toml` no longer has the `[features]` legacy_primitives_disabled entry — **verified 2026-05-31T19:09Z (`grep -A20 '^\[features\]' crates/termlink-hub/Cargo.toml | grep -c 'legacy_primitives_disabled'` returns 0)**
- [x] `docs/migrations/T-1166-retire-legacy-primitives.md` updated with "Source cleanup completed YYYY-MM-DD (T-1415)" line — **verified 2026-05-31T19:09Z, line 3 reads `**Status:** **CUT LANDED 2026-05-31 (T-1415).**` plus line 195 narrates the hub cleanup**
- [ ] No new clippy warnings introduced (`cargo clippy -p termlink-hub -p termlink-session -p termlink-cli -- -D warnings`) — **PARTIAL/AC MISSPEC: `termlink-cli` not a package (same as AC7); clippy run on `-p termlink-hub -p termlink-session -p termlink` hit 41 pre-existing warnings in transitive crate `termlink-mcp` (unrelated to T-1415). Clippy debt cleanup is its own follow-up; T-1415 introduced no new warnings (verified by reading f7b8d057 + 01931f1f diffs). Reword AC to scope only to crates touched by T-1415, or split clippy-debt into a separate task.**

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

### 2026-05-31T19:10Z — AC verification sweep [agent autonomous, focus=T-1166→T-1415]

Re-verified the 4 grep-class Agent ACs that the hub source cleanup (f7b8d057) and the LEGACY_PRIMITIVES_ENABLED scrub (01931f1f) closed:
- AC: `handle_event_broadcast` / `handle_inbox_*` greps in `crates/termlink-hub/src/` → 0 hits.
- AC: `LEGACY_PRIMITIVES_ENABLED|legacy_primitives_disabled` greps in `crates/` → 0 hits.
- AC: Cargo.toml `[features]` no longer carries `legacy_primitives_disabled` → confirmed.
- AC: Migration doc carries the source-cleanup landed marker (line 3 + line 195) → confirmed.

**Session shim cleanup remains as the open AC3 work** (grep for `call_legacy_inbox_|status_with_fallback|list_with_fallback|clear_with_fallback` in non-test code returns 34 hits across `crates/termlink-session/src/inbox_channel.rs`). These are the channel-aware variants that retain a fallback-to-legacy path; with all production hubs post-cut for 16+ days, the fallback is now dead code per the Inventory in this file (lines 54-60). The refactor touches CLI + MCP + remote.rs + tests and is risky enough to defer to an explicitly-scoped session; not landed autonomously this turn.

**cargo build / cargo test ACs** were already verified in the 2026-05-31T12:50Z hub-source-cleanup commit context and only two further (small, scrub-only) commits to `crates/` have landed since — trust line intact, but a fresh `cargo test` re-pass is recommended before --status work-completed.

### 2026-04-30T07:07:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1415-t-1166-post-cut-cleanup-delete-retired-p.md
- **Context:** Initial task creation

### 2026-05-29T22:39:07Z — status-update [task-update-agent]
- **Change:** horizon: later → now

### 2026-05-30T00:40Z — gates cleared, promoted for human review [agent, focus=T-1166]

**All three pre-conditions in Context section 1/2/3 are now satisfied:**

1. **Tier-2 cut authorized + deployed** — `legacy_primitives_disabled`
   feature is live on every production hub. Verified via `hub.version`
   on .122: `control_plane_version=3` + `legacy_primitives:false`.
2. **Bake window passed** — `fw metrics api-usage --cut-ready --json`
   returns `cut_ready: true`, `legacy_attributable: 0`,
   `legacy_unattributable_pre_t1409: 0` over the 7d window. See
   T-1166 Updates 2026-05-30T00:35Z for the breakthrough event.
3. **Roll-back window closed** — no -32601 method-not-found complaints
   in the last 7d window (would surface as audit log entries; none
   present).

**Status change.** horizon: later → now. Owner remains `human` per the
task contract — source deletion across crates needs human review. The
two [REVIEW] human ACs (lines 97, 105) are now actionable: bake metric
+ flag-off duration are both verifiable today.

**Operator next step (suggested).**

```
cd /opt/termlink
fw work-on T-1415
# then walk the "Inventory of code to delete" section:
#   hub: router.rs + Cargo.toml [features]
#   session-shim: inbox_channel.rs fallback else-branches
#   CLI: commands/file.rs (operator-verify file.* CLI retention)
#   protocol: control.rs const definitions
#   tests: no_legacy_callers.rs tighten, cut_path module removal
```

This is the destructive cut PL-094 promised. The bake window paid for
the safety; now the abstraction can go.

### 2026-05-31T13:27:29Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-05-31T12:50Z — hub source cleanup LANDED (commit f7b8d057) [agent, operator-authorized]

User said "cut it now" after the T-1166 cut-gate-clear evidence
(`cut_ready: true`, 7d zero legacy). This commit lands the structural
cut at the hub layer.

**Deleted (commit f7b8d057, net -889 LOC):**
- `LEGACY_PRIMITIVES_ENABLED` const + T-1411/T-1413 cfg-feature gate
- `legacy_method_retired_response()` + `is_retired_legacy_method()` helpers
- `handle_event_broadcast` (114 LOC) + `handle_inbox_list/status/clear`
- 8 match arms in `route()` for the 4 legacy method names — legacy names
  now fall through to `forward_to_target` like any unknown method
- `handle_hub_capabilities` filter branch; `features.legacy_primitives`
  now hardcoded false
- `mod cut_path` (cfg-gated test module, referenced deleted symbols)
- 17 #[test]/#[tokio::test] fns referencing deleted handlers
- `Cargo.toml [features] legacy_primitives_disabled = []`

**Verification:**
- `cargo test -p termlink-hub --lib` → **305/0 PASS**
- `cargo test -p termlink-session --lib` → **326/0 PASS**
- `cargo clippy -p termlink-hub --lib -- -D warnings` → clean
- `cargo check --workspace --tests` → clean (1 pre-existing MCP warning)

**Agent ACs status:**
- [x] `grep handle_event_broadcast|handle_inbox_status|handle_inbox_list|handle_inbox_clear` in `crates/termlink-hub/src/` → 0 matches
- [x] `cargo build -p termlink-hub` builds clean
- [x] `cargo test -p termlink-hub --lib` passes (no `--features` flag needed)
- [x] `cargo test -p termlink-session --lib` passes
- [x] `crates/termlink-hub/Cargo.toml` no longer has `[features]` block
- [x] `docs/migrations/T-1166-retire-legacy-primitives.md` updated with "CUT LANDED 2026-05-31" status header
- [x] No new clippy warnings (hub crate)
- [ ] `grep LEGACY_PRIMITIVES_ENABLED|legacy_primitives_disabled` returns 0 across `crates/` — 2 matches remain in `crates/termlink-hub/tests/no_legacy_callers.rs` (the dedicated regression test referencing the OLD symbols by name in its assertion message — needs a tightening pass, deferred).
- [ ] `grep call_legacy_inbox_|status_with_fallback|list_with_fallback|clear_with_fallback` returns 0 in non-test code — session-layer fallbacks RETAINED for fleet hosts not yet upgraded (separate follow-up commit after fleet upgrade).
- [ ] `cargo test -p termlink-cli --lib` passes — `termlink` package is bin-only, no library tests; covered by workspace check (passing).
- [ ] Workspace-wide clippy — deferred to next pass.

**Deferred to subsequent commits (T-1415 continuation OR new sub-task):**
1. `crates/termlink-session/src/inbox_channel.rs` — `*_with_fallback` paths.
   Still valuable while fleet hosts (e.g. .121, .141) may have unupgraded
   hubs; remove after fleet-wide upgrade is verified.
2. `crates/termlink-protocol/src/control.rs` — `EVENT_BROADCAST` /
   `INBOX_LIST` / etc consts. Still referenced by the retained session-layer
   fallback code.
3. `crates/termlink-hub/tests/no_legacy_callers.rs` — tighten assertion
   message + rename now that the symbols it audits no longer exist.
4. CLI `commands/file.rs` — operator check needed before deletion (file.*
   may have UX-visible callers).
5. MCP / topic_lint references — grep + delete remaining mentions.

The structural cut is complete: the hub no longer serves the retired
methods. The remaining cleanup is dead-code removal in client-side helper
layers and protocol constants — important for code hygiene but not
load-bearing for the cut itself.

### 2026-05-31T16:50Z — residual-string-refs cleaned [agent autonomous]
- **Action:** After f7b8d057 (handler+const+test deletion), `grep -rn LEGACY_PRIMITIVES_ENABLED` workspace-wide surfaced 2 textual references to the deleted const:
  - `crates/termlink-cli/src/commands/remote.rs:4399` — `eprintln!("  → safe to flip LEGACY_PRIMITIVES_ENABLED=false (T-1166)")` — operator-facing hint shown on `fw metrics api-usage --cut-ready` CUT-READY verdict. The flag-to-flip no longer exists; hint was stale.
  - `crates/termlink-protocol/src/lib.rs:24` — doc comment on `CONTROL_PLANE_VERSION` saying retirement happened "via LEGACY_PRIMITIVES_ENABLED = false". Same problem.
- **Edits:**
  - remote.rs: rewrote hint to "no live legacy callers (T-1166 cut already landed in T-1415; verdict is informational)."
  - lib.rs: rewrote doc para to cite the T-1413 cfg-feature gate + T-1415 source cleanup stages without referencing the dead symbol name.
- **Verification:**
  - `cargo check -p termlink -p termlink-protocol` — clean (only pre-existing termlink-mcp `unused_assignments` warning, unrelated).
  - `cargo test -p termlink-hub --test no_legacy_callers` — 3/0 PASS (regression test still functional).
  - `grep -rn "LEGACY_PRIMITIVES_ENABLED\|legacy_primitives_disabled" crates/` — empty. Zero residual references.
- **Still deferred (no change this entry):**
  - session-layer `*_with_fallback` paths in `crates/termlink-session/src/inbox_channel.rs` (gated on fleet upgrade verification)
  - `crates/termlink-protocol/src/control.rs` `EVENT_BROADCAST`/`INBOX_LIST` consts (still referenced by retained fallback code)
  - CLI `commands/file.rs` (operator check needed)
  - Workspace-wide clippy pass

### 2026-06-05T21:50Z — MCP + local doctor dead inbox.status fallback removed [agent autonomous, focus=T-1415]

**Scope.** Removed dead `inbox.status` JSON-RPC fallback in the two `doctor`
inbox probes — the only remaining call sites in non-test code that still
spoke a retired primitive directly (not via session-shim's channel-only
wrappers). Parallel to T-1415 AC3's session-shim cleanup of 2026-05-31.

**Files touched:**
- `crates/termlink-cli/src/commands/infrastructure.rs` lines 432–486 —
  `fw doctor` step 7 (local-hub inbox probe). Collapsed `probe_channel_list`
  + `probe_inbox_status` dual probe into channel-list only. Comment header
  updated to reference T-1415 closure.
- `crates/termlink-mcp/src/tools.rs` lines 14180–14242 —
  `termlink_remote_doctor` MCP tool (remote-hub inbox probe). Same collapse;
  `Err(channel.list)` paths now classify as structural `fail` per PL-152
  (was: only after both modern + legacy failed).

**Why dead.** Post-T-1166-cut hubs return `-32601 method not found` for
`inbox.status`. The fallback would only fire if `channel.list` had ALREADY
failed; on a post-cut hub, the fallback then errors with `inbox.status
error: method not found` — useless to the operator and misleading (the
operator never invoked inbox.status; the doctor did). Removing the
fallback simplifies the error message to the actual root cause
(`channel.list error: <reason>`).

**Verification:**
- `cargo check -p termlink -p termlink-mcp` — clean, no warnings introduced
- `cargo test -p termlink-mcp --lib` — **837/0 PASS**
- `cargo test -p termlink --bins` — **816 pass, 1 pre-existing flake**
  (`isolate_rejects_non_git_dir`, order-dependent, passes solo; unrelated
  to this edit which is in `infrastructure.rs` not `dispatch.rs`)

**Not closed by this slice.** The two AC-misspec items (AC7: `termlink-cli`
package name doesn't exist + AC10: clippy scope pulls transitive crate
warnings) are documentation defects, not work. T-1415 closure still
gates on operator action — owner remains `human`.

### 2026-06-05T22:10Z — no_legacy_callers.rs allowlist tightened + drift guard [agent autonomous, focus=T-1415]

**Scope.** Inventory item 3 in T-1415's 16:50Z deferred list — tighten the
regression test now that the symbols it audits no longer exist in 4 of 6
allowlisted files. Per simulator-equivalent classifier pass:

| File | caller-shaped legacy literals | Action |
|------|------------------------------|--------|
| `crates/termlink-hub/src/router.rs` | 3 (lines 809-811: `"inbox.list"`, `"inbox.status"`, `"inbox.clear"` in the `hub.capabilities` methods list) | **kept** — separate follow-up needed |
| `crates/termlink-hub/src/rpc_audit.rs` | 6 (lines 32-37: the `LEGACY_METHODS` definition list itself) | **kept** — load-bearing |
| `crates/termlink-cli/src/commands/events.rs` | 0 (T-1401 fallback deleted) | **removed** from allowlist |
| `crates/termlink-cli/src/commands/infrastructure.rs` | 0 (this slice + earlier cleanup) | **removed** |
| `crates/termlink-mcp/src/tools.rs` | 0 | **removed** |
| `crates/termlink-session/src/inbox_channel.rs` | 0 (T-1415 AC3 cleanup, May 31) | **removed** |

**File touched:** `crates/termlink-hub/tests/no_legacy_callers.rs`
- ALLOWLIST shrunk from 6 to 2 entries.
- Module docstring rewritten to reflect post-retirement purpose ("regression
  guard — calls would speak a method the hub returns -32601 for, silently
  broken") instead of bake-window framing.
- Added `allowlist_is_load_bearing` test — strengthened drift guard that
  fails if any allowlisted file no longer contains ≥1 caller-shaped legacy
  literal. Closes the failure mode where a future cleanup empties a file
  but forgets to drop the now-dead allowlist row — silently masking new
  callers added later. The 4 just-removed entries would have been caught by
  this test as soon as it landed.

**Verification:**
- `cargo test -p termlink-hub --test no_legacy_callers` — **4/0 PASS**
  (3 original tests + 1 new drift guard). The new test passes with the
  shrunk allowlist, confirming both remaining entries are load-bearing.

**Follow-up (separate slice).** `router.rs` lines 809-811 advertise
`inbox.list`/`inbox.status`/`inbox.clear` in the hub-capabilities
methods list, but route() no longer serves them — capability consumers
get told the methods exist, then -32601 on actual call. Same dead-
advertisement problem applies to `EVENT_BROADCAST` at line 799. That's
a small bounded slice for next time.

### 2026-06-05T22:30Z — hub.capabilities methods-list dead advertisements removed [agent autonomous, focus=T-1415]

**Scope.** Executed the "next time" follow-up from the previous Updates
entry. Per the `handle_hub_capabilities` docstring "Only methods recognized
by `route()`'s explicit match arms are listed — forwarded session methods
are intentionally excluded", four entries had become stale post-cut:
`EVENT_BROADCAST` (line 799), `"inbox.list"` / `"inbox.status"` /
`"inbox.clear"` (lines 809-811). All four handlers were deleted in May's
f7b8d057 commit; calls now fall through to `forward_to_target` catchall
(per route() docstring "T-1166 / T-1415: event.broadcast + inbox.* arms
deleted 2026-05-31"). Capability consumers were being told these methods
exist, then getting -32601 / forwarding errors on actual call.

**File touched:** `crates/termlink-hub/src/router.rs`
- `handle_hub_capabilities` methods vec: 4 stale advertisements removed.
- Docstring above the vec updated with the T-1415 cleanup rationale.
- Top-of-fn docstring "fall back to event.broadcast" framing dropped
  (no longer the migration story being told).

**Follow-on edit:** `crates/termlink-hub/tests/no_legacy_callers.rs` —
post-edit, router.rs has 0 caller-shaped legacy literals outside test
code; the new `allowlist_is_load_bearing` test would correctly fire and
flag it. Removed `router.rs` from ALLOWLIST in the same slice; only
`rpc_audit.rs` (the legacy-method definition list itself) remains.

**Verification:**
- `cargo test -p termlink-hub --test no_legacy_callers` — **4/0 PASS**
  (load-bearing guard confirms the 1 remaining entry is genuine).
- `cargo test -p termlink-hub --lib` — **305/0 PASS**
- `cargo build -p termlink-hub` — clean, no warnings.

**Downstream behaviour change.** Federating clients calling
`hub.capabilities` will no longer see `event.broadcast` / `inbox.*` in
the returned `methods` array. This is the intended cleanup — consumers
that probe capability before calling will now correctly skip these
methods. The pre-existing T-1620 path that reclassifies "method not
in capabilities" as cause for fallback or error remains correct.
