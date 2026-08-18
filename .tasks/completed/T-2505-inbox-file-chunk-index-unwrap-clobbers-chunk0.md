---
id: T-2505
name: "Inbox file.chunk deposit unwrap_or(0) silently clobbers chunk 0 on malformed
  index"
description: "A file.chunk spool event with a missing/non-integer index defaults to
  chunk-0000.json and fs::write silently overwrites the legitimate chunk 0 — data
  corruption in the durable offline-inbox path. Reject loud (warn + Ok(false)) like
  the sibling missing-transfer_id arm."
status: work-completed
workflow_type: build
horizon:
owner: agent
created: 2026-08-03
last_update: '2026-08-18T18:59:12Z'
tags: [reliability, silent-failure, data-corruption, inbox, no-silent-failures]
components: [crates/termlink-hub/src/inbox.rs]
bvp_scores_proposed:
  - ts: '2026-08-18T18:56:48Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 3
      D2: 0
      D3: 0
      D4: 0
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=3 (body:test-or-audit-check); D2=0 (no-signal); D3=0 
      (no-signal); D4=0 (no-signal); F-RECALL=0 (no-signal); F-ORCH=0 
      (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:59:12Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 1
      tier: 2
      effort: 6
    rationale: blast_radius=1 (no-signal); tier=2 (no-signal); effort=6 
      (no-signal)
    rubric_sha: missing
---

## Context

Part of the no-silent-failures campaign (directive #2 Reliability). Residual gap in
the SAME loud-and-never-destroy campaign that fixed the sibling site: T-2490 hardened
the **reassembly** side (`ordered_chunk_paths_checked`, inbox.rs:357) so a chunk whose
filename does not parse to an index sorts LAST (`unwrap_or(u64::MAX)`) and never
reorders the valid chunks. The **deposit** side (`deposit`, inbox.rs:124) was left with
`unwrap_or(0)`, which actively DESTROYS good data instead of quarantining bad input.

The durable offline-inbox path is live: a file transfer to an offline session is spooled
to disk (wired at router.rs:375), one `chunk-{index:04}.json` per `file.chunk` event.

## RCA

- **Symptom:** A `file.chunk` event arriving with a missing or non-integer `index`
  field is written to `chunk-0000.json`, silently overwriting the legitimate chunk-0
  bytes already spooled. `std::fs::write` truncates in place; no error, no warn, no
  trace. On reassembly the transfer fails sha256 opaquely, or worse delivers wrong data.
- **Root cause:** `payload.get("index").and_then(|v| v.as_u64()).unwrap_or(0)` fabricates
  a plausible-but-wrong default (0) at a data-integrity boundary. A malformed chunk was
  already doomed to fail its own transfer; the `unwrap_or(0)` additionally causes it to
  eat a DIFFERENT, valid chunk.
- **Why the framework was blind:** The two sides of the same transfer primitive were
  hardened in separate passes. T-2490 fixed the read/reassembly side; the write/deposit
  side has an independent index-parse with the destructive default and was not swept in
  the same pass. One-bug-one-task: this is the deposit-side sibling of T-2490.
- **Fix class:** Level A/C — make the malformed-input path loud + non-destructive,
  mirroring the missing-`transfer_id` arm 20 lines above (warn + `return Ok(false)`).

## Acceptance Criteria

### Agent
- [x] `file.chunk` deposit with a missing `index` field returns `Ok(false)` + `tracing::warn!` instead of writing `chunk-0000.json`
- [x] `file.chunk` deposit with a non-integer `index` (e.g. a string) is rejected the same way
- [x] A valid `file.chunk` with an integer `index` still spools to `chunk-{index:04}.json` (no regression)
- [x] A regression test proves a malformed-index deposit does NOT overwrite a previously-spooled chunk 0
- [x] `cargo test -p termlink-hub --lib` passes

## Verification

# Hub lib tests pass (includes the new regression tests)
out=$(cargo test -p termlink-hub --lib 2>&1); echo "$out" | tail -3; echo "$out" | grep -q "test result: ok"
# The destructive default is gone from the deposit path
! grep -n 'unwrap_or(0)' crates/termlink-hub/src/inbox.rs | grep -q 'index'

## Decisions

None — the correct behavior already exists verbatim in the same function (missing
`transfer_id` → warn + `Ok(false)`); this applies it to the sibling `index` parse.

## Reviewer Verdict (v1.5)

- **Scan ID:** R-f4602cb9
- **Timestamp:** 2026-08-02T22:21:51Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 4
     - evidence: `! grep -n 'unwrap_or(0)' crates/termlink-hub/src/inbox.rs | grep -q 'index'`

### 2026-08-02T22:20:52Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
