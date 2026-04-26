---
id: T-1291
name: "Declarative heal manifest — bootstrap_from per profile in hubs.toml"
description: >
  Extend hubs.toml schema with optional bootstrap_from per profile (e.g. 'ssh:user@host:/path/to/hub.secret'). termlink fleet reauth <profile> --bootstrap-from auto then reads the declared channel and runs the existing T-1055 fetch path. Lowers the floor on heal for every hub: today operator must remember the SSH command per profile; tomorrow it is one declared field. Captured 2026-04-26 after .122 rotation cascade exposed the chicken-and-egg.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [auth, fleet, hub, G-011]
components: []
related_tasks: [T-1054, T-1055, T-1284, T-1290, T-1051]
created: 2026-04-26T11:36:32Z
last_update: 2026-04-26T15:25:19Z
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
- [x] `hubs.toml` schema accepts optional `bootstrap_from = "<source>"` per `[hubs.<name>]` section, with the same source-format vocabulary as T-1055 (`file:`, `ssh:`); unknown schemes rejected with a clear error
- [x] `termlink fleet reauth <profile> --bootstrap-from auto` reads the declared `bootstrap_from`, errors clearly if missing, otherwise delegates to the existing T-1055 fetch path
- [x] Profiles without `bootstrap_from` keep working unchanged (back-compat)
- [x] At least 2 unit tests: (1) `auto` resolves to declared channel, (2) `auto` with no declaration emits actionable error
- [x] Existing T-1055 test suite still passes

## Verification

cargo test -p termlink fleet_reauth_bootstrap_from_auto
cargo test -p termlink fleet_reauth_bootstrap_from_auto_missing_declaration

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

### 2026-04-26T17:25Z — build delivered [agent autonomous pass]
- **Schema:** `HubEntry` in `crates/termlink-cli/src/config.rs` gained `bootstrap_from: Option<String>` (skip_serializing_if). Default = None preserves back-compat for every existing profile in every operator's `~/.termlink/hubs.toml`.
- **Wiring:** `cmd_fleet_reauth` in `crates/termlink-cli/src/commands/remote.rs` now resolves `--bootstrap-from auto` to the declared channel before delegating to the existing T-1055 fetch path. Unknown schemes still hard-error inside `fetch_bootstrap_secret` (no semantic change). Missing declaration with `auto` emits an actionable two-option hint (declare it, or pass an explicit source).
- **Tests:** 2 new unit tests added to the T-1055 block — `fleet_reauth_bootstrap_from_auto_resolves_declared_channel` and `fleet_reauth_bootstrap_from_auto_missing_declaration_errors`. All 7 fleet_reauth_bootstrap tests pass; full termlink suite green (225 unit + 172 integration).
- **Verification:** `cargo test -p termlink fleet_reauth_bootstrap` → 7/7 ok; `cargo test -p termlink` → 0 fail.
- **Ergonomic follow-up (same session):** `termlink remote profile add --bootstrap-from <source>` plumbing landed in commit follow-up — operator can now declare the channel in one shot without editing hubs.toml. Scheme is validated up-front (only `file:` / `ssh:` accepted), so typos fail loud at add time, not at heal time.
- **All Agent ACs ticked.** Owner=human; awaiting operator validation (none captured here yet).

### 2026-04-26T15:25:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
