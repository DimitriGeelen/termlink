# T-3469 — Design: agent.checkout / agent.commit / agent.publish (T-2026 GO, slice 1)

**Status:** draft; IW-3 (worktree ownership) is with the operator as a decision brief.
Signatures below assume the recommended answer (B, worker-owned separate clone) and
change if the ruling differs.

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

The verbs run on the worker (CLI `termlink agent …`, MCP parity), where its checkout is.
The hub stores no git state; it sees the claim calls and one typed event per verb.

```
agent.checkout(claim_id, repo, ref, dir?)
  -> {claim_id, dir, branch: "work/<claim_id>", head}
agent.commit(claim_id, message, paths?)
  -> {claim_id, commit, paths_added[], paths_modified[], paths_deleted[]}
agent.publish(claim_id, remote?, release?: bool = false)
  -> {claim_id, remote_ref, pushed: bool, released: bool}
```

1. Scope handle is always a real `claim_id` (IW-2). Each verb first checks the caller
   holds the claim: needs `channel.claim_get(claim_id) -> ClaimInfo` (new, read-only),
   refusing with -32016 / -32017 / -32018 unchanged.
2. `checkout` makes a separate clone (`git clone --reference` the local copy when one
   exists, for speed) under `$TERMLINK_WORK_DIR/<claim_id>`, never a linked worktree.
   A clone has its own `.git`; nothing a worker does can re-root another session.
3. `publish --release` releases with `ack=true` only after the push is confirmed, so a
   failed push leaves the claim held (and visible as stuck) instead of silently done.
4. Each verb posts one envelope to `agent-work:<claim topic>` with the return shape
   above plus `agent_id`, so the orchestrator and Watchtower can replay a unit's history.

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

1. IW-3: operator decision (brief in chat 2026-10-19; ruling to be recorded in T-3469
   Decisions).
2. Named consumer (T-3469 AC 2): §1.3 shows none in production. To be settled after IW-3.
