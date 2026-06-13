---
id: T-2219
name: "Fix substrate/queue notify env-var name drift in operator docs (silent-hook footgun)"
description: >
  Operator docs document TERMLINK_SUBSTRATE_FIELD/OLD/NEW and TERMLINK_OUTBOUND_QUEUE_PATH; source exports TERMLINK_SUBSTRATE_CHANGE_FIELD/OLD/NEW and the queue path knob is TERMLINK_IDENTITY_DIR. Copy-pasted notify-hook recipes silently never fire. Same doc-vs-source identifier drift class as T-2215/2216/2217 (error codes), now for env-var names.

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
created: 2026-06-13T16:29:07Z
last_update: 2026-06-13T16:34:01Z
date_finished: 2026-06-13T16:34:01Z
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

# T-2219: Fix substrate/queue notify env-var name drift in operator docs (silent-hook footgun)

## Context

Audit (this session) of every `TERMLINK_*` env var cited in `CLAUDE.md` /
`docs/operations/*.md` / `.claude/commands/*.md` against the union of names
actually referenced in `crates/` + `scripts/` + `systemd-templates/` surfaced
two real doc-vs-source drifts (the `LH_/SO_/SW_` families are legitimate
systemd-template `EnvironmentFile` knobs — not drift):

1. **`substrate status --watch --notify` event vars** — source exports
   `TERMLINK_SUBSTRATE_CHANGE_FIELD/OLD/NEW`
   (`crates/termlink-cli/src/commands/substrate.rs:873-875`, locked by unit
   tests at 1760-1772; the CLI's own `--help` at `cli.rs:6585` uses the
   correct `_CHANGE_` form). Three operator docs document the `_CHANGE_`-less
   names: `substrate-status.md` (7x), `substrate-tunables.md` (1x),
   `substrate-cron-recipes.md` (2x). `_SCRIPT` and `_TS` are correct.
2. **queue path override** — `.claude/commands/queue-status.md:138` tells the
   operator `echo "$TERMLINK_OUTBOUND_QUEUE_PATH"  # custom path override`, but
   no impl reads that var; the actual relocation knob is `TERMLINK_IDENTITY_DIR`
   (`crates/termlink-session/src/offline_queue.rs:96-104`).

Same class as T-2215/2216/2217 (fictional/mislabeled error codes), now for
env-var names. Severity: silent footgun — a copy-pasted notify gate like
`[ "$TERMLINK_SUBSTRATE_FIELD" = "BACKPRESSURE" ] || exit 0` evaluates an
always-empty var, so the `|| exit 0` always fires and the hook never pages.

**Second-order finding (during fix):** the same `substrate-status.md` notify
recipe also gated on the wrong field *values* — `[ ... = "BACKPRESSURE" ]` and
`case ... BACKPRESSURE) CLAIM)`. Source emits flattened keys (`substrate.rs:755-818`):
`dispatch_idle_count`, `claim_stuck_count`, `backpressure_pressured_hubs`, etc.
— never the uppercase section names. So even after the var-name fix the gate
would never match. Corrected the values + the `claim` branch (was grepping a
bare scalar for `"stuck="`) to match the CLI's own `--help` pattern
(`cli.rs:6585-6586`). Scope stayed coherent: "make the substrate/queue notify
doc recipes actually fire".

## Acceptance Criteria

### Agent
- [x] `substrate-status.md`, `substrate-tunables.md`, `substrate-cron-recipes.md` reference `TERMLINK_SUBSTRATE_CHANGE_FIELD/OLD/NEW` (the `_CHANGE_`-less FIELD/OLD/NEW forms no longer appear)
- [x] `queue-status.md:138` references the real queue-relocation knob `TERMLINK_IDENTITY_DIR` (not the non-existent `TERMLINK_OUTBOUND_QUEUE_PATH`)
- [x] No remaining `TERMLINK_SUBSTRATE_FIELD`/`_OLD`/`_NEW` (non-`_CHANGE_`) or `TERMLINK_OUTBOUND_QUEUE_PATH` token in tracked docs (worktree copies excluded)
- [x] `bash scripts/check-error-code-docs.sh` still exits 0 (no regression to the error-code lint)
- [x] substrate-status.md notify recipe gates on REAL flattened field labels (`backpressure_pressured_hubs`, `claim_stuck_count`), not the uppercase section names (`BACKPRESSURE`/`CLAIM`) — those never match the emitted value, so the gate would always `exit 0`

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

# Substrate notify event vars: only the _CHANGE_ form remains in the 3 docs
! grep -rnE 'TERMLINK_SUBSTRATE_(FIELD|OLD|NEW)\b' docs/operations/substrate-status.md docs/operations/substrate-tunables.md docs/operations/substrate-cron-recipes.md
# Correct _CHANGE_ form is present in substrate-status.md
grep -q 'TERMLINK_SUBSTRATE_CHANGE_FIELD' docs/operations/substrate-status.md
# queue-status no longer cites the non-existent override var
! grep -q 'TERMLINK_OUTBOUND_QUEUE_PATH' .claude/commands/queue-status.md
# error-code lint still clean (no regression)
bash scripts/check-error-code-docs.sh
# notify recipe gates on real flattened field keys, not uppercase section names
grep -q 'backpressure_pressured_hubs' docs/operations/substrate-status.md
! grep -nE 'CHANGE_FIELD.. = .BACKPRESSURE|^  BACKPRESSURE\)|^  CLAIM\)' docs/operations/substrate-status.md

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

**Symptom:** Operators copy-pasting `substrate status --watch --notify` hook
recipes from `substrate-status.md` / `substrate-cron-recipes.md` get a hook
that silently never fires — the gate `[ "$TERMLINK_SUBSTRATE_FIELD" = ... ]`
reads an always-empty variable, so `|| exit 0` always triggers. Likewise
`queue-status.md` points operators at a `TERMLINK_OUTBOUND_QUEUE_PATH`
override that does nothing.

**Root cause:** The docs were authored with env-var names that diverged from
the names the source actually exports (`TERMLINK_SUBSTRATE_CHANGE_FIELD/OLD/NEW`)
and from the real queue-relocation knob (`TERMLINK_IDENTITY_DIR`). The CLI's
own `--help` is correct, so the docs drifted independently of the source.

**Why structurally allowed:** The T-2217 `check-error-code-docs.sh` lint
covers the `SYMBOL(-320NN)` error-code pairing surface only. Env-var-name
drift is the SAME doc-vs-source identifier class but was outside the lint's
scope, so nothing flagged it.

**Prevention:** (a) this fix corrects all 11 occurrences across 4 files;
(b) learning captured (PL) establishing the env-var-name surface as part of
the doc-drift class; (c) a generalized env-var-name lint (scan docs `TERMLINK_*`
against the union of `crates/`+`scripts/`+`systemd-templates/`) is logged as a
Level-C follow-up — deferred from this task because the union-of-surfaces
membership test is false-positive-prone (legitimate `LH_/SO_/SW_` systemd
knobs) and warrants its own scoped task rather than rushed bundling.

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

**Recommendation:** Correct all 11 drifted env-var-name occurrences in the 4
operator docs (3 substrate docs -> `_CHANGE_` form; queue-status -> `TERMLINK_IDENTITY_DIR`).

**Rationale:** Doc-only change to identifiers the source already exports under
different names; zero source/CLI behavior change, zero remote/restart risk. The
broken-notify-hook failure mode is silent (no error, hook just never fires), so
it cannot be caught at runtime by the operator — only doc correctness prevents it.

**Evidence:** Source exports `TERMLINK_SUBSTRATE_CHANGE_FIELD/OLD/NEW`
(`substrate.rs:873-875` + tests 1760-1772 + `cli.rs:6585` help); queue knob is
`TERMLINK_IDENTITY_DIR` (`offline_queue.rs:96-104`). No impl references
`TERMLINK_SUBSTRATE_FIELD` (non-`_CHANGE_`) or `TERMLINK_OUTBOUND_QUEUE_PATH`.

## Updates

### 2026-06-13T16:29:07Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2219-fix-substratequeue-notify-env-var-name-d.md
- **Context:** Initial task creation

### 2026-06-13T16:34:01Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
