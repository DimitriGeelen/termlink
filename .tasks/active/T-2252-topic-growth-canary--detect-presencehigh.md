---
id: T-2252
name: "Topic-growth canary — detect presence/high-rate topic regrowth (R2 sweep-cron guard)"
description: >
  Topic-growth canary — detect presence/high-rate topic regrowth (R2 sweep-cron guard)

status: started-work
workflow_type: build
owner: agent
horizon: now
arc_id: arc-substrate-fitness
tags: [arc:arc-substrate-fitness]
components: []
related_tasks: [T-2242, T-2245, T-2058, T-1991]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T08:50:40Z
last_update: 2026-06-23T08:50:40Z
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

# T-2252: Topic-growth canary — detect presence/high-rate topic regrowth (R2 sweep-cron guard)

## Context

R2 (T-2245) bounds `agent-presence` via `set-retention latest-per-cv-key` + a
periodic `sweep` — but the bus runs NO background thread (T-1155 design:
enforcement is explicit), so the sweep depends on an operator **cron that may
never fire**. If it doesn't, presence regrows silently — a T-1991 recurrence with
nothing to surface it. This is the PL-168 / "framework relies on out-of-band
hygiene that may never run" class (I flagged the sibling gap in T-2251's RCA).
The canary fleet (mirror-drift, frozen-husk, framework-pickup, substrate-preflight)
guards exactly this class. This task adds the missing **topic-growth canary** so a
regrown high-rate topic is caught, not silently rotting. arc-002, AS_RESOURCE_FOOTPRINT.

## Acceptance Criteria

### Agent
- [x] `scripts/check-topic-growth-freshness.sh` reads `termlink channel list --json` and FIRES (exit 1, body printed even under `--quiet` so the cron log captures it) when a watched high-rate topic's record `count` exceeds the threshold (default 5000, `--threshold N` tunable).
- [x] Watch-set = high-rate-pattern topics (`agent-presence`, `agent-listeners-*`, `agent-conv-*`, `dm:*`; tunable via `TERMLINK_GROWTH_WATCH_PATTERNS` csv). Operator-durable topics (`channel:learnings`, `policy-decisions`, `framework:pickup`, `broadcast:global`) are excluded — never fire (mirrors runbook §1 exclusions).
- [x] Each firing topic reports `count` + `retention.kind` + a remediation hint that distinguishes `forever` (run `set-retention latest-per-cv-key` + `sweep`) from a bounded-but-large policy (the sweep cron isn't firing).
- [x] Flags: `--quiet` (cron), `--json` (envelope), `--threshold N`, `--hub ADDR`, `--no-heartbeat`, `-h/--help`. Exit 0 healthy / 1 firing / 2 tooling-error (hub unreachable). Writes `.context/working/.topic-growth-canary.heartbeat` first (T-1723) so `/canaries` can classify STALE.
- [x] Test hook `TERMLINK_GROWTH_TEST_JSON=<file>` bypasses the live hub call and reads canned `channel list` JSON — so firing logic is verifiable hub-independently (PL-213: assert the property).
- [x] `.context/cron/topic-growth-canary.crontab` authored (USER-field syntax, daily at an offset time, `--quiet` >> `.context/working/.topic-growth-canary.log`).
- [x] CLAUDE.md canary section added (mirrors the existing four); `/canaries` auto-discovers the log via its existing `.*-canary.log` glob (no code change needed — verify the glob covers it).
- [x] Verification block (hub-independent) passes: `bash -n`, `--help` exit 0, canned-over-threshold JSON → exit 1 + topic named, canned-healthy JSON → exit 0, crontab references the script.

### Human
_None — all acceptance criteria are agent-verifiable (script + canned-JSON tests). Cron INSTALLATION on a host is operator action, like every other canary._

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
bash -n scripts/check-topic-growth-freshness.sh
bash scripts/check-topic-growth-freshness.sh --help >/dev/null
out=$(TERMLINK_GROWTH_TEST_JSON=scripts/testdata/topic-growth-over.json bash scripts/check-topic-growth-freshness.sh --threshold 100 --no-heartbeat 2>&1; echo "EXIT=$?"); echo "$out" | grep -q "EXIT=1"
echo "$out" | grep -q 'agent-presence'
TERMLINK_GROWTH_TEST_JSON=scripts/testdata/topic-growth-healthy.json bash scripts/check-topic-growth-freshness.sh --threshold 100 --no-heartbeat --quiet
out2=$(TERMLINK_GROWTH_TEST_JSON=scripts/testdata/topic-growth-over.json bash scripts/check-topic-growth-freshness.sh --threshold 100 --no-heartbeat --json 2>&1); echo "$out2" | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['ok'] is False"
grep -q check-topic-growth-freshness.sh .context/cron/topic-growth-canary.crontab

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

**Symptom:** `agent-presence` grew to thousands of stale records (T-1991; T-2057
found 13,443 envelopes on one topic).

**Root cause:** retention was `forever`, and even after R2 ships per-key
compaction the enforcing `sweep` has no in-process trigger (T-1155: no background
thread) — it depends on an operator cron.

**Why structurally allowed:** the framework had no detector for "a should-be-bounded
topic has regrown" — so a missing/broken sweep cron is invisible until the next
manual audit. Same "relies on out-of-band hygiene that may never run" gap as
T-2251's audit log (sibling to PL-168).

**Prevention:** a daily canary that reads `channel list` and fires when a watched
high-rate topic exceeds a bound — surfacing both "retention never set" (`forever`)
and "sweep cron not firing" (bounded policy but large count). Closes the detection
gap; `/canaries` makes it operator-visible.

## Evolution

### 2026-06-23 — topic-growth canary (R2 sweep-cron guard)
- **What changed:** while completing R7-prevention (T-2251 audit-log rotation) I
  flagged in its RCA that R2's sweep is cron-dependent and the framework has no
  guard for a non-firing sweep cron — the same class T-2251 fixed for the audit
  log, but for presence/high-rate topics.
- **Plan impact:** adds a detection layer the arc plan didn't enumerate; complements
  R2 (bounds when swept) + R7 (bounds the audit log) by catching silent regrowth.
- **Triggered:** this task (T-2252), built as the structural guard against silent
  R2 regression. Pattern-consistent with the existing canary fleet.

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

### 2026-06-23T08:50:40Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2252-topic-growth-canary--detect-presencehigh.md
- **Context:** Initial task creation
