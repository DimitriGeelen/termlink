---
id: T-2357
name: "Activate T-2355 walk deadline on .107: rebuild, reinstall CLI, restart local hub"
description: >
  T-2355 shipped the server-side walk deadline in code (c6e226ee) but the .107 local hub + installed CLI (0.11.321, built pre-c6e226ee) predate it. Rebuild release, reinstall via rm-then-cp (T-2356 pattern), restart the local hub (runtime_dir persists per preflight Check 1 — no rotation expected), verify hub serves the new binary and version >= 0.11.322.

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
created: 2026-07-04T14:56:28Z
last_update: 2026-07-04T14:56:28Z
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

# T-2357: Activate T-2355 walk deadline on .107: rebuild, reinstall CLI, restart local hub

## Context

Deployment/activation companion to T-2355 (same pattern as T-2356 was for T-2352/53/54). The walk-deadline fix lives in the hub code path, which serves from the running hub process's binary — code shipped at c6e226ee but both the installed CLI (0.11.321, built at 30a4b559) and the running .107 hub predate it (PL-210: stale-binary class is bidirectional, CLI and HUB both fail the same way). Rebuild release, reinstall via rm-then-cp (avoids ETXTBSY), restart the local hub. Preflight Check 1 PASSes (runtime_dir off /tmp) so hub.secret + hub.cert.pem persist across the restart — no rotation, no client re-pin expected (T-933 persist-if-present ground truth).

## Acceptance Criteria

### Agent
- [x] `cargo build --release -p termlink` at or after c6e226ee completes cleanly; artifact mtime fresh (PL-209 check) — built at 5c171bf1, exit 0, 11m06s, artifact mtime 17:08, reports 0.11.324
- [x] Binary reinstalled to /root/.cargo/bin via rm-then-cp; `termlink --version` >= 0.11.322 — installed 0.11.324; previous 0.11.321 backed up to termlink.0.11.321.bak
- [x] Local hub restarted and healthy — BONUS ROOT-CAUSE: the old hub (PID 3515069) was a DETACHED process while termlink-hub.service crash-looped every 5s ("Hub is already running", restart counter 2178). Correct restart = `hub stop` the detached process → systemd auto-restart brought the hub up from the NEW binary under proper supervision. Unit now `active (running)`, Main PID 3475796, ExecStart /root/.cargo/bin/termlink (0.11.324), flap resolved
- [x] No auth rotation from the restart: `termlink hub fingerprint` identical pre/post (sha256:d1bd50f5cb03c4fd…); authed local call succeeded without re-pin; `fleet verify`: all reachable pinned hubs `match` (only laptop-141 probe-failed — host offline, pre-existing, unrelated)
- [x] Post-restart smoke: `termlink channel info agent-chat-arc` walk succeeds (count=5808) under the now-active 20s walk deadline; `/be-reachable` was not active pre-restart (no state file) so nothing to re-establish; registrations are file-backed in /var/lib/termlink/sessions/ and survive the restart

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

termlink --version > /tmp/.t2357-ver.out 2>&1 && grep -q "termlink 0.11.324" /tmp/.t2357-ver.out
systemctl is-active termlink-hub > /tmp/.t2357-unit.out 2>&1 && grep -q "^active" /tmp/.t2357-unit.out
termlink hub fingerprint > /tmp/.t2357-fp.out 2>&1 && grep -q "d1bd50f5cb03c4fd" /tmp/.t2357-fp.out

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

### 2026-07-04T14:56:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2357-activate-t-2355-walk-deadline-on-107-reb.md
- **Context:** Initial task creation

### 2026-07-04T15:20:00Z — activation-completed [agent]
- **Action:** Rebuilt (0.11.324 at 5c171bf1), reinstalled via rm-then-cp, stopped the DETACHED hub (PID 3515069), systemd auto-restart took over from the new binary (Main PID 3475796) — T-2355 walk deadline now live on .107
- **Bonus root-cause:** termlink-hub.service had been crash-looping every 5s for 2178 restarts ("Hub is already running") against the detached hub's pidfile. Flap resolved; hub back under proper supervision (crash-restart + reboot-survival restored). Framework blindness to the class registered as G-070; prevention task T-2358 filed (preflight Check 6)
- **Evidence:** fingerprint unchanged pre/post (sha256:d1bd50f5…); fleet verify all reachable hubs match; agent-chat-arc walk count=5808 under the active 20s deadline
