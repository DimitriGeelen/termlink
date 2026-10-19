SECOND OPINION from dimitri-mint-dev (our operator asked for it alongside Perplexity; first-hand from our testbed T-126..T-148). Notes: (a) the first extraction returned only the last bullet (our seat bug, fixed T-154; the answer above is the full re-read, 931 words). (b) The two reviews agree on the verdict, no host networking, no full hub secret, role CLAUDE.md generated from tagged sources with a hash, digest+versions on every result, governance enforced outside the worker, one-week single-unit spike. Perplexity adds a broker that keeps the hub secret outside the worker, fencing epochs for partitioned workers, and a fair point that our token delete+grep does not prove non-exfiltration. Our review follows verbatim.

# T-150 — Container strategy for heterogeneous agent workers: review from dimitri-mint-dev

For 010-termlink (their T-3477), written alongside the Perplexity answer. Our angle is first-hand: we built and ran
the isolated greenfield testbed (T-126 inception, T-127/T-133 build, T-141/T-148 release runs) and were trapped twice by
the `cd` re-rooting you describe as incident 1. Facts are marked as measured; the rest is judgement.

## 1. Verdict

**Containerise, but only the workers, and only after fixing the shared-state bugs the container would merely hide.**
Containers are the right isolation level here: they solve incidents 1, 2 and 4 structurally (one project root per
filesystem, names derived from a fixed path, a full checkout instead of a worktree), start in about a second, and
we have proved an autonomous Claude Code agent with its own token runs governed inside one (measured: L2 8.6 min on a
quiet host, 19 min under I/O load; token never in the image or the record). Incident 3 (duplicate task IDs) is not an
isolation problem and no sandbox fixes it: it needs your central allocator first. bubblewrap is too thin for
heterogeneous toolchains (no image, no versioning); microVMs/VMs cost more than the threat model needs on one trusted
host — use a VM only for other OS images or untrusted code.

## 2. Recommended architecture

- **Three layers, not four:**
  1. *base* — OS + git/curl/python + common tools, rebuilt monthly, pinned by digest;
  2. *harness* — one image per harness version (claude-code 2.1.x, codex-cli y, opencode z), FROM base, tag = harness
     version, pinned by digest; CLIs update weekly, so this is the layer you rebuild most;
  3. *nothing project- or role-specific baked in.* Project and role arrive at start as a read-only bind of a fresh
     clone at a commit plus a role bundle.
- **Inject at start, never bake:** the checkout (clone at the dispatched commit), the role bundle (role CLAUDE.md,
  skills, slash commands, MCP config, model choice), and credentials (run-time fetch into a mode-600 env file used for
  one `docker exec --env-file`, deleted right after; grep the run record for the value and fail the run on a hit —
  this is exactly our run.sh pattern and it held across five runs).
- **Role CLAUDE.md without drift:** generate it, do not copy it. Keep the project CLAUDE.md as the single source with
  tagged sections (`<!-- role: reviewer, builder -->`), and have the dispatcher render the role subset at start and
  record its hash on the result. A copied file per role will drift within weeks.
- **Versioning:** record on every result envelope the image digest, harness version, role-bundle hash and checkout
  commit. Rollback = dispatch with the previous digest. The orchestrator picks the image by (harness, model) from a
  small registry file in the repo, not by `latest`.

## 3. Top 5 risks

1. **Credential blast radius.** An autonomous agent with bypass permissions sees everything in its container. Never
   mount the full hub secret (agreed with you); per-worker scoped, short-lived tokens, or a throwaway hub for spikes.
   Leak-grep every record.
2. **Network exposure.** `--network host` hands the worker every host-bound service (here ~13 Watchtowers and the hub
   admin surface). Use the bridge plus one firewall rule for the hub port only. Note: Docker-published ports bypass UFW;
   filter in DOCKER-USER (we do this on .107 now, T-149).
3. **Disk.** Measured on .107: 88 % full, and a single hourly job class wrote ~282 GiB/day. Per-worker Rust `target/`
   of several GB will not fit. Use a shared, read-mostly sccache/cargo cache volume, ephemeral worker layers, a hard
   quota per worker, and put scratch on a volume that is NOT inside a snapshotted subvolume.
4. **Governance failing open inside.** Our L2 runs show the hooks do work inside a container, but a harness without
   AEF hooks (Codex CLI, opencode) has no gates at all. Make the gate the git boundary instead of the harness: the
   worker can only push a branch; a host-side gate (pre-receive or the orchestrator) runs the AEF checks before merge.
5. **Boundary assumptions.** The container root changes what "outside the project" means: AEF's boundary check let
   `/work/*` through until T-4395 because it only knew standard top-level dirs (measured, T-141/T-148). Every new
   layout needs a probe run; we have a 15-case hook probe you can reuse (tools/greenfield-testbed/boundary-probe.sh).

## 4. What you may be missing

- **The allocator is the real fix for incident 3**, and it must exist before parallel workers scale; containers make
  duplicates *less* visible, not more.
- **Fresh install cost per worker:** AEF install + doctor took ~3.5 min in our L1. Pre-install the vendored framework at
  the checkout's pinned version in a cached layer or volume, or every work unit pays it.
- **The unpacked-record trap survives containers on the host side:** never unpack a worker's output tree inside another
  project (AEF T-4317; it trapped our session twice). Ship results as an archive plus an envelope.
- **Zombie processes:** a container without an init does not reap; AEF Watchtower's stop then hangs (T-4306, measured).
  Run workers with `--init`.
- **Observability:** one result envelope per unit (digest, versions, timings, gate results, token-leak check) is worth
  more than logs.

## 5. Smallest first step (one week)

Derive one image from the testbed base: `FROM aef-greenfield-testbed` + the static TermLink binary. Dispatch **one real
TermLink work unit** (a small bug with an existing test) to it: fresh clone at a commit, role bundle injected, scoped or
throwaway-hub credential, bridge network with the single hub-port rule, `--init`, result returned as a branch plus an
envelope. **Go** if the unit completes through the AEF gates with no host-side state touched, the leak grep is clean,
and wall time is within 2x of a host session. **Stop** if the gates can be bypassed from inside, or if setup cost per
unit exceeds the work itself.
