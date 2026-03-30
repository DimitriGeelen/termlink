# T-789: Worktree Isolation for TermLink-Dispatched Agents

## Current State

### What exists today

**Shell-based isolation (agents/mesh/dispatch.sh --isolate):**
- Creates `git worktree add -b mesh-{worker-name} /tmp/termlink-worktree-XXXXX HEAD`
- Sets `CARGO_TARGET_DIR` per worker (no build artifact conflicts)
- Auto-commits worker changes on exit
- Preserves branch if worker made changes (for merge orchestration)
- Cleans up worktree + branch if no changes
- Workers execute via `termlink run` with `cd $WORKDIR`

**Rust dispatch command (termlink dispatch):**
- Spawns workers via terminal/tmux/background (spawn_via_*)
- Workers inherit parent CWD — **no isolation**
- No `--isolate` flag, no `--workdir` flag
- Collects results via event system (event.collect)

**Trust model (trust.rs):**
- `BlastRadius::Low` explicitly mentions "worktree, temp files" as isolated scope
- Architectural intent exists but not enforced in dispatch

**Claude Code Agent tool:**
- Has `isolation: "worktree"` parameter for built-in worktree isolation
- Framework hooks (T-533) push users toward TermLink dispatch instead of Agent tool
- The intent is: TermLink dispatch should provide equivalent isolation

### The gap

The Rust `termlink dispatch` command — which is the primary recommended dispatch mechanism — has **no isolation capability**. The shell script `dispatch.sh --isolate` has it, but is a separate code path that doesn't use the Rust dispatch infrastructure (event collection, structured output, JSON mode).

## Design Options

### Option A: Add --isolate to Rust `termlink dispatch`

Port the worktree logic from dispatch.sh into the Rust dispatch command.

**Implementation:**
- `dispatch.rs`: Add `--isolate` CLI flag and `--workdir` param
- Before spawning each worker: `git worktree add -b dispatch-{worker-name} /tmp/... HEAD`
- Pass `WORKDIR` env to spawned worker
- On worker completion: auto-commit, remove worktree, preserve branch if changes
- Report branches in dispatch result JSON

**Complexity:** Medium (~200 LOC). Needs git2 or Command::new("git") for worktree ops.

### Option B: Add --workdir to Rust `termlink dispatch` (no git worktree)

Let dispatch set the working directory for each worker, but don't manage git worktrees.

**Implementation:**
- `dispatch.rs`: Add `--workdir` flag
- Workers cd into the specified directory
- Caller is responsible for creating worktrees (or any other isolation)

**Complexity:** Low (~30 LOC). Just pass env var to spawned process.

### Option C: Keep shell script, improve Rust dispatch to call it

Make Rust dispatch optionally delegate to dispatch.sh for isolation, keeping worktree logic in bash.

**Implementation:**
- `dispatch.rs`: Add `--isolate` flag that wraps worker spawn in dispatch.sh
- Or: `dispatch.rs` calls dispatch.sh as the spawn backend

**Complexity:** Low (~50 LOC). But two code paths for dispatch, harder to maintain.

### Option D: Status quo — use dispatch.sh for isolation, Rust dispatch for non-isolated

Accept that dispatch.sh and `termlink dispatch` serve different use cases. Document when to use which.

**Implementation:** Documentation only.

**Complexity:** Zero code. But users need to know which tool to use when.

## Directive Scoring

### Scoring Rubric
- **++** Strongly advances the directive
- **+** Advances the directive
- **0** Neutral
- **-** Works against the directive
- **--** Strongly works against

### Option A: Add --isolate to Rust dispatch

| Directive | Score | Rationale |
|-----------|-------|-----------|
| **D1: Antifragility** | **++** | Isolation prevents cross-worker interference. Failed worker can't corrupt main branch. Auto-commit preserves work on failure. Structural enforcement > behavioral discipline. |
| **D2: Reliability** | **++** | One code path for dispatch. Worktree lifecycle tied to process lifecycle (Rust cleanup is more reliable than bash trap). Structured JSON output includes branch info. |
| **D3: Usability** | **++** | Single command: `termlink dispatch --isolate --workers 3`. No need to know about dispatch.sh. Discovery via `--help`. JSON output for automation. |
| **D4: Portability** | **+** | Git worktrees are standard git. No provider lock-in. But ties to git (not hg/svn). Rust implementation works cross-platform. |
| **Total** | **7/8** | |

#### Steelman
This is the natural evolution of the codebase. The mesh dispatch.sh was the prototype; now promote the proven pattern into the maintained Rust binary. One dispatch command, one isolation mechanism, one output format. The trust model already anticipates it.

#### Strawman
Over-engineering. The shell script works fine. Adding git operations to Rust dispatch increases binary size (git2 crate) and maintenance surface. What if users use non-git VCS? The Rust dispatch is already complex with spawn backends, event collection, and timeout handling.

---

### Option B: Add --workdir only (no git management)

| Directive | Score | Rationale |
|-----------|-------|-----------|
| **D1: Antifragility** | **0** | Doesn't prevent git conflicts. Workers in different dirs but same branch can still clash. No auto-commit safety net. |
| **D2: Reliability** | **+** | Simple mechanism, fewer failure modes. But isolation is incomplete — user must manage git state. |
| **D3: Usability** | **-** | User must create worktrees manually. Two-step process: create worktree, then dispatch. Easy to forget cleanup. |
| **D4: Portability** | **++** | VCS-agnostic. Works with any project, not just git. Minimal assumptions. |
| **Total** | **2/8** | |

#### Steelman
Maximum portability. TermLink is a terminal multiplexer, not a git tool. Worktree management belongs in the caller's domain. This keeps TermLink focused on its core mission (session coordination) and avoids git coupling.

#### Strawman
Half a solution. Users who need isolation will still need to write their own worktree scripts, duplicating dispatch.sh logic. The "portability" benefit is theoretical — every TermLink project today is git-managed.

---

### Option C: Rust dispatch delegates to dispatch.sh

| Directive | Score | Rationale |
|-----------|-------|-----------|
| **D1: Antifragility** | **+** | Gets isolation via existing proven script. But bash trap cleanup is less reliable than Rust Drop/cleanup. |
| **D2: Reliability** | **-** | Two code paths for the same operation. Shell script may drift from Rust command. Error reporting crosses process boundaries. |
| **D3: Usability** | **0** | Single CLI entry point, but implementation leaks (dispatch.sh must be on PATH, correct version, etc.). |
| **D4: Portability** | **-** | Bash dependency. Won't work on Windows. Script must be distributed alongside binary. |
| **Total** | **0/8** | |

#### Steelman
Reuses proven code without rewriting. Shell scripts are easy to modify and extend. The existing dispatch.sh has been battle-tested across multiple sessions.

#### Strawman
Worst of both worlds. Two implementations to maintain, bash portability issues (macOS bash 3.2 — a known pain point per T-160), and the Rust binary now depends on a shell script being correctly installed alongside it.

---

### Option D: Status quo (document, don't change)

| Directive | Score | Rationale |
|-----------|-------|-----------|
| **D1: Antifragility** | **0** | Isolation exists for those who know about dispatch.sh. No structural enforcement for those who don't. |
| **D2: Reliability** | **-** | Two dispatch mechanisms with different capabilities and different output formats. Users must choose correctly. |
| **D3: Usability** | **--** | "Use dispatch.sh --isolate for isolation, termlink dispatch for non-isolated" violates the principle of least surprise. New users won't discover dispatch.sh. |
| **D4: Portability** | **0** | No change. Both mechanisms have their own portability characteristics. |
| **Total** | **-3/8** | |

#### Steelman
Zero risk. No new code, no new bugs. The shell script works. Power users can find it. Documentation costs nothing.

#### Strawman
Technical debt by inaction. The trust model says "worktree = Low blast radius" but dispatch doesn't enforce it. Users follow the recommended path (`termlink dispatch`) and get no isolation. The gap between intent and implementation widens.

---

## Summary Matrix

| Option | D1 Antifragility | D2 Reliability | D3 Usability | D4 Portability | Total |
|--------|-------------------|----------------|--------------|----------------|-------|
| **A: Rust --isolate** | ++ | ++ | ++ | + | **7** |
| B: --workdir only | 0 | + | - | ++ | **2** |
| C: Delegate to bash | + | - | 0 | - | **0** |
| D: Status quo | 0 | - | -- | 0 | **-3** |

## Recommendation

**Option A: Add --isolate to Rust dispatch.**

The shell prototype validates the pattern. The Rust implementation gets proper lifecycle management (Drop vs trap), structured output (JSON with branch info), and discoverable CLI ergonomics. It closes the gap between trust model intent and dispatch behavior.

**Implementation phases:**
1. Add `--workdir` flag first (Option B as stepping stone — useful on its own)
2. Add `--isolate` flag that manages git worktrees end-to-end
3. Deprecate dispatch.sh or make it a thin wrapper

**Open question for human:** Should `--isolate` be opt-in or default? The trust model suggests it should be default for any mutating dispatch, but that changes existing behavior.

## Dialogue Log

### 2026-03-30 — Human question + research
- Human asked: "am i correct each termlink agent is spawned with its own worktree?"
- Answer: No — Rust dispatch has no isolation. Shell dispatch.sh has `--isolate` flag.
- Human requested inception with options, steelman/strawman, directive scoring
- Research: Found dispatch.sh (T-124, completed), trust.rs blast radius model, Rust dispatch gap
- Produced 4 options scored against all 4 directives
- Recommendation: Option A (7/8) — port proven shell pattern to Rust
