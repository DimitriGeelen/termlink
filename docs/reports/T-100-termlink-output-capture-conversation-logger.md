# T-100: TermLink Output Capture as Conversation Logger
## Research Artifact | Created: 2026-03-18

---

## Problem

Can TermLink's terminal output capture be used to log Claude Code conversation turns
without modifying Claude Code? The goal: wrap a Claude Code session in TermLink, capture
stdout, and extract structured conversation data from the raw terminal stream.

## What We Discovered

### 1. TermLink Output Capture Mechanics

TermLink has two output capture paths:

| Path | Mechanism | Latency | Fidelity |
|------|-----------|---------|----------|
| **attach** (`termlink attach`) | Polling `query.output` RPC every N ms | ~100ms | Scrollback snapshot (UTF-8, optional ANSI strip) |
| **stream** (`termlink stream`) | Binary frames over data plane socket | Real-time | Raw PTY output bytes (FrameType::Output = 0x0) |

**Key RPC methods:**
- `query.output` — returns scrollback buffer contents (handler.rs:294-350)
- `command.inject` — writes keystrokes to PTY master (handler.rs:455+)
- Data plane: binary frame protocol (data.rs) over `.sock.data` Unix socket

**Scrollback storage:** In-memory ring buffer (`ScrollbackBuffer` in scrollback.rs).
No persistent storage — data lives only during session lifetime. A logger would need
to consume the stream in real-time or periodically snapshot.

### 2. Wrapping Claude Code at Launch

**Current capability:** `termlink run <cmd>` does NOT create a PTY-wrapped session.
It spawns `sh -c <command>` with pipe-captured stdout/stderr (executor.rs:6-12).
This means Claude Code's TUI would break — it needs a real PTY for terminal rendering.

**What's needed:** Launch Claude Code inside a PTY-backed TermLink session. Two paths:

| Approach | How | Exists? |
|----------|-----|---------|
| **register --shell** | Register current shell as session, then run claude inside it | YES (T-156: tl-claude.sh) |
| **run --pty** | New flag: spawn command in PTY instead of pipes | NO (would need build) |

**T-156 already solved this.** `tl-claude.sh` registers the shell as a TermLink session,
then launches Claude Code inside it. Output is captured via the session's PTY scrollback.

### 3. Claude Code Terminal Output Structure

Claude Code renders to terminal using ANSI escape sequences:

```
<ESC>[1m<ESC>[36m> <ESC>[0m User message text...
<ESC>[2m  thinking... <ESC>[0m
<ESC>[1mAssistant: <ESC>[0m Response text with <ESC>[1mbold<ESC>[0m and formatting...
<ESC>[90m─────────────────<ESC>[0m  (separator lines)
<ESC>[33m⚠ Tool: Read file.rs<ESC>[0m
```

**Structural markers observed:**
- Human turns: preceded by `>` prompt character
- Assistant turns: follow human turns, often with bold/color prefix
- Tool use: typically dimmed or colored differently
- Separators: box-drawing characters between sections
- Spinners: overwrite same line repeatedly during tool execution

**Challenge:** There are NO guaranteed structural delimiters between conversation turns.
The rendering is designed for human eyes, not machine parsing. Extracting turns requires:
1. ANSI stripping (well-solved: `strip-ansi-escapes` crate or `query.output` with strip flag)
2. Heuristic turn boundary detection (fragile, version-dependent)
3. Filtering spinner/progress noise (high volume, low signal)

### 4. Comparison: TermLink Capture vs. JSONL Transcript

| Dimension | TermLink Capture | JSONL Transcript (T-101) |
|-----------|-----------------|--------------------------|
| **Structure** | Raw terminal bytes, ANSI-encoded | Structured JSON events with types |
| **Turn boundaries** | Heuristic (fragile) | Explicit (`user`/`assistant` events) |
| **Tool use** | Visual rendering only | Full tool_use + tool_result objects |
| **Completeness** | Sees everything rendered | Sees everything sent to API |
| **What's missed** | Nothing visible | Progress spinners, UI chrome |
| **Parse complexity** | HIGH — ANSI strip + heuristic parsing | LOW — JSON.parse per line |
| **Reliability** | Brittle (breaks on UI changes) | Stable (mirrors public API format) |
| **Requires** | TermLink session wrap at launch | File read access (~/.claude/) |
| **Real-time** | Yes (stream) | Yes (file grows live) |
| **Persistence** | No (in-memory ring buffer) | Yes (file on disk) |
| **Works today** | Partially (T-156 wrap exists) | Yes (T-101 reader exists) |

### 5. Feasibility Assessment

**Technical feasibility:** MEDIUM — TermLink CAN capture Claude Code output (T-156 proves this).
The hard problem is parsing: extracting structured conversation turns from raw terminal output
is inherently fragile and version-dependent.

**Value vs. JSONL:** LOW incremental value. The JSONL transcript (T-101) provides strictly
better data for conversation logging:
- Already structured with explicit turn boundaries
- Includes full tool_use/tool_result payloads
- Stable format (mirrors public Claude API)
- Already available without TermLink
- T-109 built a `/capture` skill using JSONL

**Where TermLink capture IS uniquely valuable:**
- **Live observation:** Watching an agent work in real-time (attach/stream) — already built
- **Input injection:** Sending commands to a running session — already built
- **Cross-machine monitoring:** Via TCP hub — already built (T-163 chain)
- **NOT conversation logging** — JSONL is the right tool for that

## Dialogue Log

### Q: Can TermLink capture be used for conversation logging?
**A:** Technically yes, but it's the wrong tool. TermLink captures raw terminal bytes —
useful for real-time observation and injection, but terrible for structured data extraction.
The JSONL transcript provides the same data in a machine-readable format that's already parsed.

## Go/No-Go Recommendation

**NO-GO** for TermLink output capture as conversation logger.

**Rationale:**
1. JSONL transcript (T-101) provides strictly superior data for logging
2. Parsing terminal output into structured turns is fragile and version-dependent
3. TermLink capture's unique value is real-time observation/injection, not logging
4. Building a terminal parser would be significant effort for inferior results
5. `/capture` skill (T-109) already exists using JSONL approach

**Recommended instead:**
- Continue using JSONL transcript for conversation capture (T-101/T-109)
- Continue using TermLink for real-time agent observation/injection (already built)
- If Anthropic adds PostMessage hook (T-099), that supersedes both approaches
