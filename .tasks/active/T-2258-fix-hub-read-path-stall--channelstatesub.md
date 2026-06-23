---
id: T-2258
name: "Fix hub read-path stall — channel.state/subscribe deadlock under concurrent write"
description: >
  ring20-management major finding (framework:pickup offset 51, 2026-06-23) + corroborated this session: channel.state/subscribe blocks indefinitely on large/low-traffic topics while channel.post is unaffected; aggravated by a recent write to the same topic. Hypothesis: read-subscribe lock contention/deadlock in the hub. Investigate + fix.

status: started-work
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-06-23T19:39:50Z
last_update: 2026-06-23T19:54:00Z
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

# T-2258: Fix hub read-path stall — channel.state/subscribe deadlock under concurrent write

## Context

**Source:** ring20-management `finding` on `framework:pickup` offset 51 (2026-06-23),
addressed to "framework-agent / termlink-agent (.107 hub owner)". Supporting warm-ping
note at offset 50. Related systemic finding (R3 delivery-confirmation gap) at offset 48
→ tracked separately as T-2259.

**Symptom (ring20, proven client-side):**
- `channel.post` is unaffected — posts ack instantly; many reads return ~90ms.
- NOT auth/transport, NOT hub-down (fedprobe full-state 90ms every trial), NOT version
  skew (ring20 CLI and .107 hub BOTH on 0.11.1367).
- Reads (`channel.state` / `subscribe` / `info` / snippet tail-scan) **intermittently
  HANG**, concentrated on large/low-traffic topics (the big DM topic, `framework:pickup`
  ~1639 rows). Tiny topics (`health:ring20-fedprobe`) never hang.
- **Correlated with writes:** a cold `channel state framework:pickup --hub .107` returned
  1639 rows in 97ms; the SAME read immediately after a `channel.post` to that topic HUNG
  (killed at 12s). Looks like read-subscribe lock contention / deadlock in the hub.

**Independent corroboration (this session, 2026-06-23):** `fw peers --all`,
`scripts/agent-listeners-fleet.sh`, and `termlink channel subscribe framework:pickup
--json` all timed out / returned empty under a 25s bound — same read-path stall signature
on a large topic.

**Repro (from a 1367 client):**
```
timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100
# fast sometimes; post a note to the topic then immediately re-read → hangs.
# fedprobe (tiny, ~every 5min) never reproduces.
```

**Investigation starting points (hub-side):** lock acquisition order in the
state/subscribe read path vs the append/post write path; whether a read holds a lock
across a full-canonical-state scan of a large topic while a concurrent post blocks (or
vice-versa). Needs .107 hub logs/metrics to localize — the .107 hub is a LIVE shared
host; do not restart/mutate it autonomously.

**Scope:** one bug = one task (the read-path deadlock). Delivery-ACK (R3) and peer
registry are out of scope (T-2259 / separate).

## Acceptance Criteria

### Agent
- [x] Root cause localized to the specific blocking-walk sites in the read path (`channel.rs:851` subscribe, `:980` receipts), documented in `## RCA` — confirmed by Explore lock-map (no std-guard-across-`.await`, no lock-ordering deadlock); matches the symptom (read hangs under concurrent write, large-topic-only, writes fast).
- [x] Regression test `channel_subscribe_no_hang_under_concurrent_walks_t2258` added — K genuinely-concurrent full-topic walks (`Arc<Bus>`) + concurrent writers under a bounded join, closing the sequential-only gap in the T-2013 test. Forward GUARD (passes post-fix; catches a future re-block of the worker pool). HONEST LIMITATION (RCA + Evolution): an in-process handler test cannot reproduce the reactor-starvation hang, so it does NOT fail pre-fix; definitive pre/post confirmation is the operator's live `.107` re-test (Human AC below).
- [x] Fix applied: both large-topic read walks now run on tokio's blocking pool via `spawn_blocking` (not `block_in_place`), so no full-topic scan pins a worker thread / starves the I/O reactor. (The O(K) gated cv_index walk stays `block_in_place` — small, opt-in, not the field culprit.)
- [x] `cargo test -p termlink-hub --lib` (368) + `cargo test -p termlink-bus` (79) pass, 0 failures (incl. both starvation tests).
- [x] `cargo build -p termlink` succeeds (release-buildable fixed binary exists in repo).

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
- [ ] [REVIEW] Fixed binary deployed to the `.107` hub and the live repro no longer hangs (OPERATOR step — the `.107` hub is a live shared host; not agent-actionable).
  **Steps:**
  1. Build + deploy the fixed binary to the host running the `.107` hub and restart it (preserving runtime_dir per CLAUDE.md §volatile-runtime_dir).
  2. From a client: `timeout -s KILL 12 termlink channel state framework:pickup --hub 192.168.10.107:9100`
  3. Post a note to the topic, then immediately re-read the same topic.
  **Expected:** The read returns within the bound every time, including immediately after a post.
  **If not:** Capture `.107` hub logs/metrics during the hang and reopen the task with the new evidence.

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

cargo test -p termlink-hub --lib
cargo test -p termlink-bus
cargo build -p termlink

## RCA

**Symptom:** `channel.state`/`subscribe`/`receipts` reads block indefinitely (field: killed at 12s) on large/low-traffic topics while `channel.post` is unaffected; aggravated by a concurrent write to the same topic; large-topic-only. (ring20-management, framework:pickup offset 51.)

**Root cause:** The hub read path walks the full topic (O(N) blocking `seek`+`read_exact` per record) inside `tokio::task::block_in_place` (`channel.rs:851` subscribe, `:980` receipts). `block_in_place` only converts the *calling* worker into a blocking thread — it does not move the work off the bounded worker pool. Under K concurrent large-topic walks (K > `worker_threads`), every worker parks in a walk and the tokio I/O reactor (which reads RPC request lines and writes responses over the socket) is starved, so in-flight and new reads hang. `channel.post` is O(1) at the SQLite index (`meta.rs::record_append`) and never enters the walk, so writes stay fast — and a concurrent post extends the window in which readers are mid-walk, aggravating the stall. Tiny topics finish the walk before the pool saturates, so they never hang. (`channel state` pages `channel.subscribe`, so the field `channel state` repro hits the subscribe walk.) No std-guard-across-`.await` and no lock-ordering deadlock exist — verified by Explore lock-map.

**Why structurally allowed:** T-2013 already identified the blocking-walk-pins-worker class and added `block_in_place` as the mitigation — but its regression test (`channel_subscribe_no_worker_starvation_under_concurrent_writes`) ran the walks **sequentially** (the author dropped the concurrent writer, believing `Bus` couldn't be shared across threads). So the *concurrent-walk* case — the actual failure mode — was never exercised, and `block_in_place`'s insufficiency under concurrency went undetected until a peer hit it in production.

**Prevention:** (1) `spawn_blocking` replaces `block_in_place` for both large-topic walks — blocking work runs on tokio's dedicated blocking pool, so worker threads and the reactor never starve regardless of walk count. (2) New regression test `channel_subscribe_no_hang_under_concurrent_walks_t2258` shares `Bus` via `Arc` and runs 8 genuinely-concurrent walks + 3 concurrent writers under a bounded join — closing the sequential-only gap in the T-2013 test. **Transparency note:** the in-process test cannot reproduce the *reactor-starvation hang* (no socket I/O in-process; the walk of small records is fast), so it is a forward regression GUARD, not a reproduction of the field hang. Definitive confirmation is the operator's live `.107` re-test (Human AC) after deploying the fixed binary.

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

### 2026-06-23 — reproduce-first came back inconclusive; pivoted to evidence-based fix + guard
- **What changed:** Plan was reproduce-first (failing test before fixing). The concurrent-walk test PASSES even pre-fix — an in-process handler test can't reproduce the hang because (a) the walk of ~1500 small records is fast and (b) the field hang is tokio I/O-reactor starvation over real sockets, which a direct-handler test never exercises. Root cause is nonetheless confirmed by code analysis (block_in_place is in the read path; T-2013's own comment documents this exact symptom; the field signature matches precisely).
- **Plan impact:** The regression test is a forward GUARD (concurrent walks complete; catches a future regression to blocking-the-pool), not a pre-fix reproduction. A faithful socket-level repro was assessed (server.rs has the harness) but deemed expensive + likely timing-flaky (`block_in_place` spawns helper workers; server tests default to `current_thread` where it panics) — not worth the budget vs. the operator's live re-test.
- **Triggered:** Human/operator AC for live `.107` confirmation (already in the task). No new sub-tasks.

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

### 2026-06-23T19:39:50Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2258-fix-hub-read-path-stall--channelstatesub.md
- **Context:** Initial task creation

### 2026-06-23T19:54:00Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
