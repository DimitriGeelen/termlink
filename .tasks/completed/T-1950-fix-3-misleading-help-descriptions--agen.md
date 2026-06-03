---
id: T-1950
name: "Fix 3 misleading help descriptions — agent_digest/agent_peers/remote_doctor (LLM-mispick bug)"
description: >
  Help registry descriptions inaccurately describe tool semantics; LLMs would pick wrong tool

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-03T20:35:19Z
last_update: 2026-06-03T20:37:21Z
date_finished: null
---

# T-1950: Fix 3 misleading help descriptions — agent_digest/agent_peers/remote_doctor (LLM-mispick bug)

## Context

A cross-check between `help_categories()` one-liners and the actual
`#[tool(description=…)]` macro text surfaced 3 entries where the help
description contradicts the actual tool semantics. An LLM choosing a tool
from `termlink_help` would pick the wrong one and get unexpected behavior.

**Bug 1 — `termlink_agent_digest`** (worst):
- HELP: "Single-shot fleet digest (combines presence + recent)"
- TRUTH: Period summary of agent-chat-arc with `by_msg_type`, `top_senders`,
  `latest_5_offsets` over a time window
- Mispick: An LLM wanting fleet presence info would pick this and get
  a topic-period summary instead. The actual fleet digest is
  `termlink_agent_overview`.

**Bug 2 — `termlink_agent_peers`**:
- HELP: "All peers known to the fleet"
- TRUTH: Participant directory for agent-chat-arc (single topic, post-based)
- Mispick: An LLM wanting full fleet presence would pick this and get
  only senders who have posted to chat-arc — missing silent peers.

**Bug 3 — `termlink_remote_doctor`**:
- HELP: "Health check on a remote hub (cross-host fleet doctor)"
- TRUTH: Health check on ONE remote hub (single-target)
- Mispick: Mild — the phrase "fleet doctor" implies sweeping the fleet,
  but it's single-hub.

Fix: rewrite each help description to accurately reflect tool semantics,
sourced from the macro `description = …` text. Bounded — 3 lines.

## Acceptance Criteria

### Agent
- [x] `termlink_agent_digest` help description rewritten to describe period-summary semantics (not "fleet digest")
  - Evidence: now reads `"Period summary of chat-arc (by_msg_type, top_senders, latest offsets)"` — matches macro `description = "Period summary on agent-chat-arc..."`. Commit `dbd36a67`
- [x] `termlink_agent_peers` help description rewritten to describe chat-arc participant directory (not "fleet")
  - Evidence: now reads `"Chat-arc participant directory (senders who have posted)"` — matches macro `description = "Participant directory for agent-chat-arc..."`. Commit `dbd36a67`
- [x] `termlink_remote_doctor` help description rewritten to drop the misleading "fleet doctor" phrasing
  - Evidence: now reads `"Health check on one remote hub (connectivity/sessions/inbox)"` — matches macro `description = "Health check a remote hub — connectivity, sessions, inbox status..."`. Commit `dbd36a67`
- [x] `cargo test -p termlink-mcp --lib` still passes 682 (no regression)
  - Evidence: `test result: ok. 682 passed; 0 failed; 0 ignored; 0 measured` — phantom/coverage/list_categories/unknown-cat invariants all still hold
- [x] All help/full cross-check mismatches found by the audit query are resolved (re-run the awk pipeline → 0 matches except the acceptable `termlink_fleet_status` case)
  - Evidence: re-running the keyword-mismatch awk pipeline now surfaces only `termlink_fleet_status` (description says "all configured hubs" which IS the fleet — acceptable). 3 previous mismatches no longer appear

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
cargo test -p termlink-mcp --lib help_ -- --nocapture
# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

## RCA

**Symptom:** An LLM consumer reading `termlink_help` sees descriptions
that contradict the actual tool semantics. Picking the named tool yields
unexpected output shape and behavior. 3 confirmed cases:
agent_digest claims "fleet digest" but returns single-topic period
summary; agent_peers claims "all peers" but returns chat-arc participants
only; remote_doctor claims "cross-host fleet doctor" but is single-hub.

**Root cause:** The help registry one-liners were authored at T-1942
(Tier-1 operator-essentials slice) before the corresponding tool
implementations were closely read. Some were written from the tool name
alone, leading to plausible-but-wrong summaries.

**Why structurally allowed:** The phantom-guard (T-1941) and coverage-guard
(T-1946) ensure names match — they say nothing about whether the
description is semantically true. The `name_filter` substring match
makes shallow textual error fade into the background — search "fleet"
returns hits, even when the hits don't actually do "fleet" work.

**Prevention:** No automated guard added by this task — semantic
truthfulness is hard to test without per-tool ground truth. Mitigation:
the cross-check pattern documented in this task's Context section is
the audit pipeline. Future help-registry additions should run the
keyword-mismatch awk pipeline before commit. A heavier guard (e.g.,
require help description to share ≥2 content words with the macro
description) is a future task if drift recurs.

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

### 2026-06-03T20:35:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1950-fix-3-misleading-help-descriptions--agen.md
- **Context:** Initial task creation
