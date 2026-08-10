---
id: T-2592
name: "chat-arc-recent default filter_msg_type=chat hides note/post content"
description: >
  scripts/agent-chat-arc-recent.sh defaults FILTER_MSG_TYPE='chat' and applies it as single-value equality (.msg_type == $mtype), so /recent-chat and the MCP wrapper termlink_agent_chat_arc_recent (default filter_msg_type='chat') default to showing ONLY chat-type posts — hiding all 'note' content (termlink_agent_post/agent_reply write note) and 'post' content from the default 'what's been said?' view. Same PL-316/T-2591 content-set class. The --all-msg-types escape hatch exists but disables the filter ENTIRELY (includes heartbeats/reactions/meta noise), so there is NO clean 'all content' view. FILED not auto-built (T-2468 boundary): changes the DEFAULT of a user-facing viewer AND a wire-facing MCP tool, and requires converting single-value equality to content-set membership {post,chat,note} while preserving --filter-msg-type <X> single-type narrowing and --all-msg-types (everything incl meta). Design: default=content-set, --filter-msg-type X = narrow to one type, --all-msg-types = no filter. Load-bearing test: a note post appears in default output. Touch both shell script AND tools.rs MCP wrapper default + its JSON tool schema/description. From T-2468 verb-2/reliability hunt (hunter LIKELY-BUG).

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
created: 2026-08-10T19:36:32Z
last_update: 2026-08-10T20:02:30Z
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

# T-2592: chat-arc-recent default filter_msg_type=chat hides note/post content

## Context

`scripts/agent-chat-arc-recent.sh` (the engine behind `/recent-chat` and the MCP
tool `termlink_agent_chat_arc_recent`) defaulted `FILTER_MSG_TYPE="chat"` and
applied it as single-value equality `.msg_type == $mtype`, so the default
"what's been said?" view showed ONLY `chat`-type posts — silently hiding every
`note` post (the type `termlink_agent_post` / `agent_reply` write) and every
legacy `post`. Same PL-316 / T-2591 content-set class (content = {post,chat,note}).
The only escape hatch, `--all-msg-types`, disables the filter ENTIRELY (pulls in
receipts / heartbeats / reactions / meta), so there was no clean "all content"
default. Fix: make the DEFAULT the content-set membership `{post,chat,note}`;
keep `--filter-msg-type X` as single-type narrowing; keep `--all-msg-types` as
everything-incl-meta. Empty `FILTER_MSG_TYPE` is the "default / content-set"
sentinel (mirrors the pattern `recent-dm.sh` already uses at its line 110).

## Acceptance Criteria

### Agent
- [ ] Engine default (no `--filter-msg-type`, no `--all-msg-types`) includes `note`, `chat`, AND `post` envelopes and excludes meta (e.g. `reaction`, `receipt`).
- [ ] `--filter-msg-type note` narrows to only `note` posts (single-type narrowing preserved).
- [ ] `--all-msg-types` still disables the filter entirely (meta included).
- [ ] MCP wrapper (`termlink_agent_chat_arc_recent`) default view shows `note`/`post`/`chat` — i.e. it inherits the fixed shell default (passthrough already only pushes `--filter-msg-type` when the caller supplies it); its param doc + tool description no longer claim `default 'chat'`.
- [ ] Load-bearing test (`tests/chat-arc-recent-fixtures.sh`) asserts a `note` post appears in default output and FAILS against the old `chat`-only default (temp-revert proven).
- [ ] `recent-dm.sh` behavior unchanged (it always passes an explicit `--filter-msg-type` or `--all-msg-types`, so the engine default is not on its path).

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
bash tests/chat-arc-recent-fixtures.sh
grep -q 'FILTER_MSG_TYPE=""' scripts/agent-chat-arc-recent.sh
grep -q 'msg_type == "post" or .msg_type == "chat" or .msg_type == "note"' scripts/agent-chat-arc-recent.sh
cargo check -p termlink-mcp

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

**Symptom:** `/recent-chat` and `termlink_agent_chat_arc_recent` default view
showed only `chat`-type posts; every agent that broadcasts via
`termlink_agent_post` / `agent_reply` (which write `note`) was invisible in the
default "what's been said?" tail — indistinguishable from silence.

**Root cause:** `FILTER_MSG_TYPE="chat"` default + single-value equality
`.msg_type == $mtype`. A single hardcoded content-type is not the content SET;
`note` and legacy `post` are equally content, so they were silently dropped.

**Why structurally allowed:** the PL-316 content-filter class was swept in Rust
(`tools.rs`, T-2588/T-2590) and in `fleet-adoption-snapshot.sh` (T-2591), but the
chat-arc-recent engine — the single most-used read path — re-hardcoded its own
single-value default independently, with no shared cross-language predicate
(PL-318). A silent-drop content filter reads identical to an empty topic →
violates the "no silent failures" reliability directive.

**Prevention:** `tests/chat-arc-recent-fixtures.sh` asserts a `note` post appears
in default output (load-bearing: FAILS against the `chat`-only default). Plus the
PL-318 learning already records that a one-language class fix leaves same-class
shell siblings unaudited — this task closes the last known live instance.

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

### 2026-08-10T19:36:32Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2592-chat-arc-recent-default-filtermsgtypecha.md
- **Context:** Initial task creation

### 2026-08-10T20:02:30Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
