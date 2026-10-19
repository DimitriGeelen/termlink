## 1. Verdict

**B — but B is necessary, not sufficient, and it only beats D if built against a real consumer in the same arc.** B is the only topology that survives cross-host dispatch and the standing no-worktree rule, and its failure mode (failed push → claim visibly held) is the right one. However, note that two of your own incidents were never about worktrees per se — they were about *checkout-local state* (task-ID allocation scanning local `.tasks/`; untracked framework assets). Per-claim clones reproduce both, deterministically and at higher parallelism. If you ship B without moving task-ID allocation hub-side and making governance assets tracked/vendored, you have not fixed the history, you've industrialized it.

## 2. Risks of B you haven't named

- **Task-ID collisions get worse, not better.** N clones each scanning their own branch's `.tasks/` is the 12-ID incident by design. Allocation must become a hub RPC or a claim-scoped namespace before the verbs ship.
- **Governance hooks fail open in a fresh clone too.** Fact 6 applies equally to clones: guards that were untracked in the main checkout don't exist. Either track them, or `agent.checkout` must install/verify the governance surface and refuse to proceed if guards are absent.
- **`git clone --reference` without `--dissociate` creates hidden shared state**: alternates make the local reference repo load-bearing; a `gc --prune` there corrupts every worker clone. Pick a side: fast-and-coupled, or dissociated-and-honest.
- **Cleanup is not just disk.** Abandoned clones contain the remote URL and possibly embedded credentials; per-claim dirs under a shared `$TERMLINK_WORK_DIR` may be world-readable on multi-worker hosts. You need a reaper keyed to claim-expiry events, a quarantine window, and sane perms.
- **Remote ref hygiene.** `refs/heads/work/<claim_id>` grows unbounded on a shared remote; nobody owns pruning after release. Verify claim_id charset is refname-safe.
- **Races.** `claim_get` is check-then-act: claims can expire between verify and commit, or mid-`publish`. Worst case: push confirmed but claim expired before release → `NOT_OWNED` on release, pushed-but-unowned work. After `transfer_claim`, the new owner's `checkout` must be idempotent against a branch already pushed by the prior owner. Verbs need idempotency and event-log reconciliation, not just pre-checks.
- **Integration is the deferred cost**: N parallel `work/*` branches is a merge storm the verbs don't address. Publish should target a staging namespace with a separate integration gate — which also fits AEF's human-authority rule.

## 3. The missing consumer

Build now only with a consumer in the same arc; otherwise this is D with extra steps. Smallest real consumer: retrofit AEF's own `fw termlink dispatch` — already production, already hands reviewers snapshots — onto `checkout/commit/publish`, then promote the proven backlog-drain script to a scheduled 2-host, 3-worker run through `find-idle → claim → transfer_claim`. Proven when the dashboard replays a unit's history from typed events alone, and a deliberately failed push renders as *held*, not *done*.

## 4. What you're missing

- **Prior art isolates at the container layer, not the git layer.** OpenHands/SWE-bench-style systems run ephemeral per-task containers with a clone inside: teardown solves cleanup, the image solves untracked-file fidelity and bakes in governance hooks, per-container credentials solve the push-auth problem. B + disposable container beats B alone.
- **CI workspace hygiene** (Jenkins/Buildkite workspace-per-job with reaper + quarantine) is exactly your per-claim clone; steal those policies.
- **Simpler design worth considering**: only two verbs need to *act* (`checkout`, `publish`). `commit` could be a pure structured event — the worker runs git itself, the CLI emits the event. Same observability, smaller governance and credential surface.
- Fact 6 cuts against B as much as C; the root causes (local ID allocation, untracked assets, basename-derived naming) follow you to any topology unless fixed directly.

## 5. One signature change

Drop `repo` and `ref` from `agent.checkout` and bind `{repo, ref, task_id}` to the claim at claim/transfer time; all verbs take `claim_id` only and verify via `claim_get`. This kills the checkout-a-different-ref-than-reviewed mismatch class, and enforcing AEF's "commits must reference a task" at the substrate is only possible if the claim already knows the task. (Corollary: make `publish`'s `release?` an explicit ack-carrying step mirroring `channel.release(claim_id, ack)`, not a boolean.)
