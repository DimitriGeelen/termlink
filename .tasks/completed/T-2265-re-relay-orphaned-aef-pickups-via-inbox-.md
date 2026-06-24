---
id: T-2265
name: "Re-relay orphaned AEF pickups via inbox + record framework-pickup inbound-channel-dead (G-063 root cause)"
description: >
  Re-relay orphaned AEF pickups via inbox + record framework-pickup inbound-channel-dead (G-063 root cause)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T23:19:28Z
last_update: 2026-06-23T23:24:51Z
date_finished: 2026-06-23T23:24:51Z
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

# T-2265: Re-relay orphaned AEF pickups via inbox + record framework-pickup inbound-channel-dead (G-063 root cause)

## Context

Follow-up to the P-047 relay (T-2428 created at AEF). While chasing P-047 we
proved that `fw pickup send` → `framework:pickup` hub topic **never reaches AEF**:
AEF has no inbound consumer of that topic — it ingests pickups only from its own
local `.context/pickup/inbox/` via an every-minute `fw pickup process` cron. The
topic post is an outbound mirror nothing reads. This is the verified root cause of
the long-standing **G-063 "framework:pickup zero receipts"** symptom.

Consequence: termlink's four recent upstream pickups — **P-045** (PL-222
inception-close vendored-path crash), **P-048** (no-federation FRAMEWORK.md
correction, T-2259), **P-049** (Watchtower /review+/approvals 500, T-2262/PL-227),
**P-050** (pickup observer self-echo, G-046) — were all confirmed ABSENT from AEF
(no inbox/processed file, no task). They are re-delivered the proven way: drop the
envelope into AEF's local inbox via `termlink_run` (T-559 blocks direct
cross-project paths), let AEF's cron auto-create tasks. Also file a meta-pickup to
AEF that its inbound channel is broken so the class is fixed, not just instances.
See memory `feedback_relay_to_aef_via_pickup` (corrected this session).

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] Confirmed P-045/P-048/P-049/P-050 absent from AEF inbox/processed/rejected and no covering AEF task (done in T-2265 investigation)
- [x] Re-relayed the 4 envelopes into AEF's local `.context/pickup/inbox/` via termlink_run, distinct termlink-namespaced filenames (no pickup_id collision)
- [x] AEF ingestion auto-created one task per relayed pickup under AEF governance — T-2431 (P-045), T-2433 (P-048), T-2435 (P-049), T-2437 (P-050); CLAUDECODE unset → no T-679 gate, no recommendation injected
- [x] Filed meta-pickup to AEF: `framework:pickup` has no inbound consumer (G-063 root cause); requests a topic→inbox bridge → AEF task T-2430
- [x] Registered termlink learning PL-228 recording the framework:pickup inbound-channel-dead finding + the verified relay path

<!-- ### Human (removed — all criteria agent-verifiable) -->
<!-- _unused Human template:
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

# T-2265: the relay learning must be registered (distinctive phrase grep, single-file, no pipe)
grep -q "AEF consumes its local inbox" .context/project/learnings.yaml

## RCA

**Symptom:** Termlink upstream pickups to AEF (`fw pickup send`) showed "zero
receipts" on `framework:pickup` for weeks (G-063) — AEF never acked or acted.
**Root cause:** AEF has **no inbound consumer** of the `framework:pickup` hub
topic. Its pickup pipeline ingests ONLY from its own local
`.context/pickup/inbox/` directory (every-minute `fw pickup process` cron). The
`lib/pickup-channel-bridge.sh` post-to-topic is an outbound mirror that nothing on
the AEF side reads. So topic-bridged pickups were structurally undeliverable.
**Why structurally allowed:** the relay design assumed symmetric topic
consumption (both projects post AND read the topic), but only the OUTBOUND half
was ever wired. `fw pickup send` reports success on local-envelope-creation +
topic-post, not on peer ingestion, so the dead inbound leg stayed invisible.
**Prevention:** (this task) re-deliver via AEF's real inbox channel; file a
meta-pickup asking AEF to add a topic→inbox consumer; record the verified relay
path in memory + learnings so future relays use the inbox-drop, not the topic.

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

### 2026-06-23T23:19:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2265-re-relay-orphaned-aef-pickups-via-inbox-.md
- **Context:** Initial task creation

### 2026-06-23T23:24:51Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
