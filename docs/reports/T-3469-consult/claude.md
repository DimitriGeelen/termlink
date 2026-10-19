# Review of IW-3: who owns a dispatched worker's git checkout

## 1. Verdict

**I agree with B (worker-owned clone), but only a smaller version of it, and only alongside a real consumer.** The facts that support B are cross-host operation, no shared `.git`, and no shared locks or hooks. Those are real advantages. But most of the incidents behind the no-worktree rule are not caused by worktrees. They are caused by any checkout that is not the main one. A fresh clone has the same problems:

- **It only has tracked files.** Untracked framework files and `.git/hooks` are not cloned. Your pre-commit guards would fail open again, just as they did in fact 4.
- **Name collisions.** The clone's directory name is `<claim_id>`, so tooling that builds names from the directory name (cron units, audits) breaks the same way.
- **Duplicate task IDs.** ID allocation still scans only the local `.tasks/`. Parallel clones will hand out the same IDs, just as parallel worktrees did.
- **Session re-rooting.** If an existing agent session `cd`s into the clone, the project-boundary gate re-roots it exactly as before.

So B is right about where the checkout lives. It does not by itself make the checkout safe to govern. Treat "a governable checkout" as its own deliverable.

## 2. Risks of B you have not named

- **`--reference` corruption.** Alternates make the clone depend on the reference repo's object store. A `git gc` or prune on the reference can corrupt live clones. Use `--reference … --dissociate`, or `--local` hardlinks on the same filesystem.
- **Bootstrapping the clone.** After cloning, you need an explicit step that installs hooks and framework files, then *verifies* the guards exist. If they don't, refuse to proceed rather than fail open. The worker session must also be **spawned with its working directory at the clone**, never `cd` there.
- **Task IDs.** Either the orchestrator creates the AEF task before dispatch and the worker is forbidden to allocate IDs, or IDs come from an atomic counter on the hub. Without one of these, the collision bug from fact 4 comes back.
- **Branch names across hubs.** Hubs don't federate, so claim_ids are probably unique per hub only. Two hubs pushing `work/<claim_id>` to one remote can collide. Put the hub ID in the branch name.
- **Lease expiry versus push (TOCTOU).** `claim_get` followed by a push is advisory only. A worker whose claim has expired can still push after a new owner has taken over. The real fence is at the remote. Push with `--force-with-lease=<ref>:<expected>`, or reject in a server-side hook.
- **TTL versus long builds.** A Rust build can outlast the TTL. Renewal needs a heartbeat that does not depend on the LLM remembering to call `renew`.
- **Cleanup.** Garbage collection keyed on expired claims must never delete unpushed commits. Push them to `abandoned/<hub>/<claim>` or move them to quarantine first.
- **Disk.** A full `target/` directory per clone dominates disk use, not `.git`. Plan for sccache, or a size cap per work dir.
- **Push credentials.** Every worker host needs write access. Restrict it to `work/*` with a server-side ref policy, not by convention.
- **Integration.** N branches still need someone to merge them. Nothing in IW-3 says who merges, in what order, or how conflicts are reported. That is where parallel work will actually hurt.

## 3. The missing consumer

Don't build all three verbs on speculation, but don't fully defer either. The operator expects parallel work to grow. **Build `checkout` and `publish` together with one consumer:** convert the backlog-drain script so it runs two workers on two hosts against one remote. Acceptance should cover:

- one clean run end to end;
- one forced push rejection, where the claim stays held and shows up as stuck;
- one claim expiring mid-work, where the stale push is fenced off and the clone is quarantined, not deleted;
- dashboard replay of all three runs.

**Defer `commit`.** Workers are coding agents and already commit with git, subject to AEF hooks. Wrapping commit is the verb most likely to fight those hooks. What the orchestrator actually needs is observed state, and `publish` (or a read-only `agent.status(claim_id)`) can report that: base, head, commits ahead, dirty paths.

## 4. What you're missing

- **Prior art agrees on one pattern:** ephemeral sandbox, fresh clone, push to a namespaced branch, integrate through a PR or merge queue. Examples:
  - GitHub Copilot coding agent: runner, then a `copilot/*` branch, then a PR.
  - Codex cloud, Devin, OpenHands, SWE-agent: a container per task.
  - CI runners: a cached reference repo plus a per-job clone, which is your `--reference` idea with the gc problem already solved.

  None of these run git in the coordinator. That supports rejecting A.
- **The remote can be the source of truth.** The orchestrator can check `git ls-remote <remote> work/*` itself. Events are for history and the dashboard, not for correctness. That makes the hub events nice to have and reduces how much the verbs must guarantee.
- **Containers.** A container per claim would give a pristine environment with hooks baked into the image. It also removes most of the gate, cwd and basename failure modes for good. It is worth a look before you harden bare-directory clones.
- **"Released" is not "done".** Under AEF, a task closes after verification, not after a push. Releasing the claim on push drops the chain of custody right before review.

## 5. One change to the signatures

Replace `publish(claim_id, remote?, release?)` with:

```
agent.publish(claim_id, remote?, expected_head, then: hold | transfer(<owner>) | release)
  -> {remote_ref, base, head, pushed, claim_state}
```

The default for `then` should be `transfer(orchestrator)`, so the unit moves into review still held under a claim instead of going back to the pool. `expected_head` makes the push fenced and idempotent: a retry after a network failure is safe, and a stale worker cannot overwrite a newer owner's work.
