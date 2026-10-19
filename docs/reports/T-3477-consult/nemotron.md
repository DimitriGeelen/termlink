# External Design Review: Containerising AI-Agent Workers

---

## 1. Verdict

**Containerise — but with a thin, disciplined layer, not a heavy image-per-role strategy.**

The worktree incidents (1–4) are *environment-leakage* bugs: shared filesystem namespace, shared process namespace, shared credential ambient authority. Containers fix the namespace isolation cleanly; bubblewrap fixes it faster/lighter but lacks the image+registry+versioning toolchain you need for reproducibility across hosts and heterogeneous harnesses. MicroVMs (Firecracker/Kata) and full VMs are unjustified overhead — your agents need shell access and host GPU/PCI passthrough is not a requirement. "Fix governance-state bugs instead" is necessary but insufficient: you still need *reproducible, versioned environments* for the heterogeneous matrix (models × harnesses × projects × roles). Containers give you that with the tooling you already have (Docker 29.9, Infisical secret injection, a baseline image).

**One-paragraph reason:** The failure modes are namespace collisions (cwd, directory-name tooling, task-ID allocator, untracked files) — exactly what container filesystems and PID namespaces isolate. The heterogeneity matrix (7+ dimensions) demands *versioned, composable environments* that `docker pull` + `docker run` delivers today; bubblewrap requires rebuilding that tooling. Disk pressure (88% used) is the only hard constraint — solve it with shared build caches and ephemeral layers, not by abandoning isolation.

---

## 2. Recommended Architecture

### Image Layering (4 layers, not N)

| Layer | Contents | Size | Rebuild cadence |
|-------|----------|------|-----------------|
| **L0: `aef-base`** | Ubuntu 24.04 slim, `git`, `curl`, `python3`, `build-essential`, `bubblewrap` (for nested sandboxes), **no credentials, no CLI harnesses** | ~200 MB | Quarterly (CVE patches) |
| **L1: `aef-harness-<name>`** | Single harness binary + its self-update disabled (pinned version). One image per harness: `claude-code`, `codex`, `opencode`, `gemini-cli`, etc. | +50–150 MB | Weekly (harness releases) |
| **L2: `aef-project-<name>`** | Project source (shallow clone), **compiled artifacts only** (`target/` for Rust, `node_modules` for TS), project-scoped MCP servers (as binaries, not source), project `CLAUDE.md` **template** (see injection below). | +500 MB–2 GB | Per-project release / `main` tag |
| **L3: `aef-role-<project>-<role>`** | **Empty** — exists only as a *tag* pointing to L2 + runtime injection (see below). No Dockerfile. |

**Do not bake per-role images.** The combinatorial explosion (projects × roles × harnesses × model choices) makes L3 images unmaintainable. Instead:

### Runtime Injection (at `docker run`)

| Injected | Mechanism | Source |
|----------|-----------|--------|
| `CLAUDE.md` (role-specific) | `--mount=type=bind,source=/host/generated/claude-md/<role>,target=/workspace/CLAUDE.md,readonly` | Generated at dispatch from project template + role patch (see §2.3) |
| MCP config (`~/.config/claude/mcp.json` or equiv.) | Bind-mount generated file | Orchestrator renders from role spec |
| Slash commands / skills | Bind-mount directory tree | Same |
| Model choice / API keys | `--env-file` from Infisical (mode 600, deleted post-start) | Secret store |
| Task ID + lease token | `--env TASK_ID=... --env LEASE_TOKEN=...` | Central allocator |
| Scoped hub token | `--env HUB_TOKEN=...` (short-lived, per-worker) | New auth service (§4) |
| Build cache (`target/`, `.cache/`) | `--mount=type=cache,target=/workspace/target,from=aef-buildcache:<project>` | BuildKit cache image (shared, read-only base + RW diff) |

### CLAUDE.md Composition Without Drift

```
project/
  CLAUDE.md                 # 2500 lines — source of truth
  roles/
    reviewer.md             # ~200 lines: only review-relevant rules
    hub-worker.md           # ~150 lines: hub crate only
    release-engineer.md     # ~300 lines: +release runbook
  patches/
    reviewer.patch          # git apply --reject safe subset
    hub-worker.patch
    release-engineer.patch
```

At dispatch: `cat CLAUDE.md | apply-patch roles/<role>.patch > /host/generated/claude-md/<role>/CLAUDE.md`. Patches are **subtractive only** (remove sections) — verified by CI on every `CLAUDE.md` change. No additive patches; roles that need *extra* rules get a separate `ROLE_RULES.md` injected alongside and `include`d via a one-line addition in the patch.

### Versioning & Reproducibility

- **Tags:** `aef-harness-claude-code:v1.2.3-<harness-version>`, `aef-project-termlink:v2025.04.17-<git-short-sha>`
- **Digests:** Orchestrator records `image.digest` (sha256) in the work-unit result — immutable reference.
- **Pinning:** Harness images pin exact upstream version (`claude-code@1.2.3`); `Dockerfile` uses `ARG HARNESS_VERSION=1.2.3` + `RUN install-harness.sh $HARNESS_VERSION`. Rebuild on every harness release (automated via Renovate/Dependabot).
- **Rollback:** Previous digest stays in registry; orchestrator can re-dispatch with `--image <old-digest>`.
- **Image selection:** Orchestrator chooses by `(project, harness, role)` → resolves to `(L2 digest, L1 digest, L0 digest)` tuple stored in a small SQLite index. No "latest" tags in production.

---

## 3. Top 5 Risks & Containment

| # | Risk | Containment |
|---|------|-------------|
| **1** | **Disk exhaustion** — `target/` per worker × parallel workers > 220 GB free | • **Shared BuildKit cache image** (`aef-buildcache:<project>`) — base layer has warm `target/`; workers mount as cache (RW diff only).<br>• **Ephemeral containers** — `docker run --rm` + `tmpfs /workspace/target` for scratch builds; only cache image persists.<br>• **Quota** — `docker run --storage-opt size=10G` per worker; orchestrator enforces max concurrent workers per host (start with 4). |
| **2** | **Scoped hub auth not ready** — full hub secret leaks into container | • **Short-term:** Throwaway hub per spike (hub binary + ephemeral SQLite + auto-generated secret) spun up by orchestrator on same host, bound to `127.0.0.1:random`; container gets `--network host` + that port only (ufw allow from bridge subnet to that port).<br>• **Medium-term:** Build scoped token service (JWT, 15-min TTL, `worker:<task-id>` audience, claims: `hub:read`, `hub:write`, `topic:<task-id>:*`). |
| **3** | **Harness CLI drift** — weekly updates break reproducible runs | • **Pin in L1 image** — no auto-update. Renovate PR → CI rebuild → digest pin → orchestrator index updated. Human approve for major versions only. |
| **4** | **Network exposure** — `--network host` leaks 13 local UIs + hub admin | • **Bridge + ufw rule** — `ufw allow from 172.17.0.0/16 to any port 8443 proto tcp` (hub port only).<br>• **Container-side:** `iptables -A OUTPUT -d 127.0.0.0/8 -j DROP` (drop localhost egress) — agent can't reach host UIs. |
| **5** | **Governance bypass** — agent disables gates/hooks inside container | • **Fail-closed gates baked into L0** — `pre-commit`/`pre-push` hooks are *immutable files* in image (COPY, not bind-mounted).<br>• **Sidecar monitor** — separate minimal container (distroless, 5 MB) shares PID namespace (`--pid=container:<worker>`), watches for gate-skipping syscalls (ptrace), kills worker + alerts operator.<br>• **Commit signing** — git config in L0 enforces `commit.gpgsign=true`; GPG key injected via `--env-file` (subkey per role, stored in Infisical). |

---

## 4. What We're Missing / Getting Wrong

1. **No scoped hub auth design yet** — this is the blocker for *any* containerised worker on bridge network. The "throwaway hub per spike" is a valid week-1 prototype but doesn't scale. You need a spec for: token format, TTL, audience, revocation, and how the orchestrator mints them without holding the master secret.

2. **Build cache strategy is underspecified** — "BuildKit cache image" works for Rust `target/` but not for npm/pip/cargo caches that live in `~/.cache`. Need a **cache volume per language ecosystem** (`cargo-cache`, `npm-cache`, `pip-cache`) mounted read-only base + RW overlay, garbage-collected weekly.

3. **Role CLAUDE.md patches are subtractive-only** — what if a role *needs* an extra rule (e.g., "always run `cargo deny check`")? Current design forces it into project `CLAUDE.md` (polluting other roles) or a separate `ROLE_RULES.md` that agents may ignore. Add: **patch type `add-after:<section>`** with CI verification that the added block is syntactically valid markdown and doesn't conflict.

4. **Cross-host placement ignores GPU/accelerator affinity** — local models (llama.cpp, vLLM) need GPU; Claude Code doesn't. Orchestrator needs a **capability matrix** per host (`gpu: true/false`, `arch: amd64/arm64`, `disk-free-gb`) and per image (`requires-gpu: false`, `arch: amd64`). Not in current spec.

5. **Forensics retention policy undefined** — "keep for forensics" vs disk pressure. Need: **auto-expunge after N days unless tagged `forensics:keep`**, with a separate cold-storage bucket (S3/minio) for tagged containers' logs + workspace diff.

6. **Operator authority in container** — how does the human *intervene* mid-run? Current design: `docker exec -it` into running container. But the agent runs as non-root (good); operator needs `sudo` inside container (bad). Solution: **sidecar `operator-console` container** with `nsenter -t <worker-pid> -m -u -n -p` + TTY, audited. Not designed.

---

## 5. Smallest First Step (Week 1)

**Goal:** Prove containerised worker can *replace one worktree worker* on a real task, end-to-end, with governance intact.

### Scope
- **One project:** TermLink hub crate (small, Rust-only, no GPU).
- **One harness:** Claude Code (existing `aef-greenfield-testbed` baseline).
- **One role:** `hub-worker` (subset of `CLAUDE.md`).
- **One host:** .107 (existing Docker, ufw, Infisical).
- **No cross-host, no build cache sharing, no scoped hub auth yet** — use throwaway hub.

### Steps (2 days)
1. **Derive L1 image:** `Dockerfile.harness-claude-code` FROM `aef-greenfield-testbed` + pin Claude Code version (disable auto-update). Build → `aef-harness-claude-code:v1.0.0-claude-1.2.3`.
2. **Derive L2 image:** `Dockerfile.project-termlink` FROM L1 + shallow clone TermLink `main` + `cargo build --release` (populate `target/`). Build → `aef-project-termlink:v2025.04.17-abc123`.
3. **Generate role CLAUDE.md:** Script that applies `roles/hub-worker.patch` to project `CLAUDE.md` → `/host/generated/claude-md/hub-worker/CLAUDE.md`.
4. **Spin throwaway hub:** `termlink hub --ephemeral --port 0 --secret-file /tmp/hub-secret` → prints `HUB_ADDR=127.0.0.1:XXXXX HUB_SECRET=...`.
5. **Run worker:** 
   ```bash
   docker run --rm --network host \
     --mount=type=bind,source=/host/generated/claude-md/hub-worker,target=/workspace/CLAUDE.md,readonly \
     --env-file <(infisical run --env=worker -- claude-setup-token) \
     -e TASK_ID=TEST-001 -e HUB_ADDR=$HUB_ADDR -e HUB_SECRET=$HUB_SECRET \
     aef-project-termlink:v2025.04.17-abc123 \
     /workspace/run-agent.sh
   ```
6. **Verify:** Task completes, gates pass, commit signed, push to `worker/TEST-001` branch, hub receives messages, lease renewed, container exits cleanly.

### Go/Stop Criteria
| Result | Decision |
|--------|----------|
| **Go:** Worker finishes task, all 4 governance gates pass (lint, test, typecheck, accept), commit lands on `worker/TEST-001`, no host leakage (cwd, task IDs, untracked files), disk delta < 2 GB. | Proceed to: scoped hub auth, build cache sharing, second harness (Codex), second project (AEF). |
| **Stop:** Any gate fails *due to containerisation* (missing files, network, permissions, credential leak), or disk delta > 5 GB, or start-to-ready > 60 s. | Revert; investigate bubblewrap + shared clone as lighter alternative. |

---

**Bottom line:** The container path is sound *if* you treat images as **versioned, composable infrastructure** (L0–L2) + **runtime injection** for all variance, not as per-role artefacts. The week-1 spike above tests the critical path with minimal investment.