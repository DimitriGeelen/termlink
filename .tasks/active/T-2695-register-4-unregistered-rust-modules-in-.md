---
id: T-2695
name: "Register 4 unregistered Rust modules in the component fabric"
description: >
  retention_sweeper.rs, fleet_presence.rs, identity_dir.rs and ws_consumer.rs have no
  fabric cards, so `fw fabric deps/impact/blast-radius` cannot reason about them — impact
  analysis is silently incomplete for four modules, including two that sit on charter-core
  paths.
status: started-work
workflow_type: build
horizon: now
owner: claude-code
created: 2026-08-20
last_update: 2026-08-20T01:05:18Z
tags: [governance, fabric, impact-analysis]
---

# T-2695: Register 4 unregistered Rust modules in the component fabric

## Context

`fw fabric drift` reports four source files with no component card:

```
! crates/termlink-hub/src/retention_sweeper.rs
! crates/termlink-session/src/fleet_presence.rs
! crates/termlink-session/src/identity_dir.rs
! crates/termlink-session/src/ws_consumer.rs
```

The Component Fabric is what `fw fabric blast-radius` reads before a commit to answer "what
does this change break?". A file with no card is invisible to that query — so the answer is
not "nothing depends on it", it is "I cannot see it", and the two are indistinguishable in
the output. CLAUDE.md instructs running `fw fabric blast-radius HEAD` when source files
change; for these four that instruction has been quietly returning an incomplete answer.

Two of the four are on charter-core paths, which is what makes this worth doing rather than
tolerating:

- `retention_sweeper.rs` (T-2427) — the opt-in in-process retention sweep. It exists
  *because* operator sweep crons are unreliable (T-1991 recurred on .121 when the cron was
  never installed). A module whose whole purpose is compensating for missing enforcement is
  a poor one to have missing from impact analysis.
- `fleet_presence.rs` (T-2275) — the shared pure resolver for `agent-presence` heartbeats;
  the semantic core of "contact a peer by name", used by *both* the CLI and the MCP handler.
  A change here has two consumers, which is exactly the fact blast-radius exists to surface.

## Approach

`fw fabric register <path>` for each, then set `subsystem` and `purpose` from each module's
own doc comment rather than leaving the auto-generated `unknown` — an `unknown` subsystem
answers the drift check without answering the question the card exists for.

## Acceptance Criteria

### Agent
- [x] All four files have component cards
- [x] Each card carries a real `subsystem` (not `unknown`)
- [x] Each card carries a `purpose` derived from the module's doc comment
- [x] `fw fabric drift` reports 0 unregistered

## Verification

bash -c 'test "$(.agentic-framework/bin/fw fabric drift 2>/dev/null | sed -n "s/.*unregistered: \([0-9]*\).*/\1/p")" = "0"'

## Decisions

**Enrich rather than just register.** `fw fabric register` produces a card with
`subsystem: unknown`, which clears the drift WARN without making the card useful. The WARN
is a proxy for "impact analysis is blind here"; satisfying the proxy while leaving the
blindness would be scoring the metric rather than fixing the problem.
