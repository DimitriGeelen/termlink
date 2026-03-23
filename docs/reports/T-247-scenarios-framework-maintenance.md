# T-247: Framework Maintenance Agent — Orchestration Scenarios

Agent lens: **Framework maintenance** — responsible for health checks, audits, metrics, context management, and housekeeping.

---

## Scenario 1: Delegated Health Check with Bypass Promotion

### Trigger

The maintenance agent runs `fw doctor` as part of its periodic health sweep. This is a read-only, bounded-output, deterministic command — a textbook Tier 3 candidate.

### Flow

1. **Agent calls `orchestrator.route`** with:
   ```json
   {
     "method": "fw doctor",
     "selector": { "roles": ["framework"], "capabilities": ["health-check"] },
     "params": { "project_dir": "/path/to/project" },
     "timeout_secs": 10
   }
   ```
2. **Hub checks bypass registry.** On the first 4 runs, no entry exists. Hub proceeds to discover a specialist session with role `framework` and capability `health-check`.
3. **Hub discovers candidate.** A framework-ops specialist session is found. Hub forwards the `fw doctor` request via the session's Unix socket.
4. **Specialist executes and responds** with structured health results (exit code, warnings, failures).
5. **Hub records orchestrated run** in the bypass registry: `record_orchestrated_run("fw doctor", true)`. Candidate counter increments.
6. **On the 5th successful run** (zero failures), the registry promotes `fw doctor` to bypass. `record_orchestrated_run` returns `true`.
7. **On the 6th run**, the hub's bypass check in `handle_orchestrator_route` hits the registry entry. Instead of discovering and forwarding, it returns `{ "bypassed": true, "command": "fw doctor", "tier": 3 }`. The calling agent sees the bypass signal and executes the command locally.

### Expected Outcome

- First 5 runs: routed through specialist, each ~200ms (discovery + forward + response).
- Run 6+: bypass signal returned in <5ms. Agent executes locally. No specialist session needed.
- Registry file at `{runtime_dir}/bypass-registry.json` contains an entry for `fw doctor` with `tier: 3`, `promoted_at` timestamp, and `run_count` tracking post-promotion executions.

### Failure Mode

**Specialist session dies mid-health-check.** The hub's forward times out after `timeout_secs`. Hub should try the next candidate (failover). If no candidates remain, return an error response. The orchestrated run is recorded as a failure (`record_orchestrated_run("fw doctor", false)`), resetting the candidate's fail_count and blocking promotion.

**fw doctor itself fails after bypass promotion** (e.g., a required binary was uninstalled). The agent records `record_bypass_run("fw doctor", false)`, which de-promotes the command. Next run goes back through orchestrator routing, and the promotion cycle restarts from zero.

### What We Learn

- **Bypass promotion lifecycle end-to-end:** Does the 5-run threshold feel right for a command that runs every session? (It will be promoted by the 2nd session.)
- **Bypass signal handling at the caller:** The hub returns `"bypassed": true` but does NOT execute the command. The calling agent must interpret this and run locally. This is a design boundary worth validating — the hub is a router, not an executor.
- **De-promotion recovery:** After a bypass failure, does the system correctly fall back to orchestrated routing? Does the specialist still exist, or has it been idle-timed-out during the bypass period?

### Test Approach

1. Start a hub (`termlink hub start`).
2. Register a specialist session with `--roles framework --capabilities health-check`. The session runs a mock script that echoes `{"status": "ok"}`.
3. Register a client session (the maintenance agent).
4. From the client, call `orchestrator.route` with the `fw doctor` method 5 times via `termlink hub rpc`.
5. Assert: first 5 calls return a result from the specialist (not `"bypassed": true`).
6. Assert: 6th call returns `"bypassed": true`.
7. Load `bypass-registry.json` from the runtime dir and verify the entry exists.
8. Simulate a bypass failure by calling `orchestrator.bypass_status` or directly mutating the registry, then verify de-promotion.

---

## Scenario 2: Cross-Specialist Audit Aggregation

### Trigger

The maintenance agent needs to run a full compliance audit (`fw audit`). The audit has three independent phases: task compliance (are all tasks well-formed?), context compliance (is the fabric healthy?), and git compliance (do all commits have task references?). Each phase can be handled by a different specialist.

### Flow

1. **Agent calls `orchestrator.route` three times in parallel**, each with a different method:
   ```json
   // Call A
   { "method": "audit.tasks", "selector": { "capabilities": ["audit-tasks"] } }
   // Call B
   { "method": "audit.context", "selector": { "capabilities": ["audit-context"] } }
   // Call C
   { "method": "audit.git", "selector": { "capabilities": ["audit-git"] } }
   ```
2. **Hub discovers specialists independently.** Each call goes through the full discover-filter-forward cycle. Calls A, B, C may hit the same specialist (if one session has all three capabilities) or different specialists.
3. **Each specialist runs its audit phase** and returns structured results: `{ "pass": N, "warn": N, "fail": N, "details": [...] }`.
4. **Hub records three orchestrated runs**, one per method. Each method has its own promotion track in the bypass registry.
5. **Maintenance agent aggregates** the three responses into a unified audit report and writes it to `.context/audits/`.
6. **Over multiple sessions**, `audit.tasks` (always passes, fast, deterministic) gets promoted to bypass. `audit.git` (depends on repo state, occasionally surfaces warnings that the specialist interprets) does not — it remains orchestrated because occasional "failures" (audit warnings surfaced as non-zero exit) reset its fail_count.

### Expected Outcome

- Three parallel `orchestrator.route` calls complete independently. No ordering dependency.
- The maintenance agent receives three structured responses and combines them.
- After several sessions, the bypass registry shows `audit.tasks` promoted but `audit.context` and `audit.git` still in the candidates map.
- The hub handled the parallel discovery correctly without race conditions on the session manager.

### Failure Mode

**One specialist is overloaded (already handling another request).** The hub forwards to it, but the specialist's response is slow. The hub's per-target timeout fires. Hub should try the next candidate for that specific call (failover within one `orchestrator.route`, not across all three). The other two calls complete normally.

**No specialist has the `audit-git` capability.** The hub returns an error for Call C: no matching candidates. The maintenance agent must handle partial results — it has task and context audit data but not git audit data. It should report the partial audit and flag the missing capability.

**Two audit calls routed to the same specialist session.** This is valid (one session can have multiple capabilities), but it serializes execution within that session. The test should verify that parallel calls to the same session do not deadlock and that the specialist handles concurrent RPC requests correctly.

### What We Learn

- **Parallel `orchestrator.route` calls:** The hub must handle concurrent discovery and forwarding without session manager lock contention.
- **Heterogeneous bypass promotion:** Different methods from the same agent promote at different rates based on their individual reliability profiles. The registry is per-method, not per-agent.
- **Partial failure resilience:** The maintenance agent must not treat one failed route as a total audit failure. This validates the agent-side error handling contract.

### Test Approach

1. Start a hub.
2. Register three specialist sessions, each with one audit capability (`audit-tasks`, `audit-context`, `audit-git`).
3. Register a client session.
4. Dispatch three `orchestrator.route` calls concurrently from the client (use `tokio::join!` or three parallel CLI invocations).
5. Assert: all three return valid results with the expected structure.
6. Kill the `audit-git` specialist and repeat. Assert: two succeed, one returns an error with "no matching candidates".
7. Run the task-audit call 5 times. Assert: 6th call returns `"bypassed": true`. Assert: the other two methods are NOT bypassed.

---

## Scenario 3: Stale Session Cleanup via Discovery Sweep

### Trigger

The maintenance agent runs a periodic housekeeping sweep to detect and clean stale sessions. This involves discovering all registered sessions, pinging each one, and removing unresponsive registrations. The sweep itself uses `orchestrator.route` to delegate the actual cleanup to a specialist that has filesystem access to the runtime directory.

### Flow

1. **Agent calls `session.discover`** directly (not via `orchestrator.route`) with no filters to get the full session list. This is a hub-local operation — the hub queries its own session manager.
2. **Agent identifies suspects.** For each session, the agent checks `state` and last-seen timestamps. Sessions with `state: "idle"` and no heartbeat for >5 minutes are suspects.
3. **Agent calls `orchestrator.route`** to delegate the cleanup:
   ```json
   {
     "method": "session.cleanup",
     "selector": { "roles": ["maintenance"], "capabilities": ["session-cleanup"] },
     "params": { "targets": ["stale-session-1", "stale-session-2"] }
   }
   ```
4. **Hub checks bypass registry.** On early runs, `session.cleanup` is NOT bypassed (it mutates state — removes socket files, deregisters sessions). This is Tier 1, not Tier 3. It should NEVER be promoted to bypass because it is a mutating operation.
5. **Hub discovers a maintenance specialist** and forwards the cleanup request.
6. **Specialist performs cleanup:** pings each target via `termlink ping <target>`. Unresponsive sessions get their socket files removed and their registration entries cleaned from the runtime directory. Specialist returns `{ "cleaned": ["stale-session-1"], "still_alive": ["stale-session-2"] }`.
7. **Agent records result.** `session.cleanup` is recorded as a successful orchestrated run. However, even after 5+ successes, it should NOT be promoted because the specialist's response modifies system state. The bypass decision is about the nature of the command, not just its track record.

### Expected Outcome

- Discovery returns the full session list including stale entries.
- The cleanup specialist successfully removes dead sessions and their socket files.
- The runtime directory is cleaner — only live sessions remain.
- `session.cleanup` accumulates successful runs in the bypass registry's `candidates` map but is **never promoted** because the orchestrator (or the agent) classifies it as a mutating command.

### Failure Mode

**The cleanup specialist is itself the stale session.** The hub discovers it as a candidate, but the forward times out because the specialist is unresponsive. No other candidates exist. The maintenance agent must fall back to direct cleanup (it has the capability locally) or create a new specialist session.

**Race condition: session comes back alive during cleanup.** The specialist removes a socket file, but the session was just slow (not dead). The session re-registers on its next heartbeat, creating a new socket. The cleanup is harmless — the old socket was stale, and the session self-healed. But the specialist's response should include the target in `still_alive` if the ping succeeded during the cleanup window.

**Cleanup deletes a session that another agent is actively using.** The maintenance agent's staleness heuristic (idle + no heartbeat for 5 min) was too aggressive. The deleted session's client gets a connection error on its next RPC call. This validates that the staleness threshold needs tuning, and that agents must handle "session disappeared" errors gracefully by re-discovering.

### What We Learn

- **Bypass promotion is not purely mechanical.** A command can have 100% success rate and still be ineligible for bypass because it mutates state. This means the bypass decision needs an additional signal beyond run history — either a `mutating: true` flag in the route params, or the specialist self-declaring its methods as read-only vs. read-write. The current implementation promotes purely on success count, which is a gap this scenario exposes.
- **Discovery as a precondition for routing.** The maintenance agent uses `session.discover` (direct hub RPC) for its own logic before using `orchestrator.route` for delegation. These are complementary, not competing patterns.
- **Self-referential failure.** When the maintenance infrastructure itself is stale, the system needs a fallback. This is the "who watches the watchmen" problem — and it suggests that cleanup should have a local fallback path, not just an orchestrated one.

### Test Approach

1. Start a hub.
2. Register 3 sessions: a maintenance specialist (`--roles maintenance --capabilities session-cleanup`), and 2 dummy sessions.
3. Kill the 2 dummy sessions' processes WITHOUT deregistering them (simulate unclean exit — their socket files remain but no process is listening).
4. From a client session, call `session.discover` with no filters. Assert: all 3+ sessions appear (including the dead ones).
5. Call `orchestrator.route` with `session.cleanup` targeting the dead sessions. Assert: specialist responds with the dead sessions in `"cleaned"`.
6. Call `session.discover` again. Assert: the dead sessions no longer appear.
7. Run `session.cleanup` 6 times total. Load the bypass registry. Assert: `session.cleanup` is in `candidates` (not `entries`) — it was never promoted despite 5+ successes. (Note: this assertion requires implementing the mutating-command exclusion; if the current code would promote it, the test documents the gap.)
