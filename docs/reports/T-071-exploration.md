# T-071: E2E Test Portability — Exploration Report

> Generated: 2026-03-12 | Source: TermLink agent mesh (explore-T071)

## 1. Hardcoded Paths Found (13 total across 7 scripts)

**`CLAUDE="/Users/dimidev32/.local/bin/claude"`** in levels 1-6:
- `level1-echo.sh:17`, `level2-file-task.sh:17`, `level3-persistent-agent.sh:13`
- `level4-multi-specialist.sh:22`, `level5-role-specialists.sh:21`, `level6-reflection-fleet.sh:25`

**`/Users/dimidev32/.cargo/bin/cargo build`** in all 7 scripts:
- `level1-echo.sh:31`, `level2-file-task.sh:36`, `level3-persistent-agent.sh:28`
- `level4-multi-specialist.sh:48`, `level5-role-specialists.sh:46`, `level6-reflection-fleet.sh:43`
- `level7-failure-modes.sh:52`

## 2. Duplicated Setup Patterns

Every script repeats ~15-25 lines of identical boilerplate:
- `SCRIPT_DIR`/`PROJECT_ROOT` resolution
- `TERMLINK` path
- `CLAUDE` path
- `RUNTIME_DIR`
- cargo build
- orchestrator registration + health-check loop
- `source e2e-helpers.sh` + trap

Level 7 is the outlier — it skips `e2e-helpers.sh` and has its own cleanup (no Claude dependency).

## 3. Proposed `setup.sh` Design

A single `tests/e2e/setup.sh` that provides:
- **Binary resolution via env vars**: `CARGO` (env > `command -v` > `$HOME/.cargo/bin/cargo`), `CLAUDE_BIN` (env > `command -v`), `TERMLINK_BIN` (env > `target/debug/termlink`)
- **Shared functions**: `build_termlink()`, `register_orchestrator()`, `tl()` helper
- **Auto-sources** `e2e-helpers.sh` and sets up the cleanup trap

Each level script replaces ~15 lines with one `source` line.

## 4. Recommendation: GO

- 13 hardcoded paths = completely non-portable
- Mechanical fix, no behavioral changes, low risk
- `e2e-helpers.sh` already proves the sourcing pattern works
- Level 7 needs minor adaptation (custom cleanup)
