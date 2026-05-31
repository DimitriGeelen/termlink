---
id: T-1632
name: "Bump default_protocol_version() to 3 (T-1166 AC #6 follow-up)"
description: >
  T-1166 AC #6 carve-out. CONTROL_PLANE_VERSION (lib.rs:29) was bumped 2->3 at cut time, but hub.capabilities.protocol_version is sourced from default_protocol_version() at crates/termlink-protocol/src/control.rs:239 which still returns 1. Live verification on .122 (2026-05-12T21:50Z) confirmed protocol_version=1 in the post-cut response. Functional cut works (-32601 + retired methods absent); version-handshake half is incomplete. Older clients see method-not-found rather than PROTOCOL_VERSION_TOO_OLD.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [T-1166, protocol, cut-followup]
components: []
related_tasks: [T-1166]
created: 2026-05-12T21:55:18Z
last_update: 2026-05-15T20:46:27Z
date_finished: null
---

# T-1632: Bump default_protocol_version() to 3 (T-1166 AC #6 follow-up)

## Context

T-1166 AC #6 asked for a `protocol_version` bump in the post-cut hub responses. Live verification on .122 surfaced `protocol_version=1`. Code-side investigation (2026-05-13) found the AC premise was wrong: `hub.capabilities` and `hub.version` emit `"protocol_version": DATA_PLANE_VERSION` (lib.rs:13 = 1), NOT `default_protocol_version()` (control.rs:239). `default_protocol_version()` is only used as `#[serde(default = ...)]` for deserializing `Capabilities` when a peer omits the field; changing it has zero wire effect and silently relabels v1 clients.

The cut bumped `CONTROL_PLANE_VERSION` (lib.rs:29) 2→3, which is the right axis (control-plane semantics changed: legacy primitives retired) but the constant has no reader — it never appears in any hub-emitted response.

Re-scoped fix: emit `CONTROL_PLANE_VERSION` on the wire as a NEW sibling field `control_plane_version` in both `hub.capabilities` and `hub.version` responses. Leaves `protocol_version` (= DATA_PLANE_VERSION) untouched (data-plane frame format is unchanged). Purely additive — existing clients ignore unknown fields.

Rejected alternatives:
- Bump `default_protocol_version()` → wrong axis, breaks v1-client default semantics.
- Bump `DATA_PLANE_VERSION` 1→3 → conflates frame-format axis with control-plane axis.
- Reuse `protocol_version` field for both axes → permanent ambiguity.

## Acceptance Criteria

### Agent
- [x] `hub.capabilities` response includes `control_plane_version: 3` field (sourced from `termlink_protocol::CONTROL_PLANE_VERSION`) — router.rs:1033 (commit e89ecb47)
- [x] `hub.version` response includes `control_plane_version: 3` field (same source) — router.rs:895 (commit e89ecb47)
- [x] `protocol_version` field still emits `DATA_PLANE_VERSION` (= 1) — no semantic change to the existing field
- [x] Test `hub_version_returns_binary_version_and_protocol_version` updated to assert `control_plane_version` is present and equals `CONTROL_PLANE_VERSION` — passes
- [x] Cut-path test for `hub.capabilities` (post-cut variant) asserts `control_plane_version` field — `cut_path::capabilities_emits_control_plane_version` passes under `--features legacy_primitives_disabled` (6 cut_path tests pass)
- [x] `cargo test -p termlink-hub --lib` passes — `hub_version_returns_binary_version_and_protocol_version` 1 passed; `hub_capabilities` 1 passed + 2 ignored (pre-cut tests, expected per T-1415)
- [x] `cargo check --workspace` passes — clean (1 pre-existing unrelated warning in termlink-mcp)
- [x] No client-side reader of `protocol_version` breaks (verified via grep — no consumers exist today)

### Human
- [ ] [REVIEW] On next .122 deploy (after T-1166 bake clears), `hub.capabilities` returns `control_plane_version: 3` alongside `protocol_version: 1`.
  **Steps:**
  1. After deploying the new build to .122: `termlink remote call --hub 192.168.10.122:9100 --method hub.capabilities`
  2. Inspect the JSON `result` field
  **Expected:** Both `protocol_version: 1` and `control_plane_version: 3` present
  **If not:** Capture the response, attach to this task, and revert the deploy (rollback path: previous binary at `/tmp/termlink.pre-T1166`)

## Verification

cargo check -p termlink-hub
cargo test -p termlink-hub --lib hub_version_returns
cargo test -p termlink-hub --lib hub_capabilities
grep -q "control_plane_version" crates/termlink-hub/src/router.rs

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

## Updates

### 2026-06-01T — Human REVIEW evidence captured live from .122 (T-1903 deploy) [agent autonomous]

The Human REVIEW recipe waited on a fresh deploy to .122. T-1903 delivered that on 2026-05-31 (commit 8698f25e). Captured the canonical evidence via `termlink_remote_call` MCP tool with `execute` scope:

```
$ termlink_remote_call(hub=192.168.10.122:9100, method=hub.capabilities, scope=execute)

{
  "ok": true,
  "result": {
    "control_plane_version": 3,        ← AC #6 evidence: bumped to 3 ✓
    "protocol_version": 1,             ← unchanged, no semantic regression ✓
    "hub_version": "0.11.473",
    "features": {
      "legacy_primitives": false       ← bonus: T-1166 cut is LIVE on .122
    },
    "methods": [ /* 28 RPCs incl hub.legacy_usage (T-1432) */ ]
  }
}
```

The two-axis split (control_plane_version for capability surface, protocol_version for frame format) is visible to the wire. The Human AC is materially satisfied.

**Operator-actionable:** ready to tick the [REVIEW] box + `fw task update T-1632 --status work-completed`.

### 2026-05-15T20:46Z — .121 ring20-dashboard deploy complete (second production hub)

- Deployed musl 0.9.2127 to ring20-dashboard:9100. Same binary as .122.
- **Pre-state surprise:** .121 had TWO hub processes — canonical PID 1895580 (May 3, /var/lib + TCP) and rogue PID 2391097 (May 15 07:08, /tmp UDS only, no `--tcp`). Killed the rogue first (T-1641 filed to investigate where the rogue spawn comes from); then swapped the canonical.
- **Procedure:** `fleet-deploy-binary.sh --probe` stage → atomic `mv` → detached relaunch via `nohup bash /tmp/relaunch.sh` (bypassed `hub-binary-swap.sh` for tight control over the dual-hub edge; the T-1640 pgrep fix is in the binary but wasn't exercised by this deploy).
- **Live wire verification (REVIEW AC):**
  - `hub.version` → `{hub_version: 0.9.2127, protocol_version: 1, control_plane_version: 3}` ✓
  - `hub.capabilities` (scope=execute) → same + `legacy_primitives: false`, 24 methods, no retired names ✓
  - `hub.legacy_usage` → 0 calls / 7d — T-1166 cut applied cleanly, no consumer broken ✓
- Persistence holding: `hub.secret` `1792190e37b9c033...` and `hub.cert.pem` `9dcc461cfb98dd7d...` unchanged across swap.
- Single hub post-deploy: PID 2704841, exe `/usr/local/bin/termlink` (no `(deleted)` marker), started 2026-05-15T20:45:11. `/tmp/termlink-0/` left with vestigial files (no socket bound) — harmless, /tmp gets wiped on next boot anyway.
- **Both production hubs (.122 + .121) now on 0.9.2127 with control_plane_version=3.** REVIEW AC satisfied across the fleet.

### 2026-05-15T20:11Z — .122 deploy complete, wire emit confirmed live

- Deployed musl 0.9.2127 (sha `416e980ece6f9692...`) to ring20-management:9100 via `scripts/fleet-deploy-binary.sh --probe` (stage) + `scripts/hub-binary-swap.sh` (binary mv) + detached relaunch (the swap script's PID-resolution missed the long-running orphan, swap completed; manual SIGTERM + nohup relaunch closed it).
- **Live wire verification (this task's REVIEW AC):**
  - `hub.version` → `{"hub_version": "0.9.2127", "protocol_version": 1, "control_plane_version": 3}` ✓
  - `hub.capabilities` (scope=execute) → same + `legacy_primitives: false`, 24 methods, no retired names ✓
- Persistence holding: `hub.secret` sha `3dd9d01afe4ec599...` and `hub.cert.pem` sha `2355a206cd9c306d...` unchanged across swap → no client re-auth needed.
- New hub PID 2506810, exe=`/usr/local/bin/termlink` (no `(deleted)` marker), started 2026-05-15T20:10:25.
- Human REVIEW step is satisfied by the evidence above. Ready to tick once you've eyeballed.

### 2026-05-15T19:54Z — fresh release binary built (deploy-ready)

- Built `target/release/termlink` 0.9.2125 (sha256 prefix `611d0013a748eb70`).
- Binary includes `e89ecb47` (control_plane_version emit, this task) plus T-1633 / T-1636 / T-1637 cut-followup commits.
- Re-ran `cargo test -p termlink-hub --lib --release --features legacy_primitives_disabled cut_path` against the release build: 6/6 PASS, including `cut_path::capabilities_emits_control_plane_version` which directly asserts the AC.
- Deploy on .122 is no longer binary-blocked. Human REVIEW step from this task's `### Human` AC can proceed against this binary.

### 2026-05-12T21:55:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1632-bump-defaultprotocolversion-to-3-t-1166-.md
- **Context:** Initial task creation

### 2026-05-12T22:12:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
