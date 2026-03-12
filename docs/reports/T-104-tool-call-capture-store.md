# T-104: Tool Call Capture Store — Inception Research

> Task: T-104 | Started: 2026-03-12 | Type: inception
> Design principle: Capture everything first, report later.

## Problem Statement

No persistent record of tool call activity exists across sessions. Each session's
tool calls live in the JSONL transcript, but transcripts are session-scoped and not
aggregated. After context compaction or session end, data is effectively inaccessible
without parsing raw files.

## Data Sources

### Main JSONL Transcript
- Location: `~/.claude/projects/<project-encoded>/<session-uuid>.jsonl`
- Contains: `user`, `assistant`, `progress`, `system`, `file-history-snapshot` events
- Tool calls: embedded in `assistant` events as structured `tool_use` content blocks
- Tool results: in subsequent `user` events as `tool_result` content blocks
- API errors: top-level events with `isApiErrorMessage: true` + `error` field (3 in sample session)

### Sidechain JSONL (Sub-agent Transcripts)
- Location: `~/.claude/projects/<project>/<session>/subagents/agent-<id>.jsonl`
- **CORRECTION:** Same error structure as main JSONL — NOT richer. Advantage is per-agent isolation for attribution.
- API errors: `system` events with `subtype: api_error`, include retry metadata (`retryAttempt`, `maxRetries`, `retryInMs`)
- 66 files across 2 sessions, 20KB–27MB each, avg 3.3MB

## Investigation Findings

### Main JSONL Tool Call Structure

Analyzed: `f45cfdad-ec0a-4ade-8066-3d0824af608e.jsonl` (50.3 MB, multi-day session)

**Tool call (in `assistant` events):**
```json
{
  "type": "tool_use",
  "id": "toolu_01WdvthbVB9g...",
  "name": "Bash",
  "input": {"command": "...", "description": "..."},
  "caller": {"type": "direct"}
}
```
Parent event includes: `sessionId`, `timestamp`, `cwd`, `gitBranch`, `slug`, `uuid`, `parentUuid`.
Message-level: `model`, `usage` (input/output tokens, cache stats), `stop_reason`.

**Tool result (in `user` events):**
```json
{
  "type": "tool_result",
  "tool_use_id": "toolu_...",
  "content": "output string",
  "is_error": false
}
```
Parent event has: `toolUseResult` (stdout/stderr), `sourceToolAssistantUUID` (links to calling assistant event).

**Volume from sample session:**
| Metric | Value |
|--------|-------|
| Total events | 20,500 |
| tool_use blocks | 2,751 |
| tool_result blocks | 2,749 |
| Tool errors (is_error: true) | 240 (8.7% error rate) |
| API errors | 3 |
| progress events | 12,428 |
| Avg bytes/tool_use block | ~19KB (includes full input) |

**Tool breakdown:** Bash: 1217, Edit: 645, Read: 508, Write: 169, Grep: 142, Agent: 32, TaskOutput: 16, ToolSearch: 12, Glob: 9, WebFetch: 1

**Timestamps:** On individual events (20,067/20,500). ISO 8601: `"2026-03-08T15:16:28.760Z"`.

**No task context:** Task IDs (T-XXX) appear only in tool input/output strings, not as structured metadata. Would need enrichment from focus.yaml state at capture time.

### Sidechain Tool Call Structure

**Same format as main JSONL.** Tool calls and results are identical content blocks.

**Linking fields:**
- `sessionId` — matches parent session UUID
- `agentId` — unique per sub-agent (prefix: `a` for regular, `acompact-` for compaction)
- `parentToolUseID` — links to the spawning tool_use in parent
- `sourceToolAssistantUUID` — references the dispatching assistant turn
- `isSidechain: true` — boolean flag

**API errors in sidechains:** `system` events with `subtype: api_error`:
```json
{
  "type": "system", "subtype": "api_error", "level": "error",
  "error": {"status": 500, "headers": {...}, "requestID": "..."},
  "retryAttempt": 1, "maxRetries": 10, "retryInMs": 540.44
}
```

## Storage Format Decision

**Recommendation: JSONL** (`.context/telemetry/tool-calls.jsonl`)

Rationale:
- Source data is already JSONL — consistent format, no impedance mismatch
- Append-only writes are the dominant pattern (capture, not query)
- Python one-liners for ad-hoc queries (`python3 -c "..."`)
- Human-inspectable with `head`, `tail`, `jq`
- No external dependencies (no SQLite Python module needed)
- Volume is manageable: ~2,750 calls/session × ~500 bytes/record (metadata-only) = ~1.4 MB/session
- Can always migrate to SQLite later if query patterns demand indexing

SQLite rejected: Overkill for current needs. Binary format makes debugging harder. Cross-session aggregation is achievable with JSONL + `jq` or Python. Revisit if T-105 reporting needs complex joins.

## Schema Design

Per tool call record (metadata-only, NOT full input/output):

```json
{
  "ts": "2026-03-08T15:16:28.760Z",
  "session_id": "f45cfdad-ec0a-4ade-8066-3d0824af608e",
  "task": "T-104",
  "tool": "Bash",
  "tool_use_id": "toolu_01WdvthbVB9g...",
  "is_error": false,
  "error_summary": null,
  "input_size": 245,
  "output_size": 1820,
  "model": "claude-sonnet-4-6",
  "tokens_in": 12500,
  "tokens_out": 450,
  "is_sidechain": false,
  "agent_id": null,
  "cwd": "/Users/dimidev32/001-projects/010-termlink"
}
```

**Size estimate:** ~350 bytes/record × 2,750 calls/session = ~960 KB/session. At 1 session/day: ~30 MB/month. Acceptable.

**Error records:** When `is_error: true`, `error_summary` contains the first 200 chars of the error content. Full error content stays in the source JSONL — we store a pointer (session_id + tool_use_id) for drill-down.

**Fields NOT captured (by design):**
- Full tool input (can be megabytes for Write/Edit — stored in source JSONL)
- Full tool output (same reason)
- Progress events (noise — 12K events per session, no diagnostic value)
- file-history-snapshot events (not tool calls)

## Capture Timing Decision

**Recommendation: Option A — Batch extraction at session end / PreCompact**

Rationale:
- Zero runtime overhead during sessions
- All data already in JSONL — no duplicate writes
- PreCompact hook already exists and runs reliably
- Session end is the natural aggregation boundary
- Aligns with `fw transcripts` agent (T-110) which already processes session dirs

Implementation sketch:
```bash
# In PreCompact hook or session-end agent:
python3 agents/telemetry/extract-tool-calls.py \
  --session $SESSION_ID \
  --task $(cat .context/working/focus.yaml | grep current_task | awk '{print $2}') \
  >> .context/telemetry/tool-calls.jsonl
```

PostToolUse rejected: Would add ~5ms per tool call (file open/append/close). With 2,750 calls/session, that's ~14 seconds of cumulative I/O overhead. Not worth it when batch extraction is lossless.

Hybrid rejected: Complexity without benefit. The tool counter already provides real-time counts.

## Volume Estimate (Confirmed)

| Metric | Per Session | Per Month (1/day) | Per Month (3/day) |
|--------|------------|-------------------|-------------------|
| Tool calls | ~2,750 | ~82,500 | ~247,500 |
| JSONL size | ~960 KB | ~29 MB | ~87 MB |
| Error rate | ~8.7% | — | — |

Retention: 90 days default (aligns with T-110 transcript retention). Older data archived or deleted by `fw transcripts clean`.

## Relationship to Downstream Tasks

- **T-103 (error escalation):** Can query `tool-calls.jsonl` for `"is_error": true` records. No need for separate error parser. T-103's "auto-population" becomes: read errors from store, classify by tool name, suggest escalation ladder step.
- **T-105 (reporting page):** Can read JSONL and generate HTML/terminal reports. No schema migration needed.
- **T-110 (transcript retention):** Capture extraction should run BEFORE cleanup. `fw transcripts clean` should extract tool calls from sessions before deleting them.

## GO/NO-GO Assessment

### GO Arguments
1. **Unblocks T-103 and T-105** — both are waiting on this data layer
2. **Schema is simple** — 14 fields, ~350 bytes/record, well-understood
3. **Implementation is bounded** — one Python extractor (~150 lines) + one hook integration
4. **Volume is manageable** — ~30 MB/month with 90-day retention
5. **No new dependencies** — Python + JSONL, both already in use
6. **Source data structure is now confirmed** — no unknowns remain

### NO-GO Arguments
1. **No immediate consumer** — T-103 and T-105 are also `horizon: later`
2. **Source JSONL already exists** — anyone can parse it directly (the store is a convenience, not a necessity)
3. **Task context enrichment is lossy** — we capture `current_task` at extraction time, but task may have changed mid-session

### Recommendation: **GO** — Build the extractor

The data layer is simple, bounded, and unblocks two downstream tasks. The "source already exists" argument is weak because the whole point is aggregation across sessions — nobody will parse 50 separate JSONL files manually.

## Build Tasks (if GO approved)

1. **T-XXX: Build tool call extractor** (`agents/telemetry/extract-tool-calls.py`)
   - Parse main JSONL + sidechain files for a given session
   - Output metadata-only records to stdout (JSONL)
   - Flags: `--session ID`, `--task T-XXX`, `--since DATE`
   - Type: build, ~2 hours

2. **T-XXX: Integrate extractor into session lifecycle**
   - Hook into PreCompact or session-end
   - Append to `.context/telemetry/tool-calls.jsonl`
   - Add retention alignment with T-110
   - Type: build, ~1 hour

## Dialogue Log

**2026-03-12:** Two parallel investigation agents analyzed main JSONL and sidechain structure.

Key correction: Prior assumption (from T-107) that sidechain files have "richer structured error data" was wrong. Both have identical `tool_result` blocks with `is_error` boolean. Sidechain advantage is attribution isolation (per-agent), not richer data.

Actual error data in both sources: `is_error: true` on tool_result blocks (240 in sample session, 8.7% rate), plus rare API errors (3 in sample session). Error content is a string, not structured — classification requires text parsing.
