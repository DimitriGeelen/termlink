---
id: T-2752
name: "README Installation section contradicts Quick Start — omits install.sh, the recommended path"
description: >
  README Installation section contradicts Quick Start — omits install.sh, the recommended path

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
created: 2026-08-15T21:34:07Z
last_update: 2026-08-15T21:39:33Z
date_finished: 2026-08-15T21:39:33Z
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

# T-2752: README Installation section contradicts Quick Start — omits install.sh, the recommended path

## Context

Surfaced by T-2751 while investigating herdr rank 20. README documents installation twice
and the two accounts disagree:

- **Quick Start** (README.md:88-99) lists `install.sh` **first** — "One-liner install (any
  Linux or macOS, no toolchain required)" — then Homebrew as "macOS preferred", then cargo.
- **`## Installation`** (README.md:285-314) lists only two paths: Homebrew, titled
  "recommended for macOS/**Linux**", and from-source. **`install.sh` is absent entirely.**

So a reader who skips to the section actually named for the task never learns the
recommended path exists, and is instead pointed at Homebrew on Linux — where the tap is a
heavier dependency than a checksum-verified 159-line script, and which Quick Start itself
calls "macOS preferred". The two sections also disagree about which path is recommended.

Third defect in the same neighbourhood: README.md:314 states "The binary installs to
`~/.cargo/bin/termlink`" as a flat statement after both subsections, but that is true only
of the from-source path. `install.sh` installs to `/usr/local/bin` (or `$PREFIX/bin`) and
Homebrew to the brew prefix.

Fourth: `.github/workflows/install-check.yml:4-5` justifies itself with "`cargo install
--git URL` (the documented installation path in CLAUDE.md)". That was accurate when written
and is now stale — it is the *third* documented option. The workflow is still worth running
(it catches resolver drift, T-1056/T-1060), but its stated rationale misdescribes the
project's install story, which is how the untested-primary-path situation stayed invisible.

Documentation-only. No behaviour change, no new install path — `install.sh` already exists
(T-1134) and already works; this makes the docs agree with reality.

## Acceptance Criteria

### Agent
- [x] `## Installation` presents `install.sh` as a documented option, in the same relative
      order as Quick Start (one-liner first), so the two sections no longer contradict
- [x] The Homebrew subsection's scope claim matches Quick Start's ("macOS") rather than
      claiming Linux is equally recommended
- [x] The `~/.cargo/bin/termlink` statement is scoped to the from-source path rather than
      presented as where "the binary" always lands; the other two paths' destinations are
      stated
- [x] The user-level / no-sudo option is documented (`--prefix=$HOME/.local`), since that is
      the one thing install.sh supports but does not default to
- [x] `install-check.yml`'s header no longer calls `cargo install --git` "the documented
      installation path"; it states which path it covers and names the one it does not
- [x] No behaviour change: `install.sh`, `release.yml` and the formula are untouched, proven
      by `git diff --stat` showing only README.md, the workflow and task/context files
- [x] `bash scripts/check-release-artifact-drift.sh` still clean (the T-2751 guard is not
      disturbed by the doc edits)
- [x] Guard layer still PASS across all members

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

# The Installation section now names install.sh (the contradiction is gone).
out=$(sed -n '/^## Installation/,/^## Security Model/p' README.md); echo "$out" | grep -q "install.sh"

# Homebrew subsection no longer claims Linux is equally recommended.
# (`grep -qv` would be the wrong tool here — it succeeds whenever ANY line fails to
#  match, so it can never fail. Assert the absence of a match instead.)
test -z "$(grep -n 'recommended for macOS/Linux' README.md || true)"

# The no-sudo user-level option is documented.
out=$(cat README.md); echo "$out" | grep -q "prefix=\$HOME/.local"

# install-check.yml no longer calls cargo install the documented install path.
test -z "$(grep -n 'the documented installation path' .github/workflows/install-check.yml || true)"

# The three artifact-name sources are untouched by this doc-only task.
git diff --quiet HEAD -- install.sh .github/workflows/release.yml homebrew/Formula/termlink.rb

# The T-2751 guard is undisturbed.
bash scripts/check-release-artifact-drift.sh

# Guard layer still green.
out=$(bash scripts/run-guard-layer.sh 2>&1); echo "$out" | grep -q "guard layer: PASS"

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

### 2026-08-15 — reconcile toward Quick Start, not toward the Installation section

- **Chose:** Make `## Installation` match Quick Start's ordering (one-liner, Homebrew,
  source) rather than deleting the one-liner from Quick Start to match Installation.
- **Why:** The one-liner is the better default on the merits — it needs no toolchain and no
  package manager, verifies a SHA-256 against the release's `checksums.txt` before
  installing, and picks the musl static build automatically when glibc is absent. Quick
  Start was right and `## Installation` was stale, not the other way round.
- **Rejected:** *Collapse to one section and link* — Quick Start's compact block earns its
  place (it is a copy-paste strip for someone who has already decided); the detailed section
  earns its place too. The defect was that they disagreed, not that both exist.

### 2026-08-15 — kept the Homebrew-on-Linux option, dropped only the "recommended" claim

- **Chose:** Retitle to "Homebrew (macOS)" and note it also works on Linux with the
  one-liner preferred there — rather than removing Linux mention entirely.
- **Why:** Homebrew on Linux genuinely works; the formula ships a linux-aarch64 and a
  linux-x86_64-static bottle. The false part was calling it *recommended for Linux* while
  Quick Start called it "macOS preferred". Removing the option would have been a second
  inaccuracy in the other direction.

### 2026-08-15 — the CI header states what it does NOT cover

- **Chose:** Rewrite `install-check.yml`'s header to name its scope (from-source only), say
  explicitly that `install.sh` is not exercised there and why (no published release exists
  for an unreleased commit), and point at what does guard it (T-2751's name-drift check).
- **Why:** The old header's claim that `cargo install --git` was "the documented
  installation path" is a large part of why nobody noticed the primary path had no coverage
  — a guard that misdescribes its own scope reads as broader than it is (T-2680). Naming the
  uncovered path in the covering job's header is the cheapest place for the next reader to
  find it.
- **Rejected:** *Add CI that actually runs install.sh* — it downloads a published release
  binary, which does not exist for an unreleased commit, so it would test the *previous*
  release rather than the commit under test. That is a real gap but a different task with a
  real design question (pin a known release? build-and-serve locally?), not a doc fix.

### 2026-08-15 — caught my own unfalsifiable verification line

- **What happened:** The Verification block originally contained
  `echo "$out" | grep -qv "recommended for macOS/Linux"`. `grep -qv` succeeds whenever *any*
  line fails to match, so on a multi-line file it can never fail — a check that would have
  passed whether or not the fix was made. Replaced with
  `test -z "$(grep -n '…' README.md || true)"`, which asserts genuine absence.
- **Why recorded:** this is PL-219 ("a guard whose assertion cannot fail is not a guard") in
  my own hand, in the same session I applied it to two other checks. Worth the entry
  precisely because knowing the rule did not prevent writing the bug.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-08-15T21:34:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2752-readme-installation-section-contradicts-.md
- **Context:** Initial task creation

### 2026-08-15T21:39:33Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
