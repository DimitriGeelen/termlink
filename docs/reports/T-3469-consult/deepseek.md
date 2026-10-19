## 1. Verdict

**B, with amendments.** A makes the hub a credential-bearing git executor, contradicting fact 1 and creating a single blast radius. C reintroduces every banned failure mode (shared `.git`, basename-derived names, `.tasks/` scanning) and only "works" if the worker already has a checkout. D is defensible, but the verbs themselves are small; the expensive parts are credentials and integration, which B defers. B matches prior art: disposable per-task checkout, a branch, an explicit integration step.

However, B as written is necessary but not sufficient. It does not fix the two root causes behind the no-worktree incidents, and its claim check is racy.

## 2. Risks of B not named

- **Check-then-act race.** `claim_get` before `commit`/`publish` is TOCTOU. A long pre-commit hook or test run can outlive the TTL, or the orchestrator can `transfer_claim` mid-operation. The worker then acts on a claim it no longer holds.
- **Task-ID collisions persist.** Each fresh clone scans only its own branch's `.tasks/` — precisely the mechanism that produced 12 collisions. Parallel clones do not fix this; they reproduce it. Atomic allocation must move to the hub (or a repo-wide allocator) before N grows.
- **Untracked files again.** A clone materializes only tracked files. Untracked vendored framework files and hook scripts will again be absent, so guards fail open. Worse, `.git/hooks` is never cloned at all. You need a bootstrap step and a fail-closed check that required guards exist.
- **`--reference` is fragile.** Alternates point at a live checkout; `gc`, prune, deletion, or moving the reference repo breaks every clone. Across hosts there may be no local copy to reference. Prefer a per-host bare mirror, or `--dissociate`/repack after clone.
- **Credentials.** Every worker host needs push rights, scoping, rotation, and redaction. The pre-push audit runs per clone and may need network access.
- **Disk and abandonment.** N full clones, plus clones whose claims expired while git was still running. Expiry does not kill the process. You need a reaper, process termination, and an "abandoned" event.
- **Push ≠ done.** `publish --release` marks the unit complete at push time, but integration is still a merge of N branches. Without a merge queue, ordering, conflict policy, and gates, "done" is a lie. Use `refs/heads/termlink/<claim_id>`, never a default branch.
- **Authority.** A `claim_id` should not be the only credential; bind verbs to caller identity.

## 3. The missing consumer

Build `claim_get`, `checkout`, and `commit` now behind a flag; defer `publish` until credentials and integration are designed. The smallest real consumer is the proven backlog-drain script revived with two workers on disjoint files, driving claim → checkout → commit, plus a kill/expiry test and a scripted stale-worker attempt. Demo scripts don't count as adoption: require something like 20 units, zero ID collisions, zero lost commits before AEF dispatch depends on it.

## 4. What you are missing

Prior art — SWE-bench/OpenHands, Codex cloud, Copilot coding agent, Cursor background agents — all use disposable per-task environments, a branch, and a PR/merge queue as the integration gate. B copies the first half; copy the PR gate too. A simpler design: keep the hub git-blind, have the worker post a typed result (sha, branch, patch digest) over the existing claim event stream, and make the three verbs thin CLI wrappers. That is one schema change instead of four RPCs. Finally, the no-worktree rule's root causes are governance-state problems (IDs, hooks, untracked files), not worktree-specific; fixing those matters more than the git surface.

## 5. One thing to change

Add a fencing/expected-head argument to every mutating verb: `claim_get` returns a monotonic lease token (and `checkout` returns `base_sha`), which `commit` and `publish` must present. `publish` also requires an explicit target ref and rejects non-fast-forward. This converts check-then-act into a guarded operation.
