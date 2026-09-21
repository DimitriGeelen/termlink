---
id: T-3035
name: "Triage framework:pickup@131 — ring20's fw doctor interpreter-prefix false-FAIL"
description: >
  Ring20 filed at framework:pickup offset 131 that fw doctor false-FAILs 'script not
  found: python3/bash' on interpreter-prefixed hooks. Verify the claim against the
  actual vendored code, route it to the correct owner per G-062, and unblock the pickup
  canary ack.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
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
created: 2026-09-21T08:15:32Z
last_update: '2026-09-21T08:17:30Z'
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
  - ts: '2026-09-21T08:17:30Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 3
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=3 
      (body:component-discoverability); D4=2 (body:env-class-handled); 
      F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
---

# T-3035: Triage framework:pickup@131 — ring20's fw doctor interpreter-prefix false-FAIL

## Context

`framework:pickup@131` (pickup_id P-2026-0920-001, from proxmox-ring20-management, their
T-1879) reports that `fw doctor` false-FAILs `script not found: python3/bash` on
interpreter-prefixed hooks. It is the first genuine inbound filing on the topic in this
window and had sat untriaged across two sessions — the G-063 class the pickup canary exists
to surface.

**MEASURED-VERDICT: CONFIRMED by code reading; NOT reproduced on this host.**

Measured against our own vendored copy rather than accepted on the filing's say-so (PL-367).
Ring20 cited `bin/fw` ~L1360-1393; in our revision the same code sits at
`.agentic-framework/bin/fw:2466-2508` — the line numbers differ because the vendored
revisions differ, the logic is identical:

- `bin/fw:2468-2472` — `script_path = parts[0]`, then the first token without an `=`. For
  `python3 "$CLAUDE_PROJECT_DIR/scripts/x.py"` that is the bare interpreter name `python3`.
- `bin/fw:2475` — a special case exists for `fw hook …` invocations, exactly as ring20 said.
- `bin/fw:2504` — the general branch then does `os.path.exists('python3')`, which is False,
  and appends `script not found: python3`. Their diagnosis is correct in full.

**Why it does not fire here.** Every hook in this project's `.claude/settings.json` is
`${CLAUDE_PROJECT_DIR}/.agentic-framework/bin/fw hook <name>`, so all 27 take the `is_fw`
branch. `fw doctor` reports `OK  Hook configuration valid (27 hooks in 20 matchers across
4 events)` — rc 0. The defect is real and latent here, not active.

**Adjacent gap ring20 did not name.** The `${CLAUDE_PROJECT_DIR}` expansion added by T-2709
lives at `bin/fw:2491`, INSIDE the `is_fw` branch. The general branch at `bin/fw:2504`
stats `script_path` with no expansion at all. So any non-`fw` hook written with the
portable placeholder — the form Claude Code documents — also false-FAILs, by a second
mechanism, and would survive a fix that only special-cases interpreters. Worth folding
into the same patch.

**REPLY-CONFIRMED-AT: framework:pickup@135** — read back from the topic, carrying
`in_reply_to: 131` and `from_project: 010-termlink`. Not taken from the post's own report,
which said `status: delivered-unconfirmed` (T-2876: a send reports queued, not received).

**CANARY-ACKED-AT: offset 135** — `.context/working/.framework-pickup-canary.seen-offset`
now reads 135 and `check-framework-pickup-freshness.sh` exits 0 with
`healthy — all filings surfaced`. Acked only AFTER @131 was triaged and answered, so no
genuine filing was swept away with our own root-attributed echoes at @126/@128.

**Finding, not fixed here: the canary log is deaf.**
`.context/working/.framework-pickup-canary.log` is 64,518 bytes of accumulated historical
firings. Per T-2685 the "empty log = healthy" convention is a one-bit channel, and once
dirtied a subsequent genuine finding appends to an already-non-empty file and changes
nothing an operator can see. The ack does not clear it and was never meant to. This task's
verification originally asserted an empty log, which was over-reach on my part — the line
was corrected to assert what the ack actually establishes (exit 0 + a recorded
seen-offset) rather than truncating 64KB of forensic history to make my own check pass.
Rotating the log is operator hygiene with a forensic cost; surfaced rather than done
unilaterally.

**Routing (G-062).** `bin/fw` is vendored. No local patch was made; this is upstream's to
land. Our contribution is independent confirmation on a second revision, the adjacent
finding, and getting both back to the filer.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Ring20's claim at `framework:pickup@131` is MEASURED against the actual vendored
      code, not accepted on the filing's say-so (PL-367): the cited region of
      `.agentic-framework/bin/fw` is read and the verdict — confirmed, partially
      confirmed, or not reproduced — is recorded in this task with file:line evidence
- [x] Ownership is routed per G-062: the defect sits in vendored code, so NO local patch
      is made (verified by a clean porcelain under `.agentic-framework/`) and the routing
      decision is stated explicitly rather than left implicit
- [x] A reply is delivered to the filer on the rail and CONFIRMED by reading the offset
      back from the topic — never asserted from the post's own `success`/`delivered`
      report, which means queued, not received (T-2876). The reply offset is recorded here
- [x] The pickup canary is acked ONLY after the above, and the resulting seen-offset is
      recorded in this task — so the topic goes quiet without an untriaged genuine filing
      being swept away with it (the exact G-063 miss the canary exists to prevent)

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

# T-3035. Safe redirect form only (L-387): never `cmd | grep -q PAT`.
# The measured verdict on ring20's claim must be recorded, not merely asserted.
grep -q "MEASURED-VERDICT:" .tasks/active/T-3035-triage-frameworkpickup131--ring20s-fw-do.md
grep -q "bin/fw:" .tasks/active/T-3035-triage-frameworkpickup131--ring20s-fw-do.md
# G-062: the vendored tree must carry no local patch.
git status --porcelain .agentic-framework/ > /tmp/.t3035-g 2>&1
test ! -s /tmp/.t3035-g
# T-2876: the reply offset must be one read back from the topic, not one the post claimed.
grep -q "REPLY-CONFIRMED-AT:" .tasks/active/T-3035-triage-frameworkpickup131--ring20s-fw-do.md
# The canary must be quiet, and the offset it was acked to must be recorded.
grep -q "CANARY-ACKED-AT:" .tasks/active/T-3035-triage-frameworkpickup131--ring20s-fw-do.md
bash scripts/check-framework-pickup-freshness.sh > /tmp/.t3035-c 2>&1
grep -q "healthy" /tmp/.t3035-c
test -s .context/working/.framework-pickup-canary.seen-offset

## RCA

**Symptom:** A peer's genuine bug report sat unprocessed on `framework:pickup` across two
sessions. (The reported defect itself — `fw doctor` false-FAIL — is ring20's symptom, not
ours; it does not fire on this host.)

**Root cause of the triage delay:** the pickup canary fires on "filings newer than the
last-acked offset", and the only way to quiet it is `--ack`, which advances the marker past
EVERYTHING newer. Two of our own filings (P-079 @126, P-081 @128) landed
`metadata.from_project: root` rather than `010-termlink`, so the T-2816 self-filter did not
suppress them. Acking to quiet our own echoes would have swept ring20's genuine filing away
with them — so the correct move was to not ack, which left the canary firing and the filing
untriaged. The safety property and the quiet-the-noise property were in direct conflict.

**Why structurally allowed:** the canary's ack is a single monotonic watermark over a
topic carrying two distinct populations (ours, theirs). It can express "I have seen
everything up to N" but not "I have triaged this one". With self-attribution broken, the
watermark could not be advanced without loss.

**Prevention:** partially structural, partially not. T-3034's filing and this one both set
`metadata.from_project=010-termlink` explicitly, so future own-filings are suppressed by
the T-2816 filter and the watermark can be advanced freely. That removes the conflict going
forward but does not retroactively fix @126/@128, and it does not give the canary
per-filing triage state — a watermark still cannot distinguish "triaged" from "skipped".
That limitation is recorded here rather than claimed as solved.

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

### 2026-09-21T08:15:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3035-triage-frameworkpickup131--ring20s-fw-do.md
- **Context:** Initial task creation
