# T-2021 Inception Research — Substrate primitive #3: pull/assign RPC verb

**Status:** GO with revised scope (no separate pull verb; ship `channel.transfer_claim` + envelope convention).
**Artifact created:** 2026-06-08
**Successor task on GO:** Build task for Slices 1-4 (`channel.transfer_claim` end-to-end).
**See also:** T-2018 ADR §6; T-2019 (claim semantics, shipped); T-2020 inception (idle registry).

## 1. The §6 framing

ADR §6 primitive #3: *"Every existing TermLink path is push (sender picks recipient). There is no 'give me the next unit' RPC and no clean inverse for the orchestrator to hand a specific unit to a specific worker as a first-class operation."*

Two modes are implicit in that sentence:
- **Pull** — worker asks the hub for the next unit it can do.
- **Assign** — orchestrator picks a specific worker and hands it a specific unit.

The captured open questions (IW-1..IW-4) treat these as one design problem because they share the same substrate. This artifact resolves both.

## 2. What the substrate already has post-T-2019 / T-2044 / T-2020-recommended

| Verb                          | Source              | Role in pull/assign                                          |
|-------------------------------|---------------------|--------------------------------------------------------------|
| `channel.subscribe`           | pre-T-2018          | Worker watches a work-queue topic for new items.             |
| `channel.claim`               | T-2019              | Exclusive lock on `(topic, offset)` bound to an owner.       |
| `channel.release`             | T-2019              | Release with ownership check.                                |
| `channel.force_release`       | T-2044              | Operator/orchestrator override of stuck claim.               |
| `agent.find_idle` (proposed)  | T-2020 GO           | Hub-derived set of LIVE-and-not-busy agents.                 |
| `channel.post` to `dm:A:B`    | pre-T-2018          | Direct-addressable per-agent inbox via existing DM topics.   |

Across these, **pull** has every primitive it needs. **Assign** is missing exactly one verb: atomic transfer of an existing claim to a different owner.

## 3. Pull collapses to pure composition (NO new primitive)

Worker-driven workflow:

```
1. termlink channel subscribe work-queue:role-X --since-offset <last_acked>
2. On each envelope at offset N:
     termlink channel claim work-queue:role-X N --owner $WORKER_ID --leased 30s
3. If claim succeeds → process unit → channel.release.
   If claim fails (-32014 NOT_AVAILABLE / -32017 NOT_OWNED) → another worker won, skip to N+1.
```

Two key properties from T-2019 make this work:
- Hub serializes claim attempts → at most one worker wins per offset.
- Leased claims auto-expire → a worker that crashes mid-unit releases the slot back.

**Failure modes are already named.** No new RPC, no design decision, no race. This is the same shape as competing consumers on Kafka/SQS, but with the lease semantics native to the topic.

## 4. Assign needs ONE new primitive — `channel.transfer_claim`

The naive composition fails because claims are owner-bound:

```
orchestrator:
  C = channel.claim(topic, offset, owner=orchestrator)
  channel.post(dm:orchestrator:W, {action: assign, claim_id: C, ...})

worker:
  subscribes to dm:orchestrator:W → reads envelope
  channel.claim(topic, offset, owner=W) → -32014 NOT_AVAILABLE (orchestrator still holds it)
```

Three resolution options:

| Option | Mechanism                                            | Verdict |
|--------|------------------------------------------------------|---------|
| A      | Orchestrator releases first; worker races to claim   | RACE — second worker can win |
| B      | New RPC `channel.transfer_claim(claim_id, to_owner)` | ATOMIC, owner-checked transfer; minimal surface |
| C      | Worker re-claims via `force_release` then `claim`    | Two writes, not atomic; defeats T-2044's intent |

**B wins** because it adds one well-shaped RPC and preserves the substrate's "one writer, serialized" stance from ADR §5. Implementation reuses the SQL plumbing built for claim/release.

### `channel.transfer_claim` shape

```
POST hub.bus.channel.transfer_claim
  claim_id:    str  (required)
  to_owner:    str  (required, the new agent_id)
  reason:      str? (optional, audit trail)
  by:          str  (required, the current claim owner — checked)
→
  ok, claim_id, topic, offset, from_owner, to_owner
errors:
  -32016 CLAIM_NOT_FOUND
  -32017 CLAIM_NOT_OWNED  (by != current owner)
  -32018 CLAIM_EXPIRED
```

Note: `by` is required and checked (unlike `force_release`, which bypasses ownership). Transfer is a cooperative-with-audit verb; `force_release` is the operator-intervention verb. Both are needed; neither subsumes the other.

## 5. Recommended assign workflow (composition over the new RPC)

```
orchestrator:
  W = agent.find_idle(role=builder, capabilities=["rust","aef"])
  C = channel.claim(topic=work-queue:role-builder, offset=N, owner=orch-pid, leased=120s)
  channel.post(
    topic="dm:orch-pid:W",
    payload={"action": "assign", "claim_id": C, "source_topic": "work-queue:role-builder",
             "source_offset": N, "ttl_secs": 90},
    metadata={"_from": "orch-pid"}
  )

worker:
  on dm:orch-pid:W envelope:
    if action == "assign":
      result = channel.transfer_claim(claim_id=C, to_owner=W, by=orch-pid)
      if result.ok:
        process unit
        channel.release(C, by=W)
      else:
        channel.post(dm:W:orch-pid, {"action": "reject", "reason": result.error})
```

If W never picks up the envelope within `ttl_secs`, orchestrator's claim lease expires → orchestrator finds a different worker (or itself processes). **No new timeout mechanism** is required — T-2019's lease IS the failure mode IW-3 was asking about.

## 6. IW dispositions

- **IW-1 (push vs pull vs both):** BOTH, but they cost different things. Pull = zero new primitives (composition of existing subscribe + claim). Push/assign = one new primitive (`channel.transfer_claim`). Confidence=4.
- **IW-2 (worker-selection policy — hub or orchestrator-side):** Orchestrator-side. Hub provides the LIVE-and-idle filter via `agent.find_idle`; the *choice* (round-robin / least-loaded / capability-match) is AEF policy, not substrate concern. Keeps the substrate minimal per ADR §4 boundary. Confidence=4.
- **IW-3 (failure mode — unacked assignment):** Solved by T-2019's lease. Orchestrator's lease on the source-topic offset expires if neither orchestrator nor worker acts. No new reclaim mechanism needed. Confidence=4.
- **IW-4 (new RPC or composition):** BOTH, by mode. Pull is composition. Assign needs `channel.transfer_claim`. Confidence=4.

## 7. Cost / risk

- **New code:** ~120 LOC. Roughly mirrors `force_release` from T-2044 in shape and plumbing.
- **Slices:** 4 vertical slices (bus library + hub handler, CLI verb, MCP tool, docs+example).
- **Schema migration:** none — existing claims table already carries `claimed_by`; transfer is a single-row UPDATE.
- **Risk surface:** small. Transfer is a strict generalization of release-then-claim, with the race window removed.
- **Conflicts with prior primitives:** none. `channel.transfer_claim` composes with claim/release/force_release without overlap.

## 8. Recommendation

**GO with revised scope.** The captured framing in §6 implies a complex two-way RPC; in practice the substrate already supplies pull as composition, and assign needs only one new verb (`channel.transfer_claim`). The orchestrator workflow becomes a recipe over five existing/proposed verbs, not a new protocol.

**Build slice plan (mirrors T-2019 / T-2020 verticalization):**
- **Slice 1:** `channel.transfer_claim` RPC — bus library function (atomic UPDATE in claims table), unit tests including the by-mismatch and expired cases.
- **Slice 2:** Hub handler in `crates/termlink-hub/src/channel.rs` + router allow-list + error-code wiring (-32016 / -32017 / -32018, plus the existing CLAIM_NOT_AVAILABLE -32014 inherited).
- **Slice 3:** CLI verb `termlink channel claim-transfer --claim-id C --to-owner W [--reason ...]` + JSON envelope; session-client wrapper.
- **Slice 4:** MCP tool `termlink_channel_claim_transfer` + help-registry entry + docs/example showing the orchestrator → worker assign recipe end-to-end.

**(Optional Slice 5):** Document the pure-pull recipe in `docs/operations/substrate-claim-primitive.md` — no code, just the worker-loop incantation.

## 9. GO criteria evaluation (from §Go/No-Go Criteria)

- ✅ **Composition decision is final.** Pull = pure composition, assign = `channel.transfer_claim` + envelope. No further substrate decision needed.
- ✅ **AEF orchestrator can build against the resulting interface.** The recipe in §5 is concrete: five named verbs, well-documented payload shape, error codes inherited from T-2019.
- ✅ **Failure mode is named and bounded.** Lease expiry handles unacked assignments; `force_release` handles orchestrator crash mid-handoff; `transfer_claim` itself is atomic.

## 10. ADR alignment check

| ADR section | Alignment |
|-------------|-----------|
| §2 "append-log is the primary surface" | ✓ Assignment envelopes are channel posts; transfer is a single-row mutation on the existing claims table. |
| §4 "policy lives in AEF, not substrate" | ✓ Selection policy stays in orchestrator. Substrate provides the filter (`find_idle`) and the atomic transfer, no policy. |
| §5 "one writer, serialized" | ✓ Transfer is one atomic write at the hub, like every other claim mutation. |
| §6 #3 "no give-me-the-next-unit RPC" | ✓ Resolved: composition of subscribe + claim already gives pull; explicit assign verb shipped. |
| §9 "hard-dep for AEF" | ✓ Without these verbs the orchestrator dispatcher pattern is undefined. With them, AEF can implement worker pools cleanly. |

## 11. Open follow-up tasks to file on GO

- Build task: Slices 1-4 (transfer_claim + CLI + MCP + docs).
- Documentation task: pull-recipe in substrate-claim-primitive.md (could roll into Slice 4).
- AEF-side integration task: orchestrator dispatcher implementation against the new verb (not substrate-owned; for the §9 collaboration seam).
