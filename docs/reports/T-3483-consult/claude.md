# Review: how a finished worker should announce it is done

## 1. Verdict

**Build C now and put the sidecar push off.** I'd change C in two ways (keying and crash coverage, below), so read it as "C+".

Your reason for E is correct: push and lookup are different jobs. But AEF's answer removes the case for doing the push now:

- **The receiver is wrong.** Sidecar ids are per project, not per session, so the notice can land in the wrong session.
- **Workers can't send it.** Dispatched workers are deliberately not endpoints.
- **Short-lived dispatchers can't receive it.** A cron job or one-shot script has no receiver, so the notice sits unread.
- **Push already exists.** The harness re-invokes the parent when a background wait exits. A background `termlink worker wait` is that push, built on the record instead of beside it.

E gives you two delivery paths that can disagree, and it depends on recording the dispatcher's identity, which nobody has designed yet. Add the sidecar notice later as a consumer of the record (something that watches `worker:done` and forwards), not as a second thing the worker emits.

## 2. Risks (of C as currently specified)

- **A crashed worker never announces.** `run.sh` can `trap EXIT`, but kill -9, OOM, a host reboot or a lost hub connection all skip it. `worker wait` then long-polls forever: the pgrep hang again, just moved. The runtime has to announce death, not the worker. The hub already knows when a session goes away, so it should write a terminal `lost` record if a session tied to a dispatch ends without a `done`. That is D used as a fallback, not as the main mechanism.
- **Name reuse returns a stale result.** With `cv_key = N`, a re-dispatch of `N` makes a late waiter return the *previous* run's exit code instantly. That is worse than a hang.
- **"Unknown" looks like "not done yet".** After 7 days, or for a mistyped name, the record is absent and `wait` long-polls forever. `wait` needs a distinct "no such dispatch" answer and a timeout.
- **Exit-code mirroring is ambiguous.** If the worker exits 1, does that mean the worker failed or `wait` failed? Reserve a code range for wait errors, or have `wait` output JSON with `--exit-with-worker` as an opt-in.
- **Spoofing.** Anyone with hub access can post `cv_key = N`. The `done` verb should require a per-dispatch capability token minted at dispatch and given only to that session. Alternatively, the hub checks that the caller *is* the session bound to the dispatch.
- **Retention.** Use `days:` only. A `messages:N` bound lets a burst of dispatches evict results that a waiter still needs.
- **Two sources of truth.** Make the hub record canonical and the local directory a cache. Order the steps as: write the local file, then post (idempotent upsert), retrying on failure. `fw termlink wait` checks the local file only when the hub is unreachable.
- **`--result P` is a local path.** It means nothing on another host. The record should carry a hash and a short summary inline, plus a way to fetch the artefact.
- **No federation.** "Any host reaching the hub" only works if the waiter knows *which* hub, so the dispatch handle must include it.

## 3. What we're missing

- **Prior art agrees on one point: the supervisor reports the terminal state, not the job.** Kubernetes Job status is set by the controller, not the pod. CI runners use heartbeats and mark jobs "lost". Erlang/Akka monitors deliver `DOWN` from the runtime. Celery and RQ use a result backend keyed by a unique task id, with a TTL and states PENDING/STARTED/SUCCESS/FAILURE.
- **You already have leases.** Model a dispatch as a claim on a work unit. Completion is a release carrying the outcome; a crash is lease expiry. The assignment sidecar's question, "is this worker free?", is answered by the claim state with no new mechanism. This may be simpler than a new topic, and it brings atomic transfer for retries.
- **Reuse `role-handback/1`** as the done payload instead of inventing `{exit_code, result}`. An exit code is the shell's opinion; handback items are the work's opinion. The assignment service needs both.
- **Record the start, not just the end.** Write the record at dispatch time. Then `wait` and dashboards can tell unknown, running, done and lost apart.

## 4. The one thing I'd change

**Key the record by a unique dispatch id minted at dispatch time, not by worker name.** For example, a ULID, with the name kept as a label. Write it first as `dispatched`, then move it to `done` or `lost`, keeping one `cv_key` per dispatch id.

This one change:

- removes stale results from name reuse,
- gives `wait` a clean way to say "unknown",
- gives the hub something to mark `lost`,
- gives the capability token something to bind to,
- gives the later sidecar layer and the assignment service a stable handle to pass around.

`termlink worker wait --name N` can stay as a convenience that resolves to the latest dispatch id for N.
