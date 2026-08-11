---
id: T-2615
name: "Atomic-write helpers in termlink-session rename without fsync (power-loss trust-downgrade)"
description: >
  tofu save_result, registration write_atomic, and secret writes do fs::write+rename with no sync_all/parent-dir fsync; a power-loss right after rename can surface a zero-length known_hubs so all cert pins vanish and the next connection silently re-TOFUs (MITM window). Pattern-wide: no sync_all anywhere in the crate.

status: captured
workflow_type: refactor
owner: agent
horizon: next
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-11T17:06:59Z
last_update: 2026-08-11T17:06:59Z
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

# T-2615: Atomic-write helpers in termlink-session rename without fsync (power-loss trust-downgrade)

## Context

Every "atomic write" in `crates/termlink-session/src/` fixes the *truncate* window
(T-2501: temp-file + `rename` instead of truncate-in-place) but NONE call `sync_all`
on the temp file before `rename`, nor fsync the parent dir after — a `grep sync_all
crates/termlink-session/src/` returns zero hits. Confirmed sites:
- `tofu.rs:172-174` (`save_result`) — the TOFU `known_hubs` pin store.
- `registration.rs:321+` (`write_atomic`) — session registration JSON (the cited "idiom").
- secret-file writes sharing the same shape.
POSIX `rename(2)` is atomic w.r.t. *readers* but does NOT guarantee *durability*:
after `rename` returns, on a crash / power-loss / OOM-kill before the fs flushes,
some filesystems (and ext4 under `data=writeback`) can surface a **zero-length**
file. For `known_hubs` that means all cert pins vanish → the next connection
silently re-TOFUs a fresh cert with no error (**trust-downgrade / MITM window**).
The `tofu.rs` comment already names this exact risk for the truncate case; the fix
closed truncation but left the durability residual. DELICATE + NOT cleanly
unit-testable (the failure only manifests on real power-loss), which is why this is
FILED rather than built inline — the fix is standard but its proof needs fault
injection or acceptance of a structural (code-shape) test.

## Acceptance Criteria

### Agent
- [ ] A shared durable-atomic-write helper exists (write temp → `File::sync_all()` → `rename` → fsync parent dir), replacing the raw `fs::write`+`rename` in `tofu.rs::save_result`, `registration.rs::write_atomic`, and the secret-file writer
- [ ] The helper uses a UNIQUE temp name (pid + atomic counter) so concurrent writers cannot collide on a fixed `.tmp` (also mitigates the tofu-side of T-2617)
- [ ] Errors from `sync_all` / `rename` / parent fsync propagate (no `let _ =` / `.ok()` swallowing) so a failed durable write is loud, per T-2438
- [ ] A round-trip test asserts the helper produces a complete, readable file (guards the non-crash correctness); the fsync durability itself is documented as verified-by-inspection (fault injection out of scope)
- [ ] `cargo test -p termlink-session` passes

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

**Symptom:** After a power-loss/crash immediately following a TOFU pin save, the
`known_hubs` file can be zero-length; the next connection silently re-TOFUs a new
cert (trust-downgrade) with no error surfaced.

**Root cause:** The crate's atomic-write idiom guarantees reader-atomicity via
`rename` but never fsyncs the temp file or parent dir, so `rename` returning is not
a durability guarantee — the data can still be lost before the fs flushes.

**Why structurally allowed:** T-2501 correctly closed the *truncate* window and the
comment reads as a complete fix; "atomic rename" is widely mis-assumed to imply
durability. No test (impossible without fault injection) and no lint flags a
rename-without-fsync, so the residual was invisible.

**Prevention:** A single shared durable-write helper (write→sync_all→rename→parent
fsync) used everywhere, so the durability guarantee is centralized and can't be
half-applied per site. Optional follow-up: a static check flagging `fs::rename` of a
just-written temp with no preceding `sync_all` in the daemon crates.


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

### 2026-08-11T17:06:59Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2615-atomic-write-helpers-in-termlink-session.md
- **Context:** Initial task creation
