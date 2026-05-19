---
id: T-1715
name: "termlink_agent_contact MCP — parity with CLI (T-1429 phase-2 shipped)"
description: >
  termlink_agent_contact MCP — parity with CLI (T-1429 phase-2 shipped)

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-19T22:17:03Z
last_update: 2026-05-19T22:17:03Z
date_finished: null
---

# T-1715: termlink_agent_contact MCP — parity with CLI (T-1429 phase-2 shipped)

## Context

T-1429 shipped `termlink agent contact` CLI verb with Phase-1 + Phase-2 partial: --message / --file / --thread / --json / --dry-run / --target-fp / --require-online / --ack-required. MCP-side `termlink_agent_contact` does not exist. MCP-aware agents (Claude Code, ntb-atc-plugin, etc.) wanting to do peer-to-peer contact have to either shell out to the CLI or improvise via the lower-level `termlink_channel_post` (which requires the caller to compute the dm:<a>:<b> topic themselves + know the peer's identity_fingerprint). This task closes the parity gap with a v1 scoped to core features; v2 extension for --require-online / --ack-required deferred to a follow-up task because those need extra MCP-side wiring for the presence-probe / ack-poll loops.

## Acceptance Criteria

### Agent
- [x] `termlink_agent_contact` MCP tool exists in `crates/termlink-mcp/src/tools.rs` — registered via `#[tool(name = "termlink_agent_contact", description = "...")]` attribute (tools.rs:8648-8718 region)
- [x] `AgentContactParams` struct defined with: `target` (Option<String>), `target_fp` (Option<String>), `message` (String), `thread` (Option<String>), `dry_run` (Option<bool>), `sender_id` (Option<String>) — tools.rs:1060-1093
- [x] Mutex validation: exactly one of `target` / `target_fp` is required — both or neither → JSON error response with clear message (handled in the early match block on `(&p.target, &p.target_fp)`)
- [x] `target` resolution path: calls `manager::find_session(name)` and reads `reg.metadata.identity_fingerprint`; pre-T-1436 peer (no identity_fingerprint) → JSON error with three recovery hints matching CLI message (upgrade peer / target_fp / mention via channel_post)
- [x] `target_fp` direct path: validates hex (len ≥ 8, all ascii_hexdigit); rejects invalid input with JSON error
- [x] `:project` suffix on `target` parsed via `split_target_project` helper (tools.rs:248-279); resolves to `metadata.to_project=<project>` on the dm post (T-1448 (b))
- [x] `dm:<sorted_a>:<sorted_b>` topic computed via `dm_topic_canonical` helper (tools.rs:281-291) — matches CLI canonical computation (`commands::channel::dm_topic` semantics, verified via `agent_contact_dm_topic_is_sorted_canonical` test asserting symmetric output regardless of caller order)
- [x] Topic auto-create with `retention=forever` via `channel.create` RPC — idempotent, matches CLI Phase-1 behavior (live path only; dry-run skips)
- [x] `--dry-run` returns preview JSON `{ok, dry_run: true, topic, peer_fp, my_fp, metadata, would_post: {msg_type, body_preview, body_bytes}}` without posting; no hub round-trip beyond identity load
- [x] Live post path: signs envelope with local ed25519 identity (mirror of `termlink_agent_post`), posts via `channel.post` RPC with `msg_type=chat`, returns decorated JSON envelope `{ok, topic, peer_fp, my_fp, ...hub_post_result}`
- [x] `cargo build --release -p termlink-mcp` clean — completed in 1m 20s with only the pre-existing `cur_run_end` warning (unrelated to this work)
- [x] **8** unit tests for `AgentContactParams` + helpers — exceeded the 4-test target. Coverage: `split_target_project` (3 cases — bare / with-project / empty-rejects), `dm_topic_canonical` (1 — ordering + symmetry), `preview_body` (2 — short-passthrough + truncates-with-ellipsis), `AgentContactParams` deserialize (2 — minimal target_fp + full target+optionals). All 8 pass.
- [x] Tool count bumped — `target/release/termlink doctor` now shows `174 MCP tools` (was 173 pre-T-1715, +1 as expected)

### Human
<!-- All Agent ACs are mechanically verifiable. No human REVIEW needed (per G-059 / PL-169 — don't over-defer mechanical criteria). -->
- [ ] [REVIEW] Smoke-test MCP tool from a real MCP-aware session (e.g. Claude Code in this repo)
  **Steps:**
  1. Open a Claude Code session in /opt/termlink and verify `termlink_agent_contact` appears in the MCP tool list (use `/mcp` or check the available-tools listing)
  2. Invoke with a fresh test peer (or `target_fp` of own identity for self-dm test) and `dry_run: true` — observe the JSON preview matches CLI `--dry-run` shape
  3. Invoke without `dry_run` against a real local session — observe the dm topic is created and the post lands at expected offset
  **Expected:** tool surfaces in MCP listing, dry-run preview matches CLI, live post lands on dm:* topic
  **If not:** capture the failing path + JSON response in this task's Updates and re-scope.

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
cargo build --release -p termlink-mcp
cargo test --release -p termlink-mcp agent_contact

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

### 2026-05-19T22:17:03Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1715-termlinkagentcontact-mcp--parity-with-cl.md
- **Context:** Initial task creation
