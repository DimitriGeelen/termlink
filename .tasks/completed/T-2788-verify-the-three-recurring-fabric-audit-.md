---
id: T-2788
name: "Verify the three recurring fabric audit WARNs clear via their own printed mitigations"
description: >
  Every audit run prints the same three fabric WARNs (207/358 cards no edges; 10 unregistered files; 1 card outside watch patterns), each with a printed mitigation. Verify each mitigation actually clears its WARN; where it does not, identify and state why. Same class as T-2784 (a printed remedy that does not remedy).

status: work-completed
workflow_type: test
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-18T11:47:41Z
last_update: 2026-08-18T18:53:28Z
date_finished: 2026-08-18T18:53:28Z
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

# T-2788: Verify the three recurring fabric audit WARNs clear via their own printed mitigations

## Context

Every `fw audit` run prints the same three fabric WARNs, each carrying a printed
mitigation:

| WARN | Printed mitigation |
|---|---|
| `207/358 cards have no edges` | `fw fabric enrich` |
| `10 source file(s) have no fabric card` | `fw fabric scan` |
| `1 card(s) point at files no watch pattern covers` | widen `.fabric/watch-patterns.yaml`, or remove the card |

PL-161 already records the class: *"Audit-recurring WARN where the suggested
mitigation is structurally a no-op."* PL-086 records that fabric drift recurs
(4× in 14d). So the question is not "run the mitigation" — it is whether the
mitigation the audit prints actually clears the WARN it is printed under. A
remedy that does not remedy is the T-2784 shape, and a WARN channel that never
clears is the PL-340 shape (a digest its reader stops believing).

Parked at `horizon: later` on 2026-08-18 when the operator redirected to the
0503-Codex cross-UID transport RCA.

## Acceptance Criteria

### Agent
- [x] Pre-state recorded verbatim from a single `fw audit` run (S-2026-0818-2009,
      2026-08-18T20:10:20+02:00):
      - `[WARN] Fabric: 207/358 cards have no edges` → `Run: fw fabric enrich`
      - `[WARN] Fabric drift: 12 source file(s) have no fabric card` → `Run: fw fabric scan`
      - `[WARN] Fabric: 1 card(s) point at files no watch pattern covers` → widen
        `.fabric/watch-patterns.yaml`, or remove the card
- [x] Each printed mitigation run, post-state measured numerically (see findings below).
      Measured directly against `.fabric/components/*.yaml` rather than by re-reading the
      audit line, because the audit's own counter is one of the things under test.
- [x] For the WARN that did NOT clear, the reason is cited from the mitigation's own
      code: `.agentic-framework/agents/audit/audit.sh:1541-1544`.
- [x] Outcome (b): one mitigation clears, one is a permanent no-op and is filed upstream,
      one was never a defect. Detail below; nothing left silently unchanged.

## Findings

**One of the three printed mitigations actually clears its WARN.**

**WARN 2 — unregistered files → `fw fabric scan`: WORKS.** 14 unregistered → 0, verified
by `fw fabric drift` before and after. (Pre-state said 12; it was 14 by the time this ran
because T-2792 and T-2793 had each added two files.) This mitigation is honest and the
WARN is a genuine action item.

**WARN 1 — cards with no edges → `fw fabric enrich`: PERMANENT NO-OP.** Running it
processed all 384 cards and added **0 edges**. Not "few" — zero. The enricher is
deterministic and already exhausted; it has extracted every edge its heuristics can find,
so every future run is also zero, and the audit will keep printing the same instruction
under the same WARN forever.

The structure of the gap, measured across all 384 cards:

| type | with edges | edgeless | coverage |
|---|---|---|---|
| `.rs` | 114 | 7 | **94%** |
| `.sh` | 33 | 213 | **13%** |
| `.py` | 4 | 8 | 33% |
| `.service` | 0 | 3 | 0% |

**213 of the 231 edgeless cards are shell scripts.** The Rust enricher works;
the shell one barely does. And it is not that shell is unsupported — 33 shell cards *do*
carry edges — which is exactly why the failure is invisible: partial support looks like
incomplete work rather than a ceiling.

So the WARN is mislabelled at the root. It reads as "someone forgot to run enrich" when
what it measures is a standing structural property of a shell-heavy repository. An
operator who follows the instruction sees `Total edges added: 0`, learns nothing about
why, and watches the identical WARN reappear on the next run. That is worse than an
unactionable warning: it is an *unfalsifiable* one, because nothing in the output
distinguishes "you have not run it" from "running it cannot help".

Cause, cited: `.agentic-framework/agents/audit/audit.sh:1541-1544` emits the WARN and the
`Run: fw fabric enrich` remedy on `unenriched > 10`, with no check that enrich would
change anything. `fw fabric enrich --help` offers `--dry-run`, so the audit could answer
that question before recommending the command, and does not.

This is PL-161's class ("audit-recurring WARN whose suggested mitigation is structurally
a no-op") with a specific, citable instance, and T-2784's shape (a printed remedy that
does not remedy).

**WARN 3 — card outside watch patterns: NOT A DEFECT.** Already a documented open
decision. `.fabric/watch-patterns.yaml:35-48` names it explicitly: the card is
`.claude/commands/capture.md`, 34 slash-command definitions exist, exactly one carries a
card, and the two honest resolutions are to register all 34 or to drop the singleton.
T-2722 deliberately left it open rather than hand-shaping a glob around one file — which
the file itself calls out as the PL-341 fabrication trap. The mitigation text is accurate
and the WARN is doing its job by continuing to surface an unresolved scope decision.

**Disposition.** The no-op belongs to the framework, not to this repo
(`.agentic-framework/` is a separate vendored repository), so per G-062 it is filed to
`framework:pickup` rather than patched here. Two candidate fixes were offered upstream:
gate the remedy on `enrich --dry-run` actually finding something, or re-word the WARN to
report shell-vs-Rust coverage as a structural property.

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
       Conversion: this AC should be moved to ### Agent and this line added to
       ## Verification (herestring, not a pipeline — see the L-387 hint below):
         out=$(bin/fw reviewer T-XXX 2>&1 || true); grep -q "Overall:.*PASS" <<< "$out"
-->

## Verification

# WARN 2's mitigation worked and must STAY worked: zero unregistered files.
out=$(.agentic-framework/bin/fw fabric drift 2>&1 || true); grep -q "unregistered: 0" <<< "$out"
# WARN 1's mitigation is a no-op — pin it, so if the framework ever fixes the enricher
# this line fails and the finding above gets re-read instead of quietly going stale.
out=$(.agentic-framework/bin/fw fabric enrich 2>&1 || true); grep -q "Total edges added: 0" <<< "$out"
# The cited cause still says what the finding claims it says.
out=$(sed -n '1541,1544p' .agentic-framework/agents/audit/audit.sh 2>&1 || true); grep -q "fw fabric enrich" <<< "$out"
# WARN 3 is a documented open decision, not a defect.
out=$(cat .fabric/watch-patterns.yaml 2>&1 || true); grep -q "STILL NOT COVERED" <<< "$out"

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
#
# Pipefail/SIGPIPE hint (L-387, corrected by T-2775): P-011 runs each command
# under `set -eo pipefail`. NEVER write `cmd | grep -q PATTERN`: it exits 141
# (SIGPIPE) when grep matches and closes stdin while the upstream is still
# writing — verification then "fails" BECAUSE the check succeeded, and the
# earlier the match, the more reliably it fails.
#
# USE ONE OF THESE — both measured rc=0 at 3M lines:
#     out=$(cmd 2>&1 || true); grep -q "PATTERN" <<< "$out"   # herestring (preferred)
#     test -n "$(cmd | grep -m1 PATTERN)"                     # pipeline inside $( )
#
# The herestring is preferred: a herestring spawns no producer process, so there
# is nothing to SIGPIPE and it cannot regress as output grows. In the second form
# the pipeline sits inside a command substitution, whose status is discarded — the
# OUTER `test` decides.
#
# DO NOT capture-then-pipe. This template previously prescribed
#     out=$(cmd 2>&1); echo "$out" | grep -q "PATTERN"     # UNSAFE above ~64KB
# and it is size-dependent, not safe: `echo`/`printf` is a producer like any
# other, so once $out exceeds the pipe buffer it is still writing when `grep -q`
# exits and pipefail propagates 141. The capture bounds the DATA but does not
# remove the PRODUCER. Anything wrapping `cargo test`, `fleet doctor --json`, or a
# full log is already in that size range. (T-2775 measured this; 999-AEF L-613 and
# 050-email-archive PL-161 published the capture-then-pipe form before the
# correction — both have since adopted the herestring.)
#
# Corollary (T-2090): intermediate stages are just as fatal — `... | tail -3 |
# grep -q PAT` re-introduces the same risk. With a herestring the question does
# not arise; grep scans the whole captured string anyway.
#
# Origin: L-387, captured 4× (T-1716, T-1838, T-1862, T-1863) before the hint;
# T-2775 then measured 1490 exposed lines across 802 tasks despite the hint, which
# is why `scripts/check-verification-pipefail.sh` now enforces it structurally.
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

### 2026-08-18T11:47:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2788-verify-the-three-recurring-fabric-audit-.md
- **Context:** Initial task creation

### 2026-08-18T18:53:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
