---
id: T-2666
name: "Static check: no silent non-zero process::exit on user paths (Directive #2)"
description: >
  grep/AST-lite scanner flagging std::process::exit(non-zero) with no preceding user-facing
  output; regression-guards T-2663/T-2657

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/remote.rs, 
      crates/termlink-cli/src/commands/session.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T21:19:34Z
last_update: '2026-08-18T18:59:15Z'
date_finished: 2026-08-12T21:35:07Z
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
  - ts: '2026-08-18T18:56:55Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 3
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=3 
      (body:component-silent-failure); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:15Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 3
      tier: 2
      effort: 8
    rationale: blast_radius=3 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2666: Static check: no silent non-zero process::exit on user paths (Directive #2)

## Context

Round-15 static-check prevention candidate (sibling of the T-2527 alloc-sink and
T-2531 drain-sink source checks). Directive #2 ("no silent failures"): a
`std::process::exit(<non-zero>)` reached on a user-facing CLI path with NO preceding
user-facing output (stderr `eprintln!`, stdout `println!`/`print!`, a `*json_error_exit*`
helper, or a `bail!`) leaves the user staring at a non-zero exit with zero explanation —
indistinguishable from a crash. T-2663 was exactly this (`discover --first` no-match did
a bare `std::process::exit(1)` in text mode); it was found only by a human reading that
line. The convention "loud-refuse — name what happened before exiting" exists by
discipline but nothing enforces it. This check makes it load-bearing.

The scanner is grep/AST-lite bash over `crates/termlink-cli/src` (the user-facing crate).
It targets the PRECISE recurring shape (see Evolution — a fuzzy preceding-window was tried
first and rejected): a `std::process::exit(<non-zero int literal>)` whose IMMEDIATELY-
preceding non-blank line is a lone `}` (a closed block, not an output/`flush()` line), where
that block carries a `json_error_exit` within a short window above and NO output macro sits
between the brace and the exit. That is exactly the json-gated-output / bare-text-exit
divergence (JSON branch prints via `json_error_exit`; text branch exits silent). Exit-code-
FORWARDING sites (`exit(code)`, `exit(exec_result.exit_code)`, `exit(exit_code as i32)`) are
out of scope by construction — the regex only matches a non-zero integer literal. Confirmed-
loud sites are acknowledged in an allowlist (drift-stable `file::fn::silent-exit` signature),
so the check trends toward empty. Output is a REVIEW list, not a hard gate. NOT a runtime
cron canary — a source-level static check run ad-hoc / in the meta-check tier.

## Acceptance Criteria

### Agent
- [x] `scripts/check-silent-exit.sh` scans `crates/termlink-cli/src` (default; `--root` repeatable), flags each non-zero-int-literal `std::process::exit` whose preceding statement is a lone `}` closing a `json_error_exit`-bearing block with no output macro between (the T-2663 class), honours an allowlist (`--allowlist`, default `.context/working/.silent-exit-allowlist`), and exits 0=clean / 1=unacknowledged / 2=tooling
- [x] Supports `--json` (envelope `{ok, firing:[{file,line,fn}], checked, candidates}`), `--quiet`, `--no-heartbeat`, `-h/--help`; emits a `.heartbeat` companion like the sibling checks when not suppressed
- [x] The current tree scans CLEAN (0 unacknowledged, 39 non-zero-literal exits scanned) after the T-2667 sibling migration; the check surfaced 2 real defects (session.rs:617, remote.rs:1301) on its first run
- [x] Load-bearing proof: fixture harness (`tests/silent-exit-check-fixtures.sh`, no live binary) 7/7 — silent fires, loud/forwarding/flush/allowlisted-clear all correct; reverting T-2663's `eprintln!` fires the real check on metadata.rs:292 (verified, restored)
- [x] `cargo build -p termlink` unaffected (script-only change); `bash scripts/check-silent-exit.sh` runs clean

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
bash scripts/check-silent-exit.sh
bash tests/silent-exit-check-fixtures.sh

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

**Symptom:** A user runs a CLI subcommand, it exits non-zero, and prints nothing —
indistinguishable from a crash (T-2663: `discover --first` no-match, bare `exit(1)`).

**Root cause:** `std::process::exit(<non-zero>)` on a user path with no preceding
user-facing output. Individually invisible; the "name what happened before exiting"
convention is followed only by discipline.

**Why structurally allowed:** the compiler and clippy are silent on a bare non-zero
exit — nothing distinguishes a loud exit from a silent one. The loud-refuse-with-hint
convention (PL-306) is unenforced at the source layer.

**Prevention (this task):** a source-level static check (sibling of T-2527/T-2531)
that flags any non-zero `process::exit` with no user-facing-output marker in scope,
allowlisting confirmed-loud sites. Reverting T-2663's `eprintln!` re-fires it — the
load-bearing property.

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

### 2026-08-12 — fuzzy preceding-window rejected for a precise rule
- **What changed:** the initial design flagged any non-zero exit with no output marker in a
  bounded preceding-line window. Calibration on the live tree showed it BOTH false-positived
  (12 loud sites whose output sits just beyond the window — `flush();exit`, multi-line
  `eprintln!`, report-emitter helpers) AND false-negatived the 2 REAL defects (a success-branch
  `println!` inside the window masked the sibling failure-branch bare exit).
- **Plan impact:** pivoted to a precise rule keyed on the actual divergence signature —
  immediately-preceding lone `}` + `json_error_exit` in-block + no output between brace and
  exit. Fires on exactly the T-2663 class (metadata fixed, session+remote unfixed), zero false
  positives, immune to the success-branch-println confound.
- **Triggered:** T-2667 (the check surfaced 2 un-migrated siblings — fixed there). A window-clamp
  bug (negative `sed` start when an exit is near line 1) was caught by the fixture harness and
  fixed before commit.

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

### 2026-08-12T21:19:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2666-static-check-no-silent-non-zero-processe.md
- **Context:** Initial task creation

### 2026-08-12T21:19:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-26d9b363
- **Timestamp:** 2026-08-12T21:35:16Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-12T21:35:07Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
