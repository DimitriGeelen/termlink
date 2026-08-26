---
id: T-2837
name: "Cross-checkout operator actions blocking the t2687 merge"
description: >
  Two guard-layer FAILs gate eight finished tasks and can only be cleared from checkouts this worktree cannot reach (T-559 project boundary): recover three dangling corpus_*.py into the vendored framework from /opt/termlink, and renumber the T-2690/91/92 ID collisions on two sibling worktrees. Carried as Human ACs so they surface on the /approvals route instead of living only in a session transcript.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T21:51:44Z
last_update: 2026-08-23T21:53:59Z
date_finished: null
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
---

# T-2837: Cross-checkout operator actions blocking the t2687 merge

## Context

Four guard-layer checks currently FAIL, and those FAILs gate eight otherwise-finished
tasks from finalizing. Two were cleared from this worktree (T-2836 repaired 29
unreadable episodics; T-2714 filed the audit hook-path defect upstream). The other
two **cannot** be cleared from here, and the reason is structural rather than a
matter of effort:

- **`check-framework-tracking-drift` axis B** reports three DANGLING refs —
  `tools/corpus_spec.py`, `tools/corpus_lint.py`, `tools/corpus_explain.py`, sourced
  from `.agentic-framework/bin/fw:4901-4909` and `designer.sh:302`. Per T-2806/T-2817
  a dangling ref can only be fixed in the checkout that still HAS the files, where
  they show as UNTRACKED instead. That is `/opt/termlink`; a worktree materialises
  only tracked files, so they are not here to commit.
- **`check-task-id-collisions` axis A** reports T-2690/91/92 claimed by two branches
  (`worktree-charter-review-2026-0814`, `worktree-governance-canary-signal`) as
  *different tasks*, plus 25 duplicated files. Renumbering must happen on those
  branches. T-559 keeps this session inside its own project boundary.

Delegating these over termlink was attempted and is **not available**: every session
on this host shares one identity file, so `agent contact termlink-agent` resolved
`peer_fp == my_fp` and the DM looped back to a self-topic. That is the same
identity collapse behind T-2690's ambiguous `whoami`. Cross-host peers have distinct
fingerprints but none of them owns these checkouts.

So the actions are genuinely the operator's, and they are carried here as Human ACs
specifically so `/approvals` section D surfaces them — a session transcript is not a
queue, and both items had already been re-derived more than once.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The task states, for each action, why it cannot be done from this worktree rather than merely that it was not done
- [x] Each Human AC carries a single-line copy-pasteable command prefixed with `cd <path> &&` per T-609
- [x] The task is `owner: human` so it lands in `/approvals` section D (unchecked Human ACs) rather than an agent queue

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

- [ ] [REVIEW] The three `corpus_*.py` files are recovered into the vendored framework and committed from `/opt/termlink`

  **Steps:**
  1. `cd /opt/termlink && bash scripts/check-framework-tracking-drift.sh`
  2. Confirm the three files appear under **UNTRACKED** there (not DANGLING — that is this worktree's view of the same fact).
  3. Read them before committing. Framework files carried across disks can hold machine-local paths or credentials, and a commit is permanent. T-2806's scan flagged a bare 64-hex in `policy/designer-pin.yaml` that turned out to be a `sha256:` reproducibility pin — a true pattern match and a false risk. That judgement is yours; the check surfaces, it does not decide.
  4. `cd /opt/termlink && git add -f .agentic-framework/tools/corpus_spec.py .agentic-framework/tools/corpus_lint.py .agentic-framework/tools/corpus_explain.py`
  5. Commit, then push to **OneDev only** — never GitHub.

  **Expected:** `bash scripts/check-framework-tracking-drift.sh` exits 0 in this worktree afterwards, with `dangling_count: 0`.

  **If not:** axis B's count is a lower bound — recovering these may expose further refs that were invisible while their parent was missing (T-2806 saw exactly this). Expect to converge over two rounds, not one. If a file genuinely does not exist on any disk, say so and the reference should be deleted rather than satisfied.

- [ ] [REVIEW] T-2690/91/92 are renumbered on the two sibling branches before any merge

  **Steps:**
  1. `cd /opt/termlink && bash scripts/check-task-id-collisions.sh`
  2. For each colliding ID, confirm from the printed filenames that the two branches hold *different tasks* (a cherry-pick — same ID, same task — is not reported and needs nothing).
  3. Renumber on the branches that own them, picking IDs above the highest claimed on **any** branch. `fw task create` has no `--id` flag, so this is a manual rename plus a reference sweep. T-229 is the worked example.

  **Expected:** the check exits 0; no ID is claimed by two branches beyond the merge base.

  **If not:** do not merge. Merging a collision silently discards one of the two tasks, which is the failure T-229 recorded in March 2026 and that recurred in August because nothing enforced it. Note also that this task's own ID was allocated by the same racing counter — verify T-2837 is not itself colliding.

- [ ] [REVIEW] P-043 is disposed of deliberately — acted on or dropped, not left stranded

  **Steps:**
  1. `cd /opt/termlink/.claude/worktrees/t2687-pickup-failopen && cat .context/pickup/auto-deferred/P-043-bug-report.yaml`
  2. It reports two framework bugs blocking every inception decision, with file:line references. **Both are already resolved:** BUG 2 was independently re-discovered and re-fixed as T-2304, and BUG 1 (`update-task.sh:791`) is filed upstream at `framework:pickup` offset 17. So its content is spent.
  3. My recommendation is therefore to **drop it** — delete the envelope — rather than file anything new from it.
  4. This is your call, not mine: `check-pickup-deferred-freshness.sh` detects and never drains, precisely because discarding is a judgement about whether the work still matters. An agent auto-draining would turn a visible backlog into a silent one, which is the trade the check exists to reverse.

  **Expected:** `bash scripts/check-pickup-deferred-freshness.sh` exits 0, and the guard layer drops from 3 firing to 2.

  **If not:** if you would rather keep it, that is fine — but it should then carry a breadcrumb naming a real blocking task, otherwise `fw pickup promote-deferred` can never promote it and it is stranded again by construction. The cost of leaving it is measured: this envelope sat 76 days while one of the bugs it named was solved twice.

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

## Recommendation

**Recommendation:** Apply (c) locally now - reclassify framework-tool-referenced-but-never-vendored as KNOWN-GAP rather than DANGLING in check-framework-tracking-drift - and for P-043 file BUG 1 (disposition-gate vocabulary mismatch) as its own task, then drop the envelope deliberately. Together these clear both fired guards and unblock seven agent task closures today, without pretending missing files exist. Detail below.

Two operator decisions. Both are now fully evidenced, and together they are the only
thing standing between us and SEVEN agent task closures (T-2684, T-2685, T-2686,
T-2688, T-2693, T-2699, T-2758 — each fails exactly one verification, and it is the
same one: `bash scripts/run-guard-layer.sh --quiet`, 62/64 pass, these 2 fire).

### Decision 1 — corpus_*.py (check-framework-tracking-drift)

RECOMMEND: do NOT attempt local recovery. The AC premise is wrong and chasing it wastes
a session. Verified: no git history on any branch, nothing on disk in any worktree,
and neither ./tools/ nor .agentic-framework/tools/ exists. They were never vendored.

The dangling references are in the vendored fw itself (bin/fw:4901/4907/4909/7686) and
agents/designer/designer.sh:302. Every `fw corpus ...` verb is therefore dead in every
consumer install; bin/fw:7686 hides its failure behind `2>/dev/null || true`.

Filed upstream 2026-08-25 → framework:pickup offset 40 (G-062: not patched locally).

Your call is which unblock you want:
  (a) WAIT for the upstream vendor-manifest fix (ships tools/, or gates the verbs
      honestly). Correct, but we do not control the timing.
  (b) DECIDE the corpus verbs are not wanted in consumers, and have upstream remove
      them from the vendored fw. Also correct, and smaller.
  (c) Interim, locally: treat "framework tool referenced but never vendored" as a
      KNOWN-GAP class in check-framework-tracking-drift rather than DANGLING, so the
      guard stops blocking unrelated work while (a)/(b) is pending.

I recommend (c) now + (a) or (b) upstream. (c) is first-party (scripts/), so it is ours
to make, and it unblocks the seven closures today without pretending the files exist.

### Decision 2 — P-043 disposal (check-pickup-deferred-freshness)

RECOMMEND: file one new task, then drop the envelope deliberately.

Root cause of the 78-day strand is a PICKUP-ID COLLISION, not neglect: there are two
different `P-043-bug-report.yaml` files —
  .context/pickup/processed/     P-043 from T-2155 (budget-narration bug) — handled
  .context/pickup/auto-deferred/ P-043 from T-2018 (this one)          — stranded
Processing the first marked the ID done, so the second could never be promoted. Same
root-cause class as the task-ID collision (T-2800 / G-007): IDs allocated by scanning
one directory only.

Its contents are two bugs, and they are NOT equally live:
  BUG 2 (heredoc __file__ under python3 stdin) — ALREADY TRACKED as active task T-2232.
  BUG 1 (T-2190 disposition-gate vocabulary mismatch, update-task.sh:787 — gate accepts
         answered|deferred|dissolved, authors write resolved|partial|open, silent until
         --status work-completed runs) — NOT tracked anywhere I can find.

So: create a task for BUG 1, then drop P-043 deliberately. Nothing is lost — half of it
is already T-2232 and the other half becomes its own task with a real owner.

Note BUG 1 is the same shape as two things filed this session: a gate whose vocabulary
does not match what authors actually write (cf. pickup offset 39, the RCA gate that
classifies guard tasks as bug-class because their titles contain "regression").

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

## Evidence re-measured 2026-08-26 (agent; ticks nothing)

Re-measured against the tree rather than re-read from this file. Two of the three
recorded blockers have MOVED since they were written.

**AC#1 — condition now TRUE.** All three files are present in the vendored framework:

    .agentic-framework/tools/corpus_spec.py       PRESENT
    .agentic-framework/tools/corpus_lint.py       PRESENT
    .agentic-framework/tools/corpus_explain.py    PRESENT

and `bash scripts/check-framework-tracking-drift.sh` exits 0 — "no load-bearing drift
(2634 file(s) scanned, 59 reference(s) resolved)", zero DANGLING. The DANGLING list at
:310-312 of this task is STALE; it describes a state the tree has left. Nothing here
ticks the AC — the measurement is mine, the acceptance is yours.

**AC#2 — the premise is wrong, and in your favour.** This AC says "the two sibling
branches". Measured with `git ls-tree -r --name-only <branch> .tasks/`:

    main                                T-2690 purpose-review-4 · T-2691 whoami · T-2692 macos
    integration/t2687-trial             identical to main — NO collision
    worktree-charter-review-2026-0814   identical to main — NO collision
    worktree-governance-canary-signal   T-2690 canary-stderr-sink-severs-detection
                                        T-2691 heartbeat-proves-scheduling-not-completion
                                        T-2692 static-check-allowlists-untracked
                                        ^^^ DIFFERENT tasks under the same three ids

So exactly ONE branch still collides, not two. `worktree-governance-canary-signal` is
the only merge that would collide three task ids against main. The renumbering already
done in this checkout (T-2815←T-2690, T-2819←T-2694, T-2822←T-2698) is unrelated to
main's own T-2690/91/92, which are legitimate completed tasks — worth stating because
the two look identical from the id alone and invite exactly the wrong fix.

**AC#3 — genuinely still open.** `check-pickup-deferred-freshness` exits 1; P-043 is
still stranded. Both of its blockers are handled (BUG 2 carried upstream, BUG 1 filed
as T-2801, whose own AC asserts this repo reports P-043 as STRANDED), so the guard is
firing correctly and holding the envelope visible for your disposal. That disposal is a
decision, not a measurement, and it is the one thing here nobody can do for you.

One correction to the record while re-measuring it: `.context/pickup/` holds TWO
different envelopes numbered P-043 — task_id T-2018 in `auto-deferred/`, T-2155 in
`processed/`. Comparing filenames rather than contents makes the stranded one look like
processed residue.


## Updates

### 2026-08-23T21:51:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2837-cross-checkout-operator-actions-blocking.md
- **Context:** Initial task creation

### 2026-08-23T21:53:59Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## 2026-08-25 — the blocking is now measured, and both halves are THIS task

Traced from the other end: seven agent-owned tasks (T-2684, T-2685, T-2686, T-2688,
T-2693, T-2699, T-2758) all have every Agent AC ticked and all REFUSE to close. Each
fails exactly one verification, and it is the same one:

    bash scripts/run-guard-layer.sh --quiet     -> exit 1

run-guard-layer.sh has 64 members. 62 pass. The 2 that FIRE are:

  1. check-framework-tracking-drift.sh
       DANGLING $FRAMEWORK_ROOT/tools/corpus_explain.py
       DANGLING $FRAMEWORK_ROOT/tools/corpus_lint.py
       DANGLING $FRAMEWORK_ROOT/tools/corpus_spec.py
     == this task Human AC #1 (corpus_*.py)

  2. check-pickup-deferred-freshness.sh
       STRANDED P-043-bug-report.yaml (deferred 78 days ago)
     == this task Human AC #2 (P-043 disposal)

So the two operator actions this task exists to unblock are, right now, blocking seven
unrelated task closures. That is the cost of the delay, measured rather than asserted.

CORRECTION TO HUMAN AC #1 PREMISE (evidence, not a tick)
  The AC says recover corpus_*.py "from /opt/termlink". They are NOT recoverable here:
    - `git log --all -- *corpus_*.py`            -> no history, any branch
    - `find /opt/termlink -name corpus_*.py`     -> nothing on disk, incl. all worktrees
    - ./tools/ and .agentic-framework/tools/     -> neither directory exists
  They are framework-owned files the vendor manifest never ships. The dangling refs are
  in the VENDORED fw itself (bin/fw:4901, 4907, 4909, 7686) plus
  agents/designer/designer.sh:302 (which resolves against $PROJECT_ROOT, not
  $FRAMEWORK_ROOT -- two different roots for the same three files).

  Consequence: every `fw corpus ...` verb is dead in every consumer install, and
  bin/fw:7686 hides it behind `2>/dev/null || true`.

  Filed upstream 2026-08-25: framework:pickup offset 40 (G-062, not patched locally).
  Same class as T-2546.

  => Recovery cannot be done from this checkout. The action is an upstream vendor-manifest
     fix, or a deliberate decision to drop the corpus verbs from the vendored fw.

## 2026-08-25 — AC#1 (corpus_*.py) is now satisfiable; earlier premise corrected

**Recommendation:** tick AC#1. The three files are in the vendored framework and
committed from `/opt/termlink` (`0bd97f38a`), and the verbs they back now run. Evidence
below. The AC is left unchecked because only the operator ticks Human ACs.

### The earlier finding was half right, and the wrong half mattered

An earlier pass on this task concluded the three `corpus_*.py` files were **never vendored** —
no git history, nothing on disk, `tools/` absent — and filed that upstream (framework:pickup
offset 40). The git half was correct. The disk half was not, and the reason is the interesting
part.

`.gitignore:37` blanket-ignores `.agentic-framework/*` and re-includes a fixed allowlist:
`agents/ bin/ docs/ lib/ policy/ web/ .tasks/` plus named files. The file's own comment states
that list "is exactly what `git ls-files .agentic-framework` already tracks" — it was generated
from a snapshot in time. **Anything upstream adds later is dropped silently, forever, on every
vendor event.**

`tools/` is not on that list. So the corpus scripts arrive on disk with each vendor event and
are immediately invisible to git. `git log -- '.agentic-framework/tools/*'` is empty, which
reads exactly like "never vendored" and is not the same claim.

Upstream had already noticed the consumer-side symptom: `bin/fw:400-401` carries a comment
naming this precise failure — `python3: can't open file '<proj>/.agentic-framework/tools/corpus_explain.py'`.

### What was actually dropped

| entry | files | invoked by |
|---|---|---|
| `tools/` | 30 | `bin/fw:4901/4907/4909` → `corpus_{lint,explain,spec}.py` |
| `vendor/` | 1 | designer HTML asset |
| `status-transitions.yaml` | 1 | status-transition validation |

### Verified, not assumed

Extended the allowlist with `!.agentic-framework/tools/`, `!.agentic-framework/vendor/`,
`!.agentic-framework/status-transitions.yaml`, then ran the verb that used to be a dangling
reference:

```
$ fw corpus explain --search "verification"
  corpus map: aef-task-lifecycle (v3) — matches: completion gates: Agent ACs (P-010) + ## Verification commands (P-011)
    read it: fw corpus explain aef-task-lifecycle
$ echo $?
0
```

`corpus_lint.py` 857 lines, `corpus_explain.py` 227, `corpus_spec.py` 799 — all now tracked.
33 files committed in `0bd97f38a`.

### How they arrived

A `fw upgrade` re-vendored `.agentic-framework/` and brought `tools/` with it. That same
re-vendor also deleted three of our four recorded divergences (`_resolve_hook_path` 4→0,
`pickup_dedup_hash` 5→3, `pickup_create_inception` 4→3), so the vendored tree was restored from
git and only the previously-ignored directories were kept. Divergence markers re-verified
intact afterwards. This is why the gitignore fix is the durable part: without it the next
vendor event drops `tools/` again and the corpus verbs break again, silently.

### Bearing on the other ACs

None. AC#2 (T-2690/91/92 renumbering) and AC#3 (P-043 disposal) are unaffected by this.
