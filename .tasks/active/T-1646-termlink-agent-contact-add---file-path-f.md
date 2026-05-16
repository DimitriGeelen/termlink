---
id: T-1646
name: "termlink agent contact: add --file <path> for payload-from-file (T-1429 Phase-2 AC)"
description: >
  termlink agent contact: add --file <path> for payload-from-file (T-1429 Phase-2 AC)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-16T07:33:18Z
last_update: 2026-05-16T07:43:34Z
date_finished: null
---

# T-1646: termlink agent contact: add --file <path> for payload-from-file (T-1429 Phase-2 AC)

## Context

`termlink agent contact` currently requires `--message <STRING>` as a single CLI argument. For large structured handoffs (e.g. proposals, RCAs, multi-paragraph briefs) this is awkward — operators have to either heredoc + shell-substitute (which I did for T-1643 on agent-chat-arc: `--payload "$(cat /tmp/T-1643-proposal.txt)"`) or contend with shell-quoting hazards inline.

The peer command `channel post` already supports `--payload <STRING>` with "reads from stdin if not given". `agent contact` was authored without that ergonomics. T-1429 Phase-2 explicitly lists `--file <path>` as a deferred AC (see T-1429 task file: "Phase-2 (deferred): --file payload + metadata.subject=<message> when both flags supplied"). This task ships the path-only variant; the metadata.subject piece stays deferred since it depends on an open question about subject semantics.

Mutually exclusive with `--message`. Exactly one must be set.

## Acceptance Criteria

### Agent
- [x] cli.rs Contact variant adds `file: Option<PathBuf>` arg (`--file <path>`) and changes `message: String` to `message: Option<String>` — cli.rs:3387-3401
- [x] main.rs dispatcher resolves `--message`/`--file` into a single `String` before calling `cmd_agent_contact`; passing both errors with "specify exactly one of --message or --file, not both"; passing neither errors with "specify exactly one of --message <STRING> or --file <PATH>" — main.rs:266-269 + agent.rs `resolve_contact_message`
- [x] File I/O reads to a String via `std::fs::read_to_string` — path-only, no stdin support in this task — agent.rs `resolve_contact_message`
- [x] Empty file content rejected with a clear error ("file ... is empty — refusing to post empty message") — agent.rs `resolve_contact_message`, unit-tested via `resolve_empty_file_errors`
- [x] Existing `--message` callers unchanged: `./target/release/termlink agent contact framework-agent --message "regression check"` reaches the T-1644 pre-T-1436 error as before (smoke-tested live)
- [x] Help text on Contact variant updated — cli.rs:3387-3400 describes mutually-exclusive relationship + references T-1646 + T-1429 Phase-2
- [x] `cargo build -p termlink --release` — Finished in 6m 10s
- [x] `cargo check --workspace` — Finished dev profile in 10.30s (1 pre-existing termlink-mcp warning unrelated)
- [x] `cargo test --bin termlink contact_tests` — **21/21 passed** (15 pre-existing + 6 new resolver tests: `resolve_message_only_returns_message`, `resolve_neither_errors`, `resolve_both_errors`, `resolve_file_reads_contents`, `resolve_empty_file_errors`, `resolve_missing_file_errors`)
- [x] Live smoke test 1 (`--file` alone on pre-T-1436 peer): emits the T-1644 three-path error correctly — proves --file resolved, contact path entered, error fires at the identity_fingerprint check (as expected for framework-agent)
- [x] Live smoke test 2 (`--message` + `--file`): exits with `Error: specify exactly one of --message or --file, not both`
- [x] **Bonus** end-to-end via `--dry-run --file --thread T-1646 --json`: full preview shows message body loaded from file, dm topic computed (`dm:d1993c2c3ec44c94:d1993c2c3ec44c94` self-DM), `metadata._thread=T-1646` + `from_project=010-termlink` correctly stamped. Confirms the whole contact pipeline accepts --file end-to-end.

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

cargo check --workspace 2>&1 | tail -3 | grep -q "Finished\|^$"
cargo build -p termlink --release > /tmp/T-1646-build.log 2>&1 && grep -q "Finished" /tmp/T-1646-build.log
grep -qE "file: Option<(std::path::)?PathBuf>" crates/termlink-cli/src/cli.rs
grep -q "message: Option<String>" crates/termlink-cli/src/cli.rs

# Trailing original template (commented):
# Shell commands that MUST pass before work-completed. One per line.
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

### 2026-05-16T07:33:18Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1646-termlink-agent-contact-add---file-path-f.md
- **Context:** Initial task creation
