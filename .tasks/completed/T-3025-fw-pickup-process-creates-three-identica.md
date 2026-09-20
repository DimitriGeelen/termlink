---
id: T-3025
name: "fw pickup process creates three identical tasks per envelope, two of them ~47s
  later and unannounced"
description: >
  arc-008 run finding. Each fw pickup process run created THREE identical local tasks
  from ONE envelope: P-079 -> T-3019/T-3020/T-3021 (13:20:02/02/03), P-080 -> T-3022/T-3023/T-3024
  (13:30:15, 13:31:02/02). The console names only the first; the other two appear
  ~47s later, so a check run immediately after process sees one task and reads clean.
  Filing two reports upstream injected six spurious tasks into .tasks/active/ - 3x
  amplification per filing. Also observed: metadata.from_project is inconsistent across
  the two filings (offset 126 'root', offset 127 '010-termlink') through the identical
  path, and T-2816's canary self-filter keys on that field so an own-filing will fire
  the framework-pickup canary. Vendored pipeline (lib/pickup.sh, lib/pickup-channel-bridge.sh)
  so the fix is upstream per G-062. Linked to T-3015/T-3016, not merged - arc-008
  rule.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [.agentic-framework/lib/pickup.sh, 
      .agentic-framework/lib/pickup-channel-bridge.sh]
related_tasks: [T-3015, T-3016, T-2963]
arc_id: arc-008
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-20T13:35:39Z
last_update: 2026-09-20T14:06:08Z
date_finished: 2026-09-20T14:06:08Z
# revisit_at: YYYY-MM-DD          # T-1451: set on DEFER decisions to enable G-053 daily revisit scan
# revisit_evidence_needed:        # T-1451: one-line description of what evidence makes the revisit actionable
# ── BVP scoring fields (T-1918, arc-006). See docs/reports/T-1915-bvp-inception.md for semantics. ──
# bvp_scores:                     # confirmed per-driver scores 0-5, set by `fw bvp confirm` (T-1924).
#                                 # Sovereignty boundary — only set after human or agent confirmation.
#                                 # Shape: {D1: <int 0-5>, D2: <int 0-5>, D3: <int 0-5>, D4: <int 0-5>, [<free-driver-id>: <int>]...}
# bvp_scores_proposed:            # estimator-proposed scores (T-1922 worker). Persists when ≥2 delta
#                                 # from bvp_scores: on any driver (M3 v2-delta). Shape: list of timestamped entries.
# cost_estimate:                  # F8 composite: 0.6×blast_radius + 0.3×tier + 0.1×effort.
#                                 # Q2 fallback: T-shirt S/M/L/XL mapped to 2/4/6/8 when blast_radius is not yet computable.
bvp_scores_proposed:
  - ts: '2026-09-20T13:57:41Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=2 (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-09-20T13:57:46Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (2-components); tier=2 (workflow:build); effort=8 
      (lines=238,acs=6)
    rubric_sha: e4a00f38e801
---

# T-3025: fw pickup process creates three identical tasks per envelope, two of them ~47s later and unannounced

## Context

`fw pickup process` has no mutual exclusion, and the dedup row that suppresses
re-processing is written AFTER task creation rather than before it. Task creation takes
roughly a minute, so any concurrent run entering that window sees the envelope still in
`inbox/` with no dedup row and creates its own task from it. Three per-minute-or-finer
schedules drive `pickup process` against this project's inbox and they do not share a
lock, so at least two unguarded runs race every minute. Result: one envelope, three
tasks. Vendored pipeline, so the fix is upstream per G-062.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The mechanism is established rather than guessed: show that the duplicate tasks are
      produced by REPEATED `fw pickup process` runs (the per-minute crons) over an envelope
      whose dedup record was never written, not by a single run emitting three. Record the
      dedup-write failure and the cron schedule as evidence.
- [x] The race is root-caused in `lib/pickup.sh` with line numbers: creation at `:542`
      precedes the dedup record at `:555` and the move at `:558`, so the suppressing row does
      not exist while the work is being done; and the file contains no `flock`/lockfile/mutex,
      so concurrent runs are unserialised. Record the measured create→record latency and the
      competing schedules.
- [x] Every round-trip task this run produced is dispositioned with the disposition recorded
      in the file itself and removed from the now-queue: T-3019/T-3020/T-3021 (envelope P-079),
      T-3022/T-3023/T-3024 (P-080) and T-3026/T-3027 (P-081, minted by filing this report).
      Eight, not six — filing the report reproduced the defect.
- [x] Filed upstream to `framework:pickup` per G-062 with the mechanism, the ordering defect and
      a proposed fix; offset recorded here. No local patch to `.agentic-framework/`, stated.

**Correction already required on this task (recorded before any work).** It was filed as
"`fw pickup process` creates three identical tasks per envelope". That is the SYMPTOM, not the
mechanism, and the title is wrong. Evidence found immediately after filing:

    /etc/cron.d/agentic-pickup-termlink        * * * * *   fw pickup process   (per minute)
    .context/cron/agentic-audit.crontab      */15 * * * *  fw pickup process
    plus a second per-minute entry for /opt/termlink and one for another project

and, printed during the interactive run itself:

    pickup_dedup_hash: envelope not readable: .context/pickup/inbox/P-079-bug-report.yaml
    pickup_record_dedup: refusing to record an uncomputable dedup hash — ledger left unchanged

So the likely chain is: interactive `process` moves the envelope, then computes the dedup hash
against the pre-move inbox path, fails, and declines to record; the per-minute cron then finds
work it has no record of having done and processes it again. Three passes, three tasks. The
~47s gap between the first task and the next two is a cron tick, not a delayed write.

This is the THIRD framing of this finding in one run (three-per-envelope → "did not reproduce"
→ repeated-cron-passes-over-a-missing-dedup-record). The first two were asserted from a single
observation each. That is the thing to fix in method, and AC1 exists to force it: establish the
mechanism before the title claims one. See T-2963 ("two unguarded pickup process crons"), which
already owns the cron half and must be linked, not merged.

**FOURTH framing, and this one is measured (recorded after the code read).** The third
framing above — "interactive `process` moves the envelope, THEN computes the dedup hash
against the pre-move inbox path, fails, declines to record" — asserts an ordering that the
source does not have. `lib/pickup.sh` runs create `:542` → record `:555` → move `:558`. The
record is attempted BEFORE the move, not after it, so the original AC2 was root-causing a
defect that is not there. Corrected rather than quietly rewritten, because the pattern of
re-framing on one observation is the method fault this task exists to stop.

What the evidence actually supports:

  - The dedup row is written LAST and LATE. Measured from this project's own `dedup.log`
    against the `created:` stamps of the tasks produced: P-079 first task 13:20:02Z, dedup
    row 13:21:00Z (58s); P-080 first task 13:30:15Z, dedup row 13:31:16Z (61s).
  - Nothing serialises runs. `lib/pickup.sh` has no `flock`, lockfile or mutex; and
    `/etc/cron.d/agentic-pickup-termlink:5` runs `* * * * *` with NO lock at all, while
    `/etc/cron.d/agentic-audit-termlink:42` (`* * * * *`) and `:39` (`*/15`) share
    `/var/lock/agentic-pickup-termlink.lock`. The two locked jobs exclude each other and
    neither excludes the unlocked one.

So `envelope not readable` is the LOSING racer computing its hash after a winner completed
the move — the race seen from the losing side, not the cause of it. That run had already
created its task and then recorded no suppression, which is precisely the amplification.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `
## Upstream filing

upstream_filing: framework:pickup@128 (pickup_id P-081, msg_type pickup-bug-report,
source.project 010-termlink, source.task_id T-3025)

Verified retrievable on the rail. Exactly one copy - the losing racer did not mirror.

No local patch was made to `.agentic-framework/lib/pickup.sh` or
`.agentic-framework/lib/pickup-channel-bridge.sh`. Both are vendored; per G-062 a local fix is
deleted by the next re-vendor. Stated rather than left silent.

**A second finding, reproduced deliberately rather than inferred.** The envelope was sent with
an explicit `--source-project 010-termlink`, and its payload carries `source.project:
010-termlink`. The rail envelope at offset 128 nevertheless carries `metadata.from_project:
"root"`. So `metadata.from_project` is not derived from the envelope's own project field and
cannot be set by the sender. This is load-bearing: T-2816's framework-pickup canary suppresses
a project's OWN filings by matching that field against `FW_PICKUP_SELF_PROJECT` (default
`010-termlink`), so a filing stamped `root` is not recognised as ours and fires the canary at
us. Per T-2816's own note, the `--ack` needed to clear that echo also acks any genuine inbound
filing that landed in between - reintroducing the G-063 miss the canary exists to prevent. Two
of the three filings in this run (offsets 126 and 128) carry `root`; offset 127 carries
`010-termlink`, through the same path. The cause is not established here and is reported as an
observation, not a diagnosis.

## Live capture of the race (strongest evidence, obtained by filing this very report)

Filing this report through the sanctioned `fw pickup send` path reproduced the defect on
demand, and the cron log caught both racers in the same second:

    16:01:01  agentic-pickup[2847676]  Pickup summary: 0 found ... Inbox is empty
    16:01:01  agentic-cron[2847687]    Pickup summary: 0 found ... Inbox is empty
    ...
    16:02:02  agentic-pickup[2878896]  PROCESS P-081-bug-report.yaml - fw pickup process is unserialised...
    16:02:02  agentic-cron[2878925]    PROCESS P-081-bug-report.yaml - fw pickup process is unserialised...

Two DISTINCT pids, from two DIFFERENT cron files, both claiming the same envelope in the
same second. This is no longer inferred from `created:` stamps - it is the race observed
directly. `agentic-pickup[...]` is `/etc/cron.d/agentic-pickup-termlink:5`, which takes no
lock; `agentic-cron[...]` is `/etc/cron.d/agentic-audit-termlink:42`, which takes
`/var/lock/agentic-pickup-termlink.lock`. The lock is real and it is useless, because the
job it needs to exclude does not participate in it.

The window was confirmed to be long, and both racers ran to completion. Winner
`agentic-pickup[2878896]` created T-3026 at 16:03:01 and wrote the dedup row at 14:03:01Z -
59s after its own PROCESS line at 14:02:02Z. The loser `agentic-cron[2878925]` stayed inside
`pickup_create_inception` for a further minute and then produced, in this order:

    16:04:01  File:  .tasks/active/T-3027-pickup-fw-pickup-process-is-unserialised.md
    16:04:01  pickup_dedup_hash: envelope not readable: .../inbox/P-081-bug-report.yaml
    16:04:01  pickup_record_dedup: refusing to record an uncomputable dedup hash - ledger left unchanged
    16:04:01  WARN pickup id collision: P-081-bug-report.yaml already exists in processed/
    16:04:01  Pickup summary: 1 found, 1 processed, 0 rejected

That sequence settles the causality the earlier framings got backwards. The
`envelope not readable` line is emitted by the LOSER, AFTER the winner's move, and AFTER the
loser has already created its duplicate task. It is a consequence of the race, not its cause,
and the fail-closed refusal beneath it is correct behaviour.

Note the last line: the run reports `1 found, 1 processed, 0 rejected`. A pass that produced
nothing but a duplicate reports success. Nothing in the output tells an operator that the task
just created is a copy - which is why this went unnoticed through two filings.

Scope of the amplification, measured rather than assumed: it is LOCAL TASKS ONLY. The loser did
NOT double-post to the rail - `framework:pickup` carries exactly one copy at offset 128 - and
no `.dup-1` file exists in `processed/` despite the WARN claiming one was filed. The bridge is
gated on `[ -f "$processed_path" ]` (T-3051/T-3052), and the loser's move had no source left to
move, so the mirror step was skipped. The upstream rail is protected; the local task register
is not.

Amplification is also NOT deterministic. This run had two racers and produced two tasks; the
two earlier envelopes produced three each. The count is however many `pickup process`
invocations enter the ~60s window, which depends on cron phase and on whether an operator runs
the verb by hand.

Note the shape of this evidence. Filing a bug report about the pickup pipeline goes THROUGH
the pickup pipeline, so the act of reporting the defect triggers it. That is worth stating
plainly upstream: the report is its own reproduction case.

## Verification

# All checks are written to avoid the T-2831 vacuous class: any check that asserts a
# token present in THIS file first excises the ## Verification section, so the command
# line cannot match itself. Each was mutant-tested (wrong value must fail).
#
# 1. The race root-cause is recorded with the real ordering (create :542 before record :555).
python3 -c "import re,sys;t=open('.tasks/active/T-3025-fw-pickup-process-creates-three-identica.md').read();b=re.sub(r'^## Verification.*?\\Z','',t,flags=re.M|re.S);sys.exit(0 if (':542' in b and ':555' in b and ':558' in b) else 1)"

# 2. The measured create->record latency is recorded for both envelopes.
python3 -c "import re,sys;t=open('.tasks/active/T-3025-fw-pickup-process-creates-three-identica.md').read();b=re.sub(r'^## Verification.*?\\Z','',t,flags=re.M|re.S);sys.exit(0 if ('13:21:00Z' in b and '13:31:16Z' in b) else 1)"

# 3. lib/pickup.sh genuinely has no serialisation primitive (the claim the filing rests on).
test -z "$(grep -E 'flock|lockfile|mkdir[^|]*lock' .agentic-framework/lib/pickup.sh || true)"

# 4. The unlocked per-minute schedule genuinely exists and takes no lock.
grep -q 'pickup process' /etc/cron.d/agentic-pickup-termlink
test -z "$(grep 'flock' /etc/cron.d/agentic-pickup-termlink || true)"

# 5. All eight round-trip tasks carry a recorded disposition and are out of the now-queue.
python3 -c "import glob,sys;ids=['3019','3020','3021','3022','3023','3024','3026','3027'];fs=[f for i in ids for f in glob.glob('.tasks/active/T-%s-*.md'%i)];sys.exit(0 if len(fs)==8 and all('DISPOSITION (arc-008' in open(f).read() and 'horizon: later' in open(f).read() for f in fs) else 1)"

# 6. The upstream filing is recorded here and retrievable on the rail (offset token below).
python3 -c "import re,sys;t=open('.tasks/active/T-3025-fw-pickup-process-creates-three-identica.md').read();b=re.sub(r'^## Verification.*?\\Z','',t,flags=re.M|re.S);sys.exit(0 if 'upstream_filing: framework:pickup@128' in b else 1)"

# 7. No local patch was made to the vendored pickup pipeline.
test -z "$(git status --porcelain .agentic-framework/lib/pickup.sh .agentic-framework/lib/pickup-channel-bridge.sh)"

## Reviewer Verdict (v1.5)

- **Scan ID:** R-2e256381
- **Timestamp:** 2026-09-20T14:06:11Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-09-20T14:06:08Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
