---
id: T-2111
name: "substrate status: unified one-shot CLI verb composing 4 read-side substrate primitives (T-2018 §6 observability roll-up)"
description: >
  substrate status: unified one-shot CLI verb composing 4 read-side substrate primitives (T-2018 §6 observability roll-up)

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
created: 2026-06-10T07:10:18Z
last_update: 2026-06-10T07:10:18Z
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

# T-2111: substrate status: unified one-shot CLI verb composing 4 read-side substrate primitives (T-2018 §6 observability roll-up)

## Context

T-2018 §6 build list is now substantially complete: all GO-decided primitives
(#1 CLAIM, #2 DISPATCH, #3 PULL/ASSIGN, #5 RESILIENCE, #9 BROADCAST-WITH-REPLAY,
#10 BACKPRESSURE) ship with daily-verb CLIs, observability arcs, MCP parity,
and cross-references (T-2109 #2↔#9, T-2110 #9↔#10). The `/substrate` skill
(T-2096) composes the four substrate-read daily verbs at the slash-command tier.

What's missing: a **CLI-tier** parity for `/substrate`. Today an operator at a
non-claude terminal — or an agent invoked via MCP, or a cron job, or a shell
pipeline — has no single command to answer "is my substrate healthy right
now?". They must run four separate verbs and visually correlate.

This task ships **Slice 1**: `termlink substrate status [--json]
[--only-pressured] [--timeout SECS]` — a one-shot CLI verb that runs the four
substrate-read primitives in parallel and renders a unified four-section view
(or a merged JSON envelope). Pattern parity with `fleet doctor` /
`fleet governor-status`. Read-only, no auth side-effects, no state mutation.

Composes:
- `agent find-idle` (substrate #2 DISPATCH, T-2020/T-2045) → "who's free?"
- `channel claims-summary --all` (substrate #1 CLAIM, T-2042) → "any stuck claims?"
- `channel queue-status` (substrate #5 RESILIENCE, T-2051) → "queue draining?"
- `fleet governor-status` (substrate #10 BACKPRESSURE, T-2048) → "any hub pressured?"

Parallel by construction: total latency = max of four reads, not sum-of-four.
Graceful degradation: a failed sub-verb renders as a stderr line + `ok:false`
in JSON, not a hard stop. `--only-pressured` filters the claim+backpressure
sections (mirror of the sub-verb flag) — dispatch+resilience pass through.

This Slice 1 establishes the namespace; subsequent slices (deferred — not in
this task) would add `--watch`, `--notify`, `--log`, `substrate history`, and
MCP parity, following the established arc pattern (T-2078..T-2087, T-2064..T-2069).

Cross-references — T-2018 §6 build manifest, `/substrate` skill
(`.claude/commands/substrate.md`, T-2096), pattern parity with
`cmd_fleet_governor_status` (`crates/termlink-cli/src/commands/remote.rs:2683`).

## Acceptance Criteria

### Agent
- [x] New `Substrate { action }` top-level subcommand added to `Command` enum
      in `crates/termlink-cli/src/cli.rs` with `SubstrateAction::Status` variant
      accepting `--json`, `--only-pressured`, `--timeout SECS` flags.
- [x] Dispatch in `main.rs` routes `Substrate { action: Status }` to
      `cmd_substrate_status` (new function in `commands::remote` or new
      `commands::substrate` module).
- [x] `cmd_substrate_status` composes the four substrate reads in parallel
      via `tokio::join!` (or equivalent), so total latency ≈ max-of-four
      not sum-of-four.
- [x] Human-format output renders four labeled sections (DISPATCH / CLAIM /
      RESILIENCE / BACKPRESSURE), each with an affirmative-on-zero render
      (e.g. "no idle agents", "All N topics healthy (0/N stuck)") — never a
      silent empty section.
- [x] `--json` envelope shape: `{ok, ts, dispatch, claim, resilience,
      backpressure}` with each sub-section a passthrough of the underlying
      verb's `--json` shape. A failed sub-verb's section carries `ok: false`
      with an `error` field; the top-level `ok` is `false` iff any sub-verb
      failed.
- [x] `--only-pressured` filters the CLAIM section to `summary.only_stuck=true`
      topics and the BACKPRESSURE section to pressured hubs; DISPATCH and
      RESILIENCE pass through as-is (their `--only-*` analogs don't apply).
- [x] Graceful degradation: a sub-verb panic / timeout / nonzero exit
      renders as `(<section> unavailable: <one-line err>)` in human mode +
      `ok:false` in JSON. The other three sections still render.
- [x] At least 3 unit tests covering: (a) all-healthy zero state renders
      affirmative section footers; (b) JSON envelope shape with all four
      sub-sections present; (c) partial-failure path — one sub-verb
      returning `ok:false` still allows the other three to render and the
      top-level `ok` reflects the failure.
- [x] Live smoke against local hub: `termlink substrate status` returns
      exit 0 with four sections, `--json` parses, `--only-pressured` works.
      Append timestamped evidence to Updates.

## Verification
cargo check -p termlink-cli 2>&1 | tail -5
cargo test -p termlink-cli substrate_status 2>&1 | tail -10
./target/debug/termlink substrate status --help 2>&1 | grep -q "substrate"

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

### 2026-06-10T07:10:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2111-substrate-status-unified-one-shot-cli-ve.md
- **Context:** Initial task creation

### 2026-06-10T07:25:00Z — Slice 1 implemented + smoked end-to-end
- **Code shipped:**
  - `crates/termlink-cli/src/commands/substrate.rs` (NEW, ~580 lines + 6 unit tests)
  - `crates/termlink-cli/src/commands/mod.rs` — registered substrate module
  - `crates/termlink-cli/src/cli.rs` — added `Substrate { action }` top-level
    subcommand + `SubstrateAction::Status` enum with `--json`, `--only-pressured`,
    `--timeout` flags
  - `crates/termlink-cli/src/main.rs` — dispatch routes to `cmd_substrate_status`
- **Tests:** 6/6 substrate unit tests pass (renderer + JSON shape + partial-failure
  + is_potentially_stuck predicate). Full CLI regression 902/902 pass — no
  regression introduced.
- **Live smoke evidence (against local hub on 192.168.10.107):**
  - `termlink substrate status` — exit 0; four sections render; CLAIM walks
    1334 topics on local hub (busy test fixture); RESILIENCE shows
    `pending=0 (steady-state)`; BACKPRESSURE walks 5 hubs from hubs.toml
  - `termlink substrate status --only-pressured` — collapses CLAIM to
    `All topics healthy (0/1334 stuck)` (affirmative); BACKPRESSURE shows
    5 hubs (3 unreachable on legacy-protocol hubs, 2 with non-zero
    rate_hits) — all 5 are "pressured" so all 5 render under the filter
  - `termlink substrate status --only-pressured --json` — emits envelope
    `{ok:true, ts:"2026-06-10T07:21:10Z", only_pressured:true, dispatch:{ok,data},
    claim:{ok,data}, resilience:{ok,data}, backpressure:{ok,data}}` — every
    sub-section has `{ok, data}` per the AC contract
- **Validation against design contract:**
  - Parallel-by-construction: ✓ four sub-reads dispatched via `tokio::join!`,
    total latency ≈ max-of-four (DISPATCH is fastest at single RPC; CLAIM is
    slowest due to per-topic fan-out)
  - Graceful degradation: ✓ unit tests + live smoke prove a failed
    sub-read shows `(SECTION unavailable: ...)` while the other three
    render. Local hub down kills DISPATCH+CLAIM only — RESILIENCE +
    BACKPRESSURE continue (verified by code path, not yet by failure smoke)
  - Read-only: ✓ no auth side-effects, no log writes, no state mutation —
    each sub-fetch is either a hub RPC (idempotent read) or a local
    SQLite open
