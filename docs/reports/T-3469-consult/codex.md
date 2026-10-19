## 1. Verdict

**B, delivered alongside a real consumer.** Worker-local clones keep Git execution and credentials off the hub and respect the operator’s prohibition.

However, B is not the only cross-host option: C works when each worker has a local repository. Push-before-release is also independent of checkout ownership. B’s strongest justification is operational isolation under your constraints—not either claim.

## 2. Risks of the recommended option B

- **Most reported governance failures survive cloning.** Clones still lack untracked framework files, branch-local task allocation still collides, and `$TERMLINK_WORK_DIR/<claim_id>` still changes the basename. Switching an existing session into a clone may trigger the same project-boundary failure. Bootstrap and validate governance first, then start a fresh session inside that checkout. Allocate task IDs centrally or reserve disjoint ranges.
- **`--reference` weakens independence.** It borrows objects from another repository; pruning there can break the clone. Use `--reference … --dissociate`, or explicitly manage object retention in a cache. [Git clone documentation](https://git-scm.com/docs/git-clone)
- **`claim_get` is a check, not a fence.** Ownership can expire or transfer between validation and push. Renew leases during long operations, serialize local verbs, and bind mutations to an ownership generation. Strict stale-worker rejection requires enforcement at the push destination or a publishing intermediary; another read check cannot guarantee it.
- **Git, events, and release cannot commit atomically.** A push can succeed while its response disappears; event delivery or release can fail afterward. Persist operation IDs and recovery state, reconcile the exact remote commit, and deduplicate events. Promise one logical event per operation, not exactly one transmission.
- **Expiration is not cleanup authorization.** Preserve dirty files and unpublished commits, stop or fence the worker before reclaiming storage, and apply retention limits and quotas. Build artifacts and dependencies may dwarf Git storage.
- **Push authority needs explicit provisioning.** Define allowed repositories, remotes, branch namespaces, and noninteractive credentials. A clone is not a security sandbox.
- **Publication is not integration or task completion.** Assign an integrator, serialize landing, and rerun acceptance checks against the resulting target branch. Failed push leaves the claim held only until expiry unless renewed; retain durable failure status beyond the lease.

## 3. The missing consumer

Build the approved verbs now **as a narrow vertical slice**, not a general orchestration platform.

Turn the backlog-drain script into an operator-used workflow: two genuine, preallocated AEF tasks, two workers on different hosts, claim transfer, clone/bootstrap, verified commits, publication, and explicit integration into the target branch.

Prove recovery by interrupting one worker, rejecting one push, and losing one response after a successful push. The consumer must recover without duplicate work, premature acknowledgment, or lost commits. A happy-path demo would leave the hardest semantics untested.

## 4. What we are missing

The options conflate **where Git runs**, **who provisions the directory**, and **which isolation mechanism is used**. A worker-host launcher could provision the clone before starting the agent, while the CLI owns subsequent mutations. That better matches your session-boundary incidents.

One factual correction: linked worktrees have separate indexes and HEADs; they do not share one index lock. They do share substantial repository state, including ordinary refs. This correction does not override the operator’s rule. [Git worktree documentation](https://git-scm.com/docs/git-worktree)

Claude Code documents worktrees for parallel sessions, demonstrating established practice—not compatibility with AEF’s assumptions. Your incidents primarily expose checkout-dependent governance that needs repair regardless of isolation choice. [Claude Code workflows](https://code.claude.com/docs/en/common-workflows#run-parallel-sessions-with-worktrees)

Also, namespace persisted checkout identities by hub and repository, not just claim ID, unless global uniqueness is guaranteed.

## 5. One thing you would change

Replace bare `claim_id` across the verbs with a typed claim context:

```text
claim: { hub_id, claim_id, ownership_epoch }
```

Bind the owner to authenticated identity. Increment the epoch on ownership changes and reject stale operations wherever authority is enforced. This makes reassignment explicit; `claim_get(claim_id)` alone cannot provide that guarantee.
