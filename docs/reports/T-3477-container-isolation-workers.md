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
4. **Prior work on this host:** `aef-greenfield-testbed:latest` (484 MB, Ubuntu 24.04, built
   2026-10-18). AEF agent's answer (2026-10-19, msg 0881f46f): built by dimitri-mint-dev
   (their T-126 spike, T-127 runner; `tools/greenfield-testbed/` in that repo) for AEF's
   greenfield orchestrator (AEF T-4299, awaiting a decision).
   4a. Proved: level 1 install smoke without agent or credential (137 s); level 2 a full
       greenfield run with an agent inside (315–765 s per run), framework and hooks
       installed inside the container, leak-clean.
   4b. Login: a dedicated `claude setup-token` identity (not the operator's), fetched at run
       time from a secret-store slot by a read-only machine identity, written to a mode-600
       env file and passed to one `docker exec --env-file`; never in the image, never
       mounted; run records grepped for the token before keeping. Each runner is meant to get
       its own identity. This answers IW-1 in principle: proven elsewhere on this host.
   4c. Not proved: TermLink inside (none installed), build cache, more than one image.
       Next: ask dimitri-mint-dev about adding TermLink and a network mode.
   4d. dimitri-mint-dev's answer (2026-10-19, msg e76a3140; their evidence
       docs/reports/T-126-isolated-greenfield-testbed.md, T-133, T-141):
       - Do NOT add TermLink to `aef-greenfield-testbed`: it holds only what a new user
         has, so adding TermLink changes what the greenfield test measures. Build a derived
         image in our repo (`FROM aef-greenfield-testbed:latest` + the musl binary), or ask
         for an optional run.sh flag that copies a binary into the running container.
       - Network: a ufw rule for the docker bridge subnet to 9100/tcp only (an operator
         system change on .107). Avoid `--network host`: the worker would see every
         host-bound service (~13 Watchtowers, the hub admin surface) while their L2 runs an
         autonomous agent with bypass permissions.
       - Credentials: never mount the full `hub.secret` into a container running an
         autonomous agent. For a spike, a throwaway hub inside the docker network with its
         own secret, or wait for a scoped per-worker token. Claude token pattern: fetched at
         run time (Infisical), mode-600 temp env file for one `docker exec`, deleted after,
         leak grep of the whole record (run.sh exits 4 on a hit).
       - Measured: image 484 MB; Claude Code install ~13–15 s; L1 3 min 25 s (292 s under
         I/O load); L2 8 min 37 s (19 min under I/O load, their T-135). No build-cache or
         bare cold-start figures. Note AEF T-4317: never unpack a record inside another
         project (session trap).

## 3b. External design review, round 1 (2026-10-19)

Operator asked for five external designers plus Perplexity on the wider construct:
heterogeneous workers (models, harnesses, MCP, skills, commands, role-specific `CLAUDE.md`),
per-project container sets (TermLink, Workflow designer, AEF) and image versioning. Brief
and answers verbatim in `docs/reports/T-3477-consult/`. Reviewers: codex, claude -p, opencode
GLM-5.3, DeepSeek v4.1, Nemotron-3-ultra. Perplexity: via dimitri-mint-dev's browser seat,
pending their operator's approval. (opencode note: default output mode hangs without a tty on
long prompts, and parallel runs fail with "database is locked"; run one at a time with
`--format json`.)

**Where all five agree**
1. Containerise: one disposable Docker container per work unit. Not microVMs/VMs as the
   baseline (escalation for untrusted code only); bubblewrap has no image/versioning story.
2. The container is not the security or governance boundary. Containers fix at most part of
   the four incidents; the central allocator (T-3476) and fail-closed guards are needed
   regardless. GLM: otherwise "containers just make the same bugs parallel".
3. A small image family, never one image per role: base → (project toolchain) → harness, with
   the harness version pinned and self-update off. Role = configuration injected at start
   (role `CLAUDE.md`, skills, MCP config, commands, model, credentials).
4. Compose role `CLAUDE.md` from one canonical, segmented source by a generator with a
   recorded hash and a CI check; never hand-kept copies.
5. Dispatch by image digest, never `latest`; every result records image digest, config bundle
   hash, rendered `CLAUDE.md` hash, harness version, model id, base commit. GLM/codex: this
   makes runs attributable, not replayable.
6. Never `--network host`; never the full hub secret in a container. A server-enforced scoped,
   short-lived per-worker hub token is the long pole (GLM) and the first real prerequisite;
   until then a throwaway hub. Claude: a broad hub token is a container escape, because
   TermLink can inject into terminal sessions; the token must exclude session control.
7. Disk is the hard constraint: admission floor (~15 % free), sccache and shared read-only
   registry caches (never a shared writable `target/`), cap Rust workers (~3), short
   retention, keep forensics as exported artefacts.
8. A one-week pilot with deliberate failure injection (remove a guard, `--no-verify`, expire a
   lease, reach a LAN UI) and explicit go/stop criteria.

**Additions worth keeping**
1. Enforce governance outside the container (claude, codex): the remote only accepts pushes to
   `worker|agent/<task>/*`, merges stay with operator/orchestrator; Codex and opencode do not
   run Claude hooks, so client-side gates cannot be the control. Entry point self-tests every
   guard and refuses to start if any passes a known-bad input.
2. Firewall: a ufw INPUT rule does not stop a container reaching other LAN hosts; check
   FORWARD/DOCKER-USER, IPv6 and container-to-container (claude, codex, deepseek).
3. A control arm in the pilot: plain clone + allocator without containers (deepseek).
4. One shared subscription identity across parallel workers breaks attribution, rate limits and
   revocation; check terms for concurrent automated sessions (claude, deepseek).
5. Integration, not isolation, may be the real bottleneck; stop if merging costs more operator
   time than the work saves (claude).
6. Run the harness inside a TermLink-registered PTY in the container so the operator can watch
   and kill with existing tools (claude); a designed operator console (nemotron).
7. Subsetting `CLAUDE.md` is a policy decision for the operator: define the mandatory core
   (claude, codex).

**Disagreements / weak points**
1. Which incidents containers fix: claude says only #1 (session re-rooting); GLM says #2 and
   #4 (fresh full filesystem) but not #1; deepseek says none without fixed paths and a full
   clone. Consistent reading: a container helps only together with fixed paths, explicit IDs
   and fail-closed guards.
2. Image axes: claude/deepseek/codex put a project-toolchain layer between base and harness;
   GLM uses one image per harness only.
3. Nemotron is the weakest answer: it rejects `--network host` but uses it, with the full hub
   secret in an env var, in its own first step, and bakes `target/` into the project image on
   a full disk. Kept for two additions (per-ecosystem cache volumes, operator console).

## 4. Relationship to other work

1. T-3469 IW-3 (who owns the checkout) may be answered by this: inside a container the
   clone-versus-worktree question mostly disappears. IW-3 stays open until T-3476 and
   T-3477 have shaped.
2. T-3476 (assignment sidecar) supplies central task IDs and work assignment; containers
   supply isolation. The reviewers named both as prerequisites for parallel workers.

## 5. Dialogue log

1. 2026-10-19, operator: explore containerisation further, to add isolation if it is an
   option.
2. 2026-10-19, operator: get external advice on whether and how to build containers for our
   purposes (orchestration, different models, harnesses, MCP, skills, commands, role-specific
   or subset `CLAUDE.md`, per-project container sets for TermLink / Workflow designer, image
   versioning); reflect first, write one question, send to five external designers and
   Perplexity. Done: five answered (§3b), Perplexity pending via dimitri-mint-dev's seat.
3. 2026-10-19, operator: process the result, and share it with ring20-manager (who works on
   orchestration) for feedback before finalising. Sent on conversation
   `termlink-t3477-container-workers` (4 questions: changes, where assignment lives / overlap
   with T-3476, estate constraints, pilot participation). Finalising waits for ring20 and
   Perplexity.
