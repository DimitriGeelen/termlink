# Framework Agent Pickup: Add TermLink MCP Server to Project Initialization

> Task: T-531 | Generated: 2026-03-27 | Project: 010-termlink

## What You Need To Do

When the framework initializes a new project (via `fw context init` or project setup), it should ensure the TermLink MCP server is configured in `.claude/settings.local.json` — but **only if TermLink is vendored** (`.termlink/bin/termlink` exists). This makes the 25 TermLink MCP tools (session orchestration, events, dispatch) automatically available to Claude Code in framework-consuming projects.

## Background

TermLink's `termlink vendor` command now auto-configures the MCP server entry in `.claude/settings.local.json` (T-531). However, there are two gaps:

1. **Existing vendored projects** that ran `termlink vendor` before T-531 don't have the MCP entry
2. **Framework init** is a natural checkpoint to ensure all tooling integrations are wired

The framework should check for and configure this during initialization, the same way it already checks for `.agentic-framework/` symlinks and `.framework.yaml`.

## OneDev Location

- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: create `feature/termlink-mcp-init`

## What to Implement

### Modified files:

**`agents/context/context.sh`** — In the `init` subcommand handler, after existing initialization steps, add a TermLink MCP configuration check:

```bash
# --- TermLink MCP integration ---
# If TermLink is vendored, ensure MCP server is configured in Claude Code settings
TERMLINK_BIN=".termlink/bin/termlink"
CLAUDE_SETTINGS=".claude/settings.local.json"

if [ -x "$PROJECT_ROOT/$TERMLINK_BIN" ]; then
    # Check if mcpServers.termlink already exists
    if [ -f "$PROJECT_ROOT/$CLAUDE_SETTINGS" ]; then
        has_termlink=$(python3 -c "
import json, sys
try:
    d = json.load(open('$PROJECT_ROOT/$CLAUDE_SETTINGS'))
    print('yes' if d.get('mcpServers', {}).get('termlink') else 'no')
except: print('no')
" 2>/dev/null || echo "no")
    else
        has_termlink="no"
    fi

    if [ "$has_termlink" = "no" ]; then
        # Use termlink's own vendor command to configure (idempotent)
        "$PROJECT_ROOT/$TERMLINK_BIN" vendor --target "$PROJECT_ROOT" 2>/dev/null \
            && echo "  TermLink MCP: configured" \
            || echo "  TermLink MCP: WARN — vendor failed, configure manually"
    else
        echo "  TermLink MCP: already configured"
    fi
fi
```

**Alternative (lighter touch):** If you prefer not to re-run `termlink vendor` (which also re-copies the binary), do the JSON merge directly:

```bash
if [ "$has_termlink" = "no" ]; then
    mkdir -p "$PROJECT_ROOT/.claude"
    python3 -c "
import json, os
path = '$PROJECT_ROOT/$CLAUDE_SETTINGS'
try:
    d = json.load(open(path)) if os.path.exists(path) else {}
except: d = {}
d.setdefault('mcpServers', {})['termlink'] = {
    'command': '.termlink/bin/termlink',
    'args': ['mcp', 'serve']
}
with open(path, 'w') as f:
    json.dump(d, f, indent=2)
    f.write('\n')
print('  TermLink MCP: configured')
" 2>/dev/null || echo "  TermLink MCP: WARN — failed to configure"
fi
```

**`agents/audit/audit.sh`** — Add a check under the existing tool-integration audit section:

```bash
# Check: TermLink MCP configured if vendored
if [ -x "$PROJECT_ROOT/.termlink/bin/termlink" ]; then
    if [ -f "$PROJECT_ROOT/.claude/settings.local.json" ]; then
        has_mcp=$(python3 -c "
import json
d = json.load(open('$PROJECT_ROOT/.claude/settings.local.json'))
t = d.get('mcpServers', {}).get('termlink', {})
print('yes' if t.get('command') == '.termlink/bin/termlink' else 'no')
" 2>/dev/null || echo "no")
    else
        has_mcp="no"
    fi

    if [ "$has_mcp" = "yes" ]; then
        record_result "PASS" "termlink-mcp" "TermLink MCP server configured"
    else
        record_result "WARN" "termlink-mcp" "TermLink vendored but MCP not configured in .claude/settings.local.json — run: termlink vendor"
    fi
fi
```

### What the MCP entry looks like

```json
{
  "mcpServers": {
    "termlink": {
      "command": ".termlink/bin/termlink",
      "args": ["mcp", "serve"]
    }
  }
}
```

This tells Claude Code to launch `.termlink/bin/termlink mcp serve` as a stdio MCP server, exposing 25 tools for terminal session orchestration, event messaging, KV store, and agent coordination.

## PR Description Template

**Title:** `feat: auto-configure TermLink MCP server during project init`

**Body:**
```
## Problem

When TermLink is vendored into a framework project, the MCP server (`termlink mcp serve`)
provides 25 structured tools for Claude Code (session management, events, dispatch).
But the MCP entry in `.claude/settings.local.json` isn't created during `fw context init`,
so Claude Code doesn't know the server exists until the user manually configures it.

## Solution

- `fw context init` now checks if `.termlink/bin/termlink` exists and ensures
  the MCP server entry is in `.claude/settings.local.json`
- `fw audit` warns if TermLink is vendored but MCP is not configured
- Idempotent — skips if already configured
- Only activates when TermLink is actually vendored (no-op otherwise)

## Validation

- TermLink T-531 implemented the vendor-side auto-config (tested on 2026-03-27)
- This pickup adds the framework-side init check as a complementary path

## Limitations

- Requires python3 for JSON manipulation (already a framework dependency)
- Does not install TermLink — only configures MCP if already vendored
```

## Validation Evidence

- T-531 tested vendor MCP auto-config with: fresh project, idempotent re-run, merge with existing settings
- MCP server exposes 25 tools verified via `termlink mcp serve` stdio handshake

## After Creating the PR

1. Post the PR URL in TermLink's T-531 task as a comment
2. Test on a project that has `.termlink/` vendored but no MCP entry — run `fw context init` and verify the entry appears
