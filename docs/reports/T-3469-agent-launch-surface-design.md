# T-3469 — Design: agent.checkout / agent.commit / agent.publish (T-2026 GO, slice 1)

**Status:** signatures locked 2026-10-20 (§2), conditional on IW-3 = B. IW-3 is ready for
the operator as one decision brief: `docs/reports/T-3469-iw3-decision-brief.md`. §2.6 says
what changes under another ruling.

## 1. Facts measured 2026-10-19

1. Shipped claim API (crates/termlink-hub/src/channel.rs):
   1a. `channel.claim(topic, offset, claimer, ttl_ms?)` returns
       `{claim_id, topic, offset, claimer, claimed_at, claimed_until}`.
   1b. `channel.renew(claim_id, claimer, additional_ttl_ms)`,
       `channel.release(claim_id, claimer, ack)`,
       `channel.transfer_claim(claim_id, to_owner, by, reason?)`.
   1c. Errors: CLAIM_CONFLICT -32015, CLAIM_NOT_FOUND -32016, CLAIM_NOT_OWNED -32017,
       CLAIM_EXPIRED -32018 (crates/termlink-protocol/src/control.rs:421-432).
   1d. There is no lookup by `claim_id` alone; `list_claims` is per topic. A verb that
       must check "does the caller hold this claim" needs either the topic too or a new
       read `channel.claim_get(claim_id)`.
2. Where git runs today: `termlink dispatch --isolate` creates linked worktrees with local
   git on the DISPATCHING host (crates/termlink-cli/src/commands/dispatch.rs:119-140,
   `manifest::create_worktree`). The hub never runs git.
3. Who uses the claim primitives today: only in-tree scripts —
   `scripts/orchestrator-backlog-drain.sh` + `worker-backlog-drain.sh` (T-2204..T-2206,
   proven live once in June, not scheduled anywhere), the substrate demo/smoke scripts and
   the claim skills. The vendored AEF dispatch (`fw termlink dispatch`,
   `lib/termlink_worker.py`) uses no claims and no worktrees (reviewers get a
   `git archive` snapshot, agents/termlink/termlink.sh:1460). **No production orchestrator
   calls find-idle -> claim -> claim-transfer today.**
4. Standing rule: this repo's own sessions never use a git worktree (a `cd` into one
   re-roots the session; CLAUDE.md § Session rules).

## 2. Signatures (assuming IW-3 = B)

*(The 2026-10-19 draft that stood here was superseded on 2026-10-20 by §2 below. Changes:
`channel.claim_get` dropped in favour of the shipped `channel.renew`; the clone is
`--dissociate`d; the branch carries the hub id; publish fences on the remote; the default
end of `publish` hands the claim back instead of releasing it. Previous text: git history,
commit 722f8a6d8.)*

## 2. Locked signatures (2026-10-20; conditional on IW-3 = B, see §2.6)

Re-read against the tree on 2026-10-20 (file:line references below).

### 2.1 Where they run

The three verbs are **worker-side CLI verbs** (`termlink agent checkout|commit|publish`) with
MCP parity (`termlink_agent_checkout|commit|publish`). Git runs on the worker's host, in the
worker's own clone. **The hub gets no new RPC and stores no git state.** It sees the shipped
claim calls plus one event per verb. T-3470/T-3471 currently say "hub RPC + CLI + MCP". Under
B, "hub RPC" becomes "uses the shipped claim RPCs" (§2.7).

### 2.2 Ownership check: the shipped `channel.renew`, no new read RPC

`channel.renew {claim_id, claimer, additional_ttl_ms}`
(crates/termlink-hub/src/channel.rs:2409-2473) already does everything a verb needs before
touching git:

1. It checks that the lease is live and that `claimer` holds it. The errors are
   `CLAIM_EXPIRED` -32018 (the row is lazily evicted), `CLAIM_NOT_OWNED` -32017, and
   `CLAIM_NOT_FOUND` -32016 for an unknown id.
2. It returns `{claim_id, topic, offset, claimer, claimed_at, claimed_until}`, so the verb
   learns the topic from the `claim_id` alone. That closes the gap named in §1.1d.
3. It extends the lease by the verb's own budget, clamped to 1 h; a renew never shortens a
   lease. This is the "renew by heartbeat, not by the LLM remembering" the reviewers asked
   for (§3.4).

So every scope handle is a real `claim_id`, and the check is a call that already ships with
error codes and tests (crates/termlink-bus/src/lib.rs:2296-2343). `channel.claim_get` is not
needed.

### 2.3 The verbs

```
agent.checkout {claim_id, claimer?, repo, ref, work_dir?, ttl_ms?}
  -> {claim_id, topic, offset, dir, branch, base, head, claimed_until}
agent.commit   {claim_id, claimer?, message, paths?, ttl_ms?}
  -> {claim_id, commit, paths_added[], paths_modified[], paths_deleted[], claimed_until}
agent.publish  {claim_id, claimer?, remote?, expected_head?,
                then?: "transfer" | "release" | "keep", to_owner?, ttl_ms?}
  -> {claim_id, remote, remote_ref, commit, pushed, then, claim_state}
```

1. **`claimer`** is resolved through the existing chain: `$TERMLINK_AGENT_ID`, then
   `~/.termlink/be-reachable.state`. If neither gives a value the verb refuses; it never
   invents a claimer (the /claim skill's rule).
2. **`branch`** = `work/<hub8>/<claim_id>`, where `hub8` is the first 8 hex of the hub's TLS
   fingerprint (`termlink hub fingerprint`). The hub id is needed because claim ids are per
   hub (`clm-<ns>-<seq>-<tag>-<offset>`, crates/termlink-bus/src/meta.rs:1001-1031) and hubs
   do not federate (G-060). Every character in a claim id is legal in a ref name.
3. **`checkout`** clones `repo` into `work_dir`. When a local copy exists it uses
   `git clone --reference <local copy> --dissociate`: fast, and immune to a later gc in the
   reference repo (4 of 5 reviewers, §3.3). Otherwise it does a plain clone. `work_dir`
   defaults to `$TERMLINK_WORK_DIR/<hub8>/<claim_id>`, and `TERMLINK_WORK_DIR` defaults to
   `/var/lib/termlink/work`, outside every project tree. It never runs `git worktree add`.
   It then runs the governance bootstrap (`fw git install-hooks` when the repo vendors AEF)
   and **refuses** if a guard that should exist does not (fail-closed, §3.2). Re-running
   checkout for the same claim on an existing dir is idempotent.
4. **`commit`** runs a plain `git commit` in the clone, so the repo's own hooks run
   (task reference, secret scan, large file). The verb adds the typed result; it does not
   reimplement any hook. If a hook refuses, that refusal is returned as the error and the
   claim is untouched.
5. **`publish`** pushes `branch` with `--force-with-lease=<branch>:<expected_head>`.
   `expected_head` defaults to the remote ref's value seen at checkout, i.e. "nobody else
   pushed this branch since I took it". This is the remote-side fence for the
   check-then-act window (§3.4). After a `transfer_claim` the old holder's renew fails, and
   even a push that races that check cannot overwrite a branch the new holder has moved.
6. **`then`** (default `transfer`) runs only after the push is **confirmed**, meaning the
   remote ref equals `commit`:
   6a. `transfer` hands the claim to `to_owner` for integration. `to_owner` is **required**
       with `then=transfer`. It has no default because the hub does not persist who
       transferred a claim: `transfer_claim` returns `from_owner` to the caller only, and
       `reason` is not stored (control.rs:200-216). The orchestrator names itself in the
       assignment message, and the worker passes it through.
   6b. `release` releases with `ack=true`.
   6c. `keep` keeps the claim.
   On a failed push nothing happens to the claim: it stays held and shows up as stuck
   instead of silently done. The default is `transfer`, not `release`, because a pushed
   branch is not yet integrated work (§3.5).
7. **`remote`** defaults to `origin`; the verb never picks a remote by itself. For this
   repo that is OneDev (CLAUDE.md, CI / Release Flow).

### 2.4 Errors

The claim codes pass through unchanged (-32015..-32018). There are two new codes, the next
free after T-3484's -32026..-32028 (last assigned today: -32025,
crates/termlink-protocol/src/control.rs:523):

| Code | Name | When | `data` |
|---|---|---|---|
| -32029 | `WORK_GUARD_MISSING` | the checkout bootstrap found a required hook missing | `{dir, missing[]}` |
| -32030 | `WORK_PUSH_REJECTED` | the push was refused (lease mismatch, hook, auth, network) | `{remote, remote_ref, reason: lease\|hook\|auth\|network\|other, stderr_tail}` |

A failed `git commit` is a plain error carrying the hook output. It gets its own code only
if the build finds that callers need to tell it apart from a usage error. T-2699 applies:
each new code ships with its emission site and a test in the same change. If T-3484 does not
take all three of its codes, these two still take the next free ones in order.

### 2.5 Events

Each verb posts one envelope to `agent-work:<claim topic>` (retention `messages 1000`, the
recipe for per-agent topics). The body is the return shape plus
`{verb, agent_id, hub8, binding?}`. `binding` is the caller's T-3490
`<dispatch_id>:<attempt>` when it has one. That lets a T-3484 dispatch record and a work
history join without either owning the other. A failed event post never fails the verb;
the offline queue absorbs it.

### 2.6 What changes if the ruling is not B

1. **A (hub-owned):** the verbs become hub RPCs `agent.*` that run git on the hub host, and
   `dir` is a hub path. Cross-host workers cannot use them, because hubs do not federate.
   The shape above stays; only where it runs moves.
2. **C (linked worktree):** `checkout` becomes `git worktree add` from the worker's own
   checkout (what `dispatch --isolate` does today). `--dissociate` and the bootstrap step
   drop. The no-worktree rule would need an explicit carve-out.
3. **D (defer):** nothing is built and T-3470/T-3471 stay blocked.

### 2.7 What T-3470 and T-3471 must say (to apply once IW-3 is ruled)

1. **T-3470:** replace "hub RPC" with "CLI verb + MCP tool, ownership checked with the
   shipped `channel.renew`". The commit-fails case keeps the claim. The guard bootstrap
   refuses fail-closed, tested with a missing hook.
2. **T-3471:** test the `--force-with-lease` fence with a second pusher. Default
   `then=transfer`. A push refused by a fake remote returns `WORK_PUSH_REJECTED` and leaves
   the claim held.

## 2a. Consumer and one worked call sequence

**Named consumer:** the in-tree backlog-drain pair, `scripts/orchestrator-backlog-drain.sh`
(orchestrator) and `scripts/worker-backlog-drain.sh` (worker), from T-2204..T-2206. This is
the only code that already runs find-idle → claim → transfer, it was proven live once, and 4
of 5 reviewers named it as the smallest real consumer (§3.9). The proof run is two workers
on two hosts (.107 and .122), each taking one unit.

A second consumer is named but cannot call these verbs yet: AEF `fw worker` with
`backend: hub` (T-3484). Its contract has no `claim_id` field. Joining the two (a `claim_id`
on the dispatch record, or the dispatcher claiming before `dispatch.allocate`) is an item for
contract v0.3 and is not decided here.

Worked sequence (orchestrator `orch`, worker `w1`, topic `work-queue`, unit at offset 7):

```
orch: termlink agent find-idle --capability rust          -> [w1]
orch: channel.claim {topic: work-queue, offset: 7, claimer: orch, ttl_ms: 600000}
        -> {claim_id: clm-…-work_queue-7, claimed_until: …}
orch: channel.transfer_claim {claim_id, to_owner: w1, by: orch, reason: "T-NNNN"}
orch: agent contact w1 "claim clm-…-7, repo <onedev url>, ref main, hand back to orch"
w1:   termlink agent checkout --claim clm-…-7 --repo <onedev url> --ref main
        renew(+ttl) ok -> clone --dissociate -> branch work/cacc73ea/clm-…-7 -> hooks ok
        -> {dir: /var/lib/termlink/work/cacc73ea/clm-…-7, head: <base>}
      (w1's coding session is spawned with cwd = dir; it never cd's)
w1:   termlink agent commit --claim clm-…-7 -m "T-NNNN: …"   -> {commit: abc123, …}
w1:   termlink agent publish --claim clm-…-7 --then transfer --to-owner orch
        renew ok -> push --force-with-lease -> remote ref == abc123 -> transfer to orch
        -> {pushed: true, then: transfer, claim_state: "held by orch"}
orch: reads agent-work:work-queue, merges work/cacc73ea/clm-…-7, runs verification,
      channel.release {claim_id, claimer: orch, ack: true}
```

Failure drills the build must show:

1. The push is refused: the claim stays with `w1` and becomes stuck when its lease lapses.
2. `w1` dies after commit but before publish: the lease lapses, `orch` re-claims, and the
   unpushed clone is quarantined, never deleted (§3.7).

## 3. External review, round 1 (2026-10-19)

Operator asked for five external reviewers. Brief and answers verbatim in
`docs/reports/T-3469-consult/`. Reviewers: codex, claude -p, opencode GLM-5.3 (coding plan),
opencode DeepSeek v4.1 (free), opencode Nemotron-3-ultra (free). cursor-agent was tried and
dropped (not signed in). No paid route used.

1. Verdict: 5/5 choose B as the topology. Nemotron adds "do not build the verbs yet".
2. Unanimous: a clone is NOT a fix for the worktree incidents. Clones also have only
   tracked files and no `.git/hooks` (guards fail open), allocate task IDs from their own
   `.tasks/` (the 12-ID collision again, at higher parallelism), get a basename-derived name
   (`<claim_id>`), and an existing session that `cd`s into one is re-rooted the same way.
   GLM: "you have not fixed the history, you've industrialized it". These are
   governance-state problems to fix directly: central or claim-scoped task-ID allocation, a
   bootstrap that installs and verifies guards and refuses (fail-closed) if they are
   missing, and spawning the worker session with its cwd at the clone, never `cd`.
3. 4/5: `--reference` makes the clone depend on the reference repo's objects (a gc there
   corrupts clones): use `--dissociate` or a per-host bare mirror.
4. 4/5: `claim_get` then push is check-then-act. Fence at the remote: an ownership
   epoch/lease token (codex, deepseek) or `expected_head` / `--force-with-lease`
   (claude, deepseek); renew by heartbeat, not by the LLM remembering.
5. 5/5: integration is unaddressed: who merges N `work/*` branches, in what order, with AEF
   verification on the result. Claude: push is not done; default `publish` should
   `transfer` the claim back to the orchestrator for review instead of releasing it.
6. Namespacing: put hub id in branch names / checkout identity (codex, claude); claim_ids
   are per hub.
7. Cleanup: never delete unpushed work on expiry; quarantine or push to `abandoned/…`
   (codex, claude, glm, deepseek). Disk is dominated by build output (`target/`).
8. `commit`: claude and GLM suggest not wrapping it (workers already commit under AEF hooks;
   make it an observed event). DeepSeek would build commit and defer publish. Disagreement.
9. Build now? codex: a narrow vertical slice with a real consumer and failure drills;
   claude: checkout + publish with one consumer, defer commit; GLM: only with a consumer in
   the same arc; DeepSeek: claim_get + checkout + commit behind a flag; Nemotron: not yet.
   Consumer most named: revive the proven backlog-drain script as a 2-host run (codex,
   claude, deepseek, glm); GLM also: move AEF's own `fw termlink dispatch` onto it.
10. Prior art (claude, glm, deepseek): a disposable per-task container with a clone,
    push to a namespaced branch, integrate through a PR/merge queue; none run git in the
    coordinator. Containers would also remove the hook/cwd/basename failure modes.
11. Correction (codex): linked worktrees have their own index and HEAD; they share refs and
    the object store, not one index lock. The brief overstated this.
12. Nemotron: the no-worktree rule is this repo's; make it AEF policy, not protocol.

## 4. Open

1. IW-3: operator decision. The brief is ready in `docs/reports/T-3469-iw3-decision-brief.md`;
   the ruling goes in T-3469 Decisions.
2. T-3470 / T-3471 text: apply §2.7 once IW-3 is ruled. Not done in this slice, because those
   task files are outside its write area.
3. `claim_id` on the AEF dispatch record (§2a): an item for T-3484 contract v0.3.
