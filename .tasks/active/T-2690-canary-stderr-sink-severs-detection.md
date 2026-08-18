---
id: T-2690
name: "Canary error channel is a write-only sink — an erroring canary reads HEALTHY on every operator surface (G-063 class, applied to the detection layer itself)"
description: >
  21 of 24 installed canary crontabs route the canary's stderr to a `.log.stderr`
  file that nothing reads, while the heartbeat is touched unconditionally at
  startup. A canary hitting a tooling error therefore leaves `.log` empty
  (/canaries says HEALTHY), heartbeat fresh (meta-canary says ALIVE), and its
  diagnostic in an unread sink. Close the blindness: teach `canary-status.sh` to
  read the stderr companion, stop mis-classifying the four non-cron static checks
  as STALE canaries, and reconcile the git crontab source with what is installed.

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: [governance, canary, observability, bug]
components: [scripts/canary-status.sh, scripts/check-cron-install-drift.sh, .context/cron]
related_tasks: [T-2172, T-2561, T-1723, T-2527, T-2531, T-2666, T-2672]
created: 2026-08-18T21:18:31Z
last_update: 2026-08-18T22:34:40Z
date_finished: 2026-08-18T22:34:40Z
---

# T-2690: Canary error channel is a write-only sink

## Context

The project runs 17+ cron canaries under an explicit invariant repeated throughout
CLAUDE.md: **"empty log = healthy"**. Three independent facts combine to break that
invariant across almost the whole canary fleet:

1. Every canary touches its `.heartbeat` **unconditionally at startup**, before doing
   any work (`check-dead-letter-freshness.sh:59-60` and siblings). Heartbeat freshness
   therefore proves *"cron fired"*, never *"the canary succeeded"*.
2. On a tooling error (the documented exit-2 class) a canary writes its diagnostic to
   **stderr only** — stdout stays empty. Verified:
   `TERMLINK_DEAD_LETTER_TEST_JSON=/nonexistent bash scripts/check-dead-letter-freshness.sh --quiet`
   emits nothing on stdout and `check-dead-letter: could not read queue-status (exit=1)`
   on stderr.
3. 21 of 24 **installed** crontabs redirect stderr to `.context/working/.<name>-canary.log.stderr`
   (git source says `2>&1`). Nothing in the repo reads a `.log.stderr` file —
   not `canary-status.sh` (`/canaries`), not `check-canary-aliveness.sh` (the meta-canary).

Net effect: a canary that errors **every single day forever** shows
`.log` empty → `/canaries` HEALTHY, heartbeat fresh → meta-canary ALIVE,
diagnostic → unread sink. This is precisely the G-063 "write-only sink nobody noticed"
class that T-2295 and T-2558 closed for the message path — here applied to the
detection layer itself, which is where it costs the most.

Separately, `/canaries` currently reports **4 permanent false-positive STALE entries**
(`alloc-sink`, `busy-spin`, `drain-sink`, `silent-exit`). Those four are the
**source-level static checks** which CLAUDE.md explicitly documents as
"NOT a runtime cron canary" — they have no crontab and never write a log, but they
touch a `-canary.heartbeat`, and `canary-status.sh:99-101` synthesizes a log path from
any heartbeat. `/canaries` therefore exits 1 forever, which is exactly the alarm-fatigue
that makes an operator stop reading it.

## Acceptance Criteria

### Agent
- [x] `canary-status.sh` reads the `.log.stderr` companion and classifies a canary whose
      stderr sink is non-empty **within the staleness window** as `ERRORING` (counts as
      needing attention, exit 1), so a tooling-erroring canary can no longer read HEALTHY.
- [x] `canary-status.sh` classifies a heartbeat-only entry with **no log file and no
      declaring crontab** in `.context/cron/` as `NOT_SCHEDULED` (informational,
      non-firing) instead of `STALE` — removing the 4 permanent false positives.
- [x] The `NOT_SCHEDULED` discriminator is narrow: a canary that IS declared by a crontab
      and has a stale heartbeat still fires `STALE` (fixture Case 5), so the fix cannot
      blanket-suppress a genuinely dead cron.
- [x] `--json` envelope carries the new states and a `stderr_bytes` field per canary.
- [x] The 21 drifted git-source crontabs are reconciled to the installed
      `2>> .<name>.log.stderr` convention, so `check-cron-install-drift.sh` reports
      0 drift and git honestly describes what is deployed.
- [x] `check-cron-install-drift.sh` no longer prints the word `healthy` in its summary
      when `drift_count > 0` (exit semantics unchanged — drift stays non-firing per T-2561).
- [x] A fixture test `tests/canary-status-fixtures.sh` proves the two new classes are
      load-bearing: an injected non-empty stderr sink fires `ERRORING`; removing it clears.
- [x] CLAUDE.md documents the stderr sink as a read surface so the next canary author
      wires it correctly.

Scope note: the four phantom `STALE` entries live in `.context/working/`, which is
gitignored — those heartbeat files exist only in the operator's checkout, not in this
worktree. Fixture Case 4 proves the class fix; the live "`/canaries` returns to exit 0"
observation happens on the host once this branch merges, and is recorded as the Human AC
below rather than asserted from here.

### Human
- [ ] [RUBBER-STAMP] `/canaries` on the host returns to a clean exit after merge
  **Steps:**
  1. `cd /opt/termlink && bash scripts/canary-status.sh`
  **Expected:** `alloc-sink`, `busy-spin`, `drain-sink`, `silent-exit` render as
  `NOT_SCHEDULED` (not `STALE`), and the command exits 0 unless a canary is genuinely
  firing or erroring.
  **If not:** any canary still shown `STALE` is a real one whose cron stopped — check
  `sudo systemctl status cron` and the relevant `/etc/cron.d/termlink-*` entry.

## Verification

bash tests/canary-status-fixtures.sh
out=$(bash scripts/check-cron-install-drift.sh 2>&1); echo "$out" | grep -q "0 drift-warning"
out2=$(bash scripts/canary-status.sh --json 2>&1); echo "$out2" | python3 -c "import json,sys; d=json.load(sys.stdin); assert d['ok'] is True, d; assert all('stderr_bytes' in c for c in d['canaries']), 'stderr_bytes missing'; assert 'erroring' in d['summary'] and 'not_scheduled' in d['summary'], 'summary counters missing'"
bash -n scripts/canary-status.sh
bash -n scripts/check-cron-install-drift.sh

## Recommendation

**Recommendation:** GO

**Rationale:** The change is additive to a read-only reporting verb — it introduces two new
classifications and reads one extra file per canary. No canary script, crontab schedule, or
exit-code contract on the detection path is altered by this task. The one behavioural change
that could hide a problem — `NOT_SCHEDULED` suppressing a `STALE` — is deliberately narrow
(it requires *both* "no log was ever written" *and* "no crontab under `.context/cron/`
mentions the name") and fixture Case 5 proves a genuinely scheduled canary with a dead cron
still fires `STALE`. The crontab reconciliation changed only git-side files to match what is
already installed, verified by `check-cron-install-drift` going 21 → 0 drift.

**Evidence:**
- `tests/canary-status-fixtures.sh` — 8/8 pass, host-independent (`--working-dir` +
  `CANARY_CRON_DIR` fixtures, PL-213 convention).
- Load-bearing: Case 2 injects a non-empty stderr sink and flips `HEALTHY` → `ERRORING`
  (exit 0 → 1); Case 3 truncates it and the state clears.
- Narrowness: Case 5 — declared canary + 30-day-old heartbeat still returns `STALE`.
- No false-fire on resolved history: Case 6 — stderr older than the window does not fire.
- `check-cron-install-drift.sh` → `healthy (24 installed + matching, 0 drift-warning)`.
- Root observation reproducible:
  `TERMLINK_DEAD_LETTER_TEST_JSON=/nonexistent bash scripts/check-dead-letter-freshness.sh --quiet`
  prints nothing on stdout and `check-dead-letter: could not read queue-status (exit=1)` on stderr.

**What the human is being asked for:** only the post-merge observation that the four static
checks now render `NOT_SCHEDULED` on the real host, where their heartbeat files actually
exist. That cannot be asserted from a worktree because `.context/working/` is gitignored.

## RCA

**Symptom:** `/canaries` reports 4 permanent STALE entries and would report HEALTHY for a
canary that errors on every run.

**Root cause:** Three independent design choices compose into a blind spot —
(a) heartbeat is touched before the work, so it proves scheduling not success;
(b) canary tooling-errors go to stderr, which is not the stream `.log` captures under the
installed crontabs; (c) the `.log.stderr` files those crontabs create have no reader.
Each choice is locally reasonable; the composition is a silent-failure sink.

**Why structurally allowed:** `check-cron-install-drift.sh` (T-2561) *did* detect the
crontab divergence — but classifies content drift as a **non-firing warning** and prints
`healthy` in the same line, so the signal was present and discarded. Nothing anywhere
asserted the property "a canary's error output must land somewhere a human or a check
reads". The canary layer had canaries for the substrate but none for itself.

**Prevention:** `canary-status.sh` now reads the stderr sink and fires on it, so the
sink stops being write-only; `tests/canary-status-fixtures.sh` locks the behaviour so a
future refactor that drops the stderr read fails the suite; the git crontab source is
reconciled so `check-cron-install-drift.sh` returns to 0 drift and any *future* divergence
stands out against a clean baseline instead of hiding in a list of 21.

## Decisions

### 2026-08-18 — Which side of the crontab drift is authoritative
- **Chose:** Adopt the installed `2>> .<name>.log.stderr` split into the git source, and
  make `canary-status.sh` read that sink.
- **Why:** The split is the better design — it keeps `.log` purely for firings, so
  "empty log = healthy" stays a true and precise signal instead of being polluted by
  transient stderr noise. The defect was never the split; it was that the split had no
  reader. Adding the reader makes the split correct and clears 21 drift warnings at once.
- **Rejected:** Reverting the install to git's `2>&1`. That restores source-of-truth by
  the letter but merges error output into the firing log, so any stderr warning makes a
  healthy canary look FIRING — trading a false-negative for a false-positive, which is
  how operators learn to ignore the verb.

## Updates

### 2026-08-18T21:18:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent (renumbered T-2678 → T-2690 to avoid
  collision with uncommitted pickup tasks T-2678..T-2686 in the parent checkout)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-10b396ba
- **Timestamp:** 2026-08-18T22:34:42Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-18T22:34:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
