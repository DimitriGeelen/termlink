---
id: T-2681
name: "static-check allowlists live under a gitignored path — guard layer not reproducible
  outside the origin checkout"
description: >
  static-check allowlists live under a gitignored path — guard layer not reproducible
  outside the origin checkout

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/canary-status.sh, scripts/check-alloc-sink-clamps.sh, scripts/check-busy-spin.sh, scripts/check-cron-install-drift.sh, scripts/check-drain-sink-caps.sh, scripts/check-silent-exit.sh, scripts/check-verification-misfile.sh, tests/verification-misfile-check-fixtures.sh]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-13T23:27:38Z
last_update: 2026-08-23T20:04:26Z
date_finished: 2026-08-23T20:04:26Z
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
  - ts: '2026-08-23T19:13:28Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
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

# T-2681: static-check allowlists live under a gitignored path — guard layer not reproducible outside the origin checkout

## Context

The four source-level static checks — T-2527 alloc-sink, T-2531 drain-sink, T-2666
silent-exit, T-2672 busy-spin — are the repo's structural guard layer for the
unbounded-allocation / silent-failure / busy-spin classes. Each acknowledges its
confirmed-safe sites in an allowlist "with cited reasons".

Every one of those allowlists lives under **`.context/working/`, which is gitignored**
(`.gitignore:80`). Consequences:

1. **Not reproducible.** In a fresh clone, a CI runner, or a git worktree, the allowlists
   are absent and every acknowledged site fires. Verified in this worktree: alloc-sink
   returns `ok:false` with 5 firing, drain-sink `ok:false` with 6, busy-spin `ok:false`
   with 4. CLAUDE.md documents all three trees as scanning **CLEAN** — true only on the
   one machine that happens to hold the untracked files.
2. **The cited reasons are unbacked-up.** CLAUDE.md's whole justification for the
   allowlist mechanism is that each entry carries a reason a human confirmed. Those
   reasons exist in exactly one place, on one disk, outside version control. `rm -rf
   .context/working/` loses the review history permanently and silently.
3. **Same class as T-2680.** A guard whose reported health depends on unversioned local
   state is a guard whose green is not evidence. T-2680 fixed a canary that over-reported
   its scope; this fixes checks whose result is not reproducible at all.

Note `check-silent-exit` is unaffected in practice — it currently allowlists nothing, so
it scans clean anywhere. The mechanism fix still applies to it for symmetry.

**All 15 acknowledged sites were re-verified by direct reading for this task** rather than
copied on trust, so the tracked allowlists carry confirmed reasons:
- alloc `tools.rs::with_capacity(total_tools)` — derived from the internal registry
- alloc `tools.rs::with_capacity(count as usize)` + `vec!(count as usize)` — guarded by
  `validate_dispatch_count(p.count)` above the sites (a `validate_*` guard the grep
  cannot see — the documented false-positive class)
- alloc `codec.rs::vec!(payload_len)` — `FrameHeader::decode` rejects
  `payload_length > MAX_PAYLOAD_SIZE` (16 MiB) at `data.rs:102`
- alloc `events.rs::with_capacity(capacity)` — library constructor param, internal callers
- drain ×6 — all `current_exe()` (or `bash <script>`) + fixed subcommand +
  `kill_on_drop(true)` + `stdin(null)` + `tokio::time::timeout`; bounded by the
  subcommand's own emission, not peer bytes
- busy-spin ×4 — error arm `bail!`s out of the loop, so no re-dispatch and no spin

Found by the T-2678 charter guard-coverage review (finding F4), while verifying that the
guard layer could be trusted at all.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] All four checks resolve their allowlist via a **tracked-first fallback**: `.context/checks/<name>-allowlist` when present, else the legacy `.context/working/.<name>-allowlist` (so the origin checkout keeps working un-migrated), with the explicit `--allowlist` / env override still winning over both
- [x] Tracked allowlists created under `.context/checks/` for alloc-sink (5), drain-sink (6), busy-spin (4), silent-exit (0 entries, header only), each entry carrying a re-verified cited reason
- [x] All four tracked allowlists are `git ls-files`-visible (the whole point)
- [x] All four checks exit 0 **in this worktree** — i.e. the guard layer is reproducible off the origin checkout, which it was not before
- [x] Allowlists remain load-bearing: deleting an entry re-fires that site
- [x] Each check's header documents the tracked path and why it is tracked
- [x] Existing fixture suites still pass (`tests/{alloc-sink,drain-sink,silent-exit,busy-spin}-check-fixtures.sh`) — the `--allowlist` override path must be unchanged
- [x] CLAUDE.md updated: the four checks' documented allowlist paths corrected, and the "scans CLEAN" claims made true-anywhere rather than true-on-one-machine

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

bash tests/alloc-sink-check-fixtures.sh
bash tests/drain-sink-check-fixtures.sh
bash tests/silent-exit-check-fixtures.sh
bash tests/busy-spin-check-fixtures.sh
bash scripts/check-alloc-sink-clamps.sh --no-heartbeat
bash scripts/check-drain-sink-caps.sh --no-heartbeat
bash scripts/check-silent-exit.sh --no-heartbeat
bash scripts/check-busy-spin.sh --no-heartbeat
git ls-files --error-unmatch .context/checks/alloc-sink-allowlist
git ls-files --error-unmatch .context/checks/drain-sink-allowlist
git ls-files --error-unmatch .context/checks/busy-spin-allowlist
git ls-files --error-unmatch .context/checks/silent-exit-allowlist

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

### 2026-08-13T23:27:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.claude/worktrees/charter-review-2026-0814/.tasks/active/T-2681-static-check-allowlists-live-under-a-git.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-41d9c8d6
- **Timestamp:** 2026-08-23T20:05:12Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-23T20:04:26Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
