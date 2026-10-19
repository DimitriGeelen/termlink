# External design review: how should a finished AI-agent worker announce that it is done?

You are an independent systems designer. Be critical; disagreeing with the recommendation is
useful. Answer in English, at most ~700 words, markdown, with the sections listed at the end.
Answer from this text only; do not run tools.

## Context

**TermLink** is a Rust substrate: one hub per Linux host (TLS JSON-RPC, SQLite + append-only
topic logs) through which AI coding agents discover each other, exchange durable messages on
topics, claim units of work (leases with atomic transfer), and control terminal sessions. Hubs do
not federate; a client talks to whichever hub it addresses. Topic posts may carry a
`metadata.cv_key`; the hub indexes the latest post per key, and a late subscriber can ask for the
current value per key instead of replaying the topic. Topics can be bounded (`days:N`,
`messages:N`).

**AEF** is the governance framework vendored into ~30 projects. `fw termlink dispatch` starts a
worker agent (`claude -p`, etc.) inside its own TermLink session, running a generated `run.sh`.
AEF also ships a per-agent **sidecar**: an always-on process per agent identity that sends and
receives point-to-point messages through the hub (the recipient's inbox is a hub topic), with
receipts (RECEIVED / HANDED_OVER / REPLIED), a retry ladder until confirmed, cross-host delivery
via a hub-qualified address, and surfacing: a message is shown to the receiving agent at its
next prompt.

**Direction:** orchestration. One orchestrator hands work to parallel workers, possibly on other
hosts. A separate design (an "assignment sidecar") will allocate task IDs centrally and assign
work by availability, so it needs to know when a worker is free again.

## Facts

1. When a worker finishes, `run.sh` writes `exit_code` and `result.md` into a local directory
   `/tmp/tl-dispatch/<name>/` and emits `worker.done {exit_code, result}` on the worker's own
   session event bus. That event disappears when the session ends.
2. `fw termlink wait --name N` (on the dispatching host) tries the session event, then polls the
   local `exit_code` file, so a late waiter on the same host still finds it. Nothing durable
   exists on the hub; a waiter on another host, an observer (a web dashboard), or the assignment
   service cannot learn after the fact that worker N finished.
3. A user project reported: waiters often start late (after context compaction or in a new
   session), and one waiter matched itself in `pgrep` and never returned. They want "a record we
   can query after the fact".
4. The dispatching agent's sidecar address is (as far as we know) not recorded in the dispatch,
   so the worker does not know whom to notify. AEF has been asked whether a sidecar-based
   completion notice exists in its backlog; answer pending (see note at the end if present).
5. A structured completion record format already exists elsewhere in the project
   (`role-handback/1`: output hash, input hashes, one completion item per acceptance card).

## Options

- **A. Build nothing**: the local file + an AEF upgrade (sessions now end with the worker).
- **B. AEF-only record**: `run.sh` also posts the done record to a hub topic with a plain
  `channel post`; the format is owned by AEF.
- **C. TermLink-owned record**: new verbs `termlink worker done --name N --exit X --result P`
  (posts to a bounded hub topic `worker:done`, `cv_key = N`, kept 7 days) and
  `termlink worker wait --name N` (reads the current value for N, else long-polls; exits with the
  worker's exit code; works from any host reaching the hub, also after the worker is gone).
  AEF's `run.sh` calls `worker done`; `fw termlink wait` uses `worker wait` when available.
- **D. Hub-generic**: the hub durably records every session exit with the child's exit status;
  no AEF change, but no result path, and the status is the shell's, not necessarily the worker's.
- **E. Sidecar notification + the C record**: `run.sh` ends with a sidecar message to the
  dispatching agent ("worker N finished, exit X, result P") as push (arrives at its next prompt,
  receipts, retries, cross-host), plus the C record for lookup by observers nobody addresses.
  Requires recording the dispatcher's address at dispatch time.
- **E′. Sidecar notification only**: as E without the C record.

Current recommendation: E (push to the dispatcher via sidecar, plus the C record for lookup),
because push and lookup are different jobs: the dispatcher should not have to wait at all, and
observers/the assignment service need a record keyed by worker name.

## Note: AEF's answer (arrived before this review)

1. No AEF task exists for sidecar-delivered completion or for recording the dispatcher's
   identity. Related: a captured task for "every dispatch gets a waker, plus a signal when a
   worker is done but its result is unfetched for >15 min"; a done task making run.sh write
   exit_code before exiting; a done task making dispatched workers non-endpoints (they cannot
   receive sidecar messages).
2. Constraints on a sidecar notice: the sidecar id is per PROJECT, not per session, so a notice
   surfaces in whichever session holds the project receiver (right only with one parent
   session); a short-lived dispatcher (cron, one-shot script) has no receiver, so the notice sits
   unread; the harness already re-invokes the parent when a background wait exits.
3. AEF recommends C as the primitive (independent of who dispatched) and the sidecar notice as an
   optional layer once the dispatch records the dispatcher.

## Sections to answer with

1. **Verdict**: which option (or a different one) and the main reason.
2. **Risks** of your chosen option (double sources of truth, ordering between the push and the
   record, worker crashes before announcing, dispatcher gone, retention, identity/spoofing: can a
   worker announce for another?).
3. **What we are missing**: prior art (job queues, CI systems, actor frameworks), simpler designs.
4. **The one thing you would change** in the record or verbs.
