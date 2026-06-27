---
id: T-2295
name: "V3b: delivery-confirm by default + canary"
description: >
  RC3b fix. Flip T-2286 --await-ack to the DEFAULT for /agent-handoff, /reply, agent-send.sh; recipient sidecar (V3a) auto-acks, advancing the channel.receipts frontier. Add an unconfirmed-delivery canary: local sent-but-unconfirmed mirror made observable (kills the write-only-sink class, e.g. framework:pickup 36-sent/0-recv per G-063). ACs: default send confirms or fails LOUD (never silent 'sent'); recipient auto-ack wired; canary surfaces unconfirmed sends; /check-arc read emits a receipt.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: [arc:reliable-comms]
components: []
related_tasks: [T-2291, T-2294]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-27T17:06:39Z
last_update: 2026-06-27T20:33:27Z
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

# T-2295: V3b: delivery-confirm by default + canary

## Context

Arc-003 (reliable-comms) slice. RC3b from the T-2291 inception RCA: "sent" ≠ "delivered" — sends record success on TCP-ack with no receipt. T-2286 already shipped `--await-ack`; this task flips it to the default and makes unconfirmed delivery observable (the write-only-sink class, e.g. framework:pickup at 36-sent/0-recv, G-063). Depends on T-2294 (recipient sidecar provides the auto-ack). Design trail: `docs/reports/T-2291-cross-agent-comms-inception.md`.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] `--await-ack` is the DEFAULT for `/agent-handoff`, `/reply`, and `agent-send.sh` (opt-out flag retained)
- [ ] A default send confirms delivery or fails LOUD — never silently records "sent"
- [ ] The recipient sidecar (V3a) auto-acks, advancing the `channel.receipts` frontier
- [ ] An unconfirmed-delivery canary surfaces local sent-but-unconfirmed messages (kills the write-only-sink class)
- [ ] A `/check-arc` read emits a receipt

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

### 2026-06-27 — AC3 (sidecar auto-ack) collides with V3a's wake flag — receipt must be on READ, not on detect
- **What changed:** Implementing AC3 naively ("the recipient sidecar auto-acks,
  advancing channel.receipts") would BREAK V3a. Proven empirically: V3a's
  `notify-sidecar.sh` detects mail via `channel unread --sender <self>`, and
  `channel ack` advances that SAME `<self>` receipt frontier. Live test on a real
  DM: recipient unread `2 → 0` immediately after the recipient acks. So a sidecar
  that auto-acks on *detection* erases the very flag it just raised — the agent
  would never be woken. AC3 as literally worded is self-defeating against V3a.
- **Plan impact (the resolution for next session):** Emit the receipt on **READ**,
  not on detect. The sidecar raises the flag (V3a) and does NOT ack; the ack is
  emitted when the agent actually reads (the `/check-arc respond` path / AC5's
  receipt-on-read). This collapses AC3 into AC5: a single "consumed" receipt
  serves the sender's `--await-ack` confirmation. The "delivered-but-not-yet-read"
  intermediate level (sidecar-journaled receipt) is the inception's **3-level
  confirm ladder**, which is explicitly **V6 (T-2296) scope** (build-tasks §V6,
  inception report §confirm-ladder) — do NOT build a second receipt namespace in
  V3b. Net: re-scope AC3 → "recipient acks on read (not silent), advancing
  channel.receipts" and implement it together with AC5.
- **Triggered:** Re-scope AC3 (above) before implementing; the delivered-vs-read
  two-level receipt is deferred to V6's ladder, not V3b.

### 2026-06-27 — V3b build order + budget-bounded handoff
- **What changed:** Surface map (Explore) shows V3b spans 5 surfaces:
  `agent-send.sh` (swap manual receipt-poll loop L213-246 for `--await-ack
  --retry`, add `--no-await-ack` opt-out at L72-88), `agent contact` + `agent-
  respond.sh` (the /agent-handoff and /reply transports — AC1 must cover these too,
  not just agent-send.sh), `notify-sidecar.sh` (ack-on-read per the decision above),
  `check-arc.md` (scoped receipt-on-read exception vs its hard "NEVER auto-ack"
  rule L287-288), and a NEW `check-unconfirmed-delivery-freshness.sh` canary
  (reads `termlink channel awaiting-ack --json`; tracker = `~/.termlink/
  awaiting_ack.sqlite`; MUST install crontab to /etc/cron.d/termlink-...-canary or
  the T-1722 pre-push audit lint FAILS — audit.sh L1392-1447).
- **Plan impact:** This session reached the budget ceiling for arc-003 with V3a
  shipped + V3b design-unblocked. V3b not started in code (the AC3 tension needed
  resolving first; agent-send.sh is load-bearing and needs careful live testing).
- **Triggered:** Next-session build order (smallest→largest, all independent
  except AC3+AC5 which share the receipt policy): (1) canary [additive, standalone
  G-063 value], (2) agent-send.sh await-ack default + opt-out, (3) agent contact /
  agent-respond.sh await-ack default, (4) sidecar+check-arc ack-on-read (AC3+AC5
  together per decision above).

## Decisions

<!-- Record decisions ONLY when choosing between alternatives. -->

### 2026-06-27 — receipt emitted on READ, not on sidecar detection
- **Chose:** the recipient ack (the delivery confirmation the sender's
  `--await-ack` waits for) is emitted when the agent READS the mail, not when the
  V3a sidecar detects it.
- **Why:** `channel ack` advances the `<self>` receipt frontier that V3a's
  `channel unread --sender <self>` reads; acking on detection zeros V3a's own wake
  flag (proven live: unread 2→0). One "consumed" frontier serves both the wake
  signal (before read) and the sender confirmation (after read).
- **Rejected:** (a) sidecar auto-ack on detection — breaks V3a's wake. (b) a
  second "delivered" receipt namespace distinct from "read" — that is V6's 3-level
  confirm ladder, out of scope for V3b (would duplicate V6 and add protocol
  surface here).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-27T17:06:39Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2295-v3b-delivery-confirm-by-default--canary.md
- **Context:** Initial task creation

### 2026-06-27T20:33:27Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
