# External review request: where should a dispatched agent worker's git checkout live?

You are an independent reviewer. Be critical; disagreeing with the recommendation is
useful. Answer in English, at most ~700 words, in markdown, with the sections listed at
the end.

## What we are building and why

**TermLink** is a Rust message-bus / session substrate that lets AI coding agents
(Claude Code, Codex, etc.) on several Linux hosts discover each other, exchange durable
messages, claim units of work, and control terminal sessions. Each host runs a "hub"
(TLS/JSON-RPC, SQLite metadata + append-only topic logs). Hubs do not federate; a client
talks to whichever hub it chooses.

**AEF (Agentic Engineering Framework)** is the governance layer above it: every change
needs a task, gates enforce acceptance criteria and verification before a task closes,
commits must reference a task, and a human operator keeps authority over decisions.
Many projects vendor AEF; agents in those projects use TermLink to talk to each other.

**The current focus is orchestration**: one orchestrator agent hands work units to N
parallel worker agents (possibly on different hosts), and the operator expects parallel
work to grow. The substrate already ships the coordination primitives:

- `channel.claim(topic, offset, claimer, ttl_ms)` -> exclusive lease on one work unit
  `{claim_id, claimed_until, ...}`; `renew`, `release(claim_id, claimer, ack)`, and
  `transfer_claim(claim_id, to_owner, by)` for an atomic orchestrator->worker handoff.
- `agent find-idle` (live agents minus those holding claims).
- Error codes CLAIM_CONFLICT / NOT_FOUND / NOT_OWNED / EXPIRED.

What is missing (decision "T-2026", approved by the operator) is a **typed git surface for
workers**, so the orchestrator stops inferring git state from chat messages:

```
agent.checkout(claim_id, repo, ref, dir?)  -> {dir, branch: "work/<claim_id>", head}
agent.commit(claim_id, message, paths?)    -> {commit, paths_added/modified/deleted}
agent.publish(claim_id, remote?, release?) -> {remote_ref, pushed, released}
```

Each verb is bound to a claim_id (already decided). Each would post one structured event
so the orchestrator and a web dashboard can replay a unit's history. `publish --release`
releases the claim only after the push is confirmed.

## Facts that matter

1. Today `termlink dispatch --isolate` creates **linked git worktrees** with local git on
   the dispatching host; the hub never runs git.
2. **No production orchestrator calls find-idle -> claim -> transfer today.** Only demo /
   smoke scripts and one proven-once "backlog drain" script do. The AEF's own dispatch
   (`fw termlink dispatch`) uses no claims and gives reviewers a `git archive` snapshot.
3. The operator has a standing rule: **this repository's own agent sessions never use a
   git worktree**, after repeated incidents:
   - A single `cd` into a worktree re-roots the whole agent session: the project-boundary
     gate falls back to the shell's cwd, after which `cd` back to the main checkout and
     even `git -C <main>` are refused as "another project". Only a session restart
     recovers. Three gates refused ordinary work in one session; one of them pointed at a
     tool (`fw integrate run`) that was never built.
   - Tooling derives names from the checkout's basename: the pre-push audit failed on a
     cron unit named after the worktree, and `fw cron install` from a worktree would write
     `agentic-audit-<worktree-name>` into /etc/cron.d.
   - Task-ID allocation scans only the local branch's `.tasks/`, so parallel worktrees
     allocated the same IDs: 12 IDs collided across 3 branches, and three agents
     independently fixed the same two defects. Nothing detected it for months.
   - A worktree materialises only tracked files. Framework files that were on disk but
     untracked in the main checkout were absent in worktrees: a CLI command crashed, and 3
     of 4 git pre-commit guards silently failed open because their scripts did not exist.
   - Worktrees share one `.git`: index/ref locks and hooks are shared state.

## The decision (IW-3): who owns the worker's checkout?

- **A. Hub-owned**: the hub (or dispatching host) creates checkouts and runs git on
  behalf of workers.
- **B. Worker-owned, separate clone per claim** (recommended): the worker's CLI makes its
  own clone (`git clone --reference <local copy>` for speed) under
  `$TERMLINK_WORK_DIR/<claim_id>`, never a linked worktree; the hub only sees claim calls
  plus one typed event per verb; a new read RPC `channel.claim_get(claim_id)` lets a verb
  verify the caller still holds the claim.
- **C. Worker-owned, linked worktree** of the worker's existing checkout (what
  `--isolate` does now; cheapest on disk).
- **D. Defer** until a production consumer exists; build nothing yet.

The recommendation is B, because it is the only option that works across hosts and does
not collide with the no-worktree rule, and because a failed push leaves the claim held
(visible as stuck) rather than silently "done".

## What we want from you

Sections:
1. **Verdict**: A/B/C/D or a different option, with your main reason.
2. **Risks of the recommended option B** we have not named (concurrency, disk, credentials
   for push, cleanup of abandoned clones when a claim expires, merge/integration of many
   worker branches, AEF task-ID collisions in parallel clones, hooks/governance inside a
   fresh clone, etc.).
3. **The missing consumer**: given no production orchestrator uses claims yet, should the
   verbs be built now, and what is the smallest real consumer that would prove them?
4. **What we are missing**: prior art (e.g. how other multi-agent coding systems isolate
   workers), simpler designs, anything in the facts above that changes the answer.
5. **One thing you would change** in the signatures.
