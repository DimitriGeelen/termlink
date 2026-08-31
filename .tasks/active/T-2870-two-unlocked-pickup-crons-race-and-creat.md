---
id: T-2870
name: "Two unlocked pickup crons race and create duplicate tasks; neither is in the
  registry the drift check reads"
description: >
  fw pickup process runs from two /etc/cron.d files on this host (agentic-pickup-termlink
  at * * * * *, agentic-audit-termlink at */15) and NEITHER uses flock. The AEF project
  runs the identical job on the same host WITH flock. Overlapping runs each create
  a task for the same envelope before the dedup marker is written, producing byte-identical
  duplicate tasks. Measured: 3 occurrences (T-2260/T-2261 same second, T-2862/T-2863
  and T-2867/T-2868 both 60s apart). The cron-drift check reads the registry against
  agentic-audit-termlink only, and agentic-pickup-termlink is not in the registry
  at all, so the audit reports 'Cron registry in sync' while the offending job is
  live and unregistered.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: [housekeeping, cron, pickup, duplicate, G-019]
components: []
related_tasks: []
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
created: 2026-08-31T15:27:54Z
last_update: 2026-08-31T15:31:23Z
date_finished:
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
  - ts: '2026-08-31T15:31:23Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=0 (no-signal); 
      D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2870: Two unlocked pickup crons race and create duplicate tasks; neither is in the registry the drift check reads

## Context

`fw pickup process` is scheduled **twice** on this host, from two different
`/etc/cron.d` files, and **neither invocation is serialised**:

```
/etc/cron.d/agentic-pickup-termlink   * * * * *     ... fw pickup process    # no flock
/etc/cron.d/agentic-audit-termlink    */15 * * * *  ... fw pickup process    # no flock
```

The AEF project runs the **identical job on the same machine** and does serialise it:

```
/etc/cron.d/agentic-audit-999-...  * * * * *  ... flock -n /var/lock/agentic-cron-pickup-process.lock -c '... fw pickup process'
```

The correct pattern already exists here; termlink's two copies simply lack it.

**The failure.** A run creates the task for an envelope *before* the dedup marker is
written. `.context/working/.pickup-bridge.log` records the P-054 marker at `13:22:07`,
while the two duplicate tasks were created at `13:21:02` and `13:22:01` — both before it.
A run lasting longer than 60s is still inside that window when the next per-minute run
starts, and both create a task.

**Measured, not inferred — 3 occurrences.** A normalised content-hash scan over all 2590
task files (id, `created`, `last_update`, `date_finished` and embedded timestamps masked)
found exactly three byte-identical pairs:

| Pair | Created | Gap |
|---|---|---|
| T-2260 / T-2261 (completed) | 2026-06-23T20:15:01Z both | same second |
| T-2862 / T-2863 (active) | 07:26:01 / 07:27:01 | 60s |
| T-2867 / T-2868 (active) | 13:21:02 / 13:22:01 | 59s |

The 60s gaps are the per-minute cron's period. The June pair's bridge dedup landed at
`20:15:16` — 15s after both tasks already existed — the same late-marker shape.

**Why nothing caught it, which is the more important half (G-019).** The
cron-install-drift check compares the registry against the crontab each entry *declares*.
`.context/cron-registry.yaml` declares exactly one `pickup-process` job, `*/15`,
`source_file: agentic-audit-termlink`. The every-minute file, `agentic-pickup-termlink`,
**is not in the registry at all** — so it is outside the compared set by construction.
Today's audit reported `[PASS] Cron registry in sync with /etc/cron.d/agentic-audit-termlink`,
which is true and also blind: an unregistered crontab cannot drift from a registry that has
never heard of it. Sibling to T-2682, where installed-but-unregistered job lines read as
cosmetic drift.

**Already remediated (this session):** the two active duplicates T-2863 and T-2868 were
removed after confirming byte-identity modulo id/timestamps; active duplicate groups went
2 to 0. T-2260/T-2261 are in `completed/` and were **deliberately left alone** — rewriting a
finished record so history looks tidier is the wrong repair (the T-2805 append-only lesson).
That remediation does not stop recurrence; the ACs below do.

**Scope note.** Whether `lib/pickup.sh` should also write its dedup marker *before* creating
the task is a real second question, but that file is **vendored** (G-062) — a local fix is
erased by the next re-vendor. Serialising the cron is the fix that lives in this repo and
survives one. If the marker ordering is worth changing, it is an upstream filing, not this task.


## Acceptance Criteria

### Agent
- [ ] `.context/cron-registry.yaml` declares the `agentic-pickup-termlink` crontab and its
      every-minute `fw pickup process` job, so the drift check can see it at all.
- [ ] A regression check fails if any `fw pickup process` cron line on this host lacks
      `flock` — asserting the property, not the current text, so a reinstall that drops the
      lock re-fires it.
- [ ] Duplicate-task scan is repeatable: a normalised content-hash over `.tasks/**` reports
      0 duplicate groups in `active/`, with the two `completed/` entries left untouched.

### Human
- [ ] [RUBBER-STAMP] Serialise both `fw pickup process` cron lines with `flock`, matching the pattern AEF already uses on this host.
  **Steps:**
  1. `sudo cp /etc/cron.d/agentic-pickup-termlink /root/agentic-pickup-termlink.bak && sudo cp /etc/cron.d/agentic-audit-termlink /root/agentic-audit-termlink.bak`
  2. Edit `/etc/cron.d/agentic-pickup-termlink`: wrap the invocation so the line reads
     `* * * * * root PROJECT_ROOT="/opt/termlink" flock -n /var/lock/agentic-pickup-termlink.lock -c "/opt/termlink/.agentic-framework/bin/fw pickup process" 2>&1 | logger -t agentic-pickup`
  3. Edit `/etc/cron.d/agentic-audit-termlink`: wrap its `*/15` pickup line the same way, using the **same** lock path so the two jobs exclude each other.
  4. `grep -h "pickup process" /etc/cron.d/agentic-pickup-termlink /etc/cron.d/agentic-audit-termlink`
  5. Wait ~3 minutes, then `journalctl -t agentic-pickup -n 20 --no-pager`
  **Expected:** Step 4 prints two lines, both containing `flock -n /var/lock/agentic-pickup-termlink.lock`. Step 5 shows the job still running normally — `flock -n` exits silently when the lock is held, which is the intended skip, not an error.
  **If not:** Restore with `sudo cp /root/agentic-pickup-termlink.bak /etc/cron.d/agentic-pickup-termlink` (and likewise for the audit file). If cron stops running the job entirely, check the line has not lost its trailing `| logger` pipe or its `root` user field.


## Verification

# The every-minute pickup crontab is declared in the registry (it was absent — the drift check's blind spot).
grep -q 'agentic-pickup-termlink' .context/cron-registry.yaml
# Every installed `fw pickup process` cron line for this project is serialised. Asserts the PROPERTY, not the
# current text, so a reinstall that drops flock re-fires this rather than passing on a stale match.
bash -c 'n=$(grep -h "fw pickup process" /etc/cron.d/* 2>/dev/null | grep -c "/opt/termlink"); f=$(grep -h "fw pickup process" /etc/cron.d/* 2>/dev/null | grep "/opt/termlink" | grep -c flock); test "$n" -gt 0 && test "$n" -eq "$f"'
# No content-identical duplicate task groups remain in active/ (id + timestamps normalised away).
python3 -c "import re,glob,hashlib,collections;g=collections.defaultdict(list);[g[hashlib.sha256(re.sub(r\"\\d{4}-\\d{2}-\\d{2}[T ]\\d{2}:\\d{2}:\\d{2}\\S*\",\"<TS>\",re.sub(r\"^(id|created|last_update|date_finished):.*$\",\"\",open(p,encoding=\"utf-8\",errors=\"replace\").read(),flags=re.M).replace(re.search(r\"^id:\\s*(\\S+)\",open(p,encoding=\"utf-8\",errors=\"replace\").read(),re.M).group(1),\"<ID>\")).encode()).hexdigest()].append(p) for p in glob.glob(\".tasks/active/*.md\") if re.search(r\"^id:\\s*\\S+\",open(p,encoding=\"utf-8\",errors=\"replace\").read(),re.M)];d=[v for v in g.values() if len(v)>1];assert not d,d;print(\"0 duplicate groups\")"

## RCA

<!-- REQUIRED for bug-class tasks (workflow_type=build with bug-tag, OR title matches
     fix/bug/rca/broken/crash/error/regression/fail/hotfix).
     Non-bug-class tasks may leave this section empty or remove it.

     For bug-class, fill in:
       **Symptom:** what was observed (the user-facing manifestation).
       **Root cause:** the specific structural/logical gap — not "the code was wrong".
       **Why structurally allowed:** what in the framework/code/tooling let this go undetected.
       **Prevention:** what catches the next instance (test/lint/gate/doc/learning) — distinct from the fix itself.

     The completion gate (T-1550, G-019) blocks --status work-completed when
     bug-class AND this section is empty/template-only. Use --skip-rca to bypass (logged).
-->

## Evolution

<!-- REQUIRED for arc-tagged build tasks (tags include arc:*). Captures how
     understanding evolved during build — what was learned that wasn't known at
     filing, what in the original plan no longer fits, what triggered pivots
     or new sub-tasks. Mandatory at slice boundaries (when applicable) and
     before --status work-completed.

     Origin: T-1717 grill Q4 — "the understanding of what we need and want
     evolves with the process of materialisation." Structural counter to §ACD:
     spec-vs-build divergence is logged as soon as it happens, not lost as
     folklore.

     Format (one entry per slice boundary or significant insight):
       ### YYYY-MM-DD — [topic]
       - **What changed:** [what we learned that we didn't know at filing]
       - **Plan impact:** [what in the plan no longer fits]
       - **Triggered:** [new sub-task / pivot / scope cut, with task ID if filed]

     The completion gate (T-1718) blocks --status work-completed when this
     section exists but is empty/template-only. Use --skip-evolution to bypass
     (logged Tier-2). Non-arc tasks may leave this empty.
-->

## Recommendation

<!-- T-2945: same shape as inception.md's block — the gate that reads it
     (audit_inception_recommendation, lib/task-audit.sh:117) is shared, so the
     shape is copied rather than reinvented.

     REQUIRED once this task reaches partial-complete: Agent ACs done, at least
     one `### Human` AC still unticked. `lib/review.sh:205-211` (T-2421) BLOCKS
     `fw task review` emission for build/refactor/test/decommission tasks in that
     state with no substantive block here — the operator would otherwise open
     /review/<id> to a blank Recommendation card and be asked to approve a form.

     Not required while every Human AC is ticked or the task has none: the gate
     only fires on the partial-complete transition. It is here from the start so
     you write it while you still have the evidence, not when the gate refuses.

     Format (the parser wants the `**Recommendation:**` line at the start of a
     line; a leading `-` or `*` bullet is also accepted):
     **Recommendation:** GO / NO-GO / DEFER
     **Rationale:** Why (cite evidence — what shipped, what was proven, what remains)
     **Evidence:**
     - Finding 1
     - Finding 2

     DEFER is for evidence gaps, not confidence gaps (CLAUDE.md §Presenting Work
     for Human Review). If the artefact is complete and you still don't want to
     commit, that is a calibration failure — recommend GO or NO-GO.
-->

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

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-31T15:27:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2870-two-unlocked-pickup-crons-race-and-creat.md
- **Context:** Initial task creation

### 2026-08-31T15:31:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
