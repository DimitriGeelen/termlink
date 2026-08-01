# P4 — MCP surface reduction (T-2468 purpose review)

The T-2468 purpose review found TermLink **over-built in breadth, incomplete in
core**. A large block of MCP tools implement social-media-style analytics
(reactions, emoji leaderboards, stars, pins, typing indicators, polls) that do
not trace to the coordination-substrate charter ("discover each other, exchange
durable messages, claim work, control terminal sessions"). P4 reduces that
agent-facing surface.

Distinct from arc-005 (which *shortened* tool descriptions to reclaim token
budget); P4 *removes tools from the active surface*.

## Stages

### Stage 1 — DELETE zero-consumer tools (T-2471, done)

12 Group-C social-analytics MCP tools with **zero** consumers (no CLI twin, no
slash command, no script) were deleted outright: `silent_senders`,
`peer_engagement`, `activity_rhythm`, `engagement_rate`, `msg_growth_rate`,
`co_posters`, `daily_volume`, `post_streak`, `silence_gap`, `age_distribution`,
`thread_size_dist`, `burst_detect`. `help_categories()` 278 → 266; −1130 lines.

### Stage 2 — DEPRECATE CLI-entangled tools (T-2478, this record)

**Human directive (2026-08-02): deprecate, not delete.** Groups A/B/D have CLI
twins and could be scripted against, so they are **deprecated (reversible), not
removed** — every tool stays callable:

- **Group A — reactions / emoji (13):** `agent_react`, `agent_reactions`,
  `agent_reactions_of`, `agent_reactions_by`, `agent_reaction_rate`,
  `agent_reaction_summary`, `agent_top_reacted`, `agent_emoji_stats`,
  `agent_emoji_users`, `channel_react`, `channel_reactions_of`,
  `channel_reactions_on`, `channel_emoji_stats`.
- **Group B — stars / pins / leaderboards (16):** `agent_star`, `agent_starred`,
  `agent_starred_by`, `agent_starred_history`, `agent_top_starrers`, `agent_pin`,
  `agent_pinned`, `agent_pinned_by`, `agent_pinned_history`, `agent_pin_history`,
  `agent_top_pinners`, `channel_star`, `channel_starred`, `channel_pin`,
  `channel_pinned`, `channel_pin_history`.
- **Group D — typing / polls (11):** `agent_typing`, `agent_typers`,
  `agent_poll_start`, `agent_poll_vote`, `agent_poll_end`, `channel_typing_emit`,
  `channel_typing_list`, `channel_poll_start`, `channel_poll_vote`,
  `channel_poll_end`, `channel_poll_results`.

**Deprecation mechanism** (mirrors the existing `remote_inbox_*` T-1166 pattern):

- MCP `help_categories()` entry: description appended with
  `(deprecated P4/T-2478) (use termlink_channel_post instead)` — the marker trips
  the registry's `is_deprecated()` derivation, and the `(use … instead)` hint
  satisfies the `every_deprecated_tool_has_replacement_hint` invariant test.
  Reactions / stars / pins / polls / typing are all specialized posts, so the
  replacement primitive is `termlink_channel_post`.
- MCP `#[tool(description)]`: prefixed `[DEPRECATED — use termlink_channel_post]`
  (the agent-facing runtime signal).
- CLI twins: `#[command(hide = true)]` (removed from `--help`) while remaining
  functional.

Nothing is deleted; `git revert` restores the full surface. Deletion is a
**later** step, gated on a deprecation soak and a separate human go-ahead.

**Consumer safety:** a grep of `.claude/commands/` and `scripts/` confirmed **no**
slash command or helper invokes any of the 40 tools — same zero-consumer profile
as the Stage-1 deletions, so deprecation breaks no existing workflow.

`AgentAction::Poll` (the event-bus poll) is explicitly out of scope — it is a
coordination primitive, not a social poll.
