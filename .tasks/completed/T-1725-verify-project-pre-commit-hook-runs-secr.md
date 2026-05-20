---
id: T-1725
name: "Verify project pre-commit hook runs secret-scan.sh + covers github_pat_ pattern (T-1695 prevention)"
description: >
  Verify project pre-commit hook runs secret-scan.sh + covers github_pat_ pattern (T-1695 prevention)

status: work-completed
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T10:18:48Z
last_update: 2026-05-20T18:42:49Z
date_finished: 2026-05-20T18:42:49Z
---

# T-1725: Verify project pre-commit hook runs secret-scan.sh + covers github_pat_ pattern (T-1695 prevention)

## Context

The T-1695 leak (commit 15c19f22 added a `github_pat_…` value to `.onedev-buildspec.yml` and merged into main) was eventually removed via destructive history rewrite — but the framework already ships a `secret-scan.sh` at `.agentic-framework/agents/git/lib/secret-scan.sh` whose entire purpose is to catch this class of leak pre-commit. Either the hook isn't installed in this project's `.git/hooks/pre-commit`, OR it is installed but its pattern set doesn't match `github_pat_` (the fine-grained PAT prefix introduced after the patterns were authored).

This task verifies which of the two failed and closes the gap so the next PAT leak attempt is blocked at commit-time.

## Acceptance Criteria

### Agent
- [x] Read `.git/hooks/pre-commit` and confirm whether it invokes `.agentic-framework/agents/git/lib/secret-scan.sh` (or equivalent). If not installed, run `fw git install-hooks` and re-verify
- [x] Read `secret-scan.sh` pattern list; confirm `github_pat_[A-Za-z0-9_]{82}` (or equivalent regex covering the fine-grained PAT prefix) is in the pattern set. If missing, propose patch upstream to the framework repo (`/opt/999-AEF`) via Channel-1 dispatch
- [x] Negative test: stage a file containing `github_pat_TESTTESTTEST...` (82-char fake), attempt `git commit`, observe the hook blocks. Unstage and rm the test file after
- [x] Document outcome in Updates section — installed + covers pattern → close; gap found → file follow-up against framework repo

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

### 2026-05-20T10:18:48Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1725-verify-project-pre-commit-hook-runs-secr.md
- **Context:** Initial task creation

### 2026-05-20T10:19:32Z — status-update [task-update-agent]
- **Change:** status: started-work → captured
- **Change:** horizon: now → next

### 2026-05-20T18:38:31Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)

### 2026-05-20T18:45:00Z — verification + fix
- **AC 1 (hook installed):** Hook was MISSING (`.git/hooks/pre-commit` did not exist). Ran `fw git install-hooks --force` to install. Confirmed: hook now at `/opt/termlink/.git/hooks/pre-commit` (VERSION=1.0, T-1844 lineage), invokes `secret-scan.sh` via `_hits=$("$SCANNER" scan-staged)`.
- **AC 2 (pattern coverage):** Framework ships scanner but NO default pattern catalogue — `_secret_scan_config_dir` looks for `.secret-scan-patterns` at project root or `.agentic-framework/`. Neither existed (verified via `find`). Created `/opt/termlink/.secret-scan-patterns` (TSV, 10 patterns: github_pat_finegrained, ghp_/gho_/ghu_/ghs_/ghr_ classic family, sk-ant- Anthropic, AKIA AWS, SSH PRIVATE KEY header, xox[baprs] Slack).
- **AC 3 (negative test):** Staged `.test-secret-scan.txt` containing `github_pat_` + 82 chars of `A`. Hook blocked with: `ERROR: Commit blocked — secret-scan detected matches: [github_pat_finegrained] .test-secret-scan.txt:1:github_pat_AAA...`. Unstaged + removed test fixture after.
- **AC 4 (outcome):** Gap found at TWO layers — (a) hook not installed in this project (consumer drift), (b) framework ships scanner without default patterns (silently-no-op design). Local fix complete. Upstream framework fix needed: ship a default `.secret-scan-patterns` template under `.agentic-framework/` so consumers get coverage on `fw git install-hooks` without manual catalogue authoring. Filed as T-1727 follow-up against `/opt/999-AEF`.
- **Also surfaced:** `.agentic-framework/agents/git/lib/secret-scan.sh` was not executable (`-rw-r--r--`), which caused the first negative-test attempt to silently pass with `secret-scan: scanner not found at $SCANNER (skipping)`. Ran `chmod +x` to fix locally; root-cause is the `.agentic-framework/lib/build.sh` chmod step (which T-1666 era housekeeping fixed for other scripts) — secret-scan.sh missed that pass. Adding to T-1727 scope.

## Reviewer Verdict (v1.4)

- **Scan ID:** R-e95463bc
- **Timestamp:** 2026-05-20T18:42:50Z
- **Catalogue:** v1.3-seed
- **Overall:** CONCERN
- **Needs Human:** no
- **Findings:** 1

**Per-AC findings:**

- **AC#1 (Agent)** — Read `.git/hooks/pre-commit` and confirm whether it invokes `.agentic-framework/agents/git/lib/secret-scan.sh` (or equivalent). If not installed, run `fw git install-hooks` and re-verify
  - **AC-verify-mismatch** (narrow, heuristic) — `path=agentic-framework/agents/git/lib/secret-scan.sh in: Read `.git/hooks/pre-commit` and confirm whether it invokes `.agentic-framework/agents/git/lib/secret-scan.sh` (or equivalent). If not installed, run `

### 2026-05-20T18:42:49Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
- **Reason:** All 4 Agent ACs verified — hook installed, patterns catalogue created (10 secret-type regexes including github_pat_finegrained), negative test confirmed block, follow-up T-1727 filed for upstream framework fix
