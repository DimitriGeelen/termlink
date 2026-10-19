## 1. Verdict

**B (Worker-owned, separate clone per claim)** — but only if the "claim" abstraction is treated as the *sole* coordination primitive. The recommendation correctly identifies that worktrees are fundamentally incompatible with the operator's no-worktree rule and cross-host requirements. However, B inherits the same Task-ID collision risk the operator already experienced with worktrees (Fact 3, bullet 4), because `.tasks/` lives *inside* the clone. A separate clone per claim does not solve this — it replicates it.

## 2. Risks of B not named

| Risk | Why it matters |
|------|----------------|
| **Task-ID collisions persist** | Each clone has its own `.tasks/`; parallel workers allocate overlapping IDs. The operator already saw 12 IDs collide across 3 branches. B makes this *worse* (more isolation = less visibility). |
| **Credential sprawl** | Every clone needs push credentials. If workers run on N hosts, you need N credential stores or a credential helper that works headless across hosts. |
| **Abandoned clone cleanup** | Claims expire → clones orphaned. No TTL-based reaper exists. Disk grows unbounded. |
| **Merge/integration latency** | N worker branches → N PRs or a single merge train. The orchestrator must sequence integration; no primitive exists for "merge these N branches atomically." |
| **Hook/governance drift** | Fresh clones miss uncommitted hooks, local `prepare-commit-msg`, `pre-push` scripts, and AEF gates. The operator already saw 3/4 pre-commit guards fail open. |
| **`--reference` speedup is fragile** | Requires a trusted local mirror on every host. If the mirror stales, clones fetch from remote anyway. |
| **No claim-to-clone binding in hub** | `channel.claim_get` only verifies ownership. It cannot enforce "this clone belongs to this claim" — a malicious or buggy worker could operate on any directory. |

## 3. The missing consumer

**Do not build the verbs yet.** No production orchestrator uses claims (Fact 2). Building `checkout/commit/publish` without a consumer guarantees API churn.

**Smallest real consumer**: A *single* AEF task that:
1. Spawns 2–3 parallel workers via `termlink dispatch` (or equivalent)
2. Each worker claims a unit, clones, makes a trivial change, commits, publishes
3. Orchestrator merges the resulting branches and verifies the result

Until this runs end-to-end in CI, the verbs are speculative.

## 4. What we are missing

- **Prior art**: GitHub Actions uses fresh clones per job (not worktrees). GitLab CI same. Both solve credentials via injected tokens and cleanup via TTL. They *do not* share `.git` — each job is a clean clone. This supports B *if* you add the missing pieces (credential injection, TTL reaper, Task-ID namespace).
- **Simpler design**: Keep the hub dumb. Add a **worker-side "session context"** file (written by the worker at checkout time) containing `{claim_id, repo_url, branch, task_id_namespace}`. The hub never runs git. The orchestrator reads this file via a new RPC `agent.worker_context(claim_id)` to verify/merge. This pushes complexity to the worker (where it belongs) and avoids hub-owned state.
- **Fact that changes the answer**: The operator's no-worktree rule is *repository-specific* ("this repository's own agent sessions"). Other projects vendoring AEF may not share this rule. Hardcoding "no worktrees ever" into the protocol limits adopters. Make it a *policy* (AEF config), not a protocol constraint.

## 5. One thing to change in the signatures

```
agent.checkout(claim_id, repo, ref, dir?) -> {dir, branch, head, task_id_namespace}
```

Add `task_id_namespace` (e.g., `claim_<id>_`) returned by the worker at checkout time. The worker *must* prefix all Task IDs it allocates with this namespace. The orchestrator (or a gate) rejects any task whose ID lacks the expected prefix. This turns Task-ID collisions from a silent data-corruption bug into a loud protocol violation.
