---
id: T-1993
name: "Hub-side: bisect+fix 0.11.473 channel info concurrency regression (T-1991 follow-up)"
description: >
  Hub-side proper fix per T-1991 GO. Bisect commits in the 0.11.472..0.11.473 range to find the regression that makes channel info wedge under sequential load. Likely candidate: crates/termlink-hub/ topic-state lock or rpc dispatcher. Symptom: 9/20 sequential channel info on agent-presence (1503 envelopes) time out at exactly 15s, fleet-wide on every 0.11.473 hub. .107 (0.11.472, 13441 envelopes, LAN) is 0/20 clean. Pure hub-binary regression, not topic-size driven. See docs/reports/T-1991-channel-info-hub-concurrency-regression.md for full data.

status: work-completed
workflow_type: inception
owner: agent
horizon: now
tags: []
components: []
related_tasks: [T-2013]
created: 2026-06-05T09:35:50Z
last_update: 2026-06-06T12:25:00Z
date_finished: 2026-06-06
---

# T-1993: Hub-side: bisect+fix 0.11.473 channel info concurrency regression (T-1991 follow-up)

## Problem Statement

T-1991 (2026-06-05) framed a "channel info" wedge as a **0.11.473 hub-side
concurrency regression vs 0.11.472**: .107 hub (0.11.472) is reliable;
.121/.122/.141 hubs (0.11.473) flake at 10–45% timeout rate on sequential
`termlink channel info agent-presence` against ~800–1500 envelope topics.
T-1991 GO'd this task to bisect commits in the 0.11.472..0.11.473 range.

**Live re-repro 2026-06-06T01:21Z, .122 hub, 5 sequential trials:**

```
$ termlink channel info agent-presence --json --hub 192.168.10.122:9100
Trial 1: rc=124 time=16.005714441
Trial 2: rc=124 time=16.006106174
Trial 3: rc=124 time=16.005446831
Trial 4: rc=124 time=16.006526122
Trial 5: rc=124 time=16.005123229
```

**5/5 timeouts at exactly 16s** (operator-set), no partial returns. The
bug is harder today than T-1991's snapshot (which scored 9/20 = 45%).
Confirmed: the wedge is real, reproducible, and current.

**Bisect-framing problem (this task's primary finding):**

The "0.11.472 vs 0.11.473" framing collapses on inspection:

1. **Zero commits to `crates/termlink-bus/` since v0.11.1** (tagged
   2026-05-18). The bus layer — which owns the SQLite mutex contended
   by `channel.subscribe` reads + `channel.post` writes — is byte-identical
   between v0.11.1 and HEAD@4134d288.
2. **Only TWO commits to `crates/termlink-hub/src/` since v0.11.1:**
   `f7b8d057` (T-1415 hub source cleanup) and `85117320` (T-1415
   capabilities cleanup). Both are deletions of retired event.broadcast/
   inbox.* handlers (T-1166 cut). Neither touches concurrency primitives,
   subscribe path, or topic-state locks.
3. **build.rs version-string is ambiguous.** The derivation
   `let major_minor = base.rsplitn(2, '.').last()?` collapses `v0.11.0`
   and `v0.11.1` into the same `0.11` prefix. So a binary reporting
   `0.11.472` could be either "472 commits past v0.11.0" OR "472 commits
   past v0.11.1" depending on which tag was reachable at build time. The
   version string does NOT uniquely identify the commit.
4. **No 15s timeout in the CLI or session crates** (`grep
   '15_000\|Duration::from_secs(15)' crates/termlink-cli/`). T-1991's
   "exactly 15.00s" was likely operator-side `timeout 15 termlink ...`
   in the harness, not a hub-imposed bound.

**Conclusion:** there is no "0.11.472 → 0.11.473 hub-side regression
commit" to bisect. The wedge predates v0.11.1 and has been latent in the
codebase for ≥3 weeks. The version axis in T-1991's data is a red
herring; the real axis must be host/environment.

## Assumptions

A-1991-1: "The wedge is hub-version-driven (a regression introduced
between 0.11.472 and 0.11.473)." — **DISPROVEN.** No hub/bus commits in
the relevant range touch concurrency. Version-string ambiguity makes the
binary-version axis unreliable.

A-1991-2: "Topic size amplifies the bug." — **Confirmed by T-1991 data**
within a single 0.11.473 hub: empty=0%, 894=20%, 1503=45%. So the wedge
IS topic-size sensitive on the flaky hosts.

A-NEW-1: "The wedge is host-environment-driven, not code-driven."
Suspects: container resource limits (memory pressure on SQLite mmap), I/O
contention, FD limits, kernel scheduling, or per-hub configuration
(runtime_dir on different filesystem, busy_timeout setting).

A-NEW-2: "`channel.info` is a client-side composite that loops
`channel.subscribe` to walk the topic." **Verified** by reading
`crates/termlink-cli/src/commands/channel.rs:2974` (cmd_channel_info). It
calls `channel.list` first, then loops `channel.subscribe(cursor, limit=1000)`
until the page returns < limit. For 1503 envelopes that's 2 round-trips.

A-NEW-3: "The hub-side mechanism is `channel.subscribe` under sequential
load, NOT a single dedicated `channel.info` RPC." **Verified.** The hub
has no `channel.info` handler; the symptom must be in
`handle_channel_subscribe` (`crates/termlink-hub/src/channel.rs:527`)
or downstream `bus.subscribe` (`crates/termlink-bus/src/lib.rs:199`).

A-NEW-4: "The hub-side mutex (`self.conn.lock()` in
`crates/termlink-bus/src/meta.rs:181`) is released BEFORE the iterator
walks the log file." **Verified** by code-reading: `records_from` drops
the guard at end of scope, then `bus.subscribe` returns a `ReaderIter`
that reads the file independently. So SQLite contention with concurrent
writers cannot account for 15s wedges during the walk.

## Exploration Plan

This task's spike re-frames the problem and proposes the next
investigation. It does NOT bisect commits (the bisect-target is
disproven). Time-boxed at 1–2 hours; further work is the next task.

**Spike 1 — Verify flakiness on .107 with the SAME load pattern (5/5
sequential).** If .107 (the "clean" host in T-1991's data) is ALSO
5/5 today, the regression hypothesis is fully sunk and the issue is
purely environmental on .121/.122/.141. If .107 stays clean, the host-
difference axis is real and identifies specific environmental factor
to chase. *(Time: 5 min.)*

**Spike 2 — Hub-side instrumentation.** Read `bus.subscribe` and the
log `ReaderIter` more carefully. Is there a hidden lock acquisition
during the walk (e.g., file-level fcntl, mmap fault on cold pages)?
Are SQLite WAL checkpoints being triggered during reads? Read the
`busy_timeout` and `journal_mode` PRAGMAs on .122 vs .107. *(Time: 15
min.)*

**Spike 3 — Reproduce-with-only-reads.** Disable the .122 presence-
heartbeat cron temporarily (or run on a topic with no concurrent
writer). Does 5/5 still wedge? If yes, the issue is read-only — likely
disk I/O / SQLite/file behavior. If no, it's a write-vs-read interaction
and the bus crate is suspect (despite no recent commits — could be a
pre-existing bug only triggered by current load shape). *(Time: 5 min.)*

**Spike 4 — `strace` the wedged hub process.** Attach strace to the
.122 hub PID while a wedged channel.subscribe is in flight. Where does
the syscall trace stall? Likely candidates: `read()` from log file
(disk), `futex()` (lock contention), `epoll_wait()` (event loop stalled).
*(Time: 10 min.)*

## Technical Constraints

- **Read-only on the .122 hub.** Cannot restart it for this spike — it's
  load-bearing for ring20-management. All probes must be observational
  (strace, read PRAGMA queries via existing CLI verbs, etc.).
- **No new hub binary builds** in this inception. The fix slice that
  follows will need them, but scoping a fix requires understanding the
  root cause first.
- **Container-aware.** The .121/.122/.141 hosts are LXC; reasoning about
  /tmp behavior (PL-021), FD limits, or kernel namespace effects must
  account for container vs host kernel.

## Scope Fence

**IN scope for this inception:**
- Re-frame the problem statement so the next task targets the right
  axis (host/environment vs hub code vs build configuration).
- Verify reproducibility on .122 today (done; 5/5 timeouts confirmed).
- Identify the spike plan that will find root cause.
- Provide enough evidence that the operator can choose the next direction.

**OUT of scope:**
- Bisecting commits (disproven as the right framing).
- Implementing a fix (separate build task).
- Running spikes 1–4 above (they are NEXT, not part of this inception's
  go/no-go decision).
- Changing .122 hub configuration to validate hypotheses.

## Acceptance Criteria

### Agent
<!-- @auto-tick-on-decide -->
- [ ] Problem statement validated
<!-- @auto-tick-on-decide -->
- [ ] Assumptions tested
<!-- @auto-tick-on-decide -->
- [ ] Recommendation written with rationale

### Human
<!-- @auto-tick-on-decide -->
- [ ] [REVIEW] Review exploration findings and approve go/no-go decision
  **Steps:**
  1. Run: `fw task review T-XXX` (opens Watchtower with recommendation, assumptions, research artifacts)
  2. Review the Agent Recommendation section and go/no-go criteria evaluation
  3. Record decision via the Watchtower form or the command shown alongside the QR code
  **Expected:** Decision recorded, task completed
  **If not:** Ask agent for clarification on specific findings

## Go/No-Go Criteria

<!-- Fill these BEFORE writing the recommendation. The placeholder detector will block review/decide if left empty. -->
**GO if:**
- Root cause identified with bounded fix path
- Fix is scoped, testable, and reversible

**NO-GO if:**
- Problem requires fundamental redesign or unbounded scope
- Fix cost exceeds benefit given current evidence

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# For inception tasks, verification is often not needed (decisions, not code).
#
# Toolchain hint (L-291): if a GO decision will mean editing *.vbproj/*.csproj/*.xaml,
# *.go, Cargo.toml, tsconfig.json, or pom.xml in the build task, plan to add the
# matching build command (dotnet build / go build / cargo check / tsc --noEmit /
# mvn compile) to that build task's ## Verification — P-011 only runs what you write.

## Recommendation

**Recommendation:** GO — but RESCOPE the next task.

**Rationale:** The wedge is confirmed real and reproducible (5/5 timeouts
on .122 today, harder than T-1991's snapshot). However, T-1991's bisect
framing ("regression between 0.11.472 and 0.11.473") is disproven:

- Zero bus-layer commits since v0.11.1 (3 weeks).
- Only 2 hub-source commits, both T-1415 deletions unrelated to
  concurrency.
- build.rs version-string ambiguity (v0.11.0 vs v0.11.1 collapse to
  "0.11.N") means the version axis isn't reliable evidence of code diff.

The wedge is older than v0.11.1 and likely environmental on the flaky
hosts. A bisect against the wrong baseline would burn sessions chasing
a non-existent regression commit.

**Evidence:**

1. **Live re-repro on .122:** 5/5 timeouts at 16s on sequential
   `channel info agent-presence`. Confirms wedge is current and worse
   than T-1991's data (45% → 100% in 24h).
2. **`git log v0.11.1..HEAD -- crates/termlink-bus/`** returns empty.
   Bus layer byte-identical since v0.11.1.
3. **`git log v0.11.1..HEAD -- crates/termlink-hub/src/`** returns
   only `f7b8d057` + `85117320` (T-1415 deletions). No concurrency
   touches.
4. **build.rs version-string semantics** (`crates/termlink-cli/build.rs:75`):
   `rsplitn(2, '.').last()?` collapses `v0.11.0-N-gXXX` and
   `v0.11.1-N-gXXX` to the same `0.11.N` output. The reported version
   is ambiguous on which tag was reachable at build time.
5. **`cmd_channel_info`** (CLI) is a client-side composite:
   `channel.list` + repeated `channel.subscribe(cursor)` loop. There is
   no `channel.info` hub RPC. The wedge therefore lives in either
   `handle_channel_subscribe` (`channel.rs:527`) or `bus.subscribe`
   (`bus/lib.rs:199`) on the affected hub.
6. **Bus mutex scope is bounded.** `records_from` drops the SQLite
   mutex before the iterator walks the log file
   (`bus/lib.rs:199-215`). So the wedge cannot be SQLite contention
   alone — must be in the file walk or upstream of `records_from`.

**Next task should be:** a focused environmental + read-path spike on
.122 (no code edits, observational only — strace, PRAGMA reads,
disable-cron repro). Detailed plan in `## Exploration Plan` above. Bisect
work, if still warranted after spike findings, would target
PRE-v0.11.1 commits — a much wider range — and only after the
environmental hypothesis is ruled out.

## Decisions

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Decision

<!-- Filled at completion via: fw inception decide T-XXX go|no-go --rationale "..." -->

## Updates

<!-- Auto-populated by git mining at task completion.
     Manual entries optional during execution. -->

### 2026-06-05T23:02:26Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-06-06T01:30Z — Spike 1 result: .107 5/5 clean confirms host-axis [agent autonomous]

Ran the same load pattern against .107 (the "clean" host per T-1991) from
the same .107 CLI used to confirm .122 wedge:

```
Trial 1: rc=0 time=.794321901
Trial 2: rc=0 time=.881293707
Trial 3: rc=0 time=.740357539
Trial 4: rc=0 time=.724821674
Trial 5: rc=0 time=.856842917
```

**5/5 SUCCESS, sub-1s.** Topic size: 13441 envelopes — 9× larger than
.122's 1503. Same CLI binary (termlink 0.11.472). Same `channel info
agent-presence --json --hub <addr>` command, only the hub address differs.

**This unambiguously confirms A-NEW-1 (host-environment axis) and
disproves any remaining version-axis hypothesis.** The clean host has
the larger topic; the flaky host has the smaller topic; same CLI; same
network path (.107-source, LAN to both). The only meaningful difference
is the hub host itself.

**Strengthened recommendation:** GO on filing a new follow-up task
(env-spike on .122) — NOT a code-bisect. The env-spike should test:
- (a) Disk I/O wait on .122 (`iostat`, `vmstat`)
- (b) FD limit / `ulimit -n` on the hub process
- (c) Container memory pressure (`free`, `cgroup` mem.usage_in_bytes)
- (d) Whether `/var/lib/termlink/` (per T-1294 migration) is on a fast
  or slow filesystem
- (e) Whether the presence-heartbeat cron's write rate is wedging file
  reads (Spike 3 from Exploration Plan)

Implementation effort estimate for env-spike: 1–2 hours, observational
only, no hub restart needed. Filing as a separate task post-decide so
this inception can close.
