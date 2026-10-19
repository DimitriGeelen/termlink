## 1. Verdict

Containerise — OCI containers, one image per *harness*, everything role-specific injected at start — but for the right reason. Of your four incidents, only #2 (name derivation) and #4 (missing untracked files) are structurally fixed by per-worker filesystems; #1 is harness discipline and #3 needs the central allocator you already assume. Containers win because they give (a) a complete, named, disposable filesystem per worker, killing the worktree/clone failure class, and (b) a versioned, reproducible environment, which is the actual requirement once workers are heterogeneous. Start cost is noise: 0.9 s against an 8.5–19 min run. Bubblewrap (0.02 s) has no image or versioning story, so it fails your stated goal. MicroVMs/VMs are right when you run code or models you don't trust (third-party MCP servers, foreign weights); they're overkill for your own projects under AEF and cost disk you don't have — keep them as escalation, not baseline.

## 2. Recommended architecture

**Images:** `ubuntu:24.04` → baseline layer (git/curl/python, non-root user). The greenfield testbed stays untouched as the "new user" reference; derive a *sibling* from its pattern, don't extend it. Then **one image per harness** (`worker/claude-code:X`, `worker/codex:Y`, `worker/opencode:Z`), harness version pinned, self-update disabled in-image. Never per-role or per-project images: N×M×P×R is unmaintainable, and model choice is a flag, not a layer.

**Injected at start (never baked):** fresh clone at pinned commit (not a host checkout — that recreates incidents 2/4); role `CLAUDE.md`; skills, MCP config, slash commands; secrets via your proven Infisical → mode-600 env-file → delete pattern; scoped hub token.

**CLAUDE.md composition:** keep the project file canonical and machine-segmented (stable section IDs); generate role variants by inclusion with a checked-in manifest (role → sections). CI asserts every section is included or explicitly excluded per role. Copies drift; generated subsets with a coverage check don't.

**Versioning:** dispatch by **digest**; the work unit carries `image:tag@digest` + CLAUDE.md hash + skills rev + model ID, and the result record captures all of it — that's your provenance and rollback (re-dispatch previous digest). No `latest` anywhere. Weekly CLI churn means scheduled harness-layer rebuilds, not floating tags.

## 3. Top 5 risks and containment

1. **Disk exhaustion** (88% full; `target/` is GBs): shared read-only cargo registry cache; sccache (not a shared `target/` — parallel writers collide); per-worker scratch volumes; retention policy (keep forensic images N days, delete successes); orchestrator refuses placement below a disk headroom floor.
2. **Governance silently absent inside:** hooks must verify framework files exist and abort otherwise (test this deliberately); vendor the framework as a pinned submodule/fetch so every fresh clone has it; hooks reject any task ID not currently leased to *this* worker.
3. **Credential leakage:** no host home, no ssh-agent, no docker socket, non-root, dropped caps. Pushes over HTTPS with short-lived per-worker tokens; branches `worker/<task-id>`; workers push, operator merges. The full hub secret never enters a container — the scoped short-lived token is a hard production prerequisite.
4. **"Reproducible" overclaim:** pinned images reproduce the *environment*, not the run — CLIs self-update and hold state in `$HOME` (disable both, wipe per start), models are nondeterministic. You get attributable runs, not replayable ones.
5. **Network misconfig:** never `--network host` — that hands a bypass-permissions agent 13 admin surfaces. Bridge network + ufw allow bridge-subnet → hub port only; egress elsewhere default-deny except push remote and model API.

## 4. What you're missing or getting wrong

- You implicitly frame containers as the fix for incidents 1–3; they aren't. Ship the allocator and fail-closed hooks regardless, or containers just make the same bugs parallel.
- The scoped hub credential (Fact 5), not the images, is the long pole. Start it now; use a throwaway hub only for the spike.
- 13 admin web UIs on a localhost reachable from containers is a host-hardening bug independent of this decision.
- Hubs don't federate: placement must respect "orchestrator-reachable hosts with pre-pulled images and disk headroom" — design that constraint in now.
- Prior art: copy SWE-bench's pinned-image-per-instance + artifact-out; OpenHands' per-session runtime and network policy; devcontainers' config-as-code lifecycle (not their interactive bias); CI runners' ephemerality and cleanup. Skip Nix for now: real reproducibility, but store overhead on an 88%-full disk plus weekly self-updating CLIs is cost without payoff.

## 5. Smallest first step

One week, one spike: a `worker/claude-code` Dockerfile derived from (not extending) the greenfield baseline; ufw rule bridge → throwaway hub only; existing Infisical env-file pattern; a hand-run dispatch script (no orchestrator) starting 3 containers, each with a fresh clone, a generated implementer/reviewer/tester `CLAUDE.md`, one manually assigned task ID, pushing `worker/<task-id>` branches. Deliberately omit one framework file in one container to prove gates fail closed. Measure collisions, disk per worker, wall time, hub traffic.

**Go** if: 3 parallel workers produce zero ID/name/branch collisions; the fail-closed test aborts a commit; disk per worker ≤ ~5 GB with shared caches; hub reachable only on its port.
**Stop** if: gates can't fail closed without redesign, or anything needs the full hub secret — then your bottleneck is governance and auth, not isolation, and containerising first would industrialise the current failure modes.