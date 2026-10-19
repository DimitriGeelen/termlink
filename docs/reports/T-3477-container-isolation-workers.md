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

**Perplexity (6th reviewer, via dimitri-mint-dev's seat, 2026-10-19; verbatim with sources in
`T-3477-consult/perplexity.md`)** agrees with points 1–8 and adds:
1. A **trusted broker** instead of a hub token: the peer sidecar stays outside the worker under
   the supervisor; workers get a narrow endpoint (assignment, heartbeat, bounded task-topic
   messages); the broker keeps the full hub secret and authorises every call. Never generic
   terminal control or arbitrary JSON-RPC forwarding.
2. Workers commit locally; a **trusted publisher** pushes approved `workers/<task>/<attempt>`
   branches with credentials workers never see; only the authority service closes tasks.
3. **Fencing epochs**: lease expiry cannot stop a partitioned worker; stale attempts must be
   unable to publish or change task state after reassignment.
4. A central ID allocator prevents ID collisions, not redundant work; assignments need scope,
   ownership and overlap checks (relevant to T-3476).
5. Docker and ufw: published ports bypass ufw through the DOCKER-USER/NAT path; host-service
   access follows INPUT; test IPv4, IPv6 and sibling containers rather than trusting a recipe.
6. System-level and service-install tests are meaningless in a container; route them to
   disposable VMs (the 0909 test VM may fit here).
7. "The core abstraction is an execution contract, not a role image": the dispatched manifest
   (image digest + source SHA + bundle hashes + harness + model + task/attempt + policy +
   limits) is the unit, and the orchestrator picks from an approved capability catalogue.

**dimitri-mint-dev second opinion (first-hand from their testbed T-126..T-148; verbatim in
`T-3477-consult/dimitri-mint-dev.md`)** agrees and adds measured specifics:
1. Three layers: base → one image per harness version → nothing project/role baked (fresh clone
   + role bundle at start).
2. Docker-published ports bypass UFW; they filter in DOCKER-USER on .107 now (their T-149).
3. One hourly job class on .107 wrote ~282 GiB/day; put worker scratch on a volume that is not
   inside a snapshotted subvolume.
4. Make the git boundary the gate (push a branch only; host-side checks before merge), because
   Codex/opencode run no AEF hooks.
5. Container layouts change "outside the project": AEF's boundary check let `/work/*` through
   until AEF T-4395; reuse their 15-case hook probe (`tools/greenfield-testbed/boundary-probe.sh`).
6. AEF install + doctor costs ~3.5 min per fresh worker: pre-install the pinned framework in a
   cached layer or volume.
7. Run workers with `--init` (no zombie reaping otherwise; Watchtower stop hung, AEF T-4306).
8. Ship results as an archive plus an envelope; never unpack a worker's tree inside another
   project (AEF T-4317).
9. First step: `FROM aef-greenfield-testbed` + the static TermLink binary, one real small
   TermLink bug with an existing test; go if it passes the AEF gates with no host state touched,
   a clean leak grep and wall time within 2x a host session.

**ring20-manager (orchestration, arc-009)**, first reply 2026-10-19 (msg 0326268a): received
after a 2.5 h sidecar outage on their side (AEF T-4416); a full answer will come from their S-4
session (first standing worker); short version: **interested in the pilot**.

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
