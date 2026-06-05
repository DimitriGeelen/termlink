---
id: T-2008
name: "termlink help: honor --json on positional-conflict error (cycle 13 slice 5)"
description: >
  termlink help: honor --json on positional-conflict error (cycle 13 slice 5)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-06-05T20:39:19Z
last_update: 2026-06-05T20:52:19Z
date_finished: null
---

# T-2008: termlink help: honor --json on positional-conflict error (cycle 13 slice 5)

## Context

Cycle 13 (T-2002→T-2006) brought `termlink help` CLI to parity with the MCP
`termlink_help` registry envelope, added positional-arg routing, 3-tier dispatch,
and did_you_mean. One operator-script gap remains: when a positional `<target>`
conflicts with an explicit `--tool-detail` / `--name-filter` / `--category` flag,
the CLI emits a plain stderr line and exits 2 — **even when `--json` is set**.
Scripts wrapping `termlink help --json ...` get nothing on stdout to parse.

This is the same class of gap that T-1914/T-1915/T-1917 closed for hub-down
commands (cmd_channel_list, events.rs, cmd_net_test). Help wasn't covered
because it never contacts a hub; the early `std::process::exit(2)` at
`crates/termlink-cli/src/commands/help.rs:113-115` predates the --json branch.

Fix: when `--json` is set, emit a machine-readable error envelope on stdout
in the established T-1914 shape (`{"ok": false, "error": "...", "verdict": "error"}`)
before exiting 2. Plain-text path stays for non-JSON callers.

## Acceptance Criteria

### Agent
- [x] **Bug reproduced at HEAD before fix:** `termlink help --tool-detail foo --json bogus` emits plain stderr text and exits 2, with EMPTY stdout (the regression).
- [x] **Fix:** `crates/termlink-cli/src/commands/help.rs::run` emits a JSON error envelope on stdout when `inv.json` is true AND `resolve_positional` returned `Err`. Envelope shape matches established T-1914 / execution.rs / identity.rs pattern: `{"ok": false, "error": "<msg>"}`. Exit code stays 2 (usage error, not operational error — distinct from `json_error_exit`'s exit 1).
- [x] **Plain path unchanged:** when `--json` is NOT set, behavior is byte-identical to before (stderr line + exit 2) — verified by existing tests `positional_with_explicit_tool_detail_errors`, `positional_with_explicit_name_filter_errors`, `positional_with_explicit_category_errors` still pass.
- [x] **New tests, 3 cases (one per conflict flag), each:** stdout contains valid JSON with `ok=false`, `error` field references the conflicting flag name; exit code 2; stderr is silent in JSON mode (no double-emit). Tests live under existing `mod tests` in `commands/help.rs` and exercise the routing function plus envelope serialization (no subprocess).
- [x] **Full CLI test suite passes** at >=817 (current 814 + 3 new). Single-test isolation run for any pre-existing flake (`isolate_rejects_non_git_dir` per session memory) acceptable as long as the JSON-error tests pass in the suite.
- [x] **Live smoke after release build:** `target/release/termlink help --tool-detail foo --json bogus 2>/dev/null | jq -e '.ok == false and .verdict == "error"'` exits 0, AND `2>&1 1>/dev/null` is empty (no stderr in JSON mode).

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

cargo test -p termlink --bins commands::help::tests --quiet
test -x target/release/termlink
target/release/termlink help --tool-detail foo --json bogus 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['ok'] is False and '--tool-detail' in d['error'], d"
target/release/termlink help --name-filter foo --json bogus 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['ok'] is False and '--name-filter' in d['error'], d"
target/release/termlink help --category channel --json bogus 2>/dev/null | python3 -c "import sys,json; d=json.load(sys.stdin); assert d['ok'] is False and '--category' in d['error'], d"

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

**Recommendation:** GO — ship the cycle-13 slice 5.

**Rationale:** Pure additive parity fix completing the T-1914 / T-1915 / T-1917
--json error-path family for the CLI help surface (which was correctly skipped
by the prior audit because it doesn't contact a hub — different exit reason,
same operator-script gap). Existing plain-text behavior preserved byte-for-byte;
JSON callers now get a stable `{ok:false, error:<msg>}` envelope on stdout
identical in shape to execution.rs/identity.rs/channel.rs error envelopes.
Zero risk to non-script operators; closes a discoverability/usability gap for
agents wrapping `termlink help --json ...`.

**Evidence:**
- Unit tests: 15/15 help-mod tests pass (12 pre-T-2008 + 3 new envelope tests)
- Full CLI suite: 816/0 in suite + 1 pre-existing flake `isolate_rejects_non_git_dir` (PASSES solo, known per session memory)
- Live smoke (all 3 conflict paths × JSON-mode): stdout = valid JSON envelope, stderr = 0 bytes, exit 2 — verified post-release-build
- Live smoke (plain-text path regression): stdout empty, stderr unchanged, exit 2 — non-JSON callers unaffected

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

### 2026-06-05T20:39:19Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-2008-termlink-help-honor---json-on-positional.md
- **Context:** Initial task creation
