---
id: T-1933
name: "Add termlink_whoami MCP tool — identity discovery for LLM consumers (MCP-arc parity gap)"
description: >
  CLI termlink whoami exists (metadata.rs:529 cmd_whoami); MCP has no equivalent. LLM agents calling MCP cannot answer 'who am I?'. Add termlink_whoami with same resolution chain (explicit session/name > TERMLINK_SESSION_ID env > PID-walk fallback > candidate list) and identical JSON shape. v1 copies PID-walk helpers from CLI; future task extracts to shared module.

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: []
related_tasks: []
created: 2026-06-02T19:26:27Z
last_update: 2026-06-02T19:28:15Z
date_finished: 2026-06-02T20:40:47Z
---

# T-1933: Add termlink_whoami MCP tool — identity discovery for LLM consumers (MCP-arc parity gap)

## Context

The CLI `termlink whoami` (metadata.rs:529, T-1297/T-1299/T-1303/T-1440/T-1704)
gives an operator a complete identity card: session id, display_name, state,
pid, uid, roles, tags, capabilities, cwd, identity_fingerprint,
identity_shared_with, posts_as.from_project, plus a PID-walk fallback that
finds the caller's registered session by ancestor chain when no env var hint
is set. The MCP surface has `termlink_list_sessions` (whole inventory) and
`termlink_status` (one specific session by name) but **nothing that answers
"which session is the caller?"** — exactly the question an LLM agent asks
first when it spawns the MCP server as a subprocess.

PL-172 is the precedent: MCP/CLI feature parity gaps cause silent value
strip — the operator sees rich CLI behavior, the LLM agent sees nothing.
This slice closes the whoami gap.

v1 scope: in-process implementation. Resolution chain matches CLI:
1. `session_hint` param (explicit) → `manager::find_session`
2. `name_hint` param → `manager::find_session`
3. `TERMLINK_SESSION_ID` env var → `manager::find_session`
4. PID-walk fallback (Linux /proc/<pid>/stat) → first ancestor pid that
   matches a registered session
5. Ambiguous → return candidate list with hint
6. Empty → return `{ok: false, ambiguous: false, candidates: []}`

PID-walk helpers (`walk_ancestor_pids`, `read_ppid_from_proc`,
`parse_ppid_from_stat`) are copied from `termlink-cli/src/commands/metadata.rs`
into a small `whoami_helpers` module in termlink-mcp. Future task can
extract to a shared crate; for v1, duplication is the cheaper path.

JSON envelope identical to CLI's `whoami_card_json` so the new
parity test diffs cleanly.

## Acceptance Criteria

### Agent
- [x] `termlink_whoami` MCP tool registered with `WhoamiParams { session_hint, name_hint }` — tools.rs:4571-4577 (params) + tools.rs:7920+ (tool impl)
- [x] Resolution chain matches CLI: session_hint → name_hint → TERMLINK_SESSION_ID env → PID-walk → candidate list → empty — tools.rs:7925-7990 (mirrors metadata.rs:529-627)
- [x] JSON envelope structurally identical to CLI `whoami --json` — `whoami_card_json` in tools.rs (post-helpers module) mirrors metadata.rs:655; parity test diff_json passes with empty ignore set
- [x] Help registry updated — tools.rs:10870 lists `termlink_whoami` under "session" category
- [x] Parity test added — tests/parity.rs `parity_whoami_no_sessions` (PAIR 20)
- [x] `cargo build --release -p termlink-mcp` remains warning-free — verified clean build, zero warnings
- [x] Full parity suite passes (grows from 20 to 21) — `test result: ok. 21 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 489.29s`

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
! cargo build --release -p termlink-mcp 2>&1 | grep -q "warning:"
cargo test --release -p termlink-mcp --test parity 2>&1 | grep -qE "test result: ok\."

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

### 2026-06-02T19:26:27Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1933-add-termlinkwhoami-mcp-tool--identity-di.md
- **Context:** Initial task creation
