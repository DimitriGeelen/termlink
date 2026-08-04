---
id: T-2525
name: "put_streaming has no staging size cap — peer exhausts hub disk + OOMs hub at finalize fs::read"
description: >
  put_streaming has no staging size cap — peer exhausts hub disk + OOMs hub at finalize fs::read

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/artifact_store.rs, crates/termlink-bus/src/error.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T11:10:44Z
last_update: 2026-08-04T11:15:47Z
date_finished: 2026-08-04T11:15:47Z
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

# T-2525: put_streaming has no staging size cap — peer exhausts hub disk + OOMs hub at finalize fs::read

## Context

Hub-side sibling of T-2524 (which capped the *receiver's* download RAM). `ArtifactStore::put_streaming`
(`crates/termlink-bus/src/artifact_store.rs:96`) appends each peer-supplied chunk to a per-sha staging
file (L120-125) with the **only** bound being an offset-ordering check (`offset != current_len`, L113) —
there is **no total-size cap**. Two harms from one missing cap, both driven by a peer-controlled chunk
stream (`artifact.put` is reachable by any authenticated peer):

1. **Hub disk exhaustion:** the staging file grows without bound as the peer keeps appending chunks →
   fills the hub's `runtime_dir` filesystem (which also holds `hub.secret`/`hub.cert.pem` and the bus DB).
2. **Hub RAM OOM at finalize:** on `is_final`, L135 `let bytes = fs::read(&staging_path)?` reads the
   **entire** staging file into RAM to hash it. A multi-GB staged blob → hub OOM-kill at finalize, before
   the `expected_sha256` check (L137) can reject it. The hash verify is a post-read integrity check, not
   a size guard.

**Fix (in-authority, semantics-free resource guard — same class as T-2523/T-2524):** enforce an absolute
staging ceiling in `put_streaming` *before* each append, rejecting (and cleaning up the partial staging
file) once `current_len + chunk.len()` would exceed the cap. Default 2 GiB, overridable via
`TERMLINK_MAX_ARTIFACT_BYTES` (same env var + default as T-2524's client cap, so send/stage/receive share
one symmetric bound). Bounding total size also bounds the finalize `fs::read` to ≤ cap. New
`BusError::ArtifactTooLarge { limit, got }` so the refusal is loud + typed (not a silent truncation).

**Out of scope (separate optimization, NOT this bug):** streaming the finalize hash through the hasher
instead of `fs::read`-ing the whole file — reduces the finalize RAM spike from ≤cap to O(chunk), but is a
memory-efficiency improvement, not the unbounded-DoS fix. The cap makes the spike bounded; that closes
the DoS.

## Acceptance Criteria

### Agent
- [x] `put_streaming` rejects a chunk that would push cumulative staged bytes over the cap *before* writing it, and removes the partial staging file on rejection (no orphaned partial blob)
- [x] New typed error `BusError::ArtifactTooLarge { limit, got }` returned on over-cap; cap default 2 GiB, overridable via `TERMLINK_MAX_ARTIFACT_BYTES` (positive int; zero/invalid/absent → default)
- [x] Pure guard predicate `staging_would_exceed(current_len, chunk_len, cap)` uses saturating add; unit-tested under/at/over/overflow, proven load-bearing via temp-revert (always-false → test FAILS)
- [x] Integration test: a streamed transfer exceeding a low test cap (injected via the private `put_streaming_capped` cap param — avoids racing the process-global env in parallel tests; the env→cap wiring is covered by `parse_artifact_cap`) returns `ArtifactTooLarge` and leaves no staging file
- [x] `cargo test -p termlink-bus --lib` passes; `cargo build -p termlink-bus` clean

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
cargo test -p termlink-bus --lib streaming_put
cargo build -p termlink-bus

## RCA

**Symptom:** A peer streaming an unbounded `artifact.put` chunk sequence to a hub fills the
hub's runtime_dir disk (staging file grows without bound) and, on finalize, OOMs the hub —
`fs::read` loads the whole staged blob into RAM to hash it. Either way the hub (which also
holds `hub.secret`/`hub.cert.pem`/bus DB on that filesystem) is taken down by a single peer.

**Root cause:** `ArtifactStore::put_streaming` (artifact_store.rs:96) appends each chunk to a
staging file (L120-125) guarded only by an offset-ordering check (L113) — no cumulative
size cap. The finalize `fs::read` (L135) and the `expected_sha256` verify (L137) are both
after the bytes are already on disk / in RAM, so neither bounds the transfer.

**Why structurally allowed:** Same blind spot as T-2524's client half — the artifact path is
the *large-payload* channel, so "streaming a big blob" is the designed-for case, but "big"
was never bounded on either the receiver (T-2524) or the hub-staging (this) end. No
`MAX_ARTIFACT`/limit const existed anywhere on the path. The offset-mismatch check gave a
false sense of "the stream is validated" when it only enforced ordering, not size.

**Prevention:** An absolute staging ceiling (default 2 GiB, env `TERMLINK_MAX_ARTIFACT_BYTES`
— the same knob as T-2524 so send/stage/receive share one symmetric bound) enforced *before*
each append, with the partial staging file removed on rejection and a loud typed
`BusError::ArtifactTooLarge`. The bound is computed through a pure
`staging_would_exceed(current_len, chunk_len, cap)` predicate (saturating add) unit-tested +
proven load-bearing via temp-revert, plus an integration test driving an over-cap stream to
`ArtifactTooLarge` with no leftover staging file. Closes the OOM/exhaustion class on both
ends (T-2524 receiver RAM + this hub disk & finalize RAM).

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

### 2026-08-04T11:10:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2525-putstreaming-has-no-staging-size-cap--pe.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-7318186d
- **Timestamp:** 2026-08-04T11:15:50Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-04T11:15:47Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
