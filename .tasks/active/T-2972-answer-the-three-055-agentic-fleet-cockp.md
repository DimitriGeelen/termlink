---
id: T-2972
name: "Answer the three 055-agentic-fleet-cockpit consults: topology, priority routing, orchestration model"
description: >
  Answer the three 055-agentic-fleet-cockpit consults: topology, priority routing, orchestration model

status: work-completed
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
# demo_target: true               # T-2286: optional — marks task as reserved for an orchestrated demo
#                                 # worker (e.g. arc-010 HM-A dispatches via mcp__fw__work_on). When set,
#                                 # `fw work-on T-XXX` refuses unless --i-am-demo-orchestrator (CLI) or
#                                 # FW_I_AM_DEMO_ORCHESTRATOR=1 (env) is passed. Prevents the parent
#                                 # session from consuming the captured→started-work transition the demo
#                                 # worker expects to drive. Origin OBS-057.
created: 2026-09-17T21:40:29Z
last_update: 2026-09-18T15:38:40Z
date_finished: 2026-09-18T15:38:40Z
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

# T-2972: Answer the three 055-agentic-fleet-cockpit consults: topology, priority routing, orchestration model

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Every claim in each reply is grounded in a cited path, struct, or a search that returned zero hits — no assertion that a design exists without a pointer to it
- [x] The topology answer states the 5-level model does not exist, and names the three real levels plus the two tag conventions, citing crates/termlink-session/src/registration.rs
- [x] The priority-routing answer reports the negative result (zero hits repo-wide) rather than hedging
- [x] The orchestration answer distinguishes registered `termlink spawn` sessions from in-process AEF sub-agents, and states that TERMLINK_PARENT_SESSION is a caller convention with zero references in crates/
- [x] Three draft replies exist under .context/working/drafts/ and are shown to the operator before anything is posted
- [x] Nothing is posted to any DM topic without explicit operator approval

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
- [ ] [RUBBER-STAMP] Post the three approved replies to the 055 cockpit DM topics (outward-facing; sovereignty over what leaves this project is yours — the agent AC above forbids the agent posting)
  **Steps:**
  1. Read the three drafts: `cd /opt/termlink && cat .context/working/drafts/T-2972-reply-A.txt .context/working/drafts/T-2972-reply-B.txt .context/working/drafts/T-2972-reply-C.txt`
  2. Post A (answers the 09-16 13:34 consult): `cd /opt/termlink && termlink channel post dm:8e6fd77ec6f74b37:d1993c2c3ec44c94 --msg-type chat --payload "$(cat .context/working/drafts/T-2972-reply-A.txt)" --metadata from_project=010-termlink --metadata _thread=T-2972 --json`
  3. Post B (answers the 09-16 13:55 consult): `cd /opt/termlink && termlink channel post dm:3bba15e681b3a078:d1993c2c3ec44c94 --msg-type chat --payload "$(cat .context/working/drafts/T-2972-reply-B.txt)" --metadata from_project=010-termlink --metadata _thread=T-2972 --json`
  4. Post C (answers the 09-17 20:54 T-064 consult): `cd /opt/termlink && termlink channel post dm:3bba15e681b3a078:d1993c2c3ec44c94 --msg-type chat --payload "$(cat .context/working/drafts/T-2972-reply-C.txt)" --metadata from_project=010-termlink --metadata _thread=T-2972 --json`
  **Expected:** each command prints a JSON envelope with `"ok": true` and an `offset`; `termlink channel subscribe dm:3bba15e681b3a078:d1993c2c3ec44c94 --json` shows the new offsets with `from_project: 010-termlink`
  **If not:** an auth error → `termlink fleet doctor`; a queued outcome (`Queued`) means the hub blipped and the offline queue will flush — check `/queue-status`. If you want wording changed first, edit the draft file and re-run the post; nothing is sent until you run these.

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

# AC5: three drafts exist and are non-empty
test -s .context/working/drafts/T-2972-reply-A.txt && test -s .context/working/drafts/T-2972-reply-B.txt && test -s .context/working/drafts/T-2972-reply-C.txt
# AC2: topology answer denies the 5-level model and cites registration.rs (A and B)
grep -q "DOES NOT EXIST" .context/working/drafts/T-2972-reply-A.txt && grep -q "registration.rs" .context/working/drafts/T-2972-reply-A.txt && grep -q "registration.rs" .context/working/drafts/T-2972-reply-B.txt
# AC3: priority-routing answer states the negative result as zero hits (A and B)
grep -qi "zero hits" .context/working/drafts/T-2972-reply-A.txt && grep -qi "zero hits" .context/working/drafts/T-2972-reply-B.txt
# AC4: orchestration answer distinguishes spawn sessions from sub-agents and states TERMLINK_PARENT_SESSION has zero crates/ references
grep -q "termlink spawn" .context/working/drafts/T-2972-reply-C.txt && grep -q "TERMLINK_PARENT_SESSION" .context/working/drafts/T-2972-reply-C.txt && grep -q "ZERO references in crates/" .context/working/drafts/T-2972-reply-C.txt
# AC4 ground truth: the claim in C is still true of the tree
test "$(grep -rl TERMLINK_PARENT_SESSION crates/ | wc -l)" = "0"
# Two-facts rule (operator decision 2026-09-18) present in all three
grep -q "git-common-dir" .context/working/drafts/T-2972-reply-A.txt && grep -q "git-common-dir" .context/working/drafts/T-2972-reply-B.txt && grep -q "git-common-dir" .context/working/drafts/T-2972-reply-C.txt
# Gaps the replies cite are registered (register-first)
python3 -c "import yaml; ids=[c['id'] for c in yaml.safe_load(open('.context/project/concerns.yaml'))['concerns']]; assert 'G-090' in ids and 'G-091' in ids"
# AC6 (no post without operator approval) is not mechanically checkable here: the operator is the one who posts
# (Human AC), so a "no T-2972 post exists" probe would fail on the correct outcome. AC6 is attested by the
# Human AC being the only posting path in this task.

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
**Recommendation:** GO — post all three replies as drafted (Human AC carries the exact commands).
**Rationale:** The three consults have sat unanswered since 09-16/09-17 because the cockpit's mailbox has no reader on our side (832's 2026-09-03 diagnosis); every day unanswered is a day the cockpit may build on the non-existent 5-level model. Every claim in the drafts is source-cited or a recorded zero-hit search, the two-facts project rule you decided on 2026-09-18 is encoded in all three, and the two gaps the replies mention are registered (G-090, G-091) so the drafts promise nothing that is not on the register. The one open judgement is reply C's `parent=<session-id>` tag proposal — it is phrased as a proposal to converge on, not a commitment; if you do not want to offer it, delete that paragraph before posting.
**Evidence:**
- Drafts: `.context/working/drafts/T-2972-reply-{A,B,C}.txt` (132/77/85 lines), gitignored working files
- P-011: 7/7 verification lines pass, incl. the ground-truth re-check `grep -rl TERMLINK_PARENT_SESSION crates/` → 0 files
- Register: G-091 (T-559 checkout/project conflation — live-fired in this session, root at `.agentic-framework/bin/fw:75-88` + `:220-224`), G-090 (envelope project attribution) in `.context/project/concerns.yaml`
- Precedent for the post shape: our 2026-09-01 reply on `dm:8e6fd77ec6f74b37:d1993c2c3ec44c94` offset 1 (`from_project` + `_thread` metadata)

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

### 2026-09-17T21:40:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2972-answer-the-three-055-agentic-fleet-cockp.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-a8a28a76
- **Timestamp:** 2026-09-18T15:38:42Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#2 (Agent)** — The topology answer states the 5-level model does not exist, and names the three real levels plus the two tag conventions, citing crates/termlink-session/src/registration.rs
  - **AC-verify-mismatch** (narrow, heuristic) — `path=crates/termlink-session/src/registration.rs in: The topology answer states the 5-level model does not exist, and names the three real levels plus the two tag conventions, citing crates/termlink-sess`

### 2026-09-18T15:38:40Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
