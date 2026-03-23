# T-247: Research Agent Scenarios for Orchestrator.route + Bypass Registry

## Scenario 1: Cross-Codebase Grep Delegated to Specialist

### Trigger
The research agent needs to search for all usages of a protocol method (e.g., `session.discover`) across multiple project repositories. The orchestrator project and the framework project live in separate working directories, each with its own TermLink session running as a codebase indexer.

### Flow
1. Research agent calls `orchestrator.route` with:
   - `selector: { roles: ["codebase-index"], capabilities: ["grep"] }`
   - `method: "search.grep"`
   - `params: { pattern: "session\\.discover", file_glob: "**/*.rs" }`
2. Hub checks bypass registry for `"search.grep"`. Not yet promoted (first run). Proceeds to discovery.
3. Hub runs `session.discover` with the selector. Finds two sessions: `index-termlink` (tags: `["codebase-index"]`, project: termlink) and `index-framework` (tags: `["codebase-index"]`, project: framework).
4. Hub forwards `search.grep` to `index-termlink` (first candidate). Session responds with `{ matches: [...], count: 14 }`.
5. Hub records a successful orchestrated run for `"search.grep"` in `candidates` (success_count: 1).
6. Hub returns the result with routing metadata: `{ routed_to: { id: "...", display_name: "index-termlink" }, candidates: 2, result: { matches: [...] } }`.
7. Research agent receives the result, notes it only searched one codebase. For full coverage, it would issue a second `orchestrator.route` call with `selector: { name: "index-framework" }`.

### Expected Outcome
- Research agent gets grep results from the specialist session without knowing the session's socket path.
- Bypass registry records `search.grep` with `success_count: 1, fail_count: 0`.
- After 5 successful orchestrated grep calls (across sessions), `search.grep` is auto-promoted to bypass. On the 6th call, `orchestrator.route` returns `{ bypassed: true }` immediately, and the research agent knows to run the grep locally.

### Failure Mode
- **No indexer sessions running:** Hub returns `SESSION_NOT_FOUND` ("No sessions match the selector"). Research agent falls back to running `rg` locally via shell.
- **Indexer session crashed mid-search (timeout):** Hub logs "candidate timed out, trying next" and fails over to `index-framework`. If both fail, returns error with "All 2 candidate(s) failed. Last: index-framework: timeout".
- **Indexer returns partial results (e.g., permission denied on some files):** The specialist's response includes the partial result. The research agent must check the response for completeness indicators. No de-promotion occurs because the RPC itself succeeded.

### What We Learn
- **Selector-based discovery works for role-based routing.** The research agent does not hardcode session IDs; it describes what it needs.
- **Failover across candidates works transparently.** If the first indexer is down, the second is tried without the research agent knowing.
- **Bypass promotion tracks methods, not sessions.** After 5 successful `search.grep` calls (regardless of which indexer responded), the method itself is promoted. This validates that the bypass granularity (method name) is correct for research operations.

### Test Approach
1. Use `ENV_LOCK` to isolate the test. Create a temp runtime dir.
2. Start two test sessions with `roles: ["codebase-index"]` and `capabilities: ["grep"]` using `start_test_session()`. Pre-populate each session's event queue or register a custom RPC handler that responds to `search.grep`.
3. Since test sessions only handle built-in methods (e.g., `termlink.ping`), substitute `method: "termlink.ping"` as the forwarded call (it returns session info, proving routing worked).
4. Call `handle_orchestrator_route()` directly 6 times. After 5th: verify bypass registry file contains `"termlink.ping"` in `entries` with `tier: 3`. On 6th call: verify response contains `"bypassed": true`.
5. Kill the first session between calls 2 and 3. Verify the hub fails over to the second session (response `routed_to.display_name` changes).

---

## Scenario 2: Episodic Memory Query via Documentation Specialist

### Trigger
The research agent is investigating a recurring test failure. It needs to query episodic memory for past tasks that encountered similar failures (e.g., "flaky `execute_with_env` test under full suite load"). The episodic memory index is large and maintained by a documentation specialist session that has pre-loaded `.context/episodic/` and keeps an in-memory search index.

### Flow
1. Research agent calls `orchestrator.route` with:
   - `selector: { capabilities: ["episodic-search"] }`
   - `method: "memory.search"`
   - `params: { query: "execute_with_env flaky", limit: 5, fields: ["task_id", "summary", "failure_class"] }`
2. Hub checks bypass registry. `"memory.search"` has 4 prior successes (not yet promoted). Proceeds to discovery.
3. Hub discovers `doc-agent-01` with `capabilities: ["episodic-search", "learning-capture"]`. Only one candidate.
4. Hub forwards `memory.search` to `doc-agent-01`. The specialist scans its in-memory index, finds 3 matches:
   - T-041: "SessionContext registration path not set" (env-dependent test)
   - T-058: "Pre-push hook RCA — flaky under load" (directly relevant)
   - T-114: "POC agent mesh spawning — test isolation" (ENV_LOCK pattern)
5. Hub records successful orchestrated run (success_count: 5, fail_count: 0). **Promotion triggered.** `"memory.search"` added to `entries` with `tier: 3`.
6. Hub returns result plus routing metadata. Also logs: "orchestrator.route: command promoted to bypass registry".
7. Next time the research agent calls `orchestrator.route` for `memory.search`, the hub returns `{ bypassed: true }` immediately. The research agent then knows to query episodic memory directly (local file scan) rather than routing through a specialist.

### Expected Outcome
- Research agent receives structured episodic matches ranked by relevance.
- `memory.search` is promoted to bypass on the 5th successful call.
- Subsequent calls short-circuit, saving the hub round-trip and the specialist's resources.
- The research agent's local fallback (direct file grep of `.context/episodic/`) becomes the permanent path.

### Failure Mode
- **Doc specialist not running (single candidate, 0 matches):** Hub returns `SESSION_NOT_FOUND`. Research agent degrades to `grep -r "execute_with_env" .context/episodic/` (slower, unstructured, but functional).
- **Doc specialist's index is stale (returns 0 results for a known pattern):** The RPC succeeds (no failover triggered), but results are empty. Research agent detects `count: 0` and falls back to direct file scan. This is a **semantic failure that the bypass registry cannot detect** — the call succeeded mechanically but failed informationally. This reveals a gap: bypass promotion should optionally consider result quality, not just RPC success.
- **Doc specialist crashes after promotion (bypass run fails):** Research agent calls `memory.search` locally (bypass path). If the local execution fails (e.g., missing episodic files), `record_bypass_run(..., false)` de-promotes the command. Next call goes through `orchestrator.route` again, re-entering the discovery flow.

### What We Learn
- **Promotion threshold (5 runs) is reasonable for read-only queries.** Memory searches are idempotent and low-risk; 5 runs is enough to build confidence.
- **Bypass means "do it locally," not "skip it."** The research agent must have a local implementation ready before bypass is meaningful. Bypass without a local fallback is a no-op (the hub just says "bypassed: true" but the agent has no alternative).
- **Semantic failures are invisible to the registry.** A specialist that returns wrong results still counts as "success" at the RPC layer. This validates that bypass promotion should be conservative and that agents must independently validate result quality.

### Test Approach
1. Start one test session with `capabilities: ["episodic-search"]`.
2. Use `termlink.ping` as the proxy method (since test sessions cannot handle custom methods).
3. Call `handle_orchestrator_route()` 5 times with the same selector. After 5th call, read `bypass-registry.json` from the temp dir and verify the method is in `entries`.
4. Call once more. Verify response has `"bypassed": true`.
5. Write a failing bypass scenario: manually insert a promoted entry, then call `record_bypass_run("memory.search", false)` and verify the entry is removed from `entries` (de-promotion).
6. Call `handle_orchestrator_route()` again after de-promotion. Verify it goes through full discovery (response has `routed_to`, not `bypassed`).

---

## Scenario 3: Learning Capture Routed to Context Specialist Under Concurrent Load

### Trigger
Three research agents finish their investigations simultaneously (parallel dispatch pattern from T-059). Each needs to record a learning via the context specialist. The context specialist owns the `learnings.yaml` file and serializes writes to prevent corruption.

### Flow
1. Research Agent A calls `orchestrator.route`:
   - `selector: { roles: ["context-manager"], capabilities: ["learning-capture"] }`
   - `method: "context.add_learning"`
   - `params: { text: "BSD sed requires explicit backup extension", task: "T-196", source: "P-001" }`
2. Research Agent B calls `orchestrator.route` (concurrently):
   - Same selector, `method: "context.add_learning"`
   - `params: { text: "ENV_LOCK required for TERMLINK_RUNTIME_DIR tests", task: "T-041", source: "P-001" }`
3. Research Agent C calls `orchestrator.route` (concurrently):
   - Same selector, `method: "context.add_learning"`
   - `params: { text: "Bypass promotion blocked by any failure in candidate window", task: "T-238", source: "P-001" }`
4. Hub discovers one context specialist: `ctx-agent-01`. All three requests are forwarded to the same session.
5. The session's RPC handler serializes the writes internally (write lock on `learnings.yaml`). All three succeed sequentially (within the 5-second timeout each).
6. Hub records 3 successful orchestrated runs for `"context.add_learning"`. Combined with 2 prior runs, the method reaches 5 successes and is promoted to bypass.
7. Each research agent receives its confirmation independently.

### Expected Outcome
- All three learnings are written to `learnings.yaml` without corruption or lost writes.
- `context.add_learning` is promoted after this batch (prior 2 + current 3 = 5 successes).
- No failover occurs because all three calls succeed on the same (only) candidate.
- Bypass registry file is written 3 times in quick succession (once per successful orchestrated run). The last write wins, and it contains `success_count: 5` in candidates just before promotion, then the entry moves to `entries`.

### Failure Mode
- **Context specialist's write lock contention causes timeout on Agent C's request:** Hub logs timeout, tries next candidate. No next candidate exists, so returns "All 1 candidate(s) failed." Agent C must retry. The failure also records a failed orchestrated run, resetting the zero-failure requirement for promotion. Agents A and B's learnings are safe (already written).
- **Bypass registry file corruption from concurrent saves:** Two hub goroutines call `BypassRegistry::load()` + `record_orchestrated_run()` + `save()` simultaneously. Both read `success_count: 2`, both increment to 3, both save. Result: file says 3 instead of 4. One success is lost. This is a **real race condition** in the current implementation — `load()/save()` is not atomic. The registry should use file locking or an atomic write pattern.
- **Context specialist crashes after writing 2 of 3 learnings:** Agents A and B get success responses. Agent C gets a connection error, hub reports failure. Two learnings are persisted, one is lost. Agent C must retry (possibly after the specialist restarts).

### What We Learn
- **The bypass registry has a race condition under concurrent writes.** `load()` + mutate + `save()` is not atomic. Two concurrent `record_orchestrated_run()` calls can lose an increment. This is acceptable at low volume (off-by-one in promotion count) but should be documented as a known limitation or fixed with file locking.
- **Single-candidate routing means no failover for singleton specialists.** When only one session matches, failure is terminal for that call. The architecture works well for pools of specialists (code indexers, test runners) but degrades for singletons (context manager, audit agent).
- **Concurrent routing to the same specialist is the specialist's problem, not the hub's.** The hub does not serialize requests to a single target. The specialist must handle concurrent RPC calls internally (via its own write locks). This is the correct separation of concerns.
- **Bypass promotion under concurrency may promote earlier or later than expected** due to the race condition, but never incorrectly (a failed run still blocks promotion because fail_count > 0 is checked at promotion time, and that check is per-save).

### Test Approach
1. Start one test session with `roles: ["context-manager"]` and `capabilities: ["learning-capture"]`.
2. Spawn 3 concurrent `handle_orchestrator_route()` calls using `tokio::join!`, each with the same selector but different params. Use `termlink.ping` as the proxy method.
3. After all 3 complete, verify all returned `RpcResponse::Success`.
4. Read the bypass registry from disk. Verify the method is tracked (either in `candidates` or `entries` depending on prior run count).
5. To test the race condition explicitly: use `BypassRegistry::load_from()` + `record_orchestrated_run()` + `save_to()` from two concurrent tasks writing to the same file. Read back and check whether both increments were captured. Document the result (expected: one may be lost).
6. For the timeout failure mode: start a session, use a very short `timeout_secs: 0` (or 1ms equivalent) so the call times out. Verify the error response and that `fail_count` increments in the registry.
