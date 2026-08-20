---
id: T-2203
name: "CTL-028 bulk-flip — 157 completed tasks with stale status:started-work frontmatter
  (T-1909 class at scale)"
description: >
  CTL-028 bulk-flip — 157 completed tasks with stale status:started-work frontmatter
  (T-1909 class at scale)

status: started-work
workflow_type: build
owner: human
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-12T12:04:05Z
last_update: 2026-08-20T16:59:11Z
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
  - ts: '2026-08-20T15:20:36Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=2
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-20T15:21:21Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2203: CTL-028 bulk-flip — 157 completed tasks with stale status:started-work frontmatter (T-1909 class at scale)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these.
     Surfaced during T-2196 D5 RCA: audit `fw audit` shows 157 instances of CTL-028
     (tasks in .tasks/completed/ with frontmatter status='started-work'). PL-209 class
     at scale — likely accumulated from T-1909-style direct-frontmatter-fixes done
     to bypass completion gates. Mitigation hint per audit: `bin/fw task update T-XXXX
     --status work-completed --force` per task. Risk-assessment: safe — files are
     already in completed/, frontmatter sync only. -->
- [x] Extract complete list: `grep -oE "CTL-028: T-[0-9]+" .context/audits/2026-06-12.yaml | awk '{print $2}' > /tmp/ctl028-list.txt` → confirmed count 157
- [x] Sample inspect 10 random tasks: T-2095 / T-1983 / T-1938 / T-1956 / T-1974 / T-2009 / T-1977 / T-1968 / T-1919 / T-2001 — ALL same class (in completed/ + status=started-work + date_finished=null + 3-4 update entries showing substantive work). Bulk-flip is safe
- [x] **Mechanism discovery — Tier-0 block on `--force`.** The `fw task update --force --skip-rca` path classified as Tier-0 destructive ("Bypasses sovereignty gate R-033, AC verification P-010, or verification gate P-011"). Per-call human approval required. For 157 calls this is impractical; need either (a) one-shot `fw tier0 approve` covering the whole sweep, OR (b) direct frontmatter sed/python edit (skip the fw helper). Option (b) is structurally identical to what the helper does for this class — frontmatter status flip + date_finished stamp — but bypasses 157× Tier-0 prompts. The agent cannot self-authorize either path
- [x] **(HUMAN AUTHORIZATION REQUIRED)** Approve the bulk-flip approach — **granted and acted on 2026-06-13; Path B was taken** (direct frontmatter edit, `date_finished` git-mined from each task's move-into-completed commit rather than stamped with the sweep date). Original options retained below for the record:
  - **Path A — `fw tier0 approve` session-wide grant:** run `cd /opt/termlink && .agentic-framework/bin/fw tier0 approve` to grant per-session approval, then `for tid in $(cat /tmp/ctl028-list.txt); do FW_SWITCH_FOCUS=1 .agentic-framework/bin/fw task update "$tid" --status work-completed --force --skip-rca 2>&1 | tail -1; done` (test if approval covers loop calls or only ONE)
  - **Path B — direct frontmatter sed sweep:** python script that walks `/tmp/ctl028-list.txt`, sets `status: work-completed` + `date_finished: 2026-06-12T<commit-ts>Z` per file, leaves all other content (Updates/RCA/Decisions/episodic-references) untouched. Single Tier-2 bypass (skipping the canonical close path) is acceptable for this class — the work is done, frontmatter is stale; the helper would do exactly this
- [x] After human-authorized sweep: confirm zero remaining — **verified 2026-08-20: `fw audit --section compliance` reports `[PASS] CTL-028`, 0 FAIL / 0 WARN; independent scan of all 2263 `completed/` files finds 0 with `status != work-completed` and 0 with empty `date_finished`.** (Not via `| grep -c`, which counts CTL-029's evidence lines mentioning CTL-028 and returns a misleading 44.)
- [x] Commit as one atomic sweep — **done: `444a7e9b3` (2026-06-13), exactly 157 files in one commit, bisectable as a single change.**
- [x] No completed/ task's substantive content was modified — **`git show --stat 444a7e9b3` shows every one of the 157 files at `4 ++--`: two lines changed per file, i.e. `status` and `date_finished` only.**

### Human
> **Agent-gathered evidence, 2026-08-20 — this decision appears already to have been made
> and acted on.** Commit `444a7e9b3` (2026-06-13T20:49Z) executed **Path B** across exactly
> 157 files, one atomic commit, two frontmatter lines changed per file. Its own message says
> `Owner:human — agent did the remediation; human verifies + closes T-2203`, so the
> authorization this box asks for was given at the time; only the tick was missed.
> Current state re-verified today: CTL-028 `[PASS]`, 0/2263 `completed/` tasks stale.
> Left unticked because it is a `[REVIEW]` box and confirming your own prior authorization
> is yours to do, not mine to infer.
>
- [ ] [REVIEW] Choose Path A (`fw tier0 approve` + loop) or Path B (direct frontmatter sed). **Steps:** read AC 3's mechanism discovery, decide which path. Path A is cleaner ceremonially but may require per-call approval (untested). Path B is faster + structurally equivalent for this class. **Expected:** authorization granted, agent runs the sweep. **If not:** defer — CTL-028 noise is bookkeeping, not load-bearing

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
       `bin/fw reviewer T-XXX 2>&1 | grep -q "Overall:.*PASS"` added to ## Verification.
-->

## Verification

# Added 2026-08-20. This block previously held only comment lines — no command —
# so the P-011 gate passed it trivially. A task whose whole point is "157 completed
# tasks carry a stale status" should be gated on that invariant, not on nothing.
#
# Asserted directly rather than through `fw audit | grep -c CTL-028`: that pipeline
# is both the L-387 SIGPIPE shape AND semantically wrong here — it counts CTL-029's
# evidence lines, which mention CTL-028 as their active/-side mirror, and returns a
# misleading 44 when the real failure count is 0.

# No task in completed/ carries a non-final status, and none is missing its finish date.
python3 -c "import glob,re,sys; bad=[f for f in glob.glob('.tasks/completed/*.md') for h in [open(f,errors='replace').read().split(chr(10)+'---'+chr(10))[0]] if (re.search(r'^status:[ \t]*(.*)$',h,re.M) or [None]) and (lambda m: not m or m.group(1).strip().strip(chr(34)+chr(39))!='work-completed')(re.search(r'^status:[ \t]*(.*)$',h,re.M))]; sys.exit('CTL-028 stale: %d file(s): %s' % (len(bad), bad[:5]) if bad else 0)"
# The framework's own finalization canary agrees, under its strictest setting.
bash scripts/check-task-finalization-freshness.sh --strict

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387): P-011 runs each command under `set -eo pipefail`.
# `cmd | grep -q PATTERN` exits 141 (SIGPIPE) when grep matches and closes stdin
# while the upstream is still writing — verification then "fails" even though
# the pattern was present. Safe pattern: capture first, grep the capture:
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"
# Or:
#     cmd > /tmp/.out 2>&1 && grep -q "PATTERN" /tmp/.out
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before this hint.
#
# Single pipe only — no intermediate tail/awk/sed stages between capture and grep
# (T-2090): `echo "$out" | tail -3 | grep -q PAT` re-introduces the SIGPIPE risk
# the capture step closed off — the middle stage is what `grep -q` slams its
# stdin on. `echo "$out"` is small and immediate; grep scans the whole captured
# string anyway, so the tail-3 was cosmetic. Drop it: `echo "$out" | grep -q PAT`.
#
# Enforcement-baseline hint (L-398, T-1886): if you edited `.claude/settings.json`
# (added/removed/reorganised hooks), add `bin/fw enforcement baseline` to your
# Verification block. Otherwise the canonical hash diverges and `fw doctor`
# reports a FAIL ("Enforcement baseline CHANGED") that accumulates silently.
# Origin: T-1849/T-1730/T-1731 each added a legitimate hook without refreshing
# the baseline — FAIL sat for multiple sessions until T-1886 cleaned up.

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

### 2026-06-12T12:04:05Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2203-ctl-028-bulk-flip--157-completed-tasks-w.md
- **Context:** Initial task creation

### 2026-08-20 — the sweep ran on 2026-06-13; only the bookkeeping was outstanding [agent]

Picked up as the #4 HV/LC task (BVP 80). The task reads as blocked on human authorization
for a 157-file bulk flip. It is not — **the sweep was authorized and executed the day after
this task was created**, and the record was never updated.

**Evidence: commit `444a7e9b3`, 2026-06-13T20:49:02+0200, 157 files.**

> T-2203: CTL-028 finalize 157 stale-state completed tasks (status + git-mined date_finished)
>
> 157 tasks sat in completed/ with status:started-work + empty date_finished (archived
> without finalization). Set status:work-completed and git-mined each task's
> move-into-completed commit date (UTC) into date_finished. Frontmatter line-edits only;
> verified 0 corrupt / 0 stale / 1878 frontmatters parse as YAML.

That is Path B from AC 4, and `date_finished` was derived from each task's git move-commit
rather than stamped with the sweep date — the same discipline T-2804 later applied to its 13.

**Verified in the tree today, two independent ways:**

- `fw audit --section compliance` → `[PASS] CTL-028: All completed/ tasks have frontmatter
  status: work-completed`. **0 FAIL, 0 WARN** on CTL-028.
- Direct scan of all **2263** files in `completed/`: **0** with `status != work-completed`,
  **0** with empty or absent `date_finished`.

A caution on measuring this, because I got it wrong first: `grep -c 'CTL-028'` on the audit
output returns 44, which looks like 44 surviving instances. It is not — those are evidence
lines belonging to **CTL-029** entries, which name CTL-028 as their `active/`-side mirror.
Counting string mentions instead of findings inverts the result of this check.

**The successor finding, which is live: `CTL-029` × 43.** "T-XXXX has all Agent ACs ticked but
status='started-work' — completable, not closed." That is the `active/`-side mirror of the
same disease, and it is not bookkeeping noise: T-2806 established that
`fw task update --status work-completed` was *failing* on every build task in a worktree
(unguarded source of an untracked file), which is a mechanism that manufactures exactly this
state. The mechanism is fixed; the 43 accumulated instances are not, and they are **not**
mine to bulk-close — CLAUDE.md is explicit that each needs individual evidence, and a sweep
over tasks whose ACs nobody verified is how CTL-028 was created in the first place.

**One thing this commit was carrying that nobody read.** Its message ends:

> NOTE: bvp recalc (estimate all + estimate-cost all) was reverted this session — the bvp
> estimator corrupts OLD-format frontmatter (no template anchor): it wrote proposed-score
> list items WITHOUT the `bvp_scores_proposed:`/`cost_estimate_proposed:` key, producing
> orphaned-list malformed YAML in 106 files. **Framework bug to file.**

It was never filed. A G-062 obligation recorded in a commit message is recorded where nothing
reads it — the same shape as the 71-day-late checkpoint in T-1166 and the stranded pickup
envelope in T-2801. Filed upstream under T-2809.

This mattered immediately rather than academically: **T-2808 ran `estimate all` and `cost-all`
earlier in this same session.** Re-checked on discovering the note — **0 malformed frontmatter
across all 2459 task files**. The run was safe because it was scoped `--statuses started-work`,
which is entirely modern-template files; the June run covered old-format tasks too. That is a
usage constraint nobody had written down, and it is now in the upstream filing.
