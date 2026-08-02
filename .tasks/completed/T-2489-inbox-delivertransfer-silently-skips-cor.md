---
id: T-2489
name: "inbox deliver_transfer silently skips corrupt chunk then deletes spool causing unrecoverable file loss"
description: >
  inbox deliver_transfer silently skips corrupt chunk then deletes spool causing unrecoverable file loss

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-hub/src/inbox.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-02T08:31:32Z
last_update: 2026-08-02T08:35:00Z
date_finished: 2026-08-02T08:35:00Z
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

# T-2489: inbox deliver_transfer silently skips corrupt chunk then deletes spool causing unrecoverable file loss

## Context

`deliver_transfer` (`crates/termlink-hub/src/inbox.rs:344-348`) delivers a spooled file.*
transfer to a newly-registered target. Its chunk loop reads each chunk via `read_entry`
(returns `None` on read-error OR JSON-parse-error) and, on `None`, does `continue` — silently
omitting that chunk. It then delivers `complete.json` and returns `true`. The caller
`deliver_pending` (:295-299) treats `true` as success and runs `std::fs::remove_dir_all(&xfer_dir)`,
destroying the only copy. Net: a chunk truncated/corrupted by a crash or disk-full during the
non-atomic `deposit` write (:134) is silently dropped, the receiver reassembles a file with a hole
that fails its sha256, and the spool is already gone — unrecoverable, no trace. This is the exact
directive-#2 (no silent failures) class T-2487/T-2488 hardened elsewhere, yet inbox.rs does the
OPPOSITE (skip silently, then delete). Note the init path (:324) and emit-fail path (:356) already
correctly `return false` — the chunk-read path is the lone inconsistency. Found by firing-#12
adversarial audit; verified current in code.

## Acceptance Criteria

### Agent
- [x] `deliver_transfer` no longer silently skips an unreadable/corrupt chunk: an unreadable chunk
      aborts delivery and RETAINS the spool (returns `false`, `tracing::warn!` naming the transfer),
      mirroring the existing init-unreadable (:324) and emit-fail (:356) behaviour.
- [x] A pure, unit-testable helper (no network) decides chunk deliverability: it lists + orders the
      `chunk-*` files and returns `None` if ANY is unreadable/corrupt, else the ordered paths.
      (`ordered_chunk_paths_checked`.)
- [x] New unit test: a transfer dir with a corrupt chunk → helper returns `None` (spool would be
      retained), NOT a partial success. (`ordered_chunk_paths_checked_returns_none_on_corrupt_chunk`.)
- [x] New unit test: a transfer dir with all-readable chunks → helper returns the ordered paths.
      (`ordered_chunk_paths_checked_returns_all_when_readable`.)
- [x] Existing inbox tests still pass; `cargo test -p termlink-hub --lib` passes (441/441); `cargo check -p termlink-hub` clean.

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
cargo test -p termlink-hub --lib inbox
cargo check -p termlink-hub

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

**Symptom:** An offline file transfer that had one chunk truncated/corrupted while spooled is
delivered to the target as an incomplete file (a hole where the bad chunk was); the receiver's
sha256 check fails, and the hub-side spool has already been deleted, so there is no copy to
re-deliver. The failure is invisible on the hub side (returned "delivered", logged nothing about
the skip).

**Root cause:** `deliver_transfer`'s chunk loop used `match read_entry(&chunk) { Some(e) => …,
None => continue }` — treating an unreadable/corrupt chunk as "skip and keep going" rather than a
delivery failure. It then returned `true`, and the caller's success path (`remove_dir_all`) treated
that as license to destroy the spool. The three outcomes (readable / unreadable / emit-failed) were
not handled uniformly: init-unreadable and emit-failed both `return false`, but chunk-unreadable
`continue`d.

**Why structurally allowed:** `None => continue` in a loop is an easy-to-miss silent discard — the
`Option` carried no information about WHY it was `None` (read vs parse error), and the loop's
contract ("deliver every chunk") was not enforced against its exit condition ("return true =
everything delivered"). No test exercised a corrupt-chunk spool (existing tests only build
well-formed transfers), so the skip-then-delete path was never observed. The `true` return + caller
`remove_dir_all` coupling meant the lossy skip escalated into irreversible deletion.

**Prevention:** Delivery is now gated on a pure `ordered_chunk_paths_checked` helper that returns
`None` if ANY chunk is unreadable — so a corrupt chunk aborts before any partial delivery and the
caller retains the spool. Two unit tests lock the contract (corrupt → None; all-readable → ordered
paths). The helper centralises chunk enumeration so the loop can no longer silently skip. Learning
captured: an `Option`-returning read in a delivery loop must map `None` to abort-and-retain, never
skip-and-continue, when a downstream success signal triggers destruction of the source.

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

### 2026-08-02T08:31:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2489-inbox-delivertransfer-silently-skips-cor.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-4cf7ff5b
- **Timestamp:** 2026-08-02T08:35:07Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T08:35:00Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
