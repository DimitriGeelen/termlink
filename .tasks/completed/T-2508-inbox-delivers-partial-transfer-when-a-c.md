---
id: T-2508
name: "Inbox delivers partial transfer when a chunk is missing"
description: >
  deliver_transfer verifies chunks are readable but not that the set is contiguous 0..total_chunks; a missing chunk is delivered as a hole and the only copy is destroyed

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
created: 2026-08-02T23:46:11Z
last_update: 2026-08-02T23:50:48Z
date_finished: 2026-08-02T23:50:48Z
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

# T-2508: Inbox delivers partial transfer when a chunk is missing

## Context

The offline file-inbox (`crates/termlink-hub/src/inbox.rs`) spools `file.*` events
for a session that is offline, then replays them on register via `deliver_pending`
→ `deliver_transfer`, and `remove_dir_all`s the spool on success.

`deposit` writes `complete.json` the instant a `file.complete` event arrives
(line 141) with NO check that all chunks are present. Each `file.chunk` deposit is
an independent RPC that can be lost during a hub blip (or rejected via the T-2505
missing-index path, returning `Ok(false)`). So `complete.json` can exist while an
intermediate chunk is absent.

`deliver_pending` gates only on `transfer.complete` (= `complete.json` exists, line
298), never on `chunks_received == total_chunks`. `deliver_transfer` (T-2489)
verifies each PRESENT chunk is readable via `ordered_chunk_paths_checked`, but never
that the set is CONTIGUOUS `0..total_chunks` — a missing chunk simply isn't listed by
`read_dir` (no error) so it returns `Some([0,1,2,4])`, delivers a hole + complete,
returns `true`, and the caller destroys the only copy → receiver sha256 mismatch →
unrecoverable loss. Sibling gap to T-2489 (unreadable chunk) and T-2490/T-2505 (bad
index): those hardened corrupt/misnamed chunks, not MISSING ones. `total_chunks` is
already read from the init payload (line 211), so the completeness gate is a pure
local I/O fix with no wire/protocol change.

## Acceptance Criteria

### Agent
- [x] A pure helper `chunk_set_is_complete(chunk_paths, total_chunks)` returns false when a chunk is missing / the set is non-contiguous, and true for a full contiguous set; `total_chunks == 0` returns true (back-compat with senders that omit the field)
- [x] `deliver_transfer` calls the completeness gate after `ordered_chunk_paths_checked` and, on failure, logs loud and `return false` (RETAIN the spool, deliver nothing further) — mirroring the T-2489 retain-don't-destroy convention
- [x] Unit tests cover: missing intermediate chunk → retained + not delivered; full contiguous set → delivered; `total_chunks=0` → delivered (back-compat)
- [x] `cargo test -p termlink-hub --lib` passes

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

out=$(cargo test -p termlink-hub --lib inbox 2>&1); echo "$out" | grep -q "test result: ok"
blk=$(awk '/async fn deliver_transfer/,/^}/' crates/termlink-hub/src/inbox.rs); echo "$blk" | grep -q 'chunk_set_is_complete'
grep -q 'fn chunk_set_is_complete' crates/termlink-hub/src/inbox.rs

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

**Symptom:** A file transfer to an offline session, where one chunk deposit was lost
during a hub blip, is delivered on the session's next register as a file with a hole;
the receiver's sha256 fails and rejects it, and the hub has already destroyed the only
copy — silent, unrecoverable loss on the durable file-transfer path.

**Root cause:** `deliver_transfer` treated "every present chunk is readable" as
equivalent to "the transfer is complete". `read_dir` never surfaces a MISSING chunk as
an error, so `ordered_chunk_paths_checked` returns the sparse set `[0,1,2,4]` for a
5-chunk transfer with chunk 3 lost. Delivery proceeds and returns `true`, so
`deliver_pending` `remove_dir_all`s the spool. The `complete` gate in `deliver_pending`
keys only on `complete.json` existing, which `deposit` writes independently of chunk
arrival.

**Why structurally allowed:** T-2489 (unreadable chunk) and T-2490/T-2505 (bad chunk
index) hardened the CORRUPT and MISNAMED chunk classes but not the MISSING chunk class.
`total_chunks` was already carried in the init payload and even surfaced in
`PendingTransfer.total_chunks`, but nothing compared `chunks_received`/the on-disk index
set against it at the destruction gate.

**Prevention:** the fix itself is the structural gate — a pure `chunk_set_is_complete`
helper enforced in `deliver_transfer` before any emit, so a non-contiguous set retains
the spool instead of destroying it. Unit-tested (missing-chunk retain, full-set deliver,
total_chunks=0 back-compat) so a revert breaks a test. No new canary/cron (per T-2468
no-breadth): this is the in-code gate on the one function that destroys the spool.

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

### 2026-08-02T23:46:11Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2508-inbox-delivers-partial-transfer-when-a-c.md
- **Context:** Initial task creation

### 2026-08-02T23:47:17Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8a4026ca
- **Timestamp:** 2026-08-02T23:50:54Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-02T23:50:48Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
