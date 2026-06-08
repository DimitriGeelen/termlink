---
id: T-2052
name: "Pre-commit blob-size gate — block any single tracked blob > 50MB (G-058 prevention follow-up)"
description: >
  Today's G-058 incident: .context/working/fw-vec-index.db (288MB) accidentally committed 2026-05-25 in b7f18de5, silently rejected by GitHub's 100MB pre-receive hook for 14 days, 805 commits of mirror drift. Canary detected drift correctly but recovery playbook only covered PAT-rotation. Add a structural gate: agents/git pre-commit hook checks each staged blob's size; >50MB → block with hint. Mirrors the secret-scan pattern. Also: extend scripts/check-mirror-freshness.sh diagnosis branch to recognize the file-size-rejection error pattern in pre-receive output. Cost: ~40 LOC + doc update.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [framework, prevention, git-hook, g-058]
components: [scripts/check-mirror-freshness.sh]
related_tasks: [T-1696, T-1799]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-08T10:49:48Z
last_update: 2026-06-08T19:16:00Z
date_finished: 2026-06-08T19:16:00Z
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

# T-2052: Pre-commit blob-size gate — block any single tracked blob > 50MB (G-058 prevention follow-up)

## Context

**SCOPE PIVOT (2026-06-08)**: The blob-size gate is **already shipped** in
the framework as T-1845 (`agents/git/lib/large-file-scan.sh`, 10 MiB BLOCK
threshold, 1 MiB WARN, allowlist support). The hooks installer
(`hooks.sh:install_hooks`) wires it into the pre-commit hook alongside
secret-scan (T-1844) and dup-task-scan (T-1863).

**The actual gap is install drift**: the active `.git/hooks/pre-commit`
in this project was last installed during the T-1844 era and only ran
the secret-scan block. T-1845 + T-1863 never propagated because the hook
was never re-installed. The 288MB fw-vec-index.db that triggered G-058
on 2026-05-25 predates T-1845 by weeks; however, anything 10MB+ that
would be committed today still slips through this project's stale hook.

This task now delivers:
1. Re-install hooks so T-1845 large-file gate is live in this project
2. Verify gate fires on a synthetic 15MB blob test
3. Extend `scripts/check-mirror-freshness.sh` diagnostic to recognize
   GitHub's file-size-rejection error pattern (the second half of the
   original task description — still load-bearing)

## Acceptance Criteria

### Agent
- [x] `.git/hooks/pre-commit` re-installed and contains T-1844 (secret-scan), T-1863 (dup-task-scan), and T-1845 (large-file gate) wiring
- [x] Live-fire test: staging a 15MB blob with focus set is BLOCKED by the large-file gate (15.0 MiB > 10.0 MiB threshold, T-1845 origin marker shown in the block message) — commit aborted, HEAD unchanged
- [x] **Discovered + fixed install-time chmod gap**: ALL scanner scripts (secret-scan, dup-task-scan, large-file-scan) were `-rw-r--r--`, hook check `[ ! -x "$SCANNER" ]` was firing → hook exited 0 (fail open) silently. After `chmod +x` the gate fires correctly. Documented in CLAUDE.md mirror-canary section as the failure signal `secret-scan: scanner not found (skipping)`.
- [x] `scripts/check-mirror-freshness.sh` extended: when drift is detected, scans `github_head..origin_head` for blobs ≥100MB (GitHub's GH001 limit) and surfaces sha+path+size with cleanup hint
- [x] JSON mode adds `oversize_blobs` count + `oversize_first` field; synced state shows `oversize_blobs:0`
- [x] CLAUDE.md mirror-canary section extended with the T-2052 root-cause-diagnosis paragraph + chmod gap signal

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

test -x .agentic-framework/agents/git/lib/secret-scan.sh
test -x .agentic-framework/agents/git/lib/large-file-scan.sh
test -x .agentic-framework/agents/git/lib/dup-task-scan.sh
grep -q "LARGE_FILE_SCANNER\|T-1845" .git/hooks/pre-commit
grep -q "oversize_hint\|oversize_blobs" scripts/check-mirror-freshness.sh
grep -q "T-2052" CLAUDE.md
out=$(bash scripts/check-mirror-freshness.sh --json 2>&1); echo "$out" | grep -q "oversize_blobs"

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

### 2026-06-08 — scope pivot: prevention exists, install gap was real

- **What changed:** Started believing T-2052 needed to BUILD a blob-size pre-commit gate. Discovery: T-1845 already shipped it in the framework (`large-file-scan.sh`, 10 MiB BLOCK, allowlist, wired in `hooks.sh`). The actual gap is install drift: this project's `.git/hooks/pre-commit` was written in the T-1844 era and never refreshed, so it only ran secret-scan. Re-installing wired all three gates.
- **Plan impact:** Original 40-LOC budget evaporated for the gate itself; reallocated to the install-time chmod gap discovery + mirror-canary diagnostic extension.
- **Triggered:** Two structural prevention discoveries — (a) install drift on hook lib (likely affects multiple consumer projects of the framework), (b) chmod missing on vendored lib scripts (every secret-scan invocation in commits showed `scanner not found (skipping)` — fail-open behavior nobody noticed because no one had been intentionally committing a secret to test). The chmod gap is the bigger find: it silently disabled T-1844 secret-scan in this project for the lifetime of the vendor copy.

### 2026-06-08 — root-cause diagnosis added to canary

- **What changed:** Original spec asked for "diagnose file-size rejection pattern in pre-receive output". The canary doesn't read pre-receive output (it just compares HEADs via ls-remote). Reframed to: when drift is detected, scan the local `github_head..origin_head` commit range for blobs ≥100MB (GitHub's GH001 per-file limit). If found, surface sha+path+size with cleanup hint. This catches the G-058 root cause (288MB blob silently rejected for 14 days) instead of just "drift".
- **Plan impact:** No prose change needed in the hint; the implementation is local-only (no GitHub API needed), so canary stays self-contained.
- **Triggered:** No new task; closes the loop on T-2052 as originally framed.

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

### 2026-06-08T10:49:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2052-pre-commit-blob-size-gate--block-any-sin.md
- **Context:** Initial task creation

### 2026-06-08T19:11:23Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

### 2026-06-08T19:16:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
