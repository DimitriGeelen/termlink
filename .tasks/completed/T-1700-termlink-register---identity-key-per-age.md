---
id: T-1700
name: "termlink register --identity-key per-agent path (T-1693 Shape 1)"
description: >
  termlink register --identity-key per-agent path (T-1693 Shape 1)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: [identity, T-1693, T-1159, per-agent-keys]
components: [crates/termlink-cli/src/cli.rs, crates/termlink-cli/src/commands/channel.rs, crates/termlink-cli/src/commands/session.rs, crates/termlink-cli/src/main.rs, crates/termlink-session/src/agent_identity.rs, crates/termlink-session/src/registration.rs]
related_tasks: [T-1693, T-1159, T-1436, T-1427]
created: 2026-05-18T22:11:34Z
last_update: 2026-05-18T22:21:38Z
date_finished: 2026-05-18T22:21:38Z
---

# T-1700: termlink register --identity-key per-agent path (T-1693 Shape 1)

## Context

T-1693 inception decided **GO** for per-agent ed25519 signing identity on shared
hosts, Shape 1 (agent-managed key files, per-project secrets dir, passed via
`--identity-key`). T-1159 already shipped the keypair infrastructure
(`Identity::load_or_create(&base)` reads/creates `<base>/identity.key` at chmod
600); T-1436 wired the resulting fingerprint into `SessionMetadata`. The gap
this task closes: `termlink register` currently has no way to load identity
from a non-default path, so every agent on a host shares
`$HOME/.termlink/identity.key`. PL-166 (envelope `from_fingerprint` identifies
host, not agent) is structurally unanswerable until a per-agent key can be
selected at session-start time.

Scope: add `--identity-key <PATH>` (file path) to `termlink register`. When
set, the identity is loaded/created at that file (parent dir as base via the
existing `load_or_create_from_file` API added in this task), chmod 600 on
create. When unset, behavior unchanged. The fingerprint exposed via
`SessionMetadata.identity_fingerprint` (T-1436) reflects the override.
Out of scope (separate follow-up tasks per "one task = one deliverable"):
env-var equivalent, rotation tooling, docs convention writeup.

## Acceptance Criteria

### Agent
- [x] `Identity::load_or_create_from_file(path)` added to `agent_identity.rs:64-77` — explicit file path, parent dir auto-created, chmod 600, same atomic-write path as `load_or_create`
- [x] `termlink register --identity-key <PATH>` flag added (`cli.rs:60-67`) — propagated through `RegisterOpts.identity_key` (`session.rs:68-71`) to `cmd_register` (`session.rs:75-103`), which loads/creates the key and exports `TERMLINK_IDENTITY_FILE` before `Session::register` so registration metadata + signing both pick it up
- [x] When `--identity-key` is set, registered session's `metadata.identity_fingerprint` matches override — smoke-tested: host default `d1993c2c3ec44c94`, per-agent override `13bc46a23bf4d18f` (2026-05-18T22:19Z)
- [x] When `--identity-key` unset, behavior unchanged — `load_identity_fingerprint_best_effort` falls back to `TERMLINK_IDENTITY_DIR` then `$HOME/.termlink/identity.key` (registration.rs:28-65); `resolve_identity_key_path_falls_back_to_home` test asserts the default branch
- [x] Two co-resident `--identity-key <a>` / `--identity-key <b>` produce DISTINCT fingerprints — covered by `load_or_create_from_file_two_paths_produce_distinct_identities` unit test + smoke above
- [x] `cargo test -p termlink-session --lib agent_identity resolve_identity` → 13 passed / 0 failed (2 new file-path tests, 4 new precedence tests). The unrelated `client::tests::connect_addr_with_timeout_errors_on_unreachable` failure is pre-existing on main (verified via git stash)
- [x] `cargo check --workspace` clean (only 1 pre-existing unrelated warning in termlink-mcp)
- [x] `cargo clippy -p termlink-session --lib -- -D warnings` clean; my files in termlink-cli have zero new clippy hits (termlink-mcp has 23 pre-existing errors unrelated to this task)

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

cargo check --workspace
cargo test -p termlink-session --lib agent_identity
cargo test -p termlink-session --lib registration::tests::resolve_identity_key_path
grep -q "identity_key" crates/termlink-cli/src/commands/session.rs
grep -q "load_or_create_from_file" crates/termlink-session/src/agent_identity.rs
grep -q "identity_key: Option" crates/termlink-cli/src/cli.rs

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

### 2026-05-18T22:11:34Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1700-termlink-register---identity-key-per-age.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-378d6aca
- **Timestamp:** 2026-05-18T22:21:39Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#4 (Agent)** — When `--identity-key` unset, behavior unchanged — `load_identity_fingerprint_best_effort` falls back to `TERMLINK_IDENTITY_DIR` then `$HOME/.termlink/identity.key` (registration.rs:28-65); `resolve_id
  - **AC-verify-mismatch** (narrow, heuristic) — `path=HOME/.termlink/identity.key in: When `--identity-key` unset, behavior unchanged — `load_identity_fingerprint_best_effort` falls back to `TERMLINK_IDENTITY_DIR` then `$HOME/.termlink/`

### 2026-05-18T22:21:38Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
