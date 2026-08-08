---
id: T-2533
name: "unread/ack use count-1 as latest offset — silently wrong after topic sweep (silent unread data-loss on core durable-messages verb)"
description: >
  count-1 != latest offset once a topic is swept (offsets monotonic via next_offset; count is COUNT(*)). Three sinks (channel.rs:3081 resolve_latest_offset, channel.rs:8697 compute_unread_rows, tools.rs:2898 compute_unread_rows_mcp) use count-1 as the latest offset. After a sweep (normal path for agent-chat-arc, T-2252), channel inbox silently under-reports unread (drops rows where cursor>=stale-latest) and ack frontiers stick. Fix: add latest_offset (=next_offset-1) to bus (mirror oldest_offset meta.rs:231) + channel.list response + switch the 3 sinks. Agent-buildable, medium (5 files/4 crates+test). Highest-severity campaign find.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-bus/src/lib.rs, crates/termlink-bus/src/meta.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-hub/src/channel.rs, crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-04T14:34:47Z
last_update: 2026-08-08T07:50:28Z
date_finished: 2026-08-08T07:50:28Z
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

# T-2533: unread/ack use count-1 as latest offset — silently wrong after topic sweep (silent unread data-loss on core durable-messages verb)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Context

Found + verified-in-code by the T-2468 campaign (durable-messages correctness lens).
**Silent data-loss bug** (worse than a crash) on the core "exchange durable messages"
verb.

**The broken invariant:** the code assumes `count - 1 == latest offset`. False after
any sweep/trim. Offsets are monotonic — `record_append` draws from `next_offset`
(`crates/termlink-bus/src/meta.rs:142/147`) and never rewinds it; `sweep_records`
(`meta.rs:308`, DELETEs at `:319`/`:333`) and `trim_topic` only shrink `COUNT(*)`,
leaving `next_offset`. `channel.list` `count` = `COUNT(*)`. So after a sweep, live
offsets are e.g. `4000..4999` but `count == 1000` → `count-1 == 999 ≠ 4999`.

**Three reachable sinks using `count-1` as an offset:**
1. `resolve_latest_offset` — `crates/termlink-cli/src/commands/channel.rs:3081`
   (`Some(count-1)`) — feeds `cmd_channel_ack` auto-resolve (channel.rs ~3170).
2. `compute_unread_rows` — `crates/termlink-cli/src/commands/channel.rs:8697`
   (`let latest = count - 1;`) — powers `channel inbox` / `/check-arc`.
3. `compute_unread_rows_mcp` — `crates/termlink-mcp/src/tools.rs:2898` — powers
   `termlink_agent_inbox`.

**Repro (silent under-report):** `agent-chat-arc` under `Messages(1000)`, operator
`channel sweep` cron (T-2252, the normal path — no background sweeper). Live offsets
`4000..4999`, count 1000, latest→999. A reader consumed through 4999 (persisted cursor
5000). 4 new posts at 5000..5003; sweep → live `4004..5003`, count 1000, latest→999.
`compute_unread_rows`: `cursor(5000) >= latest(999)` → **row dropped → inbox reports 0
unread while 4 messages sit unseen.** Symmetric ack bug: `ack` (no `--up-to`) resolves
`up_to = count-1 = 999`, posts a receipt frontier of 999; `unread` then starts at 1000,
sees live 4000..4999, reports 1000 unread immediately after "mark read" — frontier
stuck ≤ count-1 forever; `check-outbox`/`awaiting-ack` mis-classify the reader.

**Only existing guard** is `count == 0` (prevents underflow, not the scale error). The
offset-based receipt-frontier math (T-2494/T-2456) is itself correct — it is *fed* a
wrong offset by these three helpers.

## Recommended Fix (unambiguous, in-authority, O(1) — option (a))

The bus already has the symmetric primitive `oldest_offset` (`meta.rs:231`,
`lib.rs:287`). Add the mirror:
1. `meta.rs`: `latest_offset(topic) -> Result<Option<u64>>` = `next_offset - 1` (None
   when `next_offset == 0` / no records). Mirror `oldest_offset`.
2. `lib.rs`: expose `Bus::latest_offset` (mirror the oldest_offset passthrough).
3. Hub `channel.list` response: add a `latest_offset` field alongside `count` /
   `oldest_offset` (find where `count` is populated — channel.rs ~1530).
4. Switch all three sinks to use the returned `latest_offset` instead of `count - 1`
   (fall back to `count-1` only if the field is absent, for old-hub back-compat).
Reject option (b) (client-side O(N) walk) — the O(1) field addition is strictly better
and the bus already models the symmetric primitive.

## Acceptance Criteria

### Agent
- [x] `Bus::latest_offset(topic)` returns `next_offset - 1` (None when empty), mirroring `oldest_offset`; unit-tested incl. the post-sweep case (offsets 0..4, sweep keeps 3,4 → count 2 but latest_offset still 4, not count-1=1; also survives full trim). `latest_offset_is_monotonic_across_sweep` (bus lib)
- [x] `channel.list` response carries `latest_offset` (hub channel.rs:1530); all three sinks use it instead of `count-1` with back-compat fallback: `resolve_latest_offset` (via new pure `latest_offset_from_list_entry`), `compute_unread_rows` (new `topic_latest` param), `compute_unread_rows_mcp` (new `topic_latest` param); both `cmd_channel_inbox` + MCP `agent_inbox` callers build the latest map from the response
- [x] Load-bearing test: `compute_unread_rows_swept_topic_uses_latest_offset_not_count` (swept: count 1000, latest 4999, cursor 4990 → 9 unread). TEMP-REVERT PROVEN: neutralizing the `topic_latest` branch → test FAILS `left:0 right:1` (silent under-report), restored → green
- [x] Symmetric ack test: `latest_offset_from_list_entry_swept_topic_uses_latest_offset` — swept-topic entry resolves frontier to 4999 (true latest), not 999; legacy no-field entry falls back to count-1; never-posted → None
- [x] `cargo build` clean; `cargo test`: bus 105 pass, hub-channel 90 pass, mcp lib 886 pass, CLI unread 10 pass (incl. 2 new swept + 1 ack-path)

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

cargo test -p termlink-bus --lib latest_offset_is_monotonic_across_sweep
cargo test -p termlink-mcp --lib agent_inbox_compute_unread_rows_swept_topic_uses_latest_offset
cargo test -p termlink --bin termlink compute_unread_rows_swept_topic_uses_latest_offset_not_count
cargo test -p termlink --bin termlink latest_offset_from_list_entry_swept_topic_uses_latest_offset

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

**Symptom:** `channel inbox` / `termlink_agent_inbox` silently under-report unread (drop
rows) and `ack` frontiers stick on any topic that has been swept/trimmed — i.e. exactly
`agent-chat-arc` and other high-rate topics under `Messages(N)` retention. Readers miss
messages with no error.

**Root cause:** three sinks equate `count - 1` (from `COUNT(*)`) with the latest offset.
Offsets are monotonic (`next_offset`) and never rewound on sweep, so `count - 1` under-
states the true latest offset by exactly the number of swept records.

**Why structurally allowed:** the bus exposes `oldest_offset` but never exposed the
symmetric `latest_offset`, so client helpers reconstructed it from `count` — a
reconstruction that is only valid on a never-swept topic. No test exercised unread
math AFTER a sweep (the swept-topic + unread interaction was untested), so the framework
was blind to it for the entire time sweep and unread have coexisted (>7 days — a G-019
sustained-blindness class; consider a concerns.yaml entry if not built promptly).

**Prevention:** expose the true `latest_offset` primitive (single source of truth) so no
client reconstructs it; add a post-sweep unread regression test (the load-bearing AC)
so the swept+unread interaction can never silently regress again.

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

### 2026-08-04T14:34:47Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2533-unreadack-use-count-1-as-latest-offset--.md
- **Context:** Initial task creation

### 2026-08-08T07:37:01Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-8bf22265
- **Timestamp:** 2026-08-08T07:51:19Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-08T07:50:28Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
