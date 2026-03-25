# T-287: Cross-Project Framework Upgrade via TermLink — Inception Research

## Problem Statement

The framework agent on .107 has governance compliance fixes ready to apply to consumer projects. Today, upgrades are manual (`brew upgrade`, `fw update`, human-driven). We want the fw-agent to autonomously connect to a consumer project (.112), apply the upgrade, and validate it works — all via TermLink, no SSH.

**For whom:** Project agents and the human operator — eliminates manual upgrade ceremony.
**Why now:** We just discovered governance violations this session. The fw-agent has fixes. We need a safe, repeatable way to deliver them.

## Core Questions

### Q1: Session Topology — Who needs what?

**Here (.112 — TermLink project):**
- Claude Code runs in a non-TermLink terminal (no session registered)
- Need a TermLink session for fw-agent to target
- Options: (a) register a session in a separate terminal, (b) register --self from here

**There (.107 — framework project):**
- `fw-agent` is already registered (tags: master,claude) — confirmed via `remote list mint`
- Can reach us via our hub at 192.168.10.112:9100

**Network:** Bidirectional — both hubs are running, both machines can reach each other.

### Q2: Isolation Strategy — How to avoid destroying the live codebase?

| Option | Mechanism | Pros | Cons |
|--------|-----------|------|------|
| A: git worktree | `git worktree add /tmp/termlink-upgrade-test` | Shares git history, lightweight, easy cleanup | Still linked to main repo, accidental push risk |
| B: full clone | `git clone . /tmp/termlink-upgrade-test` | Completely isolated | Larger, separate remote config |
| C: cp -r | `cp -r . /tmp/termlink-upgrade-test` | Fastest, includes untracked files | Not a proper git repo unless .git copied |
| D: no copy | Work directly on live codebase | Zero overhead | Destructive if upgrade fails |

### Q3: Orchestration — Who drives?

| Option | Driver | Mechanism |
|--------|--------|-----------|
| A: fw-agent drives | fw-agent on .107 | Connects to .112 session, runs upgrade commands via remote exec |
| B: we drive | This agent on .112 | Sends instructions to fw-agent, receives results back |
| C: human drives | Human operator | Starts both sides manually, monitors |
| D: bidirectional | Both agents coordinate | fw-agent pushes upgrade, .112 agent validates |

### Q4: What does "validate it works" mean?

1. `fw doctor` passes
2. `fw audit` passes (or improves)
3. `cargo test --workspace` still passes (no regressions from framework changes)
4. The specific governance fixes are active (hooks fire correctly)
5. No new compiler warnings

### Q5: What does the fw-agent actually need to do?

1. Connect to .112 hub
2. Find the target session (running in the project directory)
3. Execute upgrade commands:
   - `brew upgrade agentic-fw` (or equivalent)
   - `fw update` (applies new framework files)
   - `fw doctor` (verify health)
   - `fw audit` (verify compliance)
   - `cargo test --workspace` (verify no regressions)
4. Report results back

## Dialogue Log

*(To be filled during human discussion)*

## Open Design Questions

1. Should the fw-agent connect to us, or should we pull from it?
2. Git worktree vs full clone for isolation?
3. Does the fw-agent need a "push upgrade" command, or do we build a "pull upgrade" command?
4. How does the fw-agent know which project sessions to target? (discovery? explicit push target?)
5. Should this be a one-off or a repeatable `termlink upgrade` command?
