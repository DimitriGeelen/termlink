# T-123: Agent Mesh Concurrent Builds — Isolation Strategy

> Inception exploration — 3 parallel agents investigated git worktrees, mesh dispatch architecture, and prior art isolation patterns.

## Problem Statement

The TermLink agent mesh (T-114) can dispatch parallel workers for **exploration** (read-only) tasks. But for **build** tasks (code generation), workers writing to the same repo simultaneously risk:
1. File conflicts (two workers editing the same source file)
2. Cargo build contention (shared `target/` directory locks)
3. Git state corruption (concurrent commits on same branch)

**Question:** What isolation strategy should the mesh use for parallel build tasks?

## Findings

### Three Strategies Emerged

| Strategy | Setup Cost | Merge Complexity | Build Cache | Conflict Risk | Best For |
|----------|-----------|-----------------|------------|---------------|----------|
| **A) File/crate partitioning** | Low | Low | Excellent (shared target/) | Medium (overlapping edits) | Multi-crate, non-overlapping |
| **B) Git worktree per worker** | Medium | Medium (branch merge) | Poor (isolated target/) | Low (full isolation) | Large independent modules |
| **C) CARGO_TARGET_DIR per worker** | Low | Low | Poor (N copies) | Medium (shared source) | Quick parallel builds |

### Key Technical Findings

1. **Git worktrees are lightweight** (~1-2MB, share .git/objects). Creating/destroying is cheap.
2. **Cargo handles parallel builds** with advisory locks on metadata. `CARGO_TARGET_DIR` eliminates contention entirely.
3. **TermLink executor already supports `cwd` parameter** (executor.rs:79) — `termlink run` just passes `None` currently.
4. **SessionConfig already tracks `cwd`** (registration.rs:52) — sessions register their working directory.
5. **Cargo.lock is in .gitignore** — each worktree gets its own lock state.
6. **dispatch.sh already supports workdir** — `agent-wrapper.sh "prompt" [workdir]` parameter exists.

### TermLink-Specific Analysis: T-120, T-121, T-122

| Task | Crates Touched | Files Modified | Overlap Risk |
|------|---------------|----------------|-------------|
| T-120 (EventBus) | session, hub | event_bus.rs, hub/server.rs | Overlaps T-122 on hub |
| T-121 (PTY mode) | session | pty.rs, server.rs (new RPC) | Overlaps T-122 on session server.rs |
| T-122 (Transport) | protocol, session, hub, cli | 10+ files across 4 crates | Overlaps everything |

**Conclusion:** File partitioning alone won't work — T-122 touches files that T-120 and T-121 also modify (session server.rs, hub server.rs). Worktree isolation is needed.

### Recommended Architecture: Worktree + TermLink Dispatch

**Three-tier ownership model:**

| Layer | Responsibility |
|-------|---------------|
| **TermLink (transport)** | Session lifecycle, event routing, discovery — no isolation awareness needed |
| **dispatch.sh (orchestration)** | Creates worktree, sets CARGO_TARGET_DIR, provisions workdir, collects results |
| **agent-wrapper.sh (execution)** | Runs in provided workdir, inherits env, produces artifacts |

**Merge strategy:** Branch-per-worker, orchestrator merges after all complete. Conflicts fail loudly (no auto-resolution).

### Implementation: Enhanced dispatch.sh

```bash
# dispatch.sh --isolate creates a worktree per worker
BRANCH="mesh-${WORKER_NAME}"
WORKDIR=$(mktemp -d /tmp/termlink-worktree-XXXXX)
git worktree add -b "$BRANCH" "$WORKDIR" HEAD

export CARGO_TARGET_DIR="$WORKDIR/target"
termlink run -n "$WORKER_NAME" -- agent-wrapper.sh "$PROMPT" "$WORKDIR"

# After worker completes:
git worktree remove "$WORKDIR"
# Merge: git merge "$BRANCH" (orchestrator decides)
```

**Phase 1 (minimal):** dispatch.sh `--isolate` flag, worktree creation/cleanup, CARGO_TARGET_DIR.
**Phase 2 (optional):** `termlink run --cwd` flag, merge orchestration in fw mesh command.

## Go/No-Go Criteria

- **GO if:** Worktree isolation is lightweight enough for TermLink's use case, merge overhead is acceptable
- **NO-GO if:** Build time overhead (no shared cache) makes parallel slower than sequential

## Recommendation

**GO** — Worktree isolation with CARGO_TARGET_DIR per worker.

Rationale:
1. Worktrees are cheap (~1-2MB, shared .git/objects)
2. Eliminates all file conflict risk without complex partitioning logic
3. Branch-per-worker gives clean merge semantics
4. CARGO_TARGET_DIR is zero-contention
5. Minimal code changes (dispatch.sh only, no TermLink core changes)
6. Build cache loss is acceptable — parallel savings outweigh cache miss penalty for 2-3 workers

Risk: First build in each worktree is cold (no incremental cache). Mitigated by `cargo build` before dispatch (warm main target/) — workers only rebuild changed crates.
