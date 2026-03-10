# Test Suite Analysis — termlink

**Unit tests:** Strong coverage across all 3 crates (~120 tests). Every core module (manager, handler, codec, identity, lifecycle, registration, liveness, executor, scrollback, events, data_server, pty, jsonrpc, data, control, router, server) has inline `#[cfg(test)]` unit tests. Handler alone has ~25 tests covering all RPC methods including edge cases (unknown method, missing params, KV CRUD, events).

**Integration tests:** Excellent. `termlink-session/tests/integration.rs` (~17 tests) verifies full request/response cycles over Unix sockets — ping, exec, discovery, data plane I/O, event bus, KV store. `termlink-cli/tests/cli_integration.rs` (~15 tests) spawns the real binary with isolated `TERMLINK_RUNTIME_DIR` and tests the CLI end-to-end. Interactive PTY tests (`interactive_integration.rs`, 4 tests, `#[ignore]`) cover attach/stream with `rexpect`.

**Test isolation:** Good. Each test uses `AtomicU32` counters + unique `/tmp/tl-*` directories, avoiding cross-test interference. CLI tests use `ProcessGuard` (RAII) for reliable child cleanup even on panic.

**Flaky test risks:** Moderate. Data plane tests use `tokio::time::sleep(20ms)` for startup delays and 3-second deadline loops — sensitive to CI load. CLI tests poll for socket files with timeouts. The `wait` test sleeps 1s before emitting — timing-sensitive. Interactive tests are correctly `#[ignore]`d.

**Missing edge cases:** No tests for concurrent client connections, malformed JSON input to RPC handlers, socket path length limits, or permission errors on sessions_dir. `find_by_capability`/`find_by_tag`/`find_by_role` in manager.rs are untested (they use the default directory, hard to test in isolation). No negative tests for `write_atomic` failures or disk-full scenarios. `termlink-protocol` has no integration tests (only unit tests for serialization).

**Overall:** Well-structured test strategy with clear layering (unit → integration → CLI → interactive). Coverage is strong for happy paths and core error cases. Main gaps are around concurrency, adversarial inputs, and the few helper functions that hardcode the default sessions directory.

---
**Source:** T-063 reflection fleet (Level 6, 2026-03-10)
**Feeds:** T-070 (failure-mode e2e tests), T-072 (test-utils crate)
**Governance:** [docs/reports/T-063-reflection-fleet-governance.md](T-063-reflection-fleet-governance.md)
