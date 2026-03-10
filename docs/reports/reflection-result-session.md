# Session Management Analysis — `termlink-session`

- **Lifecycle**: Clean register (`Initializing→Ready`) and deregister (`Ready→Draining→file cleanup`) flow. Atomic JSON writes via temp+rename prevent partial reads. `deregister` consumes `self`, preventing use-after-deregister.
- **Liveness**: Two-tier check (PID via `kill(pid,0)` + socket file existence). Handles `EPERM` correctly (process alive but unprivileged). No socket-level probe yet (noted as TODO); a PID-recycling attack or long-lived zombie could cause false positives.
- **State machine**: `SessionState` has 5 states but no transition validation — `set_state` accepts any state from any state. Nothing prevents `Ready→Initializing` or `Gone→Ready`. Consider adding a `valid_transition()` guard.
- **Cleanup on failure**: If `register_in` fails after binding the socket but before writing JSON, the socket file leaks (no cleanup in the error path). `deregister` ignores all `remove_file` errors (acceptable for best-effort cleanup). Stale sessions are cleaned opportunistically during `list_sessions` and `find_by_display_name`.
- **Race conditions**: The display-name uniqueness check (`find_by_display_name` → `is_alive` → `register`) is a TOCTOU race — two concurrent registrations with the same name can both pass the check. File-system operations (dir scan, JSON read/write) are not locked; concurrent list+deregister could read a half-removed session. Low risk for single-host CLI usage but relevant if scaled.
- **Missing `Drop` impl**: If a `Session` is dropped without calling `deregister()`, the socket and JSON files are leaked. A `Drop` impl with best-effort cleanup would improve robustness.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-067 (session state machine validation)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
