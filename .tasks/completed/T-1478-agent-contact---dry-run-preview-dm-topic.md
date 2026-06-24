---
id: T-1478
name: "agent contact --dry-run: preview dm topic + metadata without posting"
description: >
  agent contact --dry-run: preview dm topic + metadata without posting

status: work-completed
workflow_type: build
owner: agent
horizon: null
tags: []
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/agent.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/main.rs]
related_tasks: []
created: 2026-05-04T11:15:12Z
last_update: 2026-05-04T11:18:25Z
date_finished: 2026-05-04T11:18:25Z
---

# T-1478: agent contact --dry-run: preview dm topic + metadata without posting

## Context

`agent contact <name>[:project] --message <m>` resolves the target's
identity_fingerprint via session.discover, computes the canonical
`dm:<sorted_a>:<sorted_b>` topic, and posts the message. There's no way
to verify the resolution result before the message lands — operators
who want to confirm "is the target registered? does the project qualifier
land in to_project? what topic will this hit?" have to either send a
test message and read it back, or invoke `channel dm --topic-only`
manually with the FP, losing the name-resolution validation.

Add `--dry-run` (mutually-exclusive with the actual post). When set:
- Resolve target → FP exactly as the live path
- Compute the dm topic (canonical sorted form)
- Build the metadata block (from_project auto-injected, to_project from
  qualifier, _thread from --thread)
- Print the preview JSON (always JSON, since this is for scripting)
- Exit 0 without contacting the hub for the post

## Acceptance Criteria

### Agent
- [x] `termlink agent contact <target> --message <m> --dry-run` resolves the FP exactly as the live path and prints a JSON preview to stdout (verified)
- [x] Preview shape: `{"dry_run": true, "peer_fp": "<hex>", "topic": "dm:<a>:<b>", "metadata": {<map>}, "message": "<m>", "my_id": "<hex>"}` (verified — added my_id for completeness)
- [x] `--dry-run` is mutually exclusive with the live post — function returns Ok before reaching cmd_channel_dm (verified: dm topic offset unchanged after dry-run, still 8)
- [x] Works with `--target-fp d1993c2c3ec44c94 --dry-run` — verified, peer_fp and topic computed correctly
- [x] Works with `<name>:<project>` — verified, `to_project=T-1478` in metadata
- [x] Works with `--thread <id>` — verified, `_thread=T-1478` in metadata
- [x] Target-resolution errors fire BEFORE dry-run output — code path is the same `find_session` chain, errors short-circuit before the dry_run if-block
- [x] CLI doc-comment for `--dry-run` explains use case (CI / ops verification before commit)
- [x] Unit test for `render_dry_run_preview` in contact_tests (3 tests: basic_shape, no_extras_yields_empty_metadata_object, skips_malformed_extras_without_eq) — all pass
- [x] `cargo build -p termlink` succeeds (9.16s); `cargo test -p termlink --bin termlink contact_tests` passes (11 passed; 0 failed)
- [x] Smoke verified — preview JSON contains topic=dm:d199...:d199..., metadata.from_project=010-termlink, metadata.to_project=T-1478, metadata._thread=T-1478; dm topic Posts count unchanged at 8

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

cargo build -p termlink
cargo test -p termlink --bin termlink contact_tests > /tmp/t1478-test.txt 2>&1; grep -q "0 failed" /tmp/t1478-test.txt
target/debug/termlink agent contact termlink-agent:T-1478 --message preview --thread T-1478 --dry-run > /tmp/t1478-dr.json 2>&1; python3 -c "import json; d=json.load(open('/tmp/t1478-dr.json')); assert d.get('dry_run') is True and d.get('topic','').startswith('dm:') and d.get('metadata',{}).get('to_project')=='T-1478' and d.get('metadata',{}).get('_thread')=='T-1478' and d.get('metadata',{}).get('from_project')=='010-termlink', d"
# (no need to keep the preview file as it's a flat JSON object that fits inline)
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

### 2026-05-04T11:15:12Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1478-agent-contact---dry-run-preview-dm-topic.md
- **Context:** Initial task creation

### 2026-05-04T11:18:25Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
