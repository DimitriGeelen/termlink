---
id: T-2336
name: "Webhook fan-out S5 — CLI config verbs (webhook add/list/test)"
description: >
  Follow-up to T-2335 (decomposed from the original S4 capture). CLI config authoring for the arc-004 webhook fan-out feature: 'termlink webhook add' (append/merge a target into the TERMLINK_WEBHOOK_CONFIG JSON file), 'termlink webhook list' (render configured targets + allowed_hosts), 'termlink webhook test <url>' (dispatch a signed sample payload to a target and report HTTP status, reusing webhook::dispatch/sign_payload). Config surface today is TERMLINK_WEBHOOK_CONFIG JSON only. Telemetry already shipped (T-2335 governor_status fields). See crates/termlink-hub/src/webhook.rs + docs/reports/T-2331-webhooks-external-fan-out-inception.md.

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
created: 2026-07-03T14:09:29Z
last_update: 2026-07-03T16:01:48Z
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

# T-2336: Webhook fan-out S5 — CLI config verbs (webhook add/list/test)

## Context

Final slice of the arc-004 webhook fan-out feature (Candidate B, human-GO'd on T-2331).
Slices S1–S4 shipped the runtime: primitive (T-2333), event-wiring (T-2333), retry/backoff/
dead-letter (T-2334), governor_status telemetry (T-2335). The one authoring gap left: today
the ONLY way to configure targets is to hand-write the `TERMLINK_WEBHOOK_CONFIG` JSON file.
S5 adds operator-facing CLI verbs so a target can be added, listed, and smoke-tested without
editing JSON by hand. Reuses `termlink_hub::webhook::{WebhookConfig, WebhookTarget, dispatch,
sign_payload}` (CLI already depends on `termlink-hub`). New `Command::Webhook { action }`
mirroring the `Token` nested-subcommand pattern; handlers in `commands/webhook.rs`.
See `crates/termlink-hub/src/webhook.rs` + `docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `termlink webhook add --url <U> [--signing-key K] [--topic T ...] [--allowed-host H ...] [--config PATH]` merges a target into the config JSON: reads existing config (or starts empty), appends the target, auto-adds the URL's host to `allowed_hosts` when absent, writes back atomically (temp+rename). Refuses (non-zero exit + clear message) when the URL has no parseable host or is not http/https. — smoke: added 2 targets, host auto-added once (no dup), key auto-generated; invalid-url rejected by unit test.
- [x] `termlink webhook list [--config PATH] [--json]` renders configured targets (url, topics, host) + allowed_hosts with the `signing_key` REDACTED in human output; empty/missing config prints an explicit "webhook fan-out disabled (0 targets)" line (not a silent blank). `--json` emits the parsed config verbatim. — smoke: text shows `signing_key=<redacted, 64 chars>`, `--json` shows key verbatim.
- [x] `termlink webhook test --url <U> [--signing-key K] [--allowed-host H ...] [--topic T] [--config PATH] [--json]` dispatches a signed sample payload to the target by reusing `webhook::dispatch` (so the SSRF host-allowlist guard + HMAC signing run identically to production) and reports the HTTP status code or the classified error; SSRF refusal (host not in allowlist) is surfaced loudly, not swallowed. — smoke: local sink received `X-Termlink-Signature: sha256=…` + HTTP 204; non-allowlisted host refused loudly (exit 1) with `--allowed-host` hint.
- [x] Config path resolves from `--config` when given, else `TERMLINK_WEBHOOK_CONFIG` env; when neither is set, `add`/`test` fail with an actionable message naming both mechanisms (no silent default path). — `resolve_config_path` unit-tested; bail message names both.
- [x] Pure config-mutation logic (merge target, ensure-host-in-allowlist, redact signing key) is extracted into unit-tested helper fn(s) in `commands/webhook.rs` — tests cover: add-to-empty, add-with-host-already-present (no dup), invalid-url-rejected, redaction. — 8 unit tests pass.
- [x] `cargo build -p termlink-hub -p termlink` compiles clean; new `webhook.rs` unit tests pass; `webhook` subcommand appears in `termlink --help` and `termlink webhook --help` lists add/list/test. — build clean; 8 CLI tests + full 397+4 hub suite pass (PL-238); help verified.

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
cargo build -p termlink-hub -p termlink 2>&1 | tail -3
cargo test -p termlink --bin termlink commands::webhook 2>&1 | tail -8
grep -q 'Command::Webhook' crates/termlink-cli/src/main.rs
grep -q 'pub(crate) enum WebhookAction' crates/termlink-cli/src/cli.rs

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

### 2026-07-03 — `webhook test` mirrors production deny-by-default (does NOT auto-permit the URL host)
- **Chose:** `test` runs the URL against the config's allowlist + explicit `--allowed-host` extras only; it does NOT auto-add the URL's own host. A host absent from the allowlist refuses loudly with an actionable `--allowed-host` hint.
- **Why:** The whole point of `test` is to verify a target behaves as it WOULD in production. Auto-permitting the tested host would silently bypass the SSRF guard the feature exists to enforce — the operator would get a green "✓ dispatched" for a target the hub would actually refuse. Deny-by-default consistency > convenience.
- **Rejected:** Auto-adding the URL host to the effective allowlist (first implementation). Made the SSRF-refusal path unreachable via `test`, defeating AC-3's "surfaced loudly" clause.

### 2026-07-03 — Signing-key generation via `/dev/urandom`, not a new `rand` dependency
- **Chose:** Read 32 bytes from `/dev/urandom` directly for auto-generated signing keys.
- **Why:** The CLI crate has no direct `rand`/`getrandom` dependency; `/dev/urandom` is cryptographically adequate on Linux (the deploy target) and dependency-free. Avoids widening the CLI dependency graph for one helper.
- **Rejected:** Adding `rand`/`getrandom` as a direct dep (unnecessary surface); time-based pseudo-secret (not unpredictable — unsafe for an HMAC key).

### 2026-07-03 — Expose `url_host` + `build_test_client` from the hub crate rather than re-implement in CLI
- **Chose:** Made `webhook::url_host` public and added `webhook::build_test_client()` in the hub crate; the CLI reuses both plus the existing `dispatch`/`sign_payload`.
- **Why:** Single source of truth. Host extraction and the aws-lc-rs crypto-provider install (PL-238) must match the hub's dispatch path exactly, or the CLI's `test` could pass while production refuses (or vice-versa). Re-implementing the reqwest::Url parse + provider-pin dance in the CLI would invite drift and re-introduce the PL-238 "No provider set" trap.
- **Rejected:** Adding `reqwest` as a direct CLI dependency to parse URLs client-side (drift risk + the crypto-provider pin would have to be duplicated).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-03T14:09:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2336-webhook-fan-out-s5--cli-config-verbs-web.md
- **Context:** Initial task creation

### 2026-07-03T16:01:48Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
