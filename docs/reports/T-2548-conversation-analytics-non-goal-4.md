# T-2548 — Conversation-analytics MCP family vs charter non-goal #4

> **Retrospective consolidation.** Written 2026-08-14 under T-2716 from the
> recorded contents of `.tasks/active/T-2548-charter-non-goal-4-conversation-analytic.md`.
> The exploration it describes was carried out earlier; this file relocates that
> trail into `docs/reports/` per C-001, it does not add findings.
>
> **Status at consolidation:** assumptions stated, all three IW questions
> **deferred**. The subtract-vs-keep call is an explicit human sovereignty
> decision and remains unmade. Nothing here decides it.

## The finding

TermLink's charter (`docs/CHARTER.md`) names a hard **non-goal #4**: *the
substrate is mechanism, not policy.* The T-2468 purpose review's mandate is
subtract-and-deepen, and T-2471 / T-2478 already pruned 52 off-charter
social-analytics MCP tools (reactions, emoji, stars, pins, typing, polls).

This inception surfaces the **surviving sibling sub-family**: roughly 30-40 LIVE
(not deprecated) conversation-analytics MCP tools — thread-health scores,
reply/starter/pinner leaderboards, engagement-rate, response-latency, and message
volume/growth distributions.

These are *product analytics layered on top of the message substrate*. They trace
to none of the four charter verbs (discover / exchange durable messages / claim
work / control terminal sessions). They were verified LIVE via `is_deprecated()`
and found to have **zero first-party callers**.

This is the largest remaining off-charter tool surface the campaign has surfaced
after `orchestrator.route` (filed as T-2540).

> This family is the same ~28-tool set acknowledged in
> `.context/checks/charter-drift-allowlist` pending T-2548's resolution — the
> charter-drift canary counts and reports them as off-charter but does not fire,
> precisely so a daily alarm does not substitute for this decision. On resolution
> the allowlist entries are either deleted (tools deprecated) or re-justified
> against a charter verb.

## Assumptions

- **A-1** — the named analytics tools are LIVE, not deprecated, in the MCP
  registry. *(verify: `is_deprecated()` in `crates/termlink-mcp/src/tools.rs`)*
- **A-2** — zero first-party callers exist: no use in `scripts/`,
  `.claude/commands/`, or the CLI.
- **A-3** — they *analyze* message data (leaderboards, scores, distributions)
  rather than *retrieve* messages — i.e. policy/analytics, not the "exchange
  durable messages" mechanism. Retrieval tools such as `agent_search_thread` and
  `agent_thread_path` are explicitly OUT of scope.
- **A-4 (unverifiable here)** — no external peer or AEF process calls these MCP
  tools. This cannot be confirmed from `/opt/termlink` because of the T-559
  cross-project boundary. **This is the removal gate.**

A-4 is the load-bearing one. Zero *first-party* callers is not zero callers, and
the difference is exactly what a removal would risk.

## Open questions (all deferred to the human)

### IW-1 — Are there any external (non-first-party) consumers of these analytics MCP tools across the fleet or the AEF layer?

*confidence 1 · deferred*

In-repo first-party callers are zero. External callers cannot be confirmed from
`/opt/termlink` (T-559 boundary). **This is the removal gate** — a human or a
cross-project session must clear it before any subtract lands. Same gate shape as
T-2540 IW-1.

### IW-2 — Subtract-all, or keep a curated few?

*confidence 1 · deferred*

Zero first-party callers argues for subtract-all, but the human may want to retain
a small triage subset — `agent_stats` and `agent_digest` are the plausible
candidates for tools an operator actually reaches for. Part of the GO scoping
decision.

### IW-3 — Subtract, or grandfather?

*confidence 2 · deferred*

Does the charter get amended to sanction an analytics exception, or does the
family leave the substrate? This is a human sovereignty product-identity decision.

The **agent recommendation on record is GO to subtract**, restoring non-goal #4
and mirroring the T-2471 / T-2478 precedent set for the sibling social-analytics
set. It is a recommendation only; the decision is not made.
