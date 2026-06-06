---
id: T-1673
name: "Tag v0.10.0 release — land rotation-protocol stack T-1666..T-1672 on operator binaries"
description: >
  Tag v0.10.0 release — land rotation-protocol stack T-1666..T-1672 on operator binaries

status: work-completed
workflow_type: build
owner: human
horizon: now
tags: []
components: []
related_tasks: []
created: 2026-05-17T20:40:29Z
last_update: 2026-05-17T20:42:10Z
date_finished: 2026-05-17T20:42:10Z
---

# T-1673: Tag v0.10.0 release — land rotation-protocol stack T-1666..T-1672 on operator binaries

## Context

2216 commits past v0.9.1. Rotation-protocol stack T-1666..T-1672 (six operator-facing layers: unified single-shot, continuous monitor, event hook, auto-heal recipe, retrospective history, MCP parity) is shipped on main but absent from operators' brew-installed binaries until a v* tag triggers `.github/workflows/release.yml`. Tag scheme: `v0.10.0` (minor bump — substantial new operator-facing feature surface, six new verbs/flags + MCP exposure). Push to onedev only; mirror auto-syncs to GitHub (`.onedev-buildspec.yml::PushRepository` job).

## Acceptance Criteria

### Agent
- [x] Annotated tag `v0.10.0` created on HEAD with message naming T-1666..T-1672
- [x] Tag pushed to onedev origin (NOT GitHub)
- [x] `git tag --sort=-creatordate | head -1` returns `v0.10.0`
- [x] No tag deletion or force-push side effects on prior tags

### Human
- [ ] [REVIEW] Confirm release pipeline produced artifacts
  **Steps:**
  1. After ~10min of post-push: `gh release view v0.10.0 --repo dgaff/termlink 2>&1 | head -30` (or check via web at GitHub releases)
  2. Verify macOS + Linux binaries + checksums attached
  3. `brew upgrade termlink && termlink --version` on an operator host shows `0.10.0`
  **Expected:** Release v0.10.0 visible with binaries; brew picks it up
  **If not:** Inspect GitHub Actions log; rotation may have invalidated OneDev push-token (G-007 territory)

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

## Recommendation

**Recommendation:** GO (close)

**Rationale:** Tag created on HEAD (0e5ff5e1) and successfully pushed to onedev. The OneDev `PushRepository` job auto-mirrors to GitHub, where `.github/workflows/release.yml` triggers on `v*` tags to build macOS + Linux binaries and publish a GitHub Release. The Human REVIEW AC is for post-pipeline verification (~10min latency) and is non-blocking per agent/human split.

**Evidence:**
- `git tag --list v0.10.0` → `v0.10.0` (local)
- `git ls-remote --tags origin v0.10.0` → `3140024312499a0e3c6add68080c58391e72e038 refs/tags/v0.10.0` (remote)
- Verification gate: 2/2 PASS
- Annotated tag message names all seven shipped tasks (T-1666..T-1672)
- No destructive operations performed (no `git tag -d`, no force-push)

## Verification

bash -c "git tag --list v0.10.0 | grep -q '^v0.10.0$'"
bash -c "git ls-remote --tags origin v0.10.0 | grep -q 'refs/tags/v0.10.0$'"

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

### 2026-06-06T15:25Z — Human AC fresh re-smoke for [REVIEW] click [agent autonomous]

Per `[Fresh re-smoke before rubber-stamp]` memory: task is 19 days old; re-ran the deterministic part of the Human AC verbatim via GitHub Releases API:

```
curl -sL "https://api.github.com/repos/DimitriGeelen/termlink/releases/tags/v0.10.0"
  → name: v0.10.0
    published: 2026-05-18T20:36:57Z
    assets: 6
      - checksums.txt
      - termlink-darwin-aarch64
      - termlink-darwin-x86_64
      - termlink-linux-aarch64
      - termlink-linux-x86_64
      - termlink-linux-x86_64-static
```

**PASS:** v0.10.0 release published with macOS + Linux binaries + static + checksums (all 6 expected assets). Operator's Step 3 (`brew upgrade termlink && termlink --version` showing 0.10.0) is the only step requiring local action. Steps 1-2 are PASS based on this evidence.

### 2026-05-17T20:40:29Z — task-created [task-create-agent]
- **Action:** Created task via task-create agent
- **Output:** /opt/termlink/.tasks/active/T-1673-tag-v0100-release--land-rotation-protoco.md
- **Context:** Initial task creation

## Reviewer Verdict (v1.4)

- **Scan ID:** R-fee60c29
- **Timestamp:** 2026-05-17T20:42:10Z
- **Catalogue:** v1.3-seed
- **Overall:** PASS
- **Needs Human:** yes
- **Findings:** none

- **Layer-1 escalations:** 1
  1. **destructive-action** (high) — Destructive operation in verification or AC
     - matched: `force-push`

### 2026-05-17T20:42:10Z — status-update [task-update-agent]
- **Change:** status: started-work → work-completed
