---
id: T-2836
name: "Repair 29 unreadable episodic files so the episodic store is readable by its consumer"
description: >
  Repair 29 unreadable episodic files so the episodic store is readable by its consumer

status: work-completed
workflow_type: refactor
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-23T20:59:11Z
last_update: 2026-08-23T21:04:56Z
date_finished: 2026-08-23T21:04:56Z
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

# T-2836: Repair 29 unreadable episodic files so the episodic store is readable by its consumer

## Context

`scripts/check-episodic-parse.sh` (T-2805) asserts the real consumer's property:
every episodic must `yaml.safe_load` into a **mapping**, byte-identical to what
`web/shared.py::get_episodic_tags` requires. 29 of 2345 files fail it, so those
29 tasks' episodic memory is silently dropped by Watchtower today — the reader's
`except yaml.YAMLError: continue` says nothing.

Four classes, and the repair differs sharply per class:
- **LEGACY-MULTIDOC (8)** — `---` frontmatter plus a body; content fully intact.
- **LEGACY-MARKDOWN (14)** — an older generator's `summary:` line then markdown;
  content fully intact.
- **CORRUPT-ESCAPE (4)** — the vendored generator's double-quoted scalar emitted
  an illegal escape (`\|`, `\-`, `\*`) from a mined git subject.
- **CORRUPT-OTHER (3)** — other parse damage.

The 22 legacy files want a format migration, not a rescue. The 7 corrupt ones need
their damaged bytes repaired in place.

**Regeneration is not the fix for CORRUPT-ESCAPE.** The generator
(`lib/episodic.sh::mine_git_timeline`) is vendored and still carries the escaping
bug, so regenerating reproduces the identical bytes — reporting success while
changing nothing. Repair the files; the generator is upstream's (filed at
`framework:pickup` offset 20, G-062).

This also unblocks part of the guard layer: `run-guard-layer.sh` is one of four
FAILs, and eight otherwise-finished tasks assert a green layer in their
Verification blocks.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Every episodic under `.context/episodic/` parses via `yaml.safe_load` into a mapping — `check-episodic-parse.sh` exits 0
- [x] No episodic loses content: each repaired file still carries its original summary/body text, verified by comparing pre- and post-repair extracted text
- [x] The vendored generator is NOT edited (G-062) and no file under `.agentic-framework/` is modified
- [x] CORRUPT-ESCAPE files are repaired in place, not regenerated (regeneration reproduces the bug)
- [x] `bash tests/episodic-parse-check-fixtures.sh` still passes — the checker itself was not weakened to make the tree green

### Human

- [ ] [REVIEW] Repaired episodics render correctly in Watchtower, and the two fallback files are readable
  **Steps:**
  1. Open http://192.168.10.107:3003/approvals and pick this task, or go straight to http://192.168.10.107:3003/review/T-2836
  2. Open a repaired legacy task's episodic view — e.g. http://192.168.10.107:3003/task/T-121 — and confirm the summary and tags now appear (before this task they were silently dropped)
  3. Check the two preserve-everything fallbacks, which kept the whole original in a `body` field rather than restructuring it: `.context/episodic/T-1224.yaml` and the other file marked `*` in the repair log
  **Expected:** Previously-blank episodic sections now show content; the two `body`-field files are readable prose even though they are not fully restructured.
  **If not:** Name which task's episodic still renders empty. All 29 originals are git-tracked, so `git checkout HEAD~1 -- .context/episodic/<file>` restores any individual file.
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

bash scripts/check-episodic-parse.sh --quiet
bash tests/episodic-parse-check-fixtures.sh
test -z "$(git status --porcelain .agentic-framework/)"
out=$(bash scripts/check-episodic-parse.sh --json); echo "$out" | grep -q '"unreadable_count": 0' && ! echo "$out" | grep -q '"ok": false'

## Recommendation

**Recommendation:** GO

**Rationale:** 29 of 2345 episodic files were unreadable by the only code that
reads them, so those tasks' memory was being silently discarded by Watchtower —
the reader's `except yaml.YAMLError: continue` never said so. All 29 now parse
into mappings, and the checker's own 31 fixtures still pass, which is the pair
that matters: a green tree beside a green checker means the tree was repaired,
not the check relaxed.

The repair measures itself. Where the class-specific path would have dropped more
than 40 characters it falls back to preserving the entire original in a `body`
field — two files took that route rather than being silently slimmed.

**Evidence:**
- `scripts/check-episodic-parse.sh` exits 0; `--json` reports `unreadable_count: 0` (was 29).
- `bash tests/episodic-parse-check-fixtures.sh` → 31 passed, 0 failed.
- Per-file content delta after repair: 0 files over the 40-char threshold.
- `git status --porcelain .agentic-framework/` empty — the vendored generator was not touched (G-062).
- All 29 originals are in git history, so any individual file is one `git checkout` from restoration.

**Residual risk — and it is real:** the vendored generator still emits the illegal
escape, so this class recurs the next time a mined git subject contains a regex
character. That is upstream's, filed at `framework:pickup` offset 20. This task
repairs the corpus; it does not stop the next one being written.

**What the human AC is actually for:** the two fallback files kept their content
but not their structure, so they render as prose rather than fields. That is a
judgement about acceptable output, which is why it is not ticked here.

## Evolution

### 2026-08-23 — the content check had to be fixed before the repair could be trusted

- **What changed:** The first pass flagged **10 of 29** files as possibly losing
  content. Nine were an artefact of the check, not the repair: the comparison
  counted only mapping VALUES, so every legacy-multidoc file appeared to shed the
  bytes its KEYS occupied — T-121 read as −132 characters and, once keys were
  counted, as **−1**. A content check that cannot tell restructuring from deletion
  would have either blocked a correct repair or, worse, taught me to wave the
  flag through.

  The tenth was real. T-1224 (CORRUPT-OTHER) genuinely lost **511** characters:
  `repair_escape` recovered only 4 keys and silently dropped the rest. So the
  repair now measures itself per-file and, when the class-specific path would
  drop more than 40 characters, falls back to preserving the entire original as a
  `body` field. Two files took that path. A readable file that kept everything
  beats a tidy one that quietly lost half a kilobyte — the point of the exercise
  is that the reader can see the content, not that the YAML is pretty.

  Final: 29 repaired, 0 failed, 0 flagged. `check-episodic-parse` exits 0 and the
  31 fixtures still pass, which is the load-bearing pair — a green tree next to a
  green checker means the tree was fixed, not the checker.

- **Plan impact:** This clears ONE of the four `run-guard-layer` FAILs. It does
  **not** unblock the eight tasks that assert a green layer: two of the remaining
  three (`check-task-id-collisions`, `check-framework-tracking-drift`) cannot be
  drained from this worktree at all — collisions live on branches this session
  cannot write, and the dangling `tools/corpus_*.py` refs point at files that were
  never committed anywhere, so there is nothing here to restore. Those are
  operator actions and were surfaced as such rather than worked around.

- **Triggered:** No new task. The vendored generator
  (`lib/episodic.sh::mine_git_timeline`) still emits the illegal escape, so this
  class will recur on the next mined git subject containing a regex character;
  that is already filed upstream at `framework:pickup` offset 20 (G-062).

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

### 2026-08-23T20:59:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/t2687-pickup-failopen/.tasks/active/T-2836-repair-29-unreadable-episodic-files-so-t.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-74ccd9f7
- **Timestamp:** 2026-08-23T21:05:25Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Human)** — [REVIEW] Repaired episodics render correctly in Watchtower, and the two fallback files are readable
  - **human-ac-mechanical-signal** (partial, heuristic) — `matched='show c' in Expected: Previously-blank episodic sections now show content; the two `body`-field files are readable prose even though they are not fully restructur`

### 2026-08-23T21:04:56Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
