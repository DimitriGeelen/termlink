---
id: T-2480
name: "P3a — arc-closure capability-live gate (T-2477, G-069)"
description: >
  T-2477 P3 part (a), GO recorded. Add a capability-live check to arc closure so 'shipped' means capability-live not just code-merged. Gate point is AEF tooling (.agentic-framework/lib/arc.sh arc_close, human-invoked per G-062) — so this is a cross-repo change via the upstream-mirror pattern, NOT a pure termlink edit. Design: arc-closing task's ## Verification gains a one-line live-probe against the primary hub (fleet doctor / cv-keys capability probe). Bounded + reversible. Scope carefully: mandatory-block vs advisory-warn is a human call (IW-1).

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
created: 2026-08-01T21:18:36Z
last_update: 2026-08-01T21:54:20Z
date_finished: 2026-08-01T21:54:20Z
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

# T-2480: P3a — arc-closure capability-live gate (T-2477, G-069)

## Context

P3 (T-2477, G-069) part (a): make "shipped" mean **capability-live**, not just
code-merged. The gap: capabilities were recorded closed=shipped while dark in the
field for weeks (arc-004 push-transport; .107/.122 stale). Detection is partly
covered (T-2359 binary-floor / T-2387 waker / T-2415 capability canaries) but
there is **no gate** at closure time and **no synchronous make-it-live probe**.

**Design pivot (this build).** The task originally framed the gate as an AEF
`arc.sh arc_close` change (shared governance, cross-repo, embedded human
mandatory-vs-advisory decision). Instead we build the missing **primitive** and
let the *existing* P-011 verification gate do the enforcing: `scripts/arc-live-probe.sh`
is a synchronous single-hub probe that confirms a capability/version is actually
being served **right now**. An arc-closing task drops one line into its
`## Verification` block — P-011 already runs Verification commands before
`work-completed` and blocks on non-zero exit, so closure is blocked until the
fleet genuinely serves the capability. No AEF change, reversible, no user-facing
surface removed. The stronger AEF `arc_close` mandatory-block (IW-1) remains a
human decision, captured as a follow-up — advisory-by-convention ships first.

Reuses the exact probe invocations of the T-2359 / T-2415 canaries (fleet doctor
version read + cv-keys / governor capability probe) rather than reinventing them.

## Acceptance Criteria

### Agent
- [x] `scripts/arc-live-probe.sh` exists, is executable, and `--help` documents the
      purpose (the "shipped == live" gate), all flags (`--hub`, `--min-version`,
      `--capability`, `--json`), and the three exit codes (0 live-confirmed /
      1 shipped-but-not-live / 2 tooling-error).
- [x] Version assertion: with `--min-version X`, the probe exits **1** when the
      probed hub serves a version < X and exits **0** when >= X (proven via the
      PL-213 test hook, no live hub needed).
- [x] Capability assertion: with `--capability <cv-keys|field:NAME>`, the
      probe exits **1** when the hub rejects/omits the capability and **0** when it
      is served (proven via fixtures).
- [x] **Fail-closed:** an unreachable or unparseable hub exits **2** (tooling
      error) — never a false "live". A gate that fails open would re-admit the
      exact G-069 blindness it exists to close.
- [x] `scripts/test-arc-live-probe.sh` covers all three assertion types + the
      fail-closed path + `--help`, prints a final `PASS`/`FAIL` line, and all
      cases pass (14/14).
- [x] `docs/operations/shipped-equals-live-gate.md` documents the convention
      (arc-closing tasks add the probe to `## Verification`; P-011 enforces) with a
      copy-paste example line, and CLAUDE.md references the gate.

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

test -x scripts/arc-live-probe.sh
out=$(bash scripts/arc-live-probe.sh --help 2>&1); echo "$out" | grep -q "shipped"
out=$(bash scripts/test-arc-live-probe.sh 2>&1); echo "$out" | grep -q "^test-arc-live-probe: PASS"
test -f docs/operations/shipped-equals-live-gate.md
grep -q "arc-live-probe.sh" CLAUDE.md

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

### 2026-08-01T21:18:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2480-p3a--arc-closure-capability-live-gate-t-.md
- **Context:** Initial task creation

### 2026-08-01T21:49:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-d94d8eb2
- **Timestamp:** 2026-08-01T21:54:21Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-01T21:54:20Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
