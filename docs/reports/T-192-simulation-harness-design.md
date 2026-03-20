# T-192: Simulation Harness for Human AC Verification

> Inception task — exploring whether we can replace human-interactive verification
> with automated simulation using TermLink's own capabilities.

## Problem Statement

11 tasks have human ACs that were rubber-stamped as "structural pass" because they
appeared to require interactive human sessions (Claude TUI, framework sessions,
parallel dispatch). In reality, most can be verified by spawning ephemeral test
environments and using TermLink's inject/output capabilities to simulate the
human steps programmatically.

## The 11 Tasks and Their ACs

### Group A: Dispatch Scripts (T-124, T-126, T-127)
| Task | Human AC | Simulation Strategy |
|------|----------|-------------------|
| T-124 | Dispatch 2 parallel workers with `--isolate`, verify no file conflicts | Run dispatch with `echo` commands instead of Claude, check worktrees created |
| T-126 | Run dispatch with `--isolate`, verify branch has commit | Same, check git log on worker branch |
| T-127 | Run after parallel dispatch, verify branches merge cleanly | Run merge-branches.sh after simulated dispatch |

**Key insight:** Dispatch scripts don't care what runs inside — substitute `echo` for `claude`.

### Group B: tl-claude / Session Persistence (T-156, T-158)
| Task | Human AC | Simulation Strategy |
|------|----------|-------------------|
| T-156 | Launch tl-claude.sh, verify Claude TUI works with bidirectional mirroring | Spawn via tl-claude.sh, verify tmux session exists + TermLink registration |
| T-158 | Verify session persists across claude exit | Spawn, kill inner process, verify tmux session still alive, restart |

**Key insight:** "Claude TUI works" can be decomposed — session spawns, registers, persists. The TUI rendering is a bonus, not the AC.

### Group C: Framework Pickups (T-148, T-157, T-160)
| Task | Human AC | Simulation Strategy |
|------|----------|-------------------|
| T-148 | Paste prompt into framework session, verify it picks up | Create test project, spawn claude in it, inject pickup prompt, check output |
| T-157 | Paste prompt into framework Claude Code session, verify --termlink integration | Same approach |
| T-160 | Paste prompt into framework Claude Code session | Same approach |

**Key insight:** We can `fw init` a test project and spawn a real Claude session. TermLink injects the prompt; we read output to verify pickup.

**Cost concern:** Each Claude session consumes API tokens. These are real Claude invocations, not mocks.

### Group D: PTY Inject (T-178)
| Task | Human AC | Simulation Strategy |
|------|----------|-------------------|
| T-178 | Verify Enter submits in Claude Code TUI via pty inject | Spawn any interactive program (bash, python REPL), inject text + Enter, verify command executed |

**Key insight:** Enter key behavior is program-agnostic. Testing against bash is equally valid.

### Group E: Document Review (T-188, T-191)
| Task | Human AC | Simulation Strategy |
|------|----------|-------------------|
| T-188 | Review upstream-reporting.md for clarity | Structural checks: headings, code blocks, completeness |
| T-191 | Review evidence report and approve closures | Already done — this is the meta-task |

**Key insight:** "Clarity" is subjective, but we can verify structural completeness (all sections present, code blocks valid, no TODOs).

## Exploration Plan

### Spike 1: Dispatch simulation (T-124/126/127) — 15 min
- Run `dispatch.sh --isolate --worker-name sim-test -- echo "simulated"`
- Check: worktree created, branch exists, auto-commit happened
- Run `merge-branches.sh` on result
- Expected: Full pass without Claude

### Spike 2: tl-claude lifecycle (T-156/158) — 10 min
- `tl-claude.sh start sim-test`
- Verify: tmux session exists, TermLink registration
- Kill inner process, verify session persists
- `tl-claude.sh restart sim-test`
- Verify: session still registered

### Spike 3: PTY inject Enter (T-178) — 5 min
- `termlink spawn --name enter-test --backend tmux -- bash`
- `termlink pty inject enter-test "echo ENTER-VERIFIED" --enter`
- `termlink pty output enter-test --lines 5 --strip-ansi`
- Expected: Output contains "ENTER-VERIFIED"

### Spike 4: Framework pickup simulation (T-148/157/160) — 20 min
- `mkdir /tmp/fw-sim-test && cd /tmp/fw-sim-test && fw init`
- `termlink spawn --name fw-sim --backend tmux -- claude --project-dir /tmp/fw-sim-test`
- Inject pickup prompt via `termlink pty inject`
- Read output, check for acknowledgment
- **This is the expensive spike** — uses real API tokens

### Spike 5: Document structure verification (T-188/191) — 5 min
- Parse upstream-reporting.md: check headings, code blocks, no TODOs
- Parse evidence report: check all sections, verdict column
- This replaces subjective "clarity" with measurable "completeness"

## Go/No-Go Criteria

**GO if:**
- Spikes 1-3 work (dispatch, tl-claude, Enter inject) — these are free
- Spike 4 feasibility confirmed (framework session can be spawned and prompted)
- Total simulation time < 5 minutes per run

**NO-GO if:**
- TermLink spawn/inject is too fragile for test automation
- Framework session spawn requires manual interaction we can't automate
- API token cost for Spike 4 is prohibitive for routine verification

## Scope Fence

**IN:**
- Simulation scripts for the 11 specific tasks
- Reusable patterns for future human AC simulation
- Integration with existing `/self-test` skill

**OUT:**
- General-purpose test framework
- CI/CD integration (future task)
- Mocking TermLink itself
