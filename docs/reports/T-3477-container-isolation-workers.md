# T-3477 — Container isolation for dispatched agent workers (inception research)

**Status:** shaping. Recommendation DEFER until the spikes below have run.
**Origin:** operator, 2026-10-19: explore containerising worker runs for more isolation, after
two of five T-3469 reviewers (claude, GLM) pointed at per-task containers as prior art.

## 1. Why this is on the table

Every incident behind the no-worktree rule came from a worker sharing state with something:
the session's cwd/project root, the checkout's untracked files and hooks, the basename-derived
names, the local `.tasks/` counter. The T-3469 reviewers agreed a bare clone fixes only the
shared `.git`. A container per claim gives the worker its own filesystem, process tree and
(optionally) network, built from an image that already carries the framework, hooks and
tools, and it is thrown away afterwards.

Prior art named by the reviewers: OpenHands / SWE-bench harnesses, Codex cloud, GitHub
Copilot coding agent, Devin, CI runners: a fresh environment per task, a clone inside it,
push to a namespaced branch, integrate through a PR or merge queue.

## 2. Facts on this host (.107, measured 2026-10-19)

1. Docker 29.9.0, daemon active (already runs watchtower, open-webui, sprechloop-dev);
   bubblewrap 0.12.0 and `unshare` available (lighter, rootless sandboxes).
2. 24 cores, 62 GB RAM (39 GB available), root filesystem 88% used (220 GB free). Disk is
   the tight resource; a Rust `target/` per container would dominate it.

## 3. Spikes (mirrored as IW-1..IW-5 on the task)

1. **Auth without copying secrets:** can a Claude Code / Codex worker authenticate inside a
   container without copying the operator's credential files (standing rule: never copy
   `~/.claude/.credentials.json`)? Options to test: a dedicated worker account/token, a
   read-only bind mount, an auth proxy. The outcome decides feasibility.
2. **Governance inside the image:** an image with the vendored framework and hooks; show
   that a task created inside fails closed when a guard is missing, and that its task ID
   comes from the central allocator (T-3476), not the container's `.tasks/`.
3. **TermLink inside:** the worker must reach the hub (TLS, presence, claims) from the
   container; check discovery, DM and the waker.
4. **Cost:** cold start, image size, and a shared build cache (sccache or a cache volume)
   so N containers do not each rebuild `target/`.
5. **Lighter option and other hosts:** bubblewrap/unshare sandbox with a clone versus
   Docker on isolation, start time and complexity; hosts without Docker.

## 3a. Spike results (2026-10-19, .107)

1. **Login (IW-1), options established, nothing run:**
   1a. `claude setup-token` creates a long-lived subscription token; it reaches the
       container as an environment variable (`CLAUDE_CODE_OAUTH_TOKEN`), so no credential
       file is copied. Needs the operator to run it once (interactive), and a decision on
       where the token is stored.
   1b. An API key works but is a paid route (operator approval each time).
   1c. Bind-mounting `~/.claude` shares the operator's credentials: ruled out.
2. **TermLink from inside (IW-3):**
   2a. On Docker's default bridge network the container cannot reach the hub at
       192.168.10.107:9100: three probes timed out after 10 s. ufw is active with
       `INPUT DROP`, and the bridge subnet is not allowed.
   2b. With `--network host` the same probe returned the hub fingerprint
       (`sha256:cacc73ea…`) in 1.19 s including container start.
   2c. Hub auth has no scoped credential: `termlink token create` issues tokens for a
       session, not for the hub. A container that must post or claim would need the full
       `hub.secret`. A per-worker, scoped, short-lived hub token is a missing primitive.
3. **Start cost (IW-4):** bare `alpine:3` container start 0.92 s; a bubblewrap sandbox
   (read-only root, private /tmp and pid namespace) 0.02 s. Image and build-cache disk not
   measured yet.
4. **Prior work on this host:** an image `aef-greenfield-testbed:latest` (484 MB, Ubuntu
   24.04, built 2026-10-18) exists. It is AEF's; ask the AEF agent what it proved before
   building our own.

## 4. Relationship to other work

1. T-3469 IW-3 (who owns the checkout) may be answered by this: inside a container the
   clone-versus-worktree question mostly disappears. IW-3 stays open until T-3476 and
   T-3477 have shaped.
2. T-3476 (assignment sidecar) supplies central task IDs and work assignment; containers
   supply isolation. The reviewers named both as prerequisites for parallel workers.

## 5. Dialogue log

1. 2026-10-19, operator: explore containerisation further, to add isolation if it is an
   option.
