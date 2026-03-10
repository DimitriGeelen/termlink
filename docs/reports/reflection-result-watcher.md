## Specialist Watcher Pattern Analysis

- **Reliability**: If `claude -p` crashes, the watcher emits `task.completed` regardless — no failure detection. The `|| echo` swallows non-zero exits, so the orchestrator sees success even on failure. No retry logic exists.
- **Scalability**: Processes one event at a time (head -1), blocking on each `claude -p` invocation. Multiple concurrent delegates queue up silently with no parallelism. Each JSON field requires a separate python3 subprocess (4 per event).
- **Error recovery**: Errors in `termlink emit` are suppressed (`2>/dev/null || true`). If JSON parsing fails, variables get `?` or empty strings — the watcher proceeds with broken state rather than skipping or reporting.
- **Cursor management**: Cursor tracks the last-processed seq number, but only one event is consumed per loop iteration. If multiple events arrive, only the first is processed; the rest require additional loop cycles. Cursor is in-memory only — a watcher restart reprocesses all historical events.
- **Shutdown handling**: File-based (`$RUNTIME_DIR/shutdown`) checked once per loop top. A long-running `claude -p` call blocks shutdown detection indefinitely. No SIGTERM/SIGINT trap handlers.
- **Resource cleanup**: Prompt files (`prompt-$REQUEST_ID.txt`) accumulate in RUNTIME_DIR and are never deleted. PID file written but never cleaned up on exit. `role-watcher.sh` appends to `pids.txt` (no cleanup on exit either).
- **role-watcher.sh delta**: Adds role-based system prompts and per-role tool restrictions — good separation of concerns. Same structural issues as specialist-watcher.sh (identical polling/dispatch loop).

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-065 (fix watcher false-completion)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
