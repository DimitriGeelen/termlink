# T-247: Code Review / Quality Agent Scenarios

Lens: **Code review / quality agent** — responsible for linting, testing, blast-radius analysis, diff review, and code quality enforcement.

---

## Scenario 1: Pre-Commit Blast-Radius Check via Test Specialist

### Trigger

The code review agent runs as a PostToolUse hook (or is invoked by the orchestrator after `fw git commit`). It detects that the commit touched `crates/termlink-hub/src/router.rs` and needs to determine which integration tests must pass before the commit is safe.

### Flow

1. **Bypass check.** The review agent asks: "Is `blast-radius:router.rs` in the bypass registry?" It calls `orchestrator.route` with `method: "orchestrator.bypass_status"` or checks the registry directly. First few runs: no bypass entry exists.

2. **Discovery.** The review agent calls `orchestrator.route` with:
   ```json
   {
     "selector": { "tags": ["test"], "capabilities": ["cargo-test"] },
     "method": "review.blast_radius",
     "params": {
       "changed_files": ["crates/termlink-hub/src/router.rs"],
       "commit_ref": "HEAD"
     }
   }
   ```

3. **Hub resolves.** The hub runs `session.discover` with the selector, finds a test-specialist session (e.g., `test-agent-01` tagged `["test", "rust"]` with capability `cargo-test`). Forwards the request.

4. **Specialist executes.** The test specialist:
   - Reads `.fabric/components/` cards to find dependents of `router.rs`
   - Determines affected test modules (`router::tests`, any integration tests importing `router`)
   - Runs `cargo test -p termlink-hub -- router` and collects results
   - Returns a structured response: `{ "passed": 14, "failed": 0, "skipped": 2, "affected_modules": ["router::tests"] }`

5. **Review agent evaluates.** All tests pass. The review agent approves the commit. It records the successful orchestrated run.

6. **Promotion tracking.** After 5 consecutive successful `blast-radius:router.rs` orchestrations with 0 failures, the command is promoted to bypass. Future blast-radius checks for `router.rs` changes skip orchestration entirely and run `cargo test -p termlink-hub -- router` locally.

### Expected Outcome

- Commit proceeds with test evidence attached.
- After 5 runs, `blast-radius:router.rs` appears in `bypass-registry.json` with tier 3.
- Subsequent commits touching `router.rs` run tests locally (no hub round-trip), shaving ~2s of discovery overhead.

### Failure Mode

**Test specialist is unavailable** (crashed, not spawned). The hub tries failover to other sessions matching `tags: ["test"]`. If no candidates exist, `orchestrator.route` returns error code `SESSION_NOT_FOUND`. The review agent must degrade gracefully:
- Option A: Run `cargo test` locally (slower, unscoped, but correct).
- Option B: Block the commit and report "No test specialist available — run tests manually."
- The failed orchestration is NOT recorded against the command's promotion stats (infrastructure failure, not command failure).

**What this validates:** The distinction between command failure (de-promotes) and infrastructure failure (ignored for promotion) must be explicit in the bypass registry API or the caller's recording logic. Currently `record_orchestrated_run` takes a boolean — it needs a third state or the caller must decide what constitutes a "real" failure.

### What We Learn

1. **Bypass granularity matters.** The bypass key should encode the changed file(s), not just the method name. `blast-radius:router.rs` is different from `blast-radius:bypass.rs` — they run different test subsets.
2. **Infrastructure vs. command failure.** The bypass registry needs to distinguish "the specialist was unreachable" from "the tests failed." Only the latter should block promotion.
3. **Local fallback is mandatory.** A review agent that cannot function without a specialist is fragile. The bypass path (local execution) must work even before promotion — it is the degraded-mode fallback.

### Test Approach

**Setup:**
1. Start a hub: `termlink hub start`
2. Register a test-specialist session: `termlink register --name test-agent --tags test,rust --capabilities cargo-test`
3. The test specialist listens for `review.blast_radius` and responds with canned pass/fail data

**Happy path:**
1. Send `orchestrator.route` with the blast-radius request via `termlink hub rpc`
2. Assert response contains `routed_to.display_name == "test-agent"` and `result.passed > 0`
3. Repeat 5 times. After the 5th, verify `bypass-registry.json` contains the command key

**Failover path:**
1. Kill the test-specialist session
2. Send the same request
3. Assert error response with `SESSION_NOT_FOUND`

**Bypass path:**
1. Pre-seed `bypass-registry.json` with the command entry
2. Send `orchestrator.route` with the same request
3. Assert response contains `"bypassed": true` and no session was contacted

---

## Scenario 2: Lint Enforcement with De-Promotion on Config Change

### Trigger

The code review agent intercepts a PR or commit that modifies `.rs` files. It needs to run `cargo clippy` scoped to the affected crates and enforce zero warnings. The lint command was previously promoted to bypass (tier 3) after 5 clean runs.

### Flow

1. **Bypass check.** The review agent finds `lint:termlink-hub` in the bypass registry. It has tier 3 status with 12 successful runs.

2. **Local execution (bypassed).** Because the command is in bypass, the review agent runs `cargo clippy -p termlink-hub -- -D warnings` locally without going through `orchestrator.route`. No hub round-trip.

3. **Failure.** A developer added a new dependency that introduced a clippy warning (`unused_import` from a transitive re-export). The lint command exits non-zero.

4. **De-promotion.** The review agent calls `record_bypass_run("lint:termlink-hub", false)`. The registry removes the entry. `lint:termlink-hub` is no longer tier 3.

5. **Fallback to orchestration.** On the next commit, the review agent has no bypass entry. It calls `orchestrator.route`:
   ```json
   {
     "selector": { "capabilities": ["clippy-lint"] },
     "method": "review.lint",
     "params": { "crate": "termlink-hub", "strictness": "deny-warnings" }
   }
   ```

6. **Specialist handles.** A lint-specialist session runs clippy, collects warnings, and returns structured diagnostics:
   ```json
   {
     "status": "fail",
     "warnings": 1,
     "details": [{ "file": "src/router.rs", "line": 15, "code": "unused_import", "message": "..." }]
   }
   ```

7. **Review agent reports.** The agent formats the lint failure as actionable feedback with file:line references.

8. **Re-promotion path.** After the developer fixes the warning, subsequent orchestrated lint runs start accumulating again from 0. After 5 more clean runs, `lint:termlink-hub` re-enters the bypass registry.

### Expected Outcome

- Lint failures de-promote the command immediately (single failure).
- The system gracefully transitions from bypass to orchestrated mode.
- Re-promotion requires a full fresh streak of 5 successes — no partial credit from before the failure.

### Failure Mode

**Stale bypass after config change.** A `Cargo.toml` change adds a new workspace member. The bypassed lint command still targets only `termlink-hub`. The new crate is never linted because the bypass key is crate-scoped. The review agent must invalidate bypass entries when workspace configuration changes. This is not currently handled by the bypass registry.

**Mitigation:** The review agent should check whether `Cargo.toml` or `Cargo.lock` changed in the commit. If so, it should skip the bypass check entirely and go through orchestration, which can discover the full workspace scope dynamically.

### What We Learn

1. **De-promotion is fast, re-promotion is slow.** This asymmetry is correct — one failure should destroy trust, but trust must be re-earned gradually. The current `record_bypass_run` implements this correctly.
2. **Bypass invalidation signals.** The registry is passive (check/record). It has no mechanism for external invalidation (e.g., "config file changed, invalidate all lint bypasses"). The caller (review agent) must implement this logic.
3. **Bypass keys need structure.** Plain string keys like `lint:termlink-hub` work but are fragile. A structured key (method + scope + config hash) would make invalidation automatic.

### Test Approach

**Setup:**
1. Start hub + register a lint-specialist session with capability `clippy-lint`
2. Pre-seed `bypass-registry.json` with `lint:termlink-hub` at tier 3

**De-promotion path:**
1. Call `record_bypass_run("lint:termlink-hub", false)` (simulating lint failure)
2. Verify the entry is removed from the registry
3. Call `orchestrator.route` with the lint request
4. Assert it routes to the lint specialist (not bypassed)

**Re-promotion path:**
1. Starting from empty registry, call `record_orchestrated_run("lint:termlink-hub", true)` five times
2. Verify the entry appears in the registry after the 5th call
3. Call `orchestrator.route` — assert `"bypassed": true`

**Config invalidation (caller-side logic, not registry):**
1. Simulate a workspace config change (touch `Cargo.toml`)
2. Review agent detects the change and skips bypass check
3. Assert that `orchestrator.route` is called even though bypass entry exists

---

## Scenario 3: Diff Review Fan-Out Across Multiple Specialists

### Trigger

A large PR touches 4 crates: `termlink-protocol` (wire format changes), `termlink-session` (client API changes), `termlink-hub` (router changes), and `termlink-cli` (new subcommand). The review agent needs specialized review for each domain. No single specialist covers all four.

### Flow

1. **Diff analysis.** The review agent runs `git diff main...HEAD --stat` and groups changed files by crate/subsystem.

2. **Parallel discovery.** The review agent issues 3 `orchestrator.route` calls (the CLI changes are simple enough to review locally):

   | Call | Selector | Method | Scope |
   |------|----------|--------|-------|
   | 1 | `{ "tags": ["review", "protocol"] }` | `review.diff` | Wire format backward compatibility |
   | 2 | `{ "tags": ["review", "session"] }` | `review.diff` | Public API surface changes |
   | 3 | `{ "tags": ["review", "hub"] }` | `review.diff` | Router correctness, failover logic |

3. **Hub routes independently.** Each call discovers a different specialist (or the same one if a generalist review agent handles multiple tags). The hub handles them as independent requests.

4. **Failover within each call.** Call 2 targets `review-session-01`, which is overloaded (timeout after 5s). The hub fails over to `review-session-02` (a second instance with the same tags). Call 2 succeeds on the second candidate.

5. **Results aggregation.** The review agent collects three responses:
   - Protocol review: "Breaking change in frame header — field `version` moved from byte 0 to byte 2. Requires migration."
   - Session review: "Public method `connect()` signature changed — semver minor bump needed."
   - Hub review: "Failover loop correctly handles timeout. New handler registered in match arm. LGTM."

6. **Composite verdict.** The review agent synthesizes: 1 blocking issue (breaking wire change), 1 advisory (semver bump), 1 pass. PR is blocked with structured feedback.

7. **Bypass tracking per domain.** Each of the 3 calls is tracked independently. `review.diff:protocol` accumulates runs separately from `review.diff:hub`. The hub review (always clean) may reach bypass threshold before the protocol review (which often finds issues).

### Expected Outcome

- All three specialist reviews complete (with one failover).
- The review agent produces a single consolidated report with per-domain findings.
- Bypass promotion tracks independently per domain — frequently-clean domains earn bypass faster.

### Failure Mode

**All specialists for one domain are unavailable.** If no session matches `tags: ["review", "protocol"]`, that review leg fails. The review agent must decide:
- **Block the PR entirely** (conservative — no review means no approval).
- **Approve with caveat** ("Protocol review unavailable — manual review required for wire format changes").
- **Route to a generalist** (fall back to a broader selector like `tags: ["review"]` and include scope hints in the params).

The correct behavior depends on blast radius. Protocol wire changes are high-risk (backward compatibility), so blocking is appropriate. Hub internal changes are lower risk, so degrading to "manual review suggested" is acceptable.

**Race condition in bypass registry.** Three parallel `orchestrator.route` calls each load the bypass registry, check, and later record results. If two calls write back simultaneously, one write may clobber the other's candidate stats. The current `BypassRegistry` loads from disk on each call (`BypassRegistry::load()`) and saves after mutation — this is not concurrency-safe under parallel orchestrator calls.

**Mitigation:** Either serialize registry writes through the hub (single writer), use file locking, or accept eventual consistency (stats may lose a count occasionally — acceptable since promotion threshold is low).

### What We Learn

1. **Fan-out is natural for review.** A single `orchestrator.route` call routes to one specialist. Parallel review of independent subsystems requires multiple calls. The hub does not need a "broadcast review" primitive — the caller handles fan-out.
2. **Bypass granularity per domain.** Different subsystems have different change frequencies and review pass rates. Per-domain bypass tracking avoids promoting review of volatile subsystems while rewarding stable ones.
3. **Concurrency in the bypass registry.** The load-modify-save pattern in `BypassRegistry` is not safe under parallel writes. This is a real gap — the hub handles multiple `orchestrator.route` calls concurrently, and each one calls `BypassRegistry::load()` + `save()` independently.
4. **Failover is transparent to the caller.** The review agent does not know that call 2 failed over to a second candidate. This is correct — failover is the hub's concern, not the caller's. The response includes `routed_to` for traceability but the caller does not need to retry.

### Test Approach

**Setup:**
1. Start hub
2. Register 3 specialist sessions with different tag combinations:
   - `review-protocol` with tags `["review", "protocol"]`
   - `review-session-01` with tags `["review", "session"]`
   - `review-hub` with tags `["review", "hub"]`
3. Each specialist listens for `review.diff` and returns canned domain-specific responses

**Parallel routing:**
1. Issue 3 `orchestrator.route` calls concurrently (tokio::join or parallel CLI invocations)
2. Assert each routes to the correct specialist (check `routed_to.display_name`)
3. Assert all 3 return successfully

**Failover within fan-out:**
1. Register a second session `review-session-02` with same tags as `review-session-01`
2. Make `review-session-01` return an error response for `review.diff`
3. Issue the route call targeting `tags: ["review", "session"]`
4. Assert it fails over to `review-session-02` and returns success

**Concurrent bypass tracking:**
1. Issue 5 parallel `orchestrator.route` calls, all targeting the same selector+method (simulating rapid successive reviews)
2. After all complete, load `bypass-registry.json`
3. Verify the candidate's `success_count` is exactly 5 (tests for write race). If it is less than 5, this confirms the concurrency gap documented above.

---

## Summary of Architectural Gaps Discovered

| Gap | Scenario | Severity | Suggested Fix |
|-----|----------|----------|---------------|
| No distinction between infrastructure failure and command failure in bypass tracking | 1 | Medium | Add a third recording mode (`record_orchestrated_run` with `infra_failure` that does not count against promotion) or make it the caller's responsibility to not record infra failures |
| No external invalidation signal for bypass entries | 2 | Medium | Add `invalidate(pattern)` method to `BypassRegistry`, or document that callers must skip bypass check when config changes |
| Bypass key is unstructured string | 2, 3 | Low | Consider structured keys (method + scope + config hash) for automatic invalidation |
| `BypassRegistry` load-modify-save is not concurrency-safe | 3 | High | The hub processes multiple `orchestrator.route` calls concurrently; registry writes can clobber each other. Fix with file locking, single-writer serialization through the hub, or an in-memory registry with periodic flush |
