# T-247: Infrastructure / Deploy Agent Scenarios

Lens: **Infrastructure / deploy agent** -- server operations, remote session management, deployments, package distribution (Homebrew), environment setup.

---

## Scenario 1: Rolling Homebrew Formula Update Across Remote Build Hosts

### Trigger

A new version of the `termlink` formula is ready to publish. The deploy agent needs to run `brew audit --formula termlink.rb` and `brew install --build-from-source termlink` on each build host (macOS ARM, macOS x86) before pushing to the tap.

### Flow

1. Deploy agent sends `orchestrator.route` with selector `{ tags: ["build-host"], capabilities: ["brew"] }`, method `shell.exec`, params `{ command: "brew audit --formula termlink.rb" }`.
2. Hub discovers two candidate sessions: `build-arm64` and `build-x86_64`, both registered with the `brew` capability.
3. Hub forwards to `build-arm64` first. The session executes the audit and returns `{ exit_code: 0, stdout: "..." }`.
4. Hub records a successful orchestrated run for `shell.exec:brew-audit` (composite key).
5. Deploy agent sends a second `orchestrator.route` with the same selector but method params for `brew install --build-from-source`. This time the hub tries `build-arm64`, which succeeds.
6. Deploy agent repeats both calls targeting `build-x86_64` explicitly (by name selector) to ensure both architectures are covered.
7. After 5 successful cycles of `brew audit` across releases, the composite command earns bypass promotion. On the 6th release, the hub returns `{ bypassed: true }` immediately, and the deploy agent executes the audit locally without routing overhead.

### Expected Outcome

- Both architectures pass audit and build-from-source before the tap is updated.
- The `brew audit` command accumulates orchestrated run credit toward bypass promotion.
- After promotion, audit runs execute locally (Tier 3), cutting latency for routine releases.

### Failure Mode

- `build-arm64` session has crashed (socket exists but process is gone). Hub gets a connection error, failover kicks in, routes to `build-x86_64` instead. The deploy agent receives routing metadata showing `candidates: 2` but `routed_to: build-x86_64` -- it detects the missing architecture and alerts the operator.
- If `brew audit` fails on one host (formula syntax error), the orchestrated run is recorded as a failure, resetting the promotion candidate's `fail_count` and preventing premature bypass promotion for a flaky command.

### What We Learn

- **Failover correctness:** Does the hub correctly skip a dead session and try the next candidate without losing the request?
- **Composite command tracking:** Can the bypass registry differentiate `shell.exec` calls by their payload, or does it lump all `shell.exec` calls together? (Current impl uses `method` as the key -- this scenario reveals whether that granularity is sufficient for infra operations.)
- **Architecture coverage gap detection:** The `routed_to` metadata lets the caller detect when failover masked a capacity problem.

### Test Approach

1. Start a hub and two sessions: `termlink register --tag build-host --capability brew --name build-arm64` and `termlink register --tag build-host --capability brew --name build-x86_64`.
2. Both sessions handle `shell.exec` by returning `{ exit_code: 0 }`.
3. Send `orchestrator.route` with `{ selector: { tags: ["build-host"], capabilities: ["brew"] }, method: "shell.exec", params: { command: "brew audit" } }`.
4. Assert response has `routed_to` with one of the two sessions and `candidates: 2`.
5. Kill the first session's process (leave socket as stale file). Re-send the same route request. Assert failover occurs: `routed_to` is the surviving session.
6. Repeat 5 successful runs. Assert the 5th run's response triggers promotion (check `orchestrator.bypass_status`).
7. Send a 6th run. Assert response contains `{ bypassed: true }`.

---

## Scenario 2: Pre-Deploy Health Gate via Specialist Discovery

### Trigger

Before deploying a new release to the staging server, the deploy agent must verify the system is healthy. It needs to check: (a) all integration tests pass (test agent), (b) no critical audit findings (audit agent), and (c) the staging server is reachable (infra session on the remote host). These checks run as a gate -- any failure blocks deployment.

### Flow

1. Deploy agent sends `orchestrator.route` with selector `{ roles: ["test-runner"] }`, method `test.run`, params `{ suite: "integration", timeout_secs: 30 }`.
2. Hub discovers a test-runner session, forwards the request. Test runner executes the suite and returns `{ passed: 42, failed: 0 }`.
3. Deploy agent sends `orchestrator.route` with selector `{ roles: ["auditor"] }`, method `audit.run`, params `{ scope: "pre-deploy" }`.
4. Hub discovers the audit session. Audit agent runs `fw audit` and returns `{ status: "pass", warnings: 1, failures: 0 }`.
5. Deploy agent sends `orchestrator.route` with selector `{ tags: ["staging"], capabilities: ["ssh"] }`, method `health.check`, params `{ host: "staging.internal" }`.
6. Hub discovers the remote infra session registered on the staging box. It responds with `{ reachable: true, disk_pct: 72, load: 0.4 }`.
7. All three gates pass. Deploy agent proceeds with `rsync` + service restart via a final `orchestrator.route` to the staging session.
8. Each of these four distinct methods accumulates its own orchestrated run count. `health.check` (called every deploy) reaches bypass threshold first. On subsequent deploys, the hub short-circuits the health check with `{ bypassed: true }`, and the deploy agent runs the check locally instead of routing.

### Expected Outcome

- Deploy is gated on three independent specialist checks, each discovered dynamically.
- No hardcoded session addresses -- the deploy agent only knows roles and capabilities, not socket paths.
- Frequently-passing checks earn bypass, reducing latency for routine deploys.

### Failure Mode

- The test-runner session returns `{ passed: 40, failed: 2 }`. Deploy agent receives this via the `result` field, detects failures, and aborts deployment. The orchestrated run for `test.run` is recorded as successful at the routing level (the RPC call succeeded), but the deploy agent treats the logical result as a gate failure. This distinction matters: routing success != business logic success.
- No session matches `{ roles: ["auditor"] }` -- hub returns error code `-32001` (SESSION_NOT_FOUND). Deploy agent surfaces "no audit agent available" and blocks deployment rather than skipping the gate.
- The staging health check was previously promoted to bypass. The staging server has since been replaced. The deploy agent runs the health check locally (due to bypass), but the local check fails because it cannot reach the new server. Deploy agent calls `orchestrator.bypass_status`, sees the stale entry, and triggers a de-promotion by recording a failed bypass run.

### What We Learn

- **Routing success vs. business success:** The bypass registry tracks RPC-level outcomes, not domain-level outcomes. This scenario validates that callers must interpret `result` payloads independently of routing metadata.
- **Missing specialist handling:** The deploy agent must handle SESSION_NOT_FOUND gracefully rather than treating it as "check passed."
- **Bypass staleness:** When infrastructure changes (server replacement), bypassed commands can become stale. The de-promotion mechanism provides self-healing, but only if the caller reports failures back.

### Test Approach

1. Start hub + three sessions: one with `role: test-runner` handling `test.run`, one with `role: auditor` handling `audit.run`, one with `tags: [staging], capability: ssh` handling `health.check`.
2. Send three `orchestrator.route` calls in sequence. Assert all return successfully with correct `routed_to` metadata.
3. Modify the test-runner to return `{ failed: 2 }`. Re-send. Assert the RPC still succeeds (200-level) but the result payload contains the failure data.
4. Kill the auditor session. Send `orchestrator.route` for `audit.run`. Assert error response with code `-32001`.
5. Run `health.check` 5 times successfully. Assert bypass promotion via `orchestrator.bypass_status`.
6. On the 6th call, assert `{ bypassed: true }` is returned. Simulate a failed local execution. Call the hub to record a bypass failure. Assert de-promotion via `orchestrator.bypass_status`.

---

## Scenario 3: Remote Session Lifecycle Management During Server Provisioning

### Trigger

A new development VM has been provisioned. The deploy agent must: (a) start a TermLink session on the remote host, (b) register it with the local hub so other agents can route to it, (c) verify the remote session is functioning, and (d) tag it for discovery by role-specific agents.

### Flow

1. Deploy agent SSHs to the remote host and starts a TermLink session: `ssh dev-vm "termlink register --name dev-vm-001 --tag remote --tag dev --role build-host --capability cargo --capability brew --listen-tcp 0.0.0.0:9100"`.
2. Deploy agent registers the remote session with the local hub: `termlink hub register-remote --host dev-vm.internal --port 9100 --name dev-vm-001 --tags remote,dev --roles build-host --capabilities cargo,brew`.
3. Deploy agent verifies the remote session is routable by sending `orchestrator.route` with selector `{ name: "dev-vm-001" }`, method `session.info`, params `{}`.
4. Hub checks the bypass registry for `session.info` -- not yet promoted. Hub discovers `dev-vm-001` (matched via the remote store), connects over TCP, forwards `session.info`.
5. Remote session responds with its identity, uptime, and capabilities. Hub relays this back with `routed_to: dev-vm-001`.
6. Deploy agent confirms the response matches expected capabilities. The VM is now part of the mesh.
7. A code-review agent later sends `orchestrator.route` with selector `{ capabilities: ["cargo"] }`, method `cargo.test`, params `{ crate: "termlink-hub" }`. Hub discovers `dev-vm-001` among candidates and routes accordingly.
8. Over time, `session.info` (used by all agents for health probes) accumulates 5+ successful orchestrated runs and gets promoted to bypass. Subsequent health probes return `{ bypassed: true }`, and agents probe directly via `termlink ping` instead.

### Expected Outcome

- A freshly provisioned VM becomes discoverable and routable within seconds of SSH + register-remote.
- Other agents can find and use the VM without knowing its address -- just its capabilities.
- Health probe overhead decreases as `session.info` earns bypass status.

### Failure Mode

- The remote session is started but the `register-remote` call to the local hub fails (hub is restarting). The deploy agent retries registration 3 times with backoff. If all fail, it logs the failure and leaves the remote session running but unregistered -- the VM works standalone but is not part of the mesh.
- The TCP connection to `dev-vm-001:9100` is blocked by a firewall rule. `orchestrator.route` times out on the remote candidate and falls back to local candidates (if any match). Since no local session has the `cargo` capability, the hub returns SESSION_NOT_FOUND. The deploy agent diagnoses: "Remote registered but unreachable -- check firewall."
- The VM is decommissioned but `register-remote` is not cleaned up. Stale remote entries cause repeated timeout failures during routing. The hub logs these as failover events. A cleanup agent (or the deploy agent itself) periodically calls `orchestrator.route` with a health check to each known remote and removes entries that fail 3 consecutive times.

### What We Learn

- **Remote session integration path:** This validates the full lifecycle -- provision, register, verify, use, decommission -- through the orchestrator.
- **Remote vs. local candidate ordering:** The current implementation tries local candidates first, then remote. This scenario reveals whether that ordering is optimal for infra operations where the target is explicitly remote.
- **Stale remote cleanup:** The hub's remote store has no built-in TTL or heartbeat-based eviction. This scenario exposes the need for either: (a) periodic health sweeps by the deploy agent, or (b) hub-side heartbeat monitoring for remote entries.
- **Bypass for infrastructure probes:** High-frequency operations like health checks naturally earn bypass, which is the correct behavior -- but the deploy agent must still have a fallback path when bypass is active (direct probe instead of routing).

### Test Approach

1. Start a local hub. Start a second TermLink session listening on TCP (simulating the remote VM): `termlink register --name dev-vm-001 --tag remote --capability cargo --listen-tcp 127.0.0.1:9100`.
2. Register it as a remote session: `termlink hub register-remote --host 127.0.0.1 --port 9100 --name dev-vm-001 --tags remote --capabilities cargo`.
3. Send `orchestrator.route` with `{ selector: { name: "dev-vm-001" }, method: "session.info" }`. Assert success with `routed_to.display_name == "dev-vm-001"`.
4. Send `orchestrator.route` with `{ selector: { capabilities: ["cargo"] }, method: "cargo.test" }`. Assert the remote session is discovered as a candidate.
5. Kill the remote session process. Send the same route request. Assert timeout/connection error and appropriate failover behavior (SESSION_NOT_FOUND if no other candidates).
6. Verify stale-remote detection: send 3 consecutive route requests targeting the dead remote. Assert all fail. Confirm the hub logs show timeout entries for the remote candidate.
7. Run `session.info` route 5 times while the remote is alive. Assert bypass promotion. On the 6th call, assert `{ bypassed: true }`.
