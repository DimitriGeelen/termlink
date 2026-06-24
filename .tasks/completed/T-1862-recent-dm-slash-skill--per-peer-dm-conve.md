---
id: T-1862
name: "/recent-dm slash skill — per-peer DM conversation history (T-1830 doorbell+mail arc read-side)"
description: >
  /recent-dm slash skill — per-peer DM conversation history (T-1830 doorbell+mail arc read-side)

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [scripts/agent-chat-arc-recent.sh]
related_tasks: []
created: 2026-05-29T10:13:42Z
last_update: 2026-05-29T10:22:57Z
date_finished: 2026-05-29T10:22:57Z
---

# T-1862: /recent-dm slash skill — per-peer DM conversation history (T-1830 doorbell+mail arc read-side)

## Context

The doorbell+mail discovery toolkit covers BROADCAST history (`/recent-chat`)
and UNREAD-DM inbox (`/check-arc`). Missing: a read-only "show me the conversation
history with peer X" verb. Closes the asymmetry — directly serves the directive's
"focus on the interactive doorbell+mail conversations arc" framing.

Canonical DM topic form (per agent-send.sh + check-arc.md): `dm:<a>:<b>` where
a,b are 8-byte (16-hex) short fingerprints, sorted lexicographically. Self
appears in either slot — bidirectional.

Two-part build:

1. `scripts/agent-chat-arc-recent.sh` — add `--topic <T>` flag (default
   `agent-chat-arc`). Single source-of-truth for envelope parsing; DM read
   becomes a parameterization, not a duplicate script.
2. `scripts/recent-dm.sh` — resolves self + peer agent_id → short fingerprints
   via agent-listeners-fleet.sh, computes sorted canonical topic, delegates
   to agent-chat-arc-recent.sh with `--topic <dm-topic>` + passthrough args.
3. `.claude/commands/recent-dm.md` — slash skill wrapping the script.

PL-176 caveat: DM topics may not federate either. The underlying fleet-wide
scan still surfaces what's on each hub — operator sees fragmentation.

## Acceptance Criteria

### Agent
- [x] `scripts/agent-chat-arc-recent.sh --topic <T>` flag accepts arbitrary topic name (default `agent-chat-arc`) — verified live: `--topic agent-chat-arc --limit 2 --since 168` returns 2 known posts; `--topic ""` rejected with exit 2
- [x] `scripts/recent-dm.sh <peer>` exists, has `--help`, exits 2 on usage error, 3 on setup error — verified live (no-arg call exits 2 "is required")
- [x] `recent-dm.sh` resolves self agent_id from `~/.termlink/be-reachable.state` (or `--self <id>` override) — implemented; falls back to "no self filter" if no state
- [x] `recent-dm.sh` discovers dm:* topics via substring-match on `<peer>` across every hub in hubs.toml — verified live: search `d1993c2c` returned 30 topics; search `alice-fp-test` returned 1 specific topic; search `no-such-peer-XYZ` returned 0 with hint
- [x] `recent-dm.sh` delegates to chat-arc-recent.sh `--topic <T>` per matched topic, dedups federated copies, merges chronologically — verified live: `--topic dm:9219671e28054458:d1993c2c3ec44c94 --since 720 --limit 8` returned 4 deduped posts (vs duplicates before dedup) spanning the real T-1166/T-450 cohort exchange
- [x] `.claude/commands/recent-dm.md` skill exists with $ARGUMENTS handling matching `/recent-chat` patterns — created at `.claude/commands/recent-dm.md`, mirrors recent-chat normalization (first positive int → --limit, second → --since, --flags passthrough, --topic special-case)
- [x] CLAUDE.md Quick Reference table includes a row for `/recent-dm` placed adjacent to `/recent-chat` — added between RECEIVE (`/check-arc`) and PRESENCE (`/be-reachable`) rows, labelled "DM history per peer (READ)"
- [x] Live demo: `/recent-dm <known-peer>` returns either real envelopes OR a clear "no DM topic between you and <peer>" message — both paths verified above

**Design note:** The original AC plan assumed deriving a single canonical `dm:<sorted-fp-a>:<sorted-fp-b>` topic from peer+self agent_ids. Live data inspection revealed the dm:* naming convention is MIXED (fp-pairs, name-pairs, mixed, plus the shared-host fingerprint pattern from `reference_shared_host_identity.md`). Pivoted to substring-match discovery — more robust, also surfaces the multi-host disparity per PL-176 instead of hiding it.

### Human
<!-- Criteria requiring human verification (UI/UX, subjective quality). Not blocking.
     Remove this section if all criteria are agent-verifiable.
     Each criterion MUST include Steps/Expected/If-not so the human can act without guessing.
     Optionally prefix with [RUBBER-STAMP] or [REVIEW] for prioritization.
     Example:
       - [ ] [REVIEW] Dashboard renders correctly
         **Steps:**
         1. Open https://example.com/dashboard in browser
         2. Verify all panels load within 2 seconds
         3. Check browser console for errors
         **Expected:** All panels visible, no console errors
         **If not:** Screenshot the broken panel and note the console error
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

test -x scripts/recent-dm.sh
test -f .claude/commands/recent-dm.md
bash scripts/agent-chat-arc-recent.sh --help 2>&1 | grep -q -- "--topic"
bash scripts/recent-dm.sh --help 2>&1 | grep -qi "recent-dm"
grep -q "/recent-dm" CLAUDE.md
grep -q "scripts/recent-dm.sh" .claude/commands/recent-dm.md

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

### 2026-05-29T10:13:42Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1862-recent-dm-slash-skill--per-peer-dm-conve.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-4074bf1f
- **Timestamp:** 2026-05-29T10:22:57Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#3 (Agent)** — `recent-dm.sh` resolves self agent_id from `~/.termlink/be-reachable.state` (or `--self <id>` override) — implemented; falls back to "no self filter" if no state
  - **AC-verify-mismatch** (narrow, heuristic) — `path=termlink/be-reachable.state in: `recent-dm.sh` resolves self agent_id from `~/.termlink/be-reachable.state` (or `--self <id>` override) — implemented; falls back to "no self filter" `

### 2026-05-29T10:22:57Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
