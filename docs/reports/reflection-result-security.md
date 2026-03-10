# TermLink Security Review

**Trust boundaries:** Any process connecting to the Unix socket can send any RPC method to any session. `CommonParams.sender` is extracted but never validated—sessions can impersonate each other. Auth error codes (`AUTH_REQUIRED`, `AUTH_DENIED`) are defined in `control.rs:40-41` but never used by any handler.

**Input validation on RPC payloads:** JSON-RPC params are `serde_json::Value` with no schema validation or size limits. `handle_command_inject()` in `handler.rs:386-427` deserializes unbounded `keys` arrays. Event payloads and topic names are forwarded without sanitization.

**Socket permissions:** Session directory is created with `0o700` (manager.rs:48-54), which restricts access to the owning user. Socket file permissions are not explicitly set (relies on umask). This is the *only* trust boundary—adequate for single-user, insufficient for multi-user or container scenarios.

**Command injection in spawn:** `executor.rs:21-22` passes user-controlled strings directly to `sh -c` with no escaping, validation, or allowlisting. Any session that can reach the hub can execute arbitrary shell commands as the process owner. This is the highest-severity finding.

**`--dangerously-skip-permissions` in e2e tests:** Used in 4 test scripts (level1-echo.sh, level2-file-task.sh, specialist-watcher.sh, role-watcher.sh). Acceptable for CI e2e tests that need non-interactive execution, but these tests should run in isolated environments (ephemeral containers/VMs) and never on shared infrastructure.

**Recommendation:** Before any multi-user or networked deployment, implement sender authentication (e.g., SO_PEERCRED on the socket), per-method authorization, and input validation on `command.execute` payloads.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-064 (command injection fix), T-008 (security model)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
