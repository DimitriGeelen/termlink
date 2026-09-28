---
id: T-3194
name: "Verify T-3191's upstream amendment landed; resolve its cross-session ownership"
description: >
  Verify T-3191's upstream amendment landed; resolve its cross-session ownership

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
created: 2026-09-28T12:57:26Z
last_update: 2026-09-28T12:57:26Z
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

# T-3194: Verify T-3191's upstream amendment landed; resolve its cross-session ownership

## Context

T-3191 appeared in two commits from this session without having been authored by
it (swept in by `git add -A`). It was left untouched and flagged. This task
answers the only questions that matter about it — did its claimed upstream
filing actually land, and is it duplicated work — without mutating it.

## Findings

### The filing landed, and it is well-formed

**`framework:pickup` offset 214**, 5115 bytes, read back from the live hub.
Frontmatter: `kind: bug-report-amendment`, `task: T-3191`, `severity: high`, and
— the property its AC claimed —
`amends: "framework:pickup offset 208 (T-2958, 010-termlink)"`, closing with
*"Please link, do not treat as a duplicate."* T-3191's ticked AC is truthful.
Topic currently holds 218 filings.

### It is an AMENDMENT, not duplicated work

This is the interesting answer, and it is the opposite of what the sweep
suggested. T-2958 (offset 208) reported that `fw task review` hardcodes `go`.
T-3191 adds what 208 did not carry: the same block ALSO pre-fills `--rationale`
from the task's own `**Recommendation:**` line, so the two together emit a
**self-contradictory** command rather than merely a questionable default — a
`go` verb whose pasted rationale begins "NO-GO". It measured the blast radius
T-2958 never did: **51 of 245 inception tasks** (18 NO-GO + 33 DEFER) receive a
contradictory command; 57 more get the literal `"your rationale"` fallback.

It also carries a scope CORRECTION that this project had wrong: the Watchtower
path is NOT affected — `web/blueprints/inception.py::record_decision()` reads
`decision` from the POST form and handles go/no-go/defer symmetrically. The
fault is CLI-only. An earlier note here misattributed it.

So two sessions did touch one defect, but not redundantly: the second measured
and corrected the first. The duplication risk was real (T-2800 class, arriving
same-branch where that check is structurally blind) and did not materialise,
because the author recognised it and filed as an amendment.

### G-096 is live and it caught me

T-3191 documents `termlink` silently retargeting a stale `runtime_dir` when
`TERMLINK_RUNTIME_DIR` is unset — every surface then reports the TOPIC missing,
and the obvious remediation (`channel create`) would recreate a fleet-shared
canonical topic as empty, destroying ~218 filings.

Confirmed here: hub pid 403200 runs with `TERMLINK_RUNTIME_DIR=/var/lib/termlink`,
read from `/proc/403200/environ`. **This session's post-compaction shell did not
carry the variable** — so every read in this task set it explicitly.

A second, self-inflicted instance of the same shape occurred during this task: an
invalid flag (`--from`, which does not exist; the flag is `--cursor`) exited rc=2
while stderr was swallowed by my own `2>/dev/null`, producing 0 bytes and no
message — indistinguishable from an empty topic. Re-run with stderr to a file, it
was immediately obvious. **Do not redirect stderr to /dev/null when probing a
rail whose failure mode is silence.**

### Disposition — recommended, NOT taken

T-3191 is `owner: agent`, carries no `revisit_at:` (so PL-389/G-095 do not
apply), has 4/4 Agent ACs ticked, and its one external claim is now verified. It
is complete work sitting in `active/` — the CTL-029 "completable, not closed"
shape, here legitimately.

It was **not** closed. mtime is ~2h old and no process holds the file, but
absence of a lock is not proof of abandonment, and closing another session's
in-flight task is the cross-session lost-update race already filed upstream at
offset 211. The close is a judgement about cross-session ownership and belongs
to the operator.

### Noted, not chased

`framework:pickup` offset 217 is a triage from 050-email-archive covering
offsets 200–216, described as carrying "three measurements that may be useful".
It post-dates our filings and may contain responses to them. Out of scope here.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] T-3191's claimed upstream amendment is located on `framework:pickup` by
      READ-BACK (T-2876: delivered ≠ received), not by trusting the task's own ticked
      AC. Its offset is recorded. If it is absent, that is reported as the finding —
      a task claiming a verified filing that did not land is worse than an unfiled one.
- [x] The amendment is checked for the property its own AC claims: that it cites the
      original T-2958 filing at offset 208, so upstream links the two rather than
      treating the second as a duplicate report.
- [x] Every read against `framework:pickup` is issued with `TERMLINK_RUNTIME_DIR` set
      to the live hub's value, read from `/proc/<hub-pid>/environ` rather than assumed.
      G-096: an unset variable silently redirects the CLI to a stale runtime dir where
      the topic appears not to exist, which is indistinguishable from data loss.
- [x] T-3191 itself is NOT edited, closed, or re-owned. It is another session's
      in-flight task; mutating it is the cross-session lost-update race already filed
      upstream at offset 211. The disposition is recorded as a recommendation only.
- [x] Whether T-3191 and T-2958/T-3190 constitute duplicated work is stated with
      evidence, since two sessions working one defect in one tree is the T-2800
      duplicate-work class arriving by a route that check cannot see (same branch).

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

# --- T-3194 verification ---
# Every leg sets TERMLINK_RUNTIME_DIR explicitly (G-096) and sends stderr to a file
# rather than /dev/null — swallowing stderr on this rail is how a hard failure
# becomes indistinguishable from an empty topic, which happened once in this task.

# 1. Positive control FIRST (T-3144): the hub is reachable and the topic is real.
#    Without this, legs 2-3 could "pass" against a silently-retargeted stale hub.
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 30 termlink channel info framework:pickup --json > /tmp/.t3194-info.json 2> /tmp/.t3194-info.err
python3 -c "import json;d=json.load(open('/tmp/.t3194-info.json'));assert d['count']>=218, d['count']"

# 2. Offset 214 exists, is attributable to T-3191, and amends offset 208.
TERMLINK_RUNTIME_DIR=/var/lib/termlink timeout 60 termlink channel subscribe framework:pickup --cursor 213 --limit 5 --json > /tmp/.t3194-sub.ndjson 2> /tmp/.t3194-sub.err
python3 -c "import json,base64,sys;rows=[json.loads(l) for l in open('/tmp/.t3194-sub.ndjson') if l.strip()];m=[r for r in rows if r.get('offset')==214];assert m,'offset 214 absent';b=base64.b64decode(m[0]['payload_b64']).decode('utf-8','replace');assert 'task: T-3191' in b,'not attributable to T-3191';assert 'offset 208' in b,'does not cite offset 208';assert 'bug-report-amendment' in b,'not an amendment'"

# 3. NEGATIVE control for leg 2: the same assertion must FAIL on a different
#    offset, otherwise leg 2 would pass against any envelope the parser returns.
python3 -c "import json,base64,sys;rows=[json.loads(l) for l in open('/tmp/.t3194-sub.ndjson') if l.strip()];m=[r for r in rows if r.get('offset')==213];assert m,'control offset 213 absent';b=base64.b64decode(m[0]['payload_b64']).decode('utf-8','replace');sys.exit(1 if 'task: T-3191' in b else 0)"

# 4. T-3191 was not mutated: still active, still started-work, still owner agent,
#    and still carries its 4 ticked Agent ACs.
test -f .tasks/active/T-3191-amend-t-2958-upstream-fw-task-review-emi.md
grep -q '^status: started-work' .tasks/active/T-3191-amend-t-2958-upstream-fw-task-review-emi.md
grep -q '^owner: agent' .tasks/active/T-3191-amend-t-2958-upstream-fw-task-review-emi.md
test "$(grep -c '^- \[x\]' .tasks/active/T-3191-amend-t-2958-upstream-fw-task-review-emi.md)" -eq 4

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

### 2026-09-28T12:57:26Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-3194-verify-t-3191s-upstream-amendment-landed.md
- **Context:** Initial task creation
