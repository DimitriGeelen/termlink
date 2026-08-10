---
id: T-2588
name: "fix 4 sibling agent analytics tools sharing legacy msg_type-post filter (T-2587 sweep)"
description: >
  Four agent chat-arc analytics MCP tools share the same legacy msg_type==post filter that T-2587 fixed for response_received: termlink_agent_search_by (tools.rs ~24336), termlink_agent_threads_by (~24519), termlink_agent_busiest_threads (~24619), termlink_agent_recent_decisions (~24705). Real content is note/chat/post (agent_post/channel_post default note, bus_client/offline_queue chat) so != Some(post) continue drops all real content and these tools silently return empty/zero. Correct content set is Some(post)|Some(chat)|Some(note) per shared helpers at tools.rs 3444/5383. Scoped sweep: per-tool confirm each genuinely wants CONTENT (not a distinct post-typed subset) then apply the set + load-bearing test each. From T-2468 verb-2 hunt, sibling class of T-2587.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: [bug]
components: [crates/termlink-mcp/src/tools.rs]
related_tasks: []
# arc_id:                         # T-1849: optional — slug (e.g. "arc-grooming") OR arc-NNN (e.g. "arc-005")
#                                 # When set, must resolve to .context/arcs/<id>.yaml; PreToolUse hook
#                                 # (check-arc-id) blocks save under agent control if it doesn't resolve.
#                                 # Empty/missing → unassigned (allowed). See CLAUDE.md §Task System.
created: 2026-08-09T22:38:02Z
last_update: 2026-08-10T18:43:11Z
date_finished: 2026-08-10T18:43:11Z
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

# T-2588: fix 4 sibling agent analytics tools sharing legacy msg_type-post filter (T-2587 sweep)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Shared pure predicate `is_content_msg_type(&Value) -> bool` added, matching `Some("post")|Some("chat")|Some("note")` (the canonical content set from tools.rs 3048/3550/5489).
- [x] All 4 named sites (`termlink_agent_search_by`, `termlink_agent_threads_by`, `termlink_agent_busiest_threads`, `termlink_agent_recent_decisions`) no longer use `!= Some("post")`; each verified in code to genuinely want CONTENT (payload-bearing posts), not a distinct post-typed subset.
- [x] Each of the 4 tools' post-processing loop extracted into a pure helper taking `&[Value]` so it is unit-testable off the RPC path (T-2587 pattern).
- [x] One load-bearing test per tool: feed a `note`+`chat` envelope, assert the tool surfaces it; proven load-bearing by temp-reverting the filter to post-only → test FAILS → restore.
- [x] `cargo test -p termlink-mcp --lib` passes (899 passed, 0 failed).
- [x] Follow-up filed for the remaining `!= Some("post")` sites outside this task's 4-tool scope (class is broader than filed). → T-2590

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

cargo test -p termlink-mcp --lib search_by_finds_note_and_chat_content
cargo test -p termlink-mcp --lib threads_by_finds_note_and_chat_roots
cargo test -p termlink-mcp --lib busiest_threads_counts_note_root_with_reply
cargo test -p termlink-mcp --lib recent_decisions_finds_note_and_chat_markers
cargo test -p termlink-mcp --lib is_content_msg_type_accepts_content_rejects_noise

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

**Symptom:** Four agent chat-arc analytics MCP tools — `termlink_agent_search_by`,
`termlink_agent_threads_by`, `termlink_agent_busiest_threads`,
`termlink_agent_recent_decisions` — silently returned empty/zero regardless of
real chat-arc traffic. A content search over a busy topic found nothing; the
"recent decisions" forensic tool answered "no decisions this week" even when
GO:/DECISION: posts existed.

**Root cause:** each tool's post-processing loop opened with
`if env.get("msg_type").and_then(|v| v.as_str()) != Some("post") { continue; }`.
No modern producer emits the literal `msg_type="post"` — the T-1499 migration made
`note` the default for channel/agent posts and `chat` the value for
bus_client/offline_queue sends (verified: tools.rs 17915/17993 default to `note`,
18262/20836 emit `chat`). So the filter dropped every real content envelope and
kept only the empty set of literal-"post" messages. The canonical content
predicate is `Some("post") | Some("chat") | Some("note")`, already inlined by the
sibling helpers at tools.rs ~3048/3550/5489 — these 4 loops were simply never
migrated.

**Why structurally allowed:** the T-1499 migration updated the SHARED content
helpers but left the hand-rolled per-tool loops on the old `== "post"` convention.
The predicate was duplicated inline across ~27 sites with no single choke-point, so
a migration that touched the shared copies could not mechanically reach the inline
ones. These tools have no unit tests exercising the content filter against realistic
`note`/`chat` envelopes, so an always-empty result looked identical to a genuinely
empty topic — the "silent zero" failure mode (Reliability: no silent failures).

**Prevention:** (1) A single shared `is_content_msg_type(&Value)` predicate now
backs the 4 fixed sites — a future migration changes one function. (2) Each tool's
loop was extracted into a pure `compute_*` helper with a load-bearing unit test
that feeds `note`+`chat` envelopes and asserts they surface; each was proven
load-bearing by temp-reverting the predicate to post-only (all 5 tests FAIL) and
restoring. (3) The remaining ~23 `!= Some("post")` sites (other agent_* tools) are
filed as a follow-up for per-tool audit — the class is broader than the 4 named
here, and some (e.g. `presence_now`, which walks heartbeat envelopes) need
different semantics, not a blind predicate swap.

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

### 2026-08-09T22:38:02Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2588-fix-4-sibling-agent-analytics-tools-shar.md
- **Context:** Initial task creation

### 2026-08-10T18:32:52Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-f57f5284
- **Timestamp:** 2026-08-10T18:43:33Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-10T18:43:11Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
