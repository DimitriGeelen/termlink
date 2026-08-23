# T-2549 — `termlink dispatch` vs charter non-goal #4

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/active/T-2549-charter-non-goal-4-termlink-dispatch-orc.md`.
> The exploration it describes was carried out earlier; this file relocates that
> trail into `docs/reports/` per C-001, it does not add findings.
>
> **Status at consolidation:** assumptions stated (A-1, A-2 confirmed); both IW
> questions **deferred**. The subtract-vs-keep call is an explicit human
> sovereignty decision and remains unmade.

## The finding

Charter non-goal #4 (`docs/CHARTER.md`) reads: *"Not a workflow or orchestration
engine. TermLink provides the primitives... The substrate stays mechanism, not
policy."*

The `termlink dispatch` verb (`crates/termlink-cli/src/commands/dispatch.rs`, 1138
LOC) and its MCP tools `termlink_dispatch` / `termlink_dispatch_status` describe
themselves, in their own module docstring, as *"atomic spawn+tag+collect for
multi-worker orchestration... Provides a structural guarantee that collect is
always wired, replacing manual dispatch scripts."*

That is a workflow/orchestration engine baked into the substrate — the exact thing
non-goal #4 reserves for the AEF layer. It is a **distinct surface** from
`orchestrator.route` (filed as T-2540).

**The honest counter-case, recorded alongside the finding:** `dispatch` only
*composes* charter-legal session primitives — spawn, tag, collect — so it can be
read as a convenience wrapper rather than as policy. That reading is defensible,
and it is why this one is genuinely harder than its siblings.

## Assumptions

- **A-1** — `dispatch` implements orchestration *policy* (fan out N workers, wire
  collect), not a mechanism-only primitive. **CONFIRMED**: the docstring
  self-describes as orchestration.
- **A-2** — MCP `termlink_dispatch` / `_status` are LIVE, not deprecated.
  **CONFIRMED.**
- **A-3** — near-zero first-party callers; at most one demo/eval script.
- **A-4 (unverifiable here)** — no external peer or AEF process calls `dispatch`.
  Blocked by the T-559 cross-project boundary. **This is the removal gate.**

## Open questions (both deferred to the human)

### IW-1 — Are there any external (non-first-party) callers of `termlink dispatch` or its MCP tools across the fleet or the AEF layer?

*confidence 1 · deferred*

First-party callers are near zero — at most one eval script. External callers
cannot be confirmed from `/opt/termlink` (T-559). **Removal gate.**

### IW-2 — Is `dispatch` policy (subtract) or a thin convenience wrapper over charter-legal primitives (keep)?

*confidence 2 · deferred*

The module docstring self-describes as *"orchestration... replacing dispatch
scripts"*, which reads as policy. But it only composes spawn / tag / collect, all
charter-legal, so the convenience-wrapper reading holds up.

**This is the crux of the human decision, and it is more nuanced than its two
siblings.** Both T-2548 (the analytics family) and T-2540 (`orchestrator.route`)
carry bespoke policy logic that `dispatch` does not. Whatever is decided for those
two does not automatically settle this one.
