---
id: T-908
name: "API relay governance — local proxy for deterministic tool gate enforcement via SSE stream rewriting"
description: >
  Inception: API relay governance — local proxy for deterministic tool gate enforcement via SSE stream rewriting

status: started-work
workflow_type: inception
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-09T10:54:14Z
last_update: 2026-04-09T10:54:59Z
date_finished: null
---

# T-908: API relay governance — local proxy for deterministic tool gate enforcement via SSE stream rewriting

## Problem Statement

Claude Code's PreToolUse hooks are the only enforcement mechanism for governance, but they have 5 documented failure modes (RFC anthropics/claude-code#45427): subagent bypass, silent hook failure, model self-modification of hooks/settings, alternative tool paths via Bash, and CLAUDE.md non-compliance. These failures break the framework's "nothing gets done without a task" guarantee and corrupt the data-to-prompt lifecycle chain.

**For whom:** Any team using Claude Code with governance requirements (us, and the broader community per the RFC).

**Why now:** The RFC is filed upstream but may take months. We need deterministic enforcement now. TermLink already has a hub/proxy architecture that could be extended to intercept the Anthropic API stream.

## Assumptions

<!-- Register with: fw assumption add "Statement" --task T-908 -->

- A-1: Claude Code respects `ANTHROPIC_BASE_URL` for all API calls including subagents
- A-2: Anthropic's SSE streaming protocol for tool_use blocks can be parsed incrementally without buffering the entire response
- A-3: Rewriting/stripping a tool_use block from the SSE stream does not break Claude Code's state machine
- A-4: Subagent processes inherit the parent's environment variables (including ANTHROPIC_BASE_URL)
- A-5: Latency overhead of a local proxy is negligible (<10ms per request)
- A-6: The Anthropic API authentication (API key header) can be forwarded transparently

## Exploration Plan

**Spike 1: Protocol analysis** (1h)
- Capture a real Claude Code session's HTTPS traffic to api.anthropic.com
- Document the SSE event format for tool_use blocks (content_block_start, content_block_delta, content_block_stop)
- Determine if tool_use blocks arrive as complete JSON or fragmented across deltas
- Deliverable: `docs/reports/T-908-protocol-analysis.md`

**Spike 2: ANTHROPIC_BASE_URL behavior** (30min)
- Test that Claude Code and subagents respect the env var
- Test what happens when the proxy returns modified responses
- Deliverable: findings in research artifact

**Spike 3: Minimal relay prototype** (2h)
- Rust async proxy using hyper/axum that forwards to api.anthropic.com
- Parse SSE stream, log tool_use blocks, forward unmodified
- Validate stream integrity (Claude Code still works normally)
- Deliverable: proof-of-concept in `crates/termlink-relay/` (prototype only)

**Spike 4: Tool gate enforcement** (1h)
- Add governance check to the relay: inspect tool_use, check for active task
- Rewrite blocked tool calls: replace with text block "GOVERNANCE: blocked"
- Test that Claude Code handles the rewritten response gracefully
- Deliverable: findings in research artifact

## Technical Constraints

- Anthropic API uses HTTPS with SSE (Server-Sent Events) for streaming
- SSE events are `\n\n`-delimited, each prefixed with `data: `
- Tool use blocks span multiple SSE events (start, deltas for input JSON, stop)
- Claude Code expects exact SSE format — malformed events will crash or hang the CLI
- The proxy must handle concurrent streams (multiple sessions/subagents)
- API key must be forwarded in `x-api-key` or `Authorization` header
- `ANTHROPIC_BASE_URL` is documented in Anthropic SDK — but Claude Code's exact handling needs verification

## Scope Fence

**IN scope:**
- Validate the local relay approach is technically feasible
- Determine if SSE tool_use rewriting works without breaking Claude Code
- Assess whether this closes all 5 failure modes from the RFC
- Go/no-go recommendation for building this into TermLink

**OUT of scope:**
- Full implementation of the relay (that's a build task if GO)
- MITM proxy approach (rejected in favor of ANTHROPIC_BASE_URL)
- Upstream CLI changes (that's the RFC's domain)
- Production-grade TLS handling, rate limiting, etc.

## Acceptance Criteria

### Agent
- [ ] Problem statement validated
- [ ] Assumptions tested
- [ ] Recommendation written with rationale

### Human
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

**GO if:**
- ANTHROPIC_BASE_URL is respected by Claude Code AND subagents
- SSE tool_use blocks can be parsed and rewritten without breaking the stream
- Claude Code handles rewritten responses (blocked tool calls) gracefully
- Latency overhead is <50ms per streamed response

**NO-GO if:**
- Claude Code hardcodes the API URL or ignores ANTHROPIC_BASE_URL for subagents
- Tool_use blocks are fragmented across SSE deltas in a way that prevents reliable parsing without full buffering
- Rewriting the stream causes Claude Code to crash or enter an unrecoverable state
- Anthropic adds certificate pinning or request signing that prevents proxying

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).

## Recommendation

<!-- REQUIRED before fw inception decide. Write your recommendation here (T-974).
     Watchtower reads this section — if it's empty, the human sees nothing.
     Format:
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence from exploration)
     **Evidence:**
     - Finding 1
     - Finding 2
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

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-04-09T10:54:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
