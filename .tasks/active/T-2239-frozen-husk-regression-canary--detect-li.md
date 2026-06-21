---
id: T-2239
name: "Frozen-husk regression canary — detect live sessions whose heartbeat never advances (G-019 prevention for T-2230/T-2235)"
description: >
  Frozen-husk regression canary — detect live sessions whose heartbeat never advances (G-019 prevention for T-2230/T-2235)

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
created: 2026-06-21T15:26:57Z
last_update: 2026-06-21T15:26:57Z
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

# T-2239: Frozen-husk regression canary — detect live sessions whose heartbeat never advances (G-019 prevention for T-2230/T-2235)

## Context

G-019 prevention follow-up for the T-2230/T-2235 heartbeat-fix arc. Those tasks
fixed the *symptom* (registered sessions whose `heartbeat_at` never advanced —
"frozen husks"). The framework was structurally *blind* to that class for as long
as the bug existed: a live process could register and never heartbeat, and nothing
surfaced it. This canary closes the detection gap.

Detection: walk `$TERMLINK_RUNTIME_DIR/sessions/*.json`; a "frozen husk" is a
session whose pid is ALIVE but whose `heartbeat_at` is older than a threshold
(default 600s, well beyond the 30s heartbeat interval + slack). Dead-pid
registrations are reported as orphan cruft (informational, non-firing) since
they are a different class (cleanup, not the bug). Empty log = healthy, matching
the mirror / substrate-preflight / framework-pickup canary convention.

## Acceptance Criteria

### Agent
- [x] scripts/check-frozen-husk-freshness.sh exists, is executable, --help works, exit codes 0/1/2 documented
- [x] Detects a live frozen husk (pid alive + heartbeat_at stale > threshold); fires exit 1 with the offending session id/pid/age — found 11 live v0.9.0 husks on core host, /proc-confirmed
- [x] Healthy/no-husk case exits 0; --quiet prints nothing on healthy; --json emits a parseable envelope — verified clean dir + high-threshold both exit 0
- [x] Heartbeat-first touch (.context/working/.frozen-husk-canary.heartbeat) so /canaries auto-discovers and aliveness-checks it — /canaries shows "HEALTHY frozen-husk-canary"
- [x] Daily crontab def added (.context/cron/frozen-husk-canary.crontab) following the established empty-log-=-healthy redirect pattern — installed to /etc/cron.d/termlink-frozen-husk-canary
- [x] CLAUDE.md documents the canary alongside the other canaries (mirror/substrate-preflight/framework-pickup) — added "### Frozen-husk canary (T-2239...)" section

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

bash scripts/check-frozen-husk-freshness.sh --help >/dev/null
bash -n scripts/check-frozen-husk-freshness.sh
test -x scripts/check-frozen-husk-freshness.sh
test -f .context/cron/frozen-husk-canary.crontab
out=$(bash scripts/check-frozen-husk-freshness.sh --json 2>/dev/null); echo "$out" | python3 -c "import sys,json; json.loads(sys.stdin.read())"
grep -q "frozen-husk" CLAUDE.md

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

**Symptom:** Registered sessions ("frozen husks") whose `heartbeat_at` never
advanced — a live `termlink register` process appeared in the session store but
its presence/liveness telemetry was permanently stale. Surfaced concretely on
the core host: 11 live v0.9.0 `termlink register` processes, every one with
`created_at == heartbeat_at` and ages up to ~4 days.

**Root cause (of the framework blindness, not the heartbeat bug itself):** there
was no monitor for "live registration with stale heartbeat". The heartbeat-fix
arc (T-2230 cmd_register, T-2235 cmd_register_self) repaired the producing code,
but the framework had no detector that would have caught the original bug — or
would catch a regression / a field host still running a pre-fix binary.

**Why structurally allowed:** heartbeat freshness of the local session store was
never part of any canary/health sweep. `fleet doctor` and the presence rail look
at hub-side state and remote reachability, not at whether *this host's own*
registrations are advancing their heartbeat. The gap persisted as long as the
heartbeat bug did (G-019: a >7-day framework blindness, here much longer).

**Prevention:** `scripts/check-frozen-husk-freshness.sh` + daily cron
(`.context/cron/frozen-husk-canary.crontab`) walk the local runtime_dir and fire
(exit 1) on any live, /proc-confirmed termlink process whose heartbeat exceeds
the threshold. `/canaries` auto-discovers it via the `.heartbeat` companion.
Empty-log-=-healthy convention. A persistent husk on a current binary now
surfaces as a fired canary instead of silent rot.

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

### 2026-06-21T15:26:57Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2239-frozen-husk-regression-canary--detect-li.md
- **Context:** Initial task creation
