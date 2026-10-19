PERPLEXITY-ANSWER termlink-t3477-containers

1. Verdict

Containerise ordinary workers, one disposable container per work unit, while fixing governance-state bugs independently. Your measured startup overhead is immaterial beside multi-minute runs; reproducible provisioning and bounded filesystem access justify containers. They do not fix task collisions, duplicate assignments, pathname-derived identities, or bypassable gates. Use bubblewrap for lightweight, trusted local sessions where you own and test the complete sandbox policy—it explicitly supplies mechanisms, not a ready-made security policy. Use a conventional VM initially for hostile repositories, privileged integration tests, or workloads requiring stronger host separation; consider Firecracker/Kata when enough such workloads justify their operational complexity. Do not allow unrestricted agents on the shared host merely because governance improves.

2. Recommended architecture
Images and injected configuration

Avoid an OS × harness × project × role × model image matrix.

Bake: pinned OS/toolchain dependencies, exact harness version, immutable AEF enforcement tools, entrypoint and health checks.

Layer: common base → harness/toolchain variants → optional heavyweight project dependencies. Preserve aef-greenfield-testbed; derive a pinned child for the spike.

Inject: source at an exact commit, assignment, role bundle, model/provider settings, approved MCP configuration, and narrowly scoped credentials. Never mount the operator’s home, Docker socket, or main checkout.

Standardise: /workspace/project regardless of branch; pass explicit project identity rather than deriving it from a directory basename.

Make project guidance structured source: mandatory governance core plus named component/role fragments. Generate CLAUDE.md deterministically, with fragment provenance and a content hash; never maintain hand-edited role copies or let an LLM silently summarise mandatory rules. Generate equivalent harness-native files through adapters. Skills and slash commands also require adapters; MCP configuration grants executable/network capabilities, not merely context.

Versioning

Dispatch an immutable execution manifest:

image digest + platform + source SHA + AEF/bundle hashes + harness version + model identifier + task/attempt + policy + resource limits

Use readable release tags for humans, digests for execution. Disable harness self-updates; qualify new versions with smoke/security tests, then promote. Rollback means selecting the previous manifest. Record actual runtime versions and resolved image identity in the result.

The orchestrator selects from an approved capability catalogue—language/toolchain, harness, role, architecture—not worker-supplied image names. A pinned environment is reproducible; a hosted model’s behaviour may not be.

3. Top 5 risks and how to contain them

Host compromise. Start non-root, preferably rootless Docker; drop capabilities, retain seccomp/AppArmor, enable no-new-privileges, use a read-only root filesystem with explicit writable workspace/tmp, and impose CPU/RAM/PID limits. Rootless reduces daemon/runtime privilege exposure; it is not VM isolation.

Hub authority leakage. Keep the peer sidecar outside the worker, under the trusted supervisor. Give workers a narrow broker endpoint: assignment retrieval, heartbeat and bounded task-topic messaging. The broker retains the full hub secret and independently authorises every operation. Alternatively build genuinely scoped, expiring tokens; expiry alone is insufficient. Never expose generic terminal control or arbitrary JSON-RPC forwarding.

Network escape. Reject host networking. Allow bridge-to-host traffic only to the broker—or scoped hub endpoint—and explicitly deny other host/LAN destinations; separately control internet egress. Test IPv4, IPv6 and sibling-worker access. Your host-service connection follows INPUT; Docker-published container ports involve different forwarding/NAT paths, so do not blindly apply a DOCKER-USER recipe. Docker documents its ufw incompatibility.

Governance bypass. Writable hooks are advisory against shell-capable agents. Package required scripts, verify their manifest, and fail startup on absence. Independently validate commits and acceptance evidence outside the worker. Workers may commit locally; a trusted publisher pushes only approved workers/<task>/<attempt> branches using credentials never given to workers. Only the authority service closes tasks; protected-branch merge/release remains operator-controlled.

Disk exhaustion and contamination. Set aggregate workspace/cache budgets, admission thresholds and retention TTLs. Share immutable dependency downloads where practical; keep writable target/ directories private. Consider a bounded compiler cache with trust-separated writers rather than one concurrently writable build tree. Preserve failure logs and patches, not indefinitely running containers.

4. What we are missing or getting wrong

The core abstraction is an execution contract, not a role image.

Your long-lived subscription token remains readable to processes receiving it. Deleting its env file and grepping logs do not prove non-exfiltration. Limit account blast radius and investigate supported brokered authentication.

A central ID allocator prevents ID collisions, not redundant work. Assignments need explicit scope, ownership and overlap checks.

Lease expiry cannot stop a partitioned worker. Use monotonic fencing epochs: stale attempts cannot publish, mutate authoritative task state, or complete after reassignment.

Non-federating hubs require an explicit cross-host coordinator/relay. Local worker discovery does not provide global placement or ownership.

Containerising system-level tests can make tests meaningless. Route service-installation and privileged integration work to disposable VMs.

Prior art: copy devcontainers’ declarative environment specification, not convenience mounts; OpenHands’ separate runtime boundary, not host-network escape hatches; SWE-bench’s dependency layering, not per-role image proliferation. Copy Codex’s restricted-network posture and ephemeral CI-runner lifecycle. Nix can improve dependency pinning later, but is not an isolation boundary. Copilot similarly distinguishes cloud isolation from configurable local filesystem/network sandboxing.

5. Smallest first step

One week: one host, one project, two concurrent workers, two role bundles; add a second harness only after the first works. Use the trusted broker or a throwaway hub, never the production secret.

Supervisor lifecycle: claim → materialise pinned inputs → launch → renew lease independently → collect patch/logs → independently verify → publish if authorised → revoke access → delete workspace. Placement initially checks architecture, toolchain, free disk and concurrency; no scheduler platform needed.

Go if all four historical failures are reproduced then prevented, missing guards fail closed, expired attempts cannot publish, prohibited endpoints/secrets remain inaccessible, and repeated runs stay within declared disk/I/O budgets.

Stop rollout if success requires host networking, production hub credentials, writable enforcement authority, or unbounded build storage. That rejects the design—not necessarily containers.

Sources:
https://github.com/containers/bubblewrap/blob/main/README.md
https://docs.docker.com/engine/security/rootless/
https://docs.docker.com/engine/network/packet-filtering-firewalls/
https://docs.openhands.dev/openhands/usage/sandboxes/docker
https://www.swebench.com/SWE-bench/guides/docker_setup/
https://openai.com/fr-FR/index/introducing-upgrades-to-codex/
https://docs.github.com/en/copilot/concepts/security-governance-and-network-settings/about-cloud-and-local-sandboxes
https://github.com/OpenHands/benchmarks
https://www.bigiron.cc/guides/docker-published-ports-bypass-ufw-the-docker-user-chain-fix
https://github.com/OpenHands/OpenHands/issues/10250
https://deepwiki.com/OpenHands/benchmarks/4.1.4-swe-bench-image-building
https://www.bitdoze.com/docker-bypasses-firewall/
https://www.developersdigest.tech/blog/codex-cloud-reusable-environments
https://deepwiki.com/OpenHands/benchmarks/5.1.1-docker-workspace
https://www.virtua.cloud/learn/en/tutorials/docker-ufw-firewall-fix-vps
https://www.smfclearinghouse.com/deployment-recipes/openhands-1-0-docker-deploy/
https://codex.danielvaughan.com/2026/05/13/codex-cli-nix-flakes-reproducible-agent-environments-declarative-toolchains/
https://baeseokjae.github.io/posts/openai-codex-cloud-agent-guide-2026/
https://docs.docker.com/engine/install/debian/
https://github.com/containers/bubblewrap
https://github.com/containers/bubblewrap/security
https://github.com/spsforks/containers-bubblewrap/blob/main/README.md
https://github.com/containers/bubblewrap/blob/main/SECURITY.md
https://github.com/containers/bubblewrap/tree/main/.github
https://www.cleveradmin.de/blog/2026/10/ufw-firewall-einrichten/
https://github.com/containers/bubblewrap/issues
https://github.com/chaifeng/ufw-docker
https://blog.silverpc.hu/2025/09/13/sicherheitslucke-im-system-so-konfigurieren-sie-die-ufw-firewall-fur-ihre-docker-container-richtig/
https://www.virtua.cloud/learn/de/tutorials/docker-security-hardening-rootless-seccomp
https://github.com/spsforks/containers-bubblewrap/security

Thread: https://www.perplexity.ai/search/efd34a0f-8be8-4050-8ce3-66d842d16f1f
