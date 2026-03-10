# TermLink Workspace Architecture Analysis

**Dependency graph:** `protocol` ← `session` ← `hub` ← `cli` (strict DAG, no cycles).

**Successes:**
- Clean layered architecture: wire protocol → session lifecycle → message routing → user-facing CLI. Each crate has a single, well-scoped responsibility.
- Dependency direction is correct: lower layers have zero knowledge of higher layers. `protocol` is leaf-only (no internal deps), which is ideal for a wire-format crate.
- Workspace-level dependency centralization (`[workspace.dependencies]`) prevents version drift across crates.
- `cli` is the only crate pulling in `clap`, `tracing-subscriber`, and `anyhow` — presentation concerns stay out of library crates.

**Areas for improvement:**
- `cli` depends on all three library crates directly. If it only orchestrates `hub` (which already re-exports `session` and `protocol` transitively), the direct deps on `protocol`/`session` may indicate leaky abstractions in `hub`'s public API.
- `session` pulls in `libc` (platform-specific) and `tokio` (async runtime), coupling it to a specific runtime. Consider a trait-based transport abstraction in `protocol` to keep `session` runtime-agnostic.
- No `termlink-transport` or `termlink-io` crate — transport mechanics (Unix sockets, TCP) are likely embedded in `session` or `hub`, which may hinder future transport swaps.
- No shared `termlink-test-utils` or workspace-level integration test crate; `cli` has dev-deps but lower crates have none, suggesting limited unit test infrastructure.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-072 (test-utils crate), T-073 (transport abstraction)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
