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

## 3. Open

1. IW-3: operator decision (brief in chat 2026-10-19; ruling to be recorded in T-3469
   Decisions).
2. Named consumer (T-3469 AC 2): §1.3 shows none in production. To be settled after IW-3.
