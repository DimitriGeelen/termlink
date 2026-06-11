---
id: T-2147
name: "Fix 3 pre-existing CLI test failures (version bitrot + T-1426 deprecation noise)"
description: >
  Fix 3 pre-existing CLI test failures (version bitrot + T-1426 deprecation noise)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/tests/cli_integration.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-10T21:57:37Z
last_update: 2026-06-10T22:02:31Z
date_finished: 2026-06-10T22:02:31Z
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

# T-2147: Fix 3 pre-existing CLI test failures (version bitrot + T-1426 deprecation noise)

## Context

Three CLI integration tests have been red across the last several sessions:

1. `cli_version_text` — version pattern check predates v0.11 (asserts
   `0.9.` or `0.10.` only; live binary is 0.11.1156).
2. `cli_inbox_status_json_no_hub` — T-1426 stderr deprecation print
   noise masks the "Hub is not running" stderr assertion the test
   relies on.
3. `cli_inbox_clear_json_no_hub` — same root cause as #2.

Tests #2 and #3 verify the hub-down error path of two legacy inbox
verbs (retained for T-1166 soft-deprecation). The right fix is to set
`TERMLINK_NO_DEPRECATION_WARN=1` in the test env so the deprecation
warning is suppressed via its existing scripts/CI escape hatch — the
original assertion then sees the real hub-down stderr unchanged.
Test #1 just needs the live version family pattern added.

Direct CI-hygiene fix; no semantic changes to product behavior.

## Acceptance Criteria

### Agent
- [x] `cli_version_text` accepts `0.11.` AND `0.12.` (+ existing 0.9./0.10.) with a TODO pointing at regex if past 0.12
- [x] `cli_inbox_status_json_no_hub` sets `TERMLINK_NO_DEPRECATION_WARN=1` AND switches stderr-check to stdout-check (since T-1916 the --json hub-down path writes its JSON error envelope to stdout via json_error_exit, not stderr)
- [x] `cli_inbox_clear_json_no_hub` same two-layer fix
- [x] All three previously-failing tests pass — `cargo test -p termlink --test cli_integration -- cli_version_text cli_inbox_status_json_no_hub cli_inbox_clear_json_no_hub` → 3 passed
- [x] Full cli_integration suite: 172 pass / 0 fail (was 169 pass / 3 fail)
- [x] No regressions in main binary tests — `cargo test -p termlink --bin termlink` → 940 passed

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

cargo test -p termlink --test cli_integration --quiet cli_version_text 2>&1 | tail -3
cargo test -p termlink --test cli_integration --quiet cli_inbox_status_json_no_hub 2>&1 | tail -3
cargo test -p termlink --test cli_integration --quiet cli_inbox_clear_json_no_hub 2>&1 | tail -3

## RCA

**Symptom:** Three CLI integration tests (`cli_version_text`,
`cli_inbox_status_json_no_hub`, `cli_inbox_clear_json_no_hub`) had been
red across multiple sessions. The session's substrate work shipped on
top of a 169/172 passing baseline, leaving genuine signal masked.

**Root cause:** Two distinct bitrot vectors:
- Version test enumerated `0.9.` / `0.10.` only; v0.11 shipped 2026-05-17
  (T-1691) and v0.11.x has been live since. The test rotted the moment
  the v0.11 tag landed.
- Inbox-json hub-down tests checked stderr, but since T-1916 added
  `--json` honoring on the hub-down path, the JSON error envelope routes
  through `json_error_exit` → stdout. The test was checking the wrong
  stream. T-1426 deprecation print (added later) flooded stderr with
  the only string the test could see, completing the masking.

**Why structurally allowed:**
1. No CI gate fails on new red tests on the main branch — failures
   accumulate as background noise until someone notices.
2. The version test enumerates families instead of asserting structure
   ("looks like semver"). Each major-minor bump rots it.
3. T-1916 (--json error routing) and T-1426 (deprecation prints) each
   landed without touching the inbox tests that depended on the prior
   stderr-only contract. The tests' assertion comment ("should fail
   with clear error") doesn't pin which stream "clear" lives on, so
   the contract drift was invisible at PR review.

**Prevention:**
- Switched test #1 to enumerate up to 0.12 with a TODO comment pointing
  at regex-based matching once past that.
- Switched tests #2/#3 to read from stdout (where `--json` errors
  actually go) AND added `TERMLINK_NO_DEPRECATION_WARN=1` so future
  deprecation rollouts don't repeat the noise-mask.
- Sibling concern: a forward-looking gap is that "land a new
  deprecation print on a verb without updating tests" still has no
  structural detector. Filed as candidate observation; would need a
  separate task to address (out of scope here).

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

### 2026-06-10T21:57:37Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2147-fix-3-pre-existing-cli-test-failures-ver.md
- **Context:** Initial task creation

### 2026-06-10T22:02:31Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
