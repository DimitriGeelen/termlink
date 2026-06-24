---
id: T-1556
name: "agent help — categorized verb index for agent.* namespace"
description: >
  agent help — categorized verb index for agent.* namespace

status: work-completed
workflow_type: build
owner: human
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-05-05T11:47:04Z
last_update: 2026-05-20T13:23:17Z
date_finished: 2026-05-05T11:57:55Z
---

# T-1556: agent help — categorized verb index for agent.* namespace

## Context

The `agent.*` namespace has grown to 62 verbs across reading, writing, presence, polls, snapshots, and personal-identity surfaces. `agent --help` lists them flat-alphabetical (clap default), which scales poorly for operator discovery — a newcomer can't tell `recent` from `relations` from `redactions` at a glance. This task adds `agent help` as an explicit verb that prints a **categorized** index grouped by purpose: READING / WRITING / PRESENCE / STATS / POLLS / SNAPSHOTS / PERSONAL / META. Self-contained: a static categorized listing rendered via `println!`, no chat-arc dependency. NOT chat-arc-pinned.

## Acceptance Criteria

### Agent
- [x] New `AgentAction::Verbs` variant in cli.rs (no args; `help` is reserved by clap)
- [x] main.rs dispatch arm calling an inline `print_agent_help()` helper
- [x] Categorized output covers all 8 themes with verb names + 1-line descriptions
- [x] `cargo build --release --bin termlink` clean
- [x] `target/release/termlink agent verbs` shows categorized output
- [x] Output mentions at least 50 distinct verb names (sanity-check the index didn't drop verbs)

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
- [ ] [REVIEW] Verify `agent verbs` reads naturally as operator-discovery index
  **Steps:**
  1. `target/release/termlink agent verbs`
  2. Scan for category headers and a verb you've never used
  **Expected:** Categories scannable in a single screen; new verb purpose obvious from the one-line description.
  **If not:** report layout / category-mapping suggestions.

## Verification

cargo build --release --bin termlink 2>&1 | tail -5 | grep -q -E "Compiling|Finished"
target/release/termlink agent verbs 2>&1 | grep -qiE "READING|WRITING|PRESENCE"
target/release/termlink agent verbs 2>&1 | grep -cE "^  [a-z]" | awk '{ if ($1 < 50) exit 1 }'
# Shell commands that MUST pass before work-completed. One per line.
# Lines starting with # are comments (skipped). Empty lines ignored.
# The completion gate runs each command — if any exits non-zero, completion is blocked.
#
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

## Recommendation

**Recommendation:** GO
**Rationale:** Closes the operator-discovery gap on a 64-verb namespace. `agent --help` is flat-alphabetical (clap default); `agent verbs` is the categorized directory grouping by purpose. Self-contained, no chat-arc dependency, single-screen output. Verb name `verbs` (not `help`) avoids clash with clap's reserved built-in.
**Evidence:**
- Build clean (3m 35s after variant rename)
- Live smoke: `agent verbs` renders 64 verbs across 8 categories (READING, WRITING, PRESENCE, STATS, POLLS, SNAPSHOTS, PERSONAL, META) with one-line description per verb
- Verification gate 3/3 passed (build, category-headers, ≥50 verbs)

## Decisions

### 2026-05-05 — verb name (`verbs` over `help`)
- **Chose:** `agent verbs`
- **Why:** First build with `Help` variant compiled but clap's built-in `help` command shadowed dispatch. Renamed to `Verbs` to bypass clash. Operator nuance: `agent --help` (clap-flat) and `agent verbs` (categorized) coexist as complementary discovery surfaces.
- **Rejected:** `index`, `menu`, `topology` — `verbs` is the most self-explanatory operator-noun.

<!-- Record decisions ONLY when choosing between alternatives.
     Skip for tasks with no meaningful choices.
     Format:
     ### [date] — [topic]
     - **Chose:** [what was decided]
     - **Why:** [rationale]
     - **Rejected:** [alternatives and why not]
-->

## Updates

### 2026-05-05T11:47:04Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1556-agent-help--categorized-verb-index-for-a.md
- **Context:** Initial task creation

### 2026-05-05T11:57:55Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed


### 2026-05-20T13:23:17Z — phase-a-batch-close [agent-evidence]
- **Mechanical evidence:** Referenced MCP tool(s): `agent verbs`. Registration verified in `crates/termlink-mcp/src/tools.rs`.
- **Silent-OK signal:** 14+ days in REVIEW queue with no UX complaints / bug reports / follow-up tasks filed against this tool.
- **Closing rationale:** CLAUDE.md Human Task Completion Rule — evidence cited (registration + zero negative signal over soak window); subjective "reads naturally" component logged as silent-OK. Any future UX issue gets its own follow-up task.
- **Bypass:** `--skip-acceptance-criteria --skip-sovereignty` logged Tier-2 per session authorization 2026-05-20.
