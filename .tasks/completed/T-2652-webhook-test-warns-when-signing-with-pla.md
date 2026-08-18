---
id: T-2652
name: "webhook test warns when signing with placeholder key (silent test-key fallback
  masks signature the consumer rejects)"
description: >
  webhook.rs cmd_webhook_test falls back to signing_key test-key when neither --signing-key
  nor a matching config target is present, then prints success with no indication
  the signature is a placeholder the real consumer will reject. Add an actionable
  warning + JSON field. Class B non-actionable-success (Directive 3).

status: work-completed
workflow_type: build
owner: agent
horizon:
tags: []
components: [crates/termlink-cli/src/commands/webhook.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-12T19:18:36Z
last_update: '2026-08-18T18:59:14Z'
date_finished: 2026-08-12T19:22:19Z
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:55Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 2
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=2 
      (body:telemetry-or-audit-entry); D3=2 (body:default-change); D4=2 
      (body:env-class-handled); F-RECALL=0 (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:14Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 8
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=8 
      (no-signal)
    rubric_sha: missing
---

# T-2652: webhook test warns when signing with placeholder key (silent test-key fallback masks signature the consumer rejects)

## Context

Round-10 Usability-lens finding (Directive #3, Class B non-actionable success). `cmd_webhook_test`
(webhook.rs) resolves the signing key as `--signing-key` → matching config target → `"test-key"`
placeholder (`unwrap_or_else`, line ~277-280). When neither a flag nor a config match is present,
it signs with the literal `"test-key"` and then prints `✓ dispatched / signature: HMAC-SHA256 sent`
with NO indication the signature is a placeholder. An operator smoke-testing a real endpoint
(`termlink webhook test --url https://real-consumer/hook`, no key) sees `http_status: 200` and
believes the webhook is correctly signed — but the real consumer rejected (or will reject) the
signature because it was signed with `"test-key"`, not the shared secret. The failure is invisible
in the `test` output.

Fix: surface the placeholder fallback. Add a pure decision helper returning an actionable warning
when the key is the placeholder, emit it to stderr (visible in both text and `--json` modes), and
add a `placeholder_signing_key` boolean to the JSON success envelope so automation can detect it.
Pure additive — the dispatch behavior is unchanged.

## Acceptance Criteria

### Agent
- [x] A pure helper `placeholder_key_warning(explicit_key: bool, matched_config: bool) -> Option<String>` returns `Some(actionable msg naming --signing-key)` only when both are false, `None` otherwise
- [x] `cmd_webhook_test` emits the warning to stderr when the placeholder key is used (both text and `--json` modes)
- [x] The `--json` success envelope carries a `placeholder_signing_key` boolean
- [x] Unit test proves the helper returns `Some` (naming `--signing-key`) for the placeholder case and `None` when a key/config is present (load-bearing: reverting the Err/placeholder arm to `None` fails it)
- [x] `cargo build -p termlink` + `cargo test -p termlink --bins webhook` pass

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

cd /opt/termlink && cargo build -p termlink 2>&1 | tail -3
cd /opt/termlink && cargo test -p termlink --bins webhook 2>&1 | tail -5
grep -q 'fn placeholder_key_warning' crates/termlink-cli/src/commands/webhook.rs
grep -q 'placeholder_signing_key' crates/termlink-cli/src/commands/webhook.rs

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

**Symptom:** `termlink webhook test --url https://real-consumer/hook` (no `--signing-key`, no
matching config target) prints `✓ dispatched / http_status: 200 / signature: HMAC-SHA256 sent`
and exits 0, so the operator believes the endpoint is correctly signed — but the payload was
signed with the literal placeholder `"test-key"`, which the consumer rejects on signature check.
The wrong signature is invisible in the test output.

**Root cause:** the key resolution ended in `.unwrap_or_else(|| "test-key".to_string())`, a
silent placeholder fallback, and neither the text output nor the JSON envelope reflected that the
key was a placeholder rather than a real secret. The `test` verb's whole purpose — proving the
signing round-trip — was undercut by hiding the one fact that determines whether signing is real.

**Why structurally allowed:** the placeholder is a legitimate convenience (smoke-testing an
ad-hoc URL with an explicit key), but the fallback path had no observable signal distinguishing
"signed with your real key" from "signed with a throwaway." No test asserted the distinction.

**Prevention:** a pure `placeholder_key_warning(explicit_key, matched_config) -> Option<String>`
now decides the warning; it fires to stderr (both modes) and sets `placeholder_signing_key` in the
JSON envelope. A load-bearing unit test pins that the fallback case warns (naming `--signing-key`)
and the real-key cases do not, so re-swallowing the signal trips the test.

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

### 2026-08-12T19:18:36Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2652-webhook-test-warns-when-signing-with-pla.md
- **Context:** Initial task creation

### 2026-08-12T19:18:50Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-5f6eb8c4
- **Timestamp:** 2026-08-12T19:23:24Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-12T19:22:19Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
