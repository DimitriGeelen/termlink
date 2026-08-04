---
id: T-2524
name: "download_artifact_via_client unbounded accumulation — peer-streamed blob OOMs receiver (no size cap)"
description: >
  download_artifact_via_client unbounded accumulation — peer-streamed blob OOMs receiver (no size cap)

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
created: 2026-08-04T11:04:58Z
last_update: 2026-08-04T11:04:58Z
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

# T-2524: download_artifact_via_client unbounded accumulation — peer-streamed blob OOMs receiver (no size cap)

## Context

`download_artifact_via_client` (`crates/termlink-session/src/artifact.rs:488`) reassembles
an artifact by looping `artifact.get` calls and accumulating every returned chunk into an
in-memory `Vec<u8>` (`out`, L493) via `out.extend_from_slice(&bytes)` (L524). The loop only
terminates when the **peer-controlled** `eof` flag is set (L526). There is **no cap** on
`out.len()` — client-side, and the hub's artifact staging (`artifact_store.rs`) has no total-size
cap either (grep for `MAX_ARTIFACT`/`limit`/`cap` returns nothing anywhere on the path). The
SHA-256 integrity check (L534) runs **after** the entire blob is resident in RAM, so it cannot
prevent the exhaustion. `manifest.size` is known at the caller but never passed in or enforced.

**Failure:** a malicious/buggy peer stages a many-GB blob to the shared hub and posts an inbox
artifact envelope referencing its sha; the victim runs `file receive` →
`download_artifact_via_client` streams the whole blob into `out` (256 KB at a time) until the
peer sets `eof`, OOM-killing the receiving agent before the trailing SHA check ever executes.
Linear (not amplified) resource-exhaustion DoS — but a real availability harm to any agent that
receives a file from a peer. Reliability directive: no unbounded allocation driven by untrusted input.

**Fix (in-authority, semantics-free resource guard — same class as T-2523's clamp):** an absolute
ceiling on a single artifact download, checked in the loop *before* `extend_from_slice`, so the
accumulation can never exceed the cap. Default 2 GiB (artifacts are the large-payload path;
generous but finite), overridable via `TERMLINK_MAX_ARTIFACT_BYTES` — mirroring the codebase's
env-tunable-cap convention (`TERMLINK_MAX_CONNECTIONS`, `TERMLINK_CV_INDEX_CAP_PER_TOPIC`). No
protocol/threat-model change; purely bounds an already-untrusted stream.

**Out of scope (distinct one-bug-one-task):** the hub-side `put_streaming` staging cap — that
guards hub *disk* exhaustion (different victim, different resource) and is a separate bug to
verify and file/fix on its own. This task fixes the receiver-RAM OOM, which the client cap fully
closes regardless of what a peer manages to stage.

## Acceptance Criteria

### Agent
- [x] Absolute per-download ceiling enforced in `download_artifact_via_client` loop *before* `extend_from_slice`, returning an error (not a panic) once accumulation would exceed the cap
- [x] Cap default is 2 GiB, overridable via `TERMLINK_MAX_ARTIFACT_BYTES` (positive integer; invalid/zero/absent → default), via a pure parse helper
- [x] Pure guard predicate `artifact_download_would_exceed(current, incoming, cap)` uses saturating add (no overflow panic on `u64::MAX`); unit-tested for under/at/over/overflow, proven load-bearing via temp-revert (reverting to always-false makes the test FAIL)
- [x] `cargo test -p termlink-session --lib` passes; `cargo build -p termlink-session` clean

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
cargo test -p termlink-session --lib artifact::
cargo build -p termlink-session

## RCA

**Symptom:** A peer that streams (or stages on the hub) an unbounded blob makes a
receiving agent's `file receive` accumulate the whole thing in RAM and OOM-kill the
process, before any integrity check runs. `manifest.size` can honestly say "2 KB" while
the actual stream is many GB and nothing notices.

**Root cause:** `download_artifact_via_client` (artifact.rs:488) loops `artifact.get`
and does `out.extend_from_slice(&bytes)` (L524) with the loop terminating only on the
peer-controlled `eof` flag (L526). No ceiling on `out.len()`; the hub staging path has
no cap either; the SHA-256 verify (L534) runs after the full read, so it is a
post-mortem, not a guard.

**Why structurally allowed:** The artifact path is the *large-payload* channel (the
inline path caps at ~2 KB, everything bigger becomes an artifact), so "streaming a big
blob" is the normal case — but "big" was never bounded. No `MAX_ARTIFACT`/limit const
exists anywhere on client, hub-staging, or hub-get (confirmed by grep). The reassembly
loop trusts the sender's `eof` as the only stop condition. The other framing classes
(chunk index/count, offset arithmetic, chunk_size==0) ARE hardened (BTreeMap reads over
0..total_chunks, calculate_chunks guards chunk_size==0, no peer index*size multiply,
T-2417 fixed the chunk-accounting collision) — the missing *size* cap was the lone gap.

**Prevention:** An absolute per-download ceiling (default 2 GiB, env
`TERMLINK_MAX_ARTIFACT_BYTES`) checked *before* each `extend_from_slice`, so accumulation
can never exceed the cap regardless of the peer's `eof`. The bound is enforced through a
pure `artifact_download_would_exceed(current, incoming, cap)` predicate (saturating add,
no overflow panic) with a unit test pinning under/at/over/overflow, proven load-bearing
via temp-revert (revert to always-false → test FAILs). A future edit that drops the guard
is caught in CI. Follow-up (separate one-bug-one-task, NOT this task): a symmetric
hub-side `put_streaming` staging cap to guard hub disk exhaustion.

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

### 2026-08-04T11:04:58Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2524-downloadartifactviaclient-unbounded-accu.md
- **Context:** Initial task creation
