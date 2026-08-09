---
id: T-2558
name: "dead-letter canary — durable poison-drop sink has no cron detection (antifragility gap)"
description: >
  dead-letter canary — durable poison-drop sink has no cron detection (antifragility gap)

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
created: 2026-08-09T07:50:33Z
last_update: 2026-08-09T07:50:33Z
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

# T-2558: dead-letter canary — durable poison-drop sink has no cron detection (antifragility gap)

## Context

Antifragility-lens review (T-2468 campaign) found a durable-failure sink with no
passive detection. T-2243 correctly stopped *silent* poison-drops: a `channel post`
rejected `POISON_THRESHOLD=10` times (unknown topic, bad signature, or a governance
message rejected during a hub blip) is now durably MOVED to the offline queue's
`dead_letters` table instead of `DELETE`d — the charter-core "exchange durable
messages" path. But that row is surfaced ONLY if a human runs `/queue-status`. No
`scripts/check-*.sh` and no `.context/cron/*.crontab` reads `dead_letter`/
`queue-status`. The T-2083–2087 queue observability arc fires only on
`Drained`/`Pending` transitions, never on dead-letter growth. So a poison-dropped
*guaranteed* message sits forever with nothing firing — the exact "write-only sink
nobody noticed" class (G-063) the unconfirmed-delivery canary closed for the
await-ack path. `queue-status --json` already surfaces `dead_letters` (count) +
`dead_letter_rows`; this canary wraps it (empty-log = healthy), firing on
`dead_letters > 0`.

## Acceptance Criteria

### Agent
- [x] `scripts/check-dead-letter-freshness.sh` exists, is executable, reads `channel queue-status --json`, and FIRES (exit 1) when `dead_letters > 0`; exits 0 (healthy) when `dead_letters == 0` OR the queue file does not exist yet (`exists:false`); exit 2 on tooling error (malformed JSON)
- [x] Firing output names each dead-letter row's topic + reason + attempts (capped display, exact count always shown)
- [x] `--json`, `--quiet`, `--no-heartbeat` flags present; heartbeat touched FIRST at `.context/working/.dead-letter-canary.heartbeat`
- [x] Test hook `TERMLINK_DEAD_LETTER_TEST_JSON=<file>` feeds canned `queue-status --json` for hub-independent verification (PL-213)
- [x] `.context/cron/dead-letter-canary.crontab` exists, runs the script `--quiet` daily, appends to `.context/working/.dead-letter-canary.log`
- [x] Load-bearing PROVEN: a fixture with `dead_letters>0` fires (exit 1); a clean fixture and a `exists:false` fixture both exit 0 (temp-revert the firing gate → the dead-letter fixture stops firing)
- [x] CLAUDE.md documents the canary in a new "### Dead-letter canary" section matching the existing convention

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
test -x scripts/check-dead-letter-freshness.sh
# dead_letters>0 fires (exit 1)
f=$(mktemp); printf '{"exists":true,"pending":1,"dead_letters":2,"dead_letter_rows":[{"topic":"framework:pickup","reason":"unknown topic","attempts":10,"dead_lettered_ms":1},{"topic":"work-queue","reason":"bad signature","attempts":10,"dead_lettered_ms":2}]}' > "$f"; TERMLINK_DEAD_LETTER_TEST_JSON="$f" bash scripts/check-dead-letter-freshness.sh --no-heartbeat >/dev/null 2>&1; rc=$?; rm -f "$f"; test "$rc" -eq 1
# clean fires nothing (exit 0)
f=$(mktemp); printf '{"exists":true,"pending":0,"dead_letters":0,"dead_letter_rows":[]}' > "$f"; TERMLINK_DEAD_LETTER_TEST_JSON="$f" bash scripts/check-dead-letter-freshness.sh --no-heartbeat --quiet >/dev/null 2>&1; rc=$?; rm -f "$f"; test "$rc" -eq 0
# never-created queue (exists:false) is healthy (exit 0)
f=$(mktemp); printf '{"exists":false,"pending":0}' > "$f"; TERMLINK_DEAD_LETTER_TEST_JSON="$f" bash scripts/check-dead-letter-freshness.sh --no-heartbeat --quiet >/dev/null 2>&1; rc=$?; rm -f "$f"; test "$rc" -eq 0
test -f .context/cron/dead-letter-canary.crontab
grep -q "Dead-letter canary (T-2558" CLAUDE.md

## RCA

**Symptom:** A poison-dropped *guaranteed* message (rejected 10× and durably moved
to the offline queue's `dead_letters` table by T-2243) could sit there forever with
nothing firing — surfaced only if a human happened to run `/queue-status`.

**Root cause (blindness class):** T-2243 fixed the *silent-drop* symptom (durable
record instead of DELETE) but the record was a passive sink with no scheduled
reader. The T-2083–2087 queue observability arc watches Drained/Pending transitions
only, not dead-letter growth — so the fix created an inspectable-but-unwatched sink,
the "write-only sink nobody noticed" class (G-063).

**Why structurally allowed:** the same "detection accretes per-incident, not
per-durable-sink" gap. When T-2243 added the sink, no canary was added alongside it;
nothing audited "does every durable-failure record have a scheduled reader?"

**Prevention:** this canary is that scheduled reader — `dead_letters > 0` fires
daily. It reuses the existing `queue-status --json` field (T-2243) so detection
cannot drift from the sink's definition. Sibling of the unconfirmed-delivery canary
(T-2295), which closed the identical class for the await-ack path.

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

### 2026-08-09T07:50:33Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2558-dead-letter-canary--durable-poison-drop-.md
- **Context:** Initial task creation
