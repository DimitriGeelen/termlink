# External design review: should we containerise our AI-agent workers, and if so, how?

You are an independent systems designer. Be critical and concrete; "don't do this" is a valid
answer. Answer in English, at most ~1000 words, markdown, using the sections at the end.

## Context

**TermLink** is a Rust substrate (hub per host, TLS JSON-RPC, SQLite + append-only topic logs)
that lets AI coding agents on several Linux hosts discover each other, exchange durable
messages, claim units of work (lease-based claims with atomic transfer), and control terminal
sessions. Hubs do not federate.

**AEF (Agentic Engineering Framework)** is the governance layer vendored into ~30 projects:
every change needs a task, gates enforce acceptance criteria and verification before a task
closes, commits reference a task, and a human operator keeps authority. Each project has a
long `CLAUDE.md` (here ~2,500 lines) with project rules, plus hooks, skills, slash commands and
MCP servers configured per project. A per-agent "sidecar" process handles peer messages.

**Direction:** orchestration. One orchestrator agent hands work units to parallel worker
agents, possibly on different hosts, and the operator expects this to grow. Workers will be
**heterogeneous**:
- different models (Claude Opus/Sonnet/Haiku, GPT/Codex, GLM, DeepSeek, local models);
- different harnesses (Claude Code CLI, Codex CLI, opencode, possibly others);
- different MCP servers, skills and slash commands per role;
- a different `CLAUDE.md` per role, or a subset of the project one (a reviewer does not need
  the release runbook; a worker on the hub crate does not need the web UI rules);
- different projects: TermLink itself, a "Workflow designer" project, AEF itself, others.

The operator is considering **sets of container images** for these roles and projects, with
**image versioning**, so that a dispatched worker always starts from a known, reproducible
environment.

## Why isolation at all (incidents we already had)

Workers so far ran as extra sessions or git worktrees on the shared host. Measured failures:
1. A single `cd` into a worktree re-rooted a whole agent session; boundary gates then refused
   to return to the main checkout; only a restart recovered.
2. Tooling derives names from the checkout's directory name (cron units, audit records), so a
   worktree produced wrong system-level names.
3. Task IDs are allocated as "max in this checkout + 1": parallel branches handed out the same
   12 IDs, and three agents independently fixed the same two bugs, unnoticed for months.
4. A worktree only has tracked files; untracked framework scripts were missing, so 3 of 4
   git pre-commit guards silently failed open.
A separate review agreed that a plain clone per worker fixes only (shared .git), not 1–4.
A central task-ID allocator / assignment service is being designed separately; assume it.

## Facts measured on our main host (.107)

1. Docker 29.9 running; bubblewrap 0.12 available. 24 cores, 62 GB RAM, disk 88 % used
   (220 GB free) — disk is the tight resource; a Rust `target/` per worker is ~GBs.
2. An existing image `aef-greenfield-testbed` (Ubuntu 24.04, 484 MB, git/curl/python/Claude
   Code, no credentials in any layer) already runs an autonomous Claude Code agent inside a
   container: login via a dedicated long-lived subscription token (`claude setup-token`, its
   own identity), fetched at run time from a secret store (Infisical), written to a mode-600
   env file for one `docker exec --env-file`, deleted after; the run record is grepped for the
   token. Its owner asks that it stays a "what a new user has" baseline: derive from it, do
   not extend it.
3. Container start ~0.9 s (alpine); bubblewrap sandbox ~0.02 s; Claude Code install 13–15 s;
   a full greenfield agent run 8.5–19 min depending on host I/O load.
4. From Docker's default bridge network the container cannot reach the TermLink hub
   (ufw INPUT DROP); `--network host` works but exposes ~13 local web UIs and the hub admin
   surface to an agent running with bypass permissions. Preferred: a firewall rule for the
   bridge subnet to the hub port only.
5. Hub authentication today has no scoped credential: an authenticated call needs the full
   hub secret, which must never enter a container running an autonomous agent. A scoped,
   short-lived per-worker token would have to be built (or a throwaway hub per spike).
6. Another project created a VM as a test pad (details pending).

## Questions

1. **Should we?** Containers vs bubblewrap/namespace sandboxes vs microVMs (Firecracker,
   Kata) vs VMs vs "no isolation, fix the governance-state bugs instead". When is each right
   for AI coding agents that run shell commands with broad permissions?
2. **Image architecture for heterogeneous workers.** Layering (OS base → harness → project →
   role?), what to bake into the image vs inject at start (CLAUDE.md, skills, MCP config,
   slash commands, model choice, credentials), and how to compose a role-specific `CLAUDE.md`
   from a project one without the copies drifting.
3. **Versioning and reproducibility.** Tags, digests, pinning harness versions (these CLIs
   update weekly), recording which image a worker ran in on its result, rollback, and how the
   orchestrator chooses an image for a work unit.
4. **Governance inside.** Making AEF gates/hooks fail closed in the container, task IDs from
   the central allocator, commits and pushes from inside (credentials, branch namespacing),
   and how the operator keeps authority.
5. **Integration with orchestration.** Lifecycle per work unit (start, renew lease, finish,
   tear down, keep for forensics), state and caches (build caches on a nearly full disk),
   cross-host placement, and the TermLink hub connection (network + scoped auth).
6. **Prior art** we should copy or avoid (devcontainers, OpenHands runtime, SWE-bench
   harnesses, Codex cloud, Copilot coding agent, Nix, CI runners), with what each gets right
   or wrong for this case.
7. **Smallest first step** that would prove or kill the idea in a week.

## Sections to answer with

1. Verdict (containerise / sandbox / VM / don't) and the one-paragraph reason.
2. Recommended architecture (images, layers, what is injected, versioning).
3. Top 5 risks and how to contain them.
4. What we are missing or getting wrong.
5. Smallest first step, with what result means go or stop.
