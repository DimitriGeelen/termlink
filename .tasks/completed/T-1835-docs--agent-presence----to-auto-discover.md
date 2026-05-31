---
id: T-1835
name: "docs — agent-presence + --to auto-discover (T-1830 adoption recipe in agent-conversations.md)"
description: >
  docs — agent-presence + --to auto-discover (T-1830 adoption recipe in agent-conversations.md)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-28T13:07:43Z
last_update: 2026-05-28T13:09:37Z
date_finished: 2026-05-28T13:09:37Z
---

# T-1835: docs — agent-presence + --to auto-discover (T-1830 adoption recipe in agent-conversations.md)

## Context

T-1830 sub-builds (a/b/c) shipped: heartbeat emitter (T-1832), discovery reader (T-1833), agent-send --to auto-discover (T-1834). The verbs exist but adoption requires that operators and LLM-driven agents know the convention. This task adds a section to docs/operations/agent-conversations.md giving a copy-pasteable recipe for both the listener side (heartbeat) and the sender side (--to).

## Acceptance Criteria

### Agent
- [x] `docs/operations/agent-conversations.md` gets a new section titled "Establishing presence + auto-discover (T-1830 — T-1832/T-1833/T-1834)" inserted BEFORE the "## Limits and next steps" section
- [x] Section covers: (a) listener side — heartbeat with canonical metadata; (b) discovery side — enumerate listeners; (c) sender side — `agent-send.sh --to <agent-id>` resolution
- [x] Includes a TTL convention table (LIVE/STALE/OFFLINE thresholds)
- [x] Includes a worked example: one-line heartbeat + one-line auto-discover send
- [x] Cross-references the three scripts by path
- [x] Records what's deliberately NOT done yet (no MCP parity for the new verbs; no cross-hub discovery merge)

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
grep -q "Establishing presence + auto-discover" docs/operations/agent-conversations.md
grep -q "TTL convention" docs/operations/agent-conversations.md
test -z "$(grep -E '\[TODO\]' docs/operations/agent-conversations.md || true)"

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

### 2026-05-28T13:07:43Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1835-docs--agent-presence----to-auto-discover.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-751dff03
- **Timestamp:** 2026-05-28T13:09:37Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** no
- **Findings:** none

### 2026-05-28T13:09:37Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
