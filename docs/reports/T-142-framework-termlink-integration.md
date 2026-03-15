# T-142: TermLink Integration into Engineering Framework

> Inception research artifact — created 2026-03-15
> Status: exploring

## Problem Statement

The engineering framework agent (Claude Code + agentic-fw) operates in a single
terminal session. It cannot observe, control, or coordinate across multiple
terminal environments. TermLink provides cross-terminal communication primitives
that could unlock significant new capabilities for the framework.

Two questions to answer:
1. **What does TermLink make possible** that the framework cannot do today?
2. **How should the framework integrate TermLink** as a first-class tool?

## What TermLink Makes Possible

### Already Validated (T-136, T-137, T-138)

1. **Self-testing loop** — Agent spawns a terminal, runs `fw doctor` / `fw audit`,
   reads output, diagnoses failures, fixes them, retries. No human copy-paste needed.
   - Skill: `/self-test` (already built)
   - Primitives: `register --shell`, `interact`, `output --strip-ansi`

### New Capabilities to Explore

2. **Parallel agent dispatch via real terminals** — Instead of Claude Code's
   Agent tool (which shares context window), spawn N independent terminal
   sessions each running their own `claude` instance. True isolation.
   - Each worker gets its own context window (200K tokens)
   - Workers communicate results via TermLink events or files
   - Orchestrator observes progress via `termlink output` / `termlink events`

3. **Live service observation** — Agent starts a web server in one terminal,
   tests it from another, watches logs in a third. Full integration testing
   without leaving the agent loop.
   - `termlink register --name api-server --shell` → start server
   - `termlink register --name test-runner --shell` → run tests against it
   - `termlink output api-server` → check server logs

4. **Long-running process management** — Framework can spawn builds, test suites,
   deploys in background terminals and poll for completion.
   - `cargo test --workspace` in one terminal (takes minutes)
   - Agent continues other work
   - `termlink wait <session> --event process.exit` → notified on completion
   - `termlink output <session>` → read results

5. **Environment isolation** — Each terminal session has its own shell env.
   Agent can test with different env vars, PATH configs, or shell profiles
   without contaminating its own environment.
   - Test with `RUST_LOG=debug` in one session, clean env in another
   - Validate that scripts work from a fresh shell (not just the agent's env)

6. **Interactive program testing** — Agent can drive interactive programs
   (vim, REPLs, TUIs) via inject + observe.
   - `termlink inject <session> --keys "..."` → send keystrokes
   - `termlink output <session>` → observe screen state
   - Validates that TUI components render correctly

7. **Cross-machine coordination** (TCP transport — T-011 Phase 2, T-133) —
   With TCP transport, the framework agent on one machine could observe
   and coordinate with sessions on other machines. Human rates this HIGH
   priority despite being future work — the distributed story is core.

8. **Remote control** — Human or agent connects to a running agent session
   from anywhere and can observe, intervene, steer, or take over.
   - `termlink attach <session>` → live terminal view
   - `termlink inject <session> --keys "..."` → send input
   - `termlink output <session>` → read current state
   - With TCP: do all of the above across machines
   - Use cases: human oversight of autonomous agents, remote debugging,
     intervention when agent is stuck, pair-programming across machines
   - This is the "control tower" pattern — one seat, many agents

9. **Session-aware git operations** — Framework could spawn a dedicated
   terminal for git operations, observe hooks firing, capture hook output,
   and react to failures — all programmatically.

10. **CI/CD local simulation** — Agent spawns multiple terminals simulating
   a CI pipeline: build → test → lint → deploy, each in isolation, with
   dependency ordering via TermLink events.

11. **Agent mesh without Claude Code Agent tool** — Multiple `claude` CLI
    instances in separate terminals, coordinated by a TermLink-aware
    orchestrator script. Each agent has full context, full tools, full
    autonomy. The orchestrator assigns tasks and collects results.

## Integration Model

**Decided:** TermLink is an **external tool on PATH**. The framework pulls
updates from the TermLink repo but does not bundle or fork it.

### Ownership boundary
- **TermLink repo** owns: binary, protocol, session management, transport
- **Framework repo** owns: agent wrapper, skills, dispatch patterns, CLAUDE.md integration
- Updates flow: TermLink repo → `cargo install` / brew → framework detects on PATH

### Integration model: Hybrid (agent + transparent)

```
agents/termlink/
  AGENT.md        # Intelligence: when/how to use TermLink
  termlink.sh     # Mechanical: wrapper for common patterns
```

- **Explicit:** TermLink agent for orchestrated operations (spawn mesh, self-test, remote control)
- **Transparent:** Other agents use TermLink when available, degrade gracefully when not
- **Detection:** `fw doctor` checks TermLink availability and version
- **Task-awareness:** Agent wrapper auto-tags sessions with current task ID
- **Budget-awareness:** Don't spawn new sessions when context is critical

## Exploration Plan

1. [x] Enumerate what TermLink makes possible (capabilities list above)
2. [x] Discuss with human: which capabilities matter most?
3. [x] Evaluate integration model options → decided: hybrid (agent + transparent)
4. [x] Research: CLI primitives, fw agent patterns, fw doctor checks (3 agents)
5. [ ] Define Phase 0 (minimum viable integration)
6. [ ] Go/No-Go decision

## Agent Research Findings (3 parallel agents, 2026-03-15)

### Finding 1: TermLink CLI Primitives (30+ commands)

**Best for framework integration:**
- `interact --json` — inject command, wait for completion, return structured
  `{output, exit_code, elapsed_ms, marker_found}`. Star primitive.
- `discover --json` — find sessions by tag/role/name. Worker discovery.
- `event emit/wait/poll` — signaling between sessions. Coordination backbone.
- `event broadcast` — fan-out to multiple sessions.
- `list --json`, `status --json`, `info --json` — structured session queries.
- `run` — ephemeral session (register→execute→deregister). Fire-and-forget.
- `spawn` — start command in new terminal with `--wait` option.
- `kv set/get/list/del` — per-session key-value metadata store.

**Scripting readiness:** Most query commands support `--json`. Exit codes are
reliable and semantic (0=success, 1=timeout/not-found). `interact` is the
cleanest path for capturing command output + exit code in one call.

**Gaps:** No `--retry` built-in (must loop externally). No `--output-format
csv|yaml` (JSON only). No `--verbose` for structured diagnostics.

### Finding 2: Framework Agent Pattern

**Directory structure:**
```
agents/<name>/
  AGENT.md        # Purpose, when-to-use, workflow, validation, examples
  <name>.sh       # Main script — arg parsing, routing, operations
  lib/            # Optional shared functions (query.sh, ui.sh, etc.)
```

**Script template:**
```bash
#!/bin/bash
set -e
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
FRAMEWORK_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
PROJECT_ROOT="${PROJECT_ROOT:-$(git -C "$FRAMEWORK_ROOT" rev-parse --show-toplevel 2>/dev/null || echo "$FRAMEWORK_ROOT")}"
# Argument parsing with case/shift
# Validation → Output → Exit with semantic code (0/1/2)
```

**Routing:** `fw` routes to agents via `exec "$AGENTS_DIR/<name>/<script>.sh" "$@"`.
Agents inherit `PROJECT_ROOT` + `FRAMEWORK_ROOT` via environment.

**Inter-agent:** File-based coordination (`.tasks/`, `.context/`, `.fabric/`).
No agent-to-agent RPC. Sub-agent results via `fw bus post/read/manifest`.

### Finding 3: fw doctor Health Check Pattern

**Check format:**
```bash
if command -v termlink >/dev/null 2>&1; then
    version=$(termlink --version 2>/dev/null | head -1)
    echo -e "  ${GREEN}OK${NC}  TermLink ($version)"
else
    echo -e "  ${YELLOW}WARN${NC}  TermLink not installed (cargo install termlink)"
    warnings=$((warnings + 1))
fi
```

**Key rules:**
- Optional tools use `WARN` (yellow), not `FAIL` (red)
- Warnings don't block (exit stays 0); failures cause exit 2
- Include actionable hint in parentheses
- Precedent: bats-core and shellcheck checks follow same pattern

## Phase 0: Minimum Viable Integration

Based on research findings, Phase 0 is the smallest useful integration:

### What the framework needs to add

1. **`fw doctor` check** — detect TermLink on PATH, report version (WARN if
   missing, not FAIL — TermLink is optional)

2. **`agents/termlink/` directory:**
   ```
   agents/termlink/
     AGENT.md        # When/how to use TermLink from framework agents
     termlink.sh     # Wrapper: session lifecycle, task-tagging, health
   ```

3. **`termlink.sh` subcommands:**
   - `termlink.sh check` — is TermLink available? (used by fw doctor)
   - `termlink.sh spawn --task T-XXX` — spawn tagged session
   - `termlink.sh exec <session> <command>` — run + capture via `interact --json`
   - `termlink.sh cleanup` — clean stale sessions
   - `termlink.sh status` — list active sessions with task tags

4. **`fw termlink` route** in `fw` CLI — delegates to `agents/termlink/termlink.sh`

5. **CLAUDE.md additions:**
   - TermLink section: when to use, available primitives
   - Self-test reference: `/self-test` skill
   - Budget rule: don't spawn sessions when context > 60%

### What TermLink needs (this repo)

Nothing blocking. All primitives exist. Nice-to-haves for later:
- `termlink interact --retry N` — built-in retry
- `termlink spawn --register` — combine osascript spawn + register wait

### Phased rollout

| Phase | Scope | Owner |
|-------|-------|-------|
| **0** | fw doctor check + agents/termlink/ + fw termlink route | Framework |
| **1** | Self-test integration (move /self-test from skill to fw subcommand) | Framework |
| **2** | Parallel dispatch via TermLink (replace Agent tool mesh) | Framework |
| **3** | Remote control + attach patterns | Framework + TermLink |
| **4** | TCP transport + cross-machine | TermLink → Framework |

## Revised Priority Map

Based on human feedback, regrouped by strategic value:

### Tier 1 — Core (build first)
1. **Self-testing loop** — already validated (T-136/T-138)
2. **Parallel agent dispatch** — real terminal isolation, full context per worker
3. **Live service observation** — multi-terminal integration testing
4. **Long-running process management** — background builds/tests with notification
5. **Remote control** — human oversight, intervention, "control tower" pattern

### Tier 2 — Enabling
6. **Environment isolation** — fresh shell testing
7. **Session-aware git operations** — hook observation
8. **Agent mesh without Agent tool** — multi-claude orchestration

### Tier 3 — Transformative (needs TCP transport)
9. **Cross-machine coordination** — distributed framework ops (HIGH value per human)
10. **Remote control across machines** — the full "control tower" vision
11. **CI/CD local simulation** — multi-stage pipeline simulation

### Tier 4 — Specialist
12. **Interactive program testing** — TUI/REPL validation

## Dialogue Log

### Q1: Prioritization and missing capabilities (2026-03-15)

**Agent asked:** Which capabilities excite you most? External tool or bundled?
Missing capabilities?

**Human answered:**
1. Prioritization is correct. TCP transport is also really important — not
   just future/low priority.
2. External tool pulling updates from this repo. Framework treats TermLink
   as an independent tool on PATH.
3. Missing capability: **remote control** — human or agent connects to a
   running session from anywhere, observes, intervenes, steers, takes over.

**Outcome:** Added remote control as capability #8. Promoted TCP transport
from "future" to high priority. Integration model decided: external tool,
framework owns the agent wrapper.
