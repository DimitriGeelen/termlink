---
id: T-1405
name: "Add features.legacy_primitives flag to hub.capabilities response"
description: >
  Add features.legacy_primitives flag to hub.capabilities response

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-29T19:20:34Z
last_update: 2026-04-29T19:20:34Z
date_finished: null
---

# T-1405: Add features.legacy_primitives flag to hub.capabilities response

## Context

The T-1166 migration guide (T-1402) tells consumers to check
`hub.capabilities.features.legacy_primitives` on connect, fail fast if
`false`, and link to the migration doc. But the field doesn't exist yet —
the current `hub.capabilities` response has only `methods`, `hub_version`,
`protocol_version`. This task adds the `features` object with one initial
key set to `true` (legacy primitives are still served). When T-1166
lands, the value flips to `false`. Downstream consumers can wire startup
checks NOW against the existing `true` value, then their failure path
trips automatically when the cut happens.

Forward-compatible: existing clients that don't read the new field are
unaffected. Wire shape:

```json
{
  "methods": [...],
  "hub_version": "0.9.x",
  "protocol_version": 1,
  "features": {
    "legacy_primitives": true
  }
}
```

## Acceptance Criteria

### Agent
- [x] `handle_hub_capabilities` in `crates/termlink-hub/src/router.rs` returns a `features` object with `legacy_primitives: true`
- [x] Existing `methods`, `hub_version`, `protocol_version` keys remain unchanged
- [x] Unit test added for the new field — `hub_capabilities_advertises_legacy_primitives_feature_flag`, passes
- [x] Live verification: raw JSON-RPC `hub.capabilities` against the local hub returns `features: {"legacy_primitives": True}` (verified after binary refresh + hub restart, hub PID 4049739, version 0.9.1574)
- [x] Migration guide updated to match actual wire shape (was `capabilities.legacy_primitives`, corrected to `features.legacy_primitives`; added cross-link to T-1405)
- [x] cargo build / clippy / test clean for `termlink-hub`

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

cargo build -p termlink-hub 2>&1 | grep -qE "Finished"
cargo clippy -p termlink-hub --tests -- -D warnings 2>&1 | grep -qE "Finished"
cargo test -p termlink-hub --lib hub_capabilities 2>&1 | grep -qE "test result: ok"
grep -q '"features"' crates/termlink-hub/src/router.rs
grep -q "legacy_primitives" crates/termlink-hub/src/router.rs

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

### 2026-04-29T19:20:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1405-add-featureslegacyprimitives-flag-to-hub.md
- **Context:** Initial task creation
