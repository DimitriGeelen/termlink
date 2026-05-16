---
id: T-1654
name: "termlink doctor --fix: auto-chmod 600 on bad-perms secret cache files"
description: >
  termlink doctor --fix: auto-chmod 600 on bad-perms secret cache files

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-16T23:03:38Z
last_update: 2026-05-16T23:03:38Z
date_finished: null
---

# T-1654: termlink doctor --fix: auto-chmod 600 on bad-perms secret cache files

## Context

`termlink doctor` (T-1171) detects bad-perms files under `~/.termlink/secrets/*.hex` and emits a `secret_cache` WARN per file. `termlink doctor --fix` (the existing autoheal flag) already cleans stale sessions and removes stale hub pidfiles, but the secret_cache check just reports — operators must `chmod 600` manually. Chmod 600 on a secret cache file is the only correct mode (it's the canonical T-1055 write mode and matches every healthy file in the directory), so auto-remediation is safe and removes the only manual step in the existing autoheal flow.

Wires the existing `fix: bool` parameter through `audit_secret_cache` so that when `--fix` is on, perms-violations are repaired in-place and the per-issue line reports `fixed:` instead of `warn:`. Drift/divergence issues (the harder cases — operator must confirm whether the cache points at the local hub) remain report-only.

## Acceptance Criteria

### Agent
- [x] `audit_secret_cache` gains a `fix: bool` parameter; when `fix=true` AND a file has non-0o600 perms, `chmod 600` runs on it before recording the issue, and the issue text becomes `"fixed: <path> mode 0o{prev} → 0o600"` instead of the warn text. **Failures of the chmod call itself emit `"<path> has mode {} — chmod failed: <err>"` so operator sees the actual error.**
- [x] Drift/divergence issues (value mismatch + older mtime) are NOT auto-fixed — they remain warn-only since the operator must decide what's authoritative. The fix flag has no effect on those lines. **Test `fix_does_not_chmod_divergence_only_issues` verifies file content + perms unchanged.**
- [x] `cmd_doctor` passes its `fix` parameter through to the audit call. **Was previously hardcoded; now threaded.**
- [x] The check status changes from `warn` to `pass` (using a dedicated `fixed:` prefix) when --fix successfully chmods every bad-perm file. Mixed outcomes (some fixed, some divergence remaining) still warn for the residual divergence. **Call site partitions on `msg.starts_with("fixed:")` to choose pass vs warn class.**
- [x] Unit tests in `infrastructure.rs` test module: 4 new tests added (`fix_chmods_bad_perms_and_reports_fixed_message`, `fix_no_op_on_already_correct_perms`, `fix_does_not_chmod_divergence_only_issues`, `fix_combines_chmod_and_drift_independently`). All 7 existing tests updated to pass `false` for the new parameter (call-site signature change).
- [x] `cargo check --workspace` passes — clean (1 pre-existing unrelated warning in termlink-mcp)
- [x] `cargo test -p termlink --bin termlink -- fix_chmods fix_no_op fix_does_not_chmod fix_combines bad_perms_reported good_perms` passes — **6/6 tests PASS**
- [x] Live verification: built debug binary, staged fake HOME with `~/.termlink/secrets/test.hex` at 0o644. Ran `termlink doctor` (no --fix) → yellow warn line "has mode 644 (expected 600) — world/group-readable cache". Ran `termlink doctor --fix` → green pass line "fixed: /tmp/.../test.hex mode 0o644 → 0o600"; post-stat confirmed mode 600. Cleanup done.

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
cargo test -p termlink --bin termlink -- audit_secret_cache

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

### 2026-05-16T23:03:38Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1654-termlink-doctor---fix-auto-chmod-600-on-.md
- **Context:** Initial task creation
