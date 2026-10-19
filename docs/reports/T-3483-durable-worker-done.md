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

## 3. Dialogue log

1. 2026-10-19, 1409 support-round suggestion; converted from build to inception after the facts
   above; IW-1 answered by 1409.
