# T-105: Tool Call Reporting — Inception Research

> Task: T-105 | Started: 2026-03-12 | Type: inception
> Dependency: T-104 (capture store) — completed

## Problem Statement

Tool call data exists in `.context/telemetry/tool-calls.jsonl` (T-111/T-112) and
error analysis is available via `analyze-errors.py` (T-113). Missing: a general-purpose
tool call statistics report and handover integration.

## Inception Questions — Answered

### 1. Delivery format
**Decision: Terminal.** No web UI needed at this stage.
- Error report: `analyze-errors.py` (T-113, already built)
- General stats: `tool-stats.py` (to be built)
- Rationale: Terminal output is sufficient for single-user workflow. Web UI is over-engineering.

### 2. Filters/dimensions
From T-113 patterns and T-111 schema:
- By session (`--session UUID`)
- By tool type (breakdown in stats output)
- By error/success (error rate per tool)
- By sidechain vs main (sub-agent attribution)

### 3. Handover integration
Add tool call summary numbers to handover agent output:
- Total calls this session
- Error count and rate
- Most-used tool
- Top error pattern

### 4. Drill-down
Not needed for terminal format. The `tool_use_id` in each record links back to
the source JSONL for manual investigation when needed.

## Decision

**GO** — Build `tool-stats.py` for general reporting. T-113 already delivers error analysis.
