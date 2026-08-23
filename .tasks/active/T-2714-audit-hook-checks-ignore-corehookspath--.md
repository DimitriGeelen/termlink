---
id: T-2714
name: "Audit hook checks ignore core.hooksPath — unfixable warning in linked worktrees"
description: >
  audit.sh tests PROJECT_ROOT/.git/hooks/* which cannot exist in a worktree, so CTL-011
  and the commit-msg check warn forever and install-hooks cannot clear them

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
created: 2026-08-14T18:57:33Z
last_update: '2026-08-23T19:13:47Z'
date_finished:
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
  - ts: '2026-08-23T19:13:29Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 4
      D3: 2
      D4: 2
      F-RECALL: 2
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=4 (body:fw-audit-or-doctor); D3=2
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=2 
      (body:lightly-promoted); F-ORCH=0 (no-signal)
    rubric_sha: e4a00f38e801
cost_estimate_proposed:
  - ts: '2026-08-23T19:13:47Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 8
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: e4a00f38e801
---

# T-2714: Audit hook checks ignore core.hooksPath — unfixable warning in linked worktrees

## Context

`fw audit` raised **three** hook warnings in this worktree — CTL-011 *"pre-push
hook missing or not executable"*, *"No commit-msg hook"*, and C-002 *"commit-msg
hook missing research artifact check"*. All three are wrong, and none can be
cleared by the mitigation they recommend.

`audit.sh:2393` tests:

```sh
if [ -f "$PROJECT_ROOT/.git/hooks/commit-msg" ]; then
```

In a **linked worktree** `.git` is a text file containing `gitdir: ...`, not a
directory, so `$PROJECT_ROOT/.git/hooks/<anything>` can never exist. The check is
structurally incapable of passing here. Worse, this repo sets
`core.hooksPath = /opt/termlink/.git/hooks`, which git honours and the audit
ignores entirely.

The hooks are installed and **do fire**:

```
$ git rev-parse --git-path hooks/commit-msg
/opt/termlink/.git/hooks/commit-msg
$ git config core.hooksPath
/opt/termlink/.git/hooks
```

and empirically — the commit for T-2712 ran the post-commit hook, printing
`Task T-2712 updated (...)`.

**Third site, same root cause (found 2026-08-14 pass 2).** `audit.sh:3022`:

```sh
if grep -q "inception-research-warnings" "$PROJECT_ROOT/.git/hooks/commit-msg" 2>/dev/null; then
```

Same concatenation, and the `2>/dev/null` silences the "Not a directory" error so
the failure is indistinguishable from a genuinely missing gate. The installed hook
**does** carry the marker — `/opt/termlink/.git/hooks/commit-msg:166` reads
`# inception-research-warnings: audit marker (C-002 OE check)` — and the C-001/C-002
research-artifact enforcement block is present and functional at lines 164-195,
where it `exit 1`s an inception commit that has no `docs/reports/T-XXX-*` artifact.

This one is the most misleading of the three. CTL-011 and "No commit-msg hook"
claim a hook is absent; C-002 claims a *specific safety gate* is absent from a hook
that is present. An operator reading it concludes the inception research-artifact
enforcement is off, when it is running on every commit. A false negative about a
gate's existence is strictly worse than a false negative about a file's existence.

**The trap this sets.** The mitigation reads *"Install hooks: ./agents/git/git.sh
install-hooks"*. Running it does the right thing — it writes to the common hooks
dir, which is where git actually looks — and the warning persists anyway. An
operator following the instruction concludes either that install-hooks is broken
or that the warning is noise. During this very audit I ran it and reported CTL-011
as remediated before re-running the check; it was not, and could not have been.
A mitigation that cannot fix its own finding is worse than no mitigation, because
it burns the operator's trust in the check.

Sibling of the two other resolution defects found in the same pass: T-2711
(`PROJECT_ROOT` walk matching the vendored framework's `FRAMEWORK.md` before the
consumer's `.framework.yaml`) and T-2713 (hook telemetry counting exit-2 blocks
as failures). All three share a shape — a check whose verdict is decided by an
assumption about layout that no longer holds.

**Cross-repo.** `agents/audit/audit.sh` is vendored; a local edit is erased on
re-vendor. Deliverable is an upstream report, not an edit under
`.agentic-framework/`.

## Acceptance Criteria

### Agent
- [ ] The report cites all three sites hardcoding `$PROJECT_ROOT/.git/hooks/` — `audit.sh:2393` (commit-msg), the CTL-011 site, and `audit.sh:3022` (C-002)
- [ ] It shows that the installed hook DOES carry the `inception-research-warnings` marker and the C-001 enforcement block, so C-002's claim is a false negative about a live gate
- [ ] It shows the worktree evidence: `.git` is a file, `core.hooksPath` is set, and `git rev-parse --git-path hooks/...` resolves elsewhere
- [ ] It includes the empirical proof that hooks fire regardless (post-commit hook output on a real commit), so upstream does not "fix" this by reinstalling hooks
- [ ] It states the false-mitigation problem explicitly — `install-hooks` succeeds and the warning survives
- [ ] A concrete remedy is proposed: resolve via `git rev-parse --git-path hooks/<name>` rather than string-concatenating `.git/hooks`, which handles worktrees and `core.hooksPath` in one call
- [ ] Filed to `framework:pickup` and the post confirmed present
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

**Symptom:** CTL-011 and the commit-msg check warn in a linked worktree even
though the hooks are installed and firing, and running the recommended
`install-hooks` does not clear either warning.

**Root cause:** both checks decide by `test -f "$PROJECT_ROOT/.git/hooks/<name>"`.
That assumes `.git` is a directory — true for a normal clone, false for a linked
worktree, where it is a pointer file. It also ignores `core.hooksPath`, which
overrides the location outright. Git exposes the correct answer through
`git rev-parse --git-path hooks/<name>`; the audit reimplements the lookup by
string concatenation instead of asking git.

**Why structurally allowed:** the check predates worktree use in this project and
was written when the assumption held. The audit already knows it is running in a
worktree — it prints *"Cron drift checks skipped — linked worktree"* two sections
earlier — so the context is available; it simply is not consulted at these two
sites. Nothing tests the audit's own checks against a worktree layout, so a check
that can never pass there looks identical to one that is merely reporting a real
problem.

**Prevention:** replace the concatenation with `git rev-parse --git-path`, which
resolves worktrees and `core.hooksPath` together. The durable guard is a test
that runs the hook checks inside a `git worktree add` fixture and asserts they
PASS when hooks are installed — without it, the next path-derived check will make
the same assumption.

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

### 2026-08-14T18:57:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2714-audit-hook-checks-ignore-corehookspath--.md
- **Context:** Initial task creation

### 2026-08-14T20:21:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
