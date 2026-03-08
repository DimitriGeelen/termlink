# Component Fabric: Root Cause Analysis & Remediation

**Task:** T-043
**Date:** 2026-03-08
**Author:** Agent (prompted by human observation)

## Problem Statement

The component fabric (`.fabric/`) was structurally initialized early in the project but never populated with component cards. This meant:

- `fw fabric blast-radius` showed "no fabric card" for every changed file
- `fw fabric deps` / `fw fabric impact` returned nothing useful
- `fw fabric overview` showed "No subsystems.yaml found"
- Task completions had no meaningful impact analysis

The human noticed this gap after 40+ tasks of active development.

## Root Cause Analysis

### 1. No Structural Gate

The framework has structural gates for:
- Tasks (PreToolUse blocks Write/Edit without active task)
- Commits (commit-msg hook requires task reference)
- Verification (P-011 blocks completion without passing checks)
- Budget (budget-gate blocks at critical context)

But there is **no gate for fabric registration**. Creating a new source file triggers no reminder, no hook, and no audit check. The fabric is purely opt-in.

### 2. `fw fabric register` is Framework-Centric

The `register.sh` script infers `type` and `subsystem` from path patterns:

```bash
case "$rel_path" in
    web/blueprints/*.py) comp_type="route" ;;
    agents/context/*) subsystem="context-fabric" ;;
    # ... only framework-specific patterns
esac
```

For a non-framework project (like TermLink's Rust crates), every file gets `type: script` and `subsystem: unknown`. The agent has no reason to believe registration adds value when the output is `subsystem: unknown`.

### 3. `fw fabric scan` Requires watch-patterns.yaml

The batch registration command (`fw fabric scan`) requires a `.fabric/watch-patterns.yaml` file that was never created. Without it, there's no way to auto-discover unregistered files.

### 4. No Onboarding Prompt

The Session Start Protocol and CLAUDE.md don't mention fabric initialization for new projects. The framework assumes the fabric is already populated (it was designed for the framework's own codebase).

### 5. `depends_on` Format is Non-Obvious

The fabric tools expect typed edges (`- target: X, type: calls`) but this format isn't documented in the card template created by `register.sh`. Simple string lists (`- path/to/file`) are silently ignored by the traversal code. An agent writing cards for the first time will use the wrong format.

## Remediation

### Immediate (Done in T-043)

1. Created `subsystems.yaml` with 4 subsystems matching TermLink's crate architecture
2. Registered all 25 source files as component cards with typed dependency edges (41 edges)
3. Verified all fabric commands work: overview, blast-radius, deps, drift

### Recommended Framework Changes

#### R-1: Add `watch-patterns.yaml` on project init

When `fw context init` runs in a new project, generate a default `watch-patterns.yaml` from common source patterns:

```yaml
patterns:
  - glob: "crates/*/src/**/*.rs"
    description: "Rust source files"
  - glob: "src/**/*.py"
    description: "Python source files"
```

#### R-2: Add fabric drift check to `fw audit`

The audit agent should report unregistered source files as a warning:

```
WARN: 5 source files have no fabric card
  crates/termlink-session/src/new_module.rs
  ...
Run: fw fabric scan
```

#### R-3: PostToolUse hook for new file creation

When `Write` creates a file matching `watch-patterns.yaml` globs, emit a reminder:

```
NOTE: New source file created. Register it: fw fabric register <path>
```

#### R-4: Document `depends_on` edge format in card template

Update `register.sh` to include a comment in the skeleton card:

```yaml
depends_on:
  # Format: - target: <path>, type: calls|reads|writes|triggers|renders
  []
```

#### R-5: Make subsystem inference configurable

Add a `.fabric/subsystem-rules.yaml` that maps path patterns to subsystems:

```yaml
rules:
  - pattern: "crates/termlink-protocol/**"
    subsystem: protocol
    type: module
  - pattern: "crates/termlink-session/**"
    subsystem: session
    type: module
```

This makes `fw fabric register` and `fw fabric scan` produce useful output for any project structure.

## Agent Prompt for Future Sessions

Add to CLAUDE.md or as a learning:

> **Fabric Maintenance Rule:** When creating a new source file (not test, not config), check if `.fabric/components/` has a card for it. If not, create one with:
> - `id` and `location` = relative path
> - `subsystem` from subsystems.yaml
> - `depends_on` with typed edges for `use`/`import` statements
> - Run `fw fabric drift` periodically to catch gaps

## Evidence

- 40 tasks completed before fabric gap was noticed
- 0 component cards existed despite 25 source files
- `fw fabric blast-radius` was running but producing empty results on every commit
- No audit warning, no hook, no reminder caught this

## Classification

- **Failure type:** Process gap (missing structural enforcement)
- **Escalation level:** D (change ways of working)
- **Pattern:** "Silent degradation" — a tool works but produces no value, and no signal alerts anyone
