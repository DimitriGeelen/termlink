# T-908: API Relay Governance — Research Artifact

**Task:** T-908  
**Type:** Inception  
**Created:** 2026-04-09  
**RFC:** anthropics/claude-code#45427  

## Context

Claude Code's PreToolUse hooks have 5 documented failure modes. The RFC proposes a `toolGate` inside the CLI — correct but requires Anthropic to ship it. This inception explores an alternative: a local API relay that intercepts the Anthropic SSE stream and enforces governance at the wire level.

**Core idea:** Set `ANTHROPIC_BASE_URL=http://localhost:PORT`, run a TermLink relay that forwards to api.anthropic.com, parse the SSE response stream, and rewrite/block tool_use events that violate governance rules.

## Spike 1: Protocol Analysis

### Objective
Document the exact SSE wire format for Anthropic API streaming responses, specifically how tool_use blocks are encoded.

### Findings

**Source:** Anthropic Messages Streaming API docs (platform.claude.com/docs/en/api/messages-streaming)

**SSE Event Flow (complete sequence):**
```
message_start          → Message object with empty content[]
  content_block_start  → Opens a content block (text, tool_use, thinking, server_tool_use)
  content_block_delta* → One or more deltas for the block
  content_block_stop   → Closes the block
  ... (more content blocks)
message_delta          → Top-level changes (stop_reason, usage)
message_stop           → Stream complete
```

**Tool_use block wire format:**

1. `content_block_start` contains the **tool name and ID** upfront:
```
event: content_block_start
data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"toolu_01T1x1fJ34qAmk2tNTrN7Up6","name":"get_weather","input":{}}}
```

2. Input JSON arrives as `input_json_delta` fragments:
```
event: content_block_delta
data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"location\":"}}

event: content_block_delta
data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":" \"San"}}
...
```

3. Block closes with `content_block_stop`, then `message_delta` has `"stop_reason":"tool_use"`.

**Critical insight for relay design:**
- The **tool name is known at `content_block_start`** — we don't need to buffer the entire block to decide whether to gate it
- Input JSON is fragmented across deltas as `partial_json` strings — must accumulate and parse on `content_block_stop`
- For path-based gating (e.g., block Write to settings.json), we DO need to buffer the input JSON to extract the path
- Current models emit one complete key+value at a time, so there may be delays between deltas
- `server_tool_use` is a separate type (server-side tools like web_search) — may want different gating rules

**Relay strategy validated:** The `content_block_start` event gives us tool name + ID immediately. We can:
- **Fast-gate by tool name** without buffering (block all Write/Edit)
- **Buffer + gate by content** when we need to inspect the input (path patterns)
- **Strip the entire block** (start through stop) when blocked
- **Inject a text block** in its place with the governance message

**Assumption A-2 (SSE can be parsed incrementally): VALIDATED** — tool_use blocks have clean boundaries and the tool name arrives in the first event.

## Spike 2: ANTHROPIC_BASE_URL Behavior

### Objective
Verify that Claude Code and subagents respect the environment variable.

### Findings

**Method:** Binary string extraction from Claude Code v2.1.97 ELF binary + Anthropic SDK documentation review.

**Evidence:**
1. `strings` on `/root/.local/share/claude/versions/2.1.97` reveals `ANTHROPIC_BASE_URL` is a recognized env var
2. The Anthropic TypeScript SDK constructor accepts `baseURL` (defaults to `https://api.anthropic.com`)
3. The SDK reads `ANTHROPIC_BASE_URL` from environment automatically
4. Claude Code v2.1.97 also recognizes: `ANTHROPIC_FOUNDRY_BASE_URL`, `ANTHROPIC_BEDROCK_BASE_URL`, `ANTHROPIC_AWS_BASE_URL`, `ANTHROPIC_VERTEX_BASE_URL` — confirming base URL override is a first-class concept
5. Subagent spawning inherits the process environment (standard Node.js `child_process` / Deno behavior)

**Assumption A-1 (Claude Code respects ANTHROPIC_BASE_URL): VALIDATED** — present in binary, SDK supports it natively.
**Assumption A-4 (Subagents inherit env vars): HIGH CONFIDENCE** — standard process inheritance; would need live test to confirm 100%.
**Assumption A-6 (API key passthrough): VALIDATED** — `x-api-key` header is set by SDK from `ANTHROPIC_API_KEY`, relay just needs to forward headers.

**Additional env vars of interest for relay:**
- `ANTHROPIC_UNIX_SOCKET` — could be used for local IPC instead of TCP (lower latency)
- `ANTHROPIC_LOG` — enables SDK debug logging (useful for development)
- `ANTHROPIC_CUSTOM_HEADERS` — could inject relay-specific headers

## Spike 3: Minimal Relay Prototype

### Objective
Build a pass-through proxy, validate stream integrity.

### Findings

<!-- To be filled during spike execution -->

## Spike 4: Tool Gate Enforcement

### Objective
Rewrite tool_use blocks in the stream, test Claude Code's response.

### Findings

<!-- To be filled during spike execution -->

## Dialogue Log

### 2026-04-09 — Initial Scoping

**Human:** Referenced RFC #45427, proposed IP-layer wrapper to intercept streamed responses and lock down Claude to only send commands through the wrapper.

**Agent:** Analyzed two approaches:
- Option A: HTTPS MITM proxy (rejected — TLS complexity)
- Option B: Local API relay via ANTHROPIC_BASE_URL (recommended — clean, leverages TermLink hub architecture)

**Outcome:** Option B selected. Closes all 5 failure modes:
1. Subagent bypass — subagents inherit ANTHROPIC_BASE_URL
2. Silent hook failure — wire-level, not a hook
3. Model self-modification — can't edit env vars set before process start
4. Alternative tool paths — sees ALL tool calls regardless of tool name
5. CLAUDE.md non-compliance — structural enforcement, not advisory

**Decision:** Open inception (this task) to validate feasibility via 4 spikes.
