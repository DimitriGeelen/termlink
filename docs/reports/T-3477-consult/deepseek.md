## 1. Verdict

**Containerise the workers, but treat containers as reproducible execution envelopes, not as the security boundary — and do not use them as a substitute for fixing governance state.**

Failures 1–4 are mostly not isolation failures: 1–2 are path/name derivation bugs, 3 needs the central allocator, and 4 needs a complete workspace plus verified guards. A container can mask these, but only fixed mount paths (`/workspace/<stable-id>`), a full checkout including untracked files, and allocator-issued task IDs actually fix them.

Containers are right here because heterogeneous harnesses, toolchains, MCP servers and weekly CLI releases make host-level environments drift. They give a versioned artifact, a smaller blast radius, and a clean place to inject role config. Bubblewrap is excellent for cheap, per-command filesystem/network confinement and should remain a complement, but it packages nothing and inherits the host toolchain. Firecracker/Kata or VMs are for untrusted code, kernel-level isolation, or GPU/kernel differences; they cost boot time, image size and disk. “No isolation” is acceptable only for fully trusted code, and still requires the governance fixes. The current `--network host` plus full hub secret combination is worse than the shared-host status quo.

## 2. Recommended architecture

Use three immutable layers, plus a runtime governance bundle:

- **Base:** Ubuntu 24.04-derived, git/curl/python/Node as needed, and one pinned harness (Claude Code, Codex CLI, opencode). Derive from `aef-greenfield-testbed`; do not extend it. No credentials in any layer.
- **Project:** language toolchain and lockfile dependencies for one project; optionally warm package caches, never `target/` or secrets.
- **Role:** usually not an image at all. If a role needs different binaries, add a thin role layer; otherwise compose the role at dispatch time.

**Bake:** OS, harness, compilers, pinned MCP binaries, dependency caches. **Inject at start:** source at an exact commit, generated `CLAUDE.md`, hooks, skills, slash commands, MCP config, model choice, credentials. Credentials come from the secret store into a tmpfs/mode-600 file for one run, then are revoked and deleted.

**Compose `CLAUDE.md`, do not copy it.** Split the project file into tagged sections or a manifest; a generator emits the role-specific file. Commit the generated file plus its hash and fail CI if regeneration differs. Hand-maintained role copies will drift.

**Versioning:** tags are labels; dispatch records must use **image digests**. Pin harness version and checksum, rebuild weekly with a smoke test, and record a manifest: image digest, config-bundle commit, `CLAUDE.md` hash, harness/model, task ID and allocator epoch. The orchestrator selects from a capability contract (project, toolchain, harness, role, arch) against a pinned policy map — never `latest`. Roll back by digest.

**Lifecycle:** dispatch task/lease → start container with fixed workspace path → clone exact commit → inject config → agent renews lease/heartbeats → collect diff/artifacts → revoke token → destroy. On failure, freeze the container/volume and record its digest for forensics. Use per-worker `target/` quotas, shared registry/sccache/cargo caches, and place work on hosts with warm images. Connect through a dedicated bridge network with a scoped short-lived hub token and egress restricted to the hub and approved package endpoints — not `--network host`.

**Prior art:** devcontainers get declarative toolchains right but are weak on orchestration and post-create drift. SWE-bench harnesses get per-repo pinned images and reproducibility right, but are evaluation-oriented rather than interactive. OpenHands has useful sandbox lifecycle ideas but its own runtime assumptions. Codex cloud/Copilot coding agent show ephemeral per-task environments and egress control, but are closed and model-specific. Nix is excellent for reproducible builds, not runtime isolation. CI runners do ephemeral jobs and caching well but lack agent leases and interactive state. Copy the pinning and lifecycle; avoid golden images that drift and runtime “fixups.”

## 3. Top five risks

1. **Credential and network exposure.** A full hub secret or long-lived subscription token inside an autonomous agent is unacceptable. Build scoped, short-lived per-worker tokens first; use per-worker networks, an egress proxy/allowlist, no Docker socket, no host mounts beyond the workspace, no host network. Never rely on grepping logs to prove a token did not leak.
2. **False sense of security.** Containers share the host kernel. Use rootless/userns, read-only root, `cap-drop`, `no-new-privileges`, seccomp, and no host filesystem. For hostile or third-party code, use a microVM/VM.
3. **Disk exhaustion.** 220 GB is not much with GB-scale `target/` directories, image layers and build caches. Cap concurrency, enforce per-worker quotas, keep `target/` out of images, prune aggressively, and monitor the 88% level.
4. **Governance failing open.** A missing untracked guard silently passing is worse than a broken worker. Startup must verify every hook/gate by hash and fail closed; task IDs must come from the allocator; names must come from explicit IDs, not checkout directory names.
5. **Reproducibility drift.** Weekly CLI updates and mutable tags defeat the premise. Pin digests and versions, forbid unpinned dispatch, and record the config bundle — image versioning alone is insufficient.

## 4. What you are missing or getting wrong

- **Containers will not fix incidents 1–4.** Fix the path/name derivation, clone fully, install and verify guards, and use the central allocator independently.
- **The baseline token model does not scale.** One long-lived subscription identity shared across workers creates attribution, rate-limit and revocation problems. Prefer per-worker scoped credentials or API identities; the token-in-env-file dance is a pilot technique, not an architecture.
- **A firewall rule to the hub port is necessary but not sufficient.** All containers on a bridge may reach each other; use dedicated networks and a scoped auth proxy.
- **Role config is the real versioned artifact.** An image digest without the config-bundle commit and generated `CLAUDE.md` hash is not reproducible.
- **Commits and pushes need policy,** not just credentials: per-worker branch namespaces (`worker/<task-id>/...`), short-lived repo tokens, no direct main, human merge approval.
- **Operator authority needs an explicit approval gate and kill switch,** not just audit logs.

## 5. Smallest first step

One week, one non-critical project, one role, one harness, two parallel workers — plus a plain-clone-plus-allocator control.

Use the existing baseline as the base, inject config at runtime, fixed workspace path, scoped hub token if available (otherwise a throwaway hub only for packaging tests), and a bridge rule or egress proxy to the hub port. Run two small bugfix tasks with allocator-issued IDs.

**Go if:** both workers get distinct IDs and branches; all guards are present and fail closed; no host path/name leakage; no host UIs or full hub secret reachable; no secret in image/logs; image digest and config hash recorded; runtime stays near baseline; disk is stable. **Stop if:** the design only works with host networking or the full hub secret, gates still fail open, or containers show no measurable benefit over the control.