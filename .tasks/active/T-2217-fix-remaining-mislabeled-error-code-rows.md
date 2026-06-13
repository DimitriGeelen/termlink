---
id: T-2217
name: "fix remaining mislabeled error-code rows + ship doc-vs-enum pairing audit script (Level-C prevention)"
description: >
  fix remaining mislabeled error-code rows + ship doc-vs-enum pairing audit script (Level-C prevention)

status: started-work
workflow_type: refactor
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-13T16:12:15Z
last_update: 2026-06-13T16:12:15Z
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

# T-2217: fix remaining mislabeled error-code rows + ship doc-vs-enum pairing audit script (Level-C prevention)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] The two remaining mislabeled rows in the `substrate-claim-primitive.md` error taxonomy are corrected: `CLAIM_NOT_FOUND` (-32018) → `CLAIM_EXPIRED` (-32018); `AUTH_FAIL` (-32001) → `AUTH_DENIED` (-32010) / `AUTH_REQUIRED` (-32009). Cross-checked against `control.rs`.
- [x] `scripts/check-error-code-docs.sh` exists, is executable, passes `bash -n`, supports `--json`, and lints every `SYMBOL(-320NN)` pairing in `docs/operations/`, `.claude/commands/`, and `CLAUDE.md` against the authoritative `error_code` enum — exit 1 on any mismatch, 0 clean.
- [x] The lint reports CLEAN (exit 0) across the whole repo after the fixes (58 files scanned, 0 mismatches).

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

bash -n scripts/check-error-code-docs.sh
test -x scripts/check-error-code-docs.sh
bash scripts/check-error-code-docs.sh

## RCA

**Symptom:** Two more rows in the `substrate-claim-primitive.md` error-taxonomy table named the wrong symbol for their code: `CLAIM_NOT_FOUND` was paired with -32018 (really `CLAIM_EXPIRED`), and `AUTH_FAIL` (a doc-only label, no source emits it) was paired with -32001 (really `SESSION_NOT_FOUND`, unrelated to auth). An operator reading the recovery table greps for a symbol/code that the substrate never produces.

**Root cause:** The same doc-vs-source drift class as T-2213 (CLI flag), T-2215/T-2216 (fictional CLAIM_LAPSED/CLAIM_ALREADY_HELD). T-2216's fix was token-targeted (grep for two specific bad strings), so it could not catch rows that were *mislabeled with a different wrong symbol*. Only a full pairing audit against the enum surfaces those.

**Why structurally allowed:** Through four incidents, error-vocabulary prose had no automated cross-check against `crates/termlink-protocol/src/control.rs`. Each fix was Level-A (correct this instance); the class kept recurring because nothing gated it.

**Prevention:** Level-C escalation — `scripts/check-error-code-docs.sh` is now the standing gate. It parses the authoritative `error_code` constants and flags any doc/skill `SYMBOL(-320NN)` pairing that disagrees, exit 1 on mismatch. Run it in CI / `fw audit` / pre-release to stop the next mislabel before it ships. This task's `## Verification` runs it, so the gate is self-proving.

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

## Recommendation

**Recommendation:** Ship. Two-row doc correction PLUS the durable lint that prevents recurrence — the Level-C move after four Level-A fixes of the same class.

**Rationale:** Whack-a-mole on individual mislabeled error codes does not converge (T-2216 fixed two tokens and still left two mislabeled rows in the very same table). A pairing audit against the source enum is the only thing that finds *all* drift and keeps finding it. Cheap to run, high leverage.

**Evidence:** `scripts/check-error-code-docs.sh` exits 0 (CLEAN, 58 files, 0 mismatches) after the fixes and exited 1 listing both rows before; cross-checked against `control.rs` (-32001..-32019) and `claim_client.rs`.

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

### 2026-06-13T16:12:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2217-fix-remaining-mislabeled-error-code-rows.md
- **Context:** Initial task creation
