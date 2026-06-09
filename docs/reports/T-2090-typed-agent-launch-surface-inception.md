# T-2090: Typed agent-launch surface — inception

**Status:** Inception in progress
**ADR:** [parallel-execution-substrate §6 #8](../architecture/parallel-execution-substrate.md)
**Scope:** Substrate primitive #8 (Contract tier) — typed agent.checkout/commit/publish.
**Created:** 2026-06-09

---

## 1. The problem

**ADR quote (§6 #8):**

> "*Typed agent-launch surface aware of source-tree handoff* — `agent.checkout(ref)`, `agent.commit(scope)`, `agent.publish(branch)`. Today the only git awareness is the `dispatch --isolate` worktree wrapper; coordinating code plane vs governance plane is otherwise a shell convention sitting above TermLink. Making it typed turns the convention into a substrate concept the orchestrator can rely on."

### What already exists

The current `dispatch --isolate` flow (`crates/termlink-cli/src/commands/dispatch.rs:36`):

```
termlink dispatch --isolate --auto-merge -- claude code "do X"
  ├─ creates a git worktree per worker via manifest.rs:182 create_worktree()
  ├─ spawns the worker in that worktree
  ├─ collects exit + branch state
  └─ if --auto-merge: merges worker branches back via manifest.rs:287 merge_branch()
```

This already covers the happy path: orchestrator hands the worker a ref, worker commits, orchestrator merges. The convention works.

### What the ADR says is missing

Three named typed verbs the substrate doesn't expose:

| ADR verb | Today | Gap |
|---|---|---|
| `agent.checkout(ref)` | `create_worktree` exists, untyped | No MCP-callable substrate verb; orchestrator builds shell strings |
| `agent.commit(scope)` | Agent runs raw `git commit` | No "scope" concept (typed boundary for which files commit) |
| `agent.publish(branch)` | `merge_branch` exists, untyped | No standalone publish verb (only via `dispatch --auto-merge`) |

**Real-world consequence today.** An orchestrator (e.g. AEF Workflow agent) wanting to fan out work across 5 workers calls `termlink dispatch --isolate -- ...` and gets back a manifest. It works. There is no current incident where this is the bottleneck — making the case for #8 weaker than #9 (where the `agent-presence` O(N_heartbeats) walk IS a measured bottleneck).

---

## 2. Design alternatives

### A. Thin typed CLI wrappers

**Model.** Add three CLI verbs (`termlink agent checkout/commit/publish`) that thinly wrap the existing dispatch + git surface. No new substrate state.

```
termlink agent checkout <ref> [--worktree-path PATH]
  → creates worktree at PATH, returns {path, branch}

termlink agent commit <branch> --message "msg" [--scope <glob>]
  → runs git commit; --scope filters via git pathspec

termlink agent publish <branch> [--strategy merge|rebase]
  → merges branch back to base via manifest.rs::merge_branch
```

**Pros.**
- Minimal new code (~3 thin wrappers, ~300 LOC including tests).
- MCP parity straightforward (`termlink_agent_checkout/commit/publish` tools).
- Backward compatible — `dispatch --isolate` continues unchanged.
- Discoverability — appears in `termlink --help` as substrate primitives.

**Cons.**
- Cosmetic — wraps existing functionality; orchestrators that already use `dispatch --isolate` see no functional change.
- `--scope` enforcement requires git pathspec semantics; weakly enforced (agent could `git add` outside scope first).
- No "one agent per ref" invariant.

### B. Substrate-tracked branch-claim

**Model.** Hub maintains `branch_claims[repo][branch_ref] -> {agent_id, claimed_at_ms, lease_ttl_ms}` in-memory map. `agent.checkout` claims the ref atomically (refuses if claimed); `agent.publish` releases. Restart-safe via O(N) scan of an internal envelope log.

**Pros.**
- "One agent per ref at a time" invariant — orchestrators rely on substrate enforcement instead of coordination.
- Symmetric with T-2019 CLAIM (same shape, branch-keyed instead of offset-keyed).
- Composes with `find-idle` (T-2020): "find idle agent → checkout ref → release on done".

**Cons.**
- Substantial new state (schema for claim envelopes, restart-scan logic, lease-expiry GC).
- New failure modes (stale claims after agent crash → need force-release verb).
- No identified consumer demands this today.

### C. MCP-only typed surface

**Model.** Add only MCP tools (`termlink_agent_checkout/commit/publish`) — no new CLI verbs. Orchestrators use them via MCP; humans continue using `dispatch --isolate`.

**Pros.**
- Even lower-cost than A (~half the surface).
- Targets the actual consumer (orchestrating agents that already use MCP).

**Cons.**
- Lower discoverability for humans investigating substrate capability.
- Asymmetric — every other substrate primitive has CLI + MCP parity. Breaks the established pattern.

### D. Documentation-only

**Model.** Formalize the existing `dispatch --isolate` convention via a new `docs/operations/substrate-agent-launch.md` recipe. No new verbs. Mark ADR §6 #8 as "deferred — existing dispatch is sufficient pending a consumer".

**Pros.**
- Zero code cost.
- Acknowledges that #8 is speculative scaffolding without a load-bearing consumer.

**Cons.**
- Punts the typed surface to a future task when (if) a consumer demands it.
- Leaves the ADR §6 #8 item incomplete.

---

## 3. Recommendation

**RECOMMEND DEFER on substrate primitive #8 — pending a load-bearing consumer.**

### Why DEFER, not GO

1. **No measured pain.** Unlike #9 (where `/peers` O(N_heartbeats) walk is a real cost on `agent-presence`), #8 has no operator complaining about `dispatch --isolate` ergonomics. The existing surface works.
2. **Speculative scaffolding.** Adding typed verbs without a consumer who depends on the type contract is building API for nobody.
3. **Design B (substrate-tracked branch-claim) is the only version with substantive substrate value**, and it's the most expensive — new schema, new claim state machine, new failure modes. Not justified absent a consumer.
4. **Design A (thin wrappers)** delivers marginal value (re-packaged convention as API) at low cost — viable as a fast-follow when a consumer surfaces, but not urgent.
5. **#9 (cv_index) has higher ROI** for the same engineering attention — directly unlocks user-facing `/peers` perf.

### Why DEFER, not NO-GO

The ADR explicitly lists #8 as required. NO-GO would contradict the strategic decision recorded in the ADR. DEFER preserves the requirement, time-boxes the decision, and ties reopening to a concrete trigger:

**Revisit when:** an orchestrator (AEF Workflow agent, parallel-build dispatcher, or other) ships that uses `dispatch --isolate` at a scale where the untyped shell-string interface is a measured friction. At that point, Design A is the natural starting point and Design B is the natural follow-up if exclusive-ownership becomes a contention pattern.

### Recommendation summary table

| Design | Cost | Value | Verdict |
|---|---|---|---|
| A. Thin CLI wrappers | Low | Marginal (convention-as-API) | Fast-follow when consumer surfaces |
| B. Substrate-tracked branch-claim | High | Real substrate semantic gain | Wait for consumer |
| C. MCP-only typed surface | Low | Asymmetric, breaks pattern | Reject |
| D. Documentation-only | Zero | Acknowledges current state | Included in DEFER plan as bridge |

---

## 4. Slice plan (post-GO, if user overrides DEFER)

If the user overrides DEFER and decides GO, the natural starting design is Design A (thin wrappers). The 5-slice arc mirrors the substrate-observability pattern:

| Slice | Task | Scope |
|---|---|---|
| 1 | T-2095 | `termlink agent checkout <ref>` CLI verb + unit tests |
| 2 | T-2096 | `termlink agent commit <branch> --scope <glob>` CLI verb + git-pathspec scope enforcement |
| 3 | T-2097 | `termlink agent publish <branch>` CLI verb + tests |
| 4 | T-2098 | MCP parity (`termlink_agent_checkout/commit/publish`) |
| 5 | T-2099 | `docs/operations/substrate-agent-launch.md` recipe + migration note for existing `dispatch --isolate` consumers |

Total ~5 sessions. Design B (substrate-tracked branch-claim) would be a separate later inception.

---

## 5. Acceptance criteria (inception)

### Agent
- [ ] Problem clearly stated — typed verbs vs existing dispatch surface
- [ ] Four alternatives analyzed (A, B, C, D)
- [ ] Recommendation written with rationale
- [ ] Slice plan included for the GO-override path

### Human
- [ ] [REVIEW] Approve GO/NO-GO/DEFER via `fw task review T-2090` or Watchtower

## 6. Related primitives

- **Substrate primitive #1 (CLAIM, T-2019)** — same shape as Design B (in-memory exclusive-ownership map), different keying. If T-2089 also ships, that's the third instance; argues for a generic keyed-claim primitive at some future point.
- **Substrate primitive #2 (DISPATCH, T-2020)** — `find-idle` composes with `agent.checkout` in a Design A or B world ("find idle agent → checkout ref → spawn").
- **T-1922 (parallel-dispatch substrate)** — the existing `dispatch --isolate` consumer. Any #8 work should preserve this surface.

## 7. Dialogue Log

### 2026-06-09 — initial scoping (claude → claude)

Working from ADR §6 §"Contract" tier. The text framing suggests substrate-tracked state, not just typed wrappers — "making it typed turns the convention into a substrate concept the orchestrator can rely on". But examining the existing `dispatch --isolate` surface (`crates/termlink-cli/src/commands/dispatch.rs:36`, `manifest.rs:182,287`), the worktree-create and merge-back machinery already works. The orchestrator can use it today.

The strategic question is: are we building substrate primitives to BE used, or because the ADR says to? If a consumer demands it, the value is clear. If not, it's speculative.

Comparing against T-2089 (substrate #9), which has a measured pain point (`/peers` O(N_heartbeats) walk), #8 has no measured pain. The right answer for #8 today is DEFER pending consumer demand. This preserves the ADR requirement, time-boxes the decision, and avoids speculative scaffolding.

Five IW questions disposed: IW-1 (typed value marginal), IW-2 (substrate-tracked state unnecessary absent consumer), IW-3 (scope enforcement weak), IW-4 (generalizes CLAIM), IW-5 (no consumer today).

---

## 8. References

- ADR: [docs/architecture/parallel-execution-substrate.md §6 #8](../architecture/parallel-execution-substrate.md)
- Existing dispatch surface: `crates/termlink-cli/src/commands/dispatch.rs:36` (`cmd_dispatch`)
- Worktree create: `crates/termlink-cli/src/manifest.rs:182` (`create_worktree`)
- Merge-back: `crates/termlink-cli/src/manifest.rs:287` (`merge_branch`)
- T-1922 (parallel-dispatch substrate that uses the existing surface)
- T-2089 sibling-inception (substrate #9, GO-recommended) — higher ROI for same engineering attention
