## E2E Test Suite Analysis

**Progressive difficulty:** Well-designed 6-level ladder — L1 (single echo), L2 (file task with event verification), L3 (persistent agent handling multiple tasks via watcher), L4 (3 parallel specialists fan-out/fan-in), L5 (role-specific prompts + tool permissions + role identity in events), L6 (10-agent reflection fleet). Each level genuinely builds on prior concepts.

**Reliability:** Cleanup is solid (trap EXIT, process kill, runtime dir removal). Timeout handling is consistent (polling loops with progress reporting). Hardcoded paths (`/Users/dimidev32/.local/bin/claude`, `.cargo/bin/cargo`) reduce portability. The watchers use `set -uo pipefail` (no `-e`), which is intentional for resilience but risks silent failures.

**Watcher pattern:** `specialist-watcher.sh` and `role-watcher.sh` are well-factored — cursor-based event polling, fresh Claude context per task, clean JSON parsing via python3. Role-watcher adds tool permission differentiation and role identity in events. Main risk: no retry if Claude crashes mid-task; the watcher emits `task.completed` unconditionally.

**Reusability:** The watcher scripts are the reusable backbone (L3-L6 all use them). The orchestrator registration + health-check pattern is copy-pasted across levels — could be extracted into a shared `setup.sh`. The wait-for-completion polling loop is also duplicated.

**Missing levels:** No error/failure handling test (what happens when a specialist fails?), no event ordering/sequencing test, no resource contention test (agents writing to the same file), no graceful degradation test (orchestrator dies mid-task), and no test for the `--since` cursor edge cases in the watcher.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-070 (failure-mode e2e tests), T-071 (e2e portability)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
