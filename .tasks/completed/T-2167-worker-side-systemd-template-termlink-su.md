---
id: T-2167
name: "Worker-side systemd template termlink-substrate-worker@.service + substrate-systemd.md walkthrough (T-2165 symmetry)"
description: >
  Worker-side systemd template termlink-substrate-worker@.service + substrate-systemd.md walkthrough (T-2165 symmetry)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [systemd-templates/termlink-substrate-worker@.service]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-11T14:54:02Z
last_update: 2026-06-11T14:59:12Z
date_finished: 2026-06-11T14:59:12Z
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

# T-2167: Worker-side systemd template termlink-substrate-worker@.service + substrate-systemd.md walkthrough (T-2165 symmetry)

## Context

T-2165 shipped `systemd-templates/termlink-substrate-orchestrator@.service`
+ `docs/operations/substrate-systemd.md` so operators can `cp` one .service
file + one .env and `systemctl enable --now` to run an orchestrator as a
production service. The worker side of the substrate has no equivalent —
T-2152's `substrate-worker-pickup.sh` is the canonical long-running worker
supervisor (mirror shape: poll inbox → spawn worker-loop per dispatch DM),
but every operator must hand-roll their own systemd scaffold for it. That
gap forces ad-hoc supervisors per host and breaks the symmetry of the
production-systemd surface.

T-2166 just wired preflight into pickup.sh with the same exit-4 contract
T-2163 established for the other two loops. The Restart=on-failure +
exit-4 → loud-restart-loop contract that T-2165 documented now applies
to the worker side equally. This task ships the worker template
(`termlink-substrate-worker@.service` wrapping `substrate-worker-pickup.sh`)
and extends `substrate-systemd.md` with a Worker-template install walkthrough
+ a "How preflight interacts with Restart=" entry for pickup. Closes the
production-systemd surface end-to-end (orchestrator × 1 + worker × N).

## Acceptance Criteria

### Agent
- [x] `systemd-templates/termlink-substrate-worker@.service` exists (template unit, %i = worker-id)
- [x] Template uses `EnvironmentFile=/etc/termlink/substrate-worker/%i.env` (per-instance config)
- [x] Template defaults `TERMLINK_RUNTIME_DIR=/var/lib/termlink` (load-bearing per PL-021)
- [x] Template ExecStart resolves env→flags for pickup.sh (required: TERMLINK_SW_CMD; optional: TERMLINK_SW_HUB, TERMLINK_SW_POLL_MS, TERMLINK_SW_MAX_CLAIMS)
- [x] Template wires `--worker-id %i` so the systemd instance specifier becomes the substrate identity
- [x] `Restart=on-failure` + `RestartSec=10s` (composes with T-2166 exit-4 for the loud-restart-loop contract)
- [x] Hardening matches orchestrator template: NoNewPrivileges, PrivateTmp, ProtectSystem=strict, ReadWritePaths=/var/lib/termlink /tmp, ProtectHome
- [x] Logging to journal with SyslogIdentifier=termlink-substrate-worker-%i
- [x] `docs/operations/substrate-systemd.md` extended with a "Worker template (T-2167)" install section: copy template, author env file, enable + verify
- [x] `docs/operations/substrate-systemd.md` "How preflight interacts with Restart=" table extended to call out pickup.sh by name (now that it's in the contract per T-2166)
- [x] `docs/operations/substrate-systemd.md` "Worker-side pattern" §1 (DM-driven dispatch) is rewritten to recommend the new template as the canonical install path, with the prior hand-roll prose moved to "if you need custom logic" tail
- [x] References section gains T-2166 + T-2167 entries
- [x] `docs/operations/substrate-getting-started.md` References section gets a T-2167 entry pointing at the worker template alongside T-2165
- [x] Template file passes basic shape checks (Unit/Service/Install sections present, no obvious ini errors) — systemd-analyze verify exit 0

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

# Template file exists with required sections
test -f systemd-templates/termlink-substrate-worker@.service
grep -q "^\[Unit\]"    systemd-templates/termlink-substrate-worker@.service
grep -q "^\[Service\]" systemd-templates/termlink-substrate-worker@.service
grep -q "^\[Install\]" systemd-templates/termlink-substrate-worker@.service
# Template references the right script + identity wiring
grep -q "substrate-worker-pickup.sh"      systemd-templates/termlink-substrate-worker@.service
grep -q "%i"                              systemd-templates/termlink-substrate-worker@.service
grep -q "EnvironmentFile=/etc/termlink/substrate-worker/%i.env" systemd-templates/termlink-substrate-worker@.service
grep -q "Restart=on-failure"              systemd-templates/termlink-substrate-worker@.service
grep -q "TERMLINK_RUNTIME_DIR=/var/lib/termlink" systemd-templates/termlink-substrate-worker@.service
grep -q "TERMLINK_SW_CMD"                 systemd-templates/termlink-substrate-worker@.service
# Hardening present
grep -q "NoNewPrivileges=true"            systemd-templates/termlink-substrate-worker@.service
grep -q "PrivateTmp=true"                 systemd-templates/termlink-substrate-worker@.service
grep -q "ProtectSystem=strict"            systemd-templates/termlink-substrate-worker@.service
# Doc updated with Worker template install + preflight table entry
grep -q "Worker template (T-2167)"        docs/operations/substrate-systemd.md
grep -q "substrate-worker-pickup.sh"      docs/operations/substrate-systemd.md
grep -q "T-2166"                          docs/operations/substrate-systemd.md
grep -q "T-2167"                          docs/operations/substrate-systemd.md
# Quickstart cross-ref
grep -q "T-2167"                          docs/operations/substrate-getting-started.md
# Smoke still PASS (defensive — pickup is unaffected by template addition)
scripts/substrate-smoke.sh >/dev/null 2>&1

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

### 2026-06-11T14:54:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2167-worker-side-systemd-template-termlink-su.md
- **Context:** Initial task creation

### 2026-06-11T14:59:12Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
