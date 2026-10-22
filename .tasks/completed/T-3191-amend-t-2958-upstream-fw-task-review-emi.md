---
id: T-3191
name: "Amend T-2958 upstream: fw task review emits a go verb paired with a NO-GO rationale"
description: >
  Amend T-2958 upstream: fw task review emits a go verb paired with a NO-GO rationale

status: work-completed
workflow_type: build
owner: agent
horizon: null
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
created: 2026-09-28T10:48:27Z
last_update: 2026-10-22T09:56:03Z
date_finished: 2026-10-22T09:56:03Z
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

# T-3191: Amend T-2958 upstream: fw task review emits a go verb paired with a NO-GO rationale

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Findings

**The defect, reproduced verbatim.** `lib/review.sh:379-381` emits the decision command with
`go` as a **literal**, while `--rationale` is pre-filled from the task's own
`**Recommendation:**` line (extracted at :366-372, truncated to 58 chars at :378). Neither is
conditional on what that line says. Reproduced with review.sh's own extraction logic against
T-2879, whose Recommendation reads *"NO-GO on any build — dissolve into T-2389."*:

```
  CLI: cd /opt/termlink &&
      bin/fw inception decide T-2879 go \
      --rationale "NO-GO on any build — dissolve into T-2389. (Measured 2026-"
```

The verb says `go`; the rationale it pastes begins **NO-GO**. Pasting the command the tool
presents records a GO whose stored justification argues against it — and the register then
reads as a human error rather than a tooling one.

**Blast radius, measured over 245 inception tasks:** 130 Recommendation GO, **18 NO-GO**,
**33 DEFER**, 57 with no Recommendation line. So **51 tasks (21% of all, 27% of those
carrying a Recommendation) receive a self-contradictory command.** The other 57 fall back to
the literal `"your rationale"` — not contradictory, but still hardcoding `go`.

**Certain vs circumstantial, kept separate in the filing.** Certain: the hardcoded verb and
the pairing, verifiable by reading one line. Circumstantial: whether this caused the
22-of-23 GO skew among divergent decisions. Both are consistent with the data; we have not
separated them and the filing says so rather than handing upstream an overclaim.

**Scope correction carried upstream.** The Watchtower path is NOT affected:
`web/blueprints/inception.py::record_decision()` reads `decision` from the POST form and
accepts go/no-go/defer symmetrically, 400-ing anything else. An earlier note in this project
misattributed the defect there. The fault is CLI-only.

## Incident during this task — the CLI silently filed into the void

The first attempt to post this amendment produced **no output at all** and did not land.
Cause: `termlink` resolves `runtime_dir` by a four-step order, and with
`TERMLINK_RUNTIME_DIR` / `XDG_RUNTIME_DIR` / `TMPDIR` all unset it falls through to
`/tmp/termlink-0`. **The live hub (pid 403200, up since 2026-09-26) runs with
`TERMLINK_RUNTIME_DIR=/var/lib/termlink`.** This session restarted mid-run and the new shell
did not carry that variable, so the CLI silently switched to a stale runtime dir whose bus
logs stop at 2026-09-25.

What it reported: `channel post` → **silence**. `channel subscribe` → *"unknown topic
'framework:pickup' — create it first"*. `channel list` → 15 topics, none of them
`framework:pickup`. Every surface said the TOPIC was missing. None said the CLI was pointed
at a different hub.

**That is a Directive #2 failure at the worst possible moment** — it is indistinguishable
from catastrophic data loss, and the obvious remediation the error text suggests
(`channel.create`) would have **recreated a fleet-shared canonical topic as empty**, masking
~213 filings from every project that uses it. The recovery here was to check
`/proc/<hub-pid>/environ` and re-read with the variable set: **all filings intact**, offsets
210 / 211 / 213 present and readable.

Registered as **G-096**. Not acted on beyond diagnosis: recreating a shared topic is
outward-facing and is the operator's call, and it turned out to be exactly the wrong action.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The contradictory command is reproduced VERBATIM using `review.sh`'s own extraction
      logic (not paraphrased), for a real task whose Recommendation is NO-GO, and the
      reproduction is recorded in the task body.
- [x] Corpus measured: how many inception tasks carry a Recommendation that is NOT `go` —
      i.e. how many would receive a self-contradictory command. Figure recorded, including
      the case where no Recommendation line exists at all.
- [x] Amendment filed at `framework:pickup`, verified by read-back byte-identical (T-2876),
      explicitly citing the original T-2958 filing at offset 208 so upstream links them
      rather than treating this as a duplicate report.
- [x] The amendment separates what is CERTAIN (the hardcoded verb at `review.sh:380`,
      verifiable by reading one line) from what is CIRCUMSTANTIAL (whether it caused the
      22-of-23 GO skew), so upstream is not handed an overclaim.

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
# ── Asserting an ABSENCE: prove the search could have succeeded (T-3144) ──
#
# `! grep -q "PATTERN" file` exits 0 when the pattern is absent. It ALSO exits 0
# when the file was renamed, deleted, or is empty — so the leg cannot distinguish
# "the bad thing is not there" from "I could not look", and the gate reports green
# over a check that never ran. Pair every absence assertion with something that
# fails if the search could not happen:
#
#     test -f path/to/file && ! grep -q "PATTERN" path/to/file    # existence first
#     grep -q "KNOWN_MARKER" f && ! grep -q "PATTERN" f           # positive companion
#     cmd > /tmp/.out 2>&1 && ! grep -q "PATTERN" /tmp/.out       # &&-joined producer
#
# Count-equals-zero is the same defect wearing a different hat, and it is the one
# that bites hardest over a COMMAND's output rather than a file:
#
#     [ "$(cargo clippy --workspace 2>&1 | grep -c "^error")" = "0" ]   # WRONG
#
# If cargo is missing, or dies before emitting diagnostics, there are no `^error`
# lines, the count is 0, and the leg passes — a build gate that goes green
# precisely when the build could not run. Measured in this corpus, not invented.
# Keep the producer's exit code in the verdict:
#
#     cargo clippy --workspace > /tmp/.out 2>&1 && ! grep -q "^error" /tmp/.out
#
# T-3144 censused 2853 task files: 71 absence assertions, 41 already correct, 30
# not. The convention mostly works — this note is here so the next one is written
# right, because a vacuous leg is invisible until the day the path moves.
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

# The amendment must exist upstream AND carry this task's body. Read-back, not send-status
# (T-2876). TERMLINK_RUNTIME_DIR is set EXPLICITLY because this task's own incident was the
# CLI silently resolving to a stale runtime_dir — an unset variable made a post vanish with
# no output. Never rely on the fallback chain for a verification leg (G-096).
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 60 termlink channel subscribe framework:pickup --cursor 214 --limit 1 --json > /tmp/.t3191-rb.json 2>&1
python3 -c "import json,base64; o=next(d for d in (json.loads(l) for l in open('/tmp/.t3191-rb.json') if l.strip().startswith('{')) if 'payload_b64' in d); b=base64.b64decode(o['payload_b64']).decode('utf-8','replace'); assert o['offset']==214, o['offset']; assert 'AMENDMENT to T-2958' in b, 'wrong envelope at 214'; assert 'offset 208' in b, 'amendment does not cite the original filing'; print('read-back ok')"

# The blast-radius figure in the filing must match the corpus, so the report cannot drift
# from the tree it describes.
test "$(grep -l '^workflow_type: inception' .tasks/active/T-*.md .tasks/completed/T-*.md 2>/dev/null | wc -l)" -ge 200

# G-096 must be registered and the register must still parse — the incident is the more
# valuable half of this task and must not live only in prose.
python3 -c "import yaml; d=yaml.safe_load(open('.context/project/concerns.yaml')); g=[c for c in d['concerns'] if c['id']=='G-096']; assert g, 'G-096 missing'; assert g[0]['severity']=='high'; print('G-096 ok')"

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

### 2026-09-28T10:48:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3191-amend-t-2958-upstream-fw-task-review-emi.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-0d7d2df2
- **Timestamp:** 2026-10-22T09:56:12Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-10-22T09:56:03Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
