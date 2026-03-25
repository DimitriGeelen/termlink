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

## Spike Results

### Spike 1: Session Topology — BLOCKER FOUND

**Finding:** None of the 3 active Claude sessions on .107 (pts/18, pts/22, pts/42) are registered with TermLink. The `fw-agent` TermLink session (PID 1950442) is a standalone bash shell spawned BY a Claude session — not Claude's own input channel.

14 TermLink sessions exist on .107, but zero are AI-reachable. Every session is either:
- A bash worker shell (fw-agent, email-relay)
- A project worker (copilot-*, t586-*, openclaw-eval)

**Root cause:** Claude sessions don't run `termlink register --self` at startup. This means:
- TermLink can discover them: NO
- TermLink can inject into them: NO
- TermLink can exec on their behalf: YES (via worker shells) but output goes to bash, not AI

**Chicken-and-egg:** We can't tell Claude on .107 to register because we can't reach Claude on .107. The only way in is for the human to paste a command or for the framework's Session Start Protocol to include `register --self`.

**Structural fix needed:** `register --self` must be part of the framework Session Start Protocol. Every Claude Code session should auto-register with TermLink so other agents can discover and communicate with it. This is a framework-side change (CLAUDE.md + SessionStart hook).

### Spike 2: Bidirectional Connectivity — CONFIRMED

- .112 hub running on TCP 0.0.0.0:9100 with auth
- fw-agent on .107 can `remote list` and `remote exec` on .112 sessions
- TOFU violation resolved (hub restart regenerated cert, cleared stale fingerprint)
- Round-trip exec works: fw-agent → .112 upgrade-test → `pwd` returns `/tmp/termlink-upgrade-test`

### Spike 3: Isolation — CONFIRMED

- Full clone at `/tmp/termlink-upgrade-test` — completely isolated
- `fw doctor` runs (reports expected issues: hook paths, no git hooks)
- `termlink register` works in the clone directory
- Exec from .107 lands in the correct working directory

### Spike 4: fw upgrade — BROKEN (confirms T-614)

- `fw upgrade` reports "CLAUDE.md already up to date" — but we're 822 lines vs 1001 (180 missing)
- Reports "11/10 hooks present" — but missing `check-project-boundary` and `commit-cadence`
- Hook detection is count-based, not type-based (T-615 root cause confirmed)
- CLAUDE.md sync has a false-positive bug (unreported — new finding)
- Net result: `fw upgrade` gives all green but leaves consumer ungoverned

### Spike 5: fw-agent T-614 Investigation (read from .107)

The framework team independently investigated and found:
- All 7 consumer projects stuck at v1.2.6 (framework at 1.3.0)
- 5 root causes identified: upgrade.sh hook bug, no doctor consumer check, no audit trail, Bash task gate gap, no post-update suggestion
- Remediation: T-615 (fix hook enum), T-616 (doctor consumer check), T-617 (audit trail), T-618 (fleet upgrade), T-619 (bash task gate)
- **T-615 must land first** — it unblocks everything

## Critical Gap: AI-to-AI Session Registration

For cross-project agent communication to work, Claude Code sessions MUST be discoverable via TermLink. Today they are invisible.

**Required changes (framework-side):**
1. Add `termlink register --self` to Session Start Protocol in CLAUDE.md
2. Add a SessionStart hook that auto-registers with TermLink (if installed)
3. Convention: session name = project name, tags include `claude,agent`

**Required changes (TermLink-side):**
1. `register --self` needs to work reliably in Claude Code's Bash tool environment
2. Consider: should registration be a background process or foreground?
3. Consider: MCP server as alternative registration path (Claude Desktop on .107 may have MCP)

## Dialogue Log

### Human decisions:
1. Safety first → full clone (not worktree)
2. Push model → fw-agent pushes upgrade to consumers
3. Test manually first, build command later
4. Connection details should not be hardcoded — must work in different situations
5. "WHY ARE YOU NOT USING TERMLINK COMMANDS" → use TermLink discover/list, never raw ps/proc hacks

## Open Design Questions

1. ~~Should the fw-agent connect to us, or should we pull from it?~~ → **Push** (decided)
2. ~~Git worktree vs full clone for isolation?~~ → **Full clone** (decided)
3. Does the fw-agent need a "push upgrade" command, or do we build a "pull upgrade" command?
4. How does the fw-agent know which project sessions to target? (discovery? explicit push target?)
5. ~~Should this be a one-off or a repeatable `termlink upgrade` command?~~ → **Test first, build later** (decided)
6. **NEW:** How do Claude sessions register with TermLink at startup? SessionStart hook? CLAUDE.md protocol? MCP?
7. **NEW:** Should `termlink push` detect that the target is a bash shell (not AI) and warn?
