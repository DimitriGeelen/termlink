---
id: T-1618
name: "fleet doctor JSON: include action_items rollup (parity with text output)"
description: >
  fleet doctor JSON: include action_items rollup (parity with text output)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-06T11:43:41Z
last_update: 2026-05-06T11:43:41Z
date_finished: null
---

# T-1618: fleet doctor JSON: include action_items rollup (parity with text output)

## Context

T-1617 added an `Action items:` rollup to fleet doctor's text output. Verified live: `fleet doctor --json` does NOT include this in the JSON output. JSON consumers (CI scripts, monitoring dashboards) miss the actionable summary that text-output operators see. Lift the action-items computation up before json_doc construction so both paths share it.

## Acceptance Criteria

### Agent
- [ ] Action-items computation lifted out of the `else !json` branch into shared scope before json_doc construction
- [ ] `json_doc` includes top-level `action_items` array (Vec<String>)
- [ ] Text output behavior unchanged (still prints `Action items:` block)
- [ ] `target/release/termlink fleet doctor --json | jq .action_items` returns the same items as text output
- [ ] When fleet is clean (no stale, no fail), `action_items` is `[]` (empty array, not missing)
- [ ] Build clean

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

test -x target/release/termlink
target/release/termlink fleet doctor --json 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); assert 'action_items' in d, 'action_items missing'; assert isinstance(d['action_items'], list), 'not a list'; print('action_items present, len=', len(d['action_items']))"

## Recommendation

**Recommendation:** GO (text/JSON parity, small refactor).
**Rationale:** JSON consumers were missing the rollup; this closes the parity gap.

# Toolchain hint (L-291): if you edited *.vbproj/*.csproj/*.xaml add `dotnet build`;
# *.go → `go build ./...`; Cargo.toml → `cargo check`; tsconfig.json → `tsc --noEmit`;
# pom.xml → `mvn -q compile`. P-011 runs only what you write — broken builds slip
# past otherwise (origin: 003-NTB-ATC-Plugin T-077, broken WPF DLL on master 5 days).

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

## Updates

### 2026-05-06T11:43:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1618-fleet-doctor-json-include-actionitems-ro.md
- **Context:** Initial task creation
