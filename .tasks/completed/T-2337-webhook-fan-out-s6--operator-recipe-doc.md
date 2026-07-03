---
id: T-2337
name: "Webhook fan-out S6 — operator recipe doc"
description: >
  Webhook fan-out S6 — operator recipe doc

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
created: 2026-07-03T16:17:42Z
last_update: 2026-07-03T16:17:42Z
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

# T-2337: Webhook fan-out S6 — operator recipe doc

## Context

The arc-004 webhook fan-out feature is now feature-complete (S1–S5: primitive, event-wiring,
retry/backoff/dead-letter, governor_status telemetry, CLI config verbs) but has NO operator-facing
recipe — every substrate primitive got a `docs/operations/*.md`, webhooks did not. The existing
`docs/operations/push-transport-recipe.md` only mentions webhooks as a *deferred future* (pre-GO).
S6 closes that doc gap with a dedicated recipe covering the security model (SSRF deny-by-default
allowlist + HMAC-SHA256 signing, and how a consumer VERIFIES the signature), the config surface
(`TERMLINK_WEBHOOK_CONFIG` JSON + the `webhook add/list/test` CLI verbs), enablement, retry
semantics, and observability (`hub.governor_status` `webhook_*` fields). Doc-only; no code.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `docs/operations/webhook-fan-out-recipe.md` exists with: mental model (opt-in, external HTTP fan-out, hub→consumer POST, no hard dependency), the two-layer security model (SSRF exact-host allowlist + HMAC-SHA256 `X-Termlink-Signature` signing), a copy-pasteable consumer-side signature-verification snippet, the `TERMLINK_WEBHOOK_CONFIG` JSON schema, the three CLI verbs with examples, retry/backoff/dead-letter behavior + env knobs, and the `governor_status` `webhook_*` observability fields. — 8 sections; verification greps pass.
- [x] Every CLI flag, env var, and JSON field named in the doc matches the shipped code (spot-checked against `crates/termlink-hub/src/webhook.rs` + `crates/termlink-cli/src/commands/webhook.rs`) — no invented flags (PL-206 discipline: doc examples are inert text, so they must be verified against `--help`, not assumed). — env names/defaults (`TERMLINK_WEBHOOK_RETRY_CAP`=1000, `_RETRY_INTERVAL_MS`=2000 clamp 250..60000, `WEBHOOK_MAX_ATTEMPTS`=5, `WEBHOOK_TIMEOUT_SECS`=10) read from source; CLI flags from live `--help` + smoke output.
- [x] The `webhook test` deny-by-default behavior (non-allowlisted host refuses loudly; test does NOT auto-permit the tested host — PL-239) is documented as an explicit gotcha, not glossed. — §4 "Gotcha (PL-239)" blockquote.
- [x] Doc cross-links the T-2331 inception report + the shipped-slice map (T-2332..T-2336), mirroring the "Map — where each piece shipped" section pattern from `push-transport-recipe.md`. — §8 map table.

<!-- All criteria are agent-verifiable; no Human ACs. -->

## Verification

# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).
test -f docs/operations/webhook-fan-out-recipe.md
grep -q 'TERMLINK_WEBHOOK_CONFIG' docs/operations/webhook-fan-out-recipe.md
grep -q 'webhook add' docs/operations/webhook-fan-out-recipe.md
grep -q 'webhook list' docs/operations/webhook-fan-out-recipe.md
grep -q 'webhook test' docs/operations/webhook-fan-out-recipe.md
grep -q 'X-Termlink-Signature' docs/operations/webhook-fan-out-recipe.md
grep -q 'allowed-host' docs/operations/webhook-fan-out-recipe.md
grep -q 'webhook_dead_letter_total' docs/operations/webhook-fan-out-recipe.md
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

### 2026-07-03T16:17:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2337-webhook-fan-out-s6--operator-recipe-doc.md
- **Context:** Initial task creation
