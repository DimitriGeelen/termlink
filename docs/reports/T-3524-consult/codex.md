## Verdict

Option A is directionally right, but its partition mapping is not justified by the supplied model. Session absence and loss of contact are different observations. Conformance should distinguish backend behavior, test-adapter capabilities, and downstream consumer obligations rather than turn unsupported scenarios into n/a by agreement alone.

## Per-case

| Case | Responsibility and placement |
|---|---|
| **CC-08: restart / wait** | The bounded-wait behavior belongs at the backend interface or its conforming client adapter; a native blocking hub operation is not necessarily required. Polling `dispatch.get` could implement it if the vector permits that implementation. Restart recovery and durable state remain hub obligations. |
| **CC-09: partition / heal** | Fault injection belongs in the harness; safe behavior during loss of authority contact belongs to the backend wherever that topology is supported. The current absence mapping does not establish that behavior. If a remote worker/authority topology genuinely cannot occur, negotiate a scoped n/a with explicit topology limits. |
| **CC-10: parent loss** | A clear backend obligation: durable routing of orphaned questions **and bounces**, plus authorized ownership transfer. Existing notices without routing and adoption do not satisfy §4.6. |
| **CC-13: lock timeout** | Bounded, fail-closed writes are a backend obligation; `hold_lock` is harness plumbing, not a production API requirement. Existing unit tests support confidence but do not automatically satisfy the shared vector. Exercise the real lock through the adapter. |
| **CC-19: paused consumption** | Refusal belongs primarily to the consuming evaluation layer. The backend must expose an unambiguous paused status and must not represent it as finished work. Backend-only n/a is reasonable if consumer conformance remains tested elsewhere. |
| **CC-22: attested consumption** | Likewise, the evaluator must refuse attested completion. The hub must preserve and expose attestation and evidence metadata. A generic `done` state alone must not imply eligibility for downstream use. |

## Risks

**Partition is not disappearance.** A disconnected worker can remain alive, continue side effects, and later submit completion. Two absent sweeps plus grace reduce premature loss declarations; they do not prove termination. The proposed mapping is sound only if registration absence faithfully represents the contract’s intended observation and recovery preserves fencing. Neither condition is established here. Test a running, disconnected worker, loss classification, replacement if supported, healing, and stale completion. An undecidable observation must not silently become evidence of absence.

**Wait has registration races.** Reading state and then subscribing can miss a transition. Use a durable revision or equivalent atomic subscription/check mechanism. Specify whether callers receive current state or every transition, bound resource use, support cancellation, and avoid holding SQLite write transactions while waiting. Restart must produce a defined reconnect/error outcome without restarting the caller’s overall deadline indefinitely.

**Adopt changes authority.** Concurrent successors must not both win. Atomically compare the expected owner/generation, assign the successor, advance fencing, and reject the old owner’s subsequent writes. Session absence alone must not authorize any agent to seize ownership. Permit the named successor under authenticated authorization, or an explicitly authorized operator path; define how that successor is designated.

Questions and bounces can race with adoption. Preserve stable identities and routing so neither disappears between dispatcher and operator queues. Adoption must not retry bounced answers or reset their deadlines.

## What I would change

Expose raw attempt records for inspection, but make any API that promises **finished work** enforce a centrally defined eligibility predicate. If a read-only `finished_work` fact is added, define it normatively and return it consistently with the record. Consumers still need enforcement; a helpful flag cannot protect against consumers ignoring it.

Expand A’s orphan notice work to cover bounces explicitly. Agree with AEF on vector ownership and supported topology before reporting reclassified cases as conformant; implementation can proceed meanwhile.

## Recommendation

**Modified A:** implement wait semantics, orphan routing/adoption, and real-lock harness coverage; retain consumer refusal tests in AEF; leave partition conformance unresolved until its topology and recovery semantics are demonstrated.

The most important reason: **loss of visibility must not be mistaken for loss of execution**, especially when ownership or replacement work can advance.