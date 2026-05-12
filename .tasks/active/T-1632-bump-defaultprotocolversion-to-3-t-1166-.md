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
last_update: 2026-05-12T22:12:41Z
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
- [ ] `hub.capabilities` response includes `control_plane_version: 3` field (sourced from `termlink_protocol::CONTROL_PLANE_VERSION`)
- [ ] `hub.version` response includes `control_plane_version: 3` field (same source)
- [ ] `protocol_version` field still emits `DATA_PLANE_VERSION` (= 1) — no semantic change to the existing field
- [ ] Test `hub_version_returns_binary_version_and_protocol_version` updated to assert `control_plane_version` is present and equals `CONTROL_PLANE_VERSION`
- [ ] Cut-path test for `hub.capabilities` (post-cut variant) asserts `control_plane_version` field
- [ ] `cargo test -p termlink-hub --lib` passes
- [ ] `cargo check --workspace` passes
- [ ] No client-side reader of `protocol_version` breaks (verified via grep — no consumers exist today)

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

### 2026-05-12T21:55:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1632-bump-defaultprotocolversion-to-3-t-1166-.md
- **Context:** Initial task creation

### 2026-05-12T22:12:41Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
