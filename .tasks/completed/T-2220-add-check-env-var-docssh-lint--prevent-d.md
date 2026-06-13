---
id: T-2220
name: "Add check-env-var-docs.sh lint — prevent doc-vs-source env-var-name drift (G-019 prevention for T-2219)"
description: >
  T-2219 fixed env-var-name drift in operator docs but the prevention was deferred. Add a sibling lint to check-error-code-docs.sh that scans every TERMLINK_* env var cited in docs/CLAUDE.md/.claude against the union of names referenced in crates/+scripts/+systemd-templates/, flagging citations with no implementation surface. Wire into the existing doc-lint.yml CI job. Completes the G-019 prevention loop for the env-var-name surface of the doc-vs-source identifier drift class.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: [T-2219, T-2217, T-2218]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T16:35:14Z
last_update: 2026-06-13T16:39:27Z
date_finished: 2026-06-13T16:39:27Z
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

# T-2220: Add check-env-var-docs.sh lint — prevent doc-vs-source env-var-name drift (G-019 prevention for T-2219)

## Context

T-2219 fixed env-var-name drift in operator docs but deferred prevention. This
task delivers the prevention half (G-019: don't close the gap until prevention
exists), mirroring the T-2217 -> T-2218 error-code precedent (ship lint, wire CI).

A full-union re-audit (docs `TERMLINK_*` citations vs the union of
`crates/`+`scripts/`+`systemd-templates/` references) surfaced ONE residual real
drift that T-2219 did not cover: `.claude/commands/cv-keys.md:138` tells operators
to "set `TERMLINK_CV_KEY=<id>` env var on the producer" — but no binary or script
reads such a var. The real mechanism is `channel post --metadata cv_key=<id>`
(`cli.rs:1855`; `listener-heartbeat.sh:172` posts `--metadata cv_key=$agent_id`).
A lint cannot ship green while that drift exists, so this task fixes it too
(same fix+lint coupling as T-2217).

The `TERMLINK_WATCH_` token is a legitimate glob-prefix doc mention (matches
impl vars like `TERMLINK_WATCH_CHANGE_KIND`) — the lint must NOT flag it.

## Acceptance Criteria

### Agent
- [x] `scripts/check-env-var-docs.sh` exists, is executable, and exits 0 (CLEAN) on the current tree
- [x] `cv-keys.md` no longer cites the non-existent `TERMLINK_CV_KEY` env var (points at `--metadata cv_key=<id>`)
- [x] Lint correctly flags a reintroduced drift (demonstrated live: inject a bogus `TERMLINK_*` token into a doc -> lint exits 1 -> revert) and does NOT false-flag the `TERMLINK_WATCH_` glob-prefix mention
- [x] `.github/workflows/doc-lint.yml` runs `check-env-var-docs.sh` (CI-wired alongside the error-code lint)
- [x] `bash scripts/check-error-code-docs.sh` still exits 0 (no regression to the sibling lint)

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

# lint exists, executable, and clean on current tree
test -x scripts/check-env-var-docs.sh
bash scripts/check-env-var-docs.sh
# the cv-keys.md drift is fixed
! grep -q 'TERMLINK_CV_KEY' .claude/commands/cv-keys.md
# CI wired
grep -q 'check-env-var-docs.sh' .github/workflows/doc-lint.yml
# sibling error-code lint still clean (no regression)
bash scripts/check-error-code-docs.sh

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

## Recommendation

**Recommendation:** Ship `check-env-var-docs.sh` (docs `TERMLINK_*` vs union of crates/scripts/systemd-templates), fix the one drift it surfaces (`cv-keys.md` `TERMLINK_CV_KEY`), and wire it into `doc-lint.yml`.

**Rationale:** Completes the G-019 prevention loop for the env-var-name surface of the doc-vs-source identifier drift class (PL-217). Pure tooling+doc change, no source-behavior/remote/restart risk. Three confirmed env-var drift instances now (T-2219 substrate x3 names + queue knob, plus this cv_key) justify a structural lint over continued whack-a-mole (Level-C escalation, same as error codes).

**Evidence:** Full-union audit found exactly one residual after T-2219 (`TERMLINK_CV_KEY`, `cv-keys.md:138`), no impl reference (real mechanism `--metadata cv_key=`, cli.rs:1855). Lint must tolerate the `TERMLINK_WATCH_` glob-prefix mention (matches `TERMLINK_WATCH_CHANGE_KIND`).

## Updates

### 2026-06-13T16:35:14Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2220-add-check-env-var-docssh-lint--prevent-d.md
- **Context:** Initial task creation

### 2026-06-13T16:39:05Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-13T16:39:27Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
