## Verdict

Option A gets the core hub mechanisms right (`wait`, `adopt` + orphaned-question notice, and SQLite lock test plumbing), **but its proposed shortcuts for CC-09 (Partition) and CC-19/CC-22 (`consume`) are flawed.** 

First, equating network partition with an absent session registration misses the **heal** path—the exact scenario where a zombie worker reconnects after its attempt was swept `lost` or re-dispatched, testing whether your fencing tokens actually prevent split-brain writes. Second, pushing `consume` refusals entirely onto external evaluation consumers while offering only an optional boolean flag invites accidental ingestion of unverified (`attested`) or incomplete (`paused`) work via raw `dispatch.get` reads.

---

## Per-case

1. **CC-08 (`wait`):** **Backend obligation.** Blocking or long-polling until a state transition or bound is a core concurrency primitive of a coordination hub. *Where:* Hub RPC (`dispatch.wait`), backed by an internal condvar/broadcast channel over state commits plus a post-subscribe SQLite read to close the pre-wait race window.
2. **CC-09 (`partition` / `heal`):** **Adapter fault-injection obligation (testing backend fencing).** A single-host hub doesn't need a multi-host network model, but it *must* prove what happens when a silenced worker resumes writing. *Where:* Conformance adapter. Simulate `partition` by hiding the session from the sweep observer and dropping/delaying worker-originated RPCs in the adapter; simulate `heal` by restoring visibility and replaying the worker's late `done` write to verify that the backend's CAS/generation fencing rejects it once swept `lost`.
3. **CC-10 (`inspect operator_queue` / `adopt`):** **Backend obligation.** The worker contract (§4.6) explicitly governs parent-session loss, orphan routing, and ownership transfer. Because the hub owns session liveness, durable message routing, and supervisor generation counters, this cannot live solely in the client. *Where:* Hub state machine.
4. **CC-13 (`hold_lock`):** **Test-harness / adapter obligation.** Fail-closed lock timeout is a backend invariant, and the hub's `BEGIN IMMEDIATE` (30s bound) already enforces it. *Where:* Conformance adapter—expose a test hook that opens a second SQLite connection and holds an immediate transaction (lowering `busy_timeout` in conformance configs so tests don't take 30s).
5. **CC-19 (`consume` paused):** **Shared obligation (Backend gate + Evaluation consumer).** While downstream tools decide *what* to evaluate, the contract defines `paused` as `not_finished_work`. *Where:* Expose an explicit backend read gate (`dispatch.consume` RPC or a strict projection) that fails closed on `paused`, rather than trusting every caller of `dispatch.get` to re-implement the check.
6. **CC-22 (`consume` attested done):** **Shared obligation (Backend gate + Evaluation consumer).** An attested `done` is an operator override to close accounting loops, not a valid deliverable. *Where:* Same backend `dispatch.consume` gate as CC-19. Because the state is literally named `done`, naive consumers reading raw state *will* mistake it for finished work unless the consumption endpoint itself refuses to yield a deliverable payload.

---

## Risks

* **`dispatch.wait` lost wakeups & hub restarts:** If a caller checks state, sees `running`, and registers a watcher just after a commit to `done`, it will hang until the bound unless registration and state check are atomic (or follow a subscribe-then-verify pattern). Furthermore, CC-08 tests **hub restart**: if the hub restarts while a client is blocked in `dispatch.wait`, an EOF/disconnect must trigger immediate client re-query, or the adapter's `wait` implementation must transparently reconnect within the remaining deadline.
* **Unauthorized `adopt` & split-brain supervisors:** If any session can call `adopt`, a rogue or misconfigured agent can hijack active dispatches. Worse, if a network hiccup makes the original dispatcher appear transiently absent, an eager successor might `adopt` while the original dispatcher is still alive. `adopt` **must** verify that the prior dispatcher session is confirmed dead (or present a pre-shared successor token) and **must** atomically increment the *supervisor generation counter* so late writes or answers from the old dispatcher are fenced out.
* **`adopt` / question routing race:** If a worker asks a question concurrently with `adopt`, the question could be stranded in the operator queue while the new supervisor waits on its session inbox. `adopt` must atomically re-bind pending unanswered questions and undelivered bounces to the new supervisor's inbox and clear the orphaned operator notice.

---

## What I would change

1. **Upgrade the optional `finished_work` fact into a fail-closed `dispatch.consume` read RPC (CC-19, CC-22).** Keep `dispatch.get` for raw inspection/UI, but require artifact/result consumers to call `dispatch.consume`, which returns an error (`not_finished_work`) if `state == paused` or `attested == true`.
2. **Implement `partition`/`heal` in the adapter instead of waiving CC-09.** Add an adapter-level interceptor that hides the worker session from sweeps during `partition` and restores it on `heal`. That tests the hub's actual CAS and state-fencing guarantees when a presumed-dead worker comes back to life and tries to report `done`.
3. **Fence `adopt` strictly.** Require the old dispatcher session to be absent (or explicitly surrendered) and increment the supervisor generation counter upon adoption.

---

## Recommendation

**Modified Option A.** Build the three proposed pieces (`dispatch.wait`, orphan notice + `adopt`, and the SQLite `hold_lock` adapter hook), **plus** implement `dispatch.consume` on the hub and simulate `partition`/`heal` in the adapter.

**Single most important reason:** A conformance suite's value is catching boundary bugs between components; waving away `consume` and `heal` skips testing the two most dangerous failure modes in the contract—zombie workers overwriting `lost` attempts after a transient drop, and downstream consumers silently accepting unverified (`attested`) work as valid output.
