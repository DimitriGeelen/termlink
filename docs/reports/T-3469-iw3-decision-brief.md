# T-3469 IW-3 decision brief: who owns a dispatched worker's git checkout?

**For:** the operator. One decision. **Prepared:** 2026-10-20. **Not yet presented.**
**Ruling goes in:** T-3469 `## Decisions`.
**Inputs:** `docs/reports/T-3469-agent-launch-surface-design.md` (facts §1, locked signatures
§2, external review §3), `docs/reports/T-3469-consult/` (5 reviewer answers verbatim),
T-3476 (central task-ID allocation, shaping), T-3477 (container isolation, shaping).

## 1. Background

1. **What is being decided.** T-2026 (GO 2026-10-18) adds three typed git verbs for worker
   agents: `agent.checkout`, `agent.commit` and `agent.publish`, each bound to a `claim_id`.
   Before they can be built, someone has to own the directory the worker's git work happens
   in. That choice also fixes where the verbs run.
2. **What exists today.** `termlink dispatch --isolate` makes **linked worktrees** on the
   dispatching host. The hub never runs git. No production orchestrator uses claims yet; the
   only consumer is the in-tree backlog-drain script pair, proven live once (design doc §1.3).
3. **The standing rule.** This repo's own agent sessions never use a git worktree. One `cd`
   into a worktree re-roots the whole session, and only a restart recovers. The other
   incidents behind the rule: basename-derived cron names, 12 colliding task IDs across 3
   branches, and pre-commit guards failing open because untracked framework files were
   missing.
4. **What the reviewers said (5/5, 2026-10-19).** All five chose a worker-owned separate
   clone. All five also said a clone does **not** by itself fix the incidents: a clone also
   lacks untracked files and hooks, allocates task IDs locally, and re-roots a session that
   `cd`s into it. Those are governance-state problems, to be fixed directly:
   4a. central task-ID allocation (now T-3476);
   4b. a bootstrap that installs and verifies guards and refuses when one is missing
       (now in the locked `checkout`, design §2.3.3);
   4c. spawning the worker session with its cwd already at the clone, never `cd`.
5. **The boundary question the AC asks.** The rule is about *this repo's own interactive
   sessions* landing in a second checkout of this repo. A worker clone under
   `/var/lib/termlink/work/…`, started there by a launcher, is a different object: no session
   moves into it and no session moves out. Each option below states how it relates to the
   rule.

## 2. Options

### A. Hub-owned checkouts

The hub (or the dispatching host) creates each worker's checkout and runs git for it; the
verbs are hub RPCs.

1. Rule boundary: untouched for interactive sessions, but the hub must hold a copy of every
   repo plus push credentials.
2. Cross-host: does not work. Hubs do not federate (G-060), so a worker on .122 cannot use a
   checkout that the .107 hub owns.

### B. Worker-owned separate clone per claim (recommended)

The worker's CLI clones into `/var/lib/termlink/work/<hub8>/<claim_id>` with `--dissociate`,
installs and verifies hooks (fail-closed), and pushes `work/<hub8>/<claim_id>` with a
lease fence. The hub sees only claim calls and one event per verb. This is the shape locked
in design §2.

1. Rule boundary: **the rule stands unchanged.** Interactive sessions in this repo still
   never use a worktree, and never `cd` into a worker clone. Clones are not worktrees and
   live outside the project tree. Only a launcher starts a session inside one, with that
   session's cwd set there from the start.
2. Cross-host: yes. Each host clones for itself.
3. Prerequisite before many workers run in parallel on a vendored-AEF repo: T-3476
   central task IDs. Without it the 12-ID collision returns at higher parallelism. The
   verbs can be built and proven with one or two workers before that.
4. Forward compatibility: if T-3477 later says "container per claim", B's clone simply
   lives inside the container. The verbs do not change.

### C. Worker-owned linked worktree

`checkout` does `git worktree add` from the worker host's existing checkout (what
`--isolate` does now).

1. Rule boundary: needs an explicit carve-out ("worktrees are allowed for workers"). This
   brings back the shared `.git` (refs, hooks, gc) and the untracked-file gap: the conditions
   behind three of the incidents.
2. Cross-host: yes, but only on hosts that already hold a checkout.
3. Cheapest on disk and already built.

### D. Defer

Build nothing until T-3476 and T-3477 have shaped, and a production orchestrator exists.

1. Rule boundary: untouched.
2. T-3470 and T-3471 stay blocked, and orchestration keeps inferring git state from chat.

## 3. Scoring against the value drivers

Scores are 0–5 per driver. Weighted sum = Σ score × weight. Both files were read on
2026-10-20.

### 3a. Project drivers (`policy/value-drivers.yaml`): D1 9, D2 7, D3 5, D4 3, F-RECALL 6, F-ORCH 5 (max 175)

| Driver (weight) | A | B | C | D | Why B scores as it does |
|---|---|---|---|---|---|
| D1 Antifragility (9) | 1 | 4 | 1 | 2 | a failed push leaves the claim held and visible; a lapsed worker's clone is quarantined, not lost |
| D2 Reliability (7) | 2 | 4 | 1 | 3 | it fixes the repeated incidents' causes (fail-closed hooks, no shared `.git`); C repeats them |
| D3 Usability (5) | 3 | 3 | 4 | 1 | one more directory per claim to clean up; C is cheapest and already built |
| D4 Portability (3) | 1 | 4 | 3 | 3 | plain git and a path; no hub-held repos or credentials |
| F-RECALL (6) | 1 | 2 | 1 | 1 | per-claim event history can be replayed |
| F-ORCH (5) | 2 | 4 | 3 | 0 | a typed, cross-host worker surface: what the driver rewards |
| **Weighted** | **57** | **123** | **66** | **59** | |

### 3b. Framework drivers (`.agentic-framework/policy/value-drivers.yaml`): D1 9, D2 7, D3 5, D4 3, F-RECALL 6, F-AUTONOMY 4, F3 prompt quality 7, F1 context fabric 7, F2 component fabric 6 (max 270)

The framework file has no F-ORCH. In its place it carries F-AUTONOMY, F3, F1 and F2.

| Driver (weight) | A | B | C | D | Note |
|---|---|---|---|---|---|
| D1–D4, F-RECALL | as 3a | as 3a | as 3a | as 3a | same scores |
| F-AUTONOMY (4) | 2 | 3 | 1 | 0 | B closes a loop: the push result reaches the claim with no human relay, and it removes no gate |
| F3 V_PROMPT_QUALITY (7) | 0 | 0 | 0 | 0 | not affected by any option |
| F1 V_CONTEXT_FABRIC (7) | 1 | 2 | 1 | 0 | structured per-unit events instead of chat inference |
| F2 V_COMPONENT_FABRIC (6) | 1 | 1 | 1 | 0 | no option changes the component map much |
| **Weighted** | **68** | **135** | **68** | **59** | |

Both files rank B first by about 2×. The scores are judgement and the operator may move
them, but no single score change closes the gap. Even raising C's D1 and D2 to B's 4 (as if
the incidents had not happened) gives C 114 against B's 123 on the project file.

## 4. Steelman and strawman

1. **A.** Steelman: the merge and integrate step is simplest when one place holds every
   branch, and workers need no git credentials. Strawman: TermLink becomes a git server
   per host; it is useless across hosts without federation, which the charter excludes.
2. **B.** Steelman: the only option that works across hosts, keeps the no-worktree rule
   intact, fences pushes, and carries into containers unchanged. Strawman: a clone fixes
   only the shared `.git`. Every governance-state problem (task IDs, hooks, session cwd)
   still needs its own fix, so B with none of those fixes is "the history, industrialized"
   (GLM, §3.2). Answer: two of those fixes are inside the locked `checkout` (hook bootstrap,
   cwd-at-spawn). The third, T-3476, is a stated prerequisite for parallel use.
3. **C.** Steelman: it already exists, it is cheapest on disk, and linked worktrees do have
   their own index and HEAD (codex's correction, §3.11). Strawman: it reopens the exact
   conditions behind the incidents the rule was written for, in exchange for disk space.
4. **D.** Steelman: there is no production consumer yet. T-3476 and T-3477 may change the
   picture, and building early risks rework (Nemotron). Strawman: the design is now pinned to
   shipped calls, not guesses. The backlog-drain pair gives a real two-host consumer. And
   B's verbs carry into containers unchanged, so the rework risk is small while the cost of
   waiting is open-ended.

## 5. Recommendation

**B, with the rule boundary stated as:** "the no-worktree rule is unchanged for this
repo's sessions. Worker clones are not worktrees, live under `/var/lib/termlink/work`, and
are only ever entered by a launcher that starts the session there."

1. **Conditions:**
   1a. T-3470 and T-3471 build and prove B with the backlog-drain pair on two hosts, one
       unit each.
   1b. More than two parallel workers on a vendored-AEF repo wait for T-3476 (central task
       IDs).
   1c. If T-3477 rules for containers, the clone moves inside the container and the verbs
       stay as locked.
2. **Ask the operator:** A, B, C or D, and if B, whether the boundary sentence above is the
   one to record.

## 6. Operator input while the decision was open (2026-10-21)

### 6.1 btrfs copy-on-write instead of a full clone (how B's checkout is made)

The operator's point: create worker checkouts by btrfs snapshot or copy-on-write, so a new worker costs
almost no disk writes. Measured on .107 (btrfs, root subvolume `@`, zstd), with two cargo builds
running, in the scratch area:

| Method | Wall time | New data written | Contents |
|---|---|---|---|
| `cp -a --reflink=always` of the checkout, without `target/` | 43 s | 72 KiB of 4.05 GiB | everything, untracked framework files included |
| `git clone --local` | 35 s | 122 MiB | tracked files only |
| `btrfs subvolume snapshot` of a prepared template subvolume | 8.8 s | 0 B | the template, as a working git checkout |

Findings:
1. Both copy-on-write methods write essentially no data, which is the property the hub needs.
2. A snapshot is not free: it forces a filesystem-wide transaction commit, so on a busy disk it waits,
   and it may itself add a flush stall. Copy-on-write copying costs one metadata entry per file.
3. A template that is copied includes the untracked framework files and the installed hooks, so it removes
   the "clone lacks untracked files" gap that the reviewers raised against plain B.

This refines **how** B's `checkout` makes the clone; it is not a new option. Proposed: on btrfs hosts,
checkout = snapshot (or reflink copy) of a per-repo template kept current by fetch; elsewhere, `git clone`
(portability, D4). Asked 0909-boot-into-timeshift-snapshots, who runs btrfs snapshots here, about
pitfalls (nested subvolumes inside `@` vs Timeshift, qgroups, cleaner load on delete).

### 6.2 git-ai (github.com/feder-cr/git-ai)

What it is: an MIT-licensed, self-hosted git server for AI agents, built on GitLab's Gitaly (v0.3.0, about
1.3k stars, "interfaces may still change before 1.0"). Adopting it would mean running a second git server
beside or instead of OneDev: too much impact for now. Concepts worth borrowing:
1. **Land = rebase onto the current tip, then compare-and-swap on the ref, with batched concurrent lands.**
   This is what B's lease-fenced `publish` and ring20's landing step approximate; a CAS on the target ref
   closes the remaining race.
2. **A merged result lands only if a configured verify command passes on the exact tree**, including
   AI-resolved conflicts. This is a clean rule for `agent.publish`/merge.
3. **Per-agent tokens with RPC allowlists and branch policies.** This is directly relevant to T-3493
   (who may inject) and to B's push scope (a worker may push only `work/<hub8>/<claim_id>`).
4. **A changeset log as an event stream** (agents follow it). This matches B's per-verb events.
