---
id: T-1175
name: "Fix G-012 — teach fabric enrich.py to parse Rust mod/use"
description: >
  Fix G-012 — teach fabric enrich.py to parse Rust mod/use

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-04-20T23:28:37Z
last_update: 2026-04-20T23:31:46Z
date_finished: 2026-04-20T23:31:46Z
---

# T-1175: Fix G-012 — teach fabric enrich.py to parse Rust mod/use

## Context

Close G-012 (low-severity gap registered 2026-04-21): `agents/fabric/lib/enrich.py` has detectors for bash, python, HTML, TS/JS — but nothing for Rust. Result: new `.rs` files land in `.fabric/components/` with `depends_on: []` / `depended_by: []` and the `19/104 cards have no edges` audit warn accumulates. 16 of the 19 edgeless cards are Rust sources from the T-1155 bus wedges.

Adds `detect_rust_deps(content, source_location, project_root)` covering the two high-signal Rust edge patterns:
- `mod <name>;` → sibling file `<same-dir>/<name>.rs` OR `<same-dir>/<name>/mod.rs`
- `use <crate_name>::...` → `crates/<dashed-crate>/src/lib.rs` (workspace convention: `termlink_bus` → `termlink-bus`)

Intra-crate `use crate::foo::Bar` is intentionally not detected: the owning `lib.rs`/`mod.rs` already has `mod foo;` which captures the sibling edge cleanly. External third-party crates (`tokio`, `serde_json`, etc.) are skipped via `SKIP_RUST_CRATES`.

## Acceptance Criteria

### Agent
- [x] Add `detect_rust_deps` function in `.agentic-framework/agents/fabric/lib/enrich.py` that emits `(target_location, "calls")` tuples for `mod <name>;` (sibling file or `mod.rs` subdir) and `use <crate_name>::` → `crates/<kebab>/src/lib.rs`
- [x] Add `is_rust` branch in `compute_forward_edges` that dispatches to the new detector for `.rs` files
- [x] Skip list covers std + common third-party crates used in this workspace (`std`, `core`, `alloc`, `crate`, `self`, `super`, `tokio`, `serde`, `serde_json`, `anyhow`, `thiserror`, `tracing`, `rusqlite`, `ed25519_dalek`, `rand_core`, `base64`, `hex`, `sha2`, `chrono`, `uuid`, `clap`, `futures`, `async_trait`, `reqwest`, `jsonrpsee`, `tempfile`, `once_cell`, etc.)
- [x] Unit-test-style dry run on the current workspace: at least 50 new Rust edges detected against the 16 edgeless Rust cards (mod declarations in lib.rs files + cross-crate `use termlink_*` edges across cli/hub/mcp/session)
- [x] After applying, audit `Fabric: X/104 cards have no edges` drops by ≥10 (from 19 edgeless to ≤9)
- [x] No regressions: existing bash/python/html/ts edges still produced — count before and after the change on the same snapshot, verify zero decrease
- [x] `python3 -c "import ast; ast.parse(open('.agentic-framework/agents/fabric/lib/enrich.py').read())"` parses clean

## Verification

python3 -c "import ast; ast.parse(open('.agentic-framework/agents/fabric/lib/enrich.py').read())"
grep -q "detect_rust_deps" .agentic-framework/agents/fabric/lib/enrich.py
grep -q "is_rust" .agentic-framework/agents/fabric/lib/enrich.py
.agentic-framework/agents/fabric/fabric.sh enrich --dry-run 2>&1 | grep -q "Cards processed:"

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-04-20T23:28:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1175-fix-g-012--teach-fabric-enrichpy-to-pars.md
- **Context:** Initial task creation

### 2026-04-20T23:31:46Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
