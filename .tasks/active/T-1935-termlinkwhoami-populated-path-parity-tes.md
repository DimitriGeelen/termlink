---
id: T-1935
name: "termlink_whoami populated-path parity test — registered-session happy path (T-1933 follow-up)"
description: >
  T-1933 shipped MCP whoami + empty-state parity test. The populated path (one registered session, query via name_hint) is the LLM agent's actual production flow and is currently un-tested. Add parity_whoami_session_match: register a session, both MCP and CLI whoami with name_hint return matching identity card. Surface and document any residual shape drift (posts_as.from_project in particular).

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-02T22:00:31Z
last_update: 2026-06-02T22:00:31Z
date_finished: null
---

# T-1935: termlink_whoami populated-path parity test — registered-session happy path (T-1933 follow-up)

## Context

T-1933 shipped `termlink_whoami` MCP + a `parity_whoami_no_sessions` empty-
state test. The empty path is the safe edge — it confirms both sides agree
when nothing is registered. But an LLM agent's actual production flow goes
through the **populated path**: at least one session is registered (the
agent itself), the caller supplies a name_hint, and expects a full identity
card back.

This slice locks the populated path. Register one session in TestDir, ask
both MCP `termlink_whoami` (with `name_hint`) and CLI `termlink whoami
--name` for the identity card, and diff the envelopes. Per-process fields
(id, pid, uid, capabilities, timestamps, socket_path, cwd, identity FP)
are run-variant and must be in the diff ignore-list — the structural shape
+ display_name + state are what we lock.

If `posts_as.from_project` divergence surfaces (T-1933 v1 omitted it because
the MCP doesn't share the CLI's `resolve_project_name_from` resolver), this
slice will detect it. Resolution: either ignore the key (acceptable — it's
an optional CLI-side enrichment) or extract the resolver into a shared
module (separate slice). Document the choice in Decisions.

## Acceptance Criteria

### Agent
- [ ] New test `parity_whoami_session_match` in tests/parity.rs registers one session via `start_session`, calls MCP `termlink_whoami({name_hint: "<display>"})` AND CLI `whoami --name <display> --json`
- [ ] Both responses asserted `ok=true` + non-null `session.display_name` matching the registered name
- [ ] `diff_json` passes after stripping per-process / wall-clock / per-host fields
- [ ] If `posts_as` differs, decision recorded in `## Decisions` (ignore vs extract) — defer extraction to a follow-up task
- [ ] Full parity suite passes (grows from 23 to 24)
- [ ] `cargo build --release -p termlink-mcp` remains warning-free

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

### 2026-06-02T22:00:31Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1935-termlinkwhoami-populated-path-parity-tes.md
- **Context:** Initial task creation
