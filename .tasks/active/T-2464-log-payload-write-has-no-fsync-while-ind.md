---
id: T-2464
name: "log payload write has no fsync while index commit does — durability inversion
  yields an unrecoverable poison offset on power loss (round-16 F1)"
description: >
  post() indexes durably (SQLite synchronous=FULL, record_append tx.commit fsyncs)
  but the log payload is written with write_all+flush and NO fsync (LogAppender::append;
  Rust File::flush is a no-op). The pointer is durable while the data is not — inverted
  vs WAL discipline. On power/kernel loss (not plain process restart — page cache
  survives that), the offset-N index row survives but its log bytes are gone -> ReaderIter::next
  read_exact/decode fails -> Some(Err(..)): LOUD but UNRECOVERABLE and stream-blocking
  (no skip/repair path, every re-subscribe at/after that offset re-hits the wall forever).
  No sync_all anywhere in the crate. Two separable fixes: (a) fsync the log before
  the index commit (durability — has a design tension: fsync-per-post cost vs the
  ADR single-supervised-durable-hub / restart=recoverable-pause model that plausibly
  scopes power-loss out — this half may need a human go/no-go); (b) a skip-corrupt-record
  path so a poison offset yields a gap-marker not a permanent stream wall (reliability
  — worthwhile even if (a) is declined). Round-16 reliability hunt F1, captured in
  T-2462.

status: captured
workflow_type: build
owner: agent
horizon: later
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-23T09:28:28Z
last_update: '2026-08-18T18:58:38Z'
date_finished:
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
bvp_scores_proposed:
  - ts: '2026-08-18T18:55:33Z'
    estimator: bvp-estimator-v1-heuristic
    scores:
      D1: 4
      D2: 0
      D3: 2
      D4: 2
      F-RECALL: 0
      F-ORCH: 0
    rationale: D1=4 (body:structural-gate); D2=0 (no-signal); D3=2 
      (body:default-change); D4=2 (body:env-class-handled); F-RECALL=0 
      (no-signal); F-ORCH=0 (no-signal)
    rubric_sha: missing
cost_estimate_proposed:
  - ts: '2026-08-18T18:58:38Z'
    estimator: bvp-estimator-v1-heuristic
    cost_estimate:
      blast_radius: 0
      tier: 2
      effort: 7
    rationale: blast_radius=0 (no-signal); tier=2 (no-signal); effort=7 
      (no-signal)
    rubric_sha: missing
---

# T-2464: log payload write has no fsync while index commit does — durability inversion yields an unrecoverable poison offset on power loss (round-16 F1)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
<!-- Criteria the agent can verify (code, tests, commands). P-010 gates on these. -->
- [ ] [First criterion]
- [ ] [Second criterion]

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

### 2026-07-23T09:28:28Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2464-log-payload-write-has-no-fsync-while-ind.md
- **Context:** Initial task creation

### 2026-08-03 — severity sharpening: durability inversion is not just loss, it is silent WRONG-CONTENT aliasing (T-2468 campaign firing #35, independent re-discovery)
- **What the original description under-states:** the frontmatter framing tops out at
  "log bytes gone → `ReaderIter::next` read fails → skip/loss" (and predates T-2487's
  reader-skip, which already fixed the *stream-wall* half). The genuinely worse
  consequence — which nothing tracks — is **byte-position reuse / offset aliasing**:
- **Mechanism.** `LogAppender::append` (`log.rs:52-65`) does `seek(SeekFrom::End(0))`
  to derive the record's start byte, writes payload with `write_all` + `flush()`
  (a no-op for `std::fs::File` — never fsync'd), and returns that start. `post()`
  (`lib.rs:190-192`) then durably commits the index row (SQLite `synchronous=FULL`,
  `record_append` fsyncs) mapping the fresh monotonic offset J → `byte_pos = start_J`.
  On **power/kernel loss** after the index commit but before J's payload writeback,
  the physical EOF **regresses to `start_J`** (J's bytes never reached stable storage;
  J's index row did). On restart, the next `post` R does `seek(End(0)) == start_J`,
  writes R's `[len_R][payload_R]` there, and `record_append` stores a *second* row
  offset M (>J) → `byte_pos = start_J`. Two distinct logical offsets now alias one
  byte region.
- **Silent corruption path.** Reading offset J seeks to `start_J + 8` and reads
  `len_J` bytes = the first `len_J` bytes of R's payload. **If `len_J == len_R`**
  (common for same-shaped envelopes — heartbeats, fixed-form DMs), `read_exact`
  succeeds and `decode_envelope` succeeds → offset J durably returns **R's content**.
  No error, no skip, no gap-marker.
- **T-2487 does NOT cover this.** The reader-skip only fires on `UnexpectedEof`
  (truncation) or decode failure. On a length match both succeed, so the skip never
  triggers — the mitigation for the loss half is structurally blind to the aliasing
  half. Unequal sizes degrade to the already-documented skip (still loss).
- **Impact on the go/no-go this task gates:** this reclassifies the defect from a
  durability/availability concern (a message may be lost, loudly) to a **correctness/
  integrity** one (a durable read may silently return a *different* message). That is a
  stronger argument for fix (a) — fsync-payload-before-index — since fix (b) (the skip
  path, shipped as T-2487) provably cannot mask it. A cheaper alternative worth
  costing during the human call: on `LogAppender::open`/restart, reconcile the append
  cursor to the index's highest durable `byte_pos + 8 + length` (and truncate any
  bytes beyond a validated tail) so a regressed physical EOF can never be re-filled
  under a live index row — closes the aliasing window without a per-post fsync.
- **Disposition unchanged:** still human-gated (durability-vs-throughput / ADR
  single-supervised-hub power-loss-scope call). No code changed. This entry only
  sharpens the severity picture the eventual go/no-go decision rests on.
