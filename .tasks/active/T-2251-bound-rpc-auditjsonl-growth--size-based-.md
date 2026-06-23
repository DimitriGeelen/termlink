---
id: T-2251
name: "Bound rpc-audit.jsonl growth — size-based rotation in audit writer"
description: >
  Bound rpc-audit.jsonl growth — size-based rotation in audit writer

status: started-work
workflow_type: build
owner: agent
horizon: now
arc_id: arc-substrate-fitness
tags: [arc:arc-substrate-fitness]
components: []
related_tasks: [T-2242, T-1304]
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T08:03:38Z
last_update: 2026-06-23T08:03:38Z
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

# T-2251: Bound rpc-audit.jsonl growth — size-based rotation in audit writer

## Context

arc-002 R7 **prevention** half (the live-host cleanup stays the operator's). The
RPC audit writer (`crates/termlink-hub/src/rpc_audit.rs`) is append-forever by
documented v1 design (`:6-7`) — the structural cause of the in-the-wild 1.36GB
`rpc-audit.jsonl`. Add a size cap + single-backup rotation so the file is bounded
without operator cron. Serves AS_RESOURCE_FOOTPRINT (w4). G-019: fixes the reason
the framework allowed unbounded growth, not just the symptom.

## Acceptance Criteria

### Agent
- [ ] Audit writer rotates: when `rpc-audit.jsonl` reaches the cap, it is renamed to `rpc-audit.jsonl.1` (overwriting any prior `.1`) and a fresh file is started — at most one rotated backup; total on-disk bounded to ~2× cap.
- [ ] Cap is configurable via `TERMLINK_AUDIT_MAX_BYTES` (read at hub `init`), default 100 MiB; `0` disables rotation (back-compat append-forever, the pre-T-2251 behavior).
- [ ] Rotation logic is a pure, lock-free `append_line_capped(path, line, cap)`; the prod `append_line` wraps it in a write-lock so concurrent dispatches can't race the rotate.
- [ ] Regression tests (PL-213 — assert the property): (a) over-cap writes produce a `.1` and keep the main file ≤ cap + one line; (b) `cap=0` never creates `.1` and grows past the cap; (c) `.1` is overwritten on a second rotation (not accumulated).
- [ ] `cargo test -p termlink-hub` passes (existing + new).

### Human
_None — all acceptance criteria are agent-verifiable (code + `cargo test`)._

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
out=$(cargo test -p termlink-hub --lib rpc_audit 2>&1); echo "$out" | grep -q "test result: ok"

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

**Symptom:** `rpc-audit.jsonl` grew to 1.36GB in production (arc-002 discovery F4),
pressuring disk and slowing the whole-file readers (`summarize_legacy_usage`).

**Root cause:** the audit writer (`rpc_audit.rs:376` `append_line`) is
append-forever with no size cap, rotation, or pruning — by explicit v1 design
(`:6-7`).

**Why structurally allowed:** the v1 design delegated disk-pressure management to
an operator cron (`>90d` deletion). On the live shared host that cron either
didn't exist or didn't fire, so growth was unbounded with nothing in-process to
stop it — a "framework relies on out-of-band hygiene that may never run" gap
(sibling to PL-168).

**Prevention:** in-process size cap + single-backup rotation, on by default
(100 MiB) — the file is now bounded regardless of operator cron. Regression tests
assert the bound holds. The operator's one-time cleanup of the existing 1.36GB
file remains R7 (live host); this task ensures it can't recur.

## Evolution

### 2026-06-23 — audit-writer rotation (R7 prevention half)
- **What changed:** R7's framing is "bound growth," but its three plan ACs are all
  *operational cleanup* (live host). Source verification showed the audit writer is
  unbounded by design — so the *structural* prevention (a code-level cap) is a
  distinct, agent-buildable deliverable separable from the operator cleanup.
- **Plan impact:** R7 splits into prevention (this task, T-2251, code) + cleanup
  (operator, live host). One-task-one-deliverable.
- **Triggered:** this task (T-2251), minted as the agent-buildable half while the
  live-host cleanup stays with the operator.

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

### 2026-06-23 — single-backup rotation, default-on, 0-disables
- **Chose:** size cap (default 100 MiB) + single rename-to-`.1` rotation, on by
  default, `TERMLINK_AUDIT_MAX_BYTES=0` to disable.
- **Why:** bounds the file to ~2× cap with no external deps or background thread
  (matches T-1155 "no implicit sweep" ethos — rotation is inline at write time).
  Default-on because the v1 "operator cron handles it" assumption demonstrably
  failed in the field (1.36GB). `0`-disables preserves exact back-compat for any
  operator who relies on the old append-forever + external cron.
- **Rejected:** (a) multi-generation `.1`/`.2`/… rotation — more disk + bookkeeping
  for no real forensic gain at this signal density; (b) front-truncation to keep
  last-N-bytes — non-atomic, risks a torn JSON line; (c) line-count cap — requires
  reading the file to count, defeating the cheapness; (d) leaving it env-gated-off
  by default — would reproduce the field failure for anyone who doesn't opt in.

### 2026-06-23 — known trade-off: `summarize_legacy_usage` reads only the live file
- After a rotation the legacy-usage tally (`rpc_audit.rs:~235`) sees only the
  post-rotation live file, not `.1`. Acceptable: the summary window is recent
  (recent records are in the live file), the default 100 MiB cap makes rotation
  rare, and rotation actually *helps* that reader (it was the unbounded 1.36GB
  whole-file slurp the explorer flagged as a scaling concern). If full-history
  tally across the backup is ever needed, the reader can glob `rpc-audit.jsonl*`
  — logged here, not built (YAGNI).

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-06-23T08:03:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2251-bound-rpc-auditjsonl-growth--size-based-.md
- **Context:** Initial task creation
