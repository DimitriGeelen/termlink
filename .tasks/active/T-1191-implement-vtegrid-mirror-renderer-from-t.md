---
id: T-1191
name: "Implement vte+grid mirror renderer (from T-235 GO)"
description: >
  T-235 GO on vte tokeniser + in-process minimal grid. Add vte 0.13 + unicode-width deps, create mirror_grid.rs with Grid + vte::Perform impl handling CUP/CUU/CUD/CUF/CUB/EL/ED/SGR/DECSET/DECRST + render_diff dirty-compression. Wire into mirror_loop; gate with --raw flag for byte passthrough. Golden tests for vim/htop/ls --color/less. Benchmark <16ms/frame on 1MB vim session. See docs/reports/T-235-terminal-output-rendering.md Follow-on Build Task Scope.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [termlink, mirror, rendering, vte]
components: []
related_tasks: [T-235, T-234, T-236]
created: 2026-04-22T18:41:26Z
last_update: 2026-04-23T14:32:32Z
date_finished: null
---

# T-1191: Implement vte+grid mirror renderer (from T-235 GO)

## Context

Follow-on build task from T-235 inception (GO recorded 2026-04-22T10:07:59Z,
approach C — `vte` tokeniser + in-process minimal grid). Full research artifact
with trade-off analysis lives at `docs/reports/T-235-terminal-output-rendering.md`.

Replaces the current byte-passthrough mirror_loop (`crates/termlink-cli/src/commands/pty.rs:595-684`)
with a grid-aware render path for cursor-addressable TUIs (vim, htop, less).

## Acceptance Criteria

### Agent
- [x] `vte = "0.13"` and `unicode-width = "0.1"` added to `crates/termlink-cli/Cargo.toml` — T-1199
- [x] New file `crates/termlink-cli/src/commands/mirror_grid.rs` defines `struct Grid { cols, rows, cells: Vec<Cell>, cursor: (u16, u16), scroll_region: (u16, u16), sgr_state: SgrState }` — T-1199
- [x] `impl vte::Perform for Grid` handles `print`, `execute` (CR/LF/BS/BEL), `csi_dispatch` for CUP/CUU/CUD/CUF/CUB/EL/ED/SGR/DECSET(25/1049)/DECRST — T-1199/T-1200
- [x] `Grid::render_diff(&mut self, out: &mut impl Write) -> io::Result<()>` emits only changed cells since last render (dirty-cell compression; no full-screen redraw per frame) — T-1201
- [x] `mirror_loop` feeds each `FrameType::Output` payload through `vte::Parser::advance(&mut grid, byte)` then calls `render_diff` — T-1199 (`mirror_loop_grid` in pty.rs)
- [x] `--raw` CLI flag on `termlink mirror` preserves byte passthrough (backwards-compat for non-TUI users) — T-1199
- [ ] Tests in `crates/termlink-cli/tests/mirror_grid.rs` cover: vim opening /etc/passwd, htop 1s refresh tick, `ls --color`, `less` paging (golden-input JSON → golden-output byte stream) — **DEFERRED**: requires captured byte-stream corpora. Unit tests (12 in mirror_grid.rs) cover the dispatch table directly.
- [ ] Benchmark: `cargo bench --bench mirror_render` emits 1 MB of captured vim traffic through grid, asserts median latency <16 ms/frame (60 FPS target) — **DEFERRED**: needs `cargo bench` scaffold + corpus.
- [x] `cargo build --workspace` and `cargo test --workspace` both succeed — verified 2026-04-23: 216 tests pass.
- [x] Binary size delta <100 KB — **MEASURED 2026-04-23**: 82 KB (baseline 246afdb3 = 17,385,016 B; current 456f6765 = 17,469,816 B; delta = 84,800 B). Under the T-235 GO budget.

### Human
- [ ] [REVIEW] Accept or reject on LoC-budget threshold
  **Steps:**
  1. `git diff --stat main..HEAD -- crates/termlink-cli/src/commands/mirror_grid.rs` — read line count
  2. If ≤400 LoC: accept; T-235 GO criteria met
  3. If 400-1000 LoC: discuss whether the extra scope is justified (which CSI sequences drove the overage)
  4. If >1000 LoC: NO-GO per T-235 criteria — switch to alacritty_terminal grid OR ship A+B-only
  **Expected:** Grid stays within the 400-LoC GO budget; complexity comes from the SGR state machine, not from sequence coverage bloat.
  **If not:** The vte approach was mis-scoped; re-open T-235 with new evidence.

- [ ] [REVIEW] Live mirror comparison against vim
  **Steps:**
  1. Spawn sender: `termlink spawn vim-demo -- vim /etc/passwd`
  2. In another shell: `termlink mirror vim-demo --raw` (old path) — note any geometry/cursor artefacts
  3. Same shell: `termlink mirror vim-demo` (new grid path) — compare
  **Expected:** Grid path renders cleanly when viewer cols < source cols; raw path shows the pre-existing artefacts.
  **If not:** Either cursor-positioning CSI handling is incomplete, or the diff-render is emitting stale cells. Check `Grid::render_diff` dirty-cell logic.

## Verification

cargo build --workspace --quiet
cargo test --package termlink --test mirror_grid -- --quiet
grep -q '^vte = "0\.13"' crates/termlink-cli/Cargo.toml
test -f crates/termlink-cli/src/commands/mirror_grid.rs

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

### 2026-04-22T18:41:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1191-implement-vtegrid-mirror-renderer-from-t.md
- **Context:** Initial task creation

### 2026-04-23T13:49:19Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
