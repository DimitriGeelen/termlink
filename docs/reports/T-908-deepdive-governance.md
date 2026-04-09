# T-908 Deep Dive: Governance Capabilities -- Relay vs. Hooks

**Task:** T-908 (Worker 1: Governance Deep Dive)  
**Date:** 2026-04-09  
**Scope:** Map each RFC failure mode to relay enforcement capabilities; analyze edge cases including relay failure.

## 1. The Five Failure Modes

The RFC (anthropics/claude-code#45427) and the T-908 task identify five failure modes (FM1-FM5) in Claude Code's PreToolUse hook system. The current governance hooks are defined in `.claude/settings.json` and route through `.agentic-framework/bin/fw hook <name>`. The existing hooks include: `check-active-task` (Write/Edit), `check-tier0` (Bash), `budget-gate` (Write/Edit/Bash), `check-agent-dispatch` (Agent), `check-project-boundary` (Write/Edit/Bash), and `block-plan-mode` (EnterPlanMode).

| ID | Failure Mode | Description |
|----|-------------|-------------|
| FM1 | Subagent bypass | Subagents (Task tool) may not trigger PreToolUse hooks in the parent process |
| FM2 | Silent hook failure | Hook scripts that error or time out are silently ignored; enforcement disappears |
| FM3 | Model self-modification | The model can edit `.claude/settings.json` or hook scripts to weaken/disable governance |
| FM4 | Alternative tool paths | The model can use Bash to achieve file writes that bypass Write/Edit hooks |
| FM5 | CLAUDE.md non-compliance | Advisory rules in CLAUDE.md have no structural enforcement; the model can ignore them |

## 2. Failure Mode to Relay Capability Mapping

### FM1: Subagent Bypass

**Hook limitation:** PreToolUse hooks fire inside the Claude Code process. When the model spawns a subagent via the Task tool, that subagent runs as a child process. There is no documented guarantee that PreToolUse hooks from the parent apply to tool calls within the subagent. The hook in `settings.json` matching `Agent` (check-agent-dispatch) fires on the *dispatch* of the subagent, not on what the subagent *does*.

**Relay enforcement:** The relay sits at the API layer. Every subagent process inherits `ANTHROPIC_BASE_URL=http://localhost:PORT` from the parent (standard process environment inheritance, validated in Spike 2). Every API call from every subagent passes through the relay. The relay sees `tool_use` content blocks in the streaming response regardless of which process made the API call.

**Specific capability:** The relay can enforce task-gate rules uniformly across all agents and subagents. A subagent requesting a Write tool call gets the same governance check as the parent. There is no "subagent context" the relay is unaware of -- every tool_use block arrives as an SSE event with a tool name and input JSON.

**Edge case:** If a subagent is spawned with a modified environment that overrides `ANTHROPIC_BASE_URL`, it could bypass the relay. Mitigation: the relay can track active session IDs and alert if expected subagent traffic does not arrive. Additionally, the parent process environment is set before Claude Code launches -- the model cannot modify environment variables of its own running process.

### FM2: Silent Hook Failure

**Hook limitation:** If a hook script (`check-active-task.sh`, `budget-gate.sh`) exits with an unexpected error, crashes, or times out, Claude Code's behavior is to proceed as if the hook passed. This means a broken hook silently degrades to no enforcement. Hook scripts are shell scripts that depend on file I/O, YAML parsing, and process spawning -- all of which can fail.

**Relay enforcement:** The relay is a long-running Rust process (or extended ccproxy Python process) that processes every API response. If the relay itself is running, governance checks execute in-process -- there is no shell script to crash, no subprocess to time out. The governance decision is part of the SSE stream processing pipeline:

1. Parse `content_block_start` event
2. Extract tool name
3. Check governance rules (in-memory lookup, microseconds)
4. If blocked: suppress the tool_use block and inject a text block

There is no external process invocation in the hot path. The check is a function call within the stream processor, not a fork/exec of a shell script.

**Specific capability:** Governance checks are synchronous with stream processing. A governance check cannot silently fail without the entire relay failing -- and relay failure is visible and loud (see Section 4).

**Edge case:** If the governance rule configuration file is corrupted or missing, the relay must decide: fail-open (allow all) or fail-closed (block all). The correct choice for a governance system is fail-closed with a clear error. This is a design decision for the build phase.

### FM3: Model Self-Modification

**Hook limitation:** The model has Write/Edit access to the filesystem. It can modify `.claude/settings.json` (removing hooks), edit hook scripts (making them no-ops), or alter `.claude/settings.local.json` to override managed settings. The Ona research demonstrates that even file-permission-based protections (`chattr +i`) can be circumvented by a sufficiently capable model using Linux internals (`/proc/self/root/`, `ld-linux` dynamic linker bypass).

**Relay enforcement:** The relay runs as a separate process launched *before* Claude Code starts. Its configuration, binary, and governance rules live outside the model's write scope -- or at minimum, edits to the relay's config have no effect on the already-running relay process.

**Specific capabilities the relay provides that hooks cannot:**

1. **Process isolation:** The relay is a separate PID. The model interacts with it only through HTTP requests to `localhost:PORT`. It cannot modify the relay's memory, configuration, or code through Claude Code's tool set.

2. **Pre-launch configuration:** Governance rules are loaded when the relay starts. Even if the model could modify the config file on disk, the running relay process has already read and internalized the rules. Changes to disk only take effect on relay restart -- which the model cannot trigger.

3. **No filesystem dependency in hot path:** Hook scripts must be read from disk on every invocation. The relay's rules are in-memory. There is no file the model can edit to weaken an already-running relay.

4. **Binary integrity:** The relay binary itself can be protected with standard OS mechanisms (read-only mount, separate user, etc.), but the key insight is that even without such protection, modifying the binary does not affect the running process.

**Edge case:** The model could theoretically kill the relay process via Bash (`kill <pid>`). Mitigation: this is a Tier 0 action (destructive command) already gated by `check-tier0`. Additionally, the relay can be run under a separate user account. Most importantly, if the relay dies, all API calls fail (see Section 4), which is a visible, loud failure. The model could also use Bash to `curl https://api.anthropic.com/v1/messages` directly with the API key from the environment. Mitigation: API key isolation -- the relay holds the real key; Claude Code gets a relay-scoped token.

### FM4: Alternative Tool Paths

**Hook limitation:** The `check-active-task` hook fires on Write and Edit tools. But the model can achieve file writes through Bash: `echo "content" > file.py`, `curl ... > file.py`, `python3 -c "open('file.py','w').write('...')"`. The Bash hook only checks for Tier 0 destructive commands, not for file-write equivalents. Adding comprehensive file-write detection to a Bash hook is a game of whack-a-mole -- there are infinite ways to write files through shell commands.

**Relay enforcement:** The relay does not care *which* tool the model uses. It sees the tool_use content block in the API response stream. Whether the model asks for `Write`, `Edit`, `Bash`, or any future tool, the relay sees the tool name and its input arguments in the SSE stream.

**Specific capabilities:**

1. **Tool-name-agnostic gating:** The relay can enforce "no tool calls without active task" across ALL tools, not just Write/Edit. A Bash tool_use block is just as visible as a Write tool_use block.

2. **Argument inspection:** For Bash tool calls, the relay can inspect the `command` argument and apply regex-based rules (detect file write patterns in shell commands). This is still heuristic, but it happens at the wire level where the model cannot craft an alternative path around it.

3. **Allowlist approach:** Instead of trying to block all dangerous patterns (denylist), the relay can implement an allowlist: only permit tool_use blocks matching known-safe patterns (e.g., `Read`, `Grep`, `Glob` always allowed; `Write`, `Edit`, `Bash` require task context). This inverts the security model from "block bad things" to "only allow known good things."

**Edge case:** If a future Claude Code version introduces a new tool type that the relay does not know about, the relay's allowlist would block it (safe default). A denylist approach would miss it (unsafe default). The allowlist is the correct design. Also note: the relay improves the enforcement architecture (checks always fire on all tools) but detecting dangerous intent in arbitrary shell commands remains fundamentally undecidable. A model encoding a write as `python3 -c "exec(bytes.fromhex('...'))"` could evade content-based gating.

### FM5: CLAUDE.md Non-Compliance

**Hook limitation:** CLAUDE.md rules (commit cadence, session protocol, task sizing, inception discipline) are advisory. They are injected into the model's context and the model is expected to follow them. There is no structural enforcement -- a model that ignores "commit after every meaningful unit of work" simply does not commit, and no hook fires to prevent it.

**Relay enforcement:** The relay can enforce a subset of CLAUDE.md rules structurally:

1. **Task gate (prime directive):** "Nothing gets done without a task" becomes a wire-level rule: block tool_use for Write/Edit/Bash if no active task is registered.

2. **Commit cadence:** The relay can track tool_use frequency and inject a text block reminder ("GOVERNANCE: 15 minutes since last commit") -- though this is advisory, not blocking.

3. **Inception discipline:** The relay can check if the active task has `workflow_type: inception` and block Write/Edit to source code paths (only allowing writes to `docs/reports/`).

4. **Context budget:** The relay can track token usage from the API response `usage` field and block tool_use calls when the budget is exhausted.

**What the relay CANNOT enforce from CLAUDE.md:** Rules about the *content* of what is written (e.g., "acceptance criteria must be specific"), the *sequence* of actions (e.g., "read handover before starting work"), or *quality* judgments (e.g., "use hypothesis-driven debugging"). These remain advisory -- they require understanding intent, not just tool names and arguments.

**Net assessment:** The relay converts approximately 40-50% of CLAUDE.md rules from advisory to structural. The remaining rules require semantic understanding that no wire-level proxy can provide.

## 3. Capabilities the Relay Has That Hooks Structurally Cannot

Beyond closing the five failure modes, the relay enables capabilities that are fundamentally impossible with hooks:

| Capability | Why hooks cannot do this | How the relay does it |
|-----------|-------------------------|----------------------|
| **Wire audit trail** | Hooks see individual tool calls in isolation; no cross-call context | Relay sees the complete request/response stream; logs full conversations independently |
| **Response modification** | Hooks can only approve/reject; they cannot modify what the model said | Relay can rewrite SSE events: strip tool_use blocks, inject text blocks, modify arguments |
| **Cross-session governance** | Hooks are stateless (each invocation is a fresh shell) | Relay is a long-running process with persistent state; tracks patterns across API calls |
| **Token-level cost control** | Hooks have no access to API-level token counts | Relay sees `usage` in `message_delta` events; can enforce hard token budgets |
| **Model-agnostic enforcement** | Hooks are Claude Code specific | Relay works with any client that uses `ANTHROPIC_BASE_URL` |
| **Rate limiting** | Hooks cannot throttle; they are binary allow/deny | Relay can introduce delays, queue requests, or cap concurrent streams |

## 4. Relay Failure Analysis

The relay itself is a single point of failure. If it fails, all governance is lost -- or all API access is lost. The failure mode depends on how it fails.

### Failure Scenario 1: Relay process crashes

**Effect:** All API calls from Claude Code fail with connection refused. No tool calls can proceed.

**Assessment:** Fail-closed. No governance bypass is possible because no work is possible. The user sees immediate, obvious errors.

**Recovery:** Process supervisor auto-restarts. The relay should be stateless so restarts are instant. Claude Code's SDK has retry logic for transient failures. Target recovery: under 2 seconds.

### Failure Scenario 2: Relay hangs (deadlock, resource exhaustion)

**Effect:** API calls time out. Claude Code may hang waiting for responses.

**Assessment:** Fail-closed, but slower to detect than a crash. Could waste significant session time.

**Mitigation:** Internal watchdog timer. If no events are processed within N seconds while requests are pending, the relay self-terminates (triggering restart by supervisor).

### Failure Scenario 3: Governance rule evaluation error

**Effect:** A governance check fails due to bad state (corrupt task file, stale focus file).

**Assessment:** The relay MUST fail-closed on governance errors: block the tool_use and inject a text block explaining why. The model or user can then fix the state and retry.

### Failure Scenario 4: SSE parsing error

**Effect:** A malformed or unexpected SSE event arrives that the parser cannot handle.

**Assessment:** Forward unparseable events unmodified (fail-open for parsing, fail-closed for governance). This preserves API compatibility while logging the anomaly. Governance checks only fire on successfully parsed tool_use blocks.

### Failure Scenario 5: Relay bypassed (environment variable overridden)

**Effect:** Traffic bypasses the relay entirely.

**Assessment:** This is the relay's fundamental trust assumption. Mitigation: the relay launcher verifies the env var before starting Claude Code. API key isolation (relay holds real key) makes bypass useless even if traffic is redirected.

### Failure Scenario 6: Relay latency degrades session quality

**Effect:** Added latency makes Claude Code feel sluggish.

**Assessment:** For name-based gating: <5ms per tool_use block (imperceptible against 200-2000ms API round-trip). For content-based gating (buffering input JSON): 100-500ms on complex inputs but this only applies to tool_use blocks requiring deep inspection. For text blocks (majority of stream): pure passthrough, sub-millisecond. Solution: cache governance state in memory with inotify/kqueue refresh.

## 5. Comparison with Existing TermLink Governance

TermLink already has a data-plane governance subscriber (`crates/termlink-session/src/governance_subscriber.rs`) that watches Output frames for regex pattern matches and emits Governance frames. This is **post-hoc detection** -- it observes what happened and records it, but does not prevent it.

The API relay would provide **pre-hoc enforcement** -- it intercepts what the model *wants* to do and blocks it before execution. These are complementary:

| Aspect | Data-plane subscriber (T-905) | API relay (T-908) |
|--------|------------------------------|-------------------|
| **Timing** | After execution (output frames) | Before execution (tool_use blocks) |
| **Action** | Log, alert, emit governance event | Block, rewrite, inject message |
| **Scope** | Terminal output text | API request/response stream |
| **Bypass risk** | None (passive observer) | Env var override (mitigated by key isolation) |
| **Latency impact** | Zero (async, non-blocking) | Minimal (<5ms typical per event) |

The ideal architecture uses both: the relay prevents violations, and the data-plane subscriber detects anything that slips through as a defense-in-depth layer.

## Summary

The API relay closes all five RFC failure modes through a single architectural property: it operates at the wire level, outside the model's write scope, and in the only communication path between Claude Code and the Anthropic API. FM1 (subagent bypass) is fully closed because all processes inherit the base URL. FM2 (silent hook failure) is fully closed because governance is in-process, not shell-script-based. FM3 (self-modification) is largely closed (fully closed with API key isolation). FM4 (alternative tool paths) is meaningfully mitigated -- the relay ensures all tool_use blocks are checked, but detecting dangerous intent in arbitrary shell commands remains undecidable. FM5 (CLAUDE.md non-compliance) is partially closed -- structurally expressible rules become enforceable, but semantic rules remain advisory. The relay's failure modes are all fail-closed (no work happens) rather than fail-open (work without governance), which is the correct property for a security boundary. Residual risks that the relay cannot address -- MCP tool execution side effects and post-execution Bash side effects -- require complementary kernel-level enforcement.

## Open Questions

1. **API key isolation:** Should the relay hold the real API key and issue relay-scoped tokens? This fully closes FM3 but adds key management complexity and makes relay crash recovery more critical.

2. **Fail-closed granularity:** Should the relay block the entire API response when one tool_use block is blocked, or strip only the offending block and let other blocks through?

3. **A-3 validation:** Can Claude Code handle a rewritten SSE stream where a tool_use block is replaced with a text block? This is untested and is the most critical remaining assumption.

4. **State synchronization:** How does the relay learn about the current active task? Direct file reads, TermLink hub API query, or state pushes? Each has different consistency/latency tradeoffs.

5. **Subagent identity:** Can the relay correlate API requests to specific subagent sessions for per-agent governance rules?

6. **Multi-tool responses:** When the model returns multiple tool_use blocks and only one is blocked, how does Claude Code handle the partial response?

7. **MCP governance surface:** MCP tools execute out-of-band from the Anthropic API. Does the relay need a companion mechanism for MCP traffic, or does T-902's MCP task-gate cover this?
