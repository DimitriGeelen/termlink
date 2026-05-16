---
id: T-1644
name: "termlink agent contact: surface agent-chat-arc fallback in pre-T-1436-peer error message"
description: >
  termlink agent contact: surface agent-chat-arc fallback in pre-T-1436-peer error message

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-16T06:59:41Z
last_update: 2026-05-16T06:59:41Z
date_finished: null
---

# T-1644: termlink agent contact: surface agent-chat-arc fallback in pre-T-1436-peer error message

## Context

`termlink agent contact <peer>` refuses with "no identity_fingerprint in metadata — likely registered before T-1436" when the peer's session was registered before T-1436 shipped the metadata field. The error currently names only two recovery paths: (1) upgrade the peer's termlink binary + restart its session, or (2) pass `--target-fp <hex>` (which requires already knowing the peer's fingerprint).

There is a third practical fallback that works without restarting the peer and without knowing the fingerprint: post to a public broadcast topic with `--mention <peer>` metadata, which agent-chat-arc subscribers (the documented canonical pattern, T-1430) pick up by name regardless of identity_fingerprint state. T-1643 (2026-05-16, posted at offset 1471) used exactly this fallback when `termlink agent contact framework-agent` refused on a 12d-old session.

This task adds that fallback to the error message at both error sites in `crates/termlink-cli/src/commands/agent.rs` (lines 786-803 and 1051-1071) and updates the function-level doc comments (lines 707-708 and 1034) to match. Pure DX/UX fix — message text only, no behavior change.

## Acceptance Criteria

### Agent
- [x] Error message at agent.rs:788-797 (`cmd_agent_contact` resolution path) lists three options: (1) upgrade peer binary, (2) `--target-fp <hex>`, (3) `termlink channel post agent-chat-arc --mention <peer> --metadata _thread=<task-id>` fallback — verified in source + `strings target/release/termlink` confirms both new strings landed in the binary
- [x] Error message at agent.rs:1054-1063 (`resolve_target_name_to_fp` helper — the actual fn name is this, not `resolve_peer_fp`) lists the same three options
- [x] Function doc comments at agent.rs:707-710 + agent.rs:1034-1043 reflect the updated error text shape and reference T-1644
- [x] `cargo check --workspace` passes — Finished dev profile in 8.12s (1 pre-existing warning in termlink-mcp unrelated to this change)
- [x] `cargo build -p termlink --release` succeeds — Finished release profile in 5m 53s. (Package name is `termlink`, not `termlink-cli`; corrected in verification commands below)
- [x] `cargo test --bin termlink contact_tests` passes — **15/15 passed**, 703 other tests filtered out. No tests assert error message text so no test changes needed
- [x] No new clippy warnings on agent.rs — `grep "agent\.rs" /tmp/T-1644-clippy.log` returns 0 hits. (The 23 clippy errors that DO fire are all in `termlink-mcp/src/tools.rs` — pre-existing, unrelated to T-1644, see PL-XXX backlog for that crate's clippy debt)

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
cargo build -p termlink --release > /tmp/T-1644-build.log 2>&1 && grep -q "Finished" /tmp/T-1644-build.log
grep -q "agent-chat-arc" crates/termlink-cli/src/commands/agent.rs
grep -c "mention" crates/termlink-cli/src/commands/agent.rs | awk '{ exit ($1 >= 2) ? 0 : 1 }'

# Original verification template below (commented out)
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

### 2026-05-16T06:59:41Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1644-termlink-agent-contact-surface-agent-cha.md
- **Context:** Initial task creation
