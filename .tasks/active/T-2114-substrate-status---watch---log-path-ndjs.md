---
id: T-2114
name: "substrate status --watch --log <PATH> NDJSON audit trail — Slice 4 (T-2111 arc, T-2018 §6)"
description: >
  substrate status --watch --log <PATH> NDJSON audit trail — Slice 4 (T-2111 arc, T-2018 §6)

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
created: 2026-06-10T08:08:17Z
last_update: 2026-06-10T08:08:17Z
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

# T-2114: substrate status --watch --log <PATH> NDJSON audit trail — Slice 4 (T-2111 arc, T-2018 §6)

## Context

Slice 4 of T-2111 substrate-status observability roll-up arc under T-2018 §6.
T-2113 closed the `--notify` event-hook layer (operator-pluggable shell
command fired fire-and-forget per rollup-field change). This slice adds
the `--log <PATH>` audit-trail companion — append-only NDJSON one line
per change event so an operator can answer "when did substrate health
flip?" retrospectively without keeping the watch terminal attached.

Pattern parity:
- T-2080 `agent find-idle --watch --log` / T-2081 retrospective verb
- T-2066 `fleet governor-status --watch --log` / T-2068 retrospective
- T-2073 `channel claims-summary --watch --log` / T-2074 retrospective
- T-2085 `channel queue-status --watch --log` / T-2086 retrospective

Pure delta on top of T-2112 + T-2113:
- New clap flag `--log <PATH>` on SubstrateAction::Status (requires watch).
- New pure helper `render_log_line(field, old, new, ts) -> String` emitting
  one flat NDJSON line: `{ts, field, old, new}`.
- New best-effort `append_log_line(path, field, old, new, ts)` — parent
  dir auto-created; disk-full / permission errors print one-line stderr
  warning, watch never crashes.
- Wired into the per-event fire path next to `fire_notify` so each event
  lands in both surfaces when both flags are set.

Symmetric write-once + read-many: this slice ships the write side; the
retrospective `substrate history` verb is deferred to Slice 5 (next).

Design constraints (from T-2080/T-2085 priors):
- `--log` requires `--watch` (events only exist across ticks).
- Cardinality lock at 4 fields per line — jq-friendly, no nested objects.
- Best-effort writes — observability outage MUST NOT kill the watch loop.
- Symmetric with `--notify` — both can be set; same per-tick event source.

## Acceptance Criteria

### Agent
- [x] `--log <PATH>` flag added to `SubstrateAction::Status` clap variant
      with `requires = "watch"` constraint; `cargo check -p termlink` passes.
- [x] Pure helper `render_log_line(field, old, new, ts)` emits exactly
      one NDJSON line with 4 fields: `ts`, `field`, `old`, `new`. Unit
      test asserts the field set + JSON-parseability.
- [x] `append_log_line` auto-creates the parent directory when missing;
      unit test writes to `/tmp/T-2114/sub/dir/file.log` and asserts the
      directory chain was created + the file exists with the rendered line.
- [x] `append_log_line` is best-effort: opens with `O_APPEND | O_CREAT`;
      disk-full / permission errors print one-line stderr warning and
      return cleanly (watch never crashes).
- [x] Wired into `cmd_substrate_status_watch` next to `fire_notify` —
      every per-cycle rollup-field change event appends one log line.
      Baseline cycle skipped (matches T-2113 notify semantics).
- [x] `cargo test -p termlink --bin termlink substrate` passes (≥17 tests
      after Slice 4 adds 2 new).
- [x] CLI `--log` rejected without `--watch` (clap `requires` enforces);
      reject prints the clap usage hint.
- [x] Full regression: `cargo test -p termlink --bin termlink` passes
      (≥909 tests as of Slice 3 baseline).
- [x] Live smoke against local hub: run `substrate status --watch 5
      --log /tmp/T-2114-smoke.log` for ~2 cycles, induce a `claim_topic_count`
      change via `channel create T-2114-smoke-<ts> --retention "days:1"`,
      assert the log file ends up with at least one valid NDJSON line
      whose `field == "claim_topic_count"`. Append timestamped Updates evidence.

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

cargo check -p termlink 2>&1 | tail -5
cargo test -p termlink --bin termlink substrate 2>&1 | tail -10
./target/debug/termlink substrate status --help 2>&1 | grep -q "\-\-log"

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

### 2026-06-10T08:08:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2114-substrate-status---watch---log-path-ndjs.md
- **Context:** Initial task creation

### 2026-06-10T08:14:00Z — slice 4 shipped end-to-end
- **Action:** Implemented `substrate status --watch --log <PATH>` audit
  trail. cli.rs: added `--log <PATH>` flag with `requires = "watch"`
  constraint. main.rs: threaded `log` param into watch dispatch.
  substrate.rs: added pure `render_log_line` + best-effort
  `append_log_line` helpers + wired into the per-event fire path next to
  `fire_notify` (line ~1024).
- **Verification:**
  - `cargo check -p termlink` — PASS (15.35s)
  - `cargo test -p termlink --bin termlink substrate` — 17/17 PASS
    (2 new: `render_log_line_shape_and_fields`,
    `append_log_line_auto_creates_parent_dir`)
  - `cargo test -p termlink --bin termlink` — 913/913 PASS (was 909
    baseline pre-Slice 4)
  - `./target/debug/termlink substrate status --log /tmp/foo.log` —
    exits 2 with clap "required arguments were not provided: --watch"
    (clap `requires` constraint enforced)
  - Live smoke against local hub (`./target/debug/termlink substrate
    status --watch 5 --log /tmp/T-2114-smoke.log`):
    - Baseline cycle printed all 4 sections (no log write)
    - 6 ticks elapsed; 2 silent cycles before induced change, 4 after
    - Induced `claim_topic_count: 1338→1339` via
      `channel create T-2114-smoke-1781079213 --retention "days:1"`
    - Real-state transition fired: stdout =
      `2026-06-10T08:13:36Z  claim_topic_count: 1338→1339`
    - Log file `/tmp/T-2114-smoke.log` contains exactly 1 NDJSON line:
      `{"field":"claim_topic_count","new":"1339","old":"1338","ts":"2026-06-10T08:13:36Z"}`
    - 4-field cardinality lock holds; stringified numerics (matches
      `--notify` env-var convention); RFC3339 ts
    - SIGINT clean exit: `substrate-watch stopped (sigint, completed 7
      cycle(s))`
- **Outcome:** Slice 4 closes the write side of the audit trail. Slice 5
  (`substrate history` retrospective read verb, mirror of T-2081 / T-2068
  / T-2086) is the natural next slice — same write surface, retrospective
  read aggregator.
- **Context:** T-2018 §6 observability roll-up arc — T-2111 Slice 4.
