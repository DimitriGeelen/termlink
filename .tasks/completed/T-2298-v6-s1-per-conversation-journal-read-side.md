---
id: T-2298
name: "V6-S1 per-conversation journal read-side mirror"
description: >
  V6-S1 per-conversation journal read-side mirror

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [arc:reliable-comms]
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-01T11:41:58Z
last_update: 2026-07-01T11:48:28Z
date_finished: 2026-07-01T11:48:28Z
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

# T-2298: V6-S1 per-conversation journal read-side mirror

## Context

arc-003 reliable-comms V6 (apex, T-2296) **slice S1** — the smallest-safe-first,
peer-free step of the apex. Design: `docs/plans/T-2296-v6-direct-transport-first-design.md`
§S1. Ships **script-first** (no Rust rebuild for v1, mirroring the V3a sidecar
precedent). Pure-additive read-side mirror: `dm:*` turns are copied into a durable
per-conversation SQLite journal under `~/.termlink/journals/`; the hub firehose stays
authoritative and untouched (moving dm: OFF the firehose is S5, out of scope here).
Delivers T-2296 AC4 ("per-conversation journal is mineable") standalone and de-risks
the journal schema + query surface before S3/S5 build on it.

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [x] `scripts/journal-mirror.sh` subscribes to `dm:*` topics on a hub and appends new envelopes to `~/.termlink/journals/journal.sqlite` (`messages` table keyed by `(topic, offset)`, columns: topic, offset, conversation_id, sender_id, msg_type, ts, payload, observed_addr); reuses the `dm:*` enumeration pattern from `notify-sidecar.sh` — LIVE: 135 dm topics scanned, 1939 real rows journaled
- [x] The mirror is **idempotent** — re-running over the same range adds no duplicate rows (`INSERT OR IGNORE` on the `(topic, offset)` unique key) — test T4 (reinsert=0, total unchanged)
- [x] `scripts/agent-journal.sh <conversation> [--since-offset N] [--json]` queries the journal (NOT the firehose) and returns that conversation's mirrored messages, newest-relevant first, with a `--json` envelope — proven live on a real 3-msg conversation + tests T3/T5/T6
- [x] The firehose is untouched — the mirror only reads (`channel subscribe`) and writes its own sqlite; it never acks, trims, or posts (S5 does suppression) — no `channel ack`/`post`/`--await-ack` in journal-mirror.sh
- [x] `scripts/test-journal-mirror.sh` proves end-to-end on a local hub (self-post to a `dm:` topic → mirror → journal row exists → `agent-journal.sh` returns it → idempotent re-run adds 0 rows); SKIPs cleanly with no hub — 8/8 pass

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
bash scripts/test-journal-mirror.sh
bash -n scripts/journal-mirror.sh
bash -n scripts/agent-journal.sh

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

### 2026-07-01 — S1 shipped script-first; journal keyed by (topic, offset), queryable by topic OR conversation_id
- **What changed:** Built the V6-S1 journal as SHELL SCRIPTS (per the design's
  script-first v1: no Rust rebuild, mirrors the V3a sidecar precedent) rather than
  the `conversation_journal.rs` Rust module the design also sketched. Insert/query
  use an inline `python3` step (a framework hard-dep) with parameterized SQL —
  robust for arbitrary payloads (newlines/quotes/unicode) without shell-escaping or
  the ~10min cargo rebuild. If a later slice needs the journal in-process (hub-side
  S5 option a), the Rust module becomes a follow-up.
- **Plan impact:** Partition key resolved to **`(topic, offset)`** (a `dm:a:b` topic
  IS the peer-pair conversation, and offset is the natural idempotency key), with
  `conversation_id` stored as an indexed column so `agent-journal.sh` resolves by
  EITHER peer-pair topic OR thread cid. The design left `<convo_id>.sqlite`
  vs one-DB open; chose **one DB** (`journal.sqlite`) — simpler, one connection, and
  cross-conversation forensic (S5's aggregate-on-demand) is a single query.
- **Triggered:** T-2296 AC4 ("per-conversation journal is mineable") is now met
  standalone by S1 (live: 1939 real dm rows across 134 topics). Next V6 slice = S2
  (transport-select seam in agent-send.sh) then S3 (sidecar writes here + posts the
  journaled receipt). Deferred: journal retention/compaction (design §5 Q6) — the
  store grows unbounded; a reaper mirroring offline-queue `dead_letters` is a
  follow-up, not S1 scope.

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

### 2026-07-01T11:41:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2298-v6-s1-per-conversation-journal-read-side.md
- **Context:** Initial task creation

### 2026-07-01T11:42:16Z — status-update [task-update-agent]
- **Change:** tags: +arc:reliable-comms

### 2026-07-01T11:48:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
