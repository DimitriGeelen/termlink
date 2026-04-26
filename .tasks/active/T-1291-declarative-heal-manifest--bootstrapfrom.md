---
id: T-1291
name: "Declarative heal manifest — bootstrap_from per profile in hubs.toml"
description: >
  Extend hubs.toml schema with optional bootstrap_from per profile (e.g. 'ssh:user@host:/path/to/hub.secret'). termlink fleet reauth <profile> --bootstrap-from auto then reads the declared channel and runs the existing T-1055 fetch path. Lowers the floor on heal for every hub: today operator must remember the SSH command per profile; tomorrow it is one declared field. Captured 2026-04-26 after .122 rotation cascade exposed the chicken-and-egg.

status: captured
workflow_type: build
owner: human
horizon: next
tags: [auth, fleet, hub, G-011]
components: []
related_tasks: [T-1054, T-1055, T-1284, T-1290, T-1051]
created: 2026-04-26T11:36:32Z
last_update: 2026-04-26T11:36:32Z
date_finished: null
---

# T-1291: Declarative heal manifest — bootstrap_from per profile in hubs.toml

## Context

T-1054 (Tier-1 print-the-incantation) and T-1055 (Tier-2 `--bootstrap-from <source>`) made heal mechanically possible, but the operator still has to remember which OOB channel to use per hub. In the 2026-04-26 ring20-management cascade we hit this directly: `fleet reauth ring20-management --bootstrap-from ssh:192.168.10.122` was the exact right command, but only because the operator happened to remember it. For a fleet of N hubs each with their own correct anchor, this is per-incident lookup work.

Declarative manifest moves the per-profile OOB channel into `hubs.toml` itself:

```toml
[hubs.ring20-management]
address     = "192.168.10.122:9100"
secret_file = "~/.termlink/secrets/ring20-management.hex"
bootstrap_from = "ssh:192.168.10.122"   # NEW: declared trust anchor
```

Then `termlink fleet reauth ring20-management --bootstrap-from auto` reads the declared channel and dispatches to the existing T-1055 fetch path. Operator types one flag, not a remembered incantation. R2 (out-of-band rule) is preserved because the `bootstrap_from` value is set once at deploy time on a trusted channel, and the schema rejects sources that depend on the failed termlink auth.

Depends on T-1290 in spirit only — if T-1290 eliminates rotations on .122, the value of T-1291 drops because heal becomes rare. But T-1291 still helps every other hub that experiences a one-off rotation (operator regen, runtime_dir migration), so it is independently useful. Not a hard dependency.

## Acceptance Criteria

### Agent
- [ ] `hubs.toml` schema accepts optional `bootstrap_from = "<source>"` per `[hubs.<name>]` section, with the same source-format vocabulary as T-1055 (`file:`, `ssh:`); unknown schemes rejected with a clear error
- [ ] `termlink fleet reauth <profile> --bootstrap-from auto` reads the declared `bootstrap_from`, errors clearly if missing, otherwise delegates to the existing T-1055 fetch path
- [ ] Profiles without `bootstrap_from` keep working unchanged (back-compat)
- [ ] At least 2 unit tests: (1) `auto` resolves to declared channel, (2) `auto` with no declaration emits actionable error
- [ ] Existing T-1055 test suite still passes

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

### 2026-04-26T11:36:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1291-declarative-heal-manifest--bootstrapfrom.md
- **Context:** Initial task creation
