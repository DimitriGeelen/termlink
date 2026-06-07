---
id: T-2042
name: "Substrate Slice 9: channel claims-summary --all for fleet-wide stuck-worker sweep"
description: >
  Substrate Slice 9: channel claims-summary --all for fleet-wide stuck-worker sweep

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:arc-parallel-substrate, slice-9]
components: []
related_tasks: [T-2019, T-2018, T-2039, T-2041]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-07T22:45:18Z
last_update: 2026-06-07T22:45:18Z
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

# T-2042: Substrate Slice 9: channel claims-summary --all for fleet-wide stuck-worker sweep

## Context

The Slice 6 `claims-summary` verb (T-2039) requires an operator to already
know which topic they want to check. In a real homelab with many topics
acting as work queues, an operator running incident triage usually
doesn't know which topic has the stuck worker — they want a fleet view
to scan first.

This slice adds `--all` to `channel claims-summary`. With `--all`, the
verb queries `channel.list` to enumerate every topic, then per-topic
calls `channel.claims_summary`, renders one line per topic, and
annotates topics with `expired_count > 0` OR
`oldest_active_age_ms > 60_000` (60s) as `[POTENTIALLY STUCK]` for
quick visual scanning. Footer shows total topics + count of stuck.

Composes naturally with `--json` (array envelope) and with `--watch`
(Slice 8) — `--all --watch 30` gives "live fleet stuck-worker dashboard"
for incident triage.

No new RPC, no new bus method — pure CLI-side composition of two
already-shipped surfaces (`channel.list` + `channel.claims_summary`).

## Acceptance Criteria

### Agent
- [x] `ClaimsSummary` enum variant in `crates/termlink-cli/src/cli.rs`: `topic` field becomes `Option<String>` (was required `String`); new `all: bool` flag added with doc-comment naming T-2042
- [x] cli.rs / `cmd_channel_claims_summary` validates: exactly one of `topic` or `--all` is provided. Both / neither produces a clear error before any RPC.
- [x] When `--all` is set in text mode, the verb queries `channel.list`, iterates topics in returned order, calls `channel.claims_summary` per topic, prints one line per topic in the existing single-topic format, annotates `[POTENTIALLY STUCK]` when `expired_count > 0` OR `oldest_active_age_ms > 60_000`
- [x] When `--all` is set in JSON mode, the verb emits an envelope `{ok, topic_count, stuck_count, topics: [...]}` where each topic entry has the existing per-topic shape PLUS a `potentially_stuck: bool` field
- [x] `--all` composes with `--watch <secs>` — watch loop re-renders the fleet sweep every clamped interval (same loop/error-tolerance shape as Slice 8)
- [x] Per-topic fetch errors during the sweep are non-fatal: emit `topic "<name>": fetch error: <e>` on stderr (text mode) or `{ok: false, error: ...}` in the JSON array entry, and continue iterating
- [x] Text-mode footer: `(N topic(s), M with potentially stuck claims)` after the per-topic rows
- [x] `main.rs` dispatch arm passes new parameters through
- [x] Operator runbook (`docs/operations/substrate-claim-primitive.md`) "Stuck-worker pattern" section gains an `--all` recipe; References section gains a Slice 9 entry
- [x] `cargo build --release -p termlink` clean
- [x] `cargo test --release -p termlink` clean (3 pre-existing unrelated failures noted in T-2041 still present; no new regressions — same failure set on a56c875d before this change)

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

grep -q "all: bool" crates/termlink-cli/src/cli.rs
grep -q "T-2042" crates/termlink-cli/src/cli.rs
grep -q "T-2042" crates/termlink-cli/src/commands/channel.rs
grep -q "POTENTIALLY STUCK" crates/termlink-cli/src/commands/channel.rs
grep -q "claims-summary --all" docs/operations/substrate-claim-primitive.md
grep -q "Slice 9" docs/operations/substrate-claim-primitive.md
cargo build --release -p termlink 2>&1 | tail -3 | grep -qE "Compiling|Finished"

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

### 2026-06-08 — live verified at fleet scale (1328 topics on local hub) + happy-path on fresh hub
- **What changed:** Two unexpected gains during live verification, both ergonomic
  rather than architectural:
  (a) Local-hub stress test — pointing `--all --json` at the live `/var/lib/termlink`
  hub (1328 topics, OLD binary that lacks `channel.claims_summary`) returned a
  clean per-topic-error envelope in a single response. Every entry was the
  expected `{ok: false, error: "hub error: code=-32001 ...", topic: "..."}`
  shape, and the sweep completed without partial-state. This was an
  unintentional fleet-scale soak that the Slice 9 error-tolerance AC
  promised to handle — it does.
  (b) The text-mode `[POTENTIALLY STUCK]` annotation, designed around the
  expired-count path, fires immediately on a 100ms-TTL claim that lapses
  before the next sweep. Footer count tracks correctly (`1 with
  potentially stuck claims` after one expiry). Did not need to wait 60s
  for the age-threshold path to see the visual scan signal.
- **Plan impact:** None. All 11 Agent ACs ticked from the original plan.
- **Triggered:** None. The substrate first-primitive observability axis now
  spans single-topic + fleet-wide + watch-mode + JSON across both axes —
  the operator coverage is complete for this primitive. Next substrate work
  needs operator GO/NO-GO on T-2020..T-2028.

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

### 2026-06-07T22:45:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2042-substrate-slice-9-channel-claims-summary.md
- **Context:** Initial task creation
