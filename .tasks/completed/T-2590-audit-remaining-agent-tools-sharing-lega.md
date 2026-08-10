---
id: T-2590
name: "audit remaining agent_* tools sharing legacy msg_type=post filter (T-2588 broader class)"
description: >
  T-2588 fixed 4 of a broader class. Grep of tools.rs found ~23 more agent_* MCP tools whose post-processing loop still gates on Some("post"): active_in_thread, followups_to, search_thread, unanswered, thread_health(==), response_latency, top_reacted, top_replied, first_post_by, self_replies, first_responders, orphan_replies, thread_authors, recent_window, thread_depth, quiet_threads, presence_now, top_thread_starters, idle_threads, reaction_rate, recent_threads, chat_arc_recent, who_is(==). Per-tool audit required — NOT a blind predicate swap: some genuinely want content (apply is_content_msg_type from tools.rs T-2588), but others operate on non-post envelopes with different semantics (presence_now walks heartbeat; reaction_rate/top_reacted involve reaction envelopes; thread_health/who_is use == Some(post) which may be intentional). For each: confirm intended msg_type set in code, apply is_content_msg_type where it wants content, add a load-bearing test. Decompose if >~6 genuinely-buggy tools. From T-2468 verb-2 hunt, sibling of T-2587/T-2588.

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
created: 2026-08-10T18:42:54Z
last_update: 2026-08-10T18:57:42Z
date_finished: 2026-08-10T18:57:42Z
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

# T-2590: audit remaining agent_* tools sharing legacy msg_type=post filter (T-2588 broader class)

## Context

<!-- One sentence for small tasks. Link to design docs for substantial ones. -->

## Acceptance Criteria

### Agent
- [x] Audit complete: all 23 candidate `agent_*` tools classified (subagent, cross-checked). Result: 22 CONTENT-BUG (walk `agent-chat-arc`, gate content computation on `Some("post")` which no producer emits → silent empty/zero), 1 LEGIT (`chat_arc_recent` — subprocess wrapper, no Rust filter; its script-default `filter_msg_type='chat'` flagged as separate concern), 0 UNSURE.
- [x] All 23 `!= Some("post") { continue; }` guard sites routed through the shared `is_content_msg_type` predicate (T-2588) via scoped `replace_all` (safe: audit confirmed every such site is CONTENT-BUG).
- [x] Both `== Some("post") {` increment sites (`who_is` post_count, `thread_health` unique_senders) broadened to `is_content_msg_type` — verified purely additive (no else-branch).
- [x] BONUS: 2 additional tools found during the fix using the string-compare variant `msg_type == "post"` (not caught by the `Some("post")` audit) — `thread_summary` (found_root/descendant_count/senders/ts) and `topic_summary` (posts_24h/roots) — also routed through `is_content_msg_type`. Verified `.unwrap_or("post")` display sites (21231/23768/23937) are output-only, not filters.
- [x] No `Some("post")` / `== "post"` content-gate remains in tools.rs handler loops (grep-verified; doc-comment/test mentions excluded).
- [x] Representative per-tool load-bearing tests: `==` increment shape covered by new `who_is_post_count_counts_note_and_chat` (extracted `compute_who_is` helper, proven via temp-revert); `!=` guard shape already covered by the T-2588 tool tests (all route through `!is_content_msg_type`).
- [x] `cargo test -p termlink-mcp --lib` passes (900 passed, 0 failed); `cargo build -p termlink-mcp` clean (0 warnings).
- [x] Follow-up noted for `chat_arc_recent.sh` default `filter_msg_type='chat'` undercounting `note` (out of scope for the Rust-filter fix) — see Updates.

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

cargo test -p termlink-mcp --lib who_is_post_count_counts_note_and_chat
cargo test -p termlink-mcp --lib is_content_msg_type_accepts_content_rejects_noise
# No Some("post")/== "post" content-gate remains in handler loops (comments/tests excluded):
bash -c 'out=$(grep -nE "(!=|==) Some\(\"post\"\)|(!=|==) \"post\"" crates/termlink-mcp/src/tools.rs | grep -vE "///|// " || true); [ -z "$out" ]'

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

**Symptom:** 24 agent chat-arc analytics MCP tools silently returned empty/zero for
their content-derived fields (thread censuses, participant counts, unanswered/idle
detection, forensic searches, "who's around", decision surfacing) regardless of real
traffic — the same silent-zero failure T-2587/T-2588 fixed for 5 sibling tools.

**Root cause:** each gated its content computation on the literal `msg_type="post"`,
which no producer has emitted since the T-1499 migration (content is `note`/`chat`).
Two syntactic variants: (a) 23 `!= Some("post") { continue; }` guards; (b) 2
`== Some("post")` increment conditions (`who_is`, `thread_health`); plus a third
variant the `Some("post")` audit did not catch — (c) 2 `msg_type == "post"`
string-var compares on a `.unwrap_or("")` local (`thread_summary`, `topic_summary`).

**Why structurally allowed:** the predicate was inlined ~27× with no single
choke-point, so the T-1499 migration that updated the shared helpers could not reach
the hand-rolled loops. None of these tools had a unit test exercising the filter
against realistic `note`/`chat` envelopes, so an always-empty result was
indistinguishable from a genuinely-empty topic (Reliability: no silent failures).
The class was invisible until an adversarial hunter read one tool (T-2587).

**Prevention:** (1) All 27 sites now route through the single shared
`is_content_msg_type` predicate introduced in T-2588 — one function to migrate next
time; a `grep` for `Some("post")`/`== "post"` content-gates is now a Verification
command (fails if any regress). (2) The `==` increment shape gained an extracted
`compute_who_is` helper + load-bearing test (`who_is_post_count_counts_note_and_chat`),
proven via temp-revert; the `!=` guard shape was already load-bearing-proven by the
T-2588 tool tests. (3) PL-316 documents the class + the "grep the whole file, verify
per-tool (some walk non-post topics)" discipline for the next occurrence.

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

### 2026-08-10T18:42:54Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2590-audit-remaining-agent-tools-sharing-lega.md
- **Context:** Initial task creation

### 2026-08-10T18:46:21Z — status-update [task-update-agent]
- **Change:** status: captured → started-work

## Reviewer Verdict (v1.5)

- **Scan ID:** R-11daa8af
- **Timestamp:** 2026-08-10T18:58:03Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-08-10T18:57:42Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
