---
id: T-2333
name: "Webhook fan-out S2 — wire hub events → signed dispatch (arc-004, follows T-2332)"
description: >
  Slice 2 of the T-2331 GO webhook feature. Slice 1 (T-2332) shipped the SEND PRIMITIVE (sign_payload + host_allowed + dispatch + WebhookConfig in crates/termlink-hub/src/webhook.rs, all unit-tested). Slice 2 wires it to real hub events: subscribe each WebhookTarget to topics/event-kinds, load WebhookConfig at hub startup from the hub config surface, and fan out a signed POST when a matching event is appended. Build on the existing channel-post path (crates/termlink-hub/src/channel.rs). Keep opt-in: zero targets = no dispatch. Then S3 = retry/backoff/dead-letter (reuse T-2051 queue pattern); S4 = CLII config verbs (webhook add/list/test) + governor_status counters. See docs/reports/T-2331-webhooks-external-fan-out-inception.md.

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
created: 2026-07-03T09:59:52Z
last_update: 2026-07-03T13:12:39Z
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

# T-2333: Webhook fan-out S2 — wire hub events → signed dispatch (arc-004, follows T-2332)

## Context

Slice 2 of the T-2331 GO webhook feature. Slice 1 (T-2332) shipped the SEND
primitive (`sign_payload` + `host_allowed` + `dispatch` + `WebhookConfig`) in
`crates/termlink-hub/src/webhook.rs`. This slice wires it to real hub events:
a per-target topic filter, a process-global runtime loaded at hub startup, and
a fire-and-forget fan-out invoked from the `channel.post` `Ok(offset)` arm — the
same sibling-emit placement the inbox (T-1637) and dm-rail (T-2323) wakers use.
Opt-in preserved: no `TERMLINK_WEBHOOK_CONFIG` env ⇒ disabled ⇒ zero behaviour
change. See `docs/reports/T-2331-webhooks-external-fan-out-inception.md`.

## Acceptance Criteria

### Agent
- [x] `WebhookTarget` gains a `topics: Vec<String>` filter and `WebhookConfig::targets_for(topic)` returns only targets whose `topics` contains the exact topic OR `"*"` (empty `topics` ⇒ never fires — opt-in). Unit-tested (`matches_topic_exact_wildcard_and_empty`, `targets_for_selects_matching_only`).
- [x] Process-global runtime (`OnceLock`) holding the parsed `WebhookConfig` + a bounded-timeout `reqwest::Client`, installed by `pub fn init()` that reads `TERMLINK_WEBHOOK_CONFIG` (path to JSON); absent / unreadable / parse-fail ⇒ disabled with NO panic (opt-in, no hard dependency). Unit-tested (`webhooks_none_when_uninitialised_in_this_test_binary`).
- [x] `pub fn fan_out(topic, body)` spawns one signed `dispatch` per matching target fire-and-forget (returns immediately, never blocks the post response); selection helper unit-tested (non-matching topic ⇒ 0 targets, matching ⇒ correct set) without touching the network.
- [x] `channel.rs` `Ok(offset)` arm calls `crate::webhook::fan_out(...)` as a new sibling block after the dm-rail emit and before `Response::success` — a failed post never fans out.
- [x] `server.rs` calls `crate::webhook::init()` in BOTH startup paths (`run_with_tcp` + `run_blocking`) alongside the dedupe/cv_index init.
- [x] `cargo build -p termlink-hub` and `cargo test -p termlink-hub webhook` both pass (11 webhook tests: Slice-1's 8 + 3 new). Full workspace green: 394 hub + 945 CLI/session, zero regressions (the ring/aws-lc-rs crypto-provider fix restored the 4 tls:: tests).

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

grep -q "pub fn init" crates/termlink-hub/src/webhook.rs
grep -q "pub fn fan_out" crates/termlink-hub/src/webhook.rs
grep -q "fn targets_for" crates/termlink-hub/src/webhook.rs
grep -q "crate::webhook::fan_out" crates/termlink-hub/src/channel.rs
grep -q "crate::webhook::init" crates/termlink-hub/src/server.rs
cargo build -p termlink-hub
cargo test -p termlink-hub webhook

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

### 2026-07-03 — reqwest crypto-provider pin (ring vs aws-lc-rs)
- **Chose:** reqwest feature `rustls-tls-webpki-roots-no-provider` instead of the plain `rustls-tls`.
- **Why:** the hub's TLS stack deliberately uses rustls's aws-lc-rs backend (Cargo.toml line 54). Plain `rustls-tls` transitively enables `__rustls-ring`, so BOTH aws-lc-rs and ring provider features end up compiled into the shared rustls dep. rustls 0.23 then refuses to auto-select a process-default provider and panics ("make sure exactly one of the 'aws-lc-rs' and 'ring' features is enabled") whenever a TLS config is built without an explicit provider — this broke the 4 `tls::tests`. The regression was latent since Slice-1 (T-2332) because that slice only ran `cargo test -p termlink-hub webhook`, never the full suite. The `-no-provider` variant keeps the bundled Mozilla webpki roots (portable HTTPS validation, Directive 4 — no OS trust store) WITHOUT enabling ring, leaving aws-lc-rs the sole provider so auto-default works again.
- **Rejected:** (a) calling `CryptoProvider::install_default()` at hub startup — doesn't help the unit tests, which build TLS configs directly outside the startup path; (b) `native-roots` — needs the OS trust store, less portable than bundled webpki roots.

### 2026-07-03 — topic filter matching semantics
- **Chose:** exact topic membership OR the `"*"` wildcard; empty `topics` list never fires.
- **Why:** deny-by-default symmetry with the Slice-1 host allowlist — a target only fires for topics it explicitly subscribes to. No prefix/substring matching keeps the rule un-surprising and audit-clear.
- **Rejected:** prefix matching (`work-*`) — deferred; adds glob-parse surface with no demand yet.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-03T09:59:52Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2333-webhook-fan-out-s2--wire-hub-events--sig.md
- **Context:** Initial task creation

### 2026-07-03T13:12:39Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
