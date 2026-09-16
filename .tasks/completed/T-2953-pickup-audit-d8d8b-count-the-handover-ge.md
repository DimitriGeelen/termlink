---
id: T-2953
name: "Pickup: audit D8/D8b count the handover generators own DELIBERATE unfilled
  markers (from termlink)"
description: >
  Auto-created from pickup envelope. Source: termlink, task T-2942. Type: bug-report.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [pickup, bug-report]
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
created: 2026-09-09T23:08:02Z
last_update: 2026-09-16T16:12:44Z
date_finished: 2026-09-16T16:12:44Z
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
source_task_id_in_origin: T-2942
source_project_in_origin: "termlink"
bvp_scores_proposed:
  - ts: '2026-09-11T20:45:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 1
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=1 (body:episodic-only); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2953: Pickup: audit D8/D8b count the handover generators own DELIBERATE unfilled markers (from termlink)

## Context

T-2946, T-2947, T-2952 and T-2953 are **not inbound findings**. They are four local
tasks minted from this project's OWN outbound filings, for two findings that were
already completed and already filed upstream on 2026-09-09: T-2943 → P-073
(`framework:pickup` offset 115) and T-2942 → P-076 (offset 119). Both correctly
declined to patch the vendored `audit.sh` / `handover.sh` per G-062.

The count was three when this task was scoped. The sweep required by AC 6 found the
fourth (T-2947) and two older ones (T-2222, T-2232) — which is the reason that AC
exists: the visible instances were not the population.

They are the downstream cost of the `fw pickup send` write-location defect corrected
upstream at offset 122: `.agentic-framework/lib/pickup.sh:643` writes every outbound
envelope to `$PICKUP_INBOX`, the directory `fw pickup process` scans for INBOUND work,
so a project re-ingests its own reports as new work.

The deliverable here is therefore the **disposition** of the re-minted set and the
measured instance count contributed to the upstream thread — **not** a fix to D8/D8b,
which is upstream's and already reported twice.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Origin established by measurement, not inference. Each re-minted task is traced to the envelope and the completed local task it came from, with timestamps: T-2946 + T-2947 ← the single envelope `P-073` ← T-2943 (completed 2026-09-09T18:22:37Z); T-2952 + T-2953 ← the single envelope `P-076` ← T-2942 (completed 2026-09-09T23:08:34Z). Recorded in `## RCA`.
- [x] The **double-mint** is named as a defect distinct from the write-location one, and it is not a one-off: **both** September envelopes were minted twice, each pair exactly 60s apart (18:09:02Z/18:10:02Z and 23:07:02Z/23:08:02Z), from one file each in `.context/pickup/processed/`. The write-location defect explains why the envelopes were in the inbox at all; it does not explain why each was minted twice.
- [x] The contributing cause is measured on this host rather than inferred from the verb's source: **two unsynchronised `fw pickup process` crons** run against `/opt/termlink` — `/etc/cron.d/agentic-pickup-termlink` every minute and `/etc/cron.d/agentic-audit-termlink` every 15 — and **both installed copies have the `flock` guard that git declares stripped out**. `scripts/check-cron-install-drift.sh` was already FIRING on exactly these two job lines (UNINSTALLED_JOBS, T-2682 class); nobody had read it. Filed as its own task, not fixed here.
- [x] The underlying D8/D8b findings are confirmed already disposed — P-073 at offset 115, P-076 at offset 119, neither patched locally — so no D8/D8b work is outstanding under this task. Stated explicitly rather than left silent.
- [x] T-2946, T-2947 and T-2952 are closed through `fw task update` as duplicates of completed work — via the verb, not hand-edited and not deleted — each naming the task it duplicates.
- [x] The measurement is appended to the upstream correction thread (reply to offset 122) and read back from the hub: the write-location defect is not theoretical, it produced **6** phantom tasks in `active/` from this project's own filings, 4 of them in one day and every one of those a double-mint.
- [x] The remaining pickup-minted backlog is swept for the same shape and the **scope of the sweep is stated** (T-2680): every `.tasks/active/` task carrying `source_project_in_origin: "termlink"` is enumerated, and each is either dispositioned here or named as still-open with a reason.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.

     ── Prefix routing (T-1811, T-1878): default to [REVIEWER] if Expected is grep-able ──
     If your Expected clause is grep-able / file-exists / structural (a deterministic
     shell check), prefer [REVIEWER] — that AC should be an Agent AC with the reviewer
     command in `## Verification` instead of a Human AC here. Only keep [REVIEW] if
     verification genuinely needs human taste (tone, feel, layout rhythm).
     See CLAUDE.md §AC Classification Guidance for the conversion rule.

     [REVIEW] example (genuine human judgment):
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error

     [REVIEWER] example (static-scan-verifiable — convert to Agent AC + Verification):
       - [ ] [REVIEWER] Block message names both bypass mechanisms
         **Steps:**
         1. Run `bin/fw reviewer T-XXX`
         **Expected:** Verdict: PASS; no findings on `block-message-completeness`
         **If not:** Inspect hook block-message string and add missing mechanism
       Conversion: this AC should be moved to ### Agent and
       `bin/fw reviewer T-XXX > /tmp/.rev 2>&1 && grep -q "Overall:.*PASS" /tmp/.rev`
       added to ## Verification. NEVER `... 2>&1 | grep -q ...` — that is the shape the
       Pipefail/SIGPIPE section below forbids, and this line used to prescribe it.
-->

## Verification

# The three duplicates went through the gate into completed/, not deleted.
grep -q "^status: work-completed" .tasks/completed/T-2946-pickup-auditsh-d8-handover-quality-check.md
grep -q "^status: work-completed" .tasks/completed/T-2947-pickup-auditsh-d8-handover-quality-check.md
grep -q "^status: work-completed" .tasks/completed/T-2952-pickup-audit-d8d8b-count-the-handover-ge.md
# Both origin tasks were already completed before their envelopes were re-minted.
grep -q "^date_finished: 2026-09-09T18:22:37Z" .tasks/completed/T-2943-d8-audit-check-can-never-pass-generator-.md
grep -q "^date_finished: 2026-09-09T23:08:34Z" .tasks/completed/T-2942-d8b-10-of-10-recent-handovers-carry-unfi.md
# Each envelope exists exactly ONCE in processed/ — so two tasks from one file is a double-mint.
test "$(ls .context/pickup/processed/ | grep -c '^P-073-')" = "1"
test "$(ls .context/pickup/processed/ | grep -c '^P-076-')" = "1"
# The sweep's residual: exactly 3 self-minted tasks remain in active/ (T-2222, T-2232, and this task,
# which is still in active/ at verification time because the move happens after the gate).
test "$(grep -l 'source_project_in_origin: "termlink"' .tasks/active/*.md | wc -l)" = "3"
# The cron-drift guard still names both flock-guarded pickup job lines as declared-but-unscheduled.
bash scripts/check-cron-install-drift.sh > /tmp/.t2953-cron.out 2>&1 || true
grep -q "flock -n /var/lock/agentic-pickup-termlink.lock" /tmp/.t2953-cron.out
# The measurement is on the hub at offset 124, threaded to the offset-122 correction.
termlink channel subscribe framework:pickup --cursor 124 --limit 1 --json > /tmp/.t2953-hub.json 2>&1
python3 -c 'import json,base64,sys; d=json.loads(open("/tmp/.t2953-hub.json").read().strip().splitlines()[0]); open("/tmp/.t2953-body.txt","wb").write(base64.b64decode(d["payload_b64"])); sys.exit(0 if str(d["metadata"]["in_reply_to"])=="122" else 1)'
grep -q "THE SECOND DEFECT: DOUBLE-MINT" /tmp/.t2953-body.txt
grep -q "6 phantom tasks" /tmp/.t2953-body.txt

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# ── Pipefail/SIGPIPE: grepping a command's output (L-387, T-2090, T-2743, T-2738) ──
#
# THE DEFAULT — redirect to a file, then grep the file:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
#     curl -sf "$(bin/fw watchtower url)/page" -o /tmp/.out && grep -q "PAT" /tmp/.out
# Correct at any output size, and `&&` keeps the PRODUCING command's exit code in
# the verdict. Reach for this first; the alternative below is the special case.
#
# NEVER `cmd | grep -q PAT` (L-387) — why: P-011 runs each line under `set -eo
# pipefail`. When grep matches it exits and closes stdin while cmd is still
# writing, cmd takes SIGPIPE, the pipeline exits 141 — verification "fails" with
# the pattern present. Captured 4× (T-1716, T-1838, T-1862, T-1863).
#
# THE EXCEPTION — capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Valid ONLY while "$out" fits the 65536-byte pipe buffer, and it is on you to
# know that it does. Above that the form inverts and becomes the very failure
# L-387 describes: echo blocks on the full pipe, grep -q exits, echo takes
# SIGPIPE, rc=141 (T-2743 — measured on a 146,366-byte Watchtower page, 3/3 runs,
# deterministic not racy; rendered routes run 50-200KB, so anything that curls a
# page is over the line). It also discards cmd's exit code, so a 404 yields an
# empty capture that grep merely fails to match rather than a failed line.
# If you do use it: single pipe only, no intermediate tail/awk/sed stage between
# capture and grep (T-2090) — the middle stage is what `grep -q` slams its stdin
# on, and grep scans the whole captured string anyway, so the `tail -3` was
# cosmetic. `echo "$out" | grep -q PAT`, nothing between.
#
# TEST RUNNERS need a guard either way (T-2738). `set -e` is suppressed inside the
# `if` condition the gate runs each line in, so in `cmd1; cmd2` only cmd2 is the
# verdict — and the pass marker you grep for survives a partial failure: a suite
# printing "3 failed, 9 passed" satisfies `grep -q "9 passed"`, and generalising
# to `grep -qE "[0-9]+ passed"` matches the same output. Keep the exit code:
#     python3 -m pytest <file> -q > /tmp/.out 2>&1 && grep -q passed /tmp/.out
# or add the guard the exit code used to supply:
#     out=$(python3 -m pytest <file> -q 2>&1); echo "$out" | grep -q passed && ! echo "$out" | grep -q failed
#     out=$(bats <file> 2>&1); echo "$out" | grep -q '^ok 1 ' && ! echo "$out" | grep -q '^not ok'
# The close gate refuses the unguarded form. Bypass: FW_ALLOW_UNJUDGED_TEST_RUN=1.
#
# REHEARSING A LINE BY HAND DOES NOT REHEARSE THE GATE (T-2743). Your interactive
# shell has no `set -eo pipefail`. A line has returned 0 by hand and 141 under
# P-011, from the same directory, the same second. To rehearse for real:
#     bash -c 'set -eo pipefail; <your verification line>'
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

## RCA

**Symptom:** four local tasks (T-2946, T-2947, T-2952, T-2953) appeared in `active/`
describing two findings this project had already completed and already filed upstream
the same day. Two of them were re-opened and worked before the origin was noticed.

**Root cause — two independent defects, in sequence.**

1. *Write location.* `fw pickup send` has no outbound store: `lib/pickup.sh:643` writes
   every envelope to `$PICKUP_INBOX`, the directory `fw pickup process` scans for
   inbound work. Direction is not carried by location, so a project re-ingests its own
   reports. Corrected upstream at `framework:pickup` offset 122.
2. *Double-mint.* `P-073` and `P-076` each exist as exactly **one** file in
   `.context/pickup/processed/`, and each produced **two** tasks exactly 60s apart
   (18:09:02Z/18:10:02Z, 23:07:02Z/23:08:02Z). Defect 1 explains why the envelope was
   in the inbox at all; it does not explain the second mint. `pickup process` mints the
   task and moves the envelope as two steps, so an invocation landing between them
   mints again.

**Why structurally allowed:** nothing distinguishes an outbound record from an inbound
one, so no filter at the minting layer can be correct — and attribution, the only other
discriminator, is itself unreliable (`rail_project_label()` falls back to `basename $PWD`).
The second defect was amplified by two unsynchronised `fw pickup process` crons
(`/etc/cron.d/agentic-pickup-termlink` every minute, `/etc/cron.d/agentic-audit-termlink`
every 15) whose installed copies have the `flock` guard git declares **stripped out**.
`scripts/check-cron-install-drift.sh` was already FIRING on exactly those two job lines
and nobody had read it — the guard worked; the reading of it did not.

**Prevention:** both defects filed upstream (offsets 122 and 124) since `lib/pickup.sh`
is vendored (G-062); the cron half filed locally as **T-2963**; the two pre-September
re-mints as **T-2964**. No local patch to the vendored verb, so `.vendor-divergence.yaml`
needs no entry — stated rather than left silent.

<!-- template note below -->

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

### 2026-09-16 — the duplicate filing I made while reporting duplicate filings
- **Chose:** leave both copies on the rail (offsets 124 and 125, byte-identical,
  sha256 `751fb2f982b9`) and record the error here rather than post a third message
  correcting it.
- **Why:** a correction would be a third message on a topic whose problem I am
  reporting is message duplication. The cheaper honest move is to name it where the
  work is recorded. `channel post` mints a fresh `--client-msg-id` per invocation by
  design (T-2049 dedupe is for *retries* of one send), so two invocations are
  correctly two messages — the tool did the right thing and I called it twice.
- **Rejected:** posting a retraction (adds noise to prove a point about noise);
  saying nothing (a peer reading the topic would count two independent reports).

### 2026-09-16 — closing the duplicates through the gate, not around it
- **Chose:** write a real AC and RCA on each of T-2946/T-2947/T-2952 and close them
  through `fw task update --status work-completed` with the gates armed.
- **Why:** the mandate's rule is that a gate which refuses you is a finding, not an
  obstacle. `--skip-acceptance-criteria` would have closed them in one command and
  left three tasks in `completed/` with template ACs and no evidence trail.
- **Rejected:** `--force` / `--skip-*` (routes around P-010 and P-011); deleting the
  files (destroys the evidence that the re-mint happened, which is the finding).

### 2026-09-16 — not fixing D8/D8b here
- **Chose:** confirm the D8/D8b findings are already disposed and do no work on them.
- **Why:** T-2943 (P-073, offset 115) and T-2942 (P-076, offset 119) already measured
  and filed both halves, correctly declining to patch vendored code. Re-doing it under
  a re-minted task ID is precisely the waste the re-mint causes.
- **Rejected:** treating the re-minted titles as a live work request — that is what
  made two sessions open them in the first place.

<!-- template note below -->

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

### 2026-09-09T23:08:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2953-pickup-audit-d8d8b-count-the-handover-ge.md
- **Context:** Initial task creation

### 2026-09-11T20:45:28Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8bf8d3ff
- **Timestamp:** 2026-09-16T16:12:46Z
- **Catalogue:** v1.3-seed
- **Overall:** FAIL
- **Needs Human:** no
- **Findings:** 2

**Per-AC findings:**

- **AC#3 (Agent)** — The contributing cause is measured on this host rather than inferred from the verb's source: **two unsynchronised `fw pickup process` crons** run against `/opt/termlink` — `/etc/cron.d/agentic-pickup-
  - **AC-verify-mismatch** (narrow, heuristic) — `path=etc/cron.d in: The contributing cause is measured on this host rather than inferred from the verb's source: **two unsynchronised `fw pickup process` crons** run agai`

**Verification-level findings:**

  1. **swallowed-errors** (severe, deterministic) @ Verification:line 15
     - evidence: `bash scripts/check-cron-install-drift.sh > /tmp/.t2953-cron.out 2>&1 || true`

### 2026-09-16T16:12:44Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** Self-minted pickup backlog dispositioned; double-mint defect measured and filed upstream at offset 124; cron cause filed as T-2963, older re-mints as T-2964.
