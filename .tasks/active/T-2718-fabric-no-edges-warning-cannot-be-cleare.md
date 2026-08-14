---
id: T-2718
name: "Fabric no-edges warning cannot be cleared by its own mitigation"
description: >
  193/344 cards have no edges and fw fabric enrich adds zero; most edgeless cards are leaf scripts whose real dependency is the termlink binary, which the file-to-file edge model cannot express

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-14T20:38:32Z
last_update: 2026-08-14T20:38:32Z
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

# T-2718: Fabric no-edges warning cannot be cleared by its own mitigation

## Context

`fw audit` reports:

```
[WARN] Fabric: 193/344 cards have no edges
       Evidence: Graph coverage below target
       Mitigation: Run: fw fabric enrich
```

**Running the mitigation does nothing.** Measured 2026-08-14:

```
Cards enriched:    0
Total edges added: 0
Unresolved edge targets — Actionable: 0, Ignorable: 0
```

Zero added *and* zero unresolved: the enricher is saturated, not merely stuck on
targets it cannot resolve. This is the third instance found in one audit pass of a
check whose printed mitigation cannot fix its own finding — the others are T-2714
(`install-hooks` for a check that reads the wrong path) and T-2715 (`fw audit
schedule install` for a directory that is gitignored). It has recurred **11 times
in 14 days** per the audit's own trend analysis, which is what a warning nobody can
action looks like over time.

**Why the edges are absent, measured rather than assumed.** Of 149 files in
`scripts/`:

- **0** source or invoke another repo script through the standard
  `${SCRIPT_DIR}` / `${REPO_ROOT}` / `${PROJECT_ROOT}` idiom
- **71** reference the `termlink` binary, via `TERMLINK_BIN` or `TERMLINK="..."`

Spot-checked `scripts/check-stuck-claims-freshness.sh`: its only mention of another
script (`scripts/substrate-smoke.sh`) is inside a comment, and its actual
dependency is `TERMLINK="${TERMLINK_BIN:-termlink}"` — a **compiled binary resolved
at runtime from the environment**.

So for a large share of these cards there is no file-to-file edge to find. The
fabric's edge model is file→file; these scripts depend on an artifact the model has
no way to name. **No parser improvement can close this**, because the information
is not missing — the relationship is simply not expressible.

**Why this matters beyond one warning.** The number is now honest — T-2712 widened
`watch-patterns.yaml` so the 188 guard-layer scripts are counted at all, moving the
figure from a flattering 31/150 (18%) to a truthful 193/344 (56%), recorded as
PL-341. The risk now is the opposite one: a future pass reads 56% as a backlog and
either burns a session trying to enrich un-enrichable cards, or worse, hand-authors
plausible-looking edges to bring the number down. That would re-break exactly what
T-2712 fixed. This task exists so the next reader knows the figure is largely
structural before deciding what to do about it.

**Cross-repo.** `agents/fabric/` is vendored; a local edit is erased on re-vendor.
Deliverable is an upstream report, not an edit under `.agentic-framework/`.

## Acceptance Criteria

### Agent
- [ ] The report shows the mitigation is inert, citing the measured `enrich` output (0 enriched, 0 edges, 0 unresolved — the second zero is what proves saturation rather than blockage)
- [ ] It quantifies why: 0 of 149 `scripts/` files source another repo script via the standard idiom, while 71 reference the `termlink` binary
- [ ] It gives the worked example (`check-stuck-claims-freshness.sh`: script mention in a comment, real dependency `TERMLINK_BIN`)
- [ ] It states the structural conclusion — a file→file edge model cannot express a dependency on a compiled binary, so no parser change closes this
- [ ] It proposes the two candidate directions without choosing between them: allow non-file edge targets (binary/service/artifact), or scope the coverage metric to card types where file edges are meaningful
- [ ] It warns explicitly against the two wrong fixes: hand-authoring edges to lower the number, and re-narrowing `watch-patterns.yaml` (the PL-341 trap, hit three times already — T-2680, T-2681, T-2712)
- [ ] Filed as an upstream record under `.context/upstream/`
- [ ] No file under `.agentic-framework/` is edited by this task

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

### 2026-08-14T20:38:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2718-fabric-no-edges-warning-cannot-be-cleare.md
- **Context:** Initial task creation
