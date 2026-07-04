---
id: T-2360
name: "Deploy scripts refuse setsid-nohup hub relaunch on systemd-supervised targets (G-070 recreation vector)"
description: >
  Deploy scripts refuse setsid-nohup hub relaunch on systemd-supervised targets (G-070 recreation vector)

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
created: 2026-07-04T22:45:52Z
last_update: 2026-07-04T22:50:11Z
date_finished: 2026-07-04T22:50:11Z
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

# T-2360: Deploy scripts refuse setsid-nohup hub relaunch on systemd-supervised targets (G-070 recreation vector)

## Context

G-070 origin: T-2351 relaunched the .107 hub via `setsid nohup` while a systemd
unit supervised it — the unit crash-looped 2178 times ("Hub is already running")
against the detached ghost. Both deploy helpers still encode that exact
mechanism unconditionally: `fleet-deploy-binary.sh --swap-restart` (generated
remote runner) and `hub-binary-swap.sh` (main relaunch + rollback relaunch)
use `setsid nohup ... hub start`. hub-binary-swap.sh's header even states the
no-systemd assumption ("On hosts without a systemd unit / watchdog") but
nothing guards it. Fix: loud-refuse (IW-3) when the target host has a
termlink-hub systemd unit — restarts must go THROUGH the unit
(`systemctl restart termlink-hub`) so supervision, crash-restart and
reboot-survival are preserved; `--force-detached` overrides for operators who
know better (visible in deploy logs). Preflight Check 6 (T-2358) detects the
ghost after the fact; this guard prevents creating it.

## Acceptance Criteria

### Agent
- [x] `hub-binary-swap.sh`: pre-swap check queries the target for `termlink-hub.service` (unit file present); if found and `--force-detached` not passed, the script REFUSES before any swap action with a message naming the unit, the correct path (install staged binary + `systemctl restart termlink-hub`), and the `--force-detached` override — UNIT_PRESENT emitted from the existing PRE_STATE probe (read-only), guard fires before dry-run/swap, exit 2 (pre-condition failure class)
- [x] `fleet-deploy-binary.sh --swap-restart`: same guard before generating/pushing the detached deploy runner (staging is unaffected — refusal message names the staged path so the operator can `install && systemctl restart` directly); `--force-detached` flag added; exit 4 (deploy-phase class)
- [x] `--force-detached` on either script prints a loud one-line `!!!` warning that a detached hub on a systemd host recreates G-070 (unit flap + lost supervision) so the override is visible in deploy logs
- [x] Guard is fail-open on probe error: hub-binary-swap.sh treats non-1/0/no-systemd UNIT_PRESENT as inconclusive-proceed; fleet-deploy-binary.sh wraps the remote probe in `{ ... || true; }` so a remote-exec failure yields empty → inconclusive-proceed with an info line (unit-less hosts, the scripts' original audience, are unaffected)
- [x] Syntax check passes on both scripts (`bash -n` clean); `G-070 guard` marker present in both files; probe command smoke-tested on .107 (unit present → returns 1, guard would refuse)

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

bash -n scripts/hub-binary-swap.sh
bash -n scripts/fleet-deploy-binary.sh
grep -q "G-070 guard" scripts/hub-binary-swap.sh
grep -q "G-070 guard" scripts/fleet-deploy-binary.sh
grep -q "force-detached" scripts/hub-binary-swap.sh
grep -q "force-detached" scripts/fleet-deploy-binary.sh

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

### 2026-07-04T22:45:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2360-deploy-scripts-refuse-setsid-nohup-hub-r.md
- **Context:** Initial task creation

### 2026-07-05T01:00:00Z — shipped [agent]
- **Action:** G-070 guard added to both deploy helpers: `hub-binary-swap.sh` (UNIT_PRESENT via existing PRE_STATE probe; refuse exit 2 pre-swap) and `fleet-deploy-binary.sh --swap-restart` (dedicated remote probe before runner generation; refuse exit 4, staging unaffected). `--force-detached` override on both with loud `!!!` log line; fail-open on probe error/no-systemd. Usage headers updated.
- **Evidence:** `bash -n` clean on both; probe smoke-tested on .107 (unit present → 1); markers grep-able
- **Context:** Closes the G-070 recreation vector — T-2351's setsid-nohup relaunch is exactly what these scripts would have done to any systemd host. Preflight Check 6 (T-2358) detects the ghost after the fact; this prevents creating it.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-1b4925f4
- **Timestamp:** 2026-07-04T22:50:12Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-04T22:50:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
