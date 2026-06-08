# T-2026 Inception Research — Substrate primitive #8: typed agent-launch surface (checkout/commit/publish RPCs)

**Status:** DEFER until Foundation primitives ship. Design will reflect T-2019..T-2021's actual claim/release/transfer-claim shape, not the §6 sketch. Re-evaluate when Foundation is in production.
**Artifact created:** 2026-06-08
**revisit_at:** 2026-09-08 (90 days — Foundation primitives' production behavior informs the verbs' return shapes)
**revisit_evidence_needed:** Foundation primitives (T-2019/T-2020/T-2021/T-2027) shipped and in AEF use; ≥1 incident where shell-convention git ops produced an integration gap that typed RPCs would have caught.
**See also:** T-2018 ADR §6 #8; existing `termlink dispatch --isolate` and `--auto-merge` flags; T-2019 claim semantics.

## 1. The §6 framing

ADR §6 primitive #8: *"Today the only git awareness in TermLink is the `dispatch --isolate` worktree wrapper. Coordinating code-plane vs governance-plane is otherwise a shell convention sitting above the substrate. Typed `agent.checkout(ref)` / `agent.commit(scope)` / `agent.publish(branch)` turns the convention into a substrate concept the orchestrator can rely on."*

The primitive asks for **typed RPCs** in place of shell-convention git operations.

## 2. What the substrate has today

- `termlink dispatch --isolate` creates per-worker git worktrees (`crates/termlink-cli/src/commands/dispatch.rs`).
- `--auto-merge` merges worker branches back to base (requires `--isolate`).
- `crates/termlink-cli/src/manifest.rs` tracks worktree branches created by dispatch.
- **No** `agent.checkout` / `agent.commit` / `agent.publish` RPCs in `termlink-protocol`.
- Agents run shell git inside their worktree; orchestrator infers state from topic posts.

So the dispatch-side surface exists (worktree creation + merge), but the WORKER-side surface is convention-only.

## 3. What the typed surface would add

Three concrete benefits of moving from shell-convention to typed RPCs:

1. **Structured response.** `agent.commit` returns `{commit_hash, paths_modified: [...], paths_added: [...], paths_deleted: [...]}` instead of stdout that the orchestrator has to parse. Feeds directly into T-2022's git-hook path-declaration story (the typed surface IS the path-declaration mechanism, no separate hook needed).
2. **Audit trail.** Every checkout/commit/publish becomes a substrate event posted to a per-agent topic. Orchestrator + Watchtower can replay agent work-history cleanly.
3. **Coordination with claims.** `agent.commit --scope T-XXX` could automatically release the claim on the assignment topic for that scope (closing the orchestrator → worker → done loop via one verb).

None of these is necessary for the substrate to FUNCTION; all of them are ergonomics + integration wins.

## 4. Why defer

Three reasons the typed surface should NOT be designed now:

**a) Return shape couples to T-2019/T-2021 semantics.** `agent.commit --scope T-XXX` needs to know what `scope` is. If T-2021 ships `channel.transfer_claim` (per that artifact's GO), `scope` could be `claim_id`. If T-2021 ends up shipping a different shape, `scope` could be `assignment_envelope_id`. Designing before T-2021 lands risks throwing the design away.

**b) Worktree lifecycle (IW-3) is genuinely undecided.** Hub-owned (current `dispatch --isolate` model) vs spoke-owned matters for ring20's cross-host story. Hub-owned simplifies merge but couples hosts; spoke-owned matches the hub-and-spoke topology but requires worktree-creation primitives on the spoke side. This is a real choice that hinges on observed cross-host AEF behavior, which doesn't exist yet.

**c) The shell-convention path is not broken.** It works for the current dispatch model. T-2026 is an ergonomics improvement, not a gap that's blocking other work. Other primitives (T-2021, T-2027, T-2028 Track B) have higher marginal value.

## 5. Recommended sketch (for orientation only — DO NOT lock until revisit)

When the time comes, the verbs likely look like:

```
agent.checkout(target_ref: str, in_worktree: WorktreeRef?) -> {head: str, dirty: bool}
agent.commit(scope_id: str, message: str?, paths: [str]?) -> {commit_hash, paths_modified, paths_added, paths_deleted}
agent.publish(branch: str, remote: str?) -> {published: bool, remote_ref: str, error: str?}
```

Where:
- `scope_id` = `claim_id` from T-2019 OR `assignment_id` from T-2021 — TBD based on which primitive lands first
- `WorktreeRef` = optional handle if the worker can have >1 worktree (likely not initially)
- All three verbs post a structured event to a per-agent audit topic
- `agent.commit` optionally fires `channel.release` on the named scope when successful (one-step done-and-release)

But this is sketch. Locking shape requires Foundation in production.

## 6. IW dispositions

- **IW-1 (exact RPC signatures — return shape):** DEFERRED. Foundation primitives lock the parameter shapes. Confidence=1.
- **IW-2 (scope binding — agent_id / session_id / claim_id):** PARTIALLY RESOLVED — likely `claim_id` if T-2019 stays as-shipped + `transfer_claim` lands. But final answer hinges on T-2021's exact shape. Confidence=2.
- **IW-3 (worktree lifecycle — hub-owned or spoke-owned):** DEFERRED. Hinges on observed cross-host AEF behavior that doesn't exist yet. Confidence=1.
- **IW-4 (un-partitionable file handling — §5 hub-owned regeneration):** DEFERRED, likely a separate primitive ("primitive #11" per the IW-4 hint). Not in scope for T-2026 today. Confidence=2.

## 7. Recommendation

**DEFER until Foundation primitives ship.** revisit_at=2026-09-08. The shape of `agent.commit(scope, ...)` depends on what `scope` means after T-2019 + T-2021 stabilize; designing before then is speculative.

**Why not GO now:** Designing before T-2021 ships risks throwing the design away. Two of four IW questions can't be answered without Foundation in production.

**Why not NO-GO:** The ergonomics + integration benefits are real, and the §6 framing's argument that "the convention is invisible to the substrate" is correct. NO-GO would leave that gap permanently unaddressed.

**Conservative provisional sketch:** §5 above is for orientation. Do not lock or build against it.

## 8. GO criteria evaluation (from §Go/No-Go Criteria)

- ⏸ "Signatures locked" — DEFERRED, depend on Foundation.
- ⏸ "Lifecycle decided" — DEFERRED, needs cross-host AEF evidence.
- ⏸ "Un-partitionable-file path in scope or split" — provisionally SPLIT (file as primitive #11 if needed).

## 9. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §5 code plane / governance plane | ✓ Typed RPCs sit at the seam; respect plane split. |
| §6 #8 framing | ✓ Captured framing is correct; just not yet designable. |
| §9 hard-dep for AEF | ✓ AEF dispatch builds against this surface eventually. |

## 10. Open follow-up tasks to file

- **At revisit (2026-09-08), conditional on Foundation evidence:**
  - Build task: `agent.checkout` + `agent.commit` + `agent.publish` typed RPCs (~200 LOC, 4-5 vertical slices).
  - Possible primitive #11 inception: un-partitionable file handling per §5 (hub-owned regeneration after merge).
