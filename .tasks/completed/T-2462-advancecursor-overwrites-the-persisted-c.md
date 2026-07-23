---
id: T-2462
name: "advance_cursor overwrites the persisted cursor instead of taking the max, so a stale or retried advance can regress a subscriber's delivery frontier and re-deliver already-consumed records — make it monotonic like the claim-ack path (round-16 F3)"
description: >
  advance_cursor overwrites the persisted cursor instead of taking the max, so a stale or retried advance can regress a subscriber's delivery frontier and re-deliver already-consumed records — make it monotonic like the claim-ack path (round-16 F3)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-07-23T08:40:44Z
last_update: 2026-07-23T09:29:14Z
date_finished: 2026-07-23T09:29:14Z
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

# T-2462: advance_cursor overwrites the persisted cursor instead of taking the max, so a stale or retried advance can regress a subscriber's delivery frontier and re-deliver already-consumed records — make it monotonic like the claim-ack path (round-16 F3)

## Context

Round-16 adversarial reliability hunt (F3). `Bus::advance_cursor` (lib.rs:302)
delegates to `Meta::put_cursor` (meta.rs:249), whose upsert does a plain overwrite:
`ON CONFLICT(subscriber_id, topic) DO UPDATE SET last_offset = excluded.last_offset`
(meta.rs:257). This lets a stale/retried/out-of-order `advance_cursor(sub, topic,
LOWER)` REGRESS the persisted cursor. Since the cursor is the crash-restart resume
point ("resume where the subscriber left off", lib.rs:301), regressing it re-delivers
already-consumed records (duplicate delivery). The claim-ack path 8 lines away in the
SAME file already does this correctly — `SET last_offset = MAX(last_offset,
excluded.last_offset)` (meta.rs:445) — and the receipt frontier was fixed for the
identical class in T-2456. `advance_cursor` is documented forward-only ("mark as
read", lib.rs:159); replay features use a separate since-offset mechanism, never a
cursor rewind. `put_cursor` is called ONLY by `advance_cursor` (grep-verified), so
making it monotonic is safe and matches the method's own semantics. Fix: mirror the
claim-ack `MAX` upsert.

## Acceptance Criteria

### Agent
- [x] `Meta::put_cursor`'s upsert uses `SET last_offset = MAX(last_offset, excluded.last_offset)` so `advance_cursor` can never regress a persisted cursor (mirrors the claim-ack path at meta.rs:445 and the T-2456 receipt-frontier monotonicity fix).
- [x] A regression test proves a lower-offset `advance_cursor` after a higher one leaves the cursor at the high-water mark (set 5 → advance 3 → cursor stays 5), while a genuine forward advance still moves it (5 → 7 → 7).
- [x] Existing cursor round-trip behavior (fresh insert, monotonic advance) is unchanged; full `cargo test -p termlink-bus --lib` stays green.

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

cargo test -p termlink-bus --lib cursor 2>&1 | tail -5 | grep -qE 'test result: ok'
cargo test -p termlink-bus --lib 2>&1 | tail -5 | grep -qE 'test result: ok'

## RCA

**Symptom:** A subscriber can be re-delivered records it already consumed, after a
stale/retried/out-of-order `advance_cursor` call moved its persisted cursor
backwards.

**Root cause:** `Meta::put_cursor` (meta.rs:257) upserts the cursor with a plain
overwrite (`SET last_offset = excluded.last_offset`), so it accepts a lower offset
and regresses the delivery frontier. The cursor is the crash-restart resume point,
so a regressed cursor causes re-reads.

**Why structurally allowed:** The monotonic-frontier invariant was applied
inconsistently. The claim-ack cursor advance (meta.rs:445) and the receipt frontier
(T-2456) both correctly use `MAX`, but the general `advance_cursor` path was written
with a plain overwrite and no test ever set a lower value after a higher one
(`cursor_persists_and_rounds_trip` only ever increases), so the regression was
unguarded and invisible.

**Prevention:** (1) the `MAX` upsert makes regression impossible by construction;
(2) a regression test pins the high-water-mark behavior (lower advance is a no-op);
(3) learning: a delivery frontier (cursor / receipt up_to / claim-ack) must ALWAYS
be monotonic — this is now the third instance of the same class (T-2456 receipts,
claim-ack MAX, this), so the invariant "any persisted delivery frontier upserts with
MAX, never overwrite" should be treated as a substrate-wide rule.

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

### 2026-07-22 — NEXT STEP (source fix pending — budget gate blocked it mid-round-16)

The fix is fully specified but the one-line source edit was blocked by the
context-budget wrap-up gate before it landed. **To complete (next session):**

1. In `crates/termlink-bus/src/meta.rs::put_cursor` (~line 257), change the upsert
   `SET last_offset = excluded.last_offset` → `SET last_offset = MAX(last_offset,
   excluded.last_offset)` (mirror the claim-ack path at meta.rs:445). Add the T-2462
   comment.
2. Add the regression test in `crates/termlink-bus/src/lib.rs` (near
   `cursor_persists_and_rounds_trip` ~line 985): set cursor 5, `advance_cursor` to 3,
   assert `get_cursor` == 5; then advance to 7, assert 7.
3. `cargo test -p termlink-bus --lib cursor` and the full `-p termlink-bus --lib`.
4. Commit BEFORE `fw task update --status work-completed` (focus→null jams the gate
   otherwise). Verification block already written above.

## Related findings — round-16 reliability hunt (NOW FILED AS SEPARATE TASKS)

**Filed 2026-07-23:** F1 → **T-2464** (durability inversion), F2 → **T-2463**
(silent sweep-gap). Both `owner: agent`, `horizon: later` backlog builds — BUGS
with known fix directions, deliberately NOT go/no-go inceptions (kept out of the
human decision backlog). Full detail preserved below for lineage:

- **F1 (HIGH, power-loss-only) — durability inversion / poison offset.** `post()`
  indexes durably (SQLite `synchronous=FULL`, `record_append` `tx.commit()` fsyncs,
  meta.rs:139) but the log payload is written with `write_all + flush` and NO fsync
  (`LogAppender::append`, log.rs:61-63; Rust `File::flush` is a no-op). The *pointer*
  is durable while the *data* is not — inverted vs WAL discipline. On power/kernel
  loss (NOT plain process restart — page cache survives that), the offset-N index row
  survives but its log bytes are gone → `ReaderIter::next` `read_exact`/decode fails
  (log.rs:115-118) → `Some(Err(..))`. LOUD but UNRECOVERABLE + stream-blocking: no
  skip/repair path, so every re-subscribe at/after that offset re-hits the wall
  forever. No `sync_all` anywhere in the crate. Design tension (fsync-per-post cost
  vs the ADR "single supervised durable hub / restart = recoverable pause" model
  which plausibly scopes power-loss out) — that is why it is a CAPTURE, not a rushed
  build. Two separable fixes: (a) fsync the log before the index commit (durability);
  (b) a skip-corrupt-record path so a poison offset yields a gap-marker, not a
  permanent stream wall (reliability — worthwhile even if (a) is declined).

- **F2 (HIGH, narrow trigger) — cursor-blind sweep + SILENT subscribe skip.**
  `Bus::sweep`/`sweep_records` (lib.rs:657, meta.rs:283-319) deletes by ts/keep-last-N
  with NO guard against the slowest live cursor (never consults `cursors`).
  `subscribe(topic, cursor)` with `cursor` below the new retention floor returns
  `WHERE offset >= cursor` (`records_from`, meta.rs:231) → silently jumps from
  `cursor` to `oldest`, skipping swept records with NO error and NO gap-marker → a
  slow/paused subscriber SILENTLY loses messages (Directive #2 "no silent failures"
  violation). Gap detection exists but only as an opt-in separate call
  `oldest_offset()` that `subscribe` never uses. Bounded topics (agent-presence
  Messages(1000)) are DESIGNED to drop old records — so the fix is NOT "don't sweep
  past cursors" (that would unbound the topic); it is "make the drop LOUD": when
  `subscribe` detects `cursor < oldest_offset`, surface a gap signal (typed
  gap-marker / distinct return / at minimum a `tracing::warn!`). The exact signalling
  mechanism is a subscribe-contract choice (which is why it is a CAPTURE, not a
  same-round build). Related sub-case (F4, MED, client-contract): after
  `delete_topic`→recreate, offsets restart at 0 with no epoch/generation token, so a
  client that cached its old high cursor and re-subscribes without re-reading
  `get_cursor` silently sees nothing (same silent-skip family). Bus side is correct
  (`delete_topic` wipes cursors, meta.rs:188); the gap is the missing epoch token to
  invalidate a stale client-cached cursor.

**Verified CLEAN this round (do not re-review):** bus-internal cursor/subscribe/ack
is at-least-once-safe — `subscribe` is a pure read that never advances the cursor
(lib.rs:241-259), so a crash mid-stream re-delivers (never silent loss); offsets are
gapless/monotonic (one tx, meta.rs:124-139); claim-ack advance is monotonic
(meta.rs:445); `advance_cursor` persistence is atomic+durable.

## Decision

<!-- Filled at completion of inception tasks via:
     fw inception decide T-XXX go|no-go|defer --rationale "..."

     For non-inception tasks this section is ignored. Kept in template
     so `fw inception decide` (lib/inception.sh) finds the anchor heading
     without auto-creating; T-1832 added auto-create as fallback for
     legacy tasks lacking this section. -->

## Updates

### 2026-07-23T08:40:44Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2462-advancecursor-overwrites-the-persisted-c.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.5)

- **Scan ID:** R-e87057c3
- **Timestamp:** 2026-07-23T09:29:21Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 2

**Verification-level findings:**

  1. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 32
     - evidence: `cargo test -p termlink-bus --lib cursor 2>&1 | tail -5 | grep -qE 'test result: ok'`
  2. **l387-sigpipe-risk** (partial, heuristic) @ Verification:line 33
     - evidence: `cargo test -p termlink-bus --lib 2>&1 | tail -5 | grep -qE 'test result: ok'`

### 2026-07-23T09:29:14Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
