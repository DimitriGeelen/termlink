# Should we containerise the AI-agent workers?

## 1. Verdict

**Yes, use plain Docker containers, one per work unit and thrown away afterwards.** Don't treat the container as the security or governance boundary; it isn't one.

Containers are the right tool for **reproducible, heterogeneous environments** (harness × version × toolchain) and for **blast-radius hygiene**: no shared cwd, no shared checkout, a canonical path, and nothing leaking into host state. Bubblewrap is fast but doesn't give you an image or versioning story. MicroVMs and VMs only pay off if you run untrusted repos or have multiple tenants. Here you have one operator and your own code, and the real risks are credentials and network reach, which a VM doesn't solve either.

Containers do not fix your incidents on their own:
- **#3** is fixed by the central allocator.
- **#2 and #4** are framework bugs: names come from the directory name, and guards fail open when a script is missing. Fix them in AEF regardless. A container only hides them.
- **#1** is the only one a container genuinely removes.

So do both: fix the governance-state bugs, and containerise for reproducibility.

## 2. Recommended architecture

**Images: two dimensions only, never role.**
- `base` (pinned Ubuntu digest, git, sccache, AEF framework scripts baked in at a fixed path such as `/opt/aef`) → `env-<project>` (toolchain, for example the Rust version from `rust-toolchain.toml`) → `harness-<name>-<ver>`.
- Derive from `aef-greenfield-testbed@sha256:…` as its owner asked.
- If you also make role a layer, harness × project × role multiplies into dozens of images. **Role is configuration, not an image.**

**Inject at start through a versioned "role bundle"**, stored in the project repo and content-hashed:
- the CLAUDE.md fragments to include;
- skills, slash commands and MCP config;
- model id and harness flags;
- tool allowlist.

**Composing CLAUDE.md:**
- Split the 2,500-line CLAUDE.md into tagged fragment files. They are the single source of truth.
- A `core` fragment is mandatory for every role.
- A renderer emits `CLAUDE.md`, `AGENTS.md` (Codex/opencode) and so on per harness at dispatch time. Record the hash of what was rendered.
- Don't keep committed copies, and therefore nothing can drift.

**Source code:** clone at start from a host-local bare mirror (`--reference`, read-only) into a per-container volume. Never bind-mount a host checkout.

**Secrets:** inject at run time only, using the pattern you already have in the testbed (Infisical → env file → delete):
- model token;
- scoped hub token;
- a git credential that can push only `refs/heads/agent/<task-id>/*`.

**Versioning:**
- Tags are for humans; dispatch pins by **digest** through a lockfile in the orchestrator repo (`role → image digest + bundle hash`).
- Turn off the harness auto-updater (for Claude Code, `DISABLE_AUTOUPDATER=1`).
- A weekly rebuild produces candidate digests. Promote them after a canary set of tasks passes. Rollback means reverting one lockfile line.
- Every result records: image digest, bundle hash, rendered-CLAUDE.md hash, harness version, model id and base commit.
- Run a local `registry:2` so other hosts pull the same digests.

## 3. Top 5 risks and how to contain them

1. **The hub credential turns into a container escape.** TermLink *controls terminal sessions* (exec/inject/spawn/remote_exec). A worker holding a broad hub token can type into the operator's shell on the host.
   - The scoped token must be enforced by the server, not just named "scoped".
   - Allowlist only: register, post/read own topics, claim/renew/release own claims.
   - TTL equal to the lease.
   - No session-control methods at all.
   - Until that exists, use a throwaway hub per spike.

2. **Client-side gates are theatre.** An agent running with bypass permissions can run `git commit --no-verify`. Also, Codex and opencode don't run Claude Code hooks at all.
   - Make governance harness-neutral and enforce it **outside**: a pre-receive hook or branch protection on the remote, with merges by the orchestrator or a human only.
   - Inside the container, the entrypoint runs a self-test: each guard is fed a known-bad input. If any guard passes it, or any guard is missing, the container refuses to start.

3. **Disk exhaustion takes the host down.** The hub's SQLite and topic logs are on the same disk, at 88 %.
   - Move Docker's data root to its own volume, or at least set a hard admission rule: no dispatch below about 15 % free.
   - Use a shared sccache, a shared read-only cargo registry cache, and cap Rust workers at about 3.
   - Keep forensics as exported artefacts (transcript, diff, logs). Don't keep stopped containers for days.

4. **Network reach.**
   - Use a user-defined bridge with a fixed subnet, plus a ufw rule from that subnet to the hub port only.
   - Check the **FORWARD/DOCKER-USER chain** too: ufw INPUT rules don't stop a container reaching *other LAN hosts* and their UIs or hubs.
   - Later, add an egress allowlist (model APIs, git remote, registries) to limit token exfiltration through prompt injection.

5. **The configuration matrix sprawls and drifts.**
   - Start with at most 3 images.
   - Roles stay as bundles, never images.
   - The lockfile is the only way to dispatch.
   - CI fails the build if a bundle references a fragment that doesn't exist.

## 4. What you're missing or getting wrong

- **Integration is the real bottleneck, not isolation.** Parallel workers produce conflicting branches. Decide who rebases and merges, and how the orchestrator avoids handing out overlapping units (file or crate-level claims). Incident #3 was a *coordination* failure.
- **Subscription tokens across parallel workers.** Check rate limits and the terms of use for many concurrent automated sessions on one subscription identity. Plan for metered API keys per worker class.
- **Cold Rust builds dominate.** Container start (~1 s) is irrelevant. A cold `target/` costs minutes and gigabytes, which is why sccache is mandatory.
- **Operator visibility.** Run the harness inside a TermLink-registered PTY *inside* the container. The operator can then attach, watch and kill it with the tools they already use, and `docker stop --filter label=task=…` is the hard kill switch.
- **Cross-host placement.** Hubs don't federate, so a worker must talk to the hub that owns its work unit. Stay on a single host until that is designed.
- **Subsetting the 2,500-line CLAUDE.md is a policy change.** A reviewer that never sees a rule cannot enforce it. Define what is in `core` explicitly, and have the operator approve it.

## 5. Prior art

| System | Copy | Avoid |
|---|---|---|
| SWE-bench harness | Base → env → instance layering keyed by dependency hash: exactly your shape | Its image-count disk blowup; prune aggressively |
| Codex cloud | Setup phase with network, agent phase with network restricted; cached environment | — |
| Copilot coding agent | Ephemeral runner, egress firewall, pushes only to `copilot/*`, human merges, CI needs approval | — |
| devcontainers | Use `devcontainer.json` to *define* the `env-<project>` layer so humans and agents share one environment | Its lifecycle and secrets model |
| OpenHands | Deterministic runtime-image tags | Mounting the Docker socket or Docker-in-Docker, ever |
| Nix | Excellent pinning | Too costly to adopt now; revisit for toolchains |
| CI runners | Labels for placement | Building your own scheduler before checking whether a self-hosted runner with labels already covers it |

## 6. Smallest first step (one week)

**Setup:**
- One image: `FROM testbed@digest` + Rust toolchain + AEF baked into `/opt/aef` + pinned Claude Code.
- One project (TermLink) and one role (worker).
- A throwaway hub on a dedicated bridge network.
- sccache enabled.

**Run:** 5 real small backlog tasks, up to 3 at a time.

**Also test on purpose:**
- delete one guard script (the container must refuse to start);
- `--no-verify` push to main (the remote must reject it);
- `cd /` and back;
- check the generated system-level names;
- reach a LAN web UI from inside (must fail).

**Go if all of these hold:**
- at least 4 of 5 branches are mergeable with gates green;
- every injected failure is blocked;
- peak disk is 5 GB or less per worker with sccache, and 3 concurrent workers leave at least 15 % free;
- overhead is under 2 minutes per task;
- the result record carries full provenance (digest, bundle hash, CLAUDE.md hash, model, base commit).

**Stop, or change course, if any of these happen:**
- **Disk can't hold 3 workers:** fix the disk first.
- **Gates can't fail closed:** fix AEF first.
- **Merging and conflict resolution take more operator time than the work saved:** the bottleneck is orchestration, not isolation. Fix the AEF bugs, use bubblewrap with plain clones, and put containers on hold.
