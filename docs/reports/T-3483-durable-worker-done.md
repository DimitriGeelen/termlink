# T-3483 — Durable "worker finished" record (inception research)

**Status:** shaping; options and recommendation below, awaiting the operator's ruling.

## 1. Facts (measured 2026-10-19)

1. AEF's worker `run.sh` (written by `fw termlink dispatch`, e.g. `/tmp/tl-dispatch/<name>/run.sh`
   l.427) ends with `termlink event emit <worker> worker.done -p {"exit_code":N,"result":"…/result.md"}`.
   That event lives on the worker's own session event bus and disappears with the session.
2. Since AEF T-4152 the worker session ends when the worker ends. 1409 runs AEF 1.8.10 (1.8.12
   pending), which explains its "session stayed open for hours".
3. `fw termlink wait --name N` (`agents/termlink/termlink.sh` l.1724–1770) first tries
   `termlink event wait N --topic worker.done`, gated on `termlink list | grep -q N`, then polls
   `/tmp/tl-dispatch/N/exit_code`. That file is already a durable, late-waiter-safe record, but
   only on the dispatching host and only for whoever knows the directory.
   Side finding: before T-3481 `termlink list` truncated names to 15 chars, so for longer worker
   names the grep never matched and the event wait was always skipped. Fixed today by T-3481.
4. 1409 (msg 0712b298): waiters often start late (after compaction or in a new session); one
   waiter matched itself in `pgrep` and never returned; a record "we can query after the fact"
   would replace both their end-signal file and that wait.
5. TermLink already has the primitive for "late joiner reads the current state": posts carrying
   `metadata.cv_key` are indexed per topic, and `channel subscribe --include-current-value`
   returns the latest value per key (T-2027/T-2103..T-2106). Retention can be bounded by
   `days:N` or `messages:N` (T-2244).
6. T-3476 (assignment sidecar) needs a reliable "worker N is free again" signal, from any host.

## 2. Options

- **A. No build.** Point 1409 at `fw termlink wait --name` (file-based, works late on the same
  host) and the 1.8.12 upgrade; leave cross-host and observers without a record.
- **B. AEF-only.** `run.sh` additionally posts the done record to a hub topic with a plain
  `termlink channel post`; record shape owned by AEF (upstream change, G-062).
- **C. TermLink owns the record, AEF calls it.** TermLink adds two verbs:
  `termlink worker done --name N --exit X [--result P] [--task T]` (posts to a bounded hub topic
  `worker:done`, `cv_key = N`, retention `days:7`) and `termlink worker wait --name N
  [--timeout S]` (reads the current value for N first, then long-polls; exits with the worker's
  exit code; works from any host that can reach the hub, and for a waiter that starts after the
  worker is gone). AEF's `run.sh` calls `worker done` (one line, upstream), and `fw termlink wait`
  prefers `worker wait` when the binary has it. T-3476 reads the same topic.
- **D. Hub-generic.** The hub records every session exit (it already emits `session.exited`)
  durably with the child's exit status; no AEF change, but no result path, and the exit status
  is the shell's, not necessarily the worker's.

- **E. Sidecar notification + durable record.** `run.sh` ends with `fw sidecar send` to the
  dispatching agent ("worker N finished, exit X, result P"): push, arrives at the dispatcher's
  next prompt, durable inbox (a hub topic), receipts, retry ladder, cross-host via a
  hub-qualified address. Plus the option-C record for lookup by observers (Watchtower, T-3476,
  a second orchestrator) that nobody addresses. Needs the dispatcher's address recorded at
  dispatch time so the worker knows whom to notify.
- **E′. Sidecar only.** As E without the TermLink record; observers learn nothing unless they
  are addressed too.

Correction (operator question, 2026-10-19): an earlier draft said the sidecar "cannot reach an
orchestrator". Wrong: it reaches any addressed agent, also cross-host (we reach ring20 on .122
that way). The real limit is that it reaches only the addressees.

Backlog check (2026-10-19): no termlink task for sidecar-delivered worker completion. Nearest:
arc-011 `role-handback/1` (T-3348/T-3351/T-3352/T-3357), a JSON record a role step returns; it is
a format, not a delivery path, and could be the payload of E. Asked AEF (conv
`termlink-worker-done-sidecar`) whether their backlog has it.

**AEF answer (2026-10-19, msg d098bfb1):**
1. No AEF task (built, planned or rejected) for sidecar-delivered worker completion, nor for
   recording the dispatcher's identity in the dispatch. Nearest: AEF T-4027 (captured; 1409's
   pickup T-1815: every dispatch gets a waker, plus a "worker done, result unfetched > 15 min"
   signal, the same problem from the waiting side); T-4152 (done: run.sh writes exit_code before
   it exits; `fw termlink wait/status` read that file); T-4423 (done today: workers run with
   `FW_DISPATCHED_WORKER=1` and are not sidecar endpoints).
2. Constraints on a sidecar notice: (a) the dispatch records task, name and wdir but not the
   dispatching session; the sidecar id is per PROJECT, so a notice surfaces in whichever session
   holds the project receiver, right only with a single parent session; (b) a short-lived
   dispatcher (cron, one-shot script) has no receiver, so the inbox is durable but unread;
   (c) the harness already re-invokes the parent when a background wait exits.
3. AEF recommends: the TermLink-owned durable `worker:done` record (cv_key = worker name,
   `termlink worker wait`) as the primitive, because it does not depend on who dispatched; a
   sidecar notice as an optional layer once the dispatch records the dispatcher's agent (later
   its session). AEF will file its side once T-3483 settles the record's shape, and wants the
   chosen topic/cv_key.

## 3. Dialogue log

1. 2026-10-19, 1409 support-round suggestion; converted from build to inception after the facts
   above; IW-1 answered by 1409.
