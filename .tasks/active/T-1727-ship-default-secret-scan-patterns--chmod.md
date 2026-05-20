---
id: T-1727
name: "Ship default .secret-scan-patterns + chmod +x secret-scan.sh upstream (T-1725 follow-up)"
description: >
  Channel-1 upstream fix: framework ships secret-scan.sh + installer but no default pattern catalogue; result is silently-no-op hooks. Also chmod +x missing.

status: started-work
workflow_type: build
owner: agent
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-20T18:42:17Z
last_update: 2026-05-20T18:44:13Z
date_finished: null
---

# T-1727: Ship default .secret-scan-patterns + chmod +x secret-scan.sh upstream (T-1725 follow-up)

## Context

T-1725 verification surfaced two upstream framework defects:

1. **No default pattern catalogue.** `.agentic-framework/agents/git/lib/secret-scan.sh` ships with `_secret_scan_config_dir` that looks for `.secret-scan-patterns` at project root or under `.agentic-framework/`. Neither exists in any consumer or the framework itself. Result: every consumer's pre-commit hook silently no-ops (`secret-scan: no patterns file (...)`). T-1844 installer is wired correctly but the catalogue side of the contract is missing.
2. **Scanner not executable.** `.agentic-framework/agents/git/lib/secret-scan.sh` ships as `-rw-r--r--`. The pre-commit hook tests `[ -x "$SCANNER" ]` and falls open with `scanner not found at $SCANNER (skipping)` when the bit is missing. T-1666-era housekeeping chmod'd many scripts but missed this one (it's invoked as a sourced subcommand by `git.sh`, not directly by `fw`).

Both defects make the T-1844 secret-scan hook useless out-of-the-box. T-1725 fixed it at the consumer (`/opt/termlink`); this task closes it upstream so every other consumer gets coverage on `fw git install-hooks`.

## Acceptance Criteria

### Agent
- [ ] Add `.agentic-framework/.secret-scan-patterns` to `/opt/999-AEF` (TSV catalogue covering at minimum: github_pat_finegrained, ghp_/gho_/ghu_/ghs_/ghr_, sk-ant-, AKIA, SSH PRIVATE KEY header, xox[baprs]). Copy from `/opt/termlink/.secret-scan-patterns` as the seed.
- [ ] Add `chmod +x agents/git/lib/secret-scan.sh` to `lib/build.sh` (or wherever T-1666 housekeeping lives), AND fix the executable bit on the in-tree file.
- [ ] Verify upstream: clone fresh, run `fw git install-hooks`, attempt commit with `github_pat_` + 82 chars → confirm blocked.
- [ ] Channel-1 dispatch the change from `/opt/termlink` (per `workflow_channel1_upstream_mirror` memory: --workdir, `onedev` not `origin`, verify after).
- [ ] Tighten the "scanner missing" hook message — say "not executable" when the file exists but `! -x`, distinguish from "not found" when the file is absent. Reduces diagnostic loop for the next consumer hitting this.

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

### 2026-05-20T18:42:17Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1727-ship-default-secret-scan-patterns--chmod.md
- **Context:** Initial task creation

### 2026-05-20T18:44:13Z — status-update [task-update-agent]
- **Change:** status: captured → started-work
- **Change:** horizon: next → now (auto-sync)
