---
id: T-2718
name: "Fabric no-edges warning cannot be cleared by its own mitigation"
description: >
  193/344 cards have no edges and fw fabric enrich adds zero; most edgeless cards are leaf scripts whose real dependency is the termlink binary, which the file-to-file edge model cannot express

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
created: 2026-08-14T20:38:32Z
last_update: 2026-08-15T06:14:24Z
date_finished: 2026-08-15T06:14:24Z
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

### Correction — settled 2026-08-15, after three measurements

This section previously held a *first* correction which was also wrong. Both the
original claim and that correction are recorded here, because the pattern they form
is the finding.

**Measurement 1 (original, too narrow).** "0 of 149 source another repo script" was
true only of three variable idioms (`${SCRIPT_DIR}` / `${REPO_ROOT}` /
`${PROJECT_ROOT}`) after `source`/`.` — then reported as though it covered invocation
generally. Conclusion drawn: *no parser change closes this.*

**Measurement 2 (first correction, too broad).** A single `grep -lE` for
`(bash|sh|source|\.) +.*scripts/*.sh` returned 18 files, read as 18 real dependencies.
Printing the matching lines shows most are **console hint-text and heredoc usage
blocks** — `echo "  Reproduce: bash scripts/session-selftest.sh"` is a string, not a
call. Worse, it cited `run-guard-layer.sh` as the proof case. That file resolves its
~20 members at runtime:

```
for f in "$SCRIPTS_DIR"/check-*.sh; do ... m_cmd+=("bash $f $extra")
```

No literal target name appears anywhere in it. The one example offered as proof that
a parser *could* close the gap is the one case a parser provably **cannot** reach.

**Measurement 3 (verified by reading every matching line).** Excluding comments,
heredocs, and echo/printf text: **17 of 149 files carry 28 statically-resolvable
sibling dependencies.** The two gaps that hide them are small and specific:

1. `$HERE` is not in the recognised directory-variable set at all — and it is the
   dominant idiom here (`SEND="$HERE/agent-send.sh"`), accounting for most of the 28.
2. The recognised variables match only after `source`/`.` — not after `bash`/`exec`,
   nor in a bare `VAR="${SCRIPT_DIR}/x.sh"` binding executed later.

`substrate-smoke.sh` is the clean example: four `bash "${SCRIPT_DIR}/…-demo.sh"` calls
plus a `WORKER_LOOP=` binding, card has zero edges. `SCRIPT_DIR` is *already* a known
variable; only the verb differs.

**Settled position.** Closing both gaps takes 193 edgeless cards to ~176 — 56% to
~61%. Real, and partial. The residual is genuinely inexpressible: `run-guard-layer.sh`
by runtime glob, and 71 scripts whose dependency is the compiled `termlink` binary.

**What this cost.** Three measurements, two of them written down and one of them filed
outbound in U-006 before it was checked. Same defect each time — a single regex
generalised past what it measured — in opposite directions. A count of zero deserves
more suspicion than a count of a few, and any count headed somewhere outbound deserves
its lines printed and read first.

**This is the task's own warning turned on itself.** It cautions against measuring
over a flattering subset (PL-341) and then reaches its structural conclusion from
three hand-picked grep patterns. A measurement that finds exactly zero deserves more
suspicion than one that finds a few — zero usually means the probe was wrong, not
that the phenomenon is absent. Recording it here rather than silently amending the
paragraph, because the reasoning error is the more useful artifact.

The conclusion that survives: fixing the parser will NOT clear this warning (18
scripts gaining a few edges each does not move 193/344 materially), and
hand-authoring edges is still the wrong fix. What changes is the *claim* — from
"no parser improvement can close this" to "a parser improvement closes part of it,
and the remainder is structural".

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
- [x] The report shows the mitigation is inert, citing the measured `enrich` output (0 enriched, 0 edges, 0 unresolved — the second zero is what proves saturation rather than blockage)
- [x] It quantifies the gap with a measurement that was verified by reading every matching line, not by trusting a grep count: **17 of 149** `scripts/` files carry **28** statically-resolvable sibling dependencies, while 71 reference the `termlink` binary *(revised twice — see §Correction; the original "0 of 149" and the first correction's "18 invocation edges" were both artefacts of generalising one regex past what it measured)*
- [x] It names the two specific parser gaps that hide those 28: `$HERE` is not a recognised directory variable at all, and the recognised ones (`LIB_DIR`, `SCRIPT_DIR`) match only after `source`/`.`, never after `bash`/`exec` or in a bare `VAR=` binding
- [x] It gives the worked example (`substrate-smoke.sh`: four `bash "${SCRIPT_DIR}/…-demo.sh"` calls plus a `WORKER_LOOP=` binding, card has zero edges — `SCRIPT_DIR` is already known, only the verb differs)
- [x] It states the *bounded* structural conclusion — closing both gaps moves 193 edgeless cards to ~176 (56%→~61%), and the residual is genuinely inexpressible: `run-guard-layer.sh` resolves its ~20 members by runtime glob + header marker with no literal name in the file, and 71 scripts depend on a compiled binary a file→file model cannot name
- [x] It proposes fixing the parser gaps first (unambiguous, no model implications), then the two candidate directions for the residual without choosing between them: allow non-file edge targets (binary/service/artifact), or scope the coverage metric to card types where file edges are meaningful
- [x] It warns explicitly against the two wrong fixes: hand-authoring edges to lower the number, and re-narrowing `watch-patterns.yaml` (the PL-341 trap, hit three times already — T-2680, T-2681, T-2712)
- [x] Filed as an upstream record under `.context/upstream/` *(U-006, corrected 2026-08-15)*
- [x] No file under `.agentic-framework/` is edited by this task

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

### 2026-08-15T06:14:24Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
