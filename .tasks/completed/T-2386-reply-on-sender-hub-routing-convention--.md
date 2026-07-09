---
id: T-2386
name: "Reply-on-sender-hub routing convention — /reply + agent contact target recipient home hub derived from presence (comms loud-contract; fixes hub-split silent no-delivery, T-2380 C1/E1/G-060)"
description: >
  Sender and reader silently target different hubs for the same-named dm topic (no federation). Resolve the recipients home hub from agent-presence and route the contact/reply there; refuse or auto-route on mismatch. Attacks E1 root without full federation (C3 out of scope).

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/commands/agent.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-09T09:29:17Z
last_update: 2026-07-09T23:40:05Z
date_finished: 2026-07-09T23:40:05Z
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

# T-2386: Reply-on-sender-hub routing convention — /reply + agent contact target recipient home hub derived from presence (comms loud-contract; fixes hub-split silent no-delivery, T-2380 C1/E1/G-060)

## Context

T-2380 GO, loud-delivery-contract link #3 (the hub-split silent no-delivery,
E1/G-060). A reply/contact today posts to whatever hub the tooling defaults to; if
that is not the hub the recipient reads, the message lands durably on the wrong
history and is never seen (there is NO inter-hub federation — G-060). The recipient's
**home hub is already discoverable** from their agent-presence heartbeat:
`PresenceMatch.observed_addr` (hub-attested TCP source, T-2297 — cannot be forged) or
`PresenceMatch.addr` (self-reported `metadata.addr`, T-2293). `resolve_contact_via_fleet`
already picks the hub it *read the heartbeat from* into `fleet_hub` (agent.rs ~1080)
— but the skills (`/reply`, `/agent-handoff`) and the `--target-fp` path do not
consistently route to the recipient's home hub. This task makes "route to the
recipient's home hub, derived from presence" the default convention across the
contact/reply surface, so a reply cannot silently land on a hub the peer doesn't read.

**Scope note:** convention + routing default, NOT federation (C3, explicitly out of
scope per T-2380). Depends on T-2384 (per-agent fp) + builds on T-2385's
`fetch_recipient_presence` (the presence read is already wired). Slice if needed
(agent contact routing → skill wrappers).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `agent contact` derives the recipient's home hub from presence (self-reported `metadata.addr`, then the hub the heartbeat was read from — `observed_addr` EXCLUDED from routing, ephemeral source port; see Evolution) and routes the dm post there by default when the operator did not pass an explicit `--hub`; explicit `--hub` still wins (smoke: `--hub 192.168.10.107:9100` → `routed_hub: null`). Pure helper `resolve_home_hub(presence) -> Option<String>` + `resolve_home_hub_precedence` unit test. Live smoke 2026-07-10: heartbeat declaring `addr=192.168.10.122:9100` → dry-run `routed_hub: 192.168.10.122:9100`.
- [x] When the derived home hub differs from the hub the sender would otherwise have used, the `--json` output carries a `routed_hub` field (dry-run preview + live NDJSON annotation line, null when not derived) and human mode prints `routing to recipient's home hub <addr> (derived from presence — pass --hub to override)` — T-2385 surfacing pattern extended.
- [x] The `/reply` and `/agent-handoff` skill wrappers inherit the home-hub routing: `/agent-handoff` delegates to `termlink agent contact` with no `--hub` (inherits automatically); `/reply`/agent-respond.sh replies on the hub the inbound message arrived on, which under this convention IS the thread's home hub (sender routed it there and polls acks there) — see Evolution entry.
- [x] `--target-fp`-only path also home-hub-routes via new fp-keyed `resolve_contact_fp_via_fleet` (sender_id-matched, LIVE-only, freshest-first); presence-absent → today's behavior + loud stderr note (smoke: `--target-fp deadbeefdeadbeef` printed the degradation note and kept local default; `--target-fp d1993c2c3ec44c94` with LIVE declared-addr presence → `routed_hub: 192.168.10.122:9100`).
- [x] `cargo build --release -p termlink` succeeds (0.11.437, exit 0, 2026-07-10); the `resolve_home_hub` unit test passes (`1 passed`); no regression in the contact test module (`25 passed; 0 failed` incl. the new test).

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

cargo check -p termlink
# resolve_home_hub unit test passes (bin tests need --bin termlink, T-2384 lesson)
out=$(cargo test -p termlink --bin termlink resolve_home_hub 2>&1); echo "$out" | grep -q "1 passed"
# no regression in the contact test module
out=$(cargo test -p termlink --bin termlink contact_tests 2>&1); echo "$out" | grep -q "0 failed"
# routed_hub surfaced in the code (json annotation + dry-run preview + human note)
grep -q 'routed_hub' crates/termlink-cli/src/commands/agent.rs
# observed_addr is documented as excluded from routing
grep -q 'EXCLUDED' crates/termlink-cli/src/commands/agent.rs

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

### 2026-07-10 — observed_addr is NOT a routable hub address (AC precedence corrected)

- **What changed:** The filed AC said "prefer `observed_addr`, fall back to `addr`".
  Reading the T-2297 hub code (`apply_observed_addr`, channel.rs:512) shows
  `observed_addr` is the hub-attested TCP **source** of the heartbeat — the agent
  host's IP plus an **ephemeral port** (test fixture: `192.168.10.141:51234`).
  Routing a DM there would target a dead ephemeral port. It is authoritative for
  identity/host attestation, unpostable as a hub address.
- **Plan impact:** Precedence corrected to: self-reported `metadata.addr`
  (declared home hub, T-2293) → hub the heartbeat was read from (safe because
  agent-presence does not federate, G-060 — the source hub IS the peer's local
  hub). `observed_addr` deliberately excluded from routing; the unit test locks
  the exclusion in.
- **Triggered:** No new task — the `resolve_home_hub` doc-comment + test encode
  the correction. Also scoped down the Ok(reg) local-peer branch: adopt only a
  *declared* addr there (never the walk's read-hub), so default local-UDS
  routing/auth is not disturbed for same-host sends.

### 2026-07-10 — skill wrappers comply by convention, not new code

- **What changed:** `/agent-handoff` invokes `termlink agent contact` with no
  `--hub` → inherits home-hub routing automatically once the new binary is
  installed. `/reply` (agent-respond.sh) posts receipt+turn to the topic on the
  hub the inbound message arrived on — under this convention that IS the
  thread's home hub (the sender routed it there and polls acks there), so
  reply-on-arrival-hub is the correct symmetric behavior; `--hub` remains for
  cross-host overrides.
- **Plan impact:** AC 3 satisfied by delegation + convention; no skill edits.
- **Triggered:** none.

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

### 2026-07-09T09:29:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2386-reply-on-sender-hub-routing-convention--.md
- **Context:** Initial task creation

### 2026-07-09T12:07:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

## Reviewer Verdict (v1.5)

- **Scan ID:** R-3f5b3a19
- **Timestamp:** 2026-07-09T23:40:51Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-07-09T23:40:05Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
