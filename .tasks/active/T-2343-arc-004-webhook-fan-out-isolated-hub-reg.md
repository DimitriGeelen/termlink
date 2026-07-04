---
id: T-2343
name: "arc-004 webhook fan-out isolated-hub regression demo"
description: >
  arc-004 webhook fan-out isolated-hub regression demo

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
created: 2026-07-04T07:41:15Z
last_update: 2026-07-04T07:41:15Z
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

# T-2343: arc-004 webhook fan-out isolated-hub regression demo

## Context

arc-004 Candidate B (webhook fan-out, S1–S6 T-2332…T-2337) is feature-complete and
security-critical: outbound HTTP from the hub guarded by a deny-by-default
exact-host SSRF allowlist + HMAC-SHA256 payload signing (`X-Termlink-Signature:
sha256=<hmac>`). It shipped with unit tests and a **one-time manual smoke** (T-2336)
— but, like the dm rail before T-2342, **no reusable isolated-hub reproducer**. A
silent regression in the SSRF guard or the HMAC signing is a *security* defect, so
this is the higher-stakes sibling of T-2342. This task adds
`scripts/demo-webhook-fanout.sh`: an isolated hub started with a
`TERMLINK_WEBHOOK_CONFIG` pointing at a local sink, proving the full fan-out path
end-to-end. Contract ground: `docs/operations/webhook-fan-out-recipe.md`,
`crates/termlink-hub/src/webhook.rs`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/demo-webhook-fanout.sh` exists, is executable, `bash -n` clean, and
  is isolation-safe: runs under a temp `TERMLINK_RUNTIME_DIR` + temp `HOME` + a
  loopback sink, never touches the shared `:9100` hub or `~/.termlink`, and tears
  down the hub + sink on exit. Skips cleanly (documented exit code) if `python3`
  (the sink) is unavailable.
  *(chmod +x + `bash -n` clean; `cleanup()` trap kills sink + hub and rm's both
  temp dirs; python3-missing → exit 4 SKIP; stale pre-webhook binary → exit 2 with
  loud "predates arc-004 Candidate B" message.)*
- [x] POSITIVE fan-out proven E2E: an isolated hub started with a
  `TERMLINK_WEBHOOK_CONFIG` target (allowlisted host, topic filter) fans a
  `channel.post` on the matching topic out to the local sink as a real signed POST;
  the demo recomputes HMAC-SHA256 over the RAW received body with the configured
  `signing_key` and asserts it equals the `X-Termlink-Signature: sha256=<hex>`
  header (exercises T-2332 sign_payload + T-2333 fan_out wiring, not a stub).
  *(Canonical run 2026-07-04 vs fresh release binary: `loaded=true`; `sink POSTs
  0 -> 1`; `HMAC verified: yes` — recompute keyed on raw UTF-8 key bytes matching
  webhook.rs sign_payload `new_from_slice(signing_key.as_bytes())`.)*
- [x] TOPIC-FILTER negative proven: a `channel.post` to a NON-matching topic does
  NOT deliver to the sink (the `topics` filter gates fan-out).
  *(`non-matching 'webhook-nomatch-<pid>' -> no new delivery` — sink count
  unchanged after 3s.)*
- [x] SSRF deny-by-default proven E2E: `termlink webhook test` against a
  NON-allowlisted host (e.g. the `169.254.169.254` cloud-metadata address) is
  refused LOUDLY (non-zero exit, host-not-allowlisted message) with no delivery to
  the sink — the production `webhook::dispatch` guard, at the CLI surface (PL-239).
  *(`rc=1  Error: webhook test failed: host not allowlisted (SSRF guard):
  http://169.254.169.254/latest/meta-data/`; sink count unchanged.)*
- [x] The demo runs green end-to-end (exit 0) on the current tree; the PASS
  transcript + any defect RCA is recorded in `docs/reports/T-2343-arc-004-webhook-fanout-demo.md`.
  *(exit 0 on both target/debug and the rebuilt target/release. Report includes
  the stale-release-binary RCA (first run failed 0->0 because target/release
  predated the webhook slices — added the binary-guard + rebuilt) and the
  `.governor.webhook_enabled` jq-path display fix.)*

## Verification

# Shell commands that MUST pass before work-completed. One per line.
test -x scripts/demo-webhook-fanout.sh
bash -n scripts/demo-webhook-fanout.sh
test -f docs/reports/T-2343-arc-004-webhook-fanout-demo.md

## RCA

This task is a regression-coverage add, not a bug fix — the RCA frames the
coverage gap it closes (title matches "regression", so the gate requires this).

**Symptom:** the arc-004 webhook fan-out (SSRF guard + HMAC signing + topic filter)
had no automated reproducer. Its only integration evidence (T-2336) was a one-time
manual smoke; nothing re-provable protects a security-critical path from silent
regression.

**Root cause:** the webhook slices shipped unit tests (sign_payload vector,
suffix-attack deny, metadata-SSRF refusal) plus a manual `webhook test` smoke, but
no end-to-end script that drives a real hub → real signed POST → real sink and
verifies the HMAC over the wire. The manual smoke "proved it once" and the loop
closed without a reusable artifact — the same gap T-2342 closed for the dm rail.

**Why structurally allowed:** unit tests cover the pure crypto/allowlist helpers
but never the full `channel.post → fan_out → dispatch → HTTP` chain against a live
consumer; a one-time green smoke reads as "covered." PL-240 already established
that E2E demos catch integration gaps unit tests miss.

**Prevention:** `scripts/demo-webhook-fanout.sh` — an isolated-hub reproducer that
fails if the fan-out stops firing, the HMAC drifts, the topic filter leaks, or the
SSRF guard regresses, instead of the defect shipping silently.

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

### 2026-07-04T07:41:15Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2343-arc-004-webhook-fan-out-isolated-hub-reg.md
- **Context:** Initial task creation
