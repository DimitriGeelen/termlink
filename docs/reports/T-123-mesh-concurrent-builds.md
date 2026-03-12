# T-123: Agent Mesh Concurrent Builds — Isolation Strategy

> Inception exploration — 3 parallel agents investigated git worktrees, mesh dispatch architecture, and prior art isolation patterns.

## Problem Statement

The TermLink agent mesh (T-114) can dispatch parallel workers for **exploration** (read-only) tasks successfully. But **build** tasks (code generation, refactoring, implementation) introduce a fundamentally different problem: multiple workers writing to the same repository simultaneously.

### What breaks without isolation

1. **File conflicts** — Two workers editing the same source file produce interleaved writes. Worker A's edit overwrites Worker B's. Neither worker sees the other's changes. The result is corrupted code that doesn't compile.
2. **Cargo build contention** — The `target/` directory uses file locks for incremental compilation metadata. Two concurrent `cargo build` processes fight over locks, causing spurious build failures and cache invalidation.
3. **Git state corruption** — Two workers staging and committing on the same branch simultaneously can corrupt the index. `git add` + `git commit` is not atomic — a race window exists between staging and commit.
4. **Semantic conflicts** — Even if workers edit different files, their changes may be semantically incompatible (e.g., Worker A renames a function that Worker B calls).

### Why this matters now

We have 3 build tasks ready (T-120, T-121, T-122) that could theoretically run in parallel. Running them sequentially wastes ~2-3x wall-clock time. But running them naively in parallel risks corrupted output that takes longer to fix than sequential execution would have taken.

**Question:** What isolation strategy should the mesh use for parallel build tasks?

---

## Findings

### Strategies Evaluated

Five isolation strategies were investigated across 3 parallel exploration agents.

#### Strategy A: File/Crate Partitioning

Assign each worker a non-overlapping set of files or crates. Workers edit their assigned files only, all within the same working directory.

| Dimension | Assessment |
|-----------|-----------|
| **How it works** | Orchestrator divides files by crate boundary. Worker A gets `crates/session/`, Worker B gets `crates/hub/`. No worker touches another's files. |
| **Setup cost** | Low — just a file manifest per worker |
| **Build cache** | Excellent — shared `target/` directory, full incremental compilation |
| **Merge complexity** | Low — changes are in different files, git merge is trivial |
| **Conflict risk** | **Medium-High** — requires perfect partitioning. If a task needs to modify a shared file (e.g., `Cargo.toml`, `lib.rs` re-exports), the partition breaks. |
| **Enforcement** | None — workers must self-discipline. A worker that edits outside its partition silently corrupts. No structural guarantee. |
| **TermLink integration** | Excellent — no infrastructure changes needed |

**Pros:**
- Zero overhead — no worktree creation, no extra disk, no cold builds
- Shared incremental build cache means fast compilation
- Natural fit for multi-crate Rust workspaces
- Workers can `cargo check` the full workspace to validate their changes compile

**Cons:**
- Relies on task decomposition being perfectly non-overlapping — fragile assumption
- No protection against accidental cross-partition edits (agent discipline, not structural)
- Shared files like `Cargo.toml`, workspace `lib.rs`, or test utilities can't be partitioned
- Cargo build contention still exists (advisory locks, but not guaranteed conflict-free)
- Semantic conflicts invisible until merge (Worker A adds a field to a struct in crate X, Worker B uses old struct in crate Y)

**Verdict:** Works well for genuinely independent crates with no shared interfaces. Breaks down when tasks touch overlapping files, which is the case for T-120/T-121/T-122.

---

#### Strategy B: Git Worktree Per Worker

Each worker gets a git worktree — a lightweight checkout of the same repository at a different filesystem path, on its own branch.

| Dimension | Assessment |
|-----------|-----------|
| **How it works** | `git worktree add -b mesh-worker-N /tmp/worktree-N HEAD` creates an isolated checkout. Worker operates entirely within its worktree. |
| **Setup cost** | Medium — ~1-2MB per worktree (shares .git/objects with main repo). Creation takes <1 second. |
| **Build cache** | Poor — each worktree has its own `target/` (unless CARGO_TARGET_DIR shared, which reintroduces contention). Cold first build per worktree. |
| **Merge complexity** | Medium — standard git branch merge. Fast-forward if non-overlapping. Three-way merge if overlapping. Conflicts are explicit and toolable. |
| **Conflict risk** | **Low** — full filesystem isolation. Workers cannot interfere with each other by design. |
| **Enforcement** | Structural — filesystem separation makes cross-worker interference impossible |
| **TermLink integration** | Good — dispatch.sh creates worktree, passes path to worker |

**Pros:**
- True isolation — each worker has its own complete working directory
- Structural guarantee, not agent discipline — impossible to accidentally edit another worker's files
- Clean merge semantics — git's three-way merge handles non-overlapping changes automatically
- Conflicts are detected at merge time with clear diff output, not silently at runtime
- Workers can freely run `cargo build`, `cargo test`, `git commit` without coordination
- Worktrees share .git/objects — storage overhead is minimal (~1-2MB per worktree vs full clone)
- Branch-per-worker provides natural audit trail (what did each worker change?)

**Cons:**
- Cold build penalty — first `cargo build` in a worktree compiles everything from scratch (~30-60s for TermLink)
- Disk usage — each worktree's `target/` can grow to ~500MB-1GB for a Rust project
- Merge overhead — orchestrator must merge N branches, resolve any conflicts, verify the merged result compiles
- Worker can't see other workers' changes during execution (may duplicate work or make incompatible assumptions)
- Worktree cleanup needed — `git worktree remove` after merge, stale worktrees accumulate if cleanup fails
- Cargo.lock divergence — each worktree resolves dependencies independently (mitigated: Cargo.lock is gitignored in this project)

**Verdict:** Safest option. The cold build penalty is the main cost. For 2-3 workers with bounded tasks, the parallel wall-clock savings outweigh the cache miss.

---

#### Strategy C: CARGO_TARGET_DIR Per Worker (Shared Source)

Workers share the same source directory but each gets an isolated build target directory via environment variable.

| Dimension | Assessment |
|-----------|-----------|
| **How it works** | `CARGO_TARGET_DIR=/tmp/target-worker-N cargo build` redirects build artifacts. Source files remain shared. |
| **Setup cost** | Low — one env var per worker |
| **Build cache** | Poor — each target dir starts cold |
| **Merge complexity** | Low — only build artifacts are separated, source changes are in-place |
| **Conflict risk** | **Medium** — solves build contention but NOT source file conflicts. Two workers editing the same `.rs` file still corrupt. |
| **Enforcement** | Partial — build isolation is structural, source isolation is not |
| **TermLink integration** | Excellent — env var passthrough in dispatch.sh |

**Pros:**
- Eliminates Cargo build lock contention entirely
- Trivial to implement (one env var)
- No git worktree overhead
- Workers can run `cargo check`/`cargo test` without blocking each other

**Cons:**
- Does NOT solve the core problem — source file conflicts remain
- Half-measure: solves symptom (build contention) not cause (shared mutable state)
- Each target dir is a cold build (~500MB, ~30-60s compile)
- Git operations still conflict (staging, committing on same branch)

**Verdict:** Useful as a component of other strategies (combine with B), not sufficient alone.

---

#### Strategy D: Copy-on-Write Workspace

Full repository copy per worker (rsync, cp, or git clone --reference).

| Dimension | Assessment |
|-----------|-----------|
| **How it works** | `rsync -a --exclude=target/ . /tmp/worker-N/` or `git clone --reference . /tmp/worker-N` |
| **Setup cost** | High — full source copy per worker (though --reference shares objects) |
| **Build cache** | Poor — cold target per copy |
| **Merge complexity** | High — diff/patch between independent repos, or manual cherry-pick |
| **Conflict risk** | **None** — fully independent copies |
| **Enforcement** | Structural — complete filesystem separation |
| **TermLink integration** | Poor — path confusion, TermLink sessions register with original repo paths |

**Pros:**
- Maximum isolation — completely independent repositories
- No git worktree limitations (worktrees share some state like hooks, config)
- Workers can do anything — destructive operations, branch switches, etc.

**Cons:**
- Heavyweight — full repo copy per worker (even with --reference, working tree is duplicated)
- Merge is painful — no shared git history means manual diff/patch, not git merge
- Path confusion — TermLink sessions, framework hooks, and CLAUDE.md all reference absolute paths
- Overkill for the problem — worktrees provide the same isolation with better merge semantics
- Cleanup is expensive — full directory trees to remove

**Verdict:** Overkill. Git worktrees provide equivalent isolation with lighter weight and better merge story.

---

#### Strategy E: Layered Patches (Diff-Based)

Workers produce diffs/patches instead of modifying files. Orchestrator applies patches in sequence.

| Dimension | Assessment |
|-----------|-----------|
| **How it works** | Workers run in read-only mode, generate `git diff` or patch files. Orchestrator collects patches, applies in order, resolves conflicts. |
| **Setup cost** | Low — no filesystem setup needed |
| **Build cache** | Excellent — single workspace, single target/ |
| **Merge complexity** | High — patch application order matters, conflicts require manual resolution |
| **Conflict risk** | **Low during generation, High during application** — workers don't interfere, but patches may not apply cleanly in sequence |
| **Enforcement** | Requires workers to produce diffs, not direct edits — architectural constraint on worker behavior |
| **TermLink integration** | Medium — result bus could transport patch files |

**Pros:**
- Workers never modify source — zero runtime conflict risk
- Single build environment — full incremental cache
- Patches are reviewable artifacts (audit trail)
- Order-independent generation (all workers run from same base)

**Cons:**
- Requires fundamentally different agent behavior — agents produce diffs, not working code
- Claude Code agents (--print mode) naturally edit files, not produce patches
- Patch application failures are cryptic and hard to debug
- Workers can't test their changes (no build step, since they don't modify source)
- Adds a complex post-processing step (patch sequencing, conflict resolution)
- No existing tooling in the mesh for patch-based workflows

**Verdict:** Elegant in theory, impractical for AI agent workflows. Agents naturally write code, not patches. Forcing a diff-only mode is fighting the tool.

---

### Comparative Summary

| | A: Partition | B: Worktree | C: Target Dir | D: CoW Copy | E: Patches |
|--|-------------|-------------|--------------|------------|-----------|
| **Source isolation** | None | Full | None | Full | Full (read-only) |
| **Build isolation** | None | Full | Full | Full | N/A |
| **Git isolation** | None | Full (branches) | None | Full (clones) | N/A |
| **Setup cost** | ~0s | ~1s | ~0s | ~5s | ~0s |
| **Cold build penalty** | None | ~30-60s | ~30-60s | ~30-60s | None |
| **Merge difficulty** | Trivial | Standard git merge | N/A | Manual diff | Patch sequencing |
| **Structural guarantee** | No | Yes | Partial | Yes | Yes |
| **Agent behavior change** | Scope awareness | None | None | None | Fundamental |
| **Disk overhead** | 0 | ~1-2MB + target | ~target only | ~full repo | 0 |

---

### TermLink-Specific Analysis: T-120, T-121, T-122

| Task | Crates Touched | Key Files Modified | Overlap Risk |
|------|---------------|-------------------|-------------|
| T-120 (EventBus) | session, hub | event_bus.rs, hub/server.rs | Overlaps T-122 on hub/server.rs |
| T-121 (PTY mode) | session | pty.rs, server.rs (new RPC handler) | Overlaps T-122 on session/server.rs |
| T-122 (Transport) | protocol, session, hub, cli | 10+ files across 4 crates | Overlaps T-120 on hub, T-121 on session |

**File overlap matrix:**

| File | T-120 | T-121 | T-122 |
|------|-------|-------|-------|
| session/src/server.rs | | X (new RPC) | X (trait refactor) |
| session/src/event_bus.rs | X (gap detection) | | |
| session/src/pty.rs | | X (mode detection) | |
| session/src/client.rs | | | X (trait refactor) |
| session/src/manager.rs | | | X (trait refactor) |
| session/src/auth.rs | | | X (trait refactor) |
| session/src/registration.rs | | | X (addr migration) |
| hub/src/server.rs | X (concurrent broadcast) | | X (trait refactor) |
| protocol/src/* | | | X (TransportAddr) |
| cli/src/main.rs | | X (status output) | X (addr usage) |

**Conclusion:** File partitioning (Strategy A) is not viable — `session/server.rs` is touched by both T-121 and T-122, `hub/server.rs` by both T-120 and T-122. Any strategy that shares the working directory risks corruption on these files.

---

## Architecture Decision

### Recommended: Strategy B — Git Worktree Per Worker

Combined with `CARGO_TARGET_DIR` per worktree (element of Strategy C) to eliminate build contention.

### Three-Tier Ownership Model

| Layer | Owns | Does NOT Own |
|-------|------|-------------|
| **TermLink (transport)** | Session lifecycle, event routing, discovery. Workers register as TermLink sessions for observability. | Filesystem isolation — TermLink is transport-agnostic by design. |
| **dispatch.sh (orchestration)** | Worktree creation/destruction, CARGO_TARGET_DIR setup, branch management, merge orchestration, result collection. | What workers do inside the worktree — that's the agent's domain. |
| **agent-wrapper.sh (execution)** | Running in provided workdir, inheriting env, producing code + commits on the worktree branch. | Isolation setup — it receives an already-isolated environment. |

### Merge Strategy

**Branch-per-worker, sequential merge by orchestrator.**

1. Each worker commits on its own branch (`mesh-worker-T120`, `mesh-worker-T121`, etc.)
2. After all workers complete, orchestrator merges branches into main one at a time
3. Non-overlapping changes merge automatically (git fast-forward or clean three-way merge)
4. Overlapping changes produce merge conflicts — orchestrator stops and reports
5. After merge, `cargo test --workspace` validates the combined result

**Why not auto-resolve conflicts?** Merge conflicts in code require semantic understanding. An orchestrator agent could attempt resolution, but the risk of introducing subtle bugs outweighs the convenience. Fail-fast is safer — a human or a fresh agent with full context resolves the conflict.

### Implementation Sketch

```bash
# dispatch.sh --isolate creates a worktree per worker
BRANCH="mesh-${WORKER_NAME}"
WORKDIR=$(mktemp -d /tmp/termlink-worktree-XXXXX)
git worktree add -b "$BRANCH" "$WORKDIR" HEAD

export CARGO_TARGET_DIR="$WORKDIR/target"
termlink run -n "$WORKER_NAME" -- agent-wrapper.sh "$PROMPT" "$WORKDIR"

# After worker completes:
# Don't remove worktree yet — orchestrator needs to merge
# Merge: git merge "$BRANCH" (orchestrator decides)
# Then: git worktree remove "$WORKDIR" && git branch -d "$BRANCH"
```

### Phased Rollout

**Phase 1 (minimal — enables T-120/T-121/T-122):**
- dispatch.sh `--isolate` flag: worktree creation, CARGO_TARGET_DIR, cleanup
- Merge is manual (orchestrator runs `git merge` commands)
- ~20 lines of bash changes

**Phase 2 (convenience):**
- `termlink run --cwd` flag: pass working directory to executor natively
- Orchestrator merge script: sequential merge + `cargo test` validation
- Result reporting via TermLink event bus

**Phase 3 (optimization — if needed):**
- Shared sccache across worktrees (compile cache without target/ sharing)
- Pre-warm worktree builds (`cargo build` before agent dispatch)
- Conflict prediction: static analysis of task file-sets before dispatch

---

## Risk Analysis

### Risks of the Recommended Approach (Strategy B)

| Risk | Severity | Likelihood | Mitigation |
|------|----------|-----------|------------|
| **Cold build penalty (~30-60s per worktree)** | Medium | Certain | Pre-warm with `cargo build` before dispatch. Workers only recompile changed crates. For 3 workers, parallel cold builds still faster than 3 sequential warm builds. |
| **Disk usage (~500MB-1GB target/ per worktree)** | Low | Certain | Temporary — cleaned up after merge. /tmp filesystem handles this. Monitor with `df` in dispatch.sh. |
| **Merge conflicts on overlapping files** | Medium | Medium | Fail-fast: report conflict, human resolves. Schedule non-overlapping tasks in parallel, overlapping tasks sequentially. |
| **Stale worktrees if cleanup fails** | Low | Low | `git worktree prune` in dispatch.sh cleanup trap. Periodic `git worktree list` health check. |
| **Worker commits on wrong branch** | Medium | Low | Worktree enforces branch — worker can't accidentally commit to main. Git worktrees lock their branch. |
| **Cargo.lock divergence** | Low | Low | Cargo.lock is gitignored. Workers resolve deps from same Cargo.toml. Lock divergence only matters if deps change, which build tasks shouldn't do. |
| **TermLink session path confusion** | Low | Low | Set `TERMLINK_RUNTIME_DIR` per worktree to isolate session discovery. Workers don't need to see main repo's sessions. |
| **Agent context limitations** | High | Medium | `claude --print` is single-shot with no session persistence. Complex build tasks may exceed single-prompt capability. Mitigate with detailed, self-contained prompts. |

### Risks of NOT Doing This (Staying Sequential)

| Risk | Severity | Likelihood |
|------|----------|-----------|
| **Wall-clock waste** — 3 tasks at ~45min each = ~2.25 hours sequential vs ~45min parallel | Medium | Certain |
| **Context budget exhaustion** — sequential tasks in one session risk hitting the 170K token critical threshold | High | High |
| **Session fragmentation** — splitting across sessions loses continuity, requires handovers | Medium | High |
| **Developer wait time** — human waiting for sequential agent work is low-value time | Medium | Certain |

---

## Advantages Summary

1. **Structural safety** — Filesystem isolation is a guarantee, not a guideline. Workers cannot corrupt each other's state regardless of agent behavior.
2. **Composability** — Worktree isolation works for ANY task pair, not just carefully-partitioned ones. No need to analyze file overlap matrices before dispatch.
3. **Audit trail** — Branch-per-worker means `git log mesh-worker-T120` shows exactly what that worker changed. Clean attribution.
4. **Fail-safe merging** — Git's merge conflict detection is battle-tested. Conflicts are surfaced, never silently resolved.
5. **Minimal infrastructure** — ~20 lines of bash in dispatch.sh. No TermLink core changes. No new dependencies.
6. **Incremental adoption** — Can use `--isolate` for tasks that need it, skip for tasks that don't (exploration, single-crate work).
7. **Parallelism multiplier** — 3 agents x 45min = 45min wall-clock instead of 135min. Context budget is per-agent, not shared.

---

## Go/No-Go Criteria

**GO if:**
- Worktree creation + cold build overhead is < 2 minutes (verified: worktree creation <1s, cold build ~30-60s)
- Merge of non-overlapping branches succeeds automatically (verified: git handles this)
- dispatch.sh `--isolate` implementation fits in one task (<50 lines of bash)

**NO-GO if:**
- Cold build penalty makes parallel slower than sequential for the specific tasks at hand
- Merge conflicts between T-120/T-121/T-122 are so severe that manual resolution takes longer than sequential execution
- Agent single-shot prompt capability is insufficient for the build tasks (too complex for `claude --print`)

## Recommendation

**GO** — Worktree isolation with CARGO_TARGET_DIR per worker.

The core rationale: **structural safety > performance optimization**. Even if parallel execution were only marginally faster than sequential, the isolation guarantee eliminates an entire class of failure modes (silent file corruption, build contention, git state corruption). The cold build penalty is real but bounded (~30-60s), and is amortized across the parallel execution time savings (~90min saved for 3 tasks).

The implementation is small (~20 lines in dispatch.sh), reversible (remove `--isolate` to go back to shared workspace), and incrementally adoptable (opt-in per dispatch, not a global change).
