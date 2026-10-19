## 1. Verdict

**Containerise ordinary workers, while fixing governance independently.** Your measured startup cost is negligible beside an 8–19-minute run, and images address dependency completeness and environment distribution. They do not fix checkout-derived identity, session re-rooting, duplicate work, or bypassable gates. Use hardened containers for trusted projects with bounded host exposure; use dedicated VMs for hostile repositories, privileged operations, or workloads whose kernel compromise would be unacceptable. Bubblewrap suits lightweight local isolation, but requires you to construct the policy yourself. MicroVMs become worthwhile when stronger isolation is routine: [Kata](https://katacontainers.io/) integrates with container infrastructure; [Firecracker](https://github.com/firecracker-microvm/firecracker/blob/main/docs/jailer.md) requires more platform engineering. “No isolation” is inappropriate for autonomous shell workers. [Bubblewrap’s own documentation](https://github.com/containers/bubblewrap) explicitly distinguishes its mechanism from a complete sandbox policy.

## 2. Recommended architecture

**Use a small image family plus immutable configuration bundles, not a project × role × model image matrix.**

- **Images:** pinned OS → toolchain family → harness/runtime adapter. Add project-specific dependencies only when substantial. Derive the Claude spike from `aef-greenfield-testbed@sha256:…`, leaving that baseline unchanged.
- **Bake:** exact harness versions, compiler/tool versions, MCP executables, AEF gate binaries and bootstrap validator. Disable self-updates.
- **Inject:** source at a specified commit, task/attempt identity, model settings, immutable policy bundle, skills, commands, MCP configuration and narrowly scoped runtime credentials. Never mount the operator’s home or Docker socket.

Split the project rules into canonical modules: mandatory governance core, project invariants, component rules and role guidance. Deterministically assemble role documents with source hashes; CI rejects stale generated files. Generate harness-native instruction/configuration formats—do not assume every harness implements `CLAUDE.md`, hooks or slash commands equivalently. Role selection must never remove mandatory controls.

Give every checkout a stable container path, such as `/workspace/project`. Pass explicit `PROJECT_ID`, `TASK_ID` and `ATTEMPT_ID`; directory names must not determine operational identity.

Dispatch an immutable manifest containing image/platform digest, source SHA, policy/config hashes, harness/model identifiers, resource limits and permitted capabilities. Human-readable tags aid discovery; dispatch uses digests. Record the manifest with results. Promote tested versions through a compatibility catalog; rollback selects the previous manifest. This reproduces inputs, not model outputs.

A trusted host supervisor owns lifecycle and sidecar authority:

`allocate → stage → validate → start → renew lease → collect → independently verify → publish → destroy`

Use lease generations as fencing tokens: expired attempts cannot publish or close tasks. Keep bounded forensic artifacts, not indefinitely running containers. Since hubs do not federate, the orchestrator needs explicit per-host adapters and authoritative assignment tracking; placement considers toolchain/architecture, disk headroom, cache locality and provider capacity.

## 3. Top 5 risks and how to contain them

1. **Host compromise.** Non-root workers, preferably [rootless Docker](https://docs.docker.com/engine/security/rootless/), dropped capabilities, `no-new-privileges`, seccomp/AppArmor, read-only root filesystem and explicit writable volumes. Enforce CPU, memory and PID limits. Containers still share the host kernel; use VMs when that residual risk is unacceptable.

2. **Hub and network authority.** Until scoped authentication exists, allow no production hub access. Use a throwaway hub or a trusted broker exposing only worker-specific operations. Tokens need audience, expiry, task/attempt binding and restrictions on methods, topics and terminal sessions. Restrict bridge access to the required endpoint; independently block admin methods. Test host INPUT, forwarded traffic, IPv6 and peer-container access: [Docker/UFW interactions](https://docs.docker.com/engine/network/packet-filtering-firewalls/) make a single firewall rule insufficient evidence. Apply egress policy to MCP processes too.

3. **False governance assurance.** Startup must reject missing scripts, hooks, configuration or policy hashes. Exercise negative gate tests. A worker can bypass Git hooks or alter tests; therefore a separate trusted verifier must enforce acceptance and task closure. Allocate IDs centrally, and separately detect overlapping assignments.

4. **Credential leakage and unauthorized publishing.** Deleting an env file and grepping logs cannot establish secrecy: the process can read its environment. Prefer credential brokers; otherwise use dedicated, restricted identities and short lifetimes where supported. Commit inside under `worker/<task>/<attempt>`. Push through a broker enforcing that namespace, because repository credentials may not enforce branch scope. Human-controlled merge/release permissions remain outside workers.

5. **Disk exhaustion and cache contamination.** Set storage quotas, reserved free-space thresholds and admission control before increasing concurrency. Share bounded compiler caches with project/trust namespaces; avoid shared writable `target/` trees. Bound image retention, logs and failed-workspace TTLs. Run authoritative verification with trusted or clean caches.

## 4. What we are missing or getting wrong

Your four incidents are principally identity, state and packaging failures. Containers provide a place to enforce corrected contracts; they do not supply those contracts. In particular, a unique task ID does not prevent three agents receiving the same objective.

Also missing: a threat model, provider concurrency/budget limits, harness capability conformance tests, and evidence about the existing VM test pad.

Copy selected prior art:

- **Devcontainers:** declarative developer setup; do not treat configuration as a security boundary.
- **[OpenHands](https://github.com/OpenHands/docs/blob/main/openhands/usage/architecture/runtime.mdx):** separate runtime execution from orchestration; independently harden deployment defaults.
- **[SWE-bench](https://github.com/SWE-bench/SWE-bench/blob/main/docs/guides/docker_setup.md):** reproducible evaluation environments; avoid unbounded per-instance image retention.
- **[Codex cloud](https://learn.chatgpt.com/docs/environments/cloud-environment), Copilot and ephemeral CI runners:** explicit environments and external acceptance. Check enforcement coverage: [Copilot documents firewall exclusions for MCP and setup processes](https://docs.github.com/en/copilot/how-tos/copilot-on-github/customize-copilot/customize-the-firewall).
- **[Nix](https://nixos.org/guides/how-nix-works/):** dependency closure and rollback; defer migration unless image builds prove insufficient.

## 5. Smallest first step

Run a one-week, two-worker pilot: one project, Claude and Codex harnesses, implementer/reviewer bundles, throwaway hub, no production publishing credentials.

Replay all four incidents. Deliberately remove a guard, bypass a hook, expire a lease, interrupt a worker, probe forbidden services and exhaust its disk quota. Measure cold/warm completion time, peak storage, cleanup and manifest completeness across ten work units.

**Go:** every negative test blocks unauthorized acceptance; expired attempts cannot publish; required artifacts survive teardown; ten runs stay within declared storage budgets with acceptable operator effort.

**Stop rollout:** any production authority leaks, gates fail open, stale workers publish, or disk admission fails. Repair those contracts first; escalate to VM isolation if host containment remains inadequate.
