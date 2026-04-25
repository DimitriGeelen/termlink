# T-922: Codify MCP auto-exposure — DEFER

**Task:** T-922 (inception, DEFER 2026-04-12, owner: human)
**Status:** WORK-COMPLETED — minimal exploration, decision is DEFER
**Backfilled:** 2026-04-25 (T-1258, audit cleanup)

## Problem statement

Discovered via T-920 RCA: shipping CLI-only cross-host features
(T-163 / T-164 / T-182 / T-186) left MCP agents blind for months.
Every new CLI command should automatically be MCP-reachable. Currently
MCP tools are hand-crafted in `crates/termlink-mcp/src/tools.rs`. Process
improvement to ensure CLI-MCP parity.

## Options considered

- Code-gen MCP wrappers from the CLI command enum at build time.
- Lint that diffs `cli.rs` vs `tools.rs` and fails on missing pairings.
- Pre-commit hook blocking new `Command` variants without a matching
  MCP tool.
- Runtime registration pattern.

## Decision: DEFER

**Rationale:** Current MCP tools cover all active commands.
Process improvement, not urgent. The first enforcement step (any of the
options above) is itself a project-sized effort, and the failure mode
this would catch — agent blindness to a freshly-shipped CLI command —
is now caught earlier by routine MCP-tool review during the same PR
that adds the CLI command.

GO is preserved as an option for the future: if a wave of CLI commands
ships without matching MCP tools, the lint option (smallest diff,
deterministic, no codegen complexity) is the recommended starting point.

## References

- T-920 (the RCA that originated this concern).
- `crates/termlink-mcp/src/tools.rs` (current MCP tool registry).
- T-1258 (this artifact backfill).
