---
id: T-2484
name: "charter canonical-sentence sync canary — guard the one blessed purpose sentence across CHARTER README ARCHITECTURE"
description: >
  charter canonical-sentence sync canary — guard the one blessed purpose sentence across CHARTER README ARCHITECTURE

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
created: 2026-08-02T00:10:41Z
last_update: 2026-08-02T00:15:42Z
date_finished: 2026-08-02T00:15:42Z
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

# T-2484: charter canonical-sentence sync canary — guard the one blessed purpose sentence across CHARTER README ARCHITECTURE

## Context

Fresh critical-review gap (T-2468 mandate, 6th re-issue). P1 (T-2470) shipped
`docs/CHARTER.md` as "the single owned statement of what TermLink is". Its header
CLAIMS "README and `docs/ARCHITECTURE.md` both quote the canonical sentence below;
edit it here and the docs follow by reference." **That claim is mechanically false**
— markdown has no transclusion, so the canonical purpose sentence exists as THREE
independent copies: `docs/CHARTER.md:9-12` (bold, line-wrapped), `README.md:3`
(plain), `docs/ARCHITECTURE.md:3` (blockquote `> …`). Nothing verifies they stay
identical (`grep -r charter scripts/ .context/cron/` → only the P8 tool-drift
canary, which is unrelated). A human editing the sentence in CHARTER.md exactly as
the charter instructs would silently strand two stale copies. **The charter — the
anchor of the entire T-2468 arc — is not load-bearing for its own consistency
claim.** G-019: the framework is blind to charter-sentence fork.

This is distinct from P8 (T-2483): P8 guards the TOOL SURFACE against off-charter
breadth accretion; this guards the CHARTER SENTENCE against its three copies
diverging. Complementary — together they make the charter load-bearing in both
directions. Mirrors the existing doc-set-drift idiom (`check-preflight-doc-set-drift.sh`,
T-2188). Also fixes the false "follow by reference" wording in CHARTER.md's header.

## Acceptance Criteria

### Agent
- [x] `scripts/check-charter-sentence-drift.sh` exists, is executable, and `--help`
      documents purpose (guards the canonical purpose sentence across CHARTER/README/
      ARCHITECTURE), `--quiet`, `--no-heartbeat`, and exit codes (0 in-sync / 1 drift
      / 2 tooling).
- [x] Extracts the canonical sentence from all three files and NORMALIZES away the
      per-file decorations (CHARTER's `**bold**` + line-wrapping, ARCHITECTURE's
      blockquote `> ` prefix, whitespace/newline collapse) before comparing, then
      demands unanimity. Exits 0 when all three agree.
- [x] FIRES (exit 1) when any one file's sentence diverges — proven for BOTH a
      README drift and an ARCHITECTURE drift via fixtures (no reliance on the live
      repo state), and prints a diagnostic table naming the drifted surface(s).
- [x] Returns tooling error (exit 2) when a source file is missing or the sentence
      cannot be extracted from it (fail-closed, never a false "in-sync").
- [x] Touches `.context/working/.charter-sentence-drift-canary.heartbeat` and appends
      framed entries (`=== <ts> === … ---`) to
      `.context/working/.charter-sentence-drift-canary.log` on firing, matching the
      canary convention (empty log = healthy, `/canaries` auto-discovers it).
- [x] `scripts/test-charter-sentence-drift.sh` covers in-sync / README-drift /
      ARCHITECTURE-drift / missing-file via fixtures (a temp repo the script points at
      via an env override); prints `PASS`/`FAIL`; all pass.
- [x] Runs green against the REAL repo today: exit 0 (three copies currently agree)
      — recorded in the task Updates.
- [x] `docs/CHARTER.md` header wording fixed: the false "the docs follow by reference"
      claim is replaced with an honest "kept in sync (a canary verifies README +
      ARCHITECTURE match — T-2484)."
- [x] Cron file `.context/cron/charter-sentence-drift-canary.crontab` created (main
      check + meta-canary aliveness line) AND installed to `/etc/cron.d/`, and a
      CLAUDE.md §"Charter canonical-sentence drift canary" section added as the 13th.

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

## Updates

### 2026-08-02 — build + verification COMPLETE
- **Gap (6th mandate re-issue):** the P1 "single owned statement" (docs/CHARTER.md)
  claimed README + ARCHITECTURE "follow by reference", but markdown has no
  transclusion — three independent copies with nothing verifying they agree. Found via
  a critical review (self + a general-purpose review agent independently ranked this #1;
  the agent also confirmed the alternative "denylist→allowlist P8 rewrite" hypothesis
  as real-but-not-worth-it, FP-prone across 214 tools).
- **Built:** `scripts/check-charter-sentence-drift.sh` (13th canary) — extracts +
  normalizes the sentence from all three files (strips bold / blockquote / wrapping),
  demands unanimity. Anchor-based extraction truncated at first `machines` via pure
  bash param-expansion (no PCRE dep) survives README's second `…machines.` sentence
  (the greedy-match trap). Fail-closed on missing/unextractable file (exit 2).
- **Tests:** `scripts/test-charter-sentence-drift.sh` → `test-charter-sentence-drift: PASS`
  (7/7: in-sync w/ greedy-trap present / README-drift / ARCH-drift / missing-file /
  missing-anchor / drift-table-names-surface / --help). Live real-repo run → exit 0
  (three copies currently agree).
- **Doc-honesty fix:** replaced CHARTER.md's false "the docs follow by reference"
  header claim with an honest "kept in sync by a canary" description.
- **Cron:** `.context/cron/charter-sentence-drift-canary.crontab` (09:07 main + 09:37
  meta-aliveness) installed to `/etc/cron.d/termlink-charter-sentence-drift-canary`
  (verified byte-identical, 2 USER-field lines). CLAUDE.md §"Charter canonical-sentence
  drift canary" added as the 13th.
- **Sibling framing:** P8 (T-2483) guards the tool surface against off-charter breadth;
  this guards the charter sentence against forking — charter now load-bearing both ways.

## Verification

test -x scripts/check-charter-sentence-drift.sh
out=$(bash scripts/check-charter-sentence-drift.sh --help 2>&1); echo "$out" | grep -q "canonical"
out=$(bash scripts/test-charter-sentence-drift.sh 2>&1); echo "$out" | grep -q "^test-charter-sentence-drift: PASS"
bash scripts/check-charter-sentence-drift.sh --quiet --no-heartbeat; test $? -eq 0
test -f .context/cron/charter-sentence-drift-canary.crontab
test -f /etc/cron.d/termlink-charter-sentence-drift-canary
grep -q "kept in sync" docs/CHARTER.md

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

### 2026-08-02T00:10:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2484-charter-canonical-sentence-sync-canary--.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-f566e2b7
- **Timestamp:** 2026-08-02T00:15:43Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T00:15:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
