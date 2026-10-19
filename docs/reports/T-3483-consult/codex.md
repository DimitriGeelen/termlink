## 1. Verdict

**Choose C as the durable primitive; make sidecar notification optional, not part of the completion contract.** AEF’s answer undermines E’s main benefit: a project-level receiver cannot reliably wake the particular dispatcher, short-lived dispatchers may have no receiver, and the harness already provides a wake-up mechanism.

The essential requirement is a queryable terminal outcome independent of the worker session and its parent. C provides that. E′ provides delivery to an inbox, but not a suitable job-status interface. A does not meet the requirement; D records the wrong lifecycle boundary.

However, **C as specified is too weak**: worker names are not execution identities, seven days is not an unconditional durability guarantee, and a local result path is not a remotely accessible result.

B is a reasonable incremental implementation if it establishes the same identity, schema, and query semantics. A new CLI command alone adds little; the value is a stable completion contract. TermLink should own persistence and retrieval semantics, while AEF owns acceptance-card and handback content.

## 2. Risks

- **Multiple sources of truth.** Treat the hub record as the authoritative published outcome. Local files are recovery evidence; session events and sidecar messages are hints. Distinguish “worker execution finished” from “completion successfully published.” A publication failure must not overwrite the worker’s exit code.
- **Ordering and retries.** Commit the record before sending any optional notification. Include its execution ID in the notification. Make publication idempotent, and reject conflicting terminal outcomes for the same execution. A crash between commit and notification still loses the push unless a durable retry mechanism covers that gap.
- **Crashes before announcing.** `run.sh` cannot report its own host failure or every abrupt termination. An absent record means **unknown**, not running. Lease expiry or supervisor reconciliation should establish a separate lost/abandoned state; it must not invent successful completion.
- **Dispatcher gone.** Lookup remains useful without a parent receiver. But someone still needs to consume the result. “Execution finished,” “result fetched,” and “worker available” are separate facts. The assignment service must also verify worker liveness and capacity before assigning again.
- **Retention.** Seven days can silently turn a known outcome into “never seen.” Define expiry behavior explicitly. Topic-wide message limits can also evict records unevenly. Retain compact terminal summaries longer, or return an explicit expired state backed by retained identity metadata.
- **Identity and spoofing.** A name is neither unique nor authorization. Bind publication rights to the dispatched execution and its authenticated identity or scoped capability. Validate the assignment attempt or lease generation so a stale worker cannot complete reassigned work.
- **Hub and artifact location.** “Any host” means any host addressing the correct hub. Persist that hub address with the dispatch. `/tmp/.../result.md` is only a local locator; remote readers need an accessible artifact reference, inline content, or an explicit unavailable result.

## 3. What we are missing

Job queues distinguish task identity, execution attempts, acknowledgement, and lease expiry. CI systems distinguish a run’s terminal status from its artifacts and notifications. Actor frameworks distinguish process termination from successful application work. Those distinctions matter more here than the notification transport.

The missing design is the **execution lifecycle**: who creates the execution, who may terminate it, how stale attempts are fenced, and what happens when no terminal announcement arrives.

A simpler rollout is B with a documented schema and a thin wait wrapper, evolving into C when multiple clients need the contract. Reuse existing topic persistence rather than adding another completion store.

Also specify a race-free wait: reading “no current value” and then subscribing must not miss a completion between those operations. Return transport failure or timeout distinctly from a worker failure.

## 4. The one thing you would change

**Key completion by a dispatch-created, immutable execution ID—not worker name.**

Use `worker done --execution ID` and `worker wait --execution ID`; retain the name as display metadata. Persist the hub address and attempt association at dispatch.

This prevents a late waiter from receiving yesterday’s outcome for a reused name and provides the foundation for authorization, deduplication, retries, and assignment reconciliation.
