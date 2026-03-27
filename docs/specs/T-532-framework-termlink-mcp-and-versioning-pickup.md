# Framework Agent Pickup: Add TermLink to MCP Init + Versioning Alignment

> Task: T-532/T-533 | Generated: 2026-03-27 | Project: 010-termlink

## What You Need To Do

Two related enhancements to the framework:

### 1. Add TermLink to MCP auto-config (extends T-646)

T-646 seeds `.mcp.json` with context7 + playwright during `fw init`. TermLink should be included **when vendored** (`.termlink/bin/termlink` exists). This makes 25 TermLink MCP tools automatically available to Claude Code.

### 2. Versioning alignment documentation

TermLink T-532 ported the framework's T-648 git-derived versioning pattern to Rust via `build.rs`. The pattern is now proven across both shell (framework) and Rust (TermLink). Consider documenting this as a recommended pattern for all consumer projects.

## OneDev Location

- Instance: `onedev.docker.ring20.geelenandcompany.com`
- Repo: `agentic-engineering-framework`
- Branch: create `feature/termlink-mcp-init`

## Files to Include in the PR

### Modified files:

**`lib/init.sh`** — In `generate_claude_code_config()`, after the existing `.mcp.json` seeding for context7/playwright, add a conditional TermLink entry:

```bash
# TermLink MCP — only if vendored in this project
if [ -x "$PROJECT_ROOT/.termlink/bin/termlink" ]; then
    # Add termlink to .mcp.json
    python3 -c "
import json, os
path = '$PROJECT_ROOT/.mcp.json'
d = json.load(open(path)) if os.path.exists(path) else {}
if 'termlink' not in d:
    d['termlink'] = {
        'command': '.termlink/bin/termlink',
        'args': ['mcp', 'serve']
    }
    with open(path, 'w') as f:
        json.dump(d, f, indent=2)
        f.write('\n')
" 2>/dev/null && echo "  TermLink MCP: added to .mcp.json"
fi
```

**Also add to `.claude/settings.local.json`** (Claude Code reads MCP servers from there too):

```bash
if [ -x "$PROJECT_ROOT/.termlink/bin/termlink" ]; then
    python3 -c "
import json, os
path = os.path.join('$PROJECT_ROOT', '.claude', 'settings.local.json')
os.makedirs(os.path.dirname(path), exist_ok=True)
d = json.load(open(path)) if os.path.exists(path) else {}
servers = d.setdefault('mcpServers', {})
if 'termlink' not in servers:
    servers['termlink'] = {
        'command': '.termlink/bin/termlink',
        'args': ['mcp', 'serve']
    }
    with open(path, 'w') as f:
        json.dump(d, f, indent=2)
        f.write('\n')
" 2>/dev/null && echo "  TermLink MCP: configured in .claude/settings.local.json"
fi
```

**`lib/upgrade.sh`** — In the MCP reconciliation section, add the same conditional check so upgrades also pick up TermLink if it was vendored after initial init.

**`agents/audit/audit.sh`** — Add a check: if `.termlink/bin/termlink` exists but MCP is not configured, emit WARN.

### What the MCP entry provides

25 tools for Claude Code:
- Session lifecycle: `termlink_spawn`, `termlink_exec`, `termlink_interact`, `termlink_run`
- Discovery: `termlink_discover`, `termlink_list_sessions`, `termlink_status`
- Events: `termlink_emit`, `termlink_broadcast`, `termlink_wait`, `termlink_request`
- Terminal I/O: `termlink_output`, `termlink_inject`, `termlink_resize`
- Metadata: `termlink_kv_set/get/list/del`, `termlink_tag`
- Health: `termlink_ping`, `termlink_doctor`

## Versioning Pattern (for documentation)

TermLink ported the framework's T-648 git-derived versioning to Rust. The pattern works identically:

**Framework (shell):**
```bash
desc=$(git describe --tags --match 'v[0-9]*')
# v1.4.0-5-gabc → 1.4.5
```

**TermLink (Rust build.rs):**
```rust
// git describe --tags --match v[0-9]*
// v0.9.0-5-gabc → 0.9.5
// Overrides CARGO_PKG_VERSION at compile time
```

Consider adding this to `docs/guides/` as a recommended pattern for consumer projects. Both implementations:
- Use `git describe --tags --match 'v[0-9]*'`
- Derive `major.minor` from tag, `patch` from commit count
- Fall back to hardcoded version when not in git repo

## PR Description Template

**Title:** `feat: add TermLink to MCP auto-config during init/upgrade`

**Body:**
```
## Problem

T-646 seeds .mcp.json with context7 + playwright but not TermLink.
Projects that vendor TermLink miss 25 session orchestration tools unless
manually configured.

## Solution

- fw init: conditionally adds TermLink MCP when .termlink/bin/termlink exists
- fw upgrade: reconciles TermLink MCP entry (same condition)
- fw audit: warns if TermLink vendored but MCP not configured
- Only activates when TermLink is actually vendored (no-op otherwise)

## Validation

- TermLink T-531: vendor-side auto-config tested (fresh, idempotent, merge)
- TermLink T-532: git-derived versioning ported from framework T-648
- Pattern proven: framework shell + TermLink Rust build.rs

## Limitations

- Requires python3 for JSON manipulation (existing dependency)
- Does not install TermLink — only configures if already vendored
```

## Validation Evidence

- T-531 tested MCP auto-config: fresh project, idempotent re-run, merge with existing settings
- T-532 verified git-derived versioning: tag `v0.9.0` → `0.9.0`, 1 commit later → `0.9.1`
- Framework T-648 pattern reused verbatim (major.minor from tags, patch from commit count)

## After Creating the PR

1. Post the PR URL in TermLink's T-533 task
2. Test on a project that has `.termlink/` vendored — run `fw init` and verify MCP appears
